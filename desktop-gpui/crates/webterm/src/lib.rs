//! WebTerm desktop library: state, views, session management, and UI.

pub use webterm_core::actions;
pub use webterm_core::app_state;
pub use webterm_core::session;
pub use webterm_core::types;
pub use webterm_core::app_state::AppState;
pub use webterm_core::webterm_backend_client as backend_client;
pub use webterm_core::webterm_supervisor;

pub use webterm_ui::icons;
pub use webterm_ui::theme;
pub use webterm_ui::views;
pub use webterm_ui::WebTermRoot;

pub mod bundle;
pub mod window_state;
