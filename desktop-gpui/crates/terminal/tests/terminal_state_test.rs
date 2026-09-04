use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::index::{Column, Line, Point};
use alacritty_terminal::selection::SelectionType;
use alacritty_terminal::term::cell::Flags;
use alacritty_terminal::vte::ansi::{Color, NamedColor, Rgb};
use webterm_terminal::event::TerminalEvent;
use webterm_terminal::Terminal;

#[test]
fn test_ansi_text_and_style_parsing() {
    let mut term = Terminal::new(80, 24);
    term.process_bytes(b"\x1b[31mRed\x1b[0m \x1b[1mBold\x1b[0m");

    term.with_term(|t| {
        let grid = t.grid();

        // Check "Red" characters and red foreground color
        let c0 = &grid[Line(0)][Column(0)];
        assert_eq!(c0.c, 'R');
        assert_eq!(c0.fg, Color::Named(NamedColor::Red));

        let c1 = &grid[Line(0)][Column(1)];
        assert_eq!(c1.c, 'e');

        let c2 = &grid[Line(0)][Column(2)];
        assert_eq!(c2.c, 'd');

        // Check space
        let c3 = &grid[Line(0)][Column(3)];
        assert_eq!(c3.c, ' ');

        // Check "Bold" characters and bold flag
        let c4 = &grid[Line(0)][Column(4)];
        assert_eq!(c4.c, 'B');
        assert!(c4.flags.contains(Flags::BOLD));
    });
}

#[test]
fn test_truecolor_24bit_rgb_parsing() {
    let mut term = Terminal::new(80, 24);
    // 24-bit TrueColor: \x1b[38;2;255;100;50m
    term.process_bytes(b"\x1b[38;2;255;100;50mRGB\x1b[0m");

    term.with_term(|t| {
        let grid = t.grid();
        let c0 = &grid[Line(0)][Column(0)];
        assert_eq!(c0.c, 'R');
        assert_eq!(
            c0.fg,
            Color::Spec(Rgb {
                r: 255,
                g: 100,
                b: 50,
            })
        );
    });
}

#[test]
fn test_alternate_screen_switching() {
    let mut term = Terminal::new(80, 24);

    // Primary screen text
    term.process_bytes(b"Primary Screen Text");
    assert!(!term.is_alternate_screen());

    // Enter alternate screen (\x1b[?1049h - smcup used by vim/htop) and home cursor
    term.process_bytes(b"\x1b[?1049h\x1b[H");
    assert!(term.is_alternate_screen());

    // Write on alternate screen
    term.process_bytes(b"Vim Editor Screen");
    term.with_term(|t| {
        let grid = t.grid();
        assert_eq!(grid[Line(0)][Column(0)].c, 'V');
    });

    // Leave alternate screen (\x1b[?1049l - rmcup)
    term.process_bytes(b"\x1b[?1049l");
    assert!(!term.is_alternate_screen());

    // Primary screen text restored
    term.with_term(|t| {
        let grid = t.grid();
        assert_eq!(grid[Line(0)][Column(0)].c, 'P');
    });
}

#[test]
fn test_scrollback_accumulation_and_scrolling() {
    let mut term = Terminal::new(80, 10);

    // Write 20 lines (exceeding 10 rows capacity)
    for i in 0..20 {
        let line = format!("Line {:02}\r\n", i);
        term.process_bytes(line.as_bytes());
    }

    // Grid history has accumulated lines
    term.with_term(|t| {
        let grid = t.grid();
        assert!(grid.history_size() > 0);
    });

    // Scroll up into history
    term.scroll_display(5);

    // Reset to bottom
    term.scroll_to_bottom();
}

#[test]
fn test_grid_resize() {
    let mut term = Terminal::new(80, 24);
    assert_eq!(term.cols(), 80);
    assert_eq!(term.rows(), 24);

    term.resize(120, 40);
    assert_eq!(term.cols(), 120);
    assert_eq!(term.rows(), 40);

    term.with_term(|t| {
        let grid = t.grid();
        assert_eq!(grid.columns(), 120);
        assert_eq!(grid.screen_lines(), 40);
    });
}

#[test]
fn test_selection_text_extraction() {
    let mut term = Terminal::new(80, 24);
    term.process_bytes(b"Selected Content Here");

    let start = Point::new(Line(0), Column(0));
    term.start_selection(start, SelectionType::Simple);

    let end = Point::new(Line(0), Column(7));
    term.update_selection(end);

    let selected = term.selection_text();
    assert!(selected.is_some());
    assert!(selected.unwrap().starts_with("Selected"));

    term.clear_selection();
    assert_eq!(term.selection_text(), None);
}

#[test]
fn test_event_channel_notifications() {
    let mut term = Terminal::new(80, 24);
    let rx = term.event_channel();

    // Trigger title OSC 2: \x1b]2;New Title\x07
    term.process_bytes(b"\x1b]2;New Title\x07");

    let event = rx.recv();
    assert!(event.is_ok());
    assert_eq!(event.unwrap(), TerminalEvent::Title("New Title".to_string()));
}
