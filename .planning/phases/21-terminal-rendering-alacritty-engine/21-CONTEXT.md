# Phase 21: Terminal Rendering (Alacritty Engine) - Context

**Gathered:** 2026-09-04
**Status:** Ready for planning
**Mode:** Autonomous (domain analysis + architectural decisions)

<domain>
## Phase Boundary

Render a native terminal grid in GPUI through the `alacritty_terminal` engine, achieving parity with the web client on terminal rendering (colors, truecolor, bold/italic/underline, cursor, selection, scrollback, and resize synchronization).

Phase 21 establishes:
- The `webterm-terminal` crate behind a clean boundary
- Timeboxed evaluation / spike of `gpui-terminal` with explicit go/no-go
- `alacritty_terminal` state layer with PTY input/output parsing
- GPUI canvas/text renderer with glyph batching, cursor drawing, and selection highlights
- Mouse selection, clipboard copy/paste, and scrollback wheel navigation
- Resize synchronization (cols/rows calculation from character metrics)
- Local mock/PTY byte stream test harness verifying TERM-01 through TERM-04

Connecting real SSH sessions and multi-tab orchestration is reserved for Phase 22.

</domain>

<decisions>
## Implementation Decisions

### D-01: Engine Evaluation Strategy (gpui-terminal Spike)
- Plan 21-01 will conduct a timeboxed spike on `gpui-terminal` to evaluate dependency compatibility with `gpui-pre = "=0.3.3"`.
- If `gpui-terminal` requires an incompatible GPUI version or lacks critical features, the decision is immediate NO-GO: proceed with a clean in-tree GPUI terminal element wrapping `alacritty_terminal` directly (following the Zed architecture).
- Record the verdict in `SPIKE-gpui-terminal.md`.

### D-02: Crate Structure & Modularity
- Introduce `desktop-gpui/crates/terminal` (`webterm-terminal`) in the Cargo workspace.
- The crate isolates terminal parsing, grid state, text selection, and GPUI rendering elements from the application shell.
- Exposes a high-level `TerminalView` entity for GPUI embedding.

### D-03: Rendering & Font Metrics
- Monospace font rendering uses bundled `JetBrains Mono` registered in Phase 20.
- Cell dimensions (`cell_width`, `cell_height`) derived from `cx.text_system()` font metrics.
- Background quads and text runs batched per line to maintain 60+ FPS rendering performance during heavy terminal output.

### D-04: Color & Palette System
- Full support for 24-bit TrueColor (RGB ANSI escapes `\x1b[38;2;r;g;bm`).
- Default palette aligns with WebTerm's Dark and Light themes (Phase 20 `Theme`).
- Terminal cursor rendered with block, beam, or underline according to terminal mode, with focus-aware opacity/blinking.

### D-05: Selection & Clipboard
- Track mouse click & drag across terminal cells (start cell -> end cell).
- Copy triggered via standard shortcut or right-click, extracting text from `alacritty_terminal` grid into GPUI clipboard.
- Paste reads clipboard content and feeds raw UTF-8 into the terminal input channel.

### D-06: Scrollback Navigation
- Mouse scroll wheel increments/decrements scrollback display offset.
- User typing automatically jumps scroll offset back to 0 (bottom of screen).

</decisions>

<code_context>
## Existing Code Insights

- `desktop-gpui/crates/webterm`: Main application crate with `AppState` and navigation shell.
- `desktop-gpui/crates/settings`: Holds `Theme` preference (`Dark` / `Light`).
- Monospace font: `desktop-gpui/crates/webterm/assets/fonts/JetBrainsMono-Regular.ttf` is bundled and loaded in `cx.text_system()`.
- Dependencies in `Cargo.toml`: `alacritty_terminal` can be added to the workspace.

</code_context>

<specifics>
## Specific Requirements

- **TERM-01**: User can work in SSH/local sessions rendered by the alacritty engine (vim, htop, tmux, curses apps, truecolor).
- **TERM-02**: User can select text with the mouse and copy it, and paste into the terminal.
- **TERM-03**: User can scroll through scrollback history.
- **TERM-04**: Terminal resizes with the window and keeps the PTY grid in sync.

</specifics>

<deferred>
## Deferred Ideas

- SSH WebSocket streaming and session re-attachment: Phase 22.
- Local Windows ConPTY spawning: Phase 24.
- Dual-pane SFTP: Phase 25.
- Custom user color palette configurations in settings: Phase 26.

</deferred>
