//! WebTerm terminal emulation engine (alacritty_terminal + GPUI).

pub mod colors;
pub mod event;
pub mod terminal;

pub use colors::ColorPalette;
pub use event::{GpuiEventProxy, TerminalEvent};
pub use terminal::{TermDimensions, Terminal, TerminalConfig};
