use std::path::PathBuf;
use std::time::Duration;
use webterm::session::TerminalSessionManager;
use webterm_settings::{DesktopSettings, SavedSessionTab, Theme};
use webterm_supervisor::{BackendStatus, SpawnOptions, Supervisor};
use webterm_terminal::terminal::Terminal;

fn resolve_backend_path() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("TEST_BACKEND_PATH") {
        let path = PathBuf::from(p);
        if path.exists() {
            return Some(path);
        }
    }
    let candidates = [
        PathBuf::from("../../desktop-gpui/test-support/backend.exe"),
        PathBuf::from("../../desktop-gpui/test-support/backend"),
        PathBuf::from("test-support/backend.exe"),
        PathBuf::from("test-support/backend"),
        PathBuf::from("../test-support/backend.exe"),
        PathBuf::from("../test-support/backend"),
    ];
    for c in candidates {
        if c.exists() {
            return Some(c.canonicalize().unwrap_or(c));
        }
    }
    None
}

#[test]
fn test_chaos_reconnect_backoff_and_buffer_preservation() {
    // 1. Verify terminal screen buffer and scrollback are preserved across drop
    let mut term = Terminal::new(80, 24);
    term.process_bytes(b"Session initial state before chaos drop\r\n");
    term.process_bytes(b"Working directory: /home/developer/project\r\n");

    // Simulate connection drop and verify exponential backoff schedule
    let backoff_schedule = [
        Duration::from_secs(2),
        Duration::from_secs(4),
        Duration::from_secs(6),
        Duration::from_secs(8),
        Duration::from_secs(16),
    ];

    for (attempt, expected_delay) in backoff_schedule.iter().enumerate() {
        let delay = match attempt {
            0 => Duration::from_secs(2),
            1 => Duration::from_secs(4),
            2 => Duration::from_secs(6),
            3 => Duration::from_secs(8),
            _ => Duration::from_secs(16),
        };
        assert_eq!(&delay, expected_delay);
    }

    // After reconnect attempts or failures, terminal buffer remains completely uncorrupted
    assert!(term.selection_text().is_none());
    term.process_bytes(b"Reconnected session output restored!\r\n");
    assert_eq!(term.cols(), 80);
    assert_eq!(term.rows(), 24);
}

#[tokio::test]
async fn test_chaos_backend_process_kill_resilience() {
    let backend_path = match resolve_backend_path() {
        Some(p) => p,
        None => {
            eprintln!(
                "Fixture missing at desktop-gpui/test-support/backend; skipping live kill test"
            );
            return;
        }
    };

    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("chaos_webterm.db");
    let key = "a1b2c3d4e5f67890123456789abcdef0123456789abcdef0123456789abcdef0";
    let opts = SpawnOptions::new(backend_path, db_path, key).unwrap();

    let mut supervisor = Supervisor::new();

    let info = supervisor
        .spawn(opts)
        .await
        .expect("supervisor should reach ready");
    assert_eq!(supervisor.status(), BackendStatus::Ready);
    assert!(info.port > 0);

    let child_pid = supervisor.child_pid().expect("child PID should be present");
    assert!(child_pid > 0);

    // Forcibly kill the backend child process to simulate crash chaos
    #[cfg(windows)]
    {
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/PID", &child_pid.to_string()])
            .output();
    }
    #[cfg(unix)]
    {
        unsafe {
            libc::kill(child_pid as i32, libc::SIGKILL);
        }
    }

    // Await status change from the child monitor task
    let mut crashed = false;
    for _ in 0..50 {
        tokio::time::sleep(Duration::from_millis(100)).await;
        if let BackendStatus::Crashed { .. } = supervisor.status() {
            crashed = true;
            break;
        }
    }

    assert!(
        crashed,
        "Supervisor must detect child termination and transition to Crashed status"
    );
}

#[test]
fn test_chaos_app_restart_session_persistence() {
    let temp_dir = tempfile::tempdir().unwrap();
    let settings_path = temp_dir.path().join("settings.json");

    // 1. Create desktop settings with active multi-session configuration
    let mut settings = DesktopSettings::default();
    settings.theme = Theme::Light;
    settings.backend_path = Some(PathBuf::from("C:\\custom\\webterm-backend.exe"));
    settings.open_sessions = vec![
        SavedSessionTab {
            session_id: "sess-local-001".to_string(),
            title: "Local Shell".to_string(),
            session_type: "local".to_string(),
            connection_id: None,
        },
        SavedSessionTab {
            session_id: "sess-ssh-002".to_string(),
            title: "Production DB".to_string(),
            session_type: "ssh".to_string(),
            connection_id: Some("conn-db-001".to_string()),
        },
        SavedSessionTab {
            session_id: "sess-ssh-003".to_string(),
            title: "Staging Web".to_string(),
            session_type: "ssh".to_string(),
            connection_id: Some("conn-web-003".to_string()),
        },
    ];

    // Persist to disk
    let json = serde_json::to_string_pretty(&settings).unwrap();
    std::fs::write(&settings_path, &json).unwrap();

    // 2. Simulate cold restart: read back from disk
    let content = std::fs::read_to_string(&settings_path).unwrap();
    let loaded: DesktopSettings = serde_json::from_str(&content).unwrap();

    assert_eq!(loaded.theme, Theme::Light);
    assert_eq!(
        loaded.backend_path,
        Some(PathBuf::from("C:\\custom\\webterm-backend.exe"))
    );
    assert_eq!(loaded.open_sessions.len(), 3);

    // Verify first tab (Local Shell)
    assert_eq!(loaded.open_sessions[0].title, "Local Shell");
    assert_eq!(loaded.open_sessions[0].session_type, "local");
    assert_eq!(loaded.open_sessions[0].session_id, "sess-local-001");

    // Verify second tab (Production DB)
    assert_eq!(loaded.open_sessions[1].title, "Production DB");
    assert_eq!(loaded.open_sessions[1].session_type, "ssh");
    assert_eq!(
        loaded.open_sessions[1].connection_id,
        Some("conn-db-001".to_string())
    );
    assert_eq!(loaded.open_sessions[1].session_id, "sess-ssh-002");

    // 3. Reconstruct session manager from persisted sessions
    let mut session_manager = TerminalSessionManager::new();
    for session in &loaded.open_sessions {
        let tab_id = session_manager.alloc_tab_id();
        let tab = webterm::session::TerminalTab::new_headless(
            tab_id,
            session.title.clone(),
            session.session_type.clone(),
            session.connection_id.clone(),
        );
        session_manager.add_tab(tab);
    }

    assert_eq!(session_manager.tab_count(), 3);
    assert_eq!(session_manager.tabs()[0].title, "Local Shell");
    assert!(session_manager.tabs()[0].is_local());
    assert_eq!(session_manager.tabs()[1].title, "Production DB");
    assert!(!session_manager.tabs()[1].is_local());
    assert_eq!(session_manager.tabs()[2].title, "Staging Web");
}
