# Phase 24 Research: Local Terminal (Cross-Platform PTY)

## Executive Summary
Phase 24 validates and hardens the local terminal feature (`TERM-05`) as a first-class New Tab option across both target platforms: Linux (POSIX PTY) and Windows (ConPTY). The key architectural milestone is explicitly evaluating and deciding between Go-side backend ConPTY extension versus Rust-side `portable-pty` embedding, followed by New Tab ordering parity, process exit lifecycle handling, and an automated Windows CI smoke test.

---

## 1. ConPTY Architecture Evaluation

### Option A: Rust-Side Embedded PTY (`portable-pty`)
- **Approach:** Add `portable-pty = "0.8"` to `desktop-gpui/crates/terminal` or a new crate, spawning Windows ConPTY and Linux POSIX PTY directly within the desktop process.
- **Analysis:**
  - **Pros:** Bypasses local WebSocket hop for local shells; purely within the Rust process.
  - **Cons:**
    - High complexity: Adds heavy Win32/MSVC native bindings and PTY thread loops to the GPUI client.
    - Dual architecture: Splits terminal session handling into two disparate systems (remote SSH sessions over WebSocket vs local sessions over Rust channels).
    - Session Re-attach Parity Loss: Requirement `TERM-07` (session re-attachment across app restarts) is provided by the Go backend's session manager. An embedded Rust PTY cannot survive app restarts or reconnect after UI reloads.
    - Potential C-runtime linking conflicts with GPUI's rendering dependencies.

### Option B: Unified Go-Backend PTY Path (`github.com/UserExistsError/conpty` + `creack/pty`)
- **Approach:** Leverage the Go backend's existing cross-platform PTY engine (`spawn_windows.go` and `spawn_unix.go`) exposed over the loopback WebSocket protocol (`WsConnectRequest::for_local`).
- **Analysis:**
  - **Pros:**
    - Single unified terminal pipeline: All terminal sessions (SSH and Local) share identical GPUI rendering (`alacritty_terminal`), WebSocket multiplexing, keyboard input mapping, resize signaling, and reconnection UX.
    - Session durability & re-attach: Local sessions benefit from the Go backend's `RingBuffer` scrollback preservation and session re-attachment across UI restarts.
    - Zero additional Rust dependencies or MSVC linking headaches in `desktop-gpui`.
    - Production-tested: The Go backend already contains native ConPTY support on Windows and POSIX PTY on Linux.
    - Negligible overhead: Local loopback IPC latency is <0.2ms, virtually imperceptible compared to human keystroke intervals (>50ms).
  - **Cons:** Requires the backend supervisor to be running (which is already a core requirement of the desktop architecture established in Phase 20).

### Architecture Decision
**Standardize on Option B (Go-Backend PTY Path).**
The explicit Decision Record will be documented in `CONPTY-DECISION.md`.

---

## 2. Codebase Scout & Integration Points

### Go Backend PTY Subsystem
1. **Windows PTY (`be/internal/ssh/spawn_windows.go`):**
   - Uses `github.com/UserExistsError/conpty`.
   - Detects `powershell.exe` via `exec.LookPath`, falling back to `COMSPEC` or `cmd.exe`.
   - Supports `connectMsg.Cwd`, falling back to `os.UserHomeDir()`.
   - Dimensions set via `conpty.ConPtyDimensions(connectMsg.Cols, connectMsg.Rows)`.
   - Resize handled via `conpty.ConPty.Resize(cols, rows)`.
2. **Unix PTY (`be/internal/ssh/spawn_unix.go`):**
   - Uses `github.com/creack/pty`.
   - Reads `$SHELL`, falling back to `/bin/bash`.
   - Dimensions set via `pty.Setsize`.
   - Resize handled via `pty.Setsize`.
3. **Session & WebSocket Multiplexing (`be/internal/ssh/proxy.go`):**
   - Handles `connectMsg.ConnectionID == "local"` or `connectMsg.SessionType == "local"`.
   - Manages session lifecycle in `GlobalSessionManager`.
   - Bridges binary input/output and control messages (`resize`, `ready`, `get-cwd`, `disconnect`).

### Desktop GPUI Client
1. **Backend Client (`desktop-gpui/crates/backend-client/src/terminal_ws.rs`):**
   - `WsConnectRequest::for_local(cols, rows)` constructs the connection payload.
   - We should add `WsConnectRequest::for_local_with_cwd(cols, rows, cwd: Option<String>)` to preserve/pass working directory.
2. **Tab Lifecycle & State (`desktop-gpui/crates/webterm/src/app_state.rs`):**
   - `open_local_tab`: creates tab, connects WebSocket client, attaches handle to session manager.
3. **New Tab Launcher (`desktop-gpui/crates/webterm/src/views/new_tab_modal.rs`):**
   - Hierarchy: "💻 Local Shell" is the primary top-level action card.

---

## 3. Windows CI Smoke Test Plan
1. **Backend Fixture Availability:**
   - `.github/workflows/desktop-ci.yml` builds `desktop-gpui/test-support/backend.exe` on Windows and `desktop-gpui/test-support/backend` on Linux prior to running `cargo test --workspace`.
   - Environment variable `TEST_BACKEND_PATH` points to the built fixture.
2. **Automated Integration Test (`desktop-gpui/crates/webterm/tests/local_smoke_test.rs`):**
   - Spawns real backend fixture via `webterm_supervisor::Supervisor`.
   - Connects to local terminal via `TerminalWsClient::connect`.
   - Sends test echo command (`echo CONPTY_TEST_OK\r\n`).
   - Asserts stdout contains `CONPTY_TEST_OK`.
   - Tests PTY resize command.
   - Cleans up cleanly.
   - Runs in CI on both `windows-latest` (exercising ConPTY) and `ubuntu-latest` (exercising POSIX PTY).
