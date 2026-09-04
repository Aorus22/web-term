use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;
use webterm_backend_client::{
    normalize_ws_url, TerminalWsClient, TerminalWsError, WsConnectRequest,
};

#[test]
fn test_normalize_ws_url_variants() {
    assert_eq!(
        normalize_ws_url("http://127.0.0.1:54321"),
        "ws://127.0.0.1:54321/ws"
    );
    assert_eq!(
        normalize_ws_url("http://127.0.0.1:54321/"),
        "ws://127.0.0.1:54321/ws"
    );
    assert_eq!(
        normalize_ws_url("http://127.0.0.1:54321/ws"),
        "ws://127.0.0.1:54321/ws"
    );
    assert_eq!(
        normalize_ws_url("https://localhost:8443"),
        "wss://localhost:8443/ws"
    );
    assert_eq!(
        normalize_ws_url("ws://localhost:9000/ws"),
        "ws://localhost:9000/ws"
    );
}

#[tokio::test]
async fn test_ws_connect_handshake_and_bidirectional_streaming() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let server_task = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut ws = accept_async(stream).await.unwrap();

        // 1. Receive connect handshake
        let msg = ws.next().await.unwrap().unwrap();
        let text = match msg {
            Message::Text(t) => t,
            other => panic!("Expected text frame, got {:?}", other),
        };
        let req: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(req["type"], "connect");
        assert_eq!(req["connection_id"], "conn-test-1");
        assert_eq!(req["cols"], 80);
        assert_eq!(req["rows"], 24);

        // 2. Reply with connected response
        let resp = serde_json::json!({
            "type": "connected",
            "session_id": "session-xyz-123"
        });
        ws.send(Message::Text(resp.to_string().into())).await.unwrap();

        // 3. Receive ready frame
        let ready_msg = ws.next().await.unwrap().unwrap();
        let ready_text = match ready_msg {
            Message::Text(t) => t,
            other => panic!("Expected ready text frame, got {:?}", other),
        };
        let ready_val: serde_json::Value = serde_json::from_str(&ready_text).unwrap();
        assert_eq!(ready_val["type"], "ready");

        // 4. Send scrollback binary frame to client
        let banner = b"Hello from remote PTY\r\n";
        ws.send(Message::Binary(banner.to_vec().into())).await.unwrap();

        // 5. Receive user input binary frame
        let input_msg = ws.next().await.unwrap().unwrap();
        let input_bytes = match input_msg {
            Message::Binary(b) => b,
            other => panic!("Expected binary input frame, got {:?}", other),
        };
        assert_eq!(&input_bytes[..], b"echo hello\n");

        // 6. Receive resize control frame
        let resize_msg = ws.next().await.unwrap().unwrap();
        let resize_text = match resize_msg {
            Message::Text(t) => t,
            other => panic!("Expected resize text frame, got {:?}", other),
        };
        let resize_val: serde_json::Value = serde_json::from_str(&resize_text).unwrap();
        assert_eq!(resize_val["type"], "resize");
        assert_eq!(resize_val["cols"], 120);
        assert_eq!(resize_val["rows"], 40);

        // 7. Receive get-cwd control frame
        let cwd_msg = ws.next().await.unwrap().unwrap();
        let cwd_text = match cwd_msg {
            Message::Text(t) => t,
            other => panic!("Expected get-cwd text frame, got {:?}", other),
        };
        let cwd_val: serde_json::Value = serde_json::from_str(&cwd_text).unwrap();
        assert_eq!(cwd_val["type"], "get-cwd");

        // Reply with cwd
        let cwd_resp = serde_json::json!({
            "type": "cwd",
            "path": "/home/user/project"
        });
        ws.send(Message::Text(cwd_resp.to_string().into())).await.unwrap();

        // 8. Receive disconnect control frame
        let dc_msg = ws.next().await.unwrap().unwrap();
        let dc_text = match dc_msg {
            Message::Text(t) => t,
            other => panic!("Expected disconnect text frame, got {:?}", other),
        };
        let dc_val: serde_json::Value = serde_json::from_str(&dc_text).unwrap();
        assert_eq!(dc_val["type"], "disconnect");
    });

    let ws_url = format!("ws://127.0.0.1:{}/ws", port);
    let connect_req = WsConnectRequest::for_saved_connection("conn-test-1", 80, 24);

    let handle = TerminalWsClient::connect(&ws_url, connect_req)
        .await
        .expect("TerminalWsClient::connect should succeed");

    assert_eq!(handle.session_id(), "session-xyz-123");
    assert!(handle.is_connected());

    // Verify inbound binary message
    let output_rx = handle.output_receiver();
    let banner = output_rx.recv_async().await.expect("should receive banner");
    assert_eq!(banner, b"Hello from remote PTY\r\n");

    // Send user input
    handle
        .send_input(b"echo hello\n".to_vec())
        .expect("send_input should succeed");

    // Send resize
    handle.resize(120, 40).expect("resize should succeed");

    // Request cwd
    handle.get_cwd().expect("get_cwd should succeed");
    let ctrl_rx = handle.control_receiver();
    let cwd_resp = ctrl_rx.recv_async().await.expect("should receive cwd response");
    assert_eq!(cwd_resp.msg_type, "cwd");
    assert_eq!(cwd_resp.path.as_deref(), Some("/home/user/project"));

    // Disconnect
    handle.disconnect().expect("disconnect should succeed");

    server_task.await.unwrap();
}

#[tokio::test]
async fn test_ws_attach_handshake() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let server_task = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut ws = accept_async(stream).await.unwrap();

        // 1. Receive attach handshake
        let msg = ws.next().await.unwrap().unwrap();
        let text = match msg {
            Message::Text(t) => t,
            other => panic!("Expected text frame, got {:?}", other),
        };
        let req: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(req["type"], "attach");
        assert_eq!(req["session_id"], "sess-existing-999");

        // 2. Reply with connected response
        let resp = serde_json::json!({
            "type": "connected",
            "session_id": "sess-existing-999"
        });
        ws.send(Message::Text(resp.to_string().into())).await.unwrap();

        // 3. Receive ready frame
        let ready_msg = ws.next().await.unwrap().unwrap();
        let ready_text = match ready_msg {
            Message::Text(t) => t,
            other => panic!("Expected ready text frame, got {:?}", other),
        };
        let ready_val: serde_json::Value = serde_json::from_str(&ready_text).unwrap();
        assert_eq!(ready_val["type"], "ready");

        // 4. Send scrollback buffer
        let scrollback = b"Restored session scrollback\r\n";
        ws.send(Message::Binary(scrollback.to_vec().into())).await.unwrap();
    });

    let ws_url = format!("ws://127.0.0.1:{}/ws", port);
    let handle = TerminalWsClient::attach(&ws_url, "sess-existing-999")
        .await
        .expect("TerminalWsClient::attach should succeed");

    assert_eq!(handle.session_id(), "sess-existing-999");
    assert!(handle.is_connected());

    let output_rx = handle.output_receiver();
    let scrollback = output_rx.recv_async().await.expect("should receive scrollback");
    assert_eq!(scrollback, b"Restored session scrollback\r\n");

    server_task.await.unwrap();
}

#[tokio::test]
async fn test_ws_connect_server_error_response() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let server_task = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut ws = accept_async(stream).await.unwrap();

        // Read connect request
        let _ = ws.next().await.unwrap().unwrap();

        // Reply with error
        let err_resp = serde_json::json!({
            "type": "error",
            "message": "Host not authorized by security policy"
        });
        ws.send(Message::Text(err_resp.to_string().into())).await.unwrap();
    });

    let ws_url = format!("ws://127.0.0.1:{}/ws", port);
    let req = WsConnectRequest::for_quick_connect("forbidden.host", 22, "root", "secret", 80, 24);

    let err = TerminalWsClient::connect(&ws_url, req)
        .await
        .expect_err("should return error");

    match err {
        TerminalWsError::Protocol(msg) => {
            assert!(
                msg.contains("Host not authorized"),
                "Expected SSRF error message, got: {}",
                msg
            );
        }
        other => panic!("Expected Protocol error, got {:?}", other),
    }

    server_task.await.unwrap();
}

#[test]
fn test_ws_connect_request_local_with_and_without_cwd() {
    // 1. Without cwd
    let req_no_cwd = WsConnectRequest::for_local(80, 24);
    assert_eq!(req_no_cwd.session_type.as_deref(), Some("local"));
    assert_eq!(req_no_cwd.connection_id.as_deref(), Some("local"));
    assert!(req_no_cwd.cwd.is_none());

    let val_no_cwd = serde_json::to_value(&req_no_cwd).unwrap();
    assert_eq!(val_no_cwd["type"], "connect");
    assert_eq!(val_no_cwd["session_type"], "local");
    assert_eq!(val_no_cwd["connection_id"], "local");
    assert!(val_no_cwd.get("cwd").is_none());

    // 2. With cwd
    let req_with_cwd = WsConnectRequest::for_local_with_cwd(
        120,
        35,
        Some("/home/developer/workspace".to_string()),
    );
    assert_eq!(req_with_cwd.cols, 120);
    assert_eq!(req_with_cwd.rows, 35);
    assert_eq!(
        req_with_cwd.cwd.as_deref(),
        Some("/home/developer/workspace")
    );

    let val_with_cwd = serde_json::to_value(&req_with_cwd).unwrap();
    assert_eq!(val_with_cwd["cwd"], "/home/developer/workspace");
    assert_eq!(val_with_cwd["cols"], 120);
    assert_eq!(val_with_cwd["rows"], 35);
}
