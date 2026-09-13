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

/// Map the settings cursor style string to an alacritty cursor shape.
pub fn cursor_shape_from_style(style: &str) -> Option<alacritty_terminal::vte::ansi::CursorShape> {
    match style {
        "underline" => Some(alacritty_terminal::vte::ansi::CursorShape::Underline),
        "bar" => Some(alacritty_terminal::vte::ansi::CursorShape::Beam),
        "block" => Some(alacritty_terminal::vte::ansi::CursorShape::Block),
        _ => None,
    }
}
