---
phase: 21-terminal-rendering-alacritty-engine
plan: 02
subsystem: terminal-state
tags: [rust, alacritty_terminal, flume, terminal-state, vte-parser, alternate-screen, scrollback]

requires:
  - phase: 21-01
    provides: webterm-terminal crate scaffold and color palette
provides:
  - Terminal state engine wrapping alacritty_terminal::Term with thread-safe parking_lot::Mutex
  - GpuiEventProxy bridging Alacritty EventListener events (Wakeup, Title, Bell, Exit) to flume channel
  - VTE processor integration parsing byte streams into styled Grid cells
  - Alternate screen support (smcup / rmcup for vim/htop) preserving primary buffer
  - Scrollback history accumulation and viewport navigation
  - Selection manipulation (start, update, clear, extract text to String)
affects: [21-03, 22-ssh-terminal-sessions-tabs]

actuals:
  tokens: 1800
  tasks: 2
  commits: 1

tech-stack:
  added: []
  patterns: [thread-safe Term wrapper with parking_lot::Mutex, flume event bridge, VTE Processor advancement]

key-files:
  created:
    - desktop-gpui/crates/terminal/src/event.rs
    - desktop-gpui/crates/terminal/src/terminal.rs
    - desktop-gpui/crates/terminal/tests/terminal_state_test.rs
  modified:
    - desktop-gpui/crates/terminal/src/lib.rs

key-decisions:
  - "Used flume::unbounded channel inside GpuiEventProxy for lock-free event transmission to GPUI"
  - "Configured scrolling_history to 10,000 lines by default in TerminalConfig"
  - "TermDimensions adapts columns and rows to alacritty_terminal Dimensions trait"

patterns-established:
  - "Terminal::process_bytes advancing Processor over locked Term"
  - "Terminal::selection_text extracting selected grid range into String"

requirements-completed: [TERM-01, TERM-03, TERM-04]

coverage:
  - id: T1
    description: "Terminal parses ANSI and 24-bit TrueColor into styled grid cells"
    requirement: TERM-01
    verification:
      - kind: automated
        ref: "cargo test -p webterm-terminal --test terminal_state_test test_truecolor_24bit_rgb_parsing"
        status: pass
  - id: T2
    description: "Alternate screen switches for full-screen curses applications (vim, htop)"
    requirement: TERM-01
    verification:
      - kind: automated
        ref: "cargo test -p webterm-terminal --test terminal_state_test test_alternate_screen_switching"
        status: pass
  - id: T3
    description: "Scrollback history accumulates and shifts viewport"
    requirement: TERM-03
    verification:
      - kind: automated
        ref: "cargo test -p webterm-terminal --test terminal_state_test test_scrollback_accumulation_and_scrolling"
        status: pass
  - id: T4
    description: "Terminal grid resizes columns and rows"
    requirement: TERM-04
    verification:
      - kind: automated
        ref: "cargo test -p webterm-terminal --test terminal_state_test test_grid_resize"
        status: pass
---

# Plan 21-02: Terminal State Layer & Event Loop Summary

Plan 21-02 implemented the core terminal state engine and event loop in `webterm-terminal`, backed by `alacritty_terminal` 0.25.1.

## Deliverables

1. **Event Proxy (`event.rs`)**:
   - `GpuiEventProxy` implements `alacritty_terminal::event::EventListener`.
   - Bridges `Event::Wakeup`, `Event::Title`, `Event::Bell`, `Event::Exit`, and clipboard requests through an asynchronous `flume` channel to the UI.

2. **Terminal Engine (`terminal.rs`)**:
   - `Terminal` encapsulates `Arc<Mutex<Term<GpuiEventProxy>>>` and `Processor`.
   - `process_bytes(&mut self, bytes: &[u8])`: advances VTE parsing for incoming PTY/SSH byte streams.
   - `resize(&mut self, cols, rows)`: adapts to window/container dimensions.
   - Alternate screen detection via `is_alternate_screen()`.
   - Viewport scrolling via `scroll_display(delta)`, `scroll_to_bottom()`, and `scroll_to_top()`.
   - Text selection management via `start_selection()`, `update_selection()`, `clear_selection()`, and `selection_text()`.

3. **Automated Verification Suite (`tests/terminal_state_test.rs`)**:
   - 7 automated tests covering ANSI escape parsing, 24-bit RGB TrueColor, alternate screen switching, scrollback buffer accumulation, grid resize, text selection extraction, and event channel notifications.
   - 100% test pass rate with zero clippy warnings.
