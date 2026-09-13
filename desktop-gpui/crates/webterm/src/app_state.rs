//! Root application state, session orchestration, and view routing.

use gpui::*;
use std::collections::HashSet;
use std::sync::LazyLock;
use webterm_backend_client::{
    BackendClient, Connection, CreateConnectionRequest, CreateForwardRequest, CreateKeyRequest,
    ImportResult, PortForward, SessionInfo, SftpFileInfo, SftpTransferStatus, SshKey,
    TerminalWsHandle, UpdateConnectionRequest, UpdateForwardRequest, UpdateKeyRequest,
    WsConnectRequest,
};
use webterm_settings::{DesktopSettings, SavedSessionTab, Theme as SettingsTheme};
use webterm_supervisor::{BackendInfo, BackendStatus, SpawnOptions, Supervisor};

use crate::session::{SessionStatus, TerminalSessionManager, TerminalTab};
use crate::views::{nav::render_nav_shell, status::render_status_page};
use gpui_component::input::{InputEvent, InputState, TextareaState};

/// Editable text input entities shared by every form in the app.
///
/// gpui-component `InputState`/`TextareaState` entities are created once at
/// startup (they need a `Window`), reused across form opens, and are the
/// source of truth for text fields; form structs keep only non-text state.
pub struct FormInputs {
    pub conn_label: Entity<InputState>,
    pub conn_host: Entity<InputState>,
    pub conn_port: Entity<InputState>,
    pub conn_username: Entity<InputState>,
    pub conn_password: Entity<InputState>,
    pub conn_tags: Entity<InputState>,
    pub new_key_name: Entity<InputState>,
    pub new_key_pem: Entity<TextareaState>,
    pub edit_key_name: Entity<InputState>,
    pub edit_key_pem: Entity<TextareaState>,
    pub fwd_name: Entity<InputState>,
    pub fwd_local_port: Entity<InputState>,
    pub fwd_remote_port: Entity<InputState>,
    pub passphrase: Entity<InputState>,
    pub quick_connect: Entity<InputState>,
    pub hosts_search: Entity<InputState>,
    pub sftp_search_left: Entity<InputState>,
    pub sftp_search_right: Entity<InputState>,
    pub sftp_modal_name: Entity<InputState>,
    pub backend_path: Entity<InputState>,
}

impl AppState {
    /// Create every form input entity and wire live-filter subscriptions.
    /// Must be called once from a `Window` context (input states need it).
    pub fn init_form_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let mk_input = |placeholder: &'static str,
                        window: &mut Window,
                        cx: &mut Context<AppState>|
         -> Entity<InputState> {
            cx.new(|cx| InputState::new(window, cx).placeholder(placeholder))
        };

        let conn_label = mk_input("My Production Server", window, cx);
        let conn_host = mk_input("10.0.0.1", window, cx);
        let conn_port = mk_input("22", window, cx);
        let conn_username = mk_input("root", window, cx);
        let conn_password = cx.new(|cx| InputState::new(window, cx).placeholder("Password").masked(true));
        let conn_tags = mk_input("prod, aws, web", window, cx);
        let new_key_name = mk_input("e.g. id_ed25519_deploy", window, cx);
        let new_key_pem: Entity<TextareaState> = cx.new(|cx| {
            TextareaState::new(window, cx).placeholder("Paste OpenSSH private key PEM content here...")
        });
        let edit_key_name = mk_input("e.g. id_ed25519_deploy", window, cx);
        let edit_key_pem: Entity<TextareaState> = cx.new(|cx| {
            TextareaState::new(window, cx).placeholder("(leave empty to keep current key)")
        });
        let fwd_name = mk_input("e.g. Production Database", window, cx);
        let fwd_local_port = mk_input("e.g. 5432", window, cx);
        let fwd_remote_port = mk_input("e.g. 5432", window, cx);
        let passphrase = cx.new(|cx| InputState::new(window, cx).placeholder("Passphrase").masked(true));
        let quick_connect = mk_input("Search or user@host...", window, cx);
        let hosts_search = mk_input("Search hosts...", window, cx);
        let sftp_search_left = mk_input("Filter files...", window, cx);
        let sftp_search_right = mk_input("Filter files...", window, cx);
        let sftp_modal_name = mk_input("Name", window, cx);
        let backend_path = mk_input("Path to backend executable", window, cx);

        // Live filters: propagate typing into AppState and re-render.
        let hosts = hosts_search.clone();
        cx.subscribe(&hosts, move |this, _, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change) {
                this.search_query = this.inputs().hosts_search.read(cx).value().to_string();
                cx.notify();
            }
        })
        .detach();

        let quick = quick_connect.clone();
        cx.subscribe(&quick, move |this, _, event: &InputEvent, cx| {
            match event {
                InputEvent::Change => {
                    this.quick_connect_query =
                        this.inputs().quick_connect.read(cx).value().to_string();
                    cx.notify();
                }
                InputEvent::PressEnter { .. } => {
                    let text = this.inputs().quick_connect.read(cx).value().to_string();
                    this.quick_connect_submit(&text, cx);
                }
                _ => {}
            }
        })
        .detach();

        for (entity, pane) in [
            (sftp_search_left.clone(), 0usize),
            (sftp_search_right.clone(), 1usize),
        ] {
            cx.subscribe(&entity, move |this, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    let query = if pane == 0 {
                        this.inputs().sftp_search_left.read(cx).value().to_string()
                    } else {
                        this.inputs().sftp_search_right.read(cx).value().to_string()
                    };
                    let target = if pane == 0 {
                        &mut this.sftp_manager.left_pane
                    } else {
                        &mut this.sftp_manager.right_pane
                    };
                    target.search_query = query;
                    cx.notify();
                }
            })
            .detach();
        }

        self.form_inputs = Some(FormInputs {
            conn_label,
            conn_host,
            conn_port,
            conn_username,
            conn_password,
            conn_tags,
            new_key_name,
            new_key_pem,
            edit_key_name,
            edit_key_pem,
            fwd_name,
            fwd_local_port,
            fwd_remote_port,
            passphrase,
            quick_connect,
            hosts_search,
            sftp_search_left,
            sftp_search_right,
            sftp_modal_name,
            backend_path,
        });
    }

    /// Panics only if called before `init_form_inputs`, which main.rs runs
    /// right after the window opens, before any view can render.
    pub fn inputs(&self) -> &FormInputs {
        self.form_inputs
            .as_ref()
            .expect("form inputs initialized at startup")
    }

    /// Read the current text of an input entity.
    pub fn input_value(entity: &Entity<InputState>, cx: &App) -> String {
        entity.read(cx).value().to_string()
    }

    /// Replace the text of an input entity (used when (re)opening forms).
    pub fn set_input_value(entity: &Entity<InputState>, value: &str, window: &mut Window, cx: &mut App) {
        entity.update(cx, |state, cx| {
            state.set_value(value, window, cx);
        });
    }

    /// Read the current text of a textarea entity.
    pub fn input_value_textarea(entity: &Entity<TextareaState>, cx: &App) -> String {
        entity.read(cx).value().to_string()
    }

    /// Replace the text of a textarea entity.
    pub fn set_textarea_value(
        entity: &Entity<TextareaState>,
        value: &str,
        window: &mut Window,
        cx: &mut App,
    ) {
        entity.update(cx, |state, cx| {
            state.set_value(value, window, cx);
        });
    }
}

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

/// Form state for Editing an existing SSH Key (rename / replace key material).
#[derive(Debug, Clone, PartialEq)]
pub struct EditKeyFormState {
    pub key_id: String,
    pub original_name: String,
    pub name: String,
    /// New PEM content; empty means keep the current key material.
    pub new_pem: String,
    pub error_message: Option<String>,
}

impl EditKeyFormState {
    pub fn new(key: &SshKey) -> Self {
        Self {
            key_id: key.id.clone(),
            original_name: key.name.clone(),
            name: key.name.clone(),
            new_pem: String::new(),
            error_message: None,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Key name is required".to_string());
        }
        if !self.new_pem.is_empty() && self.new_pem.trim().is_empty() {
            return Err("New key content cannot be empty".to_string());
        }
        Ok(())
    }
}

/// Mode of the Port Forward modal: Create or Edit with forward ID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForwardModalMode {
    Create,
    Edit(String),
}

/// Form state for Creating or Editing a Port Forward rule.
#[derive(Debug, Clone, PartialEq)]
pub struct ForwardFormState {
    pub mode: ForwardModalMode,
    pub name: String,
    pub connection_id: String,
    pub local_port: String,
    pub remote_port: String,
    pub forward_type: String, // "local" | "reverse"
    pub error_message: Option<String>,
}

impl ForwardFormState {
    pub fn new_create(default_conn_id: Option<String>) -> Self {
        Self {
            mode: ForwardModalMode::Create,
            name: String::new(),
            connection_id: default_conn_id.unwrap_or_default(),
            local_port: String::new(),
            remote_port: String::new(),
            forward_type: "local".to_string(),
            error_message: None,
        }
    }

    pub fn new_edit(forward: &PortForward) -> Self {
        Self {
            mode: ForwardModalMode::Edit(forward.id.clone()),
            name: forward.name.clone(),
            connection_id: forward.connection_id.clone(),
            local_port: forward.local_port.to_string(),
            remote_port: forward.remote_port.to_string(),
            forward_type: forward.forward_type.clone(),
            error_message: None,
        }
    }

    pub fn parse_local_port(&self) -> Result<u16, String> {
        let p = self
            .local_port
            .trim()
            .parse::<u16>()
            .map_err(|_| "Local port must be a valid number (1-65535)".to_string())?;
        if p == 0 {
            return Err("Local port must be between 1 and 65535".to_string());
        }
        Ok(p)
    }

    pub fn parse_remote_port(&self) -> Result<u16, String> {
        let p = self
            .remote_port
            .trim()
            .parse::<u16>()
            .map_err(|_| "Remote port must be a valid number (1-65535)".to_string())?;
        if p == 0 {
            return Err("Remote port must be between 1 and 65535".to_string());
        }
        Ok(p)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Forward rule name is required".to_string());
        }
        if self.connection_id.trim().is_empty() {
            return Err("Please select a SSH connection".to_string());
        }
        self.parse_local_port()?;
        self.parse_remote_port()?;
        if self.forward_type != "local" && self.forward_type != "reverse" {
            return Err("Forward type must be 'local' or 'reverse'".to_string());
        }
        Ok(())
    }

    pub fn to_create_request(&self) -> Result<CreateForwardRequest, String> {
        self.validate()?;
        Ok(CreateForwardRequest {
            name: self.name.trim().to_string(),
            connection_id: self.connection_id.clone(),
            local_port: self.parse_local_port()?,
            remote_port: self.parse_remote_port()?,
            forward_type: self.forward_type.clone(),
        })
    }

    pub fn to_update_request(&self) -> Result<UpdateForwardRequest, String> {
        self.validate()?;
        Ok(UpdateForwardRequest {
            name: self.name.trim().to_string(),
            connection_id: self.connection_id.clone(),
            local_port: self.parse_local_port()?,
            remote_port: self.parse_remote_port()?,
            forward_type: self.forward_type.clone(),
        })
    }
}

/// Active view in the main navigation sidebar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    NewTab,
    Hosts,
    Keys,
    Forwards,
    Sftp,
    Settings,
}

/// Active pane in the dual-pane SFTP manager.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SftpActivePane {
    Left,
    Right,
}

/// Sort column for SFTP file list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SftpSortColumn {
    Name,
    Size,
    ModTime,
}

/// Sort direction for SFTP file list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SftpSortOrder {
    Ascending,
    Descending,
}

/// State of a single SFTP pane.
#[derive(Debug, Clone)]
pub struct SftpPaneState {
    pub source_id: String,
    pub source_label: String,
    pub current_path: String,
    pub files: Vec<SftpFileInfo>,
    pub is_loading: bool,
    pub error: Option<String>,
    pub selected: HashSet<String>,
    pub search_query: String,
    pub show_hidden: bool,
    pub sort_column: SftpSortColumn,
    pub sort_order: SftpSortOrder,
    pub show_source_picker: bool,
    pub show_actions_menu: bool,
    pub show_drive_picker: bool,
    pub show_path_picker: bool,
    pub history: Vec<String>,
    pub history_index: usize,
}

impl SftpPaneState {
    pub fn new(
        source_id: impl Into<String>,
        source_label: impl Into<String>,
        initial_path: impl Into<String>,
    ) -> Self {
        let p = initial_path.into();
        Self {
            source_id: source_id.into(),
            source_label: source_label.into(),
            current_path: p.clone(),
            files: Vec::new(),
            is_loading: false,
            error: None,
            selected: HashSet::new(),
            search_query: String::new(),
            show_hidden: false,
            sort_column: SftpSortColumn::Name,
            sort_order: SftpSortOrder::Ascending,
            show_source_picker: false,
            show_actions_menu: false,
            show_drive_picker: false,
            show_path_picker: false,
            history: vec![p],
            history_index: 0,
        }
    }

    pub fn can_go_back(&self) -> bool {
        self.history_index > 0
    }

    pub fn can_go_forward(&self) -> bool {
        self.history_index + 1 < self.history.len()
    }

    /// Return filtered and sorted files according to search_query, show_hidden, and sort column/order.
    pub fn visible_files(&self) -> Vec<SftpFileInfo> {
        let mut list: Vec<SftpFileInfo> = self
            .files
            .iter()
            .filter(|f| {
                if !self.show_hidden && f.name.starts_with('.') {
                    return false;
                }
                if !self.search_query.is_empty()
                    && !f
                        .name
                        .to_lowercase()
                        .contains(&self.search_query.to_lowercase())
                {
                    return false;
                }
                true
            })
            .cloned()
            .collect();

        list.sort_by(|a, b| {
            if a.is_dir != b.is_dir {
                return b.is_dir.cmp(&a.is_dir);
            }
            let ord = match self.sort_column {
                SftpSortColumn::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                SftpSortColumn::Size => a.size.cmp(&b.size),
                SftpSortColumn::ModTime => a.mod_time.cmp(&b.mod_time),
            };
            match self.sort_order {
                SftpSortOrder::Ascending => ord,
                SftpSortOrder::Descending => ord.reverse(),
            }
        });

        list
    }
}

static SFTP_TX_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

/// Generate unique transfer ID.
pub fn next_transfer_id() -> String {
    format!(
        "tx-{}",
        SFTP_TX_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    )
}

/// Modal state for SFTP file operations.
#[derive(Debug, Clone, PartialEq)]
pub enum SftpModalState {
    NewFolder {
        pane: SftpActivePane,
        name: String,
        error: Option<String>,
    },
    Rename {
        pane: SftpActivePane,
        old_name: String,
        new_name: String,
        error: Option<String>,
    },
    DeleteConfirm {
        pane: SftpActivePane,
        targets: Vec<String>,
        error: Option<String>,
    },
    Conflict {
        src_pane: SftpActivePane,
        dst_pane: SftpActivePane,
        conflict_name: String,
        remaining_transfers: Vec<String>,
    },
}

/// Transfer item tracking an active or past transfer.
#[derive(Debug, Clone, PartialEq)]
pub struct SftpTransferItem {
    pub id: String,
    pub name: String,
    pub from_source: String,
    pub to_source: String,
    pub bytes_transferred: i64,
    pub total_bytes: i64,
    pub status: String,
    pub error: Option<String>,
}

impl From<SftpTransferStatus> for SftpTransferItem {
    fn from(st: SftpTransferStatus) -> Self {
        Self {
            id: st.id,
            name: "Transfer".to_string(),
            from_source: String::new(),
            to_source: String::new(),
            bytes_transferred: st.bytes_transferred,
            total_bytes: st.total_bytes,
            status: st.status,
            error: st.error,
        }
    }
}

/// Context menu state for a file/folder row.
#[derive(Debug, Clone, PartialEq)]
pub struct SftpContextMenu {
    pub pane: SftpActivePane,
    pub filename: String,
    pub is_dir: bool,
    pub position: (f32, f32),
}

/// Dragged item payload for inter-pane drag and drop.
#[derive(Debug, Clone, PartialEq)]
pub struct SftpDraggedItem {
    pub source_pane: SftpActivePane,
    pub filenames: Vec<String>,
}

/// Manager state orchestrating both panes, modals, and transfers.
#[derive(Debug, Clone)]
pub struct SftpManager {
    pub left_pane: SftpPaneState,
    pub right_pane: SftpPaneState,
    pub focused_pane: SftpActivePane,
    pub modal: Option<SftpModalState>,
    pub context_menu: Option<SftpContextMenu>,
    pub transfers: Vec<SftpTransferItem>,
    pub transfers_drawer_open: bool,
    pub focus_handle: Option<FocusHandle>,
}

/// Return the default local home directory path for file browsing (e.g. C:/Users/name on Windows).
pub fn local_home_path() -> String {
    #[cfg(target_os = "windows")]
    {
        if let Ok(profile) = std::env::var("USERPROFILE") {
            let s = profile.replace('\\', "/");
            if !s.is_empty() {
                return s;
            }
        }
        if let (Ok(drive), Ok(path)) = (std::env::var("HOMEDRIVE"), std::env::var("HOMEPATH")) {
            let s = format!("{}{}", drive, path).replace('\\', "/");
            if !s.is_empty() {
                return s;
            }
        }
        "C:/".to_string()
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::env::var("HOME").unwrap_or_else(|_| "/".to_string())
    }
}

impl Default for SftpManager {
    fn default() -> Self {
        let home = local_home_path();
        Self {
            left_pane: SftpPaneState::new("local", "Local Filesystem", &home),
            right_pane: SftpPaneState::new("local", "Local Filesystem", &home),
            focused_pane: SftpActivePane::Left,
            modal: None,
            context_menu: None,
            transfers: Vec::new(),
            transfers_drawer_open: false,
            focus_handle: None,
        }
    }
}

/// Split a file path into clickable breadcrumb (segment_label, absolute_path_to_segment) pairs.
pub fn split_breadcrumbs(path: &str) -> Vec<(String, String)> {
    let clean = path.trim();
    if clean.is_empty() || clean == "." {
        return vec![(".".to_string(), ".".to_string())];
    }

    let is_windows = clean.len() >= 2 && clean.chars().nth(1) == Some(':');
    let normalized = clean.replace('\\', "/");
    let is_unix_abs = normalized.starts_with('/');

    let mut result = Vec::new();

    if is_windows {
        let parts: Vec<&str> = normalized.split('/').filter(|s| !s.is_empty()).collect();
        if let Some(drive) = parts.first() {
            let mut current_acc = format!("{}/", drive);
            result.push((drive.to_string(), current_acc.clone()));
            for part in &parts[1..] {
                if !current_acc.ends_with('/') {
                    current_acc.push('/');
                }
                current_acc.push_str(part);
                result.push((part.to_string(), current_acc.clone()));
            }
        }
    } else if is_unix_abs {
        result.push(("/".to_string(), "/".to_string()));
        let parts: Vec<&str> = normalized.split('/').filter(|s| !s.is_empty()).collect();
        let mut current_acc = String::new();
        for part in parts {
            current_acc.push('/');
            current_acc.push_str(part);
            result.push((part.to_string(), current_acc.clone()));
        }
    } else {
        let parts: Vec<&str> = normalized.split('/').filter(|s| !s.is_empty()).collect();
        let mut current_acc = String::new();
        for (i, part) in parts.iter().enumerate() {
            if i > 0 {
                current_acc.push('/');
            }
            current_acc.push_str(part);
            result.push((part.to_string(), current_acc.clone()));
        }
    }

    if result.is_empty() {
        vec![(clean.to_string(), clean.to_string())]
    } else {
        result
    }
}

/// Compute the parent directory for a path.
pub fn parent_path(path: &str) -> String {
    let clean = path.trim().replace('\\', "/");
    if clean.is_empty() || clean == "." || clean == "/" {
        return clean;
    }
    if clean.len() <= 3 && clean.chars().nth(1) == Some(':') {
        return "/".to_string();
    }
    let trimmed = clean.trim_end_matches('/');
    if let Some(pos) = trimmed.rfind('/') {
        if pos == 0 {
            "/".to_string()
        } else {
            trimmed[..pos].to_string()
        }
    } else if clean.len() >= 2 && clean.chars().nth(1) == Some(':') {
        "/".to_string()
    } else {
        ".".to_string()
    }
}

/// Join a base path and a child name cleanly with forward slashes.
pub fn join_path(base: &str, child: &str) -> String {
    let b = base.trim().replace('\\', "/");
    if b.is_empty() || b == "." {
        child.to_string()
    } else if b.ends_with('/') {
        format!("{}{}", b, child)
    } else {
        format!("{}/{}", b, child)
    }
}

/// Format file size into a human-readable string (B, KB, MB, GB).
pub fn format_file_size(bytes: i64) -> String {
    if bytes < 0 {
        return "-".to_string();
    }
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    let b = bytes as f64;
    if b >= GB {
        format!("{:.1} GB", b / GB)
    } else if b >= MB {
        format!("{:.1} MB", b / MB)
    } else if b >= KB {
        format!("{:.1} KB", b / KB)
    } else {
        format!("{} B", bytes)
    }
}

/// Format Unix/POSIX permissions mode into standard string (e.g. `drwxrwxrwx`).
pub fn format_permissions(mode: u32, is_dir: bool) -> String {
    let r = |m: u32| if (mode & m) != 0 { 'r' } else { '-' };
    let w = |m: u32| if (mode & m) != 0 { 'w' } else { '-' };
    let x = |m: u32| if (mode & m) != 0 { 'x' } else { '-' };
    let mut s = String::with_capacity(10);
    s.push(if is_dir { 'd' } else { '-' });
    s.push(r(0o400));
    s.push(w(0o200));
    s.push(x(0o100));
    s.push(r(0o040));
    s.push(w(0o020));
    s.push(x(0o010));
    s.push(r(0o004));
    s.push(w(0o002));
    s.push(x(0o001));
    s
}

/// Format ISO-8601 modTime string into friendly US format `M/D/YYYY, h:mm A`.
pub fn format_date_modified(iso: &str) -> String {
    let clean = iso.trim();
    if clean.is_empty() {
        return String::new();
    }
    // Pattern: YYYY-MM-DDTHH:MM:SS or YYYY-MM-DD HH:MM:SS
    if clean.len() >= 16 && &clean[4..5] == "-" && &clean[7..8] == "-" {
        let year = &clean[0..4];
        let month = clean[5..7].trim_start_matches('0');
        let day = clean[8..10].trim_start_matches('0');
        if let (Ok(h), Ok(m)) = (clean[11..13].parse::<u32>(), clean[14..16].parse::<u32>()) {
            let ampm = if h >= 12 { "PM" } else { "AM" };
            let h12 = if h == 0 {
                12
            } else if h > 12 {
                h - 12
            } else {
                h
            };
            return format!("{month}/{day}/{year}, {h12}:{m:02} {ampm}");
        }
    }
    if clean.len() >= 19 {
        clean[..19].replace('T', " ")
    } else {
        clean.to_string()
    }
}

/// Query available disk drives on the local machine (Windows drives, or root on Unix).
pub fn get_available_drives() -> Vec<String> {
    #[cfg(target_os = "windows")]
    {
        extern "system" {
            fn GetLogicalDrives() -> u32;
        }
        let mask = unsafe { GetLogicalDrives() };
        let mut drives = Vec::new();
        for i in 0..26 {
            if (mask & (1 << i)) != 0 {
                let letter = (b'A' + i as u8) as char;
                drives.push(format!("{}:", letter));
            }
        }
        if drives.is_empty() {
            drives.push("C:".to_string());
        }
        drives
    }
    #[cfg(not(target_os = "windows"))]
    {
        vec!["/".to_string()]
    }
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
    pub show_new_tab_popover: bool,
    pub quick_connect_query: String,
    pub show_hosts_catalog: bool,
    pub connections: Vec<Connection>,
    pub search_query: String,
    pub selected_tag: Option<String>,
    pub is_loading_connections: bool,
    pub ssh_keys: Vec<SshKey>,
    pub is_loading_keys: bool,
    pub connection_modal: Option<ConnectionFormState>,
    pub connection_sheet_closing: bool,
    pub show_import_modal: bool,
    pub import_payload: String,
    pub notification: Option<String>,
    pub notification_is_error: bool,
    pub notification_serial: u64,
    pub show_add_key_modal: bool,
    pub add_key_sheet_closing: bool,
    pub edit_key_modal: Option<EditKeyFormState>,
    pub edit_key_sheet_closing: bool,
    pub new_key_name: String,
    pub new_key_pem: String,
    pub add_key_error: Option<String>,
    pub pending_passphrase_conn: Option<(String, String)>,
    pub passphrase_input: String,
    pub passphrase_error: Option<String>,
    pub passphrase_cache: std::collections::HashMap<String, String>,
    pub sftp_manager: SftpManager,
    pub forwards: Vec<PortForward>,
    pub is_loading_forwards: bool,
    pub forward_modal: Option<ForwardFormState>,
    pub forward_sheet_closing: bool,
    pub delete_forward_target: Option<PortForward>,
    pub terminal_font_size: f32,
    pub terminal_font_family: String,
    pub cursor_style: String,
    pub cursor_blink: bool,
    pub scrollback: u32,
    pub theme_mode_filter: String,
    pub show_theme_mode_picker: bool,
    pub show_cursor_style_picker: bool,
    pub show_scrollback_picker: bool,
    pub show_font_dialog: bool,
    pub font_dialog_family: String,
    pub font_dialog_size: f32,
    pub show_font_dialog_picker: bool,
    pub is_maximized: bool,
    pub backend_path_input: String,
    pub sidebar_open: bool,
    /// Editable form inputs, initialized once the window exists (see
    /// `init_form_inputs`, called from main.rs right after the window opens).
    pub form_inputs: Option<FormInputs>,
}

impl AppState {
    /// Create initial AppState from desktop settings.
    pub fn new(settings: DesktopSettings, spawn_opts: Option<SpawnOptions>) -> Self {
        let theme = settings.theme;
        let backend_path_input = settings
            .backend_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        let terminal_font_size = settings.font_size;
        let terminal_font_family = settings.font_family.clone();
        let cursor_style = settings.cursor_style.clone();
        let cursor_blink = settings.cursor_blink;
        let scrollback = settings.scrollback;
        let theme_mode_filter = settings.theme_mode_filter.clone();
        let is_maximized = settings
            .window_state
            .as_ref()
            .map(|s| s.maximized)
            .unwrap_or(false);

        Self {
            backend_status: BackendStatus::Starting,
            client: None,
            theme,
            active_view: View::Hosts,
            settings,
            spawn_opts,
            session_manager: TerminalSessionManager::new(),
            show_new_tab_popover: false,
            quick_connect_query: String::new(),
            show_hosts_catalog: true,
            connections: Vec::new(),
            search_query: String::new(),
            selected_tag: None,
            is_loading_connections: false,
            ssh_keys: Vec::new(),
            is_loading_keys: false,
            connection_modal: None,
            connection_sheet_closing: false,
            show_import_modal: false,
            import_payload: String::new(),
            notification: None,
            notification_is_error: false,
            notification_serial: 0,
            show_add_key_modal: false,
            add_key_sheet_closing: false,
            edit_key_modal: None,
            edit_key_sheet_closing: false,
            new_key_name: String::new(),
            new_key_pem: String::new(),
            add_key_error: None,
            pending_passphrase_conn: None,
            passphrase_input: String::new(),
            passphrase_error: None,
            passphrase_cache: std::collections::HashMap::new(),
            sftp_manager: SftpManager::default(),
            forwards: Vec::new(),
            is_loading_forwards: false,
            forward_modal: None,
            forward_sheet_closing: false,
            delete_forward_target: None,
            terminal_font_size,
            terminal_font_family: terminal_font_family.clone(),
            cursor_style,
            cursor_blink,
            scrollback,
            theme_mode_filter,
            show_theme_mode_picker: false,
            show_cursor_style_picker: false,
            show_scrollback_picker: false,
            show_font_dialog: false,
            font_dialog_family: terminal_font_family,
            font_dialog_size: terminal_font_size,
            show_font_dialog_picker: false,
            is_maximized,
            backend_path_input,
            sidebar_open: true,
            form_inputs: None,
        }
    }

    /// Retrieve the currently active ThemePreset.
    pub fn current_theme(&self) -> &'static crate::theme::ThemePreset {
        crate::theme::find_theme_preset(&self.settings.theme_preset)
    }

    /// Derive the terminal ColorPalette for the currently active theme preset.
    pub fn current_terminal_palette(&self) -> webterm_terminal::ColorPalette {
        crate::theme::terminal_palette_for_preset(self.current_theme())
    }

    /// Set a new theme preset, synchronize settings, update terminal palette, and trigger live redraw.
    pub fn set_theme_preset(&mut self, preset_id: &str, cx: &mut Context<Self>) {
        self.settings.theme_preset = preset_id.to_string();
        let preset = crate::theme::find_theme_preset(preset_id);
        self.theme = if preset.is_dark {
            SettingsTheme::Dark
        } else {
            SettingsTheme::Light
        };
        self.settings.theme = self.theme;
        let _ = self.settings.save();

        crate::theme::apply_theme(self.theme, cx);

        let palette = self.current_terminal_palette();
        for session in self.session_manager.tabs_mut() {
            if let Some(ref view) = session.view {
                let p = palette.clone();
                view.update(cx, |this, cx| {
                    this.set_palette(p, cx);
                });
            }
        }

        cx.notify();
    }

    pub fn bg_color(&self) -> Rgba {
        self.current_theme().bg()
    }
    pub fn card_bg(&self) -> Rgba {
        self.current_theme().card_bg()
    }
    pub fn card_fg(&self) -> Rgba {
        self.current_theme().card_fg()
    }
    pub fn border_color(&self) -> Rgba {
        self.current_theme().border()
    }
    pub fn text_color(&self) -> Rgba {
        self.current_theme().fg()
    }
    pub fn muted_text(&self) -> Rgba {
        self.current_theme().muted_fg()
    }
    pub fn muted_bg(&self) -> Rgba {
        self.current_theme().muted()
    }
    pub fn primary_color(&self) -> Rgba {
        self.current_theme().primary()
    }
    pub fn primary_fg(&self) -> Rgba {
        self.current_theme().primary_fg()
    }
    pub fn accent_color(&self) -> Rgba {
        self.current_theme().accent()
    }
    pub fn accent_fg(&self) -> Rgba {
        self.current_theme().accent_fg()
    }
    pub fn secondary_bg(&self) -> Rgba {
        self.current_theme().secondary()
    }
    pub fn secondary_fg(&self) -> Rgba {
        self.current_theme().secondary_fg()
    }
    pub fn destructive_color(&self) -> Rgba {
        self.current_theme().destructive()
    }
    pub fn is_dark(&self) -> bool {
        self.current_theme().is_dark
    }

    /// Toggle new tab launcher popover on the plus button.
    pub fn toggle_new_tab_popover(&mut self, cx: &mut Context<Self>) {
        self.show_new_tab_popover = !self.show_new_tab_popover;
        cx.notify();
    }

    /// Open the onboarding / New Tab page (Welcome to WebTerm).
    pub fn open_new_tab_page(&mut self, cx: &mut Context<Self>) {
        self.show_new_tab_popover = false;
        self.show_hosts_catalog = false;
        self.active_view = View::NewTab;
        self.quick_connect_query.clear();
        cx.notify();
    }

    /// Duplicate current active session (either local shell or SSH).
    pub fn duplicate_active_tab(&mut self, cx: &mut Context<Self>) {
        self.show_new_tab_popover = false;
        let active_tab = match self.session_manager.active_tab() {
            Some(t) => t,
            None => {
                cx.notify();
                return;
            }
        };

        let session_type = active_tab.session_type.clone();
        let title = active_tab.title.clone();
        let last_req = active_tab.last_connect_req.clone();

        if session_type == "local" {
            let cwd = last_req.and_then(|r| r.cwd);
            self.open_local_tab_with_cwd(cwd, cx);
        } else if let Some(req) = last_req {
            self.open_ssh_tab(req, &title, cx);
        } else if let Some(conn_id) = active_tab.connection_id.clone() {
            self.connect_to_host(&conn_id, cx);
        } else {
            self.open_local_tab(cx);
        }
    }

    /// Parse and submit quick connect text (e.g. user@host[:port] or saved connection match).
    pub fn quick_connect_submit(&mut self, text: &str, cx: &mut Context<Self>) {
        let input = text.trim();
        if input.is_empty() {
            return;
        }

        // Check if matching a saved connection
        if let Some(saved) = self
            .connections
            .iter()
            .find(|c| {
                c.label.eq_ignore_ascii_case(input)
                    || format!("{}@{}:{}", c.username, c.host, c.port).eq_ignore_ascii_case(input)
                    || format!("{}@{}", c.username, c.host).eq_ignore_ascii_case(input)
                    || c.host.eq_ignore_ascii_case(input)
            })
            .cloned()
        {
            self.connect_to_host(&saved.id, cx);
            self.quick_connect_query.clear();
            return;
        }

        // Parse user@host[:port]
        let at_idx = match input.find('@') {
            Some(i) => i,
            None => return,
        };

        let username = input[..at_idx].trim().to_string();
        let rest = input[at_idx + 1..].trim();
        if username.is_empty() || rest.is_empty() {
            return;
        }

        let (host, port) = if let Some(colon_idx) = rest.find(':') {
            let h = rest[..colon_idx].trim().to_string();
            let p = rest[colon_idx + 1..].trim().parse::<u16>().unwrap_or(22);
            (h, p)
        } else {
            (rest.to_string(), 22)
        };

        if host.is_empty() {
            return;
        }

        let title = format!("{}@{}", username, host);
        let req = WsConnectRequest::for_quick_connect(host, port, username, "", 80, 24);
        self.open_ssh_tab(req, &title, cx);
        self.quick_connect_query.clear();
    }

    /// Close new tab launcher modal.
    pub fn close_new_tab_modal(&mut self, cx: &mut Context<Self>) {
        cx.notify();
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
                // Encrypted key without a cached passphrase: prompt first,
                // matching the web client's needs-passphrase flow.
                let needs_passphrase = self
                    .ssh_keys
                    .iter()
                    .find(|k| &k.id == key_id)
                    .map(|k| k.has_passphrase)
                    .unwrap_or(false);
                if needs_passphrase {
                    self.prompt_passphrase_for_connection(conn_id, key_id, cx);
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
    pub fn prompt_passphrase_for_connection(&mut self, conn_id: &str, key_id: &str, cx: &mut Context<Self>) {
        if let Some(cached_pass) = self.passphrase_cache.get(key_id).cloned() {
            self.connect_to_host_with_passphrase(conn_id, Some(&cached_pass), cx);
            return;
        }

        self.pending_passphrase_conn = Some((conn_id.to_string(), key_id.to_string()));
        self.passphrase_error = None;
        cx.notify();
    }

    /// Submit passphrase entered in PassphraseModal, caching it in memory for this session.
    pub fn submit_passphrase(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let (conn_id, key_id) = match self.pending_passphrase_conn.take() {
            Some(pair) => pair,
            None => return,
        };

        let pass = AppState::input_value(&self.inputs().passphrase, cx);
        let pass = pass.trim().to_string();
        if !pass.is_empty() {
            self.passphrase_cache.insert(key_id, pass.clone());
        }

        AppState::set_input_value(&self.inputs().passphrase, "", window, cx);
        self.passphrase_error = None;
        self.connect_to_host_with_passphrase(&conn_id, Some(&pass), cx);
        cx.notify();
    }

    /// Cancel passphrase entry dialog.
    pub fn cancel_passphrase(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.pending_passphrase_conn = None;
        AppState::set_input_value(&self.inputs().passphrase, "", window, cx);
        self.passphrase_error = None;
        cx.notify();
    }

    /// Run `done` once a sheet's exit animation has finished playing.
    fn after_sheet_exit(cx: &mut Context<Self>, done: impl FnOnce(&mut Self) + 'static) {
        let view_weak = cx.entity().downgrade();
        cx.spawn(move |_view, cx: &mut AsyncApp| {
            let view_weak = view_weak.clone();
            let cx_handle = cx.clone();
            async move {
                cx_handle
                    .background_executor()
                    .timer(std::time::Duration::from_millis(
                        crate::views::sheet::SHEET_ANIM_MS + 40,
                    ))
                    .await;
                cx_handle.update(|cx: &mut App| {
                    if let Some(app) = view_weak.upgrade() {
                        app.update(cx, |this, cx| {
                            done(this);
                            cx.notify();
                        });
                    }
                });
            }
        })
        .detach();
    }

    /// Open Add SSH Key sheet (right-side push-aside panel).
    pub fn open_add_key_modal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.show_add_key_modal = true;
        self.add_key_sheet_closing = false;
        // Only one sheet open at a time; force-close the others instantly.
        self.connection_modal = None;
        self.connection_sheet_closing = false;
        self.edit_key_modal = None;
        self.edit_key_sheet_closing = false;
        self.forward_modal = None;
        self.forward_sheet_closing = false;
        let inputs = self.inputs();
        AppState::set_input_value(&inputs.new_key_name, "", window, cx);
        AppState::set_textarea_value(&inputs.new_key_pem, "", window, cx);
        self.add_key_error = None;
        cx.notify();
    }

    /// Play the Add SSH Key sheet's exit animation, then unmount it.
    pub fn close_add_key_modal(&mut self, cx: &mut Context<Self>) {
        if !self.show_add_key_modal || self.add_key_sheet_closing {
            return;
        }
        self.add_key_sheet_closing = true;
        Self::after_sheet_exit(cx, |this| {
            // Re-opening during the exit cancels the pending unmount.
            if this.add_key_sheet_closing {
                this.show_add_key_modal = false;
                this.add_key_sheet_closing = false;
            }
        });
        cx.notify();
    }

    /// Open Edit SSH Key sheet (rename / replace key material).
    pub fn open_edit_key_modal(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        let Some(key) = self.ssh_keys.iter().find(|k| k.id == id) else {
            return;
        };
        self.edit_key_modal = Some(EditKeyFormState::new(key));
        let inputs = self.inputs();
        AppState::set_input_value(&inputs.edit_key_name, &key.name, window, cx);
        AppState::set_textarea_value(&inputs.edit_key_pem, "", window, cx);
        self.edit_key_sheet_closing = false;
        self.connection_modal = None;
        self.connection_sheet_closing = false;
        self.show_add_key_modal = false;
        self.add_key_sheet_closing = false;
        self.forward_modal = None;
        self.forward_sheet_closing = false;
        cx.notify();
    }

    /// Play the Edit SSH Key sheet's exit animation, then unmount it.
    pub fn close_edit_key_modal(&mut self, cx: &mut Context<Self>) {
        if !self.edit_key_modal.is_some() || self.edit_key_sheet_closing {
            return;
        }
        self.edit_key_sheet_closing = true;
        Self::after_sheet_exit(cx, |this| {
            if this.edit_key_sheet_closing {
                this.edit_key_modal = None;
                this.edit_key_sheet_closing = false;
            }
        });
        cx.notify();
    }

    /// Save Edit SSH Key form (rename and optionally replace key material).
    pub fn save_edit_key_form(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let name_value = AppState::input_value(&self.inputs().edit_key_name, cx);
        let pem_value = AppState::input_value_textarea(&self.inputs().edit_key_pem, cx);
        if let Some(f) = &mut self.edit_key_modal {
            f.name = name_value;
            f.new_pem = pem_value;
        }
        let form = match &mut self.edit_key_modal {
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
                if let Some(f) = &mut self.edit_key_modal {
                    f.error_message = Some("Backend client is not connected".to_string());
                    cx.notify();
                }
                return;
            }
        };

        let key_id = form.key_id.clone();
        let has_new_key = !form.new_pem.trim().is_empty();
        let req = UpdateKeyRequest {
            name: form.name.trim().to_string(),
            key_base64: if has_new_key {
                Some(encode_base64(form.new_pem.trim().as_bytes()))
            } else {
                None
            },
        };

        let view_weak = cx.entity().downgrade();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Result<SshKey, String>>();

        TOKIO_RT.spawn(async move {
            match client.update_key(&key_id, &req).await {
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
                                        let msg = if has_new_key {
                                            "SSH key updated with new key material".to_string()
                                        } else {
                                            format!("SSH key '{}' updated", key.name)
                                        };
                                        this.close_edit_key_modal(cx);
                                        this.push_notification(msg, false, cx);
                                        this.fetch_ssh_keys(cx);
                                    }
                                    Err(e) => {
                                        if let Some(f) = &mut this.edit_key_modal {
                                            f.error_message =
                                                Some(format!("Failed to update key: {e}"));
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

    /// Upload and save new SSH Key to backend pool.
    pub fn create_ssh_key(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let name_value = AppState::input_value(&self.inputs().new_key_name, cx);
        let pem_value = AppState::input_value_textarea(&self.inputs().new_key_pem, cx);
        self.new_key_name = name_value.clone();
        self.new_key_pem = pem_value.clone();

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
                                        this.close_add_key_modal(cx);
                                        this.push_notification(format!("SSH Key '{}' added", key.name), false, cx);
                                        this.fetch_ssh_keys(cx);
                                    }
                                    Err(e) => {
                                        this.add_key_error =
                                            Some(format!("Failed to add key: {e}"));
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
                                        this.push_notification("SSH Key deleted".to_string(), false, cx);
                                    }
                                    Err(e) => {
                                        this.push_notification(format!("Delete key failed: {e}"), false, cx);
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

    /// Toggle left navigation sidebar visibility.
    pub fn toggle_sidebar(&mut self, cx: &mut Context<Self>) {
        self.sidebar_open = !self.sidebar_open;
        cx.notify();
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
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Result<Vec<SshKey>, String>>();

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
    pub fn open_create_connection_modal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.connection_modal = Some(ConnectionFormState::new_create());
        self.connection_sheet_closing = false;
        self.close_other_sheets_for("connection");
        let inputs = self.inputs();
        AppState::set_input_value(&inputs.conn_label, "", window, cx);
        AppState::set_input_value(&inputs.conn_host, "", window, cx);
        AppState::set_input_value(&inputs.conn_port, "22", window, cx);
        AppState::set_input_value(&inputs.conn_username, "root", window, cx);
        AppState::set_input_value(&inputs.conn_password, "", window, cx);
        AppState::set_input_value(&inputs.conn_tags, "", window, cx);
        self.fetch_ssh_keys(cx);
        cx.notify();
    }

    /// Open edit connection sheet.
    pub fn open_edit_connection_modal(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(conn) = self.connections.iter().find(|c| c.id == id).cloned() {
            self.connection_modal = Some(ConnectionFormState::new_edit(&conn));
            self.connection_sheet_closing = false;
            self.close_other_sheets_for("connection");
            let inputs = self.inputs();
            AppState::set_input_value(&inputs.conn_label, &conn.label, window, cx);
            AppState::set_input_value(&inputs.conn_host, &conn.host, window, cx);
            AppState::set_input_value(&inputs.conn_port, &conn.port.to_string(), window, cx);
            AppState::set_input_value(&inputs.conn_username, &conn.username, window, cx);
            AppState::set_input_value(&inputs.conn_password, "", window, cx);
            AppState::set_input_value(&inputs.conn_tags, &conn.tags.join(", "), window, cx);
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
                                            if form.mode
                                                == ConnectionModalMode::Edit(full_conn.id.clone())
                                            {
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

    /// Play the connection sheet's exit animation, then unmount it.
    pub fn close_connection_modal(&mut self, cx: &mut Context<Self>) {
        if !self.connection_modal.is_some() || self.connection_sheet_closing {
            return;
        }
        self.connection_sheet_closing = true;
        Self::after_sheet_exit(cx, |this| {
            // Re-opening during the exit cancels the pending unmount.
            if this.connection_sheet_closing {
                this.connection_modal = None;
                this.connection_sheet_closing = false;
            }
        });
        cx.notify();
    }

    /// Force-close every sheet except `keep` ("connection" | "edit_key" |
    /// "add_key" | "forward"); mirrors wa-bot's drawers being mutually
    /// exclusive.
    pub fn close_other_sheets_for(&mut self, keep: &str) {
        if keep != "connection" {
            self.connection_modal = None;
            self.connection_sheet_closing = false;
        }
        if keep != "edit_key" {
            self.edit_key_modal = None;
            self.edit_key_sheet_closing = false;
        }
        if keep != "add_key" {
            self.show_add_key_modal = false;
            self.add_key_sheet_closing = false;
        }
        if keep != "forward" {
            self.forward_modal = None;
            self.forward_sheet_closing = false;
        }
    }

    /// Save connection form (Create or Update).
    pub fn save_connection_form(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let (label, host, port, username, password, tags) = {
            let inputs = self.inputs();
            (
                AppState::input_value(&inputs.conn_label, cx),
                AppState::input_value(&inputs.conn_host, cx),
                AppState::input_value(&inputs.conn_port, cx),
                AppState::input_value(&inputs.conn_username, cx),
                AppState::input_value(&inputs.conn_password, cx),
                AppState::input_value(&inputs.conn_tags, cx),
            )
        };
        if let Some(f) = &mut self.connection_modal {
            f.label = label;
            f.host = host;
            f.port = if port.trim().is_empty() { "22".to_string() } else { port };
            f.username = if username.trim().is_empty() { "root".to_string() } else { username };
            f.password = password;
            f.tags = tags;
        }
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
                                        this.close_connection_modal(cx);
                                        this.push_notification(msg, false, cx);
                                        this.fetch_connections(cx);
                                    }
                                    Err(e) => {
                                        if let Some(f) = &mut this.connection_modal {
                                            f.error_message =
                                                Some(format!("Error saving host: {e}"));
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
                                        this.push_notification("Connection deleted".to_string(), false, cx);
                                    }
                                    Err(e) => {
                                        this.push_notification(format!("Delete failed: {e}"), false, cx);
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
                                        this.push_notification(format!(
                                            "Exported {count} connection(s) to webterm-connections-export.json"
                                        ), false, cx);
                                    }
                                    Err(e) => {
                                        this.push_notification(format!("Export failed: {e}"), false, cx);
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
                self.push_notification("Backend client is not connected".to_string(), false, cx);
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
                            self.push_notification(format!("JSON parsing error from export file: {e}"), true, cx);
                            cx.notify();
                            return;
                        }
                    },
                    Err(e) => {
                        self.push_notification(format!("Failed to read export file: {e}"), true, cx);
                        cx.notify();
                        return;
                    }
                }
            } else {
                self.push_notification("Please paste JSON array or create webterm-connections-export.json".to_string(), false, cx);
                cx.notify();
                return;
            }
        } else {
            match serde_json::from_str::<Vec<Connection>>(payload) {
                Ok(c) => c,
                Err(e) => {
                    self.push_notification(format!("Invalid JSON array: {e}"), true, cx);
                    cx.notify();
                    return;
                }
            }
        };

        let view_weak = cx.entity().downgrade();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Result<ImportResult, String>>();

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
                                        this.push_notification(format!(
                                            "Import finished: {} imported, {} skipped",
                                            result.imported, result.skipped
                                        ), false, cx);
                                        this.fetch_connections(cx);
                                    }
                                    Err(e) => {
                                        this.push_notification(format!("Import error: {e}"), true, cx);
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

    /// Set the notification toast; auto-dismisses after 4 seconds unless a
    /// newer toast replaces it first.
    pub fn push_notification(&mut self, message: String, is_error: bool, cx: &mut Context<Self>) {
        self.notification_serial += 1;
        let serial = self.notification_serial;
        self.notification = Some(message);
        self.notification_is_error = is_error;
        let view_weak = cx.entity().downgrade();
        cx.spawn(move |_view, cx: &mut AsyncApp| {
            let view_weak = view_weak.clone();
            let cx_handle = cx.clone();
            async move {
                cx_handle
                    .background_executor()
                    .timer(std::time::Duration::from_millis(4000))
                    .await;
                cx_handle.update(|cx: &mut App| {
                    if let Some(app) = view_weak.upgrade() {
                        app.update(cx, |this, cx| {
                            if this.notification_serial == serial {
                                this.notification = None;
                                this.notification_is_error = false;
                                cx.notify();
                            }
                        });
                    }
                });
            }
        })
        .detach();
        cx.notify();
    }

    /// Dismiss the notification toast.
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
                    stderr_tail: "Please build the backend binary: scripts/build-test-backend"
                        .to_string(),
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
                        webterm_supervisor::SupervisorError::Failed {
                            reason,
                            stderr_tail,
                        } => (reason, stderr_tail),
                        other => (other.to_string(), String::new()),
                    };
                    let _ = tx.send(SupervisorEvent::Failed {
                        reason,
                        stderr_tail,
                    });
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
                                        this.fetch_forwards(cx);
                                    }
                                    SupervisorEvent::Failed {
                                        reason,
                                        stderr_tail,
                                    } => {
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
        })
        .detach();
    }

    /// Restore detached sessions from backend or open a default local tab.
    pub fn restore_sessions_or_default(&mut self, cx: &mut Context<Self>) {
        let client = match self.client.clone() {
            Some(c) => c,
            None => return,
        };

        let saved_sessions = self.settings.open_sessions.clone();
        let view_weak = cx.entity().downgrade();

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Vec<SessionInfo>>();

        TOKIO_RT.spawn(async move {
            let backend_sessions = client.list_sessions().await.unwrap_or_default();
            let _ = tx.send(backend_sessions);
        });

        cx.spawn(move |_view, cx: &mut AsyncApp| {
            let view_weak = view_weak.clone();
            let cx_handle = cx.clone();
            async move {
                let backend_sessions = rx.recv().await.unwrap_or_default();

                let mut to_attach: Vec<(String, String, String, Option<String>)> = Vec::new();

                if !saved_sessions.is_empty() {
                    for saved in &saved_sessions {
                        if let Some(matched) =
                            backend_sessions.iter().find(|s| s.id == saved.session_id)
                        {
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
                                    let palette = this.current_terminal_palette();
                                    let mut tab = TerminalTab::new(
                                        tab_id, title, stype, conn_id, palette, cx,
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
        })
        .detach();
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

    /// Open a new local shell terminal tab using the default working directory.
    pub fn open_local_tab(&mut self, cx: &mut Context<Self>) {
        self.open_local_tab_with_cwd(None, cx);
    }

    /// Open a new local shell terminal tab with an optional initial working directory.
    pub fn open_local_tab_with_cwd(&mut self, cwd: Option<String>, cx: &mut Context<Self>) {
        let client = match self.client.clone() {
            Some(c) => c,
            None => return,
        };

        let tab_id = self.session_manager.alloc_tab_id();
        let palette = self.current_terminal_palette();
        let mut tab = TerminalTab::new(
            tab_id,
            "Local Shell",
            "local",
            Some("local".to_string()),
            palette,
            cx,
        );
        let connect_req = WsConnectRequest::for_local_with_cwd(80, 24, cwd);
        tab.last_connect_req = Some(connect_req.clone());
        self.show_hosts_catalog = false;
        self.active_view = View::Hosts;
        self.session_manager.add_tab(tab);
        cx.notify();

        let view_weak = cx.entity().downgrade();
        let (tx, mut rx) =
            tokio::sync::mpsc::unbounded_channel::<Result<TerminalWsHandle, String>>();

        TOKIO_RT.spawn(async move {
            match client.connect_terminal(connect_req).await {
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
        })
        .detach();
    }

    /// Open a new SSH terminal tab with given connection request.
    #[allow(dead_code)]
    pub fn open_ssh_tab(&mut self, req: WsConnectRequest, title: &str, cx: &mut Context<Self>) {
        let client = match self.client.clone() {
            Some(c) => c,
            None => return,
        };

        let tab_id = self.session_manager.alloc_tab_id();
        let palette = self.current_terminal_palette();
        let mut tab =
            TerminalTab::new(tab_id, title, "ssh", req.connection_id.clone(), palette, cx);
        tab.last_connect_req = Some(req.clone());
        self.show_hosts_catalog = false;
        self.active_view = View::Hosts;
        self.session_manager.add_tab(tab);
        cx.notify();

        let view_weak = cx.entity().downgrade();
        let (tx, mut rx) =
            tokio::sync::mpsc::unbounded_channel::<Result<TerminalWsHandle, String>>();

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
        })
        .detach();
    }

    /// Invoked when a tab's WebSocket transport connection terminates unexpectedly.
    pub fn on_tab_dropped(&mut self, tab_id: u64, cx: &mut Context<Self>) {
        let tab = match self
            .session_manager
            .tabs_mut()
            .iter_mut()
            .find(|t| t.id == tab_id)
        {
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
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<()>();
        TOKIO_RT.spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(delay_secs as u64)).await;
            let _ = tx.send(());
        });

        cx.spawn(move |_view, cx: &mut AsyncApp| {
            let view_weak = view_weak.clone();
            let cx_handle = cx.clone();
            async move {
                if rx.recv().await.is_some() {
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
            }
        })
        .detach();
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
        let (tx, mut rx) =
            tokio::sync::mpsc::unbounded_channel::<Result<TerminalWsHandle, String>>();

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
                                            this.schedule_reconnect(
                                                tab_id,
                                                next_attempt,
                                                next_secs,
                                                cx,
                                            );
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
        })
        .detach();
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

    /// Set application theme, apply to GPUI context, propagate palette across all terminal tabs, and persist.
    pub fn set_theme(&mut self, theme: SettingsTheme, cx: &mut Context<Self>) {
        self.theme = theme;
        self.settings.theme = self.theme;
        let current_preset = self.current_theme();
        if theme == SettingsTheme::Dark && !current_preset.is_dark {
            self.settings.theme_preset = "default-dark".to_string();
        } else if theme == SettingsTheme::Light && current_preset.is_dark {
            self.settings.theme_preset = "default-light".to_string();
        }
        let _ = self.settings.save();

        crate::theme::apply_theme(self.theme, cx);

        let palette = self.current_terminal_palette();

        for session in self.session_manager.tabs_mut() {
            if let Some(ref view) = session.view {
                let p = palette.clone();
                view.update(cx, |this, cx| {
                    this.set_palette(p, cx);
                });
            }
        }

        cx.notify();
    }

    /// Toggle theme between Dark and Light.
    pub fn toggle_theme(&mut self, cx: &mut Context<Self>) {
        let next = crate::theme::toggle_theme(self.theme);
        self.set_theme(next, cx);
    }

    /// Update terminal font size and propagate to all active terminal sessions.
    pub fn set_terminal_font_size(&mut self, size: f32, cx: &mut Context<Self>) {
        self.terminal_font_size = size.clamp(10.0, 24.0);
        self.settings.font_size = self.terminal_font_size;
        let _ = self.settings.save();
        let px_size = px(self.terminal_font_size);
        for session in self.session_manager.tabs_mut() {
            if let Some(ref view) = session.view {
                view.update(cx, |this, cx| {
                    this.set_font_size(px_size, cx);
                });
            }
        }
        cx.notify();
    }

    /// Update terminal font family & size and propagate to all active terminal sessions.
    pub fn set_terminal_font(&mut self, family: String, size: f32, cx: &mut Context<Self>) {
        self.terminal_font_family = family.clone();
        self.terminal_font_size = size.clamp(10.0, 24.0);
        self.settings.font_family = family.clone();
        self.settings.font_size = self.terminal_font_size;
        let _ = self.settings.save();

        let px_size = px(self.terminal_font_size);
        for session in self.session_manager.tabs_mut() {
            if let Some(ref view) = session.view {
                let f = family.clone();
                view.update(cx, |this, cx| {
                    this.set_font(f, px_size, cx);
                });
            }
        }
        cx.notify();
    }

    /// Set theme mode filter ("all", "dark", "light") and auto-match active theme.
    pub fn set_theme_mode_filter(&mut self, mode: &str, cx: &mut Context<Self>) {
        self.theme_mode_filter = mode.to_string();
        self.settings.theme_mode_filter = mode.to_string();
        let _ = self.settings.save();
        self.show_theme_mode_picker = false;

        // Auto-match theme when switching filter mode
        if mode != "all" {
            let current_preset = self.current_theme();
            let target_dark = mode == "dark";
            if current_preset.is_dark != target_dark {
                let current_id = current_preset.id;
                let base = current_id
                    .trim_end_matches("-dark")
                    .trim_end_matches("-light");
                let counterpart = if target_dark {
                    format!("{base}-dark")
                } else {
                    format!("{base}-light")
                };
                if crate::theme::THEME_PRESETS
                    .iter()
                    .any(|p| p.id == counterpart)
                {
                    self.set_theme_preset(&counterpart, cx);
                } else if let Some(first_match) = crate::theme::THEME_PRESETS
                    .iter()
                    .find(|p| p.is_dark == target_dark)
                {
                    self.set_theme_preset(first_match.id, cx);
                }
            }
        }
        cx.notify();
    }

    /// Set cursor style ("block", "underline", "bar").
    pub fn set_cursor_style(&mut self, style: String, cx: &mut Context<Self>) {
        self.cursor_style = style.clone();
        self.settings.cursor_style = style;
        self.show_cursor_style_picker = false;
        let _ = self.settings.save();
        cx.notify();
    }

    /// Set cursor blink toggle.
    pub fn set_cursor_blink(&mut self, blink: bool, cx: &mut Context<Self>) {
        self.cursor_blink = blink;
        self.settings.cursor_blink = blink;
        let _ = self.settings.save();
        cx.notify();
    }

    /// Set scrollback buffer lines (1000, 5000, 10000, 50000, 0).
    pub fn set_scrollback(&mut self, lines: u32, cx: &mut Context<Self>) {
        self.scrollback = lines;
        self.settings.scrollback = lines;
        self.show_scrollback_picker = false;
        let _ = self.settings.save();
        cx.notify();
    }

    /// Set a custom backend executable path override and save to settings.
    pub fn set_backend_path_override(&mut self, path_str: &str, cx: &mut Context<Self>) {
        let trimmed = path_str.trim();
        if trimmed.is_empty() {
            self.settings.backend_path = None;
            self.backend_path_input.clear();
            self.push_notification("Reset backend path to default bundled binary".to_string(), false, cx);
        } else {
            self.settings.backend_path = Some(std::path::PathBuf::from(trimmed));
            self.backend_path_input = trimmed.to_string();
            self.push_notification("Backend path override saved. Restart app to launch with updated binary."
                    .to_string(), false, cx);
        }
        let _ = self.settings.save();
        cx.notify();
    }

    /// Reset backend executable path override to default bundled binary.
    pub fn reset_backend_path_override(&mut self, cx: &mut Context<Self>) {
        self.settings.backend_path = None;
        self.backend_path_input.clear();
        let _ = self.settings.save();
        self.push_notification("Reset backend path to default bundled binary".to_string(), false, cx);
        cx.notify();
    }

    // ==========================================
    // SFTP Dual-Pane File Manager Methods
    // ==========================================

    /// Get reference to pane state.
    pub fn sftp_pane(&self, pane: SftpActivePane) -> &SftpPaneState {
        match pane {
            SftpActivePane::Left => &self.sftp_manager.left_pane,
            SftpActivePane::Right => &self.sftp_manager.right_pane,
        }
    }

    /// Get mutable reference to pane state.
    pub fn sftp_pane_mut(&mut self, pane: SftpActivePane) -> &mut SftpPaneState {
        match pane {
            SftpActivePane::Left => &mut self.sftp_manager.left_pane,
            SftpActivePane::Right => &mut self.sftp_manager.right_pane,
        }
    }

    /// Set focused pane.
    pub fn sftp_focus_pane(&mut self, pane: SftpActivePane, cx: &mut Context<Self>) {
        self.sftp_manager.focused_pane = pane;
        cx.notify();
    }

    /// Toggle source picker dropdown for a pane.
    pub fn sftp_toggle_source_picker(&mut self, pane: SftpActivePane, cx: &mut Context<Self>) {
        let state = self.sftp_pane_mut(pane);
        state.show_source_picker = !state.show_source_picker;
        if state.show_source_picker {
            state.show_actions_menu = false;
            state.show_drive_picker = false;
        }
        cx.notify();
    }

    /// Toggle actions dropdown menu for a pane.
    pub fn sftp_toggle_actions_menu(&mut self, pane: SftpActivePane, cx: &mut Context<Self>) {
        let state = self.sftp_pane_mut(pane);
        state.show_actions_menu = !state.show_actions_menu;
        if state.show_actions_menu {
            state.show_source_picker = false;
            state.show_drive_picker = false;
        }
        cx.notify();
    }

    /// Toggle drive picker popover for a pane.
    pub fn sftp_toggle_drive_picker(&mut self, pane: SftpActivePane, cx: &mut Context<Self>) {
        let state = self.sftp_pane_mut(pane);
        state.show_drive_picker = !state.show_drive_picker;
        if state.show_drive_picker {
            state.show_source_picker = false;
            state.show_actions_menu = false;
            state.show_path_picker = false;
        }
        cx.notify();
    }

    /// Toggle breadcrumb ellipsis path picker popover for a pane.
    pub fn sftp_toggle_path_picker(&mut self, pane: SftpActivePane, cx: &mut Context<Self>) {
        let state = self.sftp_pane_mut(pane);
        state.show_path_picker = !state.show_path_picker;
        if state.show_path_picker {
            state.show_source_picker = false;
            state.show_actions_menu = false;
            state.show_drive_picker = false;
        }
        cx.notify();
    }

    /// Change source connection for a pane ("local" or connection ID).
    pub fn sftp_set_source(
        &mut self,
        pane: SftpActivePane,
        conn_id: String,
        cx: &mut Context<Self>,
    ) {
        let label = if conn_id == "local" || conn_id.is_empty() {
            "Local Machine".to_string()
        } else if let Some(conn) = self.connections.iter().find(|c| c.id == conn_id) {
            conn.label.clone()
        } else {
            format!("Host ({})", conn_id)
        };

        let initial_path = if conn_id == "local" || conn_id.is_empty() {
            local_home_path()
        } else {
            ".".to_string()
        };

        let state = self.sftp_pane_mut(pane);
        state.source_id = conn_id;
        state.source_label = label;
        state.current_path = initial_path.clone();
        state.selected.clear();
        state.search_query.clear();
        state.show_source_picker = false;
        state.show_actions_menu = false;
        state.show_drive_picker = false;
        state.show_path_picker = false;
        state.history = vec![initial_path];
        state.history_index = 0;
        self.sftp_load_pane(pane, cx);
    }

    /// Navigate pane to a specified directory path, maintaining history.
    pub fn sftp_navigate(&mut self, pane: SftpActivePane, path: String, cx: &mut Context<Self>) {
        let state = self.sftp_pane_mut(pane);
        if state.history.is_empty() {
            state.history.push(state.current_path.clone());
            state.history_index = 0;
        }
        if state.history_index + 1 < state.history.len() {
            state.history.truncate(state.history_index + 1);
        }
        if state.history.last().map(|s| s.as_str()) != Some(&path) {
            state.history.push(path.clone());
            state.history_index = state.history.len() - 1;
        }
        state.current_path = path;
        state.selected.clear();
        state.search_query.clear();
        state.show_source_picker = false;
        state.show_actions_menu = false;
        state.show_drive_picker = false;
        state.show_path_picker = false;
        self.sftp_load_pane(pane, cx);
    }

    /// Navigate back in directory history.
    pub fn sftp_navigate_back(&mut self, pane: SftpActivePane, cx: &mut Context<Self>) {
        let state = self.sftp_pane_mut(pane);
        if state.history_index > 0 {
            state.history_index -= 1;
            let prev_path = state.history[state.history_index].clone();
            state.current_path = prev_path;
            state.selected.clear();
            state.show_source_picker = false;
            state.show_actions_menu = false;
            state.show_drive_picker = false;
            state.show_path_picker = false;
            self.sftp_load_pane(pane, cx);
        }
    }

    /// Navigate forward in directory history.
    pub fn sftp_navigate_forward(&mut self, pane: SftpActivePane, cx: &mut Context<Self>) {
        let state = self.sftp_pane_mut(pane);
        if state.history_index + 1 < state.history.len() {
            state.history_index += 1;
            let next_path = state.history[state.history_index].clone();
            state.current_path = next_path;
            state.selected.clear();
            state.show_source_picker = false;
            state.show_actions_menu = false;
            state.show_drive_picker = false;
            state.show_path_picker = false;
            self.sftp_load_pane(pane, cx);
        }
    }

    /// Navigate pane to parent directory.
    pub fn sftp_navigate_up(&mut self, pane: SftpActivePane, cx: &mut Context<Self>) {
        let current = self.sftp_pane(pane).current_path.clone();
        let parent = parent_path(&current);
        self.sftp_navigate(pane, parent, cx);
    }

    /// Toggle sort order or switch sort column for a pane.
    pub fn sftp_toggle_sort(
        &mut self,
        pane: SftpActivePane,
        col: SftpSortColumn,
        cx: &mut Context<Self>,
    ) {
        let state = self.sftp_pane_mut(pane);
        if state.sort_column == col {
            state.sort_order = match state.sort_order {
                SftpSortOrder::Ascending => SftpSortOrder::Descending,
                SftpSortOrder::Descending => SftpSortOrder::Ascending,
            };
        } else {
            state.sort_column = col;
            state.sort_order = SftpSortOrder::Ascending;
        }
        cx.notify();
    }

    /// Toggle hidden files visibility in a pane.
    pub fn sftp_toggle_hidden(&mut self, pane: SftpActivePane, cx: &mut Context<Self>) {
        let state = self.sftp_pane_mut(pane);
        state.show_hidden = !state.show_hidden;
        cx.notify();
    }

    /// Toggle item selection in a pane.
    pub fn sftp_toggle_selection(
        &mut self,
        pane: SftpActivePane,
        name: String,
        multi: bool,
        cx: &mut Context<Self>,
    ) {
        self.sftp_manager.focused_pane = pane;
        let state = self.sftp_pane_mut(pane);
        if multi {
            if state.selected.contains(&name) {
                state.selected.remove(&name);
            } else {
                state.selected.insert(name);
            }
        } else {
            let was_only_selected = state.selected.contains(&name) && state.selected.len() == 1;
            state.selected.clear();
            if !was_only_selected {
                state.selected.insert(name);
            }
        }
        cx.notify();
    }

    /// Clear selection in a pane.
    pub fn sftp_clear_selection(&mut self, pane: SftpActivePane, cx: &mut Context<Self>) {
        self.sftp_pane_mut(pane).selected.clear();
        cx.notify();
    }

    /// Select exactly one item in a pane.
    pub fn sftp_select_single(
        &mut self,
        pane: SftpActivePane,
        name: String,
        cx: &mut Context<Self>,
    ) {
        self.sftp_manager.focused_pane = pane;
        let state = self.sftp_pane_mut(pane);
        state.selected.clear();
        state.selected.insert(name);
        cx.notify();
    }

    /// Open context menu for an item in a pane.
    pub fn sftp_open_context_menu(
        &mut self,
        pane: SftpActivePane,
        filename: String,
        is_dir: bool,
        position: (f32, f32),
        cx: &mut Context<Self>,
    ) {
        self.sftp_manager.focused_pane = pane;
        self.sftp_manager.context_menu = Some(SftpContextMenu {
            pane,
            filename,
            is_dir,
            position,
        });
        cx.notify();
    }

    /// Close SFTP context menu.
    pub fn sftp_close_context_menu(&mut self, cx: &mut Context<Self>) {
        self.sftp_manager.context_menu = None;
        cx.notify();
    }

    /// Copy item path to clipboard.
    pub fn sftp_copy_path(&mut self, pane: SftpActivePane, filename: &str, cx: &mut Context<Self>) {
        let current_path = self.sftp_pane(pane).current_path.clone();
        let full_path = join_path(&current_path, filename);
        cx.write_to_clipboard(ClipboardItem::new_string(full_path.clone()));
        self.push_notification(format!("Copied '{}' to clipboard", full_path), false, cx);
        cx.notify();
    }

    /// Transfer all selected items from `from_pane` to the opposite pane.
    pub fn sftp_transfer_selected(&mut self, from_pane: SftpActivePane, cx: &mut Context<Self>) {
        let targets: Vec<String> = self.sftp_pane(from_pane).selected.iter().cloned().collect();
        if targets.is_empty() {
            self.push_notification("No files selected to transfer".to_string(), false, cx);
            cx.notify();
            return;
        }
        let to_pane = match from_pane {
            SftpActivePane::Left => SftpActivePane::Right,
            SftpActivePane::Right => SftpActivePane::Left,
        };
        self.sftp_transfer_between_panes(from_pane, to_pane, targets, cx);
    }

    /// Handle SFTP keyboard shortcuts: Enter, Backspace, Alt+Up, F2, Delete, F5.
    pub fn sftp_handle_key(&mut self, key: &str, is_alt: bool, window: &mut Window, cx: &mut Context<Self>) {
        let focused = self.sftp_manager.focused_pane;
        match key {
            "f5" => {
                self.sftp_load_pane(focused, cx);
            }
            "f2" => {
                let pane_state = self.sftp_pane(focused);
                if pane_state.selected.len() == 1 {
                    let target = pane_state.selected.iter().next().unwrap().clone();
                    self.sftp_open_rename_modal(focused, target, window, cx);
                }
            }
            "delete" => {
                let pane_state = self.sftp_pane(focused);
                if !pane_state.selected.is_empty() {
                    let targets: Vec<String> = pane_state.selected.iter().cloned().collect();
                    self.sftp_open_delete_modal(focused, targets, cx);
                }
            }
            "backspace" => {
                self.sftp_navigate_up(focused, cx);
            }
            "up" if is_alt => {
                self.sftp_navigate_up(focused, cx);
            }
            "enter" => {
                let pane_state = self.sftp_pane(focused);
                if pane_state.selected.len() == 1 {
                    let name = pane_state.selected.iter().next().unwrap().clone();
                    if let Some(f) = pane_state
                        .files
                        .iter()
                        .find(|item| item.name == name && item.is_dir)
                    {
                        let target_path = join_path(&pane_state.current_path, &f.name);
                        self.sftp_navigate(focused, target_path, cx);
                    }
                }
            }
            _ => {}
        }
    }

    /// Set search/filter query in a pane.
    pub fn sftp_set_search_query(
        &mut self,
        pane: SftpActivePane,
        query: String,
        cx: &mut Context<Self>,
    ) {
        self.sftp_pane_mut(pane).search_query = query;
        cx.notify();
    }

    /// Switch active view to SFTP Manager and load directory if needed.
    pub fn navigate_to_sftp(&mut self, cx: &mut Context<Self>) {
        self.active_view = View::Sftp;
        if self.sftp_manager.left_pane.files.is_empty() && !self.sftp_manager.left_pane.is_loading {
            self.sftp_load_pane(SftpActivePane::Left, cx);
        }
        if self.sftp_manager.right_pane.files.is_empty() && !self.sftp_manager.right_pane.is_loading
        {
            self.sftp_load_pane(SftpActivePane::Right, cx);
        }
        cx.notify();
    }

    /// Toggle between SFTP manager and hosts view.
    pub fn toggle_sftp_manager(&mut self, cx: &mut Context<Self>) {
        if self.active_view == View::Sftp {
            self.active_view = View::Hosts;
            self.show_hosts_catalog = true;
        } else {
            self.navigate_to_sftp(cx);
        }
        cx.notify();
    }

    /// Open modal for creating a new folder in a pane.
    pub fn sftp_open_new_folder_modal(
        &mut self,
        pane: SftpActivePane,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.sftp_manager.modal = Some(SftpModalState::NewFolder {
            pane,
            name: "new-folder".to_string(),
            error: None,
        });
        AppState::set_input_value(&self.inputs().sftp_modal_name, "new-folder", window, cx);
        cx.notify();
    }

    /// Open modal for renaming a file/folder in a pane.
    pub fn sftp_open_rename_modal(
        &mut self,
        pane: SftpActivePane,
        old_name: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        AppState::set_input_value(&self.inputs().sftp_modal_name, &old_name, window, cx);
        self.sftp_manager.modal = Some(SftpModalState::Rename {
            pane,
            new_name: old_name.clone(),
            old_name,
            error: None,
        });
        cx.notify();
    }

    /// Open modal confirming deletion of selected items in a pane.
    pub fn sftp_open_delete_modal(
        &mut self,
        pane: SftpActivePane,
        targets: Vec<String>,
        cx: &mut Context<Self>,
    ) {
        if targets.is_empty() {
            return;
        }
        self.sftp_manager.modal = Some(SftpModalState::DeleteConfirm {
            pane,
            targets,
            error: None,
        });
        cx.notify();
    }

    /// Close any open SFTP modal.
    pub fn sftp_close_modal(&mut self, cx: &mut Context<Self>) {
        self.sftp_manager.modal = None;
        cx.notify();
    }

    /// Create new folder in a pane.
    pub fn sftp_create_folder(&mut self, pane: SftpActivePane, name: &str, cx: &mut Context<Self>) {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            if let Some(SftpModalState::NewFolder { error, .. }) = &mut self.sftp_manager.modal {
                *error = Some("Folder name cannot be empty".to_string());
                cx.notify();
            }
            return;
        }

        let client = match self.client.clone() {
            Some(c) => c,
            None => return,
        };

        let name_owned = trimmed.to_string();
        let current_path = self.sftp_pane(pane).current_path.clone();
        let full_path = join_path(&current_path, trimmed);
        let conn_id = self.sftp_pane(pane).source_id.clone();

        self.sftp_close_modal(cx);

        let view_weak = cx.entity().downgrade();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Result<(), String>>();

        TOKIO_RT.spawn(async move {
            match client.sftp_mkdir(&conn_id, &full_path).await {
                Ok(_) => {
                    let _ = tx.send(Ok(()));
                }
                Err(e) => {
                    let _ = tx.send(Err(e.to_string()));
                }
            }
        });

        cx.spawn(move |_view, cx: &mut AsyncApp| {
            let view_weak = view_weak.clone();
            let cx_handle = cx.clone();
            let name_for_notif = name_owned.clone();
            async move {
                if let Some(res) = rx.recv().await {
                    cx_handle.update(|cx: &mut App| {
                        if let Some(app) = view_weak.upgrade() {
                            app.update(cx, |this, cx| {
                                match res {
                                    Ok(_) => {
                                        this.push_notification(format!("Directory '{}' created", name_for_notif), false, cx);
                                        this.sftp_load_pane(pane, cx);
                                    }
                                    Err(err) => {
                                        this.push_notification(format!("Failed to create folder: {err}"), true, cx);
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

    /// Rename entry in a pane.
    pub fn sftp_rename_entry(
        &mut self,
        pane: SftpActivePane,
        old_name: &str,
        new_name: &str,
        cx: &mut Context<Self>,
    ) {
        let trimmed_new = new_name.trim();
        if trimmed_new.is_empty() {
            if let Some(SftpModalState::Rename { error, .. }) = &mut self.sftp_manager.modal {
                *error = Some("New name cannot be empty".to_string());
                cx.notify();
            }
            return;
        }
        if trimmed_new == old_name {
            self.sftp_close_modal(cx);
            return;
        }

        let client = match self.client.clone() {
            Some(c) => c,
            None => return,
        };

        let old_owned = old_name.to_string();
        let new_owned = trimmed_new.to_string();
        let current_path = self.sftp_pane(pane).current_path.clone();
        let old_full_path = join_path(&current_path, old_name);
        let new_full_path = join_path(&current_path, trimmed_new);
        let conn_id = self.sftp_pane(pane).source_id.clone();

        self.sftp_close_modal(cx);

        let view_weak = cx.entity().downgrade();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Result<(), String>>();

        TOKIO_RT.spawn(async move {
            match client
                .sftp_rename(&conn_id, &old_full_path, &new_full_path)
                .await
            {
                Ok(_) => {
                    let _ = tx.send(Ok(()));
                }
                Err(e) => {
                    let _ = tx.send(Err(e.to_string()));
                }
            }
        });

        cx.spawn(move |_view, cx: &mut AsyncApp| {
            let view_weak = view_weak.clone();
            let cx_handle = cx.clone();
            let old_label = old_owned.clone();
            let new_label = new_owned.clone();
            async move {
                if let Some(res) = rx.recv().await {
                    cx_handle.update(|cx: &mut App| {
                        if let Some(app) = view_weak.upgrade() {
                            app.update(cx, |this, cx| {
                                match res {
                                    Ok(_) => {
                                        this.push_notification(format!(
                                            "Renamed '{}' to '{}'",
                                            old_label, new_label
                                        ), false, cx);
                                        this.sftp_load_pane(pane, cx);
                                    }
                                    Err(err) => {
                                        this.push_notification(format!("Failed to rename: {err}"), true, cx);
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

    /// Delete selected items in a pane.
    pub fn sftp_delete_selected(
        &mut self,
        pane: SftpActivePane,
        targets: Vec<String>,
        cx: &mut Context<Self>,
    ) {
        if targets.is_empty() {
            self.sftp_close_modal(cx);
            return;
        }

        let client = match self.client.clone() {
            Some(c) => c,
            None => return,
        };

        let current_path = self.sftp_pane(pane).current_path.clone();
        let conn_id = self.sftp_pane(pane).source_id.clone();

        self.sftp_close_modal(cx);

        let view_weak = cx.entity().downgrade();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Result<usize, String>>();

        TOKIO_RT.spawn(async move {
            let mut deleted = 0;
            for target in targets {
                let full = join_path(&current_path, &target);
                if let Err(e) = client.sftp_remove(&conn_id, &full).await {
                    let _ = tx.send(Err(format!("Error deleting '{}': {e}", target)));
                    return;
                }
                deleted += 1;
            }
            let _ = tx.send(Ok(deleted));
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
                                    Ok(deleted) => {
                                        this.push_notification(format!("Deleted {} item(s)", deleted), false, cx);
                                        this.sftp_clear_selection(pane, cx);
                                        this.sftp_load_pane(pane, cx);
                                    }
                                    Err(err) => {
                                        this.push_notification(err, false, cx);
                                        this.sftp_load_pane(pane, cx);
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

    /// Single entry delete helper.
    pub fn sftp_delete_entry(&mut self, pane: SftpActivePane, path: &str, cx: &mut Context<Self>) {
        self.sftp_delete_selected(pane, vec![path.to_string()], cx);
    }

    /// Upload a file into target pane.
    pub fn sftp_upload_file(
        &mut self,
        target_pane: SftpActivePane,
        filename: String,
        data: Vec<u8>,
        cx: &mut Context<Self>,
    ) {
        let client = match self.client.clone() {
            Some(c) => c,
            None => return,
        };

        let conn_id = self.sftp_pane(target_pane).source_id.clone();
        let target_path = self.sftp_pane(target_pane).current_path.clone();
        let total_bytes = data.len() as i64;
        let transfer_id = next_transfer_id();

        let transfer_item = SftpTransferItem {
            id: transfer_id.clone(),
            name: filename.clone(),
            from_source: "Upload".to_string(),
            to_source: self.sftp_pane(target_pane).source_label.clone(),
            bytes_transferred: 0,
            total_bytes,
            status: "in_progress".to_string(),
            error: None,
        };

        self.sftp_manager.transfers.push(transfer_item);
        self.sftp_manager.transfers_drawer_open = true;
        cx.notify();

        let tx_id = transfer_id.clone();
        let view_weak = cx.entity().downgrade();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Result<String, String>>();

        TOKIO_RT.spawn(async move {
            match client
                .sftp_upload(&conn_id, &target_path, &filename, data)
                .await
            {
                Ok(remote_tx_id) => {
                    let _ = tx.send(Ok(remote_tx_id));
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
                                if let Some(item) = this
                                    .sftp_manager
                                    .transfers
                                    .iter_mut()
                                    .find(|t| t.id == tx_id)
                                {
                                    match res {
                                        Ok(_) => {
                                            item.status = "completed".to_string();
                                            item.bytes_transferred = item.total_bytes;
                                            this.sftp_load_pane(target_pane, cx);
                                        }
                                        Err(err) => {
                                            item.status = "failed".to_string();
                                            item.error = Some(err);
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

    /// Download a file from source pane.
    pub fn sftp_download_file(
        &mut self,
        source_pane: SftpActivePane,
        file_path: String,
        cx: &mut Context<Self>,
    ) {
        let client = match self.client.clone() {
            Some(c) => c,
            None => return,
        };

        let conn_id = self.sftp_pane(source_pane).source_id.clone();
        let full_path = join_path(&self.sftp_pane(source_pane).current_path, &file_path);
        let transfer_id = next_transfer_id();

        let transfer_item = SftpTransferItem {
            id: transfer_id.clone(),
            name: file_path.clone(),
            from_source: self.sftp_pane(source_pane).source_label.clone(),
            to_source: "Download".to_string(),
            bytes_transferred: 0,
            total_bytes: 0,
            status: "in_progress".to_string(),
            error: None,
        };

        self.sftp_manager.transfers.push(transfer_item);
        self.sftp_manager.transfers_drawer_open = true;
        cx.notify();

        let tx_id = transfer_id.clone();
        let view_weak = cx.entity().downgrade();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Result<Vec<u8>, String>>();

        TOKIO_RT.spawn(async move {
            match client.sftp_download(&conn_id, &full_path).await {
                Ok(bytes) => {
                    let _ = tx.send(Ok(bytes));
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
                                if let Some(item) = this
                                    .sftp_manager
                                    .transfers
                                    .iter_mut()
                                    .find(|t| t.id == tx_id)
                                {
                                    match res {
                                        Ok(bytes) => {
                                            item.status = "completed".to_string();
                                            item.total_bytes = bytes.len() as i64;
                                            item.bytes_transferred = bytes.len() as i64;
                                        }
                                        Err(err) => {
                                            item.status = "failed".to_string();
                                            item.error = Some(err);
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

    /// Transfer selected items from source pane to destination pane.
    pub fn sftp_transfer_between_panes(
        &mut self,
        src_pane: SftpActivePane,
        dst_pane: SftpActivePane,
        targets: Vec<String>,
        cx: &mut Context<Self>,
    ) {
        if targets.is_empty() {
            return;
        }

        let client = match self.client.clone() {
            Some(c) => c,
            None => return,
        };

        let src_conn = self.sftp_pane(src_pane).source_id.clone();
        let src_base = self.sftp_pane(src_pane).current_path.clone();
        let dst_conn = self.sftp_pane(dst_pane).source_id.clone();
        let dst_base = self.sftp_pane(dst_pane).current_path.clone();
        let src_label = self.sftp_pane(src_pane).source_label.clone();
        let dst_label = self.sftp_pane(dst_pane).source_label.clone();

        self.sftp_manager.transfers_drawer_open = true;

        for filename in targets {
            let tx_id = next_transfer_id();
            let transfer_item = SftpTransferItem {
                id: tx_id.clone(),
                name: filename.clone(),
                from_source: src_label.clone(),
                to_source: dst_label.clone(),
                bytes_transferred: 0,
                total_bytes: 0,
                status: "in_progress".to_string(),
                error: None,
            };
            self.sftp_manager.transfers.push(transfer_item);

            let client = client.clone();
            let src_conn = src_conn.clone();
            let dst_conn = dst_conn.clone();
            let src_file_path = join_path(&src_base, &filename);
            let dst_target_dir = dst_base.clone();
            let fname = filename.clone();

            let view_weak = cx.entity().downgrade();
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Result<usize, String>>();

            TOKIO_RT.spawn(async move {
                match client.sftp_download(&src_conn, &src_file_path).await {
                    Ok(data) => {
                        let len = data.len();
                        match client
                            .sftp_upload(&dst_conn, &dst_target_dir, &fname, data)
                            .await
                        {
                            Ok(_) => {
                                let _ = tx.send(Ok(len));
                            }
                            Err(e) => {
                                let _ = tx.send(Err(format!("Upload error: {e}")));
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(format!("Download error: {e}")));
                    }
                }
            });

            cx.spawn(move |_view, cx: &mut AsyncApp| {
                let view_weak = view_weak.clone();
                let cx_handle = cx.clone();
                let tx_id = tx_id.clone();
                async move {
                    if let Some(res) = rx.recv().await {
                        cx_handle.update(|cx: &mut App| {
                            if let Some(app) = view_weak.upgrade() {
                                app.update(cx, |this, cx| {
                                    if let Some(item) = this
                                        .sftp_manager
                                        .transfers
                                        .iter_mut()
                                        .find(|t| t.id == tx_id)
                                    {
                                        match res {
                                            Ok(bytes_len) => {
                                                item.status = "completed".to_string();
                                                item.total_bytes = bytes_len as i64;
                                                item.bytes_transferred = bytes_len as i64;
                                                this.sftp_load_pane(dst_pane, cx);
                                            }
                                            Err(err) => {
                                                item.status = "failed".to_string();
                                                item.error = Some(err);
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

        cx.notify();
    }

    /// Poll backend transfers API and synchronize active status.
    pub fn sftp_poll_transfers(&mut self, cx: &mut Context<Self>) {
        let client = match self.client.clone() {
            Some(c) => c,
            None => return,
        };

        let view_weak = cx.entity().downgrade();
        let (tx, mut rx) =
            tokio::sync::mpsc::unbounded_channel::<Result<Vec<SftpTransferStatus>, String>>();

        TOKIO_RT.spawn(async move {
            match client.sftp_list_transfers().await {
                Ok(list) => {
                    let _ = tx.send(Ok(list));
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
                                if let Ok(remote_list) = res {
                                    for remote in remote_list {
                                        if let Some(existing) = this
                                            .sftp_manager
                                            .transfers
                                            .iter_mut()
                                            .find(|t| t.id == remote.id)
                                        {
                                            existing.bytes_transferred = remote.bytes_transferred;
                                            existing.total_bytes = remote.total_bytes;
                                            existing.status = remote.status;
                                            existing.error = remote.error;
                                        } else {
                                            this.sftp_manager
                                                .transfers
                                                .push(SftpTransferItem::from(remote));
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

    /// Toggle visibility of the transfers drawer.
    pub fn sftp_toggle_transfers_drawer(&mut self, cx: &mut Context<Self>) {
        self.sftp_manager.transfers_drawer_open = !self.sftp_manager.transfers_drawer_open;
        cx.notify();
    }

    /// Asynchronously load directory contents for a pane.
    pub fn sftp_load_pane(&mut self, pane: SftpActivePane, cx: &mut Context<Self>) {
        let client = match self.client.clone() {
            Some(c) => c,
            None => return,
        };

        let state = self.sftp_pane_mut(pane);
        state.is_loading = true;
        state.error = None;
        cx.notify();

        let conn_id = state.source_id.clone();
        let path = state.current_path.clone();

        let view_weak = cx.entity().downgrade();
        let (tx, mut rx) =
            tokio::sync::mpsc::unbounded_channel::<Result<Vec<SftpFileInfo>, String>>();

        TOKIO_RT.spawn(async move {
            match client.sftp_list(&conn_id, &path).await {
                Ok(files) => {
                    let _ = tx.send(Ok(files));
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
                                let pane_state = this.sftp_pane_mut(pane);
                                pane_state.is_loading = false;
                                match res {
                                    Ok(files) => {
                                        pane_state.files = files;
                                        pane_state.error = None;
                                    }
                                    Err(e) => {
                                        pane_state.error = Some(e);
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

    // ==========================================
    // Port Forwarding Methods
    // ==========================================

    /// Fetch all port forward rules from the backend.
    pub fn fetch_forwards(&mut self, cx: &mut Context<Self>) {
        let client = match self.client.clone() {
            Some(c) => c,
            None => return,
        };

        self.is_loading_forwards = true;
        cx.notify();

        let view_weak = cx.entity().downgrade();
        let (tx, mut rx) =
            tokio::sync::mpsc::unbounded_channel::<Result<Vec<PortForward>, String>>();

        TOKIO_RT.spawn(async move {
            match client.list_forwards().await {
                Ok(forwards) => {
                    let _ = tx.send(Ok(forwards));
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
                                this.is_loading_forwards = false;
                                match res {
                                    Ok(forwards) => {
                                        this.forwards = forwards;
                                    }
                                    Err(err) => {
                                        eprintln!("Failed to fetch forwards: {err}");
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

    /// Open create port forward modal.
    pub fn open_create_forward_modal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let default_conn_id = self.connections.first().map(|c| c.id.clone());
        self.forward_modal = Some(ForwardFormState::new_create(default_conn_id));
        self.forward_sheet_closing = false;
        self.close_other_sheets_for("forward");
        let inputs = self.inputs();
        AppState::set_input_value(&inputs.fwd_name, "", window, cx);
        AppState::set_input_value(&inputs.fwd_local_port, "", window, cx);
        AppState::set_input_value(&inputs.fwd_remote_port, "", window, cx);
        cx.notify();
    }

    /// Open edit port forward sheet.
    pub fn open_edit_forward_modal(
        &mut self,
        id: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(forward) = self.forwards.iter().find(|f| f.id == id).cloned() {
            self.forward_modal = Some(ForwardFormState::new_edit(&forward));
            self.forward_sheet_closing = false;
            self.close_other_sheets_for("forward");
            let inputs = self.inputs();
            AppState::set_input_value(&inputs.fwd_name, &forward.name, window, cx);
            AppState::set_input_value(&inputs.fwd_local_port, &forward.local_port.to_string(), window, cx);
            AppState::set_input_value(&inputs.fwd_remote_port, &forward.remote_port.to_string(), window, cx);
            cx.notify();
        }
    }

    /// Play the forward sheet's exit animation, then unmount it.
    pub fn close_forward_modal(&mut self, cx: &mut Context<Self>) {
        if !self.forward_modal.is_some() || self.forward_sheet_closing {
            return;
        }
        self.forward_sheet_closing = true;
        Self::after_sheet_exit(cx, |this| {
            // Re-opening during the exit cancels the pending unmount.
            if this.forward_sheet_closing {
                this.forward_modal = None;
                this.forward_sheet_closing = false;
            }
        });
        cx.notify();
    }

    /// Save port forward modal form (Create or Update).
    pub fn save_forward_form(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let (name, local_port, remote_port) = {
            let inputs = self.inputs();
            (
                AppState::input_value(&inputs.fwd_name, cx),
                AppState::input_value(&inputs.fwd_local_port, cx),
                AppState::input_value(&inputs.fwd_remote_port, cx),
            )
        };
        if let Some(f) = &mut self.forward_modal {
            f.name = name;
            f.local_port = local_port;
            f.remote_port = remote_port;
        }
        let form = match &mut self.forward_modal {
            Some(f) => f,
            None => return,
        };

        let mode = form.mode.clone();
        let client = match self.client.clone() {
            Some(c) => c,
            None => {
                form.error_message = Some("Backend client unavailable".to_string());
                cx.notify();
                return;
            }
        };

        match mode {
            ForwardModalMode::Create => {
                match form.to_create_request() {
                    Ok(req) => {
                        let view_weak = cx.entity().downgrade();
                        let (tx, mut rx) =
                            tokio::sync::mpsc::unbounded_channel::<Result<PortForward, String>>();

                        TOKIO_RT.spawn(async move {
                            match client.create_forward(&req).await {
                                Ok(f) => {
                                    let _ = tx.send(Ok(f));
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
                                                Ok(f) => {
                                                    this.close_forward_modal(cx);
                                                    this.push_notification(format!("Port forward '{}' created successfully", f.name), false, cx);
                                                    this.fetch_forwards(cx);
                                                }
                                                Err(err) => {
                                                    if let Some(f) = &mut this.forward_modal {
                                                        f.error_message = Some(err);
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
                    Err(err) => {
                        form.error_message = Some(err);
                        cx.notify();
                    }
                }
            }
            ForwardModalMode::Edit(id) => {
                match form.to_update_request() {
                    Ok(req) => {
                        let view_weak = cx.entity().downgrade();
                        let id_clone = id.clone();
                        let (tx, mut rx) =
                            tokio::sync::mpsc::unbounded_channel::<Result<PortForward, String>>();

                        TOKIO_RT.spawn(async move {
                            match client.update_forward(&id_clone, &req).await {
                                Ok(f) => {
                                    let _ = tx.send(Ok(f));
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
                                                Ok(f) => {
                                                    this.close_forward_modal(cx);
                                                    this.push_notification(format!("Port forward '{}' updated successfully", f.name), false, cx);
                                                    this.fetch_forwards(cx);
                                                }
                                                Err(err) => {
                                                    if let Some(f) = &mut this.forward_modal {
                                                        f.error_message = Some(err);
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
                    Err(err) => {
                        form.error_message = Some(err);
                        cx.notify();
                    }
                }
            }
        }
    }

    /// Toggle port forward tunnel active status (start if inactive, stop if active).
    pub fn toggle_forward_active(&mut self, id: &str, cx: &mut Context<Self>) {
        let forward = match self.forwards.iter().find(|f| f.id == id) {
            Some(f) => f.clone(),
            None => return,
        };

        let client = match self.client.clone() {
            Some(c) => c,
            None => return,
        };

        let id_str = id.to_string();
        let was_active = forward.active;
        let view_weak = cx.entity().downgrade();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Result<String, String>>();

        TOKIO_RT.spawn(async move {
            let res = if was_active {
                client.stop_forward(&id_str).await.map(|r| r.status)
            } else {
                client.start_forward(&id_str).await.map(|r| r.status)
            };

            match res {
                Ok(st) => {
                    let _ = tx.send(Ok(st));
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
                                    Ok(status) => {
                                        this.push_notification(format!(
                                            "Port forward '{}' is now {}",
                                            forward.name, status
                                        ), false, cx);
                                        this.fetch_forwards(cx);
                                    }
                                    Err(err) => {
                                        this.push_notification(format!(
                                            "Failed to toggle forward '{}': {err}",
                                            forward.name
                                        ), true, cx);
                                        this.fetch_forwards(cx);
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

    /// Open delete confirmation dialog for a port forward rule.
    pub fn open_delete_forward_modal(&mut self, forward: PortForward, cx: &mut Context<Self>) {
        self.delete_forward_target = Some(forward);
        cx.notify();
    }

    /// Close delete confirmation dialog.
    pub fn close_delete_forward_modal(&mut self, cx: &mut Context<Self>) {
        self.delete_forward_target = None;
        cx.notify();
    }

    /// Confirm and delete targeted port forward rule.
    pub fn confirm_delete_forward(&mut self, cx: &mut Context<Self>) {
        let target = match self.delete_forward_target.take() {
            Some(t) => t,
            None => return,
        };

        let client = match self.client.clone() {
            Some(c) => c,
            None => return,
        };

        let id_str = target.id.clone();
        let name_str = target.name.clone();
        let view_weak = cx.entity().downgrade();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Result<(), String>>();

        TOKIO_RT.spawn(async move {
            match client.delete_forward(&id_str).await {
                Ok(_) => {
                    let _ = tx.send(Ok(()));
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
                                    Ok(_) => {
                                        this.push_notification(format!("Port forward '{}' deleted", name_str), false, cx);
                                        this.fetch_forwards(cx);
                                    }
                                    Err(err) => {
                                        this.push_notification(format!(
                                            "Failed to delete forward '{}': {err}",
                                            name_str
                                        ), true, cx);
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

    /// Navigate to port forwards view.
    pub fn navigate_to_forwards(&mut self, cx: &mut Context<Self>) {
        self.active_view = View::Forwards;
        if self.forwards.is_empty() && !self.is_loading_forwards {
            self.fetch_forwards(cx);
        }
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
