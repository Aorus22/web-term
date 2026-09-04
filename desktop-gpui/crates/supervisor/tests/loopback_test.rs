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
async fn test_backend_binds_strictly_to_loopback() {
    let backend_path = match resolve_backend_path() {
        Some(p) => p,
        None => {
            eprintln!("Fixture missing at desktop-gpui/test-support/backend; skipping loopback test");
            return;
        }
    };

    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("loopback_webterm.db");
    let key = "a1b2c3d4e5f67890123456789abcdef0123456789abcdef0123456789abcdef0";
    let opts = SpawnOptions::new(backend_path, db_path, key).unwrap();

    // Verify WEBTERM_HOST is injected with 127.0.0.1
    let env = opts.env_vars();
    let host_pair = env.iter().find(|(k, _)| *k == "WEBTERM_HOST");
    assert_eq!(
        host_pair.map(|(_, v)| v.as_str()),
        Some("127.0.0.1"),
        "Supervisor must inject WEBTERM_HOST=127.0.0.1"
    );

    let mut supervisor = Supervisor::new();
    let info = supervisor.spawn(opts).await.expect("supervisor should reach ready");
    assert_eq!(supervisor.status(), BackendStatus::Ready);
    assert!(info.port > 0);
    assert!(info.base_url.starts_with("http://127.0.0.1:"));

    // Verify loopback connection succeeds
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/api/settings", info.base_url))
        .send()
        .await
        .expect("Loopback request should succeed");
    assert!(resp.status().is_success());

    supervisor.stop(Duration::from_secs(2)).await.unwrap();
}
