//! WebTerm UI: pure GPUI presentation layer, themes, icons, and views.

pub mod icons;
pub mod theme;
pub mod views;

pub use webterm_core::{actions, app_state, session, types};
pub use webterm_core::app_state::AppState;
pub use webterm_core::webterm_supervisor;
pub use webterm_core::webterm_backend_client;
pub use icons::*;
pub use theme::*;

pub mod webterm_settings {
    pub use webterm_core::app_state::Theme;
}

pub mod window_state {
    use gpui::Window;

    pub fn toggle_maximize(window: &mut Window) -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        {
            window.is_maximized()
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = window;
            false
        }
    }
}

use gpui::*;

pub struct WebTermRoot {
    pub app: Entity<AppState>,
}

impl WebTermRoot {
    pub fn new(app: Entity<AppState>) -> Self {
        Self { app }
    }
}

impl Render for WebTermRoot {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.app.update(cx, |this, cx| {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let key_secret = this.settings.encryption_key.as_deref();
                match this.backend_status {
                    crate::webterm_supervisor::BackendStatus::Ready => {
                        crate::views::nav::render_nav_shell(this, cx)
                    }
                    _ => {
                        let status = this.backend_status.clone();
                        crate::views::status::render_status_page(
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
            #[cfg(target_arch = "wasm32")]
            {
                crate::views::nav::render_nav_shell(this, cx)
            }
        })
    }
}
