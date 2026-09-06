//! WebTerm Core: target-agnostic data models, session management, actions, themes, and application state.

pub mod actions;
pub mod app_state;
pub mod session;
pub mod theme;
pub mod types;

#[cfg(target_arch = "wasm32")]
pub mod terminal_view;

pub use actions::*;
pub use app_state::*;
pub use session::*;
pub use theme::*;
pub use types::*;

#[cfg(not(target_arch = "wasm32"))]
pub use webterm_supervisor;
#[cfg(not(target_arch = "wasm32"))]
pub use webterm_backend_client;

#[cfg(target_arch = "wasm32")]
pub use types as webterm_backend_client;

#[cfg(target_arch = "wasm32")]
pub mod webterm_supervisor {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum BackendStatus {
        Starting,
        Ready,
        Failed { reason: String, stderr_tail: String },
        Crashed { exit_code: Option<i32> },
    }
}
