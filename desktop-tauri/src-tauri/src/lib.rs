use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;

// ── State ────────────────────────────────────────────────────────────

struct BackendState {
    port: Mutex<Option<u16>>,
}

// Held as Option so we can take the child on cleanup to kill it.
struct BackendChild {
    child: Mutex<Option<tauri_plugin_shell::process::CommandChild>>,
}

// ── Commands ─────────────────────────────────────────────────────────

#[tauri::command]
fn minimize_window(window: tauri::Window) {
    let _ = window.minimize();
}

#[tauri::command]
fn maximize_window(window: tauri::Window) {
    if window.is_maximized().unwrap_or(false) {
        let _ = window.unmaximize();
    } else {
        let _ = window.maximize();
    }
}

#[tauri::command]
fn close_window(window: tauri::Window) {
    let _ = window.close();
}

#[tauri::command]
async fn get_window_state(window: tauri::Window) -> String {
    if window.is_maximized().unwrap_or(false) {
        "maximized".to_string()
    } else {
        "restored".to_string()
    }
}

#[tauri::command]
fn get_backend_port(state: tauri::State<BackendState>) -> u16 {
    state.port.lock().unwrap().unwrap_or(0)
}

// ── Encryption key ───────────────────────────────────────────────────

/// Generate or load a 64-hex-char encryption key from the app data dir.
/// Mirrors the Electron `getOrCreateEncryptionKey` logic.
fn get_or_create_encryption_key(app_data_dir: &std::path::Path) -> String {
    let key_path = app_data_dir.join(".encryption_key");

    // Try loading existing key
    if let Ok(existing) = std::fs::read_to_string(&key_path) {
        let trimmed = existing.trim().to_string();
        if trimmed.len() == 64 {
            return trimmed;
        }
    }

    // Generate a new 32-byte random key → 64 hex chars
    let bytes = rand_bytes(32);
    let key: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
    let _ = std::fs::write(&key_path, &key);
    key
}

fn rand_bytes(len: usize) -> Vec<u8> {
    let mut buf = vec![0u8; len];
    getrandom::getrandom(&mut buf).expect("failed to generate random bytes");
    buf
}

// ── Backend spawn ────────────────────────────────────────────────────

fn start_backend<R: Runtime>(app: &AppHandle<R>) {
    let shell = app.shell();

    let app_data_dir = app
        .path()
        .app_data_dir()
        .expect("failed to get app data dir");
    std::fs::create_dir_all(&app_data_dir).expect("failed to create app data dir");

    let db_path = app_data_dir.join("webterm.db");
    let db_path_str = db_path.to_string_lossy().to_string();
    let encryption_key = get_or_create_encryption_key(&app_data_dir);

    let sidecar_command = shell
        .sidecar("webterm-backend")
        .expect("failed to create sidecar command")
        .env("WEBTERM_PORT", ":0") // Request dynamic port
        .env("WEBTERM_DB_PATH", &db_path_str)
        .env("WEBTERM_ALLOWED_ORIGINS", "*")
        .env("WEBTERM_ENCRYPTION_KEY", &encryption_key);

    let (mut rx, child) = sidecar_command
        .spawn()
        .expect("failed to spawn sidecar");

    // Store child handle so we can kill it on quit
    let backend_child = app.state::<BackendChild>();
    *backend_child.child.lock().unwrap() = Some(child);

    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(line) => {
                    let text = String::from_utf8_lossy(&line);
                    println!("Backend: {}", text);
                    if text.contains("BACKEND_PORT:") {
                        if let Some(port_str) = text.split("BACKEND_PORT:").last() {
                            if let Ok(port) = port_str.trim().parse::<u16>() {
                                let state = app_handle.state::<BackendState>();
                                *state.port.lock().unwrap() = Some(port);
                                let _ = app_handle.emit("backend-ready", port);
                            }
                        }
                    }
                }
                CommandEvent::Stderr(line) => {
                    eprintln!("Backend Error: {}", String::from_utf8_lossy(&line));
                }
                CommandEvent::Terminated(status) => {
                    println!("Backend exited with status: {:?}", status);
                }
                CommandEvent::Error(err) => {
                    eprintln!("Backend command error: {}", err);
                }
                _ => {}
            }
        }
    });
}

// ── Entry point ──────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(BackendState {
            port: Mutex::new(None),
        })
        .manage(BackendChild {
            child: Mutex::new(None),
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_log::Builder::default().build())
        .setup(|app| {
            start_backend(app.handle());

            let window = app.get_webview_window("main").unwrap();

            let window_clone = window.clone();
            window.on_window_event(move |event| match event {
                tauri::WindowEvent::Resized(_) | tauri::WindowEvent::Moved(_) => {
                    let w = window_clone.clone();
                    tauri::async_runtime::spawn(async move {
                        // Small delay to let OS update window properties
                        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                        let state = if w.is_maximized().unwrap_or(false) {
                            "maximized"
                        } else {
                            "restored"
                        };
                        let _ = w.emit("window-state-change", state);
                    });
                }
                _ => {}
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                // Kill the backend process on window close
                let child_state = window.try_state::<BackendChild>();
                if let Some(backend) = child_state {
                    if let Some(child) = backend.child.lock().unwrap().take() {
                        let _ = child.kill();
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            minimize_window,
            maximize_window,
            close_window,
            get_window_state,
            get_backend_port
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
