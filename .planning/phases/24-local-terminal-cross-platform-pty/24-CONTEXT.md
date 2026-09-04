# Phase 24: Local Terminal (Cross-Platform PTY) - Context

**Gathered:** 2026-09-05
**Status:** Ready for planning
**Mode:** Smart discuss (batch proposals accepted)

<domain>
## Phase Boundary

Phase 24 establishes the local terminal as a first-class New Tab option on both Linux (POSIX PTY) and Windows (ConPTY), explicitly resolving and documenting the Windows ConPTY architectural decision, ensuring robust process lifecycle, resize propagation, and automated CI smoke testing.

- Primary Requirement: `TERM-05` (User can open a local terminal tab (shell on the desktop host), first-class in New Tab)
- Scope includes:
  - Explicit ConPTY architectural decision record (`CONPTY-DECISION.md`) comparing Go backend ConPTY vs Rust `portable-pty`.
  - Backend and frontend integration verification for local shell spawning, default shell detection (`$SHELL`, `COMSPEC`, PowerShell/cmd.exe), and initial working directory (CWD) inheritance.
  - New Tab modal ordering and UI parity placing Local Shell as the first-class default launcher option.
  - Process exit detection and terminal status banner indicating exit code with options to close tab or restart session.
  - Resize event propagation over WebSocket control frames to backend PTY.
  - Windows CI smoke test and integration test suite exercising local terminal spawn and I/O.

</domain>

<decisions>
## Implementation Decisions

### ConPTY Architecture & Implementation Path
- **ConPTY Architecture Decision:** Standardize on Go backend ConPTY path (`github.com/UserExistsError/conpty` on Windows and `github.com/creack/pty` on Unix). The backend already has production-tested implementations of both, handles WebSocket I/O multiplexing, and avoids introducing duplicate PTY implementations or C-runtime linking conflicts (such as `portable-pty` MSVC dependencies) into the Rust GPUI frontend.
- **Shell Spawning & Detection Defaults:** Unix/Linux reads `$SHELL` (falling back to `/bin/bash` or `/bin/sh`); Windows reads `COMSPEC` / PowerShell (`powershell.exe` falling back to `cmd.exe`).
- **Initial Working Directory (CWD):** Inherits working directory from desktop app launch context, falling back to user home directory (`~` / `%USERPROFILE%`).

### New Tab UX & Local Terminal Lifecycle
- **New Tab Modal Hierarchy:** "💻 Local Shell" is placed prominently at the top of the New Tab modal as the default first-class action, positioned above Quick SSH and Saved Hosts.
- **Process Exit & Termination Behavior:** When the local shell process exits (e.g. user types `exit`), display a status banner in the terminal indicating `[Process exited with code X]` with "Close Tab" and "Restart Session" buttons, matching SSH drop UX.
- **Terminal Resize & Signal Handling:** Propagate terminal resize events immediately over WebSocket control frames (`{"type":"resize","cols":...,"rows":...}`) to resize backend ConPTY / POSIX PTY handles.

### Automated Testing & CI Verification Strategy
- **Windows Local Terminal Smoke Test:** Dedicated automated integration/smoke test in Rust (`tests/local_tab_test.rs` / `tests/local_smoke_test.rs`) and Go backend test exercising backend ConPTY spawn, echoing a test token, asserting output, and verifying clean shutdown.
- **Cross-Platform CI Matrix Coverage:** Ensure GitHub Actions CI workflow runs tests on both `windows-latest` and `ubuntu-latest` in headless/CI-friendly mode.

### the agent's Discretion
- Exact styling details of process exit banners in GPUI adhering to existing theme variables (`theme.status_error`, `theme.surface_panel`, etc.).
- Internal test timeout thresholds and echo framing payloads.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `desktop-gpui/crates/backend-client/src/terminal_ws.rs`: `WsConnectRequest::for_local(cols, rows)` handles local shell connection requests over WebSocket.
- `desktop-gpui/crates/webterm/src/views/new_tab_modal.rs`: New Tab modal UI presenting Local Shell and Quick SSH.
- `desktop-gpui/crates/webterm/src/session.rs`: `TerminalSessionManager` tab and session lifecycle management.
- `be/internal/ssh/spawn_windows.go`: Go backend Windows ConPTY spawning via `github.com/UserExistsError/conpty`.
- `be/internal/ssh/spawn_unix.go`: Go backend Unix POSIX PTY spawning via `github.com/creack/pty`.
- `be/internal/ssh/proxy.go`: WebSocket to PTY bridge and control message handling (`local` session type).

### Established Patterns
- Binary WebSocket frames for raw terminal stdin/stdout.
- JSON WebSocket frames for control signals (`resize`, `get-cwd`, `ready`, `disconnect`).
- Clean separation between GPUI view layer and async Tokio backend client channels.

### Integration Points
- `desktop-gpui/crates/webterm/src/views/new_tab_modal.rs` -> Action triggers local session creation.
- `desktop-gpui/crates/webterm/src/app_state.rs` -> `open_local_tab` coordinates session creation, WebSocket connection, and manager insertion.
- `.github/workflows/ci.yml` -> CI matrix verification for Windows and Linux.

</code_context>

<specifics>
## Specific Ideas

- Ensure explicit decision documentation is saved in `.planning/phases/24-local-terminal-cross-platform-pty/CONPTY-DECISION.md` documenting why Go backend ConPTY was chosen over Rust `portable-pty`.
- Verify that terminal resize is immediately reflected in backend PTY without perceptible delay.

</specifics>

<deferred>
## Deferred Ideas

- Local shell profile customization (custom shell binary path or arguments configured in settings) -> Phase 26 (Settings & Theme Polish).
- SFTP local filesystem pane browsing -> Phase 25 (SFTP Dual-Pane Manager).

</deferred>
