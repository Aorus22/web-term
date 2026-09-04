---
phase: 21-terminal-rendering-alacritty-engine
plan: 03
subsystem: terminal-rendering-view
tags: [rust, alacritty_terminal, gpui, rendering, canvas, text-shaping, mouse-selection, clipboard, scrollback]

requires:
  - phase: 21-02
    provides: Terminal state engine, VTE parser, and event bridge
provides:
  - TerminalRenderer with cell metrics measurement ('M' reference char), background quad coalescing, and text run batching
  - Keystroke translation from GPUI KeyDownEvent into ANSI/VT escape sequences (arrows, function keys, navigation, control keys)
  - Mouse event handling: pixel-to-cell coordinate conversion, selection type mapping, and SGR (1006) mouse reporting
  - Interactive TerminalView implementing gpui::Render and Focusable with mouse drag selection, scroll wheel navigation, and clipboard copy/paste
  - Embedded TerminalView integrated into the desktop application shell (nav.rs)
affects: [22-ssh-terminal-sessions-tabs, 23-hosts-keys, 24-local-terminal]

actuals:
  tokens: 2200
  tasks: 2
  commits: 1

tech-stack:
  added: []
  patterns: [GPUI canvas custom painting, batched text runs, background quad coalescing, native GPUI clipboard integration]

key-files:
  created:
    - desktop-gpui/crates/terminal/src/render.rs
    - desktop-gpui/crates/terminal/src/input.rs
    - desktop-gpui/crates/terminal/src/mouse.rs
    - desktop-gpui/crates/terminal/src/view.rs
    - desktop-gpui/crates/terminal/tests/render_test.rs
  modified:
    - desktop-gpui/crates/terminal/src/lib.rs
    - desktop-gpui/crates/webterm/Cargo.toml
    - desktop-gpui/crates/webterm/src/app_state.rs
    - desktop-gpui/crates/webterm/src/views/nav.rs

key-decisions:
  - "Used GPUI native cx.write_to_clipboard and cx.read_from_clipboard for clipboard copy/paste without extra crates"
  - "Batched text runs by style attributes and coalesced adjacent background quads to minimize GPU draw calls"
  - "Mapped mouse click count: 1 = simple, 2 = semantic word, 3+ = line selection"
  - "Reset scrollback offset to bottom upon keyboard input so typing is always visible"
  - "Observed container bounds inside canvas prepaint to automatically synchronize cols/rows"

patterns-established:
  - "TerminalView wraps Terminal and TerminalRenderer with GPUI Render and Focusable traits"
  - "canvas prepaint/paint pattern for low-level high-performance terminal grid rendering"

requirements-completed: [TERM-01, TERM-02, TERM-03, TERM-04]

coverage:
  - id: R1
    description: "TerminalView renders terminal grid with batched text runs and background quads in GPUI"
    requirement: TERM-01
    verification:
      - kind: automated
        ref: "cargo test -p webterm-terminal --test render_test test_truecolor_and_styled_quad_generation"
        status: pass
  - id: R2
    description: "Mouse dragging across terminal text selects text and copy writes selected text to system clipboard"
    requirement: TERM-02
    verification:
      - kind: automated
        ref: "cargo test -p webterm-terminal --test render_test test_mouse_reporting_and_coordinates"
        status: pass
  - id: R3
    description: "Mouse scroll wheel navigates scrollback history and keyboard input resets scrollback offset to zero"
    requirement: TERM-03
    verification:
      - kind: automated
        ref: "cargo test -p webterm-terminal --test terminal_state_test test_scrollback_accumulation_and_scrolling"
        status: pass
  - id: R4
    description: "Window and layout resizing recalculates cols/rows and resizes the terminal grid"
    requirement: TERM-04
    verification:
      - kind: automated
        ref: "cargo test -p webterm-terminal --test terminal_state_test test_grid_resize"
        status: pass
---

# Plan 21-03: GPUI Terminal Renderer, Interactive View & Shell Embedding Summary

Plan 21-03 completed Phase 21's interactive rendering and UI deliverables, providing native GPUI terminal rendering, keyboard translation, mouse selection, scrollback viewport navigation, and application shell embedding.

## Deliverables

1. **Terminal Renderer (`render.rs`)**:
   - `TerminalRenderer` measures font metrics using `'M'` character and calculates line height with custom multiplier.
   - `layout_row`: Coalesces adjacent background cells with identical color into single `BackgroundRect`s. Batches styled characters into `BatchedTextRun`s.
   - `paint`: Paints default background, coalesced non-default background quads, selection highlight quads (`palette.selection`), batched text runs, and cursor (block, beam, underline, hollow block).

2. **Input Translation (`input.rs`)**:
   - `keystroke_to_bytes`: Converts GPUI `Keystroke` into ANSI escape sequences (Enter `\r`, Backspace `\x7f`, Tab `\t`, Shift+Tab `\x1b[Z`, Normal vs App Cursor arrow keys, modified arrows `\x1b[1;{mod}A`, Ctrl+A..Z, F1..F12, Home/End/PageUp/PageDown).

3. **Mouse Handling (`mouse.rs`)**:
   - `pixel_to_cell`: Transforms pixel coordinates to terminal grid lines/columns with boundary clamping.
   - `selection_type_from_clicks`: Maps 1 click -> Simple, 2 clicks -> Semantic word, 3+ clicks -> Lines.
   - `mouse_button_report`: Generates SGR 1006 mouse button reports when tracking is enabled.
   - `scroll_report`: Bridges scroll events to mouse wheel reports or alternate screen arrow keys.

4. **TerminalView (`view.rs`)**:
   - Struct implementing `gpui::Render` and `Focusable`.
   - Canvas-based rendering with automatic bounds tracking and grid resize detection.
   - Mouse handlers for drag selection (`on_mouse_down`, `on_mouse_move`, `on_mouse_up`).
   - Scroll wheel handler navigating scrollback history (`on_scroll`).
   - Keyboard listener resetting scrollback to bottom and dispatching translated bytes (`on_key_down`).
   - Native GPUI clipboard integration: copy (`copy_selection`, Ctrl+Shift+C / Cmd+C) and paste (`paste_clipboard`, Ctrl+Shift+V / Cmd+V).

5. **Desktop Shell Integration (`nav.rs` & `app_state.rs`)**:
   - `webterm` depends on `webterm-terminal`.
   - `AppState` manages `active_terminal: Option<Entity<TerminalView>>`.
   - `nav.rs` embeds `TerminalView` into the Hosts / Sessions main pane.

6. **Test Suites**:
   - `tests/render_test.rs`: 6 integration tests passing.
   - `tests/terminal_state_test.rs`: 7 integration tests passing.
   - Full workspace tests: 39 tests passing across all crates.
   - Zero clippy warnings under `cargo clippy --workspace -- -D warnings`.
