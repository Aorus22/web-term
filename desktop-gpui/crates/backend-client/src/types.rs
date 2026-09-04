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
