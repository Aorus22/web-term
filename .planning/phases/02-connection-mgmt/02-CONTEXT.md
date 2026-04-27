# Phase 2: Connection Management - Context

**Gathered:** 2026-04-27
**Status:** Ready for planning

<domain>
## Phase Boundary

Full vertical slice: Go backend + React frontend for saving, organizing, and managing SSH connections. Users can create, edit, delete, and organize SSH connections with tags, quick-connect without saving, and export/import connection data. Includes setting up shadcn/ui with Tailwind v4 for the Vercel-inspired UI and addressing all 4 critical security findings from Phase 1 review.

Covers requirements: CONN-01, CONN-02, CONN-03, CONN-04, CONN-05, CONN-06, CONN-07, UI-01.
Does NOT include: actual SSH terminal connections (Phase 3), multi-tab (Phase 4), auth system, SSH key management.

</domain>

<decisions>
## Implementation Decisions

### Sidebar & Layout
- **D-01:** Collapsible left sidebar (~260px) with toggle button at the left edge of the tab bar area
- **D-02:** Tab bar across the top of the main area — shows open session tabs + sidebar toggle icon at far left
- **D-03:** When sidebar is collapsed: only tabs + toggle icon visible
- **D-04:** Sidebar contains: quick-connect bar at top, flat scrollable connection list with tag filter row
- **D-05:** Connection items: click to connect, right-click or "..." button for context menu (Edit, Delete, Duplicate, Copy host)
- **D-06:** Tags displayed as small badges on each connection item; filter row at top of list to filter by tag

### Connection Form & Validation UX
- **D-07:** Create/edit form appears as a slide-over panel from the right side
- **D-08:** Form fields: Label, Host, Port (default 22), Username, Password (optional), Tags
- **D-09:** Tags input: multi-select or comma-separated (agent discretion on exact UX)
- **D-10:** Inline validation on blur — red border + error message below field, submit button disabled until valid
- **D-11:** Delete uses confirmation dialog showing connection name

### Quick-Connect Bar
- **D-12:** Lives at the top of the sidebar, always visible when sidebar is open
- **D-13:** Dual-purpose: type to search existing connections (autocomplete dropdown) OR type new user@host:port to quick-connect
- **D-14:** Quick-connect is ephemeral — does NOT auto-save. User can choose "Save this connection" from context menu afterward

### Data Model & API
- **D-15:** SQLite with single `connections` table; `tags` stored as JSON array in a TEXT column (e.g., `["prod","web"]`)
- **D-16:** REST JSON API: GET/POST/PUT/DELETE `/api/connections`
- **D-17:** Export: full dump — downloads all connections as JSON file
- **D-18:** Import: merges with existing connections (skip duplicates by host+port+user)
- **D-19:** Passwords stored encrypted using AES-256-GCM with server-side key (env var or config file)

### shadcn/ui Setup & Components
- **D-20:** Full shadcn/ui with Tailwind v4 — use shadcn CLI to init and add components
- **D-21:** Components to install: Button, Input, Dialog, Sheet, Dropdown, ScrollArea, Badge, Tooltip, Card, Command, Popover, Separator
- **D-22:** Sheet component used for the slide-over connection form panel
- **D-23:** Command component available for the quick-connect bar (search/autocomplete pattern)

### Security Fixes (from Phase 1 review)
- **D-24:** Fix all 4 critical issues in Phase 2: CR-01 (TLS), CR-02 (origin check), CR-03 (SSRF), CR-04 (jsonEscape)
- **D-25:** TLS handled by reverse proxy (nginx/caddy) — Go backend stays HTTP on localhost
- **D-26:** SSRF protection via configurable allowlist of allowed host patterns (e.g., `*.internal`, `10.0.0.*`)
- **D-27:** Host key verification: skip for v1 (keep InsecureIgnoreHostKey), document as known limitation
- **D-28:** WebSocket origin check — validate against expected origins
- **D-29:** Fix jsonEscape to handle all control characters (tabs, CR, etc.) or use json.Marshal

### Go Backend Structure
- **D-30:** Standard Go layout: `cmd/server/main.go` (entry), `internal/api/` (HTTP handlers), `internal/db/` (SQLite/GORM), `internal/ssh/` (SSH proxy), `internal/config/` (config + encryption key)
- **D-31:** Single HTTP server, route-based: `/api/*` for REST CRUD, `/ws` for WebSocket
- **D-32:** GORM for database operations with auto-migration from Go structs
- **D-33:** SQLite database file stored alongside the binary or in a configurable data directory

### React Frontend Architecture
- **D-34:** Feature-based folder structure: `src/features/connections/`, `src/features/terminal/`, `src/components/ui/` (shadcn)
- **D-35:** Zustand for client-side state management (sidebar open/close, active tab, UI state)
- **D-36:** TanStack Query for server state (API calls, caching, refetching, mutations for CRUD)
- **D-37:** No client-side router needed yet — single-page layout
- **D-38:** Custom `useWebSocket` hook for WebSocket lifecycle (connect, disconnect, reconnect)

### the agent's Discretion
- Exact visual styling within the Vercel-inspired aesthetic
- Component internal implementation details
- Error message wording
- Exact folder/file naming within the established structure
- Tags input UX (multi-select dropdown vs comma-separated input)
- Configuration file format for SSRF allowlist and encryption key

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase 1 Artifacts (context from spike)
- `.planning/phases/01-wterm-spike/01-SPIKE-RESULT.md` — @wterm/react validation results, DECISION: PROCEED
- `.planning/phases/01-wterm-spike/01-REVIEW.md` — 4 critical + 4 warning security findings to address
- `.planning/phases/01-wterm-spike/01-01-SUMMARY.md` — Spike implementation details, WebSocket protocol, established patterns

### Project Configuration
- `.planning/PROJECT.md` — Tech stack decisions (Go, React, shadcn, SQLite), constraints, key decisions
- `.planning/REQUIREMENTS.md` — CONN-01 through CONN-07, UI-01 requirements with traceability
- `.planning/ROADMAP.md` — Phase 2 definition, dependencies, success criteria

### Existing Codebase
- `be/main.go` — Current spike Go backend (single-file, to be restructured)
- `be/go.mod` — Go dependencies (gorilla/websocket, golang.org/x/crypto/ssh)
- `fe/src/App.tsx` — Current spike React frontend (single-file, to be restructured)
- `fe/package.json` — Frontend dependencies (@wterm/react, React 19, Vite 8)
- `fe/vite.config.ts` — Vite configuration with WebSocket proxy

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `be/main.go`: WebSocket upgrade handler, SSH dial+pipe logic, resize handling — can be refactored into `internal/ssh/` package
- `be/go.mod`: gorilla/websocket and golang.org/x/crypto/ssh dependencies already established
- `fe/src/App.tsx`: useTerminal hook pattern, WebSocket connect message protocol — can be refactored into custom hooks
- `fe/vite.config.ts`: WebSocket proxy config exists but is unused (frontend connects directly to :8080)

### Established Patterns
- WebSocket protocol: JSON text frames for control messages (connect, resize), binary frames for SSH I/O data
- `@wterm/react` v0.1.9 with useTerminal hook for imperative terminal control
- Frontend connects directly to Go backend on :8080 (bypasses Vite proxy)

### Integration Points
- New REST API endpoints (`/api/connections/*`) alongside existing `/ws` WebSocket endpoint
- SQLite database new — no existing DB layer
- shadcn/ui is new — no existing UI component library (spike used plain CSS)
- All spike CSS (`fe/src/App.css`, `fe/src/index.css`) will be replaced by Tailwind v4 + shadcn

</code_context>

<specifics>
## Specific Ideas

- Vercel-inspired UI: clean, minimal, dark-first aesthetic
- Sidebar layout similar to Vercel dashboard: left panel with navigation, main content area
- Tab bar across top for managing open sessions (foreshadows Phase 4 multi-tab)
- Quick-connect bar serves dual purpose: search + connect in one input

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 02-connection-mgmt*
*Context gathered: 2026-04-27*
