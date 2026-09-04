use std::collections::HashMap;
use serde::{Deserialize, Serialize};

fn default_port() -> u16 {
    22
}

fn default_auth_method() -> String {
    "password".to_string()
}

/// Settings map returned by GET /api/settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Settings {
    pub settings: HashMap<String, String>,
}

/// Connection definition mirrored from the backend db.Connection model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Connection {
    pub id: String,
    pub label: String,
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    pub username: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_auth_method")]
    pub auth_method: String,
    #[serde(default)]
    pub ssh_key_id: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// Active or detached session returned by GET /api/sessions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    #[serde(rename = "type")]
    pub session_type: String, // "ssh" | "local"
    pub host: String,
    pub user: String,
    pub port: u16,
    #[serde(default)]
    pub connection_id: Option<String>,
    pub status: String, // "active" | "detached"
    #[serde(default)]
    pub cwd: Option<String>,
}

/// SSH Key definition mirrored from backend db.SSHKey model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SshKey {
    pub id: String,
    pub name: String,
    pub key_type: String,
    pub fingerprint: String,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// Request body for POST /api/keys.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateKeyRequest {
    pub name: String,
    pub key_base64: String,
}

/// Request body for POST /api/connections.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateConnectionRequest {
    pub label: String,
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_auth_method")]
    pub auth_method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh_key_id: Option<String>,
}

/// Request body for PUT /api/connections/:id.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateConnectionRequest {
    pub label: String,
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_auth_method")]
    pub auth_method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh_key_id: Option<String>,
}

/// Response from POST /api/connections/import.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportResult {
    pub imported: usize,
    pub skipped: usize,
}

/// Information about a remote or local file in SFTP browsing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SftpFileInfo {
    pub name: String,
    #[serde(default)]
    pub size: i64,
    #[serde(default)]
    pub mode: u32,
    #[serde(rename = "modTime", default)]
    pub mod_time: String,
    #[serde(rename = "isDir", default)]
    pub is_dir: bool,
}

/// Status of an active or completed background file transfer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SftpTransferStatus {
    pub id: String,
    #[serde(default)]
    pub bytes_transferred: i64,
    #[serde(default)]
    pub total_bytes: i64,
    #[serde(default)]
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

fn default_forward_type() -> String {
    "local".to_string()
}

/// Port Forward definition returned by GET /api/forwards.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PortForward {
    pub id: String,
    pub name: String,
    pub connection_id: String,
    pub local_port: u16,
    pub remote_port: u16,
    #[serde(rename = "type", default = "default_forward_type")]
    pub forward_type: String, // "local" | "reverse"
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub auto_start: bool,
    #[serde(default)]
    pub error: String,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

impl PortForward {
    pub fn is_reverse(&self) -> bool {
        self.forward_type == "reverse"
    }

    pub fn mapping_display(&self) -> String {
        if self.is_reverse() {
            format!(":{} \u{2190} localhost:{}", self.remote_port, self.local_port)
        } else {
            format!("localhost:{} \u{2192} :{}", self.local_port, self.remote_port)
        }
    }
}

/// Request body for POST /api/forwards.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateForwardRequest {
    pub name: String,
    pub connection_id: String,
    pub local_port: u16,
    pub remote_port: u16,
    #[serde(rename = "type", default = "default_forward_type")]
    pub forward_type: String, // "local" | "reverse"
}

/// Request body for PUT /api/forwards/:id.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateForwardRequest {
    pub name: String,
    pub connection_id: String,
    pub local_port: u16,
    pub remote_port: u16,
    #[serde(rename = "type", default = "default_forward_type")]
    pub forward_type: String, // "local" | "reverse"
}

/// Response returned by POST /api/forwards/:id/start or /stop.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForwardActionResponse {
    pub status: String,
}

