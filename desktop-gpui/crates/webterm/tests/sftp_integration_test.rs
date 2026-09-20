//! End-to-end integration test for SFTP local filesystem operations
//! exercising the real backend supervisor, REST endpoints, and filesystem lifecycle.

use std::path::PathBuf;
use std::time::Duration;
use webterm::backend_client::BackendClient;
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
async fn test_sftp_supervisor_e2e_filesystem_lifecycle() {
    let backend_path = match resolve_backend_path() {
        Some(p) => p,
        None => {
            eprintln!("Fixture missing at desktop-gpui/test-support/backend: skipping e2e test");
            return;
        }
    };

    let temp_root = tempfile::tempdir().expect("tempdir should be created");
    let db_path = temp_root.path().join("webterm.db");
    let test_dir = temp_root.path().join("sftp_test_dir");
    std::fs::create_dir_all(&test_dir).expect("test_dir should be created");

    let key = "a1b2c3d4e5f67890123456789abcdef0123456789abcdef0123456789abcdef0";
    let opts = SpawnOptions::new(backend_path, db_path, key).unwrap();

    let mut supervisor = Supervisor::new();
    let info = supervisor
        .spawn(opts)
        .await
        .expect("supervisor should start and reach ready");
    assert_eq!(supervisor.status(), BackendStatus::Ready);

    let client = BackendClient::new(info.base_url);
    let test_dir_str = test_dir.to_str().unwrap().replace('\\', "/");

    // 1. Verify Home directory endpoint
    let home = client
        .sftp_home("local")
        .await
        .expect("sftp_home should succeed for local");
    assert!(!home.is_empty(), "Home directory should not be empty");

    // 2. List initially empty test directory
    let initial_list = client
        .sftp_list("local", &test_dir_str)
        .await
        .expect("sftp_list should succeed");
    assert_eq!(initial_list.len(), 0, "Directory should initially be empty");

    // 3. Create a subdirectory via sftp_mkdir
    let subfolder_path = format!("{}/folder_alpha", test_dir_str);
    client
        .sftp_mkdir("local", &subfolder_path)
        .await
        .expect("sftp_mkdir should create subdirectory");

    // Verify subdirectory appears in sftp_list
    let list_after_mkdir = client
        .sftp_list("local", &test_dir_str)
        .await
        .expect("sftp_list should succeed after mkdir");
    assert_eq!(list_after_mkdir.len(), 1);
    assert_eq!(list_after_mkdir[0].name, "folder_alpha");
    assert!(list_after_mkdir[0].is_dir);

    // 4. Rename subdirectory via sftp_rename
    let renamed_subfolder = format!("{}/folder_beta", test_dir_str);
    client
        .sftp_rename("local", &subfolder_path, &renamed_subfolder)
        .await
        .expect("sftp_rename should succeed");

    let list_after_rename = client
        .sftp_list("local", &test_dir_str)
        .await
        .expect("sftp_list should succeed after rename");
    assert_eq!(list_after_rename.len(), 1);
    assert_eq!(list_after_rename[0].name, "folder_beta");
    assert!(list_after_rename[0].is_dir);

    // 5. Upload a file via sftp_upload
    let file_payload = b"Hello from WebTerm SFTP Integration Test!".to_vec();
    let backend_transfer_id = format!("wt-it-{}", std::process::id());
    let tx_id = client
        .sftp_upload(
            "local",
            &test_dir_str,
            "sample.txt",
            file_payload.clone(),
            &backend_transfer_id,
        )
        .await
        .expect("sftp_upload should succeed");
    assert!(
        !tx_id.is_empty(),
        "Upload should return non-empty transfer ID"
    );

    // Wait for background staging-to-destination goroutine to complete
    let mut uploaded_found = false;
    for _ in 0..20 {
        tokio::time::sleep(Duration::from_millis(100)).await;
        if let Ok(list) = client.sftp_list("local", &test_dir_str).await {
            if list.iter().any(|f| f.name == "sample.txt") {
                uploaded_found = true;
                break;
            }
        }
    }
    assert!(
        uploaded_found,
        "Uploaded file sample.txt should appear in listing"
    );

    // Verify uploaded file appears in sftp_list with correct attributes
    let list_after_upload = client
        .sftp_list("local", &test_dir_str)
        .await
        .expect("sftp_list should succeed after upload");
    assert_eq!(list_after_upload.len(), 2);
    let sample_file = list_after_upload
        .iter()
        .find(|f| f.name == "sample.txt")
        .expect("sample.txt should be in listing");
    assert!(!sample_file.is_dir);
    assert_eq!(sample_file.size, file_payload.len() as i64);

    // 6. Download file via sftp_download
    let sample_file_path = format!("{}/sample.txt", test_dir_str);
    let downloaded_data = client
        .sftp_download("local", &sample_file_path)
        .await
        .expect("sftp_download should succeed");
    assert_eq!(downloaded_data, file_payload);

    // 8. Delete uploaded file and subdirectory via sftp_remove
    client
        .sftp_remove("local", &sample_file_path)
        .await
        .expect("sftp_remove should delete file");
    client
        .sftp_remove("local", &renamed_subfolder)
        .await
        .expect("sftp_remove should delete folder");

    let list_after_cleanup = client
        .sftp_list("local", &test_dir_str)
        .await
        .expect("sftp_list should succeed after cleanup");
    assert_eq!(
        list_after_cleanup.len(),
        0,
        "Directory should be empty after deletion"
    );

    // 9. Cleanly stop backend supervisor
    supervisor
        .stop(Duration::from_secs(5))
        .await
        .expect("supervisor should stop cleanly");
}
