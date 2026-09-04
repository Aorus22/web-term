use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

/// Errors arising from WebSocket terminal operations.
#[derive(Debug, Error)]
pub enum TerminalWsError {
    #[error("WebSocket connection failed to {url}: {source}")]
    Connect {
        url: String,
        #[source]
        source: Box<tokio_tungstenite::tungstenite::Error>,
    },
    #[error("WebSocket protocol error: {0}")]
    Protocol(String),
    #[error("WebSocket connection closed")]
    Disconnected,
    #[error("JSON serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Channel send failed: {0}")]
    ChannelSend(String),
}

/// Normalizes an HTTP or WS URL to a valid WebSocket `/ws` endpoint.
pub fn normalize_ws_url(url: &str) -> String {
    let trimmed = url.trim_end_matches('/');
    let with_scheme = if let Some(stripped) = trimmed.strip_prefix("http://") {
        format!("ws://{}", stripped)
    } else if let Some(stripped) = trimmed.strip_prefix("https://") {
        format!("wss://{}", stripped)
    } else {
        trimmed.to_string()
    };

    if with_scheme.ends_with("/ws") {
        with_scheme
    } else {
        format!("{}/ws", with_scheme)
    }
}

/// Connection handshake payload sent from client to backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsConnectRequest {
    #[serde(rename = "type")]
    pub msg_type: String, // "connect"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_type: Option<String>, // "ssh" | "local"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh_key_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub passphrase: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    pub cols: u16,
    pub rows: u16,
    pub term: String,
}

impl WsConnectRequest {
    /// Create a request for an existing saved connection from the database.
    pub fn for_saved_connection(connection_id: impl Into<String>, cols: u16, rows: u16) -> Self {
        Self {
            msg_type: "connect".to_string(),
            session_type: Some("ssh".to_string()),
            session_id: None,
            host: None,
            port: None,
            user: None,
            password: None,
            auth_method: None,
            ssh_key_id: None,
            passphrase: None,
            connection_id: Some(connection_id.into()),
            cwd: None,
            cols,
            rows,
            term: "xterm-256color".to_string(),
        }
    }

    /// Create a request for a local shell terminal session.
    pub fn for_local(cols: u16, rows: u16) -> Self {
        Self {
            msg_type: "connect".to_string(),
            session_type: Some("local".to_string()),
            session_id: None,
            host: None,
            port: None,
            user: None,
            password: None,
            auth_method: None,
            ssh_key_id: None,
            passphrase: None,
            connection_id: Some("local".to_string()),
            cwd: None,
            cols,
            rows,
            term: "xterm-256color".to_string(),
        }
    }

    /// Create a request for a quick-connect SSH session.
    pub fn for_quick_connect(
        host: impl Into<String>,
        port: u16,
        user: impl Into<String>,
        password: impl Into<String>,
        cols: u16,
        rows: u16,
    ) -> Self {
        Self {
            msg_type: "connect".to_string(),
            session_type: Some("ssh".to_string()),
            session_id: None,
            host: Some(host.into()),
            port: Some(port),
            user: Some(user.into()),
            password: Some(password.into()),
            auth_method: Some("password".to_string()),
            ssh_key_id: None,
            passphrase: None,
            connection_id: None,
            cwd: None,
            cols,
            rows,
            term: "xterm-256color".to_string(),
        }
    }

    /// Set an initial working directory for tab duplication.
    pub fn with_cwd(mut self, cwd: impl Into<String>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }
}

/// Request to attach / reconnect to an existing active backend session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsAttachRequest {
    #[serde(rename = "type")]
    pub msg_type: String, // "attach"
    pub session_id: String,
}

impl WsAttachRequest {
    pub fn new(session_id: impl Into<String>) -> Self {
        Self {
            msg_type: "attach".to_string(),
            session_id: session_id.into(),
        }
    }
}

/// Structured response message from backend to client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsServerResponse {
    #[serde(rename = "type")]
    pub msg_type: String, // "connected", "error", "disconnected", "cwd"
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
}

/// Status of the terminal WebSocket transport connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum WsStatus {
    Connecting = 0,
    Connected = 1,
    Disconnected = 2,
    Error = 3,
}

impl From<u8> for WsStatus {
    fn from(val: u8) -> Self {
        match val {
            0 => WsStatus::Connecting,
            1 => WsStatus::Connected,
            2 => WsStatus::Disconnected,
            _ => WsStatus::Error,
        }
    }
}

/// Internal outbound message queue variants.
#[derive(Debug)]
enum WsOutbound {
    Binary(Vec<u8>),
    Resize { cols: u16, rows: u16 },
    GetCwd,
    Disconnect,
}

/// Handle to an active WebSocket terminal connection.
#[derive(Debug, Clone)]
pub struct TerminalWsHandle {
    session_id: Arc<str>,
    outbound_tx: flume::Sender<WsOutbound>,
    inbound_data_rx: flume::Receiver<Vec<u8>>,
    inbound_ctrl_rx: flume::Receiver<WsServerResponse>,
    status: Arc<AtomicU8>,
}

impl TerminalWsHandle {
    /// Return the unique backend session ID.
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Return the current connection status.
    pub fn status(&self) -> WsStatus {
        WsStatus::from(self.status.load(Ordering::SeqCst))
    }

    /// Return true if the transport is actively connected.
    pub fn is_connected(&self) -> bool {
        self.status() == WsStatus::Connected
    }

    /// Send raw keyboard or mouse bytes to the backend PTY.
    pub fn send_input(&self, data: Vec<u8>) -> Result<(), TerminalWsError> {
        if !self.is_connected() {
            return Err(TerminalWsError::Disconnected);
        }
        self.outbound_tx
            .send(WsOutbound::Binary(data))
            .map_err(|e| TerminalWsError::ChannelSend(e.to_string()))
    }

    /// Notify the backend PTY of a terminal dimension change.
    pub fn resize(&self, cols: u16, rows: u16) -> Result<(), TerminalWsError> {
        if !self.is_connected() {
            return Err(TerminalWsError::Disconnected);
        }
        self.outbound_tx
            .send(WsOutbound::Resize { cols, rows })
            .map_err(|e| TerminalWsError::ChannelSend(e.to_string()))
    }

    /// Request the current remote working directory.
    pub fn get_cwd(&self) -> Result<(), TerminalWsError> {
        if !self.is_connected() {
            return Err(TerminalWsError::Disconnected);
        }
        self.outbound_tx
            .send(WsOutbound::GetCwd)
            .map_err(|e| TerminalWsError::ChannelSend(e.to_string()))
    }

    /// Explicitly terminate the session on the backend.
    pub fn disconnect(&self) -> Result<(), TerminalWsError> {
        self.status
            .store(WsStatus::Disconnected as u8, Ordering::SeqCst);
        let _ = self.outbound_tx.send(WsOutbound::Disconnect);
        Ok(())
    }

    /// Receiver for raw binary bytes emitted by the backend PTY.
    pub fn output_receiver(&self) -> flume::Receiver<Vec<u8>> {
        self.inbound_data_rx.clone()
    }

    /// Receiver for control messages (e.g., CWD responses, server errors).
    pub fn control_receiver(&self) -> flume::Receiver<WsServerResponse> {
        self.inbound_ctrl_rx.clone()
    }
}

/// Client for initiating and managing WebSocket terminal sessions.
pub struct TerminalWsClient;

impl TerminalWsClient {
    /// Connect to a new terminal session (SSH or local).
    pub async fn connect(
        url: &str,
        request: WsConnectRequest,
    ) -> Result<TerminalWsHandle, TerminalWsError> {
        let ws_url = normalize_ws_url(url);
        let (ws_stream, _) = connect_async(&ws_url).await.map_err(|source| {
            TerminalWsError::Connect {
                url: ws_url.clone(),
                source: Box::new(source),
            }
        })?;

        let (mut ws_write, mut ws_read) = ws_stream.split();

        // 1. Send connect handshake message
        let handshake_json = serde_json::to_string(&request)?;
        ws_write
            .send(Message::Text(handshake_json.into()))
            .await
            .map_err(|e| TerminalWsError::Protocol(format!("Failed to send connect handshake: {e}")))?;

        // 2. Await connected response
        let session_id = Self::await_connected(&mut ws_read).await?;

        // 3. Send "ready" control frame to trigger backend scrollback replay
        let ready_json = serde_json::to_string(&serde_json::json!({ "type": "ready" }))?;
        ws_write
            .send(Message::Text(ready_json.into()))
            .await
            .map_err(|e| TerminalWsError::Protocol(format!("Failed to send ready frame: {e}")))?;

        // 4. Setup message pumps
        Ok(Self::spawn_pumps(session_id, ws_write, ws_read))
    }

    /// Attach / reconnect to an existing session by its session ID.
    pub async fn attach(
        url: &str,
        session_id: &str,
    ) -> Result<TerminalWsHandle, TerminalWsError> {
        let ws_url = normalize_ws_url(url);
        let (ws_stream, _) = connect_async(&ws_url).await.map_err(|source| {
            TerminalWsError::Connect {
                url: ws_url.clone(),
                source: Box::new(source),
            }
        })?;

        let (mut ws_write, mut ws_read) = ws_stream.split();

        // 1. Send attach handshake message
        let attach_req = WsAttachRequest::new(session_id);
        let handshake_json = serde_json::to_string(&attach_req)?;
        ws_write
            .send(Message::Text(handshake_json.into()))
            .await
            .map_err(|e| TerminalWsError::Protocol(format!("Failed to send attach handshake: {e}")))?;

        // 2. Await connected response
        let verified_session_id = Self::await_connected(&mut ws_read).await?;

        // 3. Send "ready" control frame to trigger backend scrollback replay
        let ready_json = serde_json::to_string(&serde_json::json!({ "type": "ready" }))?;
        ws_write
            .send(Message::Text(ready_json.into()))
            .await
            .map_err(|e| TerminalWsError::Protocol(format!("Failed to send ready frame: {e}")))?;

        // 4. Setup message pumps
        Ok(Self::spawn_pumps(verified_session_id, ws_write, ws_read))
    }

    async fn await_connected<S>(read_stream: &mut S) -> Result<String, TerminalWsError>
    where
        S: StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin,
    {
        let msg = read_stream
            .next()
            .await
            .ok_or_else(|| {
                TerminalWsError::Protocol("Connection closed before receiving handshake response".into())
            })?
            .map_err(|e| TerminalWsError::Protocol(format!("Handshake read error: {e}")))?;

        let text = match msg {
            Message::Text(t) => t,
            other => {
                return Err(TerminalWsError::Protocol(format!(
                    "Expected text handshake response, received: {other:?}"
                )))
            }
        };

        let response: WsServerResponse = serde_json::from_str(&text).map_err(|e| {
            TerminalWsError::Protocol(format!(
                "Failed to parse handshake response: {e}, raw text: {text}"
            ))
        })?;

        if response.msg_type == "error" {
            let error_msg = response
                .message
                .unwrap_or_else(|| "Unknown backend error during connection handshake".into());
            return Err(TerminalWsError::Protocol(error_msg));
        }

        if response.msg_type != "connected" {
            return Err(TerminalWsError::Protocol(format!(
                "Unexpected handshake message type '{}', expected 'connected'",
                response.msg_type
            )));
        }

        response.session_id.ok_or_else(|| {
            TerminalWsError::Protocol("Connected message missing session_id".into())
        })
    }

    fn spawn_pumps<W, R>(session_id: String, mut ws_write: W, mut ws_read: R) -> TerminalWsHandle
    where
        W: SinkExt<Message> + Unpin + Send + 'static,
        W::Error: std::fmt::Display,
        R: StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>>
            + Unpin
            + Send
            + 'static,
    {
        let (outbound_tx, outbound_rx) = flume::unbounded();
        let (inbound_data_tx, inbound_data_rx) = flume::unbounded();
        let (inbound_ctrl_tx, inbound_ctrl_rx) = flume::unbounded();

        let status = Arc::new(AtomicU8::new(WsStatus::Connected as u8));
        let status_write = Arc::clone(&status);
        let status_read = Arc::clone(&status);

        // Task 1: Write pump (Outbound)
        tokio::spawn(async move {
            while let Ok(msg) = outbound_rx.recv_async().await {
                let ws_msg = match msg {
                    WsOutbound::Binary(bytes) => Message::Binary(bytes.into()),
                    WsOutbound::Resize { cols, rows } => {
                        let json = serde_json::to_string(&serde_json::json!({
                            "type": "resize",
                            "cols": cols,
                            "rows": rows,
                        }))
                        .unwrap_or_default();
                        Message::Text(json.into())
                    }
                    WsOutbound::GetCwd => {
                        let json = serde_json::to_string(&serde_json::json!({
                            "type": "get-cwd",
                        }))
                        .unwrap_or_default();
                        Message::Text(json.into())
                    }
                    WsOutbound::Disconnect => {
                        let json = serde_json::to_string(&serde_json::json!({
                            "type": "disconnect",
                        }))
                        .unwrap_or_default();
                        let _ = ws_write.send(Message::Text(json.into())).await;
                        let _ = ws_write.send(Message::Close(None)).await;
                        break;
                    }
                };

                if ws_write.send(ws_msg).await.is_err() {
                    break;
                }
            }
            status_write.store(WsStatus::Disconnected as u8, Ordering::SeqCst);
            let _ = ws_write.close().await;
        });

        // Task 2: Read pump (Inbound)
        tokio::spawn(async move {
            while let Some(msg_res) = ws_read.next().await {
                match msg_res {
                    Ok(Message::Binary(bytes)) => {
                        if inbound_data_tx.send_async(bytes.to_vec()).await.is_err() {
                            break;
                        }
                    }
                    Ok(Message::Text(text)) => {
                        if let Ok(server_msg) = serde_json::from_str::<WsServerResponse>(&text) {
                            let is_fatal = server_msg.msg_type == "error"
                                || server_msg.msg_type == "disconnected";
                            let _ = inbound_ctrl_tx.send_async(server_msg).await;
                            if is_fatal {
                                status_read.store(WsStatus::Disconnected as u8, Ordering::SeqCst);
                                break;
                            }
                        }
                    }
                    Ok(Message::Ping(_payload)) => {
                        // Protocol ping handled automatically by underlying tungstenite connection
                    }
                    Ok(Message::Close(_)) => {
                        status_read.store(WsStatus::Disconnected as u8, Ordering::SeqCst);
                        break;
                    }
                    Err(_) => {
                        status_read.store(WsStatus::Error as u8, Ordering::SeqCst);
                        break;
                    }
                    _ => {}
                }
            }
            status_read.store(WsStatus::Disconnected as u8, Ordering::SeqCst);
        });

        TerminalWsHandle {
            session_id: session_id.into(),
            outbound_tx,
            inbound_data_rx,
            inbound_ctrl_rx,
            status,
        }
    }
}
