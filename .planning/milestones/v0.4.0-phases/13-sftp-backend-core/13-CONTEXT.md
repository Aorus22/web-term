# Phase 13: SFTP Backend Core - Context

**Gathered:** 2026-05-03
**Status:** Ready for planning

<domain>
## Phase Boundary

This phase delivers the core backend infrastructure for SFTP. It includes a unified `FileSystem` interface in Go with implementations for `Local` and `SFTP`, and REST endpoints for directory listing and file operations (upload, download, delete, rename). Authentication is handled by establishing standalone SFTP connections using existing credentials from the database.

</domain>

<decisions>
## Implementation Decisions

### SFTP Connection Strategy
- **D-01:** Use **standalone** SSH connections for SFTP operations. 
- **D-02:** SFTP connections will have an independent lifecycle from terminal connections to prevent cross-tab interference.
- **D-03:** Credentials (password/keys) will be retrieved from the database using the provided `ConnectionID`.

### Filesystem Access Policy
- **D-04:** Provide **full filesystem access** (restricted only by the OS permissions of the connecting user). 
- **D-05:** No artificial jailing to specific directories; users can navigate the entire structure they have permission to see.

### Data Transfer Pattern
- **D-06:** Implement **streaming** for file uploads and downloads.
- **D-07:** Use `io.Pipe` or similar streaming constructs to handle large files without high memory overhead on the backend.
- **D-08:** Uploads will be handled via streamed request bodies (or multipart streaming) rather than loading full files into memory.

### Cross-Platform Support
- **D-09:** The `FileSystem` interface and its implementations must be **cross-platform**.
- **D-10:** Use `path/filepath` for path manipulation to ensure compatibility with Windows, Linux, and macOS.
- **D-11:** Avoid Linux-specific syscalls or `/proc` dependencies in the core SFTP/Local logic.

### Claude's Discretion
- Choice of the specific streaming library/construct (e.g., standard `io` package vs specialized stream handlers).
- The internal structure of the `FileSystem` interface methods.
- Error handling strategies for specific filesystem edge cases (e.g., permission denied, file busy).

</decisions>

<specifics>
## Specific Ideas

- "Standalone connections for SFTP so it doesn't break if I close a terminal tab."
- "Streaming is a must for large files."
- "Full access like a real sysadmin tool, no jailing."

</specifics>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Backend Infrastructure
- `be/internal/ssh/session_manager.go` — Existing session management logic to adapt/reference.
- `be/internal/api/routes.go` — Current routing pattern to follow for new SFTP endpoints.
- `pkg/sftp` (External) — Primary library for SFTP implementation.

### Milestone Requirements
- `.planning/milestones/v0.4.0-phases/v0.4.0-REQUIREMENTS.md` — High-level requirements for the SFTP milestone.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ConnectionHandler` in `be/internal/api/connections.go`: Provides logic for retrieving connection details from DB.
- `ManagedSession` in `be/internal/ssh/session_manager.go`: Patterns for managing lifecycle and I/O.

### Established Patterns
- **Go 1.22+ Routing:** Use the `METHOD /path` pattern in `routes.go`.
- **JSON API:** Consistent error responses and success payloads.

### Integration Points
- `be/internal/api/routes.go`: New endpoints need to be registered here.
- `be/internal/db/models.go`: Ensure SFTP logic respects existing `Connection` and `SSHKey` models.

</code_context>

<deferred>
## Deferred Ideas

- Dual-pane Frontend UI — Handled in Phase 14.
- Inter-host transfers (FXP or Proxying) — Handled in Phase 15.
- Drag-and-drop UI logic — Handled in Phase 15.

</deferred>

---

*Phase: 13-sftp-backend-core*
*Context gathered: 2026-05-03*
