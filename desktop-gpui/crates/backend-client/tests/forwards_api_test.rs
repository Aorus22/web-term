//! Contract tests for Port Forwarding REST endpoints.

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use webterm_backend_client::{
    BackendClient, CreateForwardRequest, UpdateForwardRequest,
};

#[tokio::test]
async fn test_forwards_crud_and_lifecycle_endpoints() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let server_task = tokio::spawn(async move {
        // Handle 6 sequential requests:
        // 1. GET /api/forwards (list)
        // 2. POST /api/forwards (create)
        // 3. PUT /api/forwards/f-100 (update)
        // 4. POST /api/forwards/f-100/start (start)
        // 5. POST /api/forwards/f-100/stop (stop)
        // 6. DELETE /api/forwards/f-100 (delete)
        for i in 0..6 {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 4096];
            let n = socket.read(&mut buf).await.unwrap();
            let req_str = String::from_utf8_lossy(&buf[..n]);

            match i {
                0 => {
                    // 1. GET /api/forwards
                    assert!(req_str.starts_with("GET /api/forwards HTTP/1.1"));
                    let body = r#"[
                        {
                            "id":"f-100",
                            "name":"Postgres Tunnel",
                            "connection_id":"c-1",
                            "local_port":5432,
                            "remote_port":5432,
                            "type":"local",
                            "active":false,
                            "auto_start":false,
                            "error":"",
                            "created_at":"2026-09-05T00:00:00Z",
                            "updated_at":"2026-09-05T00:00:00Z"
                        }
                    ]"#;
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    socket.write_all(resp.as_bytes()).await.unwrap();
                }
                1 => {
                    // 2. POST /api/forwards
                    assert!(req_str.starts_with("POST /api/forwards HTTP/1.1"));
                    let json_start = req_str.find("\r\n\r\n").unwrap() + 4;
                    let payload: serde_json::Value =
                        serde_json::from_str(&req_str[json_start..]).unwrap();
                    assert_eq!(payload["name"], "Redis Reverse");
                    assert_eq!(payload["connection_id"], "c-2");
                    assert_eq!(payload["local_port"], 6379);
                    assert_eq!(payload["remote_port"], 16379);
                    assert_eq!(payload["type"], "reverse");

                    let body = r#"{
                        "id":"f-101",
                        "name":"Redis Reverse",
                        "connection_id":"c-2",
                        "local_port":6379,
                        "remote_port":16379,
                        "type":"reverse",
                        "active":false,
                        "auto_start":false,
                        "error":"",
                        "created_at":"2026-09-05T01:00:00Z",
                        "updated_at":"2026-09-05T01:00:00Z"
                    }"#;
                    let resp = format!(
                        "HTTP/1.1 201 Created\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    socket.write_all(resp.as_bytes()).await.unwrap();
                }
                2 => {
                    // 3. PUT /api/forwards/f-100
                    assert!(req_str.starts_with("PUT /api/forwards/f-100 HTTP/1.1"));
                    let json_start = req_str.find("\r\n\r\n").unwrap() + 4;
                    let payload: serde_json::Value =
                        serde_json::from_str(&req_str[json_start..]).unwrap();
                    assert_eq!(payload["name"], "Postgres Tunnel Renamed");

                    let body = r#"{
                        "id":"f-100",
                        "name":"Postgres Tunnel Renamed",
                        "connection_id":"c-1",
                        "local_port":5432,
                        "remote_port":5432,
                        "type":"local",
                        "active":false,
                        "auto_start":false,
                        "error":"",
                        "created_at":"2026-09-05T00:00:00Z",
                        "updated_at":"2026-09-05T02:00:00Z"
                    }"#;
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    socket.write_all(resp.as_bytes()).await.unwrap();
                }
                3 => {
                    // 4. POST /api/forwards/f-100/start
                    assert!(req_str.starts_with("POST /api/forwards/f-100/start HTTP/1.1"));
                    let body = r#"{"status":"active"}"#;
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    socket.write_all(resp.as_bytes()).await.unwrap();
                }
                4 => {
                    // 5. POST /api/forwards/f-100/stop
                    assert!(req_str.starts_with("POST /api/forwards/f-100/stop HTTP/1.1"));
                    let body = r#"{"status":"inactive"}"#;
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    socket.write_all(resp.as_bytes()).await.unwrap();
                }
                5 => {
                    // 6. DELETE /api/forwards/f-100
                    assert!(req_str.starts_with("DELETE /api/forwards/f-100 HTTP/1.1"));
                    let resp = "HTTP/1.1 204 No Content\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
                    socket.write_all(resp.as_bytes()).await.unwrap();
                }
                _ => unreachable!(),
            }
        }
    });

    let client = BackendClient::new(format!("http://127.0.0.1:{}", port));

    // 1. List forwards
    let list = client.list_forwards().await.expect("should list forwards");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, "f-100");
    assert_eq!(list[0].name, "Postgres Tunnel");
    assert_eq!(list[0].local_port, 5432);
    assert_eq!(list[0].remote_port, 5432);
    assert!(!list[0].is_reverse());
    assert_eq!(list[0].mapping_display(), "localhost:5432 \u{2192} :5432");

    // 2. Create forward
    let create_req = CreateForwardRequest {
        name: "Redis Reverse".to_string(),
        connection_id: "c-2".to_string(),
        local_port: 6379,
        remote_port: 16379,
        forward_type: "reverse".to_string(),
    };
    let created = client
        .create_forward(&create_req)
        .await
        .expect("should create forward");
    assert_eq!(created.id, "f-101");
    assert_eq!(created.name, "Redis Reverse");
    assert!(created.is_reverse());
    assert_eq!(created.mapping_display(), ":16379 \u{2190} localhost:6379");

    // 3. Update forward
    let update_req = UpdateForwardRequest {
        name: "Postgres Tunnel Renamed".to_string(),
        connection_id: "c-1".to_string(),
        local_port: 5432,
        remote_port: 5432,
        forward_type: "local".to_string(),
    };
    let updated = client
        .update_forward("f-100", &update_req)
        .await
        .expect("should update forward");
    assert_eq!(updated.name, "Postgres Tunnel Renamed");

    // 4. Start forward
    let start_res = client
        .start_forward("f-100")
        .await
        .expect("should start forward");
    assert_eq!(start_res.status, "active");

    // 5. Stop forward
    let stop_res = client
        .stop_forward("f-100")
        .await
        .expect("should stop forward");
    assert_eq!(stop_res.status, "inactive");

    // 6. Delete forward
    client
        .delete_forward("f-100")
        .await
        .expect("should delete forward");

    server_task.await.unwrap();
}

#[tokio::test]
async fn test_list_forwards_handles_null_slice() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        if let Ok((mut socket, _)) = listener.accept().await {
            let mut buf = [0u8; 1024];
            let _ = socket.read(&mut buf).await;
            let body = "null";
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = socket.write_all(resp.as_bytes()).await;
        }
    });

    let client = BackendClient::new(format!("http://127.0.0.1:{}", port));
    let list = client
        .list_forwards()
        .await
        .expect("should deserialize null as empty vec");
    assert!(list.is_empty());
}
