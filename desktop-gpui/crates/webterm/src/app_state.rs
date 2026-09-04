//! Root application state, session orchestration, and view routing.

use std::sync::LazyLock;
use gpui::*;
use webterm_backend_client::{
    BackendClient, Connection, CreateConnectionRequest, CreateKeyRequest, ImportResult, SshKey,
    TerminalWsHandle, UpdateConnectionRequest, WsConnectRequest,
};
use webterm_settings::{DesktopSettings, SavedSessionTab, Theme as SettingsTheme};
use webterm_supervisor::{BackendInfo, BackendStatus, SpawnOptions, Supervisor};

use crate::session::{SessionStatus, TerminalSessionManager, TerminalTab};
use crate::views::{nav::render_nav_shell, status::render_status_page};

/// Standard RFC 4648 Base64 encoding for PEM payloads.
pub fn encode_base64(bytes: &[u8]) -> String {
    const CHARSET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0];
        let b1 = if chunk.len() > 1 { chunk[1] } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] } else { 0 };

        let n = ((b0 as u32) << 16) | ((b1 as u32) << 8) | (b2 as u32);
        out.push(CHARSET[((n >> 18) & 63) as usize] as char);
        out.push(CHARSET[((n >> 12) & 63) as usize] as char);
        if chunk.len() > 1 {
            out.push(CHARSET[((n >> 6) & 63) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(CHARSET[(n & 63) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

/// Mode of the Connection modal: Create or Edit with connection ID.
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionModalMode {
    Create,
    Edit(String),
}

/// Form state for Creating or Editing a Connection.
#[derive(Debug, Clone, PartialEq)]
pub struct ConnectionFormState {
    pub mode: ConnectionModalMode,
    pub label: String,
    pub host: String,
    pub port: String,
    pub username: String,
    pub password: String,
    pub tags: String,
    pub auth_method: String,
    pub ssh_key_id: Option<String>,
    pub error_message: Option<String>,
}

impl ConnectionFormState {
    pub fn new_create() -> Self {
        Self {
            mode: ConnectionModalMode::Create,
            label: String::new(),
            host: String::new(),
            port: "22".to_string(),
            username: "root".to_string(),
            password: String::new(),
            tags: String::new(),
            auth_method: "password".to_string(),
            ssh_key_id: None,
            error_message: None,
        }
    }

    pub fn new_edit(conn: &Connection) -> Self {
        Self {
            mode: ConnectionModalMode::Edit(conn.id.clone()),
            label: conn.label.clone(),
            host: conn.host.clone(),
            port: conn.port.to_string(),
            username: conn.username.clone(),
            password: String::new(),
            tags: conn.tags.join(", "),
            auth_method: conn.auth_method.clone(),
            ssh_key_id: conn.ssh_key_id.clone(),
            error_message: None,
        }
    }

    pub fn parse_port(&self) -> u16 {
        self.port.trim().parse::<u16>().unwrap_or(22)
    }

    pub fn parse_tags(&self) -> Vec<String> {
        self.tags
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.label.trim().is_empty() {
            return Err("Connection label is required".to_string());
        }
        if self.host.trim().is_empty() {
            return Err("Host / IP address is required".to_string());
        }
        if self.username.trim().is_empty() {
            return Err("Username is required".to_string());
        }
        if self.auth_method == "key" && self.ssh_key_id.is_none() {
            return Err("Please select an SSH key for key authentication".to_string());
        }
        Ok(())
    }

    pub fn to_create_request(&self) -> CreateConnectionRequest {
        CreateConnectionRequest {
            label: self.label.trim().to_string(),
            host: self.host.trim().to_string(),
            port: self.parse_port(),
            username: self.username.trim().to_string(),
            password: if self.auth_method == "password" && !self.password.is_empty() {
                Some(self.password.clone())
            } else {
                None
            },
            tags: self.parse_tags(),
            auth_method: self.auth_method.clone(),
            ssh_key_id: if self.auth_method == "key" {
                self.ssh_key_id.clone()
            } else {
                None
            },
        }
    }

    pub fn to_update_request(&self) -> UpdateConnectionRequest {
        UpdateConnectionRequest {
            label: self.label.trim().to_string(),
            host: self.host.trim().to_string(),
            port: self.parse_port(),
            username: self.username.trim().to_string(),
            password: if self.auth_method == "password" && !self.password.is_empty() {
                Some(self.password.clone())
            } else {
                None
            },
            tags: self.parse_tags(),
            auth_method: self.auth_method.clone(),
            ssh_key_id: if self.auth_method == "key" {
                self.ssh_key_id.clone()
            } else {
                None
            },
        }
    }
}

/// Active view in the main navigation sidebar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Hosts,
    Keys,
    Sftp,
    Settings,
}

pub static TOKIO_RT: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to initialize Tokio runtime for supervisor and WebSocket pumps")
});

enum SupervisorEvent {
    Status(BackendStatus),
    Ready(BackendInfo),
    Failed { reason: String, stderr_tail: String },
}

/// Root application entity holding state, client handle, tabs, and routing views.
pub struct AppState {
    pub backend_status: BackendStatus,
    pub client: Option<BackendClient>,
    pub theme: SettingsTheme,
    pub active_view: View,
    pub settings: DesktopSettings,
    pub spawn_opts: Option<SpawnOptions>,
    pub session_manager: TerminalSessionManager,
    pub show_new_tab_modal: bool,
    pub new_tab_host: String,
    pub new_tab_user: String,
    pub new_tab_port: String,
    pub new_tab_password: String,
    pub show_hosts_catalog: bool,
    pub connections: Vec<Connection>,
    pub search_query: String,
    pub selected_tag: Option<String>,
    pub is_loading_connections: bool,
    pub ssh_keys: Vec<SshKey>,
    pub is_loading_keys: bool,
    pub connection_modal: Option<ConnectionFormState>,
    pub show_import_modal: bool,
    pub import_payload: String,
    pub notification: Option<String>,
    pub show_add_key_modal: bool,
    pub new_key_name: String,
    pub new_key_pem: String,
    pub add_key_error: Option<String>,
    pub pending_passphrase_conn: Option<(String, String)>,
    pub passphrase_input: String,
    pub passphrase_error: Option<String>,
    pub passphrase_cache: std::collections::HashMap<String, String>,
}

impl AppState {
    /// Create initial AppState from desktop settings.
    pub fn new(settings: DesktopSettings, spawn_opts: Option<SpawnOptions>) -> Self {
        let theme = settings.theme;
        Self {
            backend_status: BackendStatus::Starting,
            client: None,
            theme,
            active_view: View::Hosts,
            settings,
            spawn_opts,
            session_manager: TerminalSessionManager::new(),
            show_new_tab_modal: false,
            new_tab_host: String::new(),
            new_tab_user: "root".to_string(),
            new_tab_port: "22".to_string(),
            new_tab_password: String::new(),
            show_hosts_catalog: true,
            connections: Vec::new(),
            search_query: String::new(),
            selected_tag: None,
            is_loading_connections: false,
            ssh_keys: Vec::new(),
            is_loading_keys: false,
            connection_modal: None,
            show_import_modal: false,
            import_payload: String::new(),
            notification: None,
            show_add_key_modal: false,
            new_key_name: String::new(),
            new_key_pem: String::new(),
            add_key_error: None,
            pending_passphrase_conn: None,
            passphrase_input: String::new(),
            passphrase_error: None,
            passphrase_cache: std::collections::HashMap::new(),
        }
    }

    /// Toggle new tab launcher modal.
    pub fn toggle_new_tab_modal(&mut self, cx: &mut Context<Self>) {
        self.show_new_tab_modal = !self.show_new_tab_modal;
        cx.notify();
    }

    /// Close new tab launcher modal.
    pub fn close_new_tab_modal(&mut self, cx: &mut Context<Self>) {
        self.show_new_tab_modal = false;
        cx.notify();
    }

    /// Open a quick SSH tab from the new tab launcher modal inputs.
    pub fn open_quick_ssh_tab(&mut self, cx: &mut Context<Self>) {
        self.show_new_tab_modal = false;
        let host = if self.new_tab_host.trim().is_empty() {
            "localhost".to_string()
        } else {
            self.new_tab_host.trim().to_string()
        };
        let user = if self.new_tab_user.trim().is_empty() {
            "root".to_string()
        } else {
            self.new_tab_user.trim().to_string()
        };
        let port: u16 = self.new_tab_port.trim().parse().unwrap_or(22);
        let password = if self.new_tab_password.is_empty() {
            String::new()
        } else {
            self.new_tab_password.clone()
        };

        let title = format!("{user}@{host}:{port}");
        let req = WsConnectRequest::for_quick_connect(host, port, user, password, 80, 24);
        self.show_hosts_catalog = false;
        self.open_ssh_tab(req, &title, cx);
    }

    /// Fetch all saved connections from the backend.
    pub fn fetch_connections(&mut self, cx: &mut Context<Self>) {
        let client = match self.client.clone() {
            Some(c) => c,
            None => return,
        };

        self.is_loading_connections = true;
        cx.notify();

        let view_weak = cx.entity().downgrade();
        let (tx, mut rx) =
            tokio::sync::mpsc::unbounded_channel::<Result<Vec<Connection>, String>>();

        TOKIO_RT.spawn(async move {
            match client.list_connections().await {
                Ok(conns) => {
                    let _ = tx.send(Ok(conns));
                }
                Err(e) => {
                    let _ = tx.send(Err(e.to_string()));
                }
            }
        });

        cx.spawn(move |_view, cx: &mut AsyncApp| {
            let view_weak = view_weak.clone();
            let cx_handle = cx.clone();
            async move {
                if let Some(res) = rx.recv().await {
                    cx_handle.update(|cx: &mut App| {
                        if let Some(app) = view_weak.upgrade() {
                            app.update(cx, |this, cx| {
                                this.is_loading_connections = false;
                                match res {
                                    Ok(conns) => {
                                        this.connections = conns;
                                    }
                                    Err(err) => {
                                        eprintln!("Failed to fetch connections: {err}");
                                    }
                                }
                                cx.notify();
                            });
                        }
                    });
                }
            }
        })
        .detach();
    }

    /// Single-click connect to a saved host connection.
    pub fn connect_to_host(&mut self, conn_id: &str, cx: &mut Context<Self>) {
        let conn = match self.connections.iter().find(|c| c.id == conn_id).cloned() {
            Some(c) => c,
            None => return,
        };

        if conn.auth_method == "key" {
            if let Some(key_id) = &conn.ssh_key_id {
                if let Some(cached_pass) = self.passphrase_cache.get(key_id).cloned() {
                    self.connect_to_host_with_passphrase(conn_id, Some(&cached_pass), cx);
                    return;
                }
            }
        }

        self.connect_to_host_with_passphrase(conn_id, None, cx);
    }

    /// Connect to a saved host with an optional key passphrase.
    pub fn connect_to_host_with_passphrase(
        &mut self,
        conn_id: &str,
        passphrase: Option<&str>,
        cx: &mut Context<Self>,
    ) {
        let conn = match self.connections.iter().find(|c| c.id == conn_id).cloned() {
            Some(c) => c,
            None => return,
        };

        let title = format!("{} ({}:{})", conn.label, conn.host, conn.port);
        let mut req = WsConnectRequest::for_saved_connection(&conn.id, 80, 24);
        if let Some(pass) = passphrase {
            req.passphrase = Some(pass.to_string());
        }
        self.show_hosts_catalog = false;
        self.open_ssh_tab(req, &title, cx);
    }

    /// Prompt user for passphrase before connecting with an encrypted SSH key.
    pub fn prompt_passphrase_for_connection(
        &mut self,
        conn_id: &str,
        key_id: &str,
        cx: &mut Context<Self>,
    ) {
        if let Some(cached_pass) = self.passphrase_cache.get(key_id).cloned() {
            self.connect_to_host_with_passphrase(conn_id, Some(&cached_pass), cx);
            return;
        }

        self.pending_passphrase_conn = Some((conn_id.to_string(), key_id.to_string()));
        self.passphrase_input.clear();
        self.passphrase_error = None;
        cx.notify();
    }

    /// Submit passphrase entered in PassphraseModal, caching it in memory for this session.
    pub fn submit_passphrase(&mut self, cx: &mut Context<Self>) {
        let (conn_id, key_id) = match self.pending_passphrase_conn.take() {
            Some(pair) => pair,
            None => return,
        };

        let pass = self.passphrase_input.trim().to_string();
        if !pass.is_empty() {
            self.passphrase_cache.insert(key_id, pass.clone());
        }

        self.passphrase_input.clear();
        self.passphrase_error = None;
        self.connect_to_host_with_passphrase(&conn_id, Some(&pass), cx);
        cx.notify();
    }

    /// Cancel passphrase entry dialog.
    pub fn cancel_passphrase(&mut self, cx: &mut Context<Self>) {
        self.pending_passphrase_conn = None;
        self.passphrase_input.clear();
        self.passphrase_error = None;
        cx.notify();
    }

    /// Open Add SSH Key modal dialog.
    pub fn open_add_key_modal(&mut self, cx: &mut Context<Self>) {
        self.show_add_key_modal = true;
        self.new_key_name.clear();
        self.new_key_pem.clear();
        self.add_key_error = None;
        cx.notify();
    }

    /// Close Add SSH Key modal dialog.
    pub fn close_add_key_modal(&mut self, cx: &mut Context<Self>) {
        self.show_add_key_modal = false;
        self.new_key_name.clear();
        self.new_key_pem.clear();
        self.add_key_error = None;
        cx.notify();
    }

    /// Upload and save new SSH Key to backend pool.
    pub fn create_ssh_key(&mut self, cx: &mut Context<Self>) {
        if self.new_key_name.trim().is_empty() {
            self.add_key_error = Some("Key name is required".to_string());
            cx.notify();
            return;
        }

        if self.new_key_pem.trim().is_empty() {
            self.add_key_error = Some("Private key PEM content is required".to_string());
            cx.notify();
            return;
        }

        self.add_key_error = None;

        let client = match self.client.clone() {
            Some(c) => c,
            None => {
                self.add_key_error = Some("Backend client is not connected".to_string());
                cx.notify();
                return;
            }
        };

        let name = self.new_key_name.trim().to_string();
        let key_base64 = encode_base64(self.new_key_pem.trim().as_bytes());
        let req = CreateKeyRequest { name, key_base64 };

        let view_weak = cx.entity().downgrade();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Result<SshKey, String>>();

        TOKIO_RT.spawn(async move {
            match client.create_key(&req).await {
                Ok(k) => {
                    let _ = tx.send(Ok(k));
                }
                Err(e) => {
                    let _ = tx.send(Err(e.to_string()));
                }
            }
        });

        cx.spawn(move |_view, cx: &mut AsyncApp| {
            let view_weak = view_weak.clone();
            let cx_handle = cx.clone();
            async move {
                if let Some(res) = rx.recv().await {
                    cx_handle.update(|cx: &mut App| {
                        if let Some(app) = view_weak.upgrade() {
                            app.update(cx, |this, cx| {
                                match res {
                                    Ok(key) => {
                                        this.show_add_key_modal = false;
                                        this.new_key_name.clear();
                                        this.new_key_pem.clear();
                                        this.notification = Some(format!("SSH Key '{}' added", key.name));
                                        this.fetch_ssh_keys(cx);
                                    }
                                    Err(e) => {
                                        this.add_key_error = Some(format!("Failed to add key: {e}"));
                                    }
                                }
                                cx.notify();
                            });
                        }
                    });
                }
            }
        })
        .detach();
    }

    /// Delete SSH key by ID.
    pub fn delete_ssh_key(&mut self, id: &str, cx: &mut Context<Self>) {
        let client = match self.client.clone() {
            Some(c) => c,
            None => return,
        };

        let id_str = id.to_string();
        let view_weak = cx.entity().downgrade();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Result<(), String>>();

        TOKIO_RT.spawn(async move {
            match client.delete_key(&id_str).await {
                Ok(_) => {
                    let _ = tx.send(Ok(()));
                }
                Err(e) => {
                    let _ = tx.send(Err(e.to_string()));
                }
            }
        });

        self.ssh_keys.retain(|k| k.id != id);
        self.passphrase_cache.remove(id);
        cx.notify();

        cx.spawn(move |_view, cx: &mut AsyncApp| {
            let view_weak = view_weak.clone();
            let cx_handle = cx.clone();
            async move {
                if let Some(res) = rx.recv().await {
                    cx_handle.update(|cx: &mut App| {
                        if let Some(app) = view_weak.upgrade() {
                            app.update(cx, |this, cx| {
                                match res {
                                    Ok(()) => {
                                        this.notification = Some("SSH Key deleted".to_string());
                                    }
                                    Err(e) => {
                                        this.notification = Some(format!("Delete key failed: {e}"));
                                        this.fetch_ssh_keys(cx);
                                    }
                                }
                                cx.notify();
                            });
                        }
                    });
                }
            }
        })
        .detach();
    }

    /// Set search filter query.
    pub fn set_search_query(&mut self, query: String, cx: &mut Context<Self>) {
        self.search_query = query;
        cx.notify();
    }

    /// Select tag filter pill.
    pub fn select_tag_filter(&mut self, tag: Option<String>, cx: &mut Context<Self>) {
        self.selected_tag = tag;
        cx.notify();
    }

    /// Fetch all uploaded SSH keys from backend.
    pub fn fetch_ssh_keys(&mut self, cx: &mut Context<Self>) {
        let client = match self.client.clone() {
            Some(c) => c,
            None => return,
        };

        self.is_loading_keys = true;
        cx.notify();

        let view_weak = cx.entity().downgrade();
        let (tx, mut rx) =
            tokio::sync::mpsc::unbounded_channel::<Result<Vec<SshKey>, String>>();

        TOKIO_RT.spawn(async move {
            match client.list_keys().await {
                Ok(keys) => {
                    let _ = tx.send(Ok(keys));
                }
                Err(e) => {
                    let _ = tx.send(Err(e.to_string()));
                }
            }
        });

        cx.spawn(move |_view, cx: &mut AsyncApp| {
            let view_weak = view_weak.clone();
            let cx_handle = cx.clone();
            async move {
                if let Some(res) = rx.recv().await {
                    cx_handle.update(|cx: &mut App| {
                        if let Some(app) = view_weak.upgrade() {
                            app.update(cx, |this, cx| {
                                this.is_loading_keys = false;
                                match res {
                                    Ok(keys) => {
                                        this.ssh_keys = keys;
                                    }
                                    Err(err) => {
                                        eprintln!("Failed to fetch SSH keys: {err}");
                                    }
                                }
                                cx.notify();
                            });
                        }
                    });
                }
            }
        })
        .detach();
    }

    /// Open create connection modal.
    pub fn open_create_connection_modal(&mut self, cx: &mut Context<Self>) {
        self.connection_modal = Some(ConnectionFormState::new_create());
        self.fetch_ssh_keys(cx);
        cx.notify();
    }

    /// Open edit connection modal.
    pub fn open_edit_connection_modal(&mut self, id: &str, cx: &mut Context<Self>) {
        if let Some(conn) = self.connections.iter().find(|c| c.id == id).cloned() {
            self.connection_modal = Some(ConnectionFormState::new_edit(&conn));
            self.fetch_ssh_keys(cx);

            if let Some(client) = self.client.clone() {
                let id_str = id.to_string();
                let view_weak = cx.entity().downgrade();
                let (tx, mut rx) =
                    tokio::sync::mpsc::unbounded_channel::<Result<Connection, String>>();

                TOKIO_RT.spawn(async move {
                    match client.get_connection(&id_str).await {
                        Ok(c) => {
                            let _ = tx.send(Ok(c));
                        }
                        Err(e) => {
                            let _ = tx.send(Err(e.to_string()));
                        }
                    }
                });

                cx.spawn(move |_view, cx: &mut AsyncApp| {
                    let view_weak = view_weak.clone();
                    let cx_handle = cx.clone();
                    async move {
                        if let Some(Ok(full_conn)) = rx.recv().await {
                            cx_handle.update(|cx: &mut App| {
                                if let Some(app) = view_weak.upgrade() {
                                    app.update(cx, |this, cx| {
                                        if let Some(form) = &mut this.connection_modal {
                                            if form.mode == ConnectionModalMode::Edit(full_conn.id.clone()) {
                                                if let Some(key_id) = full_conn.ssh_key_id {
                                                    form.ssh_key_id = Some(key_id);
                                                }
                                                cx.notify();
                                            }
                                        }
                                    });
                                }
                            });
                        }
                    }
                })
                .detach();
            }
        }
        cx.notify();
    }

    /// Close connection modal.
    pub fn close_connection_modal(&mut self, cx: &mut Context<Self>) {
        self.connection_modal = None;
        cx.notify();
    }

    /// Save connection form (Create or Update).
    pub fn save_connection_form(&mut self, cx: &mut Context<Self>) {
        let form = match &mut self.connection_modal {
            Some(f) => f,
            None => return,
        };

        if let Err(err) = form.validate() {
            form.error_message = Some(err);
            cx.notify();
            return;
        }

        form.error_message = None;

        let client = match self.client.clone() {
            Some(c) => c,
            None => {
                if let Some(f) = &mut self.connection_modal {
                    f.error_message = Some("Backend client is not connected".to_string());
                    cx.notify();
                }
                return;
            }
        };

        let mode = form.mode.clone();
        let view_weak = cx.entity().downgrade();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Result<String, String>>();

        match mode {
            ConnectionModalMode::Create => {
                let req = form.to_create_request();
                TOKIO_RT.spawn(async move {
                    match client.create_connection(&req).await {
                        Ok(_) => {
                            let _ = tx.send(Ok("Host connection created successfully".to_string()));
                        }
                        Err(e) => {
                            let _ = tx.send(Err(e.to_string()));
                        }
                    }
                });
            }
            ConnectionModalMode::Edit(id) => {
                let req = form.to_update_request();
                TOKIO_RT.spawn(async move {
                    match client.update_connection(&id, &req).await {
                        Ok(_) => {
                            let _ = tx.send(Ok("Host connection updated successfully".to_string()));
                        }
                        Err(e) => {
                            let _ = tx.send(Err(e.to_string()));
                        }
                    }
                });
            }
        }

        cx.spawn(move |_view, cx: &mut AsyncApp| {
            let view_weak = view_weak.clone();
            let cx_handle = cx.clone();
            async move {
                if let Some(res) = rx.recv().await {
                    cx_handle.update(|cx: &mut App| {
                        if let Some(app) = view_weak.upgrade() {
                            app.update(cx, |this, cx| {
                                match res {
                                    Ok(msg) => {
                                        this.connection_modal = None;
                                        this.notification = Some(msg);
                                        this.fetch_connections(cx);
                                    }
                                    Err(e) => {
                                        if let Some(f) = &mut this.connection_modal {
                                            f.error_message = Some(format!("Error saving host: {e}"));
                                        }
                                    }
                                }
                                cx.notify();
                            });
                        }
                    });
                }
            }
        })
        .detach();
    }

    /// Delete connection by ID.
    pub fn delete_connection(&mut self, id: &str, cx: &mut Context<Self>) {
        let client = match self.client.clone() {
            Some(c) => c,
            None => return,
        };

        let id_str = id.to_string();
        let view_weak = cx.entity().downgrade();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Result<(), String>>();

        TOKIO_RT.spawn(async move {
            match client.delete_connection(&id_str).await {
                Ok(_) => {
                    let _ = tx.send(Ok(()));
                }
                Err(e) => {
                    let _ = tx.send(Err(e.to_string()));
                }
            }
        });

        self.connections.retain(|c| c.id != id);
        cx.notify();

        cx.spawn(move |_view, cx: &mut AsyncApp| {
            let view_weak = view_weak.clone();
            let cx_handle = cx.clone();
            async move {
                if let Some(res) = rx.recv().await {
                    cx_handle.update(|cx: &mut App| {
                        if let Some(app) = view_weak.upgrade() {
                            app.update(cx, |this, cx| {
                                match res {
                                    Ok(()) => {
                                        this.notification = Some("Connection deleted".to_string());
                                    }
                                    Err(e) => {
                                        this.notification = Some(format!("Delete failed: {e}"));
                                        this.fetch_connections(cx);
                                    }
                                }
                                cx.notify();
                            });
                        }
                    });
                }
            }
        })
        .detach();
    }

    /// Export connections to disk.
    pub fn export_connections_to_disk(&mut self, cx: &mut Context<Self>) {
        let client = match self.client.clone() {
            Some(c) => c,
            None => return,
        };

        let view_weak = cx.entity().downgrade();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Result<usize, String>>();

        TOKIO_RT.spawn(async move {
            match client.export_connections().await {
                Ok(conns) => {
                    let count = conns.len();
                    match serde_json::to_string_pretty(&conns) {
                        Ok(json_str) => {
                            let path = std::path::Path::new("webterm-connections-export.json");
                            if let Err(e) = std::fs::write(path, json_str) {
                                let _ = tx.send(Err(format!("Failed to write export file: {e}")));
                            } else {
                                let _ = tx.send(Ok(count));
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(Err(format!("Failed to serialize connections: {e}")));
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(e.to_string()));
                }
            }
        });

        cx.spawn(move |_view, cx: &mut AsyncApp| {
            let view_weak = view_weak.clone();
            let cx_handle = cx.clone();
            async move {
                if let Some(res) = rx.recv().await {
                    cx_handle.update(|cx: &mut App| {
                        if let Some(app) = view_weak.upgrade() {
                            app.update(cx, |this, cx| {
                                match res {
                                    Ok(count) => {
                                        this.notification = Some(format!(
                                            "Exported {count} connection(s) to webterm-connections-export.json"
                                        ));
                                    }
                                    Err(e) => {
                                        this.notification = Some(format!("Export failed: {e}"));
                                    }
                                }
                                cx.notify();
                            });
                        }
                    });
                }
            }
        })
        .detach();
    }

    /// Open import connections modal.
    pub fn open_import_modal(&mut self, cx: &mut Context<Self>) {
        self.show_import_modal = true;
        self.import_payload = String::new();
        cx.notify();
    }

    /// Close import connections modal.
    pub fn close_import_modal(&mut self, cx: &mut Context<Self>) {
        self.show_import_modal = false;
        self.import_payload.clear();
        cx.notify();
    }

    /// Submit imported connections JSON to backend.
    pub fn submit_import_connections(&mut self, cx: &mut Context<Self>) {
        let client = match self.client.clone() {
            Some(c) => c,
            None => {
                self.notification = Some("Backend client is not connected".to_string());
                cx.notify();
                return;
            }
        };

        let payload = self.import_payload.trim();
        let conns: Vec<Connection> = if payload.is_empty() {
            let path = std::path::Path::new("webterm-connections-export.json");
            if path.exists() {
                match std::fs::read_to_string(path) {
                    Ok(content) => match serde_json::from_str::<Vec<Connection>>(&content) {
                        Ok(c) => c,
                        Err(e) => {
                            self.notification = Some(format!("JSON parsing error from export file: {e}"));
                            cx.notify();
                            return;
                        }
                    },
                    Err(e) => {
                        self.notification = Some(format!("Failed to read export file: {e}"));
                        cx.notify();
                        return;
                    }
                }
            } else {
                self.notification = Some("Please paste JSON array or create webterm-connections-export.json".to_string());
                cx.notify();
                return;
            }
        } else {
            match serde_json::from_str::<Vec<Connection>>(payload) {
                Ok(c) => c,
                Err(e) => {
                    self.notification = Some(format!("Invalid JSON array: {e}"));
                    cx.notify();
                    return;
                }
            }
        };

        let view_weak = cx.entity().downgrade();
        let (tx, mut rx) =
            tokio::sync::mpsc::unbounded_channel::<Result<ImportResult, String>>();

        TOKIO_RT.spawn(async move {
            match client.import_connections(&conns).await {
                Ok(res) => {
                    let _ = tx.send(Ok(res));
                }
                Err(e) => {
                    let _ = tx.send(Err(e.to_string()));
                }
            }
        });

        cx.spawn(move |_view, cx: &mut AsyncApp| {
            let view_weak = view_weak.clone();
            let cx_handle = cx.clone();
            async move {
                if let Some(res) = rx.recv().await {
                    cx_handle.update(|cx: &mut App| {
                        if let Some(app) = view_weak.upgrade() {
                            app.update(cx, |this, cx| {
                                match res {
                                    Ok(result) => {
                                        this.show_import_modal = false;
                                        this.import_payload.clear();
                                        this.notification = Some(format!(
                                            "Import finished: {} imported, {} skipped",
                                            result.imported, result.skipped
                                        ));
                                        this.fetch_connections(cx);
                                    }
                                    Err(e) => {
                                        this.notification = Some(format!("Import error: {e}"));
                                    }
                                }
                                cx.notify();
                            });
                        }
                    });
                }
            }
        })
        .detach();
    }

    /// Dismiss active notification banner.
    pub fn dismiss_notification(&mut self, cx: &mut Context<Self>) {
        self.notification = None;
        cx.notify();
    }

    /// Spawn the supervisor lifecycle in a background thread and observe transitions.
    pub fn start_supervisor(&mut self, cx: &mut Context<Self>) {
        let spawn_opts = match self.spawn_opts.clone() {
            Some(opts) => opts,
            None => {
                self.backend_status = BackendStatus::Failed {
                    reason: "No backend spawn options configured".to_string(),
                    stderr_tail: "Please build the backend binary: scripts/build-test-backend".to_string(),
                };
                cx.notify();
                return;
            }
        };

        self.backend_status = BackendStatus::Starting;
        cx.notify();

        let view_weak = cx.entity().downgrade();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<SupervisorEvent>();

        // Run supervisor inside Tokio runtime
        TOKIO_RT.spawn(async move {
            let mut supervisor = Supervisor::new();
            let mut status_rx = supervisor.subscribe();

            let tx_status = tx.clone();
            tokio::spawn(async move {
                while status_rx.changed().await.is_ok() {
                    let st = status_rx.borrow().clone();
                    if tx_status.send(SupervisorEvent::Status(st)).is_err() {
                        break;
                    }
                }
            });

            match supervisor.spawn(spawn_opts).await {
                Ok(info) => {
                    let _ = tx.send(SupervisorEvent::Ready(info));
                }
                Err(e) => {
                    let (reason, stderr_tail) = match e {
                        webterm_supervisor::SupervisorError::Failed { reason, stderr_tail } => {
                            (reason, stderr_tail)
                        }
                        other => (other.to_string(), String::new()),
                    };
                    let _ = tx.send(SupervisorEvent::Failed { reason, stderr_tail });
                }
            }
        });

        // Observe events inside GPUI foreground executor
        cx.spawn(move |_view, cx: &mut AsyncApp| {
            let view_weak = view_weak.clone();
            let cx_handle = cx.clone();
            async move {
                while let Some(event) = rx.recv().await {
                    let view_weak = view_weak.clone();
                    cx_handle.update(|cx: &mut App| {
                        if let Some(entity) = view_weak.upgrade() {
                            entity.update(cx, |this, cx| {
                                match event {
                                    SupervisorEvent::Status(st) => {
                                        this.backend_status = st;
                                    }
                                    SupervisorEvent::Ready(info) => {
                                        this.client = Some(BackendClient::new(info.base_url));
                                        this.backend_status = BackendStatus::Ready;
                                        this.restore_sessions_or_default(cx);
                                        this.fetch_connections(cx);
                                        this.fetch_ssh_keys(cx);
                                    }
                                    SupervisorEvent::Failed { reason, stderr_tail } => {
                                        this.backend_status = BackendStatus::Failed {
                                            reason,
                                            stderr_tail,
                                        };
                                    }
                                }
                                cx.notify();
                            });
                        }
                    });
                }
            }
        }).detach();
    }

    /// Restore detached sessions from backend or open a default local tab.
    pub fn restore_sessions_or_default(&mut self, cx: &mut Context<Self>) {
        let client = match self.client.clone() {
            Some(c) => c,
            None => return,
        };

        let saved_sessions = self.settings.open_sessions.clone();
        let is_dark = self.theme != SettingsTheme::Light;
        let view_weak = cx.entity().downgrade();

        cx.spawn(move |_view, cx: &mut AsyncApp| {
            let view_weak = view_weak.clone();
            let cx_handle = cx.clone();
            async move {
                let backend_sessions = client.list_sessions().await.unwrap_or_default();

                let mut to_attach: Vec<(String, String, String, Option<String>)> = Vec::new();

                if !saved_sessions.is_empty() {
                    for saved in &saved_sessions {
                        if let Some(matched) = backend_sessions.iter().find(|s| s.id == saved.session_id) {
                            to_attach.push((
                                matched.id.clone(),
                                saved.title.clone(),
                                matched.session_type.clone(),
                                matched.connection_id.clone(),
                            ));
                        }
                    }
                } else {
                    for s in &backend_sessions {
                        let title = if s.session_type == "local" {
                            "Local Shell".to_string()
                        } else {
                            format!("{}:{}", s.user, s.host)
                        };
                        to_attach.push((
                            s.id.clone(),
                            title,
                            s.session_type.clone(),
                            s.connection_id.clone(),
                        ));
                    }
                }

                cx_handle.update(|cx: &mut App| {
                    if let Some(app) = view_weak.upgrade() {
                        app.update(cx, |this, cx| {
                            if to_attach.is_empty() {
                                if this.session_manager.is_empty() {
                                    this.open_local_tab(cx);
                                }
                            } else {
                                for (session_id, title, stype, conn_id) in to_attach {
                                    let tab_id = this.session_manager.alloc_tab_id();
                                    let mut tab = TerminalTab::new(
                                        tab_id,
                                        title,
                                        stype,
                                        conn_id,
                                        is_dark,
                                        cx,
                                    );
                                    tab.session_id = Some(session_id.clone());
                                    this.session_manager.add_tab(tab);
                                    this.reconnect_tab(tab_id, cx);
                                }
                                this.persist_open_sessions();
                            }
                            cx.notify();
                        });
                    }
                });
            }
        }).detach();
    }

    /// Persist current open session IDs to settings store.
    pub fn persist_open_sessions(&mut self) {
        let saved: Vec<SavedSessionTab> = self
            .session_manager
            .tabs()
            .iter()
            .filter_map(|t| {
                t.session_id.as_ref().map(|sid| SavedSessionTab {
                    session_id: sid.clone(),
                    title: t.title.clone(),
                    session_type: t.session_type.clone(),
                    connection_id: t.connection_id.clone(),
                })
            })
            .collect();

        self.settings.open_sessions = saved;
        let _ = self.settings.save();
    }

    /// Open a new local shell terminal tab.
    pub fn open_local_tab(&mut self, cx: &mut Context<Self>) {
        let client = match self.client.clone() {
            Some(c) => c,
            None => return,
        };

        let tab_id = self.session_manager.alloc_tab_id();
        let is_dark = self.theme != SettingsTheme::Light;
        let tab = TerminalTab::new(
            tab_id,
            "Local Shell",
            "local",
            Some("local".to_string()),
            is_dark,
            cx,
        );
        self.show_hosts_catalog = false;
        self.session_manager.add_tab(tab);
        cx.notify();

        let view_weak = cx.entity().downgrade();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Result<TerminalWsHandle, String>>();

        TOKIO_RT.spawn(async move {
            let req = WsConnectRequest::for_local(80, 24);
            match client.connect_terminal(req).await {
                Ok(handle) => {
                    let _ = tx.send(Ok(handle));
                }
                Err(e) => {
                    let _ = tx.send(Err(e.to_string()));
                }
            }
        });

        cx.spawn(move |_view, cx: &mut AsyncApp| {
            let view_weak = view_weak.clone();
            let cx_handle = cx.clone();
            async move {
                if let Some(res) = rx.recv().await {
                    cx_handle.update(|cx: &mut App| {
                        if let Some(app) = view_weak.upgrade() {
                            app.update(cx, |this, cx| {
                                match res {
                                    Ok(handle) => {
                                        let session_id = handle.session_id().to_string();
                                        let app_weak = cx.entity().downgrade();
                                        this.session_manager.attach_handle(
                                            tab_id,
                                            handle,
                                            session_id,
                                            Some(app_weak),
                                            cx,
                                        );
                                        this.persist_open_sessions();
                                    }
                                    Err(err_msg) => {
                                        this.session_manager.set_tab_status(
                                            tab_id,
                                            SessionStatus::Disconnected(Some(err_msg)),
                                        );
                                    }
                                }
                                cx.notify();
                            });
                        }
                    });
                }
            }
        }).detach();
    }

    /// Open a new SSH terminal tab with given connection request.
    #[allow(dead_code)]
    pub fn open_ssh_tab(&mut self, req: WsConnectRequest, title: &str, cx: &mut Context<Self>) {
        let client = match self.client.clone() {
            Some(c) => c,
            None => return,
        };

        let tab_id = self.session_manager.alloc_tab_id();
        let is_dark = self.theme != SettingsTheme::Light;
        let mut tab = TerminalTab::new(
            tab_id,
            title,
            "ssh",
            req.connection_id.clone(),
            is_dark,
            cx,
        );
        tab.last_connect_req = Some(req.clone());
        self.show_hosts_catalog = false;
        self.session_manager.add_tab(tab);
        cx.notify();

        let view_weak = cx.entity().downgrade();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Result<TerminalWsHandle, String>>();

        TOKIO_RT.spawn(async move {
            match client.connect_terminal(req).await {
                Ok(handle) => {
                    let _ = tx.send(Ok(handle));
                }
                Err(e) => {
                    let _ = tx.send(Err(e.to_string()));
                }
            }
        });

        cx.spawn(move |_view, cx: &mut AsyncApp| {
            let view_weak = view_weak.clone();
            let cx_handle = cx.clone();
            async move {
                if let Some(res) = rx.recv().await {
                    cx_handle.update(|cx: &mut App| {
                        if let Some(app) = view_weak.upgrade() {
                            app.update(cx, |this, cx| {
                                match res {
                                    Ok(handle) => {
                                        let session_id = handle.session_id().to_string();
                                        let app_weak = cx.entity().downgrade();
                                        this.session_manager.attach_handle(
                                            tab_id,
                                            handle,
                                            session_id,
                                            Some(app_weak),
                                            cx,
                                        );
                                        this.persist_open_sessions();
                                    }
                                    Err(err_msg) => {
                                        this.session_manager.set_tab_status(
                                            tab_id,
                                            SessionStatus::Disconnected(Some(err_msg)),
                                        );
                                    }
                                }
                                cx.notify();
                            });
                        }
                    });
                }
            }
        }).detach();
    }

    /// Invoked when a tab's WebSocket transport connection terminates unexpectedly.
    pub fn on_tab_dropped(&mut self, tab_id: u64, cx: &mut Context<Self>) {
        let tab = match self.session_manager.tabs_mut().iter_mut().find(|t| t.id == tab_id) {
            Some(t) => t,
            None => return,
        };

        if matches!(tab.status, SessionStatus::Disconnected(_)) {
            return;
        }

        tab.status = SessionStatus::Reconnecting {
            attempt: 1,
            next_retry_secs: 2,
        };
        cx.notify();

        self.schedule_reconnect(tab_id, 1, 2, cx);
    }

    /// Schedule an automatic reconnection attempt after specified delay.
    pub fn schedule_reconnect(
        &mut self,
        tab_id: u64,
        _attempt: usize,
        delay_secs: usize,
        cx: &mut Context<Self>,
    ) {
        let view_weak = cx.entity().downgrade();
        cx.spawn(move |_view, cx: &mut AsyncApp| {
            let view_weak = view_weak.clone();
            let cx_handle = cx.clone();
            async move {
                tokio::time::sleep(std::time::Duration::from_secs(delay_secs as u64)).await;
                cx_handle.update(|cx: &mut App| {
                    if let Some(app) = view_weak.upgrade() {
                        app.update(cx, |this, cx| {
                            let should_reconnect = if let Some(tab) =
                                this.session_manager.tabs().iter().find(|t| t.id == tab_id)
                            {
                                matches!(tab.status, SessionStatus::Reconnecting { .. })
                            } else {
                                false
                            };

                            if should_reconnect {
                                this.reconnect_tab(tab_id, cx);
                            }
                        });
                    }
                });
            }
        }).detach();
    }

    /// Attempt to reconnect/re-attach a tab.
    pub fn reconnect_tab(&mut self, tab_id: u64, cx: &mut Context<Self>) {
        let client = match self.client.clone() {
            Some(c) => c,
            None => return,
        };

        let (session_id, attempt, last_req) =
            match self.session_manager.tabs().iter().find(|t| t.id == tab_id) {
                Some(tab) => {
                    let attempt = match tab.status {
                        SessionStatus::Reconnecting { attempt, .. } => attempt,
                        _ => 1,
                    };
                    (
                        tab.session_id.clone(),
                        attempt,
                        tab.last_connect_req.clone(),
                    )
                }
                None => return,
            };

        self.session_manager
            .set_tab_status(tab_id, SessionStatus::Connecting);
        cx.notify();

        let view_weak = cx.entity().downgrade();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Result<TerminalWsHandle, String>>();

        TOKIO_RT.spawn(async move {
            if let Some(ref sid) = session_id {
                match client.attach_terminal(sid).await {
                    Ok(handle) => {
                        let _ = tx.send(Ok(handle));
                    }
                    Err(e) => {
                        let _ = tx.send(Err(e.to_string()));
                    }
                }
            } else if let Some(req) = last_req {
                match client.connect_terminal(req).await {
                    Ok(handle) => {
                        let _ = tx.send(Ok(handle));
                    }
                    Err(e) => {
                        let _ = tx.send(Err(e.to_string()));
                    }
                }
            } else {
                let req = WsConnectRequest::for_local(80, 24);
                match client.connect_terminal(req).await {
                    Ok(handle) => {
                        let _ = tx.send(Ok(handle));
                    }
                    Err(e) => {
                        let _ = tx.send(Err(e.to_string()));
                    }
                }
            }
        });

        cx.spawn(move |_view, cx: &mut AsyncApp| {
            let view_weak = view_weak.clone();
            let cx_handle = cx.clone();
            async move {
                if let Some(res) = rx.recv().await {
                    cx_handle.update(|cx: &mut App| {
                        if let Some(app) = view_weak.upgrade() {
                            app.update(cx, |this, cx| {
                                match res {
                                    Ok(handle) => {
                                        let session_id = handle.session_id().to_string();
                                        let app_weak = cx.entity().downgrade();
                                        this.session_manager.attach_handle(
                                            tab_id,
                                            handle,
                                            session_id,
                                            Some(app_weak),
                                            cx,
                                        );
                                        this.persist_open_sessions();
                                    }
                                    Err(err_msg) => {
                                        if attempt < 5 {
                                            let next_attempt = attempt + 1;
                                            let next_secs = std::cmp::min(2 * next_attempt, 16);
                                            this.session_manager.set_tab_status(
                                                tab_id,
                                                SessionStatus::Reconnecting {
                                                    attempt: next_attempt,
                                                    next_retry_secs: next_secs,
                                                },
                                            );
                                            this.schedule_reconnect(tab_id, next_attempt, next_secs, cx);
                                        } else {
                                            this.session_manager.set_tab_status(
                                                tab_id,
                                                SessionStatus::Disconnected(Some(err_msg)),
                                            );
                                        }
                                    }
                                }
                                cx.notify();
                            });
                        }
                    });
                }
            }
        }).detach();
    }

    /// Close tab at given index and persist open sessions.
    pub fn close_tab(&mut self, index: usize, cx: &mut Context<Self>) {
        self.session_manager.close_tab(index);
        if self.session_manager.tab_count() == 0 {
            self.show_hosts_catalog = true;
        }
        self.persist_open_sessions();
        cx.notify();
    }

    /// Switch active tab to given index.
    pub fn switch_tab(&mut self, index: usize, cx: &mut Context<Self>) {
        self.show_hosts_catalog = false;
        if self.session_manager.switch_tab(index) {
            cx.notify();
        }
    }

    /// Cycle to next tab (Ctrl+Tab).
    pub fn cycle_next_tab(&mut self, cx: &mut Context<Self>) {
        self.show_hosts_catalog = false;
        self.session_manager.cycle_next();
        cx.notify();
    }

    /// Cycle to previous tab (Ctrl+Shift+Tab).
    pub fn cycle_prev_tab(&mut self, cx: &mut Context<Self>) {
        self.show_hosts_catalog = false;
        self.session_manager.cycle_prev();
        cx.notify();
    }

    /// Jump directly to tab (Alt+1..9).
    pub fn jump_to_tab(&mut self, index: usize, cx: &mut Context<Self>) {
        self.show_hosts_catalog = false;
        if self.session_manager.jump_to(index) {
            cx.notify();
        }
    }

    /// Toggle theme between Dark and Light and persist to settings store.
    pub fn toggle_theme(&mut self, cx: &mut Context<Self>) {
        self.theme = crate::theme::toggle_theme(self.theme);
        self.settings.theme = self.theme;
        let _ = self.settings.save();
        cx.notify();
    }
}

impl Render for AppState {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let key_secret = self.settings.encryption_key.as_deref();

        match self.backend_status {
            BackendStatus::Ready => render_nav_shell(self, cx),
            _ => {
                let status = self.backend_status.clone();
                render_status_page(
                    &status,
                    key_secret,
                    |this, _ev, _window, cx| {
                        this.start_supervisor(cx);
                    },
                    cx,
                )
            }
        }
    }
}
