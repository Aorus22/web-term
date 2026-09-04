# Architecture Decision Record: Windows Local PTY Implementation (ConPTY)

**Status:** Accepted
**Date:** 2026-09-05
**Deciders:** Core Engineering Team
**Requirement:** `TERM-05` (User can open a local terminal tab (shell on the desktop host), first-class in New Tab)

---

## Context and Problem Statement
To support local terminal tabs in the WebTerm desktop GPUI client on Windows and Linux, the application requires a pseudo-terminal (PTY) interface. On Linux, POSIX pseudo-terminals (`pty`) are standard. On Windows, Windows 10 build 1809+ provides the ConPTY (`CreatePseudoConsole`) subsystem for pseudo-terminal emulation.

We must decide whether to:
1. Embed PTY management directly inside the Rust GPUI client using `portable-pty` or Win32 bindings.
2. Leverage the existing Go backend PTY management subsystem (`github.com/UserExistsError/conpty` on Windows and `github.com/creack/pty` on Unix) accessed over the unified loopback WebSocket protocol.

---

## Decision Drivers
1. **Pipeline Uniformity:** Terminal rendering in GPUI uses `alacritty_terminal`. Having SSH sessions and local sessions share the exact same WebSocket stream format, control framing, and event loop eliminates branching logic in the frontend.
2. **Session Persistence & Re-attachment (`TERM-07`):** The Go backend session manager maintains ring buffers and manages active session lifecycles across UI restarts. Local sessions hosted in the backend can survive client crashes/restarts and be re-attached just like remote SSH sessions.
3. **Build & Linking Simplicity:** Compiling native Windows ConPTY or POSIX PTY bindings into the Rust GPUI client introduces platform-specific C-runtime linkage requirements and crates.io dependencies that may conflict with GPUI.
4. **Latency & Performance:** Local loopback IPC over WebSocket on Windows/Linux has a transmission latency of <0.2ms, which is completely negligible for terminal interaction.
5. **Existing Investment & Code Parity:** The Go backend already contains a robust, tested implementation of `spawn_windows.go` (ConPTY) and `spawn_unix.go` (creack/pty).

---

## Considered Options

### Option 1: Rust-Side Embedded PTY (`portable-pty`)
Implement local terminal spawning directly in Rust using the `portable-pty` crate.
- **Advantages:**
  - Operates in-process without requiring loopback networking.
- **Disadvantages:**
  - Creates a split architecture: two separate terminal pipelines (remote SSH via WebSocket vs local terminal via Rust channels).
  - Cannot survive GPUI client restarts (breaks `TERM-07` parity for local tabs).
  - High risk of dependency / build incompatibilities on Windows MSVC.
  - Requires duplicating PTY resize, scrollback buffer, and signal handling in Rust.

### Option 2: Go Backend ConPTY Extension (Chosen)
Utilize the Go backend's `spawnLocalPTY` implementation (`github.com/UserExistsError/conpty` on Windows, `github.com/creack/pty` on Unix) bridged via loopback WebSocket (`WsConnectRequest::for_local`).
- **Advantages:**
  - 100% unified terminal architecture across all session types (SSH and Local).
  - Preserves session durability and re-attachment (`TERM-07`) for local shells.
  - Zero added C-runtime or Win32 linkage dependencies in the Rust client.
  - Already implemented, tested, and working in the Go backend.
  - Cross-platform parity by design (POSIX PTY on Linux, ConPTY on Windows).
- **Disadvantages:**
  - Depends on the local backend process being alive (which is already guaranteed by the Phase 20 supervisor lifecycle).

---

## Decision Outcome
**Chosen Option: Option 2 (Go Backend ConPTY Extension).**

### Implementation Details
- **Windows PTY:** Backend uses `github.com/UserExistsError/conpty` to spawn `powershell.exe` (with fallback to `cmd.exe` or `COMSPEC`).
- **Unix PTY:** Backend uses `github.com/creack/pty` to spawn `$SHELL` (with fallback to `/bin/bash`).
- **Protocol:** Desktop client sends `WsConnectRequest::for_local(cols, rows)` (or with custom `cwd`).
- **Resize:** Dynamic window resize propagates via JSON control frame `{"type":"resize","cols":...,"rows":...}` to `cpty.Resize` on Windows and `pty.Setsize` on Unix.
- **Verification:** An automated smoke test exercises this full stack against the built backend fixture in CI on both Windows (`windows-latest`) and Linux (`ubuntu-latest`).
