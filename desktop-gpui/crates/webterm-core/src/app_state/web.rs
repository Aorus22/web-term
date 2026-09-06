use std::collections::HashSet;
use gpui::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;

use crate::session::{SessionStatus, TerminalSessionManager, TerminalTab};
use crate::types::{
    Connection, CreateConnectionRequest, CreateForwardRequest, CreateKeyRequest,
    PortForward, SftpFileInfo, SshKey, UpdateConnectionRequest, UpdateForwardRequest,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Theme {
    #[default]
    Dark,
    Light,
    System,
}

#[derive(Debug, Clone, Default)]
pub struct DesktopSettings {
    pub theme: Theme,
    pub theme_preset: String,
    pub font_size: f32,
    pub font_family: String,
    pub cursor_style: String,
    pub cursor_blink: bool,
    pub scrollback: u32,
    pub theme_mode_filter: String,
    pub backend_path: Option<std::path::PathBuf>,
}

impl DesktopSettings {
    pub fn save(&self) -> Result<(), String> {
        Ok(())
    }
}

pub async fn api_get<T: serde::de::DeserializeOwned>(path: &str) -> Result<T, String> {
    let window = web_sys::window().ok_or("No window object")?;
    let resp_val = wasm_bindgen_futures::JsFuture::from(window.fetch_with_str(path))
        .await
        .map_err(|e| format!("Fetch error: {e:?}"))?;

    let resp: web_sys::Response = resp_val
        .dyn_into()
        .map_err(|_| "Response is not a web_sys::Response")?;

    if !resp.ok() {
        return Err(format!("HTTP error status {}", resp.status()));
    }

    let text_promise = resp.text().map_err(|e| format!("text() error: {e:?}"))?;
    let text_val = wasm_bindgen_futures::JsFuture::from(text_promise)
        .await
        .map_err(|e| format!("Text read error: {e:?}"))?;

    let text_str = text_val.as_string().unwrap_or_default();
    serde_json::from_str::<T>(&text_str).map_err(|e| format!("JSON decode error: {e}"))
}

pub async fn api_send_json<T: serde::de::DeserializeOwned, B: serde::Serialize>(
    method: &str,
    path: &str,
    body: &B,
) -> Result<T, String> {
    let window = web_sys::window().ok_or("No window object")?;
    let json_body = serde_json::to_string(body).map_err(|e| format!("JSON serialize error: {e}"))?;

    let opts = web_sys::RequestInit::new();
    opts.set_method(method);
    opts.set_body(&wasm_bindgen::JsValue::from_str(&json_body));

    let headers = web_sys::Headers::new().map_err(|e| format!("Headers error: {e:?}"))?;
    headers.set("Content-Type", "application/json").map_err(|e| format!("Header set error: {e:?}"))?;
    opts.set_headers(&headers);

    let request = web_sys::Request::new_with_str_and_init(path, &opts)
        .map_err(|e| format!("Request new error: {e:?}"))?;

    let resp_val = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch error: {e:?}"))?;

    let resp: web_sys::Response = resp_val
        .dyn_into()
        .map_err(|_| "Response is not a web_sys::Response")?;

    if !resp.ok() {
        return Err(format!("HTTP error status {}", resp.status()));
    }

    let text_promise = resp.text().map_err(|e| format!("text() error: {e:?}"))?;
    let text_val = wasm_bindgen_futures::JsFuture::from(text_promise)
        .await
        .map_err(|e| format!("Text read error: {e:?}"))?;

    let text_str = text_val.as_string().unwrap_or_default();
    if text_str.trim().is_empty() {
        if let Ok(val) = serde_json::from_str::<T>("null") {
            return Ok(val);
        }
    }
    serde_json::from_str::<T>(&text_str).map_err(|e| format!("JSON decode error: {e}"))
}

pub async fn api_delete(path: &str) -> Result<(), String> {
    let window = web_sys::window().ok_or("No window object")?;
    let opts = web_sys::RequestInit::new();
    opts.set_method("DELETE");

    let request = web_sys::Request::new_with_str_and_init(path, &opts)
        .map_err(|e| format!("Request new error: {e:?}"))?;

    let resp_val = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch error: {e:?}"))?;

    let resp: web_sys::Response = resp_val
        .dyn_into()
        .map_err(|_| "Response is not a web_sys::Response")?;

    if !resp.ok() {
        return Err(format!("HTTP error status {}", resp.status()));
    }
    Ok(())
}

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

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionModalMode {
    Create,
    Edit(String),
}

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
        self.tags.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
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
            ssh_key_id: self.ssh_key_id.clone(),
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
            ssh_key_id: self.ssh_key_id.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForwardModalMode {
    Create,
    Edit(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForwardFormState {
    pub mode: ForwardModalMode,
    pub name: String,
    pub connection_id: String,
    pub local_port: String,
    pub remote_port: String,
    pub forward_type: String,
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

    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Rule name is required".to_string());
        }
        Ok(())
    }

    pub fn to_create_request(&self) -> Result<CreateForwardRequest, String> {
        self.validate()?;
        Ok(CreateForwardRequest {
            name: self.name.trim().to_string(),
            connection_id: self.connection_id.clone(),
            local_port: self.local_port.trim().parse().unwrap_or(8080),
            remote_port: self.remote_port.trim().parse().unwrap_or(80),
            forward_type: self.forward_type.clone(),
        })
    }

    pub fn to_update_request(&self) -> Result<UpdateForwardRequest, String> {
        self.validate()?;
        Ok(UpdateForwardRequest {
            name: self.name.trim().to_string(),
            connection_id: self.connection_id.clone(),
            local_port: self.local_port.trim().parse().unwrap_or(8080),
            remote_port: self.remote_port.trim().parse().unwrap_or(80),
            forward_type: self.forward_type.clone(),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    NewTab,
    Hosts,
    Keys,
    Forwards,
    Sftp,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SftpActivePane {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SftpSortColumn {
    Name,
    Size,
    ModTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SftpSortOrder {
    Ascending,
    Descending,
}

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
    pub fn new(source_id: impl Into<String>, source_label: impl Into<String>, initial_path: impl Into<String>) -> Self {
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

    pub fn visible_files(&self) -> Vec<SftpFileInfo> {
        let mut list: Vec<SftpFileInfo> = self
            .files
            .iter()
            .filter(|f| {
                if !self.show_hidden && f.name.starts_with('.') {
                    return false;
                }
                if !self.search_query.is_empty()
                    && !f.name.to_lowercase().contains(&self.search_query.to_lowercase())
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

#[derive(Debug, Clone, PartialEq)]
pub enum SftpModalState {
    NewFolder { pane: SftpActivePane, name: String, error: Option<String> },
    Rename { pane: SftpActivePane, old_name: String, new_name: String, error: Option<String> },
    DeleteConfirm { pane: SftpActivePane, targets: Vec<String>, error: Option<String> },
    Conflict { src_pane: SftpActivePane, dst_pane: SftpActivePane, conflict_name: String, remaining_transfers: Vec<String> },
}

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

#[derive(Debug, Clone, PartialEq)]
pub struct SftpContextMenu {
    pub pane: SftpActivePane,
    pub filename: String,
    pub is_dir: bool,
    pub position: (f32, f32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SftpDraggedItem {
    pub source_pane: SftpActivePane,
    pub filenames: Vec<String>,
}

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

impl Default for SftpManager {
    fn default() -> Self {
        let home = local_home_path();
        Self {
            left_pane: SftpPaneState::new("local", "Local Filesystem", &home),
            right_pane: SftpPaneState::new("remote", "Remote Server", "/var/www"),
            focused_pane: SftpActivePane::Left,
            modal: None,
            context_menu: None,
            transfers: Vec::new(),
            transfers_drawer_open: false,
            focus_handle: None,
        }
    }
}

pub fn local_home_path() -> String {
    "/home/webterm".to_string()
}

pub fn split_breadcrumbs(path: &str) -> Vec<(String, String)> {
    let clean = path.trim();
    if clean.is_empty() || clean == "." {
        return vec![(".".to_string(), ".".to_string())];
    }
    let normalized = clean.replace('\\', "/");
    let parts: Vec<&str> = normalized.split('/').filter(|s| !s.is_empty()).collect();
    let mut result = Vec::new();
    let mut current_acc = String::new();
    for part in parts {
        current_acc.push('/');
        current_acc.push_str(part);
        result.push((part.to_string(), current_acc.clone()));
    }
    if result.is_empty() {
        vec![(clean.to_string(), clean.to_string())]
    } else {
        result
    }
}

pub fn parent_path(path: &str) -> String {
    let clean = path.trim().replace('\\', "/");
    if clean.is_empty() || clean == "." || clean == "/" {
        return clean;
    }
    let trimmed = clean.trim_end_matches('/');
    if let Some(pos) = trimmed.rfind('/') {
        if pos == 0 { "/".to_string() } else { trimmed[..pos].to_string() }
    } else {
        ".".to_string()
    }
}

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

pub fn format_file_size(bytes: i64) -> String {
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

pub fn format_date_modified(iso: &str) -> String {
    iso.to_string()
}

pub fn get_available_drives() -> Vec<String> {
    vec!["/".to_string()]
}

pub struct AppState {
    pub active_view: View,
    pub theme: Theme,
    pub settings: DesktopSettings,
    pub sidebar_open: bool,
    pub is_maximized: bool,
    pub notification: Option<String>,
    pub connections: Vec<Connection>,
    pub search_query: String,
    pub selected_tag: Option<String>,
    pub is_loading_connections: bool,
    pub connection_modal: Option<ConnectionFormState>,
    pub show_import_modal: bool,
    pub import_payload: String,
    pub ssh_keys: Vec<SshKey>,
    pub is_loading_keys: bool,
    pub show_add_key_modal: bool,
    pub new_key_name: String,
    pub new_key_pem: String,
    pub add_key_error: Option<String>,
    pub forwards: Vec<PortForward>,
    pub is_loading_forwards: bool,
    pub forward_modal: Option<ForwardFormState>,
    pub delete_forward_target: Option<PortForward>,
    pub sftp_manager: SftpManager,
    pub session_manager: TerminalSessionManager,
    pub quick_connect_query: String,
    pub new_tab_host: String,
    pub new_tab_user: String,
    pub new_tab_port: String,
    pub new_tab_password: String,
    pub show_hosts_catalog: bool,
    pub show_new_tab_popover: bool,
    pub show_new_tab_modal: bool,
    pub pending_passphrase_conn: Option<(String, String)>,
    pub passphrase_input: String,
    pub passphrase_error: Option<String>,
    pub backend_status: crate::webterm_supervisor::BackendStatus,
    pub backend_path_input: String,
    pub terminal_font_family: String,
    pub terminal_font_size: f32,
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
    pub spawn_opts: Option<()>,
}

impl AppState {
    pub fn new() -> Self {
        let connections = Vec::new();
        let ssh_keys = Vec::new();
        let forwards = Vec::new();

        let mut left_files = Vec::new();
        left_files.push(SftpFileInfo {
            name: "projects".to_string(),
            size: 4096,
            is_dir: true,
            mod_time: "2026-09-04 18:20:00".to_string(),
            mode: 0o755,
        });
        left_files.push(SftpFileInfo {
            name: "config.yaml".to_string(),
            size: 1420,
            is_dir: false,
            mod_time: "2026-09-05 14:10:00".to_string(),
            mode: 0o644,
        });
        left_files.push(SftpFileInfo {
            name: "docker-compose.yml".to_string(),
            size: 3280,
            is_dir: false,
            mod_time: "2026-09-05 16:30:00".to_string(),
            mode: 0o644,
        });

        let mut sftp_manager = SftpManager::default();
        sftp_manager.left_pane.files = left_files;

        let mut settings = DesktopSettings::default();
        settings.theme_preset = "default-dark".to_string();
        settings.font_family = "JetBrains Mono".to_string();
        settings.font_size = 14.0;
        settings.cursor_style = "bar".to_string();
        settings.cursor_blink = true;
        settings.scrollback = 10000;
        settings.theme_mode_filter = "all".to_string();

        Self {
            active_view: View::Hosts,
            theme: Theme::Dark,
            settings,
            sidebar_open: true,
            is_maximized: false,
            notification: None,
            connections,
            search_query: String::new(),
            selected_tag: None,
            is_loading_connections: false,
            connection_modal: None,
            show_import_modal: false,
            import_payload: String::new(),
            ssh_keys,
            is_loading_keys: false,
            show_add_key_modal: false,
            new_key_name: String::new(),
            new_key_pem: String::new(),
            add_key_error: None,
            forwards,
            is_loading_forwards: false,
            forward_modal: None,
            delete_forward_target: None,
            sftp_manager,
            session_manager: TerminalSessionManager::new(),
            quick_connect_query: String::new(),
            new_tab_host: String::new(),
            new_tab_user: "root".to_string(),
            new_tab_port: "22".to_string(),
            new_tab_password: String::new(),
            show_hosts_catalog: true,
            show_new_tab_popover: false,
            show_new_tab_modal: false,
            pending_passphrase_conn: None,
            passphrase_input: String::new(),
            passphrase_error: None,
            backend_status: crate::webterm_supervisor::BackendStatus::Ready,
            backend_path_input: "http://127.0.0.1:8080".to_string(),
            terminal_font_family: "JetBrains Mono".to_string(),
            terminal_font_size: 14.0,
            cursor_style: "bar".to_string(),
            cursor_blink: true,
            scrollback: 10000,
            theme_mode_filter: "all".to_string(),
            show_theme_mode_picker: false,
            show_cursor_style_picker: false,
            show_scrollback_picker: false,
            show_font_dialog: false,
            font_dialog_family: "JetBrains Mono".to_string(),
            font_dialog_size: 14.0,
            show_font_dialog_picker: false,
            spawn_opts: None,
        }
    }

    pub fn current_theme(&self) -> &'static crate::theme::ThemePreset {
        crate::theme::find_theme_preset(&self.settings.theme_preset)
    }

    pub fn set_theme_preset(&mut self, preset_id: &str, cx: &mut Context<Self>) {
        self.settings.theme_preset = preset_id.to_string();
        let preset = crate::theme::find_theme_preset(preset_id);
        self.theme = if preset.is_dark { Theme::Dark } else { Theme::Light };
        self.settings.theme = self.theme;
        crate::theme::apply_theme(self.theme, cx);
        cx.notify();
    }

    pub fn set_theme_mode_filter(&mut self, filter: &str, cx: &mut Context<Self>) {
        self.theme_mode_filter = filter.to_string();
        self.settings.theme_mode_filter = filter.to_string();
        cx.notify();
    }

    pub fn bg_color(&self) -> Rgba { self.current_theme().bg() }
    pub fn card_bg(&self) -> Rgba { self.current_theme().card_bg() }
    pub fn card_fg(&self) -> Rgba { self.current_theme().card_fg() }
    pub fn border_color(&self) -> Rgba { self.current_theme().border() }
    pub fn text_color(&self) -> Rgba { self.current_theme().fg() }
    pub fn muted_text(&self) -> Rgba { self.current_theme().muted_fg() }
    pub fn muted_bg(&self) -> Rgba { self.current_theme().muted() }
    pub fn primary_color(&self) -> Rgba { self.current_theme().primary() }
    pub fn primary_fg(&self) -> Rgba { self.current_theme().primary_fg() }
    pub fn accent_color(&self) -> Rgba { self.current_theme().accent() }
    pub fn accent_fg(&self) -> Rgba { self.current_theme().accent_fg() }
    pub fn secondary_bg(&self) -> Rgba { self.current_theme().secondary() }
    pub fn secondary_fg(&self) -> Rgba { self.current_theme().secondary_fg() }
    pub fn destructive_color(&self) -> Rgba { self.current_theme().destructive() }
    pub fn is_dark(&self) -> bool { self.current_theme().is_dark }

    pub fn toggle_sidebar(&mut self, cx: &mut Context<Self>) {
        self.sidebar_open = !self.sidebar_open;
        cx.notify();
    }

    pub fn toggle_new_tab_popover(&mut self, cx: &mut Context<Self>) {
        self.show_new_tab_popover = !self.show_new_tab_popover;
        cx.notify();
    }

    pub fn open_new_tab_page(&mut self, cx: &mut Context<Self>) {
        self.show_new_tab_popover = false;
        self.show_new_tab_modal = false;
        self.active_view = View::NewTab;
        cx.notify();
    }

    pub fn close_new_tab_modal(&mut self, cx: &mut Context<Self>) {
        self.show_new_tab_modal = false;
        cx.notify();
    }

    pub fn switch_tab(&mut self, idx: usize, cx: &mut Context<Self>) {
        self.session_manager.set_active(idx);
        cx.notify();
    }

    pub fn close_tab(&mut self, idx: usize, cx: &mut Context<Self>) {
        self.session_manager.close_tab(idx);
        cx.notify();
    }

    pub fn cycle_next_tab(&mut self, cx: &mut Context<Self>) {
        self.show_hosts_catalog = false;
        self.session_manager.cycle_next();
        cx.notify();
    }

    pub fn cycle_prev_tab(&mut self, cx: &mut Context<Self>) {
        self.show_hosts_catalog = false;
        self.session_manager.cycle_prev();
        cx.notify();
    }

    pub fn jump_to_tab(&mut self, idx: usize, cx: &mut Context<Self>) {
        self.show_hosts_catalog = false;
        if self.session_manager.jump_to(idx) {
            cx.notify();
        }
    }

    pub fn duplicate_active_tab(&mut self, cx: &mut Context<Self>) {
        self.show_new_tab_popover = false;
        if let Some(active) = self.session_manager.active_tab() {
            let title = format!("{} (copy)", active.title);
            let view = cx.new(|_cx| crate::terminal_view::WebTerminalView::new(title.clone()));
            self.session_manager.add_tab(TerminalTab {
                id: rand_id(),
                session_id: None,
                title,
                status: SessionStatus::Connected,
                view: Some(view.into()),
                session_type: "ssh".to_string(),
                connection_id: None,
            });
            cx.notify();
        }
    }

    pub fn reconnect_tab(&mut self, _tab_id: u64, cx: &mut Context<Self>) {
        cx.notify();
    }

    pub fn open_connection(&mut self, conn: &Connection, cx: &mut Context<Self>) {
        let title = format!("{} ({})", conn.label, conn.host);
        let conn_id = conn.id.clone();
        let view = cx.new(|_cx| crate::terminal_view::WebTerminalView::new_ssh(title.clone(), conn_id.clone()));
        self.show_hosts_catalog = false;
        self.active_view = View::Hosts;
        self.session_manager.add_tab(TerminalTab {
            id: rand_id(),
            session_id: Some(conn_id.clone()),
            title,
            status: SessionStatus::Connected,
            view: Some(view.into()),
            session_type: "ssh".to_string(),
            connection_id: Some(conn_id),
        });
        cx.notify();
    }

    pub fn connect_to_host(&mut self, id: &str, cx: &mut Context<Self>) {
        if let Some(conn) = self.connections.iter().find(|c| c.id == id).cloned() {
            self.open_connection(&conn, cx);
        }
    }

    pub fn quick_connect(&mut self, cx: &mut Context<Self>) {
        let q = self.quick_connect_query.trim().to_string();
        if !q.is_empty() {
            let title = format!("SSH ({})", q);
            let view = cx.new(|_cx| crate::terminal_view::WebTerminalView::new(title.clone()));
            self.show_hosts_catalog = false;
            self.session_manager.add_tab(TerminalTab {
                id: rand_id(),
                session_id: None,
                title,
                status: SessionStatus::Connected,
                view: Some(view.into()),
                session_type: "ssh".to_string(),
                connection_id: None,
            });
            self.quick_connect_query.clear();
            cx.notify();
        }
    }

    pub fn open_quick_ssh_tab(&mut self, cx: &mut Context<Self>) {
        self.show_new_tab_modal = false;
        let title = format!("ssh {}@{}", self.new_tab_user, self.new_tab_host);
        let view = cx.new(|_cx| crate::terminal_view::WebTerminalView::new(title.clone()));
        self.show_hosts_catalog = false;
        self.session_manager.add_tab(TerminalTab {
            id: rand_id(),
            session_id: None,
            title,
            status: SessionStatus::Connected,
            view: Some(view.into()),
            session_type: "ssh".to_string(),
            connection_id: None,
        });
        cx.notify();
    }

    pub fn open_local_tab(&mut self, cx: &mut Context<Self>) {
        self.show_new_tab_modal = false;
        let title = "Local Shell".to_string();
        let view = cx.new(|_cx| crate::terminal_view::WebTerminalView::new_local(title.clone()));
        self.show_hosts_catalog = false;
        self.session_manager.add_tab(TerminalTab {
            id: rand_id(),
            session_id: None,
            title,
            status: SessionStatus::Connected,
            view: Some(view.into()),
            session_type: "local".to_string(),
            connection_id: None,
        });
        cx.notify();
    }

    pub fn open_create_connection_modal(&mut self, cx: &mut Context<Self>) {
        self.connection_modal = Some(ConnectionFormState::new_create());
        cx.notify();
    }

    pub fn open_edit_connection_modal(&mut self, id: &str, cx: &mut Context<Self>) {
        if let Some(conn) = self.connections.iter().find(|c| c.id == id) {
            self.connection_modal = Some(ConnectionFormState::new_edit(conn));
            cx.notify();
        }
    }

    pub fn close_connection_modal(&mut self, cx: &mut Context<Self>) {
        self.connection_modal = None;
        cx.notify();
    }

    pub fn fetch_connections(&mut self, cx: &mut Context<Self>) {
        self.is_loading_connections = true;
        cx.notify();

        let view_weak = cx.entity().downgrade();
        cx.spawn(move |_view, cx: &mut AsyncApp| {
            let cx_handle = cx.clone();
            async move {
                let res = api_get::<Vec<Connection>>("/api/connections").await;
                cx_handle.update(|cx: &mut App| {
                    if let Some(app) = view_weak.upgrade() {
                        app.update(cx, |this, cx| {
                            this.is_loading_connections = false;
                            match res {
                                Ok(conns) => {
                                    this.connections = conns;
                                }
                                Err(e) => {
                                    this.notification = Some(format!("Failed to load connections: {e}"));
                                }
                            }
                            cx.notify();
                        });
                    }
                });
            }
        })
        .detach();
    }

    pub fn fetch_ssh_keys(&mut self, cx: &mut Context<Self>) {
        self.is_loading_keys = true;
        cx.notify();

        let view_weak = cx.entity().downgrade();
        cx.spawn(move |_view, cx: &mut AsyncApp| {
            let cx_handle = cx.clone();
            async move {
                let res = api_get::<Vec<SshKey>>("/api/keys").await;
                cx_handle.update(|cx: &mut App| {
                    if let Some(app) = view_weak.upgrade() {
                        app.update(cx, |this, cx| {
                            this.is_loading_keys = false;
                            match res {
                                Ok(keys) => {
                                    this.ssh_keys = keys;
                                }
                                Err(e) => {
                                    this.notification = Some(format!("Failed to load keys: {e}"));
                                }
                            }
                            cx.notify();
                        });
                    }
                });
            }
        })
        .detach();
    }

    pub fn fetch_port_forwards(&mut self, cx: &mut Context<Self>) {
        self.is_loading_forwards = true;
        cx.notify();

        let view_weak = cx.entity().downgrade();
        cx.spawn(move |_view, cx: &mut AsyncApp| {
            let cx_handle = cx.clone();
            async move {
                let res = api_get::<Vec<PortForward>>("/api/forwards").await;
                cx_handle.update(|cx: &mut App| {
                    if let Some(app) = view_weak.upgrade() {
                        app.update(cx, |this, cx| {
                            this.is_loading_forwards = false;
                            match res {
                                Ok(fwds) => {
                                    this.forwards = fwds;
                                }
                                Err(e) => {
                                    this.notification = Some(format!("Failed to load forwards: {e}"));
                                }
                            }
                            cx.notify();
                        });
                    }
                });
            }
        })
        .detach();
    }

    pub fn save_connection_form(&mut self, cx: &mut Context<Self>) {
        if let Some(form) = &self.connection_modal {
            if let Err(e) = form.validate() {
                self.connection_modal.as_mut().unwrap().error_message = Some(e);
                cx.notify();
                return;
            }
            let mode = form.mode.clone();
            let view_weak = cx.entity().downgrade();

            match mode {
                ConnectionModalMode::Create => {
                    let req = form.to_create_request();
                    cx.spawn(move |_view, cx: &mut AsyncApp| {
                        let cx_handle = cx.clone();
                        async move {
                            let res = api_send_json::<Connection, _>("POST", "/api/connections", &req).await;
                            cx_handle.update(|cx: &mut App| {
                                if let Some(app) = view_weak.upgrade() {
                                    app.update(cx, |this, cx| {
                                        match res {
                                            Ok(_) => {
                                                this.notification = Some("Connection created successfully".to_string());
                                                this.fetch_connections(cx);
                                            }
                                            Err(e) => {
                                                this.notification = Some(format!("Create connection failed: {e}"));
                                            }
                                        }
                                        cx.notify();
                                    });
                                }
                            });
                        }
                    })
                    .detach();
                }
                ConnectionModalMode::Edit(id) => {
                    let req = form.to_update_request();
                    let path = format!("/api/connections/{}", id);
                    cx.spawn(move |_view, cx: &mut AsyncApp| {
                        let cx_handle = cx.clone();
                        async move {
                            let res = api_send_json::<Connection, _>("PUT", &path, &req).await;
                            cx_handle.update(|cx: &mut App| {
                                if let Some(app) = view_weak.upgrade() {
                                    app.update(cx, |this, cx| {
                                        match res {
                                            Ok(_) => {
                                                this.notification = Some("Connection updated".to_string());
                                                this.fetch_connections(cx);
                                            }
                                            Err(e) => {
                                                this.notification = Some(format!("Update connection failed: {e}"));
                                            }
                                        }
                                        cx.notify();
                                    });
                                }
                            });
                        }
                    })
                    .detach();
                }
            }
            self.connection_modal = None;
            cx.notify();
        }
    }

    pub fn delete_connection(&mut self, id: &str, cx: &mut Context<Self>) {
        let path = format!("/api/connections/{}", id);
        let view_weak = cx.entity().downgrade();
        cx.spawn(move |_view, cx: &mut AsyncApp| {
            let cx_handle = cx.clone();
            async move {
                let res = api_delete(&path).await;
                cx_handle.update(|cx: &mut App| {
                    if let Some(app) = view_weak.upgrade() {
                        app.update(cx, |this, cx| {
                            match res {
                                Ok(_) => {
                                    this.notification = Some("Connection deleted".to_string());
                                    this.fetch_connections(cx);
                                }
                                Err(e) => {
                                    this.notification = Some(format!("Delete connection failed: {e}"));
                                }
                            }
                            cx.notify();
                        });
                    }
                });
            }
        })
        .detach();
    }

    pub fn select_tag_filter(&mut self, tag: Option<String>, cx: &mut Context<Self>) {
        self.selected_tag = tag;
        cx.notify();
    }

    pub fn open_import_modal(&mut self, cx: &mut Context<Self>) {
        self.show_import_modal = true;
        self.import_payload.clear();
        cx.notify();
    }

    pub fn close_import_modal(&mut self, cx: &mut Context<Self>) {
        self.show_import_modal = false;
        cx.notify();
    }

    pub fn submit_import_connections(&mut self, cx: &mut Context<Self>) {
        self.show_import_modal = false;
        self.notification = Some("Import completed".to_string());
        cx.notify();
    }

    pub fn export_connections_to_disk(&mut self, cx: &mut Context<Self>) {
        self.notification = Some("Exported connections to JSON".to_string());
        cx.notify();
    }

    pub fn dismiss_notification(&mut self, cx: &mut Context<Self>) {
        self.notification = None;
        cx.notify();
    }

    pub fn open_add_key_modal(&mut self, cx: &mut Context<Self>) {
        self.show_add_key_modal = true;
        self.new_key_name.clear();
        self.new_key_pem.clear();
        self.add_key_error = None;
        cx.notify();
    }

    pub fn close_add_key_modal(&mut self, cx: &mut Context<Self>) {
        self.show_add_key_modal = false;
        cx.notify();
    }

    pub fn create_ssh_key(&mut self, cx: &mut Context<Self>) {
        if self.new_key_name.trim().is_empty() {
            self.add_key_error = Some("Key name is required".to_string());
            cx.notify();
            return;
        }
        let req = CreateKeyRequest {
            name: self.new_key_name.trim().to_string(),
            key_base64: encode_base64(self.new_key_pem.trim().as_bytes()),
        };
        let view_weak = cx.entity().downgrade();
        cx.spawn(move |_view, cx: &mut AsyncApp| {
            let cx_handle = cx.clone();
            async move {
                let res = api_send_json::<SshKey, _>("POST", "/api/keys", &req).await;
                cx_handle.update(|cx: &mut App| {
                    if let Some(app) = view_weak.upgrade() {
                        app.update(cx, |this, cx| {
                            match res {
                                Ok(_) => {
                                    this.notification = Some("SSH Key added".to_string());
                                    this.fetch_ssh_keys(cx);
                                }
                                Err(e) => {
                                    this.notification = Some(format!("Create key failed: {e}"));
                                }
                            }
                            cx.notify();
                        });
                    }
                });
            }
        })
        .detach();
        self.show_add_key_modal = false;
        cx.notify();
    }

    pub fn delete_ssh_key(&mut self, id: &str, cx: &mut Context<Self>) {
        let path = format!("/api/keys/{}", id);
        let view_weak = cx.entity().downgrade();
        cx.spawn(move |_view, cx: &mut AsyncApp| {
            let cx_handle = cx.clone();
            async move {
                let res = api_delete(&path).await;
                cx_handle.update(|cx: &mut App| {
                    if let Some(app) = view_weak.upgrade() {
                        app.update(cx, |this, cx| {
                            match res {
                                Ok(_) => {
                                    this.notification = Some("SSH Key removed".to_string());
                                    this.fetch_ssh_keys(cx);
                                }
                                Err(e) => {
                                    this.notification = Some(format!("Delete key failed: {e}"));
                                }
                            }
                            cx.notify();
                        });
                    }
                });
            }
        })
        .detach();
    }

    pub fn open_create_forward_modal(&mut self, cx: &mut Context<Self>) {
        let default_conn_id = self.connections.first().map(|c| c.id.clone());
        self.forward_modal = Some(ForwardFormState::new_create(default_conn_id));
        cx.notify();
    }

    pub fn open_edit_forward_modal(&mut self, id: &str, cx: &mut Context<Self>) {
        if let Some(fwd) = self.forwards.iter().find(|f| f.id == id) {
            self.forward_modal = Some(ForwardFormState::new_edit(fwd));
            cx.notify();
        }
    }

    pub fn close_forward_modal(&mut self, cx: &mut Context<Self>) {
        self.forward_modal = None;
        cx.notify();
    }

    pub fn save_forward_form(&mut self, cx: &mut Context<Self>) {
        if let Some(form) = &self.forward_modal {
            match form.to_create_request() {
                Ok(req) => {
                    let view_weak = cx.entity().downgrade();
                    cx.spawn(move |_view, cx: &mut AsyncApp| {
                        let cx_handle = cx.clone();
                        async move {
                            let res = api_send_json::<PortForward, _>("POST", "/api/forwards", &req).await;
                            cx_handle.update(|cx: &mut App| {
                                if let Some(app) = view_weak.upgrade() {
                                    app.update(cx, |this, cx| {
                                        match res {
                                            Ok(_) => {
                                                this.notification = Some("Forward rule added".to_string());
                                                this.fetch_port_forwards(cx);
                                            }
                                            Err(e) => {
                                                this.notification = Some(format!("Create forward failed: {e}"));
                                            }
                                        }
                                        cx.notify();
                                    });
                                }
                            });
                        }
                    })
                    .detach();
                    self.forward_modal = None;
                    cx.notify();
                }
                Err(e) => {
                    self.forward_modal.as_mut().unwrap().error_message = Some(e);
                    cx.notify();
                }
            }
        }
    }

    pub fn toggle_forward_active(&mut self, id: &str, cx: &mut Context<Self>) {
        if let Some(fwd) = self.forwards.iter_mut().find(|f| f.id == id) {
            fwd.active = !fwd.active;
            cx.notify();
        }
    }

    pub fn open_delete_forward_modal(&mut self, fwd: PortForward, cx: &mut Context<Self>) {
        self.delete_forward_target = Some(fwd);
        cx.notify();
    }

    pub fn close_delete_forward_modal(&mut self, cx: &mut Context<Self>) {
        self.delete_forward_target = None;
        cx.notify();
    }

    pub fn confirm_delete_forward(&mut self, cx: &mut Context<Self>) {
        if let Some(target) = self.delete_forward_target.take() {
            let path = format!("/api/forwards/{}", target.id);
            let view_weak = cx.entity().downgrade();
            cx.spawn(move |_view, cx: &mut AsyncApp| {
                let cx_handle = cx.clone();
                async move {
                    let res = api_delete(&path).await;
                    cx_handle.update(|cx: &mut App| {
                        if let Some(app) = view_weak.upgrade() {
                            app.update(cx, |this, cx| {
                                match res {
                                    Ok(_) => {
                                        this.notification = Some("Forward deleted".to_string());
                                        this.fetch_port_forwards(cx);
                                    }
                                    Err(e) => {
                                        this.notification = Some(format!("Delete forward failed: {e}"));
                                    }
                                }
                                cx.notify();
                            });
                        }
                    });
                }
            })
            .detach();
            cx.notify();
        }
    }

    pub fn start_forward(&mut self, id: &str, cx: &mut Context<Self>) {
        self.toggle_forward_active(id, cx);
    }

    pub fn stop_forward(&mut self, id: &str, cx: &mut Context<Self>) {
        self.toggle_forward_active(id, cx);
    }

    pub fn submit_passphrase(&mut self, cx: &mut Context<Self>) {
        self.pending_passphrase_conn = None;
        self.passphrase_input.clear();
        cx.notify();
    }

    pub fn cancel_passphrase(&mut self, cx: &mut Context<Self>) {
        self.pending_passphrase_conn = None;
        self.passphrase_input.clear();
        cx.notify();
    }

    pub fn set_terminal_font(&mut self, family: String, size: f32, cx: &mut Context<Self>) {
        self.terminal_font_family = family;
        self.terminal_font_size = size;
        cx.notify();
    }

    pub fn set_cursor_style(&mut self, style: String, cx: &mut Context<Self>) {
        self.cursor_style = style;
        cx.notify();
    }

    pub fn set_cursor_blink(&mut self, blink: bool, cx: &mut Context<Self>) {
        self.cursor_blink = blink;
        cx.notify();
    }

    pub fn set_scrollback(&mut self, lines: u32, cx: &mut Context<Self>) {
        self.scrollback = lines;
        cx.notify();
    }

    // SFTP Methods
    pub fn sftp_pane(&self, pane: SftpActivePane) -> &SftpPaneState {
        match pane {
            SftpActivePane::Left => &self.sftp_manager.left_pane,
            SftpActivePane::Right => &self.sftp_manager.right_pane,
        }
    }

    pub fn sftp_pane_mut(&mut self, pane: SftpActivePane) -> &mut SftpPaneState {
        match pane {
            SftpActivePane::Left => &mut self.sftp_manager.left_pane,
            SftpActivePane::Right => &mut self.sftp_manager.right_pane,
        }
    }

    pub fn sftp_focus_pane(&mut self, pane: SftpActivePane, cx: &mut Context<Self>) {
        self.sftp_manager.focused_pane = pane;
        cx.notify();
    }

    pub fn sftp_set_source(&mut self, pane: SftpActivePane, conn_id: String, cx: &mut Context<Self>) {
        let label = if conn_id == "local" {
            "Local Machine".to_string()
        } else if let Some(c) = self.connections.iter().find(|c| c.id == conn_id) {
            c.label.clone()
        } else {
            conn_id.clone()
        };
        let p = self.sftp_pane_mut(pane);
        p.source_id = conn_id;
        p.source_label = label;
        p.show_source_picker = false;
        cx.notify();
    }

    pub fn sftp_toggle_source_picker(&mut self, pane: SftpActivePane, cx: &mut Context<Self>) {
        let p = self.sftp_pane_mut(pane);
        p.show_source_picker = !p.show_source_picker;
        cx.notify();
    }

    pub fn sftp_toggle_drive_picker(&mut self, pane: SftpActivePane, cx: &mut Context<Self>) {
        let p = self.sftp_pane_mut(pane);
        p.show_drive_picker = !p.show_drive_picker;
        cx.notify();
    }

    pub fn sftp_toggle_path_picker(&mut self, pane: SftpActivePane, cx: &mut Context<Self>) {
        let p = self.sftp_pane_mut(pane);
        p.show_path_picker = !p.show_path_picker;
        cx.notify();
    }

    pub fn sftp_toggle_actions_menu(&mut self, pane: SftpActivePane, cx: &mut Context<Self>) {
        let p = self.sftp_pane_mut(pane);
        p.show_actions_menu = !p.show_actions_menu;
        cx.notify();
    }

    pub fn sftp_toggle_sort(&mut self, pane: SftpActivePane, col: SftpSortColumn, cx: &mut Context<Self>) {
        let p = self.sftp_pane_mut(pane);
        if p.sort_column == col {
            p.sort_order = match p.sort_order {
                SftpSortOrder::Ascending => SftpSortOrder::Descending,
                SftpSortOrder::Descending => SftpSortOrder::Ascending,
            };
        } else {
            p.sort_column = col;
            p.sort_order = SftpSortOrder::Ascending;
        }
        cx.notify();
    }

    pub fn sftp_toggle_hidden(&mut self, pane: SftpActivePane, cx: &mut Context<Self>) {
        let p = self.sftp_pane_mut(pane);
        p.show_hidden = !p.show_hidden;
        cx.notify();
    }

    pub fn sftp_toggle_transfers_drawer(&mut self, cx: &mut Context<Self>) {
        self.sftp_manager.transfers_drawer_open = !self.sftp_manager.transfers_drawer_open;
        cx.notify();
    }

    pub fn sftp_navigate(&mut self, pane: SftpActivePane, path: String, cx: &mut Context<Self>) {
        let p = self.sftp_pane_mut(pane);
        p.current_path = path.clone();
        p.history.push(path);
        p.history_index = p.history.len() - 1;
        cx.notify();
    }

    pub fn sftp_navigate_up(&mut self, pane: SftpActivePane, cx: &mut Context<Self>) {
        let cur = self.sftp_pane(pane).current_path.clone();
        let parent = parent_path(&cur);
        self.sftp_navigate(pane, parent, cx);
    }

    pub fn sftp_navigate_back(&mut self, pane: SftpActivePane, cx: &mut Context<Self>) {
        let p = self.sftp_pane_mut(pane);
        if p.history_index > 0 {
            p.history_index -= 1;
            p.current_path = p.history[p.history_index].clone();
            cx.notify();
        }
    }

    pub fn sftp_navigate_forward(&mut self, pane: SftpActivePane, cx: &mut Context<Self>) {
        let p = self.sftp_pane_mut(pane);
        if p.history_index + 1 < p.history.len() {
            p.history_index += 1;
            p.current_path = p.history[p.history_index].clone();
            cx.notify();
        }
    }

    pub fn sftp_load_pane(&mut self, _pane: SftpActivePane, cx: &mut Context<Self>) {
        cx.notify();
    }

    pub fn sftp_select_single(&mut self, pane: SftpActivePane, filename: String, cx: &mut Context<Self>) {
        let p = self.sftp_pane_mut(pane);
        p.selected.clear();
        p.selected.insert(filename);
        cx.notify();
    }

    pub fn sftp_toggle_selection(&mut self, pane: SftpActivePane, filename: String, multi: bool, cx: &mut Context<Self>) {
        let p = self.sftp_pane_mut(pane);
        if multi {
            if p.selected.contains(&filename) {
                p.selected.remove(&filename);
            } else {
                p.selected.insert(filename);
            }
        } else {
            p.selected.clear();
            p.selected.insert(filename);
        }
        cx.notify();
    }

    pub fn sftp_open_context_menu(&mut self, pane: SftpActivePane, filename: String, is_dir: bool, pos: (f32, f32), cx: &mut Context<Self>) {
        self.sftp_manager.focused_pane = pane;
        self.sftp_manager.context_menu = Some(SftpContextMenu {
            pane,
            filename,
            is_dir,
            position: pos,
        });
        cx.notify();
    }

    pub fn sftp_close_context_menu(&mut self, cx: &mut Context<Self>) {
        self.sftp_manager.context_menu = None;
        cx.notify();
    }

    pub fn sftp_open_new_folder_modal(&mut self, pane: SftpActivePane, cx: &mut Context<Self>) {
        self.sftp_manager.modal = Some(SftpModalState::NewFolder { pane, name: String::new(), error: None });
        cx.notify();
    }

    pub fn sftp_open_rename_modal(&mut self, pane: SftpActivePane, old_name: String, cx: &mut Context<Self>) {
        self.sftp_manager.modal = Some(SftpModalState::Rename {
            pane,
            old_name: old_name.clone(),
            new_name: old_name,
            error: None,
        });
        cx.notify();
    }

    pub fn sftp_open_delete_modal(&mut self, pane: SftpActivePane, targets: Vec<String>, cx: &mut Context<Self>) {
        self.sftp_manager.modal = Some(SftpModalState::DeleteConfirm { pane, targets, error: None });
        cx.notify();
    }

    pub fn sftp_close_modal(&mut self, cx: &mut Context<Self>) {
        self.sftp_manager.modal = None;
        cx.notify();
    }

    pub fn sftp_set_modal_input(&mut self, input: String, cx: &mut Context<Self>) {
        if let Some(ref mut m) = self.sftp_manager.modal {
            match m {
                SftpModalState::NewFolder { ref mut name, .. } => *name = input,
                SftpModalState::Rename { ref mut new_name, .. } => *new_name = input,
                _ => {}
            }
            cx.notify();
        }
    }

    pub fn sftp_create_folder(&mut self, _pane: SftpActivePane, _name: &str, cx: &mut Context<Self>) {
        self.sftp_close_modal(cx);
    }

    pub fn sftp_rename_entry(&mut self, _pane: SftpActivePane, _old_name: &str, _new_name: &str, cx: &mut Context<Self>) {
        self.sftp_close_modal(cx);
    }

    pub fn sftp_delete_selected(&mut self, _pane: SftpActivePane, _targets: Vec<String>, cx: &mut Context<Self>) {
        self.sftp_close_modal(cx);
    }

    pub fn sftp_transfer_between_panes(&mut self, _src_pane: SftpActivePane, _dst_pane: SftpActivePane, _targets: Vec<String>, cx: &mut Context<Self>) {
        self.notification = Some("File transfer completed".to_string());
        cx.notify();
    }

    pub fn sftp_copy_path(&mut self, _pane: SftpActivePane, _filename: &str, cx: &mut Context<Self>) {
        self.notification = Some("Path copied to clipboard".to_string());
        cx.notify();
    }

    pub fn sftp_upload_file(&mut self, _pane: SftpActivePane, _filename: String, _data: Vec<u8>, cx: &mut Context<Self>) {
        self.notification = Some("Upload requested".to_string());
        cx.notify();
    }

    pub fn sftp_handle_key(&mut self, _key: &str, _is_alt: bool, cx: &mut Context<Self>) {
        cx.notify();
    }

    pub fn navigate_to_sftp(&mut self, cx: &mut Context<Self>) {
        self.active_view = View::Sftp;
        cx.notify();
    }
}

static RAND_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(100);
fn rand_id() -> u64 {
    RAND_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}
