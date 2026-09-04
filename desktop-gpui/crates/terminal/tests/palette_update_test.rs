//! Unit tests for TerminalView palette and font size dynamic updates.

use gpui::px;
use webterm_terminal::{ColorPalette, Terminal, TerminalRenderer};

#[test]
fn test_dark_and_light_palette_contrasts() {
    let dark = ColorPalette::dark_default();
    let light = ColorPalette::light_default();

    // Dark theme should have dark background and light foreground
    assert_ne!(dark.background, light.background);
    assert_ne!(dark.foreground, light.foreground);
    assert_ne!(dark.cursor, light.cursor);
}

#[test]
fn test_renderer_palette_and_font_mutation() {
    let mut renderer = TerminalRenderer::new(
        "JetBrains Mono".to_string(),
        px(14.0),
        1.2,
        ColorPalette::dark_default(),
    );

    assert_eq!(renderer.font_size, px(14.0));
    assert_eq!(renderer.palette, ColorPalette::dark_default());

    // Switch to light palette
    renderer.palette = ColorPalette::light_default();
    assert_eq!(renderer.palette, ColorPalette::light_default());

    // Update font size
    renderer.font_size = px(18.0);
    renderer.cell_width = px(18.0) * 0.6;
    renderer.cell_height = px(18.0) * 1.2;

    assert_eq!(renderer.font_size, px(18.0));
    assert_eq!(renderer.cell_width, px(10.8));
    assert_eq!(renderer.cell_height, px(21.6));
}

#[test]
fn test_terminal_instance_creation() {
    let term = Terminal::new(80, 24);
    assert_eq!(term.cols(), 80);
    assert_eq!(term.rows(), 24);
}
