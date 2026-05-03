use tauri::{AppHandle, Manager, Runtime, Emitter};
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;

#[tauri::command]
fn minimize_window(window: tauri::Window) {
    window.minimize().unwrap();
}

#[tauri::command]
fn maximize_window(window: tauri::Window) {
    if window.is_maximized().unwrap() {
        window.unmaximize().unwrap();
    } else {
        window.maximize().unwrap();
    }
}

#[tauri::command]
fn close_window(window: tauri::Window) {
    window.close().unwrap();
}

#[tauri::command]
async fn get_window_state(window: tauri::Window) -> String {
    if window.is_maximized().unwrap() {
        "maximized".to_string()
    } else {
        "restored".to_string()
    }
}

use std::sync::Mutex;

struct BackendState {
    port: Mutex<Option<u16>>,
}

#[tauri::command]
fn get_backend_port(state: tauri::State<BackendState>) -> u16 {
    state.port.lock().unwrap().unwrap_or(0)
}

fn start_backend<R: Runtime>(app: &AppHandle<R>) {
    let shell = app.shell();
    
    let app_data_dir = app.path().app_data_dir().expect("failed to get app data dir");
    std::fs::create_dir_all(&app_data_dir).expect("failed to create app data dir");
    let db_path = app_data_dir.join("webterm.db");
    let db_path_str = db_path.to_string_lossy().to_string();

    let sidecar_command = shell.sidecar("webterm-backend").unwrap()
        .env("WEBTERM_PORT", ":0") // Request dynamic port
        .env("WEBTERM_DB_PATH", &db_path_str)
        .env("WEBTERM_ALLOWED_ORIGINS", "*");

    let (mut rx, _child) = sidecar_command.spawn().expect("failed to spawn sidecar");

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
                                app_handle.emit("backend-ready", port).unwrap();
                            }
                        }
                    }
                }
                CommandEvent::Stderr(line) => {
                    eprintln!("Backend Error: {}", String::from_utf8_lossy(&line));
                }
                _ => {}
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(BackendState { port: Mutex::new(None) })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_log::Builder::default().build())
        .setup(|app| {
            start_backend(app.handle());
            
            let window = app.get_webview_window("main").unwrap();
            
            let window_clone = window.clone();
            window.on_window_event(move |event| {
                match event {
                    tauri::WindowEvent::Resized(_) | 
                    tauri::WindowEvent::Moved(_) => {
                        let w = window_clone.clone();
                        tauri::async_runtime::spawn(async move {
                            // Small delay to let OS update window properties
                            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                            let state = if w.is_maximized().unwrap() {
                                "maximized"
                            } else {
                                "restored"
                            };
                            w.emit("window-state-change", state).unwrap();
                        });
                    }
                    _ => {}
                }
            });

            Ok(())
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
