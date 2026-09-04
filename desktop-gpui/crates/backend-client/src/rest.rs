use std::time::Duration;
use thiserror::Error;
use crate::types::{Connection, Settings};

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("HTTP request error ({url}): {source}")]
    Request {
        url: String,
        #[source]
        source: reqwest::Error,
    },
    #[error("Server returned error status {status} for {url}: {body}")]
    Status {
        url: String,
        status: reqwest::StatusCode,
        body: String,
    },
    #[error("Failed to parse response from {url}: {source}")]
    Decode {
        url: String,
        #[source]
        source: reqwest::Error,
    },
}

/// Client for communicating with the local WebTerm backend REST API.
#[derive(Debug, Clone)]
pub struct BackendClient {
    base_url: String,
    http: reqwest::Client,
}

impl BackendClient {
    /// Create a new BackendClient for the given base URL (e.g. "http://127.0.0.1:54321").
    pub fn new(base_url: impl Into<String>) -> Self {
        let base_url = base_url.into().trim_end_matches('/').to_string();
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap_or_default();

        Self { base_url, http }
    }

    /// Return the configured base URL.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Check if the backend is reachable and healthy with a 1-second timeout.
    pub async fn is_ready(&self) -> bool {
        let url = format!("{}/api/settings", self.base_url);
        let check_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(1))
            .build()
            .unwrap_or_default();

        match check_client.get(&url).send().await {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }

    /// Fetch system settings from GET /api/settings.
    pub async fn get_settings(&self) -> Result<Settings, ClientError> {
        let url = format!("{}/api/settings", self.base_url);
        let resp = self.http.get(&url).send().await.map_err(|e| ClientError::Request {
            url: url.clone(),
            source: e,
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ClientError::Status { url, status, body });
        }

        resp.json::<Settings>().await.map_err(|e| ClientError::Decode {
            url,
            source: e,
        })
    }

    /// Fetch connections list from GET /api/connections.
    pub async fn list_connections(&self) -> Result<Vec<Connection>, ClientError> {
        let url = format!("{}/api/connections", self.base_url);
        let resp = self.http.get(&url).send().await.map_err(|e| ClientError::Request {
            url: url.clone(),
            source: e,
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ClientError::Status { url, status, body });
        }

        resp.json::<Vec<Connection>>().await.map_err(|e| ClientError::Decode {
            url,
            source: e,
        })
    }
}
