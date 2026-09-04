use std::time::Duration;
use thiserror::Error;
use crate::terminal_ws::{normalize_ws_url, TerminalWsClient, TerminalWsError, TerminalWsHandle, WsConnectRequest};
use crate::types::{
    Connection, CreateConnectionRequest, CreateForwardRequest, CreateKeyRequest,
    ForwardActionResponse, ImportResult, PortForward, SessionInfo, Settings, SftpFileInfo,
    SftpTransferStatus, SshKey, UpdateConnectionRequest, UpdateForwardRequest,
};

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
    #[error("WebSocket terminal error: {0}")]
    WebSocket(#[from] TerminalWsError),
}

/// Client for communicating with the local WebTerm backend REST and WebSocket APIs.
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

    /// Fetch single connection from GET /api/connections/:id.
    pub async fn get_connection(&self, id: &str) -> Result<Connection, ClientError> {
        let url = format!("{}/api/connections/{}", self.base_url, id);
        let resp = self.http.get(&url).send().await.map_err(|e| ClientError::Request {
            url: url.clone(),
            source: e,
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ClientError::Status { url, status, body });
        }

        resp.json::<Connection>().await.map_err(|e| ClientError::Decode {
            url,
            source: e,
        })
    }

    /// Create new connection via POST /api/connections.
    pub async fn create_connection(
        &self,
        req: &CreateConnectionRequest,
    ) -> Result<Connection, ClientError> {
        let url = format!("{}/api/connections", self.base_url);
        let resp = self
            .http
            .post(&url)
            .json(req)
            .send()
            .await
            .map_err(|e| ClientError::Request {
                url: url.clone(),
                source: e,
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ClientError::Status { url, status, body });
        }

        resp.json::<Connection>().await.map_err(|e| ClientError::Decode {
            url,
            source: e,
        })
    }

    /// Update existing connection via PUT /api/connections/:id.
    pub async fn update_connection(
        &self,
        id: &str,
        req: &UpdateConnectionRequest,
    ) -> Result<Connection, ClientError> {
        let url = format!("{}/api/connections/{}", self.base_url, id);
        let resp = self
            .http
            .put(&url)
            .json(req)
            .send()
            .await
            .map_err(|e| ClientError::Request {
                url: url.clone(),
                source: e,
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ClientError::Status { url, status, body });
        }

        resp.json::<Connection>().await.map_err(|e| ClientError::Decode {
            url,
            source: e,
        })
    }

    /// Delete connection via DELETE /api/connections/:id.
    pub async fn delete_connection(&self, id: &str) -> Result<(), ClientError> {
        let url = format!("{}/api/connections/{}", self.base_url, id);
        let resp = self.http.delete(&url).send().await.map_err(|e| ClientError::Request {
            url: url.clone(),
            source: e,
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ClientError::Status { url, status, body });
        }

        Ok(())
    }

    /// Export all connections via GET /api/connections/export.
    pub async fn export_connections(&self) -> Result<Vec<Connection>, ClientError> {
        let url = format!("{}/api/connections/export", self.base_url);
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

    /// Import connections via POST /api/connections/import.
    pub async fn import_connections(
        &self,
        req: &[Connection],
    ) -> Result<ImportResult, ClientError> {
        let url = format!("{}/api/connections/import", self.base_url);
        let resp = self
            .http
            .post(&url)
            .json(req)
            .send()
            .await
            .map_err(|e| ClientError::Request {
                url: url.clone(),
                source: e,
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ClientError::Status { url, status, body });
        }

        resp.json::<ImportResult>().await.map_err(|e| ClientError::Decode {
            url,
            source: e,
        })
    }

    /// Fetch SSH keys list from GET /api/keys.
    pub async fn list_keys(&self) -> Result<Vec<SshKey>, ClientError> {
        let url = format!("{}/api/keys", self.base_url);
        let resp = self.http.get(&url).send().await.map_err(|e| ClientError::Request {
            url: url.clone(),
            source: e,
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ClientError::Status { url, status, body });
        }

        resp.json::<Vec<SshKey>>().await.map_err(|e| ClientError::Decode {
            url,
            source: e,
        })
    }

    /// Fetch single SSH key from GET /api/keys/:id.
    pub async fn get_key(&self, id: &str) -> Result<SshKey, ClientError> {
        let url = format!("{}/api/keys/{}", self.base_url, id);
        let resp = self.http.get(&url).send().await.map_err(|e| ClientError::Request {
            url: url.clone(),
            source: e,
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ClientError::Status { url, status, body });
        }

        resp.json::<SshKey>().await.map_err(|e| ClientError::Decode {
            url,
            source: e,
        })
    }

    /// Create new SSH key via POST /api/keys.
    pub async fn create_key(&self, req: &CreateKeyRequest) -> Result<SshKey, ClientError> {
        let url = format!("{}/api/keys", self.base_url);
        let resp = self
            .http
            .post(&url)
            .json(req)
            .send()
            .await
            .map_err(|e| ClientError::Request {
                url: url.clone(),
                source: e,
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ClientError::Status { url, status, body });
        }

        resp.json::<SshKey>().await.map_err(|e| ClientError::Decode {
            url,
            source: e,
        })
    }

    /// Delete SSH key via DELETE /api/keys/:id.
    pub async fn delete_key(&self, id: &str) -> Result<(), ClientError> {
        let url = format!("{}/api/keys/{}", self.base_url, id);
        let resp = self.http.delete(&url).send().await.map_err(|e| ClientError::Request {
            url: url.clone(),
            source: e,
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ClientError::Status { url, status, body });
        }

        Ok(())
    }

    /// Fetch active backend sessions from GET /api/sessions.
    pub async fn list_sessions(&self) -> Result<Vec<SessionInfo>, ClientError> {
        let url = format!("{}/api/sessions", self.base_url);
        let resp = self.http.get(&url).send().await.map_err(|e| ClientError::Request {
            url: url.clone(),
            source: e,
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ClientError::Status { url, status, body });
        }

        resp.json::<Vec<SessionInfo>>().await.map_err(|e| ClientError::Decode {
            url,
            source: e,
        })
    }

    /// Terminate an active backend session via DELETE /api/sessions/:id.
    pub async fn delete_session(&self, session_id: &str) -> Result<(), ClientError> {
        let url = format!("{}/api/sessions/{}", self.base_url, session_id);
        let resp = self.http.delete(&url).send().await.map_err(|e| ClientError::Request {
            url: url.clone(),
            source: e,
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ClientError::Status { url, status, body });
        }

        Ok(())
    }

    /// Open a WebSocket terminal connection to the backend.
    pub async fn connect_terminal(
        &self,
        request: WsConnectRequest,
    ) -> Result<TerminalWsHandle, ClientError> {
        let ws_url = normalize_ws_url(&self.base_url);
        let handle = TerminalWsClient::connect(&ws_url, request).await?;
        Ok(handle)
    }

    /// Re-attach to an existing backend terminal session by session ID.
    pub async fn attach_terminal(
        &self,
        session_id: &str,
    ) -> Result<TerminalWsHandle, ClientError> {
        let ws_url = normalize_ws_url(&self.base_url);
        let handle = TerminalWsClient::attach(&ws_url, session_id).await?;
        Ok(handle)
    }

    /// List files in a directory via GET /api/sftp/ls?connectionId={id}&path={path}.
    pub async fn sftp_list(
        &self,
        connection_id: &str,
        path: &str,
    ) -> Result<Vec<SftpFileInfo>, ClientError> {
        let url = format!("{}/api/sftp/ls", self.base_url);
        let resp = self
            .http
            .get(&url)
            .query(&[("connectionId", connection_id), ("path", path)])
            .send()
            .await
            .map_err(|e| ClientError::Request {
                url: url.clone(),
                source: e,
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ClientError::Status { url, status, body });
        }

        resp.json::<Option<Vec<SftpFileInfo>>>()
            .await
            .map(|opt| opt.unwrap_or_default())
            .map_err(|e| ClientError::Decode {
                url,
                source: e,
            })
    }

    /// Create a directory via POST /api/sftp/mkdir?connectionId={id}&path={path}.
    pub async fn sftp_mkdir(
        &self,
        connection_id: &str,
        path: &str,
    ) -> Result<(), ClientError> {
        let url = format!("{}/api/sftp/mkdir", self.base_url);
        let resp = self
            .http
            .post(&url)
            .query(&[("connectionId", connection_id), ("path", path)])
            .send()
            .await
            .map_err(|e| ClientError::Request {
                url: url.clone(),
                source: e,
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ClientError::Status { url, status, body });
        }

        Ok(())
    }

    /// Rename or move a file/directory via POST /api/sftp/rename?connectionId={id}&oldPath={old}&newPath={new}.
    pub async fn sftp_rename(
        &self,
        connection_id: &str,
        old_path: &str,
        new_path: &str,
    ) -> Result<(), ClientError> {
        let url = format!("{}/api/sftp/rename", self.base_url);
        let resp = self
            .http
            .post(&url)
            .query(&[
                ("connectionId", connection_id),
                ("oldPath", old_path),
                ("newPath", new_path),
            ])
            .send()
            .await
            .map_err(|e| ClientError::Request {
                url: url.clone(),
                source: e,
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ClientError::Status { url, status, body });
        }

        Ok(())
    }

    /// Remove a file or directory via DELETE /api/sftp/remove?connectionId={id}&path={path}.
    pub async fn sftp_remove(
        &self,
        connection_id: &str,
        path: &str,
    ) -> Result<(), ClientError> {
        let url = format!("{}/api/sftp/remove", self.base_url);
        let resp = self
            .http
            .delete(&url)
            .query(&[("connectionId", connection_id), ("path", path)])
            .send()
            .await
            .map_err(|e| ClientError::Request {
                url: url.clone(),
                source: e,
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ClientError::Status { url, status, body });
        }

        Ok(())
    }

    /// Download a file via GET /api/sftp/download?connectionId={id}&path={path}.
    pub async fn sftp_download(
        &self,
        connection_id: &str,
        path: &str,
    ) -> Result<Vec<u8>, ClientError> {
        let url = format!("{}/api/sftp/download", self.base_url);
        let resp = self
            .http
            .get(&url)
            .query(&[("connectionId", connection_id), ("path", path)])
            .send()
            .await
            .map_err(|e| ClientError::Request {
                url: url.clone(),
                source: e,
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ClientError::Status { url, status, body });
        }

        let bytes = resp.bytes().await.map_err(|e| ClientError::Decode {
            url,
            source: e,
        })?;
        Ok(bytes.to_vec())
    }

    /// Upload a file via multipart POST /api/sftp/upload?connectionId={id}&path={path}.
    /// Returns the background transfer ID.
    pub async fn sftp_upload(
        &self,
        connection_id: &str,
        path: &str,
        filename: &str,
        data: Vec<u8>,
    ) -> Result<String, ClientError> {
        let url = format!("{}/api/sftp/upload", self.base_url);
        let part = reqwest::multipart::Part::bytes(data).file_name(filename.to_string());
        let form = reqwest::multipart::Form::new().part("file", part);

        let full_path = if path.ends_with(filename) {
            path.to_string()
        } else if path.ends_with('/') || path.ends_with('\\') {
            format!("{}{}", path, filename)
        } else {
            format!("{}/{}", path, filename)
        };

        let resp = self
            .http
            .post(&url)
            .query(&[("connectionId", connection_id), ("path", &full_path)])
            .multipart(form)
            .send()
            .await
            .map_err(|e| ClientError::Request {
                url: url.clone(),
                source: e,
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ClientError::Status { url, status, body });
        }

        #[derive(serde::Deserialize)]
        struct UploadResp {
            #[serde(rename = "transferId")]
            transfer_id: String,
        }

        let parsed = resp.json::<UploadResp>().await.map_err(|e| ClientError::Decode {
            url,
            source: e,
        })?;
        Ok(parsed.transfer_id)
    }

    /// Query all active and recent background transfers via GET /api/sftp/transfers.
    pub async fn sftp_list_transfers(&self) -> Result<Vec<SftpTransferStatus>, ClientError> {
        let url = format!("{}/api/sftp/transfers", self.base_url);
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| ClientError::Request {
                url: url.clone(),
                source: e,
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ClientError::Status { url, status, body });
        }

        resp.json::<Option<Vec<SftpTransferStatus>>>()
            .await
            .map(|opt| opt.unwrap_or_default())
            .map_err(|e| ClientError::Decode {
                url,
                source: e,
            })
    }

    /// Query status of a specific transfer via GET /api/sftp/transfer/status?transferId={id}.
    pub async fn sftp_transfer_status(
        &self,
        transfer_id: &str,
    ) -> Result<SftpTransferStatus, ClientError> {
        let url = format!("{}/api/sftp/transfer/status", self.base_url);
        let resp = self
            .http
            .get(&url)
            .query(&[("transferId", transfer_id)])
            .send()
            .await
            .map_err(|e| ClientError::Request {
                url: url.clone(),
                source: e,
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ClientError::Status { url, status, body });
        }

        resp.json::<SftpTransferStatus>().await.map_err(|e| ClientError::Decode {
            url,
            source: e,
        })
    }

    /// Get the home directory path via GET /api/sftp/home?connectionId={id}.
    pub async fn sftp_home(&self, connection_id: &str) -> Result<String, ClientError> {
        let url = format!("{}/api/sftp/home", self.base_url);
        let resp = self
            .http
            .get(&url)
            .query(&[("connectionId", connection_id)])
            .send()
            .await
            .map_err(|e| ClientError::Request {
                url: url.clone(),
                source: e,
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ClientError::Status { url, status, body });
        }

        #[derive(serde::Deserialize)]
        struct HomeResp {
            path: String,
        }

        let parsed = resp.json::<HomeResp>().await.map_err(|e| ClientError::Decode {
            url,
            source: e,
        })?;
        Ok(parsed.path)
    }

    // ==========================================
    // Port Forwarding Endpoints
    // ==========================================

    /// List all port forwards via GET /api/forwards.
    pub async fn list_forwards(&self) -> Result<Vec<PortForward>, ClientError> {
        let url = format!("{}/api/forwards", self.base_url);
        let resp = self.http.get(&url).send().await.map_err(|e| ClientError::Request {
            url: url.clone(),
            source: e,
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ClientError::Status { url, status, body });
        }

        // Deserialize as Option<Vec<PortForward>> to handle Go null slice gracefully
        let forwards = resp
            .json::<Option<Vec<PortForward>>>()
            .await
            .map_err(|e| ClientError::Decode {
                url,
                source: e,
            })?;
        Ok(forwards.unwrap_or_default())
    }

    /// Create a new port forward rule via POST /api/forwards.
    pub async fn create_forward(
        &self,
        req: &CreateForwardRequest,
    ) -> Result<PortForward, ClientError> {
        let url = format!("{}/api/forwards", self.base_url);
        let resp = self
            .http
            .post(&url)
            .json(req)
            .send()
            .await
            .map_err(|e| ClientError::Request {
                url: url.clone(),
                source: e,
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ClientError::Status { url, status, body });
        }

        resp.json::<PortForward>().await.map_err(|e| ClientError::Decode {
            url,
            source: e,
        })
    }

    /// Update an existing port forward rule via PUT /api/forwards/:id.
    pub async fn update_forward(
        &self,
        id: &str,
        req: &UpdateForwardRequest,
    ) -> Result<PortForward, ClientError> {
        let url = format!("{}/api/forwards/{}", self.base_url, id);
        let resp = self
            .http
            .put(&url)
            .json(req)
            .send()
            .await
            .map_err(|e| ClientError::Request {
                url: url.clone(),
                source: e,
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ClientError::Status { url, status, body });
        }

        resp.json::<PortForward>().await.map_err(|e| ClientError::Decode {
            url,
            source: e,
        })
    }

    /// Delete a port forward rule via DELETE /api/forwards/:id.
    pub async fn delete_forward(&self, id: &str) -> Result<(), ClientError> {
        let url = format!("{}/api/forwards/{}", self.base_url, id);
        let resp = self.http.delete(&url).send().await.map_err(|e| ClientError::Request {
            url: url.clone(),
            source: e,
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ClientError::Status { url, status, body });
        }

        Ok(())
    }

    /// Start a port forward tunnel via POST /api/forwards/:id/start.
    pub async fn start_forward(&self, id: &str) -> Result<ForwardActionResponse, ClientError> {
        let url = format!("{}/api/forwards/{}/start", self.base_url, id);
        let resp = self.http.post(&url).send().await.map_err(|e| ClientError::Request {
            url: url.clone(),
            source: e,
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ClientError::Status { url, status, body });
        }

        resp.json::<ForwardActionResponse>()
            .await
            .map_err(|e| ClientError::Decode {
                url,
                source: e,
            })
    }

    /// Stop an active port forward tunnel via POST /api/forwards/:id/stop.
    pub async fn stop_forward(&self, id: &str) -> Result<ForwardActionResponse, ClientError> {
        let url = format!("{}/api/forwards/{}/stop", self.base_url, id);
        let resp = self.http.post(&url).send().await.map_err(|e| ClientError::Request {
            url: url.clone(),
            source: e,
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ClientError::Status { url, status, body });
        }

        resp.json::<ForwardActionResponse>()
            .await
            .map_err(|e| ClientError::Decode {
                url,
                source: e,
            })
    }
}


