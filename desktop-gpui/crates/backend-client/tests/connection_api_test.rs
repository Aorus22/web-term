//! Contract tests for Connection and SSH Key REST endpoints.

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use webterm_backend_client::{
    BackendClient, CreateConnectionRequest, CreateKeyRequest, UpdateConnectionRequest,
};

#[tokio::test]
async fn test_connection_crud_endpoints() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let server_task = tokio::spawn(async move {
        // Handle 4 sequential requests: GET single, POST, PUT, DELETE
        for i in 0..4 {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 4096];
            let n = socket.read(&mut buf).await.unwrap();
            let req_str = String::from_utf8_lossy(&buf[..n]);

            match i {
                0 => {
                    // 1. GET /api/connections/c-test
                    assert!(req_str.starts_with("GET /api/connections/c-test HTTP/1.1"));
                    let body = r#"{"id":"c-test","label":"Staging DB","host":"db.staging.local","port":5432,"username":"postgres","tags":["db","staging"],"auth_method":"password"}"#;
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    socket.write_all(resp.as_bytes()).await.unwrap();
                }
                1 => {
                    // 2. POST /api/connections
                    assert!(req_str.starts_with("POST /api/connections HTTP/1.1"));
                    let json_start = req_str.find("\r\n\r\n").unwrap() + 4;
                    let payload: serde_json::Value =
                        serde_json::from_str(&req_str[json_start..]).unwrap();
                    assert_eq!(payload["label"], "New Server");
                    assert_eq!(payload["host"], "192.168.1.50");
                    assert_eq!(payload["port"], 22);
                    assert_eq!(payload["username"], "ubuntu");
                    assert_eq!(payload["auth_method"], "password");

                    let body = r#"{"id":"c-new-1","label":"New Server","host":"192.168.1.50","port":22,"username":"ubuntu","tags":["web"],"auth_method":"password"}"#;
                    let resp = format!(
                        "HTTP/1.1 201 Created\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    socket.write_all(resp.as_bytes()).await.unwrap();
                }
                2 => {
                    // 3. PUT /api/connections/c-test
                    assert!(req_str.starts_with("PUT /api/connections/c-test HTTP/1.1"));
                    let json_start = req_str.find("\r\n\r\n").unwrap() + 4;
                    let payload: serde_json::Value =
                        serde_json::from_str(&req_str[json_start..]).unwrap();
                    assert_eq!(payload["label"], "Updated DB");

                    let body = r#"{"id":"c-test","label":"Updated DB","host":"db.staging.local","port":5432,"username":"postgres","tags":["db"],"auth_method":"password"}"#;
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    socket.write_all(resp.as_bytes()).await.unwrap();
                }
                3 => {
                    // 4. DELETE /api/connections/c-test
                    assert!(req_str.starts_with("DELETE /api/connections/c-test HTTP/1.1"));
                    let resp = "HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n";
                    socket.write_all(resp.as_bytes()).await.unwrap();
                }
                _ => {}
            }
        }
    });

    let client = BackendClient::new(format!("http://127.0.0.1:{}", port));

    // 1. GET
    let conn = client.get_connection("c-test").await.unwrap();
    assert_eq!(conn.id, "c-test");
    assert_eq!(conn.label, "Staging DB");
    assert_eq!(conn.port, 5432);

    // 2. CREATE
    let create_req = CreateConnectionRequest {
        label: "New Server".to_string(),
        host: "192.168.1.50".to_string(),
        port: 22,
        username: "ubuntu".to_string(),
        password: Some("secretpass".to_string()),
        tags: vec!["web".to_string()],
        auth_method: "password".to_string(),
        ssh_key_id: None,
    };
    let created = client.create_connection(&create_req).await.unwrap();
    assert_eq!(created.id, "c-new-1");
    assert_eq!(created.label, "New Server");

    // 3. UPDATE
    let update_req = UpdateConnectionRequest {
        label: "Updated DB".to_string(),
        host: "db.staging.local".to_string(),
        port: 5432,
        username: "postgres".to_string(),
        password: None,
        tags: vec!["db".to_string()],
        auth_method: "password".to_string(),
        ssh_key_id: None,
    };
    let updated = client.update_connection("c-test", &update_req).await.unwrap();
    assert_eq!(updated.label, "Updated DB");

    // 4. DELETE
    client.delete_connection("c-test").await.unwrap();

    server_task.await.unwrap();
}

#[tokio::test]
async fn test_connection_export_import_endpoints() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let server_task = tokio::spawn(async move {
        // Request 1: Export
        {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 2048];
            let n = socket.read(&mut buf).await.unwrap();
            let req_str = String::from_utf8_lossy(&buf[..n]);
            assert!(req_str.starts_with("GET /api/connections/export HTTP/1.1"));

            let body = r#"[
                {"id":"c1","label":"Server 1","host":"10.0.0.1","port":22,"username":"root","tags":[],"auth_method":"password"},
                {"id":"c2","label":"Server 2","host":"10.0.0.2","port":2222,"username":"dev","tags":["dev"],"auth_method":"key","ssh_key_id":"k1"}
            ]"#;
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            socket.write_all(resp.as_bytes()).await.unwrap();
        }

        // Request 2: Import
        {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 4096];
            let n = socket.read(&mut buf).await.unwrap();
            let req_str = String::from_utf8_lossy(&buf[..n]);
            assert!(req_str.starts_with("POST /api/connections/import HTTP/1.1"));

            let body = r#"{"imported":2,"skipped":0}"#;
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            socket.write_all(resp.as_bytes()).await.unwrap();
        }
    });

    let client = BackendClient::new(format!("http://127.0.0.1:{}", port));

    // Export
    let exported = client.export_connections().await.unwrap();
    assert_eq!(exported.len(), 2);
    assert_eq!(exported[0].id, "c1");
    assert_eq!(exported[1].auth_method, "key");

    // Import
    let import_result = client.import_connections(&exported).await.unwrap();
    assert_eq!(import_result.imported, 2);
    assert_eq!(import_result.skipped, 0);

    server_task.await.unwrap();
}

#[tokio::test]
async fn test_keys_crud_endpoints() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let server_task = tokio::spawn(async move {
        // 4 sequential requests: list, get, create, delete
        for i in 0..4 {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 4096];
            let n = socket.read(&mut buf).await.unwrap();
            let req_str = String::from_utf8_lossy(&buf[..n]);

            match i {
                0 => {
                    // 1. GET /api/keys
                    assert!(req_str.starts_with("GET /api/keys HTTP/1.1"));
                    let body = r#"[
                        {"id":"k1","name":"id_ed25519","key_type":"Ed25519","fingerprint":"SHA256:abc123def456","created_at":"2026-09-01T12:00:00Z"}
                    ]"#;
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    socket.write_all(resp.as_bytes()).await.unwrap();
                }
                1 => {
                    // 2. GET /api/keys/k1
                    assert!(req_str.starts_with("GET /api/keys/k1 HTTP/1.1"));
                    let body = r#"{"id":"k1","name":"id_ed25519","key_type":"Ed25519","fingerprint":"SHA256:abc123def456"}"#;
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    socket.write_all(resp.as_bytes()).await.unwrap();
                }
                2 => {
                    // 3. POST /api/keys
                    assert!(req_str.starts_with("POST /api/keys HTTP/1.1"));
                    let json_start = req_str.find("\r\n\r\n").unwrap() + 4;
                    let payload: serde_json::Value =
                        serde_json::from_str(&req_str[json_start..]).unwrap();
                    assert_eq!(payload["name"], "deploy_rsa");
                    assert_eq!(payload["key_base64"], "bW9jay1rZXktZGF0YQ==");

                    let body = r#"{"id":"k2","name":"deploy_rsa","key_type":"RSA","fingerprint":"SHA256:xyz789"}"#;
                    let resp = format!(
                        "HTTP/1.1 201 Created\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    socket.write_all(resp.as_bytes()).await.unwrap();
                }
                3 => {
                    // 4. DELETE /api/keys/k1
                    assert!(req_str.starts_with("DELETE /api/keys/k1 HTTP/1.1"));
                    let resp = "HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n";
                    socket.write_all(resp.as_bytes()).await.unwrap();
                }
                _ => {}
            }
        }
    });

    let client = BackendClient::new(format!("http://127.0.0.1:{}", port));

    // 1. List
    let keys = client.list_keys().await.unwrap();
    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0].name, "id_ed25519");
    assert_eq!(keys[0].key_type, "Ed25519");

    // 2. Get
    let key = client.get_key("k1").await.unwrap();
    assert_eq!(key.id, "k1");
    assert_eq!(key.fingerprint, "SHA256:abc123def456");

    // 3. Create
    let create_req = CreateKeyRequest {
        name: "deploy_rsa".to_string(),
        key_base64: "bW9jay1rZXktZGF0YQ==".to_string(),
    };
    let created = client.create_key(&create_req).await.unwrap();
    assert_eq!(created.id, "k2");
    assert_eq!(created.key_type, "RSA");

    // 4. Delete
    client.delete_key("k1").await.unwrap();

    server_task.await.unwrap();
}
