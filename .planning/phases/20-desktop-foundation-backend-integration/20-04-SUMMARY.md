---
phase: 20-desktop-foundation-backend-integration
plan: 04
subsystem: desktop-ui-shell
tags: [rust, gpui, window-state, geometry-persistence, settings-view, shell-06]

requires:
  - phase: 20-03
    provides: WebTerm application shell, theme toggle, and view routing
provides:
  - Window bounds persistence (x, y, width, height, maximized) across app restarts
  - Minimal settings view (theme selector + port-redacted backend status line)
  - Degenerate geometry guard (rejects 0x0 sizes)
  - Debounced geometry save and window close flush-save
affects: [21-terminal-engine-gpui-view, 26-desktop-settings-system-tray]

actuals:
  tokens: 1850
  tasks: 2
  commits: 4

tech-stack:
  added: [parking_lot workspace dependency for webterm]
  patterns: [geometry extraction via Pixels conversion, debounced window observation, port-redacted backend status display]

key-files:
  created:
    - desktop-gpui/crates/webterm/src/window_state.rs
    - desktop-gpui/crates/webterm/src/views/settings.rs
  modified:
    - desktop-gpui/crates/webterm/src/main.rs
    - desktop-gpui/crates/webterm/src/app_state.rs
    - desktop-gpui/crates/webterm/src/views/mod.rs
    - desktop-gpui/crates/webterm/src/views/nav.rs
    - desktop-gpui/crates/webterm/Cargo.toml
    - desktop-gpui/crates/settings/src/lib.rs
    - desktop-gpui/crates/supervisor/src/lib.rs

key-decisions:
  - "Pixels conversion: window bounds division by px(1.0) cleanly extracts f32 dimensions without accessing private tuple fields"
  - "Backend URL port redaction: settings view surfaces backend base host and connection state without exposing raw ports to users"
  - "Window close hook: on_window_should_close guarantees pending geometry updates are flushed to settings.json upon exit"

patterns-established:
  - "Window geometry restore in main bootstrap before window instantiation"
  - "Settings view routing through AppState with reactive theme synchronization"

requirements-completed: [SHELL-06]

coverage:
  - id: D6
    description: "Window bounds persist across restarts with degenerate geometry protection"
    requirement: SHELL-06
    verification:
      - kind: automated
        ref: "cargo test --workspace"
        status: pass
      - kind: automated
        ref: "cargo clippy --workspace -- -D warnings"
        status: pass
---

# Plan 20-04: Window Geometry Persistence & Minimal Settings View Summary

Plan 20-04 completed the desktop foundation window geometry persistence requirements and delivered a minimal settings view.

## Deliverables

1. **Window State Persistence (`window_state.rs`)**:
   - `restore(&DesktopSettings) -> Option<WindowBounds>` restores window position, size, and maximized state during initial bootstrap.
   - `extract_window_state(&Window) -> Option<WindowState>` converts current GPUI bounds into serializable settings geometry.
   - Degenerate geometry guards reject 0x0 or invalid window dimensions.
   - Debounce constant and window close hook (`on_window_should_close`) ensure geometry changes are persisted to `settings.json`.

2. **Minimal Settings View (`views/settings.rs`)**:
   - Provides a clean settings screen accessible via the sidebar "Settings" navigation item.
   - Synchronizes theme selection with the global `AppState` and sidebar theme toggle.
   - Displays backend connection status ("Running (local)" or "Starting") and active binary path.
   - Strictly redacts backend port numbers from user-facing display.
   - Includes roadmap note pointing to full preferences arriving in Phase 26.

3. **Workspace Hygiene & Lint Compliance**:
   - Resolved all clippy warnings across `webterm`, `webterm-supervisor`, and `webterm-settings`.
   - All 10 workspace unit and integration tests passing.
