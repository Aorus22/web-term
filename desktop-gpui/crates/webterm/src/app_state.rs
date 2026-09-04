//! Root application state, session orchestration, and view routing.

use std::sync::LazyLock;
use gpui::*;
use webterm_backend_client::{BackendClient, TerminalWsHandle, WsConnectRequest};
use webterm_settings::{DesktopSettings, Theme as SettingsTheme};
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
                                        if this.session_manager.is_empty() {
                                            this.open_local_tab(cx);
                                        }
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
                                        this.session_manager.attach_handle(tab_id, handle, session_id, cx);
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
                                        this.session_manager.attach_handle(tab_id, handle, session_id, cx);
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

    /// Close tab at given index.
    pub fn close_tab(&mut self, index: usize, cx: &mut Context<Self>) {
        self.session_manager.close_tab(index);
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
