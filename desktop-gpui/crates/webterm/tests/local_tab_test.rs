//! Contract and integration tests for local terminal tabs and new tab launcher.

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;
use webterm::session::{SessionStatus, TerminalSessionManager, TerminalTab};
use webterm_backend_client::{
    normalize_ws_url, TerminalWsClient, WsConnectRequest,
};

#[test]
fn test_local_connect_request_structure_and_serialization() {
    let req = WsConnectRequest::for_local(80, 24);

    assert_eq!(req.msg_type, "connect");
    assert_eq!(req.session_type.as_deref(), Some("local"));
    assert_eq!(req.connection_id.as_deref(), Some("local"));
    assert_eq!(req.cols, 80);
    assert_eq!(req.rows, 24);
    assert_eq!(req.term, "xterm-256color");
    assert!(req.host.is_none());
    assert!(req.port.is_none());
    assert!(req.user.is_none());
    assert!(req.password.is_none());

    let val = serde_json::to_value(&req).expect("must serialize");
    assert_eq!(val["type"], "connect");
    assert_eq!(val["session_type"], "local");
    assert_eq!(val["connection_id"], "local");
    assert_eq!(val["cols"], 80);
    assert_eq!(val["rows"], 24);
    assert_eq!(val["term"], "xterm-256color");
    // Omitted fields should not be present in JSON
    assert!(val.get("host").is_none());
    assert!(val.get("port").is_none());
    assert!(val.get("user").is_none());
    assert!(val.get("password").is_none());
}

#[test]
fn test_quick_ssh_connect_request_structure() {
    let req = WsConnectRequest::for_quick_connect("192.168.1.100", 2222, "alice", "secret123", 100, 30);

    assert_eq!(req.msg_type, "connect");
    assert_eq!(req.session_type.as_deref(), Some("ssh"));
    assert_eq!(req.host.as_deref(), Some("192.168.1.100"));
    assert_eq!(req.port, Some(2222));
    assert_eq!(req.user.as_deref(), Some("alice"));
    assert_eq!(req.password.as_deref(), Some("secret123"));
    assert_eq!(req.auth_method.as_deref(), Some("password"));
    assert_eq!(req.cols, 100);
    assert_eq!(req.rows, 30);

    let val = serde_json::to_value(&req).expect("must serialize");
    assert_eq!(val["type"], "connect");
    assert_eq!(val["session_type"], "ssh");
    assert_eq!(val["host"], "192.168.1.100");
    assert_eq!(val["port"], 2222);
    assert_eq!(val["user"], "alice");
    assert_eq!(val["password"], "secret123");
}

#[test]
fn test_local_session_tab_lifecycle() {
    let mut manager = TerminalSessionManager::new();
    let tab_id = manager.alloc_tab_id();
    let tab = TerminalTab::new_headless(
        tab_id,
        "Local Shell",
        "local",
        Some("local".to_string()),
    );
    manager.add_tab(tab);

    assert_eq!(manager.tab_count(), 1);
    assert_eq!(manager.active_index(), 0);

    let active = manager.active_tab().unwrap();
    assert_eq!(active.id, tab_id);
    assert_eq!(active.title, "Local Shell");
    assert_eq!(active.session_type, "local");
    assert_eq!(active.connection_id.as_deref(), Some("local"));
    assert_eq!(active.status, SessionStatus::Connecting);

    // Transition to Connected
    manager.set_tab_status(tab_id, SessionStatus::Connected);
    assert_eq!(manager.active_tab().unwrap().status, SessionStatus::Connected);
}

#[tokio::test]
async fn test_local_shell_mock_ws_stream_and_resize() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let ws_url = normalize_ws_url(&format!("http://127.0.0.1:{}", port));

    let server_task = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut ws = accept_async(stream).await.unwrap();

        // 1. Receive connect handshake for local shell
        let msg = ws.next().await.unwrap().unwrap();
        let text = match msg {
            Message::Text(t) => t,
            other => panic!("Expected text frame, got {:?}", other),
        };
        let req: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(req["type"], "connect");
        assert_eq!(req["session_type"], "local");
        assert_eq!(req["connection_id"], "local");
        assert_eq!(req["cols"], 80);
        assert_eq!(req["rows"], 24);

        // 2. Respond with connected frame
        let resp = serde_json::json!({
            "type": "connected",
            "session_id": "session-local-xyz-100"
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

        // 4. Stream local shell prompt
        let prompt = b"user@localhost:~$ ";
        ws.send(Message::Binary(prompt.to_vec().into())).await.unwrap();

        // 5. Receive client binary input
        let input_msg = ws.next().await.unwrap().unwrap();
        let input_bytes = match input_msg {
            Message::Binary(b) => b,
            other => panic!("Expected binary input frame, got {:?}", other),
        };
        assert_eq!(&input_bytes[..], b"ls\n");

        // 6. Receive resize frame
        let resize_msg = ws.next().await.unwrap().unwrap();
        let resize_text = match resize_msg {
            Message::Text(t) => t,
            other => panic!("Expected resize text frame, got {:?}", other),
        };
        let resize_val: serde_json::Value = serde_json::from_str(&resize_text).unwrap();
        assert_eq!(resize_val["type"], "resize");
        assert_eq!(resize_val["cols"], 120);
        assert_eq!(resize_val["rows"], 35);

        // 7. Receive disconnect frame
        let dc_msg = ws.next().await.unwrap().unwrap();
        let dc_text = match dc_msg {
            Message::Text(t) => t,
            other => panic!("Expected disconnect frame, got {:?}", other),
        };
        let dc_val: serde_json::Value = serde_json::from_str(&dc_text).unwrap();
        assert_eq!(dc_val["type"], "disconnect");
    });

    let req = WsConnectRequest::for_local(80, 24);
    let handle = TerminalWsClient::connect(&ws_url, req).await.unwrap();

    assert_eq!(handle.session_id(), "session-local-xyz-100");
    assert!(handle.is_connected());

    // Receive prompt
    let output_rx = handle.output_receiver();
    let rx_data = output_rx.recv_async().await.unwrap();
    assert_eq!(&rx_data[..], b"user@localhost:~$ ");

    // Send input
    handle.send_input(b"ls\n".to_vec()).unwrap();

    // Send resize
    handle.resize(120, 35).unwrap();

    // Disconnect
    handle.disconnect().unwrap();

    server_task.await.unwrap();
}
