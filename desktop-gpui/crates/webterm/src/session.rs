//! Multi-tab terminal session manager and tab models.

use gpui::*;
use webterm_backend_client::{TerminalWsHandle, WsConnectRequest};
use webterm_terminal::{ColorPalette, Terminal, TerminalRenderer, TerminalView};

/// Status of a terminal session tab.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SessionStatus {
    Connecting,
    Connected,
    Reconnecting {
        attempt: usize,
        next_retry_secs: usize,
    },
    Disconnected(Option<String>),
}

/// An open terminal tab holding view state, transport handle, and metadata.
#[allow(dead_code)]
pub struct TerminalTab {
    pub id: u64,
    pub session_id: Option<String>,
    pub title: String,
    pub status: SessionStatus,
    pub view: Option<Entity<TerminalView>>,
    pub ws_handle: Option<TerminalWsHandle>,
    pub session_type: String, // "local" | "ssh"
    pub connection_id: Option<String>,
    pub last_connect_req: Option<WsConnectRequest>,
}

#[allow(dead_code)]
impl TerminalTab {
    /// Create a new terminal tab with an active terminal view.
    pub fn new(
        id: u64,
        title: impl Into<String>,
        session_type: impl Into<String>,
        connection_id: Option<String>,
        palette: ColorPalette,
        scrollback_limit: usize,
        cursor_style: &str,
        cx: &mut App,
    ) -> Self {
        let cursor_shape = webterm_terminal::cursor_shape_from_style(cursor_style);
        let config = webterm_terminal::terminal::TerminalConfig { scrollback_limit };

        let view = cx.new(|cx| {
            let terminal = Terminal::with_config(80, 24, config);
            let mut renderer =
                TerminalRenderer::new("JetBrains Mono".to_string(), px(14.0), 1.2, palette);
            renderer.cursor_shape_override = cursor_shape;
            TerminalView::new(terminal, cx).with_renderer(renderer)
        });

        Self {
            id,
            session_id: None,
            title: title.into(),
            status: SessionStatus::Connecting,
            view: Some(view),
            ws_handle: None,
            session_type: session_type.into(),
            connection_id,
            last_connect_req: None,
        }
    }

    /// Create a headless tab without a GPUI entity (for testing).
    pub fn new_headless(
        id: u64,
        title: impl Into<String>,
        session_type: impl Into<String>,
        connection_id: Option<String>,
    ) -> Self {
        Self {
            id,
            session_id: None,
            title: title.into(),
            status: SessionStatus::Connecting,
            view: None,
            ws_handle: None,
            session_type: session_type.into(),
            connection_id,
            last_connect_req: None,
        }
    }

    /// Return true if the tab is connected and live.
    pub fn is_connected(&self) -> bool {
        self.status == SessionStatus::Connected
    }

    /// Return true if this is a local shell terminal tab.
    pub fn is_local(&self) -> bool {
        self.session_type == "local"
    }

    /// Return true if the session is disconnected and can be restarted.
    pub fn is_restartable(&self) -> bool {
        matches!(self.status, SessionStatus::Disconnected(_))
    }

    /// Explicitly disconnect this tab's transport.
    pub fn disconnect(&mut self) {
        if let Some(ref handle) = self.ws_handle {
            let _ = handle.disconnect();
        }
        self.ws_handle = None;
        self.status = SessionStatus::Disconnected(None);
    }
}

/// Orchestrates multiple terminal tabs and handles switching, closing, and shortcuts.
#[derive(Default)]
pub struct TerminalSessionManager {
    tabs: Vec<TerminalTab>,
    active_tab_index: usize,
    next_tab_id: u64,
}

#[allow(dead_code)]
impl TerminalSessionManager {
    /// Create an empty session manager.
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab_index: 0,
            next_tab_id: 0,
        }
    }

    /// Generate a unique sequential tab ID.
    pub fn alloc_tab_id(&mut self) -> u64 {
        self.next_tab_id += 1;
        self.next_tab_id
    }

    /// List of all open tabs.
    pub fn tabs(&self) -> &[TerminalTab] {
        &self.tabs
    }

    /// Mutable list of all open tabs.
    pub fn tabs_mut(&mut self) -> &mut [TerminalTab] {
        &mut self.tabs
    }

    /// Total number of open tabs.
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    /// Return true if no tabs are open.
    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    /// Currently active tab index (0-indexed).
    pub fn active_index(&self) -> usize {
        self.active_tab_index
    }

    /// Reference to the currently active tab, if one exists.
    pub fn active_tab(&self) -> Option<&TerminalTab> {
        self.tabs.get(self.active_tab_index)
    }

    /// Mutable reference to the currently active tab, if one exists.
    pub fn active_tab_mut(&mut self) -> Option<&mut TerminalTab> {
        self.tabs.get_mut(self.active_tab_index)
    }

    /// Add an existing tab to the manager and make it active.
    pub fn add_tab(&mut self, tab: TerminalTab) -> usize {
        self.tabs.push(tab);
        self.active_tab_index = self.tabs.len() - 1;
        self.active_tab_index
    }

    /// Switch to tab at given index.
    pub fn switch_tab(&mut self, index: usize) -> bool {
        if index < self.tabs.len() {
            self.active_tab_index = index;
            true
        } else {
            false
        }
    }

    /// Close tab at given index. Disconnects transport and clamps active index.
    pub fn close_tab(&mut self, index: usize) -> Option<TerminalTab> {
        if index >= self.tabs.len() {
            return None;
        }

        let mut tab = self.tabs.remove(index);
        tab.disconnect();

        if self.tabs.is_empty() {
            self.active_tab_index = 0;
        } else if self.active_tab_index >= self.tabs.len() {
            self.active_tab_index = self.tabs.len() - 1;
        }

        Some(tab)
    }

    /// Cycle to next tab (Ctrl+Tab).
    pub fn cycle_next(&mut self) -> usize {
        if self.tabs.is_empty() {
            return 0;
        }
        self.active_tab_index = (self.active_tab_index + 1) % self.tabs.len();
        self.active_tab_index
    }

    /// Cycle to previous tab (Ctrl+Shift+Tab).
    pub fn cycle_prev(&mut self) -> usize {
        if self.tabs.is_empty() {
            return 0;
        }
        self.active_tab_index = if self.active_tab_index == 0 {
            self.tabs.len() - 1
        } else {
            self.active_tab_index - 1
        };
        self.active_tab_index
    }

    /// Jump directly to tab at 0-indexed position (Alt+1..9).
    pub fn jump_to(&mut self, index: usize) -> bool {
        self.switch_tab(index)
    }

    /// Update status for a specific tab ID.
    pub fn set_tab_status(&mut self, id: u64, status: SessionStatus) {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) {
            tab.status = status;
        }
    }

    /// Update title for a specific tab ID.
    pub fn set_tab_title(&mut self, id: u64, title: impl Into<String>) {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) {
            tab.title = title.into();
        }
    }

    /// Attach a WebSocket handle to a specific tab ID and bind I/O pumps.
    pub fn attach_handle(
        &mut self,
        tab_id: u64,
        handle: TerminalWsHandle,
        session_id: String,
        app_weak: Option<WeakEntity<crate::app_state::AppState>>,
        cx: &mut App,
    ) {
        let tab = match self.tabs.iter_mut().find(|t| t.id == tab_id) {
            Some(t) => t,
            None => return,
        };

        tab.session_id = Some(session_id);
        tab.status = SessionStatus::Connected;

        let output_rx = handle.output_receiver();
        let handle_for_input = handle.clone();
        let handle_for_resize = handle.clone();

        if let Some(ref view) = tab.view {
            // 1. Hook input callback: forwarding user keystrokes into WS binary messages
            view.update(cx, |this, _cx| {
                this.set_input_callback(move |bytes| {
                    let _ = handle_for_input.send_input(bytes.to_vec());
                });
                this.set_resize_callback(move |cols, rows| {
                    let _ = handle_for_resize.resize(cols as u16, rows as u16);
                });
            });

            // 2. Spawn async task pumping backend PTY bytes into the terminal
            let view_weak = view.downgrade();
            let app_weak_clone = app_weak.clone();
            cx.spawn(move |cx: &mut AsyncApp| {
                let view_weak = view_weak.clone();
                let cx_handle = cx.clone();
                async move {
                    while let Ok(bytes) = output_rx.recv_async().await {
                        let view_weak = view_weak.clone();
                        let ok = cx_handle.update(|cx: &mut App| {
                            if let Some(view) = view_weak.upgrade() {
                                view.update(cx, |this, cx| {
                                    this.terminal().lock().process_bytes(&bytes);
                                    cx.notify();
                                });
                                true
                            } else {
                                false
                            }
                        });
                        if !ok {
                            break;
                        }
                    }

                    // WebSocket output dropped: inform AppState for reconnection handling
                    if let Some(ref app_weak) = app_weak_clone {
                        let app_weak = app_weak.clone();
                        cx_handle.update(move |cx: &mut App| {
                            if let Some(app) = app_weak.upgrade() {
                                app.update(cx, |this, cx| {
                                    this.on_tab_dropped(tab_id, cx);
                                });
                            }
                        });
                    }
                }
            })
            .detach();
        }

        tab.ws_handle = Some(handle);
    }
}
