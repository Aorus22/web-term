//! WebTerm terminal emulation engine (alacritty_terminal + GPUI).

pub mod colors;
pub mod event;
pub mod input;
pub mod mouse;
pub mod render;
pub mod terminal;
pub mod view;

pub use colors::ColorPalette;
pub use event::{GpuiEventProxy, TerminalEvent};
pub use input::keystroke_to_bytes;
pub use mouse::{pixel_to_cell, selection_type_from_clicks};
pub use render::{BackgroundRect, BatchedTextRun, CellDimensions, TerminalRenderer};
pub use terminal::{TermDimensions, Terminal, TerminalConfig};
pub use view::{BellCallback, InputCallback, ResizeCallback, TerminalView, TitleCallback};
