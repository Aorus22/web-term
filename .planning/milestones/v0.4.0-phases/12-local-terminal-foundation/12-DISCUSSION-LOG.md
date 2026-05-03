# Phase 12: Local Terminal Foundation — Discussion Log

## 2026-05-03 — Initial Alignment
- **Topic:** Architecture for Local Terminal.
- **Discussion:**
    - Current `ManagedSession` is tightly coupled to `golang.org/x/crypto/ssh`.
    - Agreed to refactor `ManagedSession` to be more generic.
    - Decided to use `github.com/creack/pty` for local PTY support.
    - Discussed I/O forwarding: Local PTY provides a single `*os.File` for both read/write, unlike SSH which has separate pipes. The logic in `proxy.go` needs to handle this branch.
    - UI: "Local Terminal" should be prominent in `NewTabView`.
- **Decisions:**
    - Add `SessionType` enum.
    - Make `SSHClient` and `SSHSession` optional in `ManagedSession`.
    - Use `ConnectionID: "local"` as the trigger for local sessions.
- **Next Steps:**
    - Finalize implementation plans for Backend and Frontend.
