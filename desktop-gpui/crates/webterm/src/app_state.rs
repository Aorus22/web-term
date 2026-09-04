//! Root application state, session orchestration, and view routing.

use std::sync::LazyLock;
use gpui::*;
use webterm_backend_client::{BackendClient, TerminalWsHandle, WsConnectRequest};
use webterm_settings::{DesktopSettings, SavedSessionTab, Theme as SettingsTheme};
use webterm_supervisor::{BackendInfo, BackendStatus, SpawnOptions, Supervisor};

use crate::session::{SessionStatus, TerminalSessionManager, TerminalTab};
use crate::views::{nav::render_nav_shell, status::render_status_page};

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
        }
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
        self.persist_open_sessions();
        cx.notify();
    }

    /// Switch active tab to given index.
    pub fn switch_tab(&mut self, index: usize, cx: &mut Context<Self>) {
        if self.session_manager.switch_tab(index) {
            cx.notify();
        }
    }

    /// Cycle to next tab (Ctrl+Tab).
    pub fn cycle_next_tab(&mut self, cx: &mut Context<Self>) {
        self.session_manager.cycle_next();
        cx.notify();
    }

    /// Cycle to previous tab (Ctrl+Shift+Tab).
    pub fn cycle_prev_tab(&mut self, cx: &mut Context<Self>) {
        self.session_manager.cycle_prev();
        cx.notify();
    }

    /// Jump directly to tab (Alt+1..9).
    pub fn jump_to_tab(&mut self, index: usize, cx: &mut Context<Self>) {
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
