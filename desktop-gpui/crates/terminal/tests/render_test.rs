//! Integration tests for TerminalRenderer, mouse mapping, and keyboard translation.

use alacritty_terminal::term::cell::{Cell, Flags};
use alacritty_terminal::term::TermMode;
use alacritty_terminal::vte::ansi::{Color, NamedColor, Rgb};
use gpui::{point, px, Keystroke, Modifiers, MouseButton};
use webterm_terminal::{
    keystroke_to_bytes, pixel_to_cell, selection_type_from_clicks, BackgroundRect, ColorPalette,
    TerminalRenderer,
};

#[test]
fn test_renderer_initialization_and_cell_metrics() {
    let palette = ColorPalette::dark_default();
    let renderer = TerminalRenderer::new("JetBrains Mono".into(), px(16.0), 1.25, palette);

    assert_eq!(renderer.font_family, "JetBrains Mono");
    assert_eq!(renderer.font_size, px(16.0));
    assert_eq!(renderer.line_height_multiplier, 1.25);

    let dims = renderer.cell_dimensions();
    assert!(dims.width > px(0.0));
    assert!(dims.height > px(0.0));
    assert_eq!(dims.height, px(16.0) * 1.25);
}

#[test]
fn test_truecolor_and_styled_quad_generation() {
    let palette = ColorPalette::dark_default();
    let renderer = TerminalRenderer::new("JetBrains Mono".into(), px(14.0), 1.2, palette.clone());

    // Create a row of cells with TrueColor RGB and styling flags
    let rgb_fg = Color::Spec(Rgb {
        r: 0x22,
        g: 0xc5,
        b: 0x5e,
    }); // green
    let rgb_bg = Color::Spec(Rgb {
        r: 0x18,
        g: 0x18,
        b: 0x1b,
    }); // zinc-900

    let mut cell1 = Cell::default();
    cell1.c = 'T';
    cell1.fg = rgb_fg;
    cell1.bg = rgb_bg;
    cell1.flags = Flags::BOLD;

    let mut cell2 = Cell::default();
    cell2.c = 'E';
    cell2.fg = rgb_fg;
    cell2.bg = rgb_bg;
    cell2.flags = Flags::BOLD;

    let mut cell3 = Cell::default();
    cell3.c = 'R';
    cell3.fg = rgb_fg;
    cell3.bg = rgb_bg;
    cell3.flags = Flags::BOLD | Flags::UNDERLINE;

    let cells = vec![(0, cell1), (1, cell2), (2, cell3)];
    let (backgrounds, text_runs) = renderer.layout_row(0, cells.into_iter());

    // Both backgrounds have same color and row, should merge into one 3-cell rectangle
    assert_eq!(backgrounds.len(), 1);
    assert_eq!(backgrounds[0].start_col, 0);
    assert_eq!(backgrounds[0].end_col, 3);
    assert_eq!(backgrounds[0].color, palette.resolve(&rgb_bg));

    // Text runs: 'TE' (bold) is separate from 'R' (bold + underline)
    assert_eq!(text_runs.len(), 2);
    assert_eq!(text_runs[0].text, "TE");
    assert_eq!(text_runs[0].start_col, 0);
    assert!(text_runs[0].bold);
    assert!(!text_runs[0].underline);

    assert_eq!(text_runs[1].text, "R");
    assert_eq!(text_runs[1].start_col, 2);
    assert!(text_runs[1].bold);
    assert!(text_runs[1].underline);
}

#[test]
fn test_background_merging_logic() {
    let renderer = TerminalRenderer::new(
        "monospace".into(),
        px(12.0),
        1.2,
        ColorPalette::dark_default(),
    );

    let red = gpui::red();
    let blue = gpui::blue();

    let rects = vec![
        BackgroundRect {
            start_col: 0,
            end_col: 4,
            row: 0,
            color: red,
        },
        BackgroundRect {
            start_col: 4,
            end_col: 8,
            row: 0,
            color: red,
        },
        BackgroundRect {
            start_col: 8,
            end_col: 12,
            row: 0,
            color: blue,
        },
        BackgroundRect {
            start_col: 0,
            end_col: 4,
            row: 1,
            color: red,
        },
    ];

    let merged = renderer.merge_backgrounds(rects);
    assert_eq!(merged.len(), 3);
    // Merged row 0 red (0..8)
    assert_eq!(merged[0].start_col, 0);
    assert_eq!(merged[0].end_col, 8);
    assert_eq!(merged[0].row, 0);
    assert_eq!(merged[0].color, red);

    // Row 0 blue (8..12)
    assert_eq!(merged[1].start_col, 8);
    assert_eq!(merged[1].end_col, 12);
    assert_eq!(merged[1].row, 0);
    assert_eq!(merged[1].color, blue);

    // Row 1 red (0..4)
    assert_eq!(merged[2].start_col, 0);
    assert_eq!(merged[2].end_col, 4);
    assert_eq!(merged[2].row, 1);
    assert_eq!(merged[2].color, red);
}

#[test]
fn test_keystroke_escape_sequence_coverage() {
    // 1. Enter, Escape, Backspace, Tab
    let enter = Keystroke {
        modifiers: Modifiers::default(),
        key: "enter".into(),
        key_char: None,
    };
    assert_eq!(keystroke_to_bytes(&enter, TermMode::empty()), Some(b"\r".to_vec()));

    let esc = Keystroke {
        modifiers: Modifiers::default(),
        key: "escape".into(),
        key_char: None,
    };
    assert_eq!(keystroke_to_bytes(&esc, TermMode::empty()), Some(b"\x1b".to_vec()));

    let bs = Keystroke {
        modifiers: Modifiers::default(),
        key: "backspace".into(),
        key_char: None,
    };
    assert_eq!(keystroke_to_bytes(&bs, TermMode::empty()), Some(b"\x7f".to_vec()));

    let tab = Keystroke {
        modifiers: Modifiers::default(),
        key: "tab".into(),
        key_char: None,
    };
    assert_eq!(keystroke_to_bytes(&tab, TermMode::empty()), Some(b"\t".to_vec()));

    let shift_tab = Keystroke {
        modifiers: Modifiers {
            shift: true,
            ..Default::default()
        },
        key: "tab".into(),
        key_char: None,
    };
    assert_eq!(keystroke_to_bytes(&shift_tab, TermMode::empty()), Some(b"\x1b[Z".to_vec()));

    // 2. Arrows in normal and app cursor mode
    let up = Keystroke {
        modifiers: Modifiers::default(),
        key: "up".into(),
        key_char: None,
    };
    assert_eq!(keystroke_to_bytes(&up, TermMode::empty()), Some(b"\x1b[A".to_vec()));
    assert_eq!(keystroke_to_bytes(&up, TermMode::APP_CURSOR), Some(b"\x1bOA".to_vec()));

    let down = Keystroke {
        modifiers: Modifiers::default(),
        key: "down".into(),
        key_char: None,
    };
    assert_eq!(keystroke_to_bytes(&down, TermMode::empty()), Some(b"\x1b[B".to_vec()));
    assert_eq!(keystroke_to_bytes(&down, TermMode::APP_CURSOR), Some(b"\x1bOB".to_vec()));

    let right = Keystroke {
        modifiers: Modifiers::default(),
        key: "right".into(),
        key_char: None,
    };
    assert_eq!(keystroke_to_bytes(&right, TermMode::empty()), Some(b"\x1b[C".to_vec()));
    assert_eq!(keystroke_to_bytes(&right, TermMode::APP_CURSOR), Some(b"\x1bOC".to_vec()));

    let left = Keystroke {
        modifiers: Modifiers::default(),
        key: "left".into(),
        key_char: None,
    };
    assert_eq!(keystroke_to_bytes(&left, TermMode::empty()), Some(b"\x1b[D".to_vec()));
    assert_eq!(keystroke_to_bytes(&left, TermMode::APP_CURSOR), Some(b"\x1bOD".to_vec()));

    // 3. Navigation and function keys
    let home = Keystroke {
        modifiers: Modifiers::default(),
        key: "home".into(),
        key_char: None,
    };
    assert_eq!(keystroke_to_bytes(&home, TermMode::empty()), Some(b"\x1b[H".to_vec()));

    let end = Keystroke {
        modifiers: Modifiers::default(),
        key: "end".into(),
        key_char: None,
    };
    assert_eq!(keystroke_to_bytes(&end, TermMode::empty()), Some(b"\x1b[F".to_vec()));

    let f5 = Keystroke {
        modifiers: Modifiers::default(),
        key: "f5".into(),
        key_char: None,
    };
    assert_eq!(keystroke_to_bytes(&f5, TermMode::empty()), Some(b"\x1b[15~".to_vec()));

    let f12 = Keystroke {
        modifiers: Modifiers::default(),
        key: "f12".into(),
        key_char: None,
    };
    assert_eq!(keystroke_to_bytes(&f12, TermMode::empty()), Some(b"\x1b[24~".to_vec()));

    // 4. Control keys
    let ctrl_c = Keystroke {
        modifiers: Modifiers {
            control: true,
            ..Default::default()
        },
        key: "c".into(),
        key_char: None,
    };
    assert_eq!(keystroke_to_bytes(&ctrl_c, TermMode::empty()), Some(vec![0x03]));

    let ctrl_z = Keystroke {
        modifiers: Modifiers {
            control: true,
            ..Default::default()
        },
        key: "z".into(),
        key_char: None,
    };
    assert_eq!(keystroke_to_bytes(&ctrl_z, TermMode::empty()), Some(vec![0x1a]));
}

#[test]
fn test_mouse_reporting_and_coordinates() {
    let pos = point(px(120.0), px(60.0));
    let origin = point(px(20.0), px(10.0));
    let cell_w = px(10.0);
    let cell_h = px(20.0);

    let cell = pixel_to_cell(pos, origin, cell_w, cell_h, 80, 24);
    assert_eq!(cell.column.0, 10);
    assert_eq!(cell.line.0, 2);

    let mode = TermMode::MOUSE_REPORT_CLICK;
    let sgr_press = webterm_terminal::mouse::mouse_button_report(
        MouseButton::Left,
        true,
        cell,
        0,
        mode,
    );
    assert_eq!(
        String::from_utf8(sgr_press.unwrap()).unwrap(),
        "\x1b[<0;11;3M"
    );

    let sgr_release = webterm_terminal::mouse::mouse_button_report(
        MouseButton::Left,
        false,
        cell,
        0,
        mode,
    );
    assert_eq!(
        String::from_utf8(sgr_release.unwrap()).unwrap(),
        "\x1b[<0;11;3m"
    );

    let sel_type = selection_type_from_clicks(1);
    assert_eq!(sel_type, alacritty_terminal::selection::SelectionType::Simple);
}

#[test]
fn test_inverse_and_dim_cells() {
    let palette = ColorPalette::dark_default();
    let renderer = TerminalRenderer::new("JetBrains Mono".into(), px(14.0), 1.2, palette.clone());

    let mut inv_cell = Cell::default();
    inv_cell.c = 'X';
    inv_cell.fg = Color::Named(NamedColor::Foreground);
    inv_cell.bg = Color::Named(NamedColor::Background);
    inv_cell.flags = Flags::INVERSE;

    let cells = vec![(0, inv_cell)];
    let (_bgs, runs) = renderer.layout_row(0, cells.into_iter());

    assert_eq!(runs.len(), 1);
    // Inverted: fg should be background color, bg should be foreground color
    assert_eq!(runs[0].fg_color, palette.background);
    assert_eq!(runs[0].bg_color, palette.foreground);
}
