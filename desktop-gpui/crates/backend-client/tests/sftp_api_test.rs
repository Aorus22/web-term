//! Contract tests for SFTP REST endpoints.

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use webterm_backend_client::BackendClient;

#[tokio::test]
async fn test_sftp_list_and_mkdir_endpoints() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let server_task = tokio::spawn(async move {
        // 1. GET /api/sftp/ls?connectionId=local&path=/home/user
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut buf = [0u8; 4096];
        let n = socket.read(&mut buf).await.unwrap();
        let req_str = String::from_utf8_lossy(&buf[..n]);
        assert!(req_str.starts_with("GET /api/sftp/ls?connectionId=local&path="));

        let body = r#"[
            {"name":"documents","size":4096,"mode":2147484141,"modTime":"2026-09-05T00:00:00Z","isDir":true},
            {"name":"notes.txt","size":128,"mode":420,"modTime":"2026-09-05T01:00:00Z","isDir":false}
        ]"#;
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        socket.write_all(resp.as_bytes()).await.unwrap();

        // 2. POST /api/sftp/mkdir?connectionId=local&path=/home/user/newfolder
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut buf = [0u8; 4096];
        let n = socket.read(&mut buf).await.unwrap();
        let req_str = String::from_utf8_lossy(&buf[..n]);
        assert!(req_str.starts_with("POST /api/sftp/mkdir?connectionId=local&path="));

        let resp = "HTTP/1.1 201 Created\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
        socket.write_all(resp.as_bytes()).await.unwrap();
    });

    let client = BackendClient::new(format!("http://127.0.0.1:{}", port));

    let files = client
        .sftp_list("local", "/home/user")
        .await
        .expect("sftp_list should succeed");
    assert_eq!(files.len(), 2);
    assert_eq!(files[0].name, "documents");
    assert!(files[0].is_dir);
    assert_eq!(files[1].name, "notes.txt");
    assert!(!files[1].is_dir);
    assert_eq!(files[1].size, 128);

    client
        .sftp_mkdir("local", "/home/user/newfolder")
        .await
        .expect("sftp_mkdir should succeed");

    server_task.await.unwrap();
}

#[tokio::test]
async fn test_sftp_rename_and_remove_endpoints() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let server_task = tokio::spawn(async move {
        // 1. POST /api/sftp/rename
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut buf = [0u8; 4096];
        let n = socket.read(&mut buf).await.unwrap();
        let req_str = String::from_utf8_lossy(&buf[..n]);
        assert!(req_str.starts_with("POST /api/sftp/rename?"));

        let resp = "HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
        socket.write_all(resp.as_bytes()).await.unwrap();

        // 2. DELETE /api/sftp/remove
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut buf = [0u8; 4096];
        let n = socket.read(&mut buf).await.unwrap();
        let req_str = String::from_utf8_lossy(&buf[..n]);
        assert!(req_str.starts_with("DELETE /api/sftp/remove?"));

        let resp = "HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n";
        socket.write_all(resp.as_bytes()).await.unwrap();
    });

    let client = BackendClient::new(format!("http://127.0.0.1:{}", port));

    client
        .sftp_rename("local", "/tmp/old.txt", "/tmp/new.txt")
        .await
        .expect("sftp_rename should succeed");

    client
        .sftp_remove("local", "/tmp/new.txt")
        .await
        .expect("sftp_remove should succeed");

    server_task.await.unwrap();
}

#[tokio::test]
async fn test_sftp_transfers_and_status_endpoints() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let server_task = tokio::spawn(async move {
        // 1. GET /api/sftp/transfers
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut buf = [0u8; 4096];
        let n = socket.read(&mut buf).await.unwrap();
        let req_str = String::from_utf8_lossy(&buf[..n]);
        assert!(req_str.starts_with("GET /api/sftp/transfers HTTP/1.1"));

        let body = r#"[
            {"id":"tx-1","bytes_transferred":500,"total_bytes":1000,"status":"transferring"},
            {"id":"tx-2","bytes_transferred":2048,"total_bytes":2048,"status":"completed"}
        ]"#;
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        socket.write_all(resp.as_bytes()).await.unwrap();

        // 2. GET /api/sftp/transfer/status?transferId=tx-1
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut buf = [0u8; 4096];
        let n = socket.read(&mut buf).await.unwrap();
        let req_str = String::from_utf8_lossy(&buf[..n]);
        assert!(req_str.starts_with("GET /api/sftp/transfer/status?transferId=tx-1 HTTP/1.1"));

        let body = r#"{"id":"tx-1","bytes_transferred":750,"total_bytes":1000,"status":"transferring"}"#;
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        socket.write_all(resp.as_bytes()).await.unwrap();
    });

    let client = BackendClient::new(format!("http://127.0.0.1:{}", port));

    let list = client
        .sftp_list_transfers()
        .await
        .expect("sftp_list_transfers should succeed");
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].id, "tx-1");
    assert_eq!(list[0].status, "transferring");
    assert_eq!(list[1].id, "tx-2");
    assert_eq!(list[1].status, "completed");

    let status = client
        .sftp_transfer_status("tx-1")
        .await
        .expect("sftp_transfer_status should succeed");
    assert_eq!(status.id, "tx-1");
    assert_eq!(status.bytes_transferred, 750);
    assert_eq!(status.total_bytes, 1000);

    server_task.await.unwrap();
}
