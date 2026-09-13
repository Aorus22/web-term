//! End-to-end smoke test for cross-platform local terminal (ConPTY on Windows, POSIX PTY on Linux)
//! exercising real backend supervisor spawning, WebSocket multiplexing, terminal I/O, and resize.

use std::path::PathBuf;
use std::time::Duration;
use tokio::time::timeout;
use webterm_backend_client::{normalize_ws_url, TerminalWsClient, WsConnectRequest};
use webterm_supervisor::{BackendStatus, SpawnOptions, Supervisor};

fn resolve_backend_path() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("TEST_BACKEND_PATH") {
        let path = PathBuf::from(p);
        if path.exists() {
            return Some(path);
        }
    }
    let candidates = [
        PathBuf::from("../../test-support/backend.exe"),
        PathBuf::from("../../test-support/backend"),
        PathBuf::from("desktop-gpui/test-support/backend.exe"),
        PathBuf::from("desktop-gpui/test-support/backend"),
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

#[tokio::test]
async fn test_local_terminal_e2e_smoke() {
    let backend_path = match resolve_backend_path() {
        Some(p) => p,
        None => {
            eprintln!("Fixture missing at desktop-gpui/test-support/backend: build via scripts/build-test-backend");
            return;
        }
    };

    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("webterm.db");
    let key = "a1b2c3d4e5f67890123456789abcdef0123456789abcdef0123456789abcdef0";
    let opts = SpawnOptions::new(backend_path, db_path, key).unwrap();

    let mut supervisor = Supervisor::new();
    let info = supervisor
        .spawn(opts)
        .await
        .expect("supervisor should reach ready");
    assert_eq!(supervisor.status(), BackendStatus::Ready);

    let ws_url = normalize_ws_url(&info.base_url);

    // 1. Connect to local shell via WebSocket
    let req = WsConnectRequest::for_local(80, 24);
    let handle = TerminalWsClient::connect(&ws_url, req)
        .await
        .expect("TerminalWsClient should connect to local shell");

    assert!(handle.is_connected());
    assert!(!handle.session_id().is_empty());

    // 2. Wait for initial shell prompt / send echo command
    let rx = handle.output_receiver();

    // Send test echo command
    let cmd: &[u8] = if cfg!(windows) {
        b"cmd.exe /c echo CONPTY_SMOKE_OK\r\n"
    } else {
        b"echo CONPTY_SMOKE_OK\n"
    };
    handle
        .send_input(cmd.to_vec())
        .expect("send_input should succeed");

    // 3. Read output until token is found or timeout expires
    let mut collected = Vec::new();
    let mut found_marker = false;

    let read_result = timeout(Duration::from_secs(10), async {
        while let Ok(chunk) = rx.recv_async().await {
            collected.extend_from_slice(&chunk);
            let text = String::from_utf8_lossy(&collected);
            if text.contains("CONPTY_SMOKE_OK") {
                found_marker = true;
                break;
            }
        }
    })
    .await;

    assert!(
        read_result.is_ok() && found_marker,
        "Expected output containing CONPTY_SMOKE_OK, got: {}",
        String::from_utf8_lossy(&collected)
    );

    // 4. Test terminal resize
    handle.resize(120, 40).expect("resize should succeed");

    // 5. Clean disconnect
    let _ = handle.disconnect();

    // 6. Stop backend supervisor
    supervisor
        .stop(Duration::from_secs(5))
        .await
        .expect("supervisor should stop");
}

#[tokio::test]
async fn test_local_terminal_e2e_with_custom_cwd() {
    let backend_path = match resolve_backend_path() {
        Some(p) => p,
        None => {
            eprintln!("Fixture missing at desktop-gpui/test-support/backend: build via scripts/build-test-backend");
            return;
        }
    };

    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("webterm.db");
    let key = "a1b2c3d4e5f67890123456789abcdef0123456789abcdef0123456789abcdef0";
    let opts = SpawnOptions::new(backend_path, db_path, key).unwrap();

    let mut supervisor = Supervisor::new();
    let info = supervisor
        .spawn(opts)
        .await
        .expect("supervisor should reach ready");
    assert_eq!(supervisor.status(), BackendStatus::Ready);

    let ws_url = normalize_ws_url(&info.base_url);

    let work_dir = tempfile::tempdir().unwrap();
    let cwd_path = work_dir.path().to_string_lossy().to_string();

    let req = WsConnectRequest::for_local_with_cwd(80, 24, Some(cwd_path));
    let handle = TerminalWsClient::connect(&ws_url, req)
        .await
        .expect("TerminalWsClient should connect with custom CWD");

    assert!(handle.is_connected());
    assert!(!handle.session_id().is_empty());

    // Disconnect and stop
    let _ = handle.disconnect();
    supervisor
        .stop(Duration::from_secs(5))
        .await
        .expect("supervisor should stop");
}
