use std::time::Instant;
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::term::cell::{Cell, Flags};
use alacritty_terminal::vte::ansi::{Color, Rgb};
use gpui::px;
use webterm_terminal::terminal::{Terminal, TerminalConfig};
use webterm_terminal::render::TerminalRenderer;
use webterm_terminal::colors::ColorPalette;

#[test]
fn test_terminal_high_throughput_burst() {
    let mut term = Terminal::new(80, 24);

    // Prepare a large burst buffer of 20,000 formatted ANSI log lines
    let mut burst_data = Vec::with_capacity(1024 * 1024);
    for i in 0..20_000 {
        // Multi-color ANSI sequence mimicking realistic high-throughput CLI tool (e.g. cargo, docker, htop)
        let line = format!(
            "\x1b[32m[INFO]\x1b[0m \x1b[1;34mworker-{:03}\x1b[0m: processed packet {:06} with \x1b[38;2;255;165;0mstatus=200 OK\x1b[0m in 1.42ms\r\n",
            i % 64,
            i
        );
        burst_data.extend_from_slice(line.as_bytes());
    }

    let start = Instant::now();
    term.process_bytes(&burst_data);
    let elapsed = start.elapsed();

    // Verify throughput: 20,000 lines processed
    println!(
        "High-throughput test: processed {} bytes (20,000 lines) in {:?}",
        burst_data.len(),
        elapsed
    );

    // Performance SLA: Must process >10,000 lines/second (< 2.0s on any desktop environment)
    assert!(
        elapsed.as_secs_f64() < 2.0,
        "Processing took {:?}, exceeding SLA of 2.0 seconds",
        elapsed
    );

    // Terminal state remains healthy
    assert_eq!(term.cols(), 80);
    assert_eq!(term.rows(), 24);
}

#[test]
fn test_terminal_bounded_memory_under_infinite_stream() {
    // Configure bounded scrollback limit of 500 lines
    let config = TerminalConfig {
        scrollback_limit: 500,
    };
    let mut term = Terminal::with_config(80, 24, config);

    // Stream 15,000 lines (30x the scrollback capacity)
    for i in 0..15_000 {
        let line = format!("Infinite log stream line {:06}\r\n", i);
        term.process_bytes(line.as_bytes());
    }

    // Verify that the scrollback history buffer is strictly capped at 500 lines
    term.with_term(|t| {
        let history_size = t.grid().history_size();
        assert!(
            history_size <= 500,
            "History size {} exceeded bounded scrollback limit of 500",
            history_size
        );
        assert_eq!(
            history_size, 500,
            "History size should be full at exactly 500"
        );
    });
}

#[test]
fn test_renderer_row_batching_performance() {
    let renderer = TerminalRenderer::new("JetBrains Mono".into(), px(14.0), 1.2, ColorPalette::dark_default());

    // Create 1,000 simulated terminal rows each having 120 cells with alternating TrueColor and bold styles
    let mut rows_cells = Vec::with_capacity(1000);
    for row in 0..1000 {
        let mut row_cells = Vec::with_capacity(120);
        for col in 0..120 {
            let mut cell = Cell::default();
            cell.c = if col % 2 == 0 { 'A' } else { ' ' };
            cell.fg = Color::Spec(Rgb {
                r: (col * 2) as u8,
                g: ((row + col) % 255) as u8,
                b: (row % 255) as u8,
            });
            cell.bg = Color::Spec(Rgb {
                r: 0x18,
                g: 0x18,
                b: 0x1b,
            });
            if col % 3 == 0 {
                cell.flags.insert(Flags::BOLD);
            }
            row_cells.push((col, cell));
        }
        rows_cells.push(row_cells);
    }

    let start = Instant::now();
    let mut total_quads = 0;
    let mut total_runs = 0;
    for (row_idx, cells) in rows_cells.into_iter().enumerate() {
        let (backgrounds, text_runs) = renderer.layout_row(row_idx, cells.into_iter());
        total_quads += backgrounds.len();
        total_runs += text_runs.len();
    }
    let elapsed = start.elapsed();

    println!(
        "Renderer row batching performance: processed 1,000 rows ({} quads, {} text runs) in {:?}",
        total_quads,
        total_runs,
        elapsed
    );

    // 1,000 rows layout batching (120,000 text runs) should complete comfortably in < 250ms even in unoptimized debug mode
    assert!(
        elapsed.as_millis() < 250,
        "Row batching took {:?}, exceeding 250ms SLA in debug mode",
        elapsed
    );
    assert!(total_quads > 0);
    assert!(total_runs > 0);
}
