use std::path::PathBuf;
use std::time::Duration;
use webterm_supervisor::{BackendStatus, SpawnOptions, Supervisor};

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

#[tokio::test]
async fn spawns_real_backend_and_reaches_ready() {
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
    let info = supervisor.spawn(opts).await.expect("supervisor should reach ready");
    assert!(info.base_url.starts_with("http://127.0.0.1:"));
    assert_eq!(supervisor.status(), BackendStatus::Ready);

    supervisor.stop(Duration::from_secs(5)).await.unwrap();
}

#[tokio::test]
async fn invalid_path_reports_failed() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("webterm.db");
    let key = "a1b2c3d4e5f67890123456789abcdef0123456789abcdef0123456789abcdef0";
    let opts = SpawnOptions::new("nonexistent_backend_path_xyz", db_path, key).unwrap();

    let mut supervisor = Supervisor::new();
    let result = supervisor.spawn(opts).await;
    assert!(result.is_err());
    match supervisor.status() {
        BackendStatus::Failed { reason, stderr_tail } => {
            assert!(!reason.is_empty());
            assert!(!stderr_tail.is_empty());
        }
        other => panic!("expected BackendStatus::Failed, got {:?}", other),
    }
}

#[tokio::test]
async fn stop_terminates_child() {
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
    let info = supervisor.spawn(opts).await.unwrap();
    assert_eq!(supervisor.status(), BackendStatus::Ready);

    supervisor.stop(Duration::from_secs(2)).await.unwrap();

    // Verify child is not accepting connections
    let probe = Supervisor::adopt_or_clear(Some(info.base_url)).await;
    assert!(probe.is_none(), "backend should be terminated after stop");
}

#[test]
fn env_only_secrets() {
    let key = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
    let opts = SpawnOptions::new("backend", "db.sqlite", key).unwrap();
    let env = opts.env_vars();
    let found = env.iter().find(|(k, v)| *k == "WEBTERM_ENCRYPTION_KEY" && v == key);
    assert!(found.is_some(), "encryption key must be present in env_vars");

    let cmd = opts.build_command();
    let std_cmd = cmd.as_std();
    for arg in std_cmd.get_args() {
        assert!(!arg.to_string_lossy().contains(key), "argv must never contain key material");
    }
}
