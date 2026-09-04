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
///
/// Fields match JSON tags in be/internal/db/models.go.
/// TODO: Expand in Phase 22/23 as connection management and terminal views mature.
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
