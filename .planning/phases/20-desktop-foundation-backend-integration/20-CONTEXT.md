# Phase 20: Desktop Foundation & Backend Integration - Context

**Gathered:** 2026-09-04
**Status:** Ready for execution
**Mode:** Documented architecture & research synthesis

<domain>
## Phase Boundary

Phase 20 delivers the cross-platform desktop shell (GPUI) with an integrated Go backend child process:
- Desktop cargo workspace with exact dependency pins (gpui =0.2.2, gpui-component =0.6.0, tokio =1.53, reqwest =0.12 with rustls-tls).
- Backend process supervisor: child spawn with ephemeral port (:0), stdout handshake capture (BACKEND_PORT), readiness polling, and graceful termination.
- Desktop settings store (paths, JSON persistence, encryption key custody with strict file permissions).
- Headless backend-client crate skeleton for REST communication.
- Main GPUI window with application lifecycle (Starting -> Ready -> shell with 4 navigation entries, or Failed with stderr tail and retry).
- Window geometry persistence and initial settings view.

</domain>

<decisions>
## Implementation Decisions

### Process Supervisor Seam
The desktop client launches the existing Go backend as a child process using `tokio::process::Command`. The child binds to `:0` and prints `BACKEND_PORT:<port>` to stdout. The supervisor captures this port and verifies readiness against `/api/settings`.

### Encryption Key Management
The supervisor generates and manages a persistent 32-byte hex encryption key in the desktop settings store, passed to the backend via `WEBTERM_ENCRYPTION_KEY` env variable so credentials remain decryptable across restarts.

### Headless Crate Architecture
`webterm-supervisor`, `webterm-settings`, and `webterm-backend-client` are decoupled from GPUI to enable headless CI testing across Windows and Linux.

</decisions>

<code_context>
## Existing Code Insights

- `be/cmd/server/main.go` emits `BACKEND_PORT:%d\n` upon binding.
- `be/internal/config/config.go` uses `WEBTERM_ENCRYPTION_KEY`, `WEBTERM_PORT`, `WEBTERM_DB_PATH`.
- Backend has no `/health` route; `/api/settings` acts as readiness probe.
- Cross-platform CI matrix defined for Windows (DirectX) and Linux (X11/Wayland).

</code_context>

<specifics>
## Specific Ideas

- Plans 20-01 through 20-04 already generated and reviewed.
- Plan 20-01: Workspace scaffold, CI workflow, supervisor crate & tests.
- Plan 20-02: Settings store and backend-client crate.
- Plan 20-03: GPUI tracer slice (window, supervisor integration, app state, views).
- Plan 20-04: Window geometry persistence & settings view.

</specifics>

<deferred>
## Deferred Ideas

- Loopback-only enforcement (0.0.0.0 bind) addressed in Phase 27 (QA-03).
- Terminal engine integration deferred to Phase 21.

</deferred>
