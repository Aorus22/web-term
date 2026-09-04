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
