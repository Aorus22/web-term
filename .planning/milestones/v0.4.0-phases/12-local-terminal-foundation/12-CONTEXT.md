# Phase 12: Local Terminal Foundation — Context

## Goal
Enable users to open a terminal session on the machine running the WebTerm backend. This expands the application beyond a pure SSH client.

## Scope
- **Backend:**
    - Integrate `github.com/creack/pty` for PTY management.
    - Refactor `ManagedSession` and `SessionManager` to support non-SSH session types.
    - Implement local PTY spawning and I/O forwarding.
    - Support terminal resizing for local sessions.
- **Frontend:**
    - Add "Local Terminal" option to `NewTabView.tsx`.
    - Update connection logic to handle the "local" pseudo-connection.

## Architecture Decisions
- **Session Type:** A new `SessionType` enum will distinguish between `ssh` and `local` sessions.
- **PTY Library:** `creack/pty` is chosen for its simplicity and wide adoption in Go for PTY handling.
- **I/O Loop:** The existing `masterForwarder` and `wsMessageLoop` will be updated to handle the single `*os.File` provided by the PTY (which acts as both reader and writer).
- **Shell Detection:** The backend will attempt to use the `SHELL` environment variable, falling back to `/bin/bash` or `sh`.

## Risks & Mitigations
- **Security:** Local terminal access allows full control over the backend host. *Mitigation:* In this single-user tool, this is the intended functionality. Future versions might add an `ENABLE_LOCAL_TERMINAL` config flag.
- **Session Cleanup:** Ensuring local child processes are terminated when a session is removed. *Mitigation:* `RemoveSession` will close the PTY file descriptor, which should send SIGHUP to the child process.

## Requirements Mapping
- REQ-PTY-INTEGRATION -> Backend PTY spawning.
- REQ-LOCAL-WS-HANDLER -> WebSocket support for local I/O.
- REQ-LOCAL-UI-OPTION -> "Local Terminal" button in UI.
