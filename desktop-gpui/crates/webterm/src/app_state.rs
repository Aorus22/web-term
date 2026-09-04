//! Root application state and view routing.

use gpui::*;
use webterm_backend_client::BackendClient;
use webterm_settings::{DesktopSettings, Theme as SettingsTheme};
use webterm_supervisor::{BackendStatus, SpawnOptions, Supervisor};
use crate::views::{nav::render_nav_shell, status::render_status_page};

/// Active view in the main content area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Hosts,
    Keys,
    Sftp,
    Settings,
}

/// Root application entity holding state, client handle, and routing views.
pub struct AppState {
    pub backend_status: BackendStatus,
    pub client: Option<BackendClient>,
    pub theme: SettingsTheme,
    pub active_view: View,
    pub settings: DesktopSettings,
    pub spawn_opts: Option<SpawnOptions>,
}

use std::sync::LazyLock;
use webterm_supervisor::BackendInfo;

static TOKIO_RT: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to initialize Tokio runtime for supervisor")
});

enum SupervisorEvent {
    Status(BackendStatus),
    Ready(BackendInfo),
    Failed { reason: String, stderr_tail: String },
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
