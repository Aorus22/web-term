//! WebTerm desktop library: state, views, session management, and UI.

pub mod actions;
pub mod app_state;
pub mod bundle;
pub mod icons;
pub mod session;
pub mod theme;
pub mod views;
pub mod window_state;

pub use webterm_backend_client as backend_client;
