# Phase 3: SSH Terminal - Context

**Gathered:** 2026-04-27
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can SSH to any server from the browser with a smooth, reliable terminal experience — this is the product's core value. This phase productionizes the spike's terminal component into the app shell, adds password authentication with keyboard-interactive support, handles connection lifecycle (connect, disconnect, reconnect), and integrates saved connections from Phase 2.

**Requirements covered:** TERM-01, TERM-02, TERM-03, TERM-04, TERM-05, UI-04

**In scope:**
- Terminal component replacing spike code, integrated into app shell
- WebSocket SSH proxy enhanced from spike (already handles connect/resize/binary I/O)
- Password authentication flow: saved connections + quick-connect
- Keyboard-interactive SSH auth rendered in terminal
- Terminal auto-resize (already validated in spike)
- Copy/paste via native browser selection (already works in @wterm/react)
- WebSocket disconnection/reconnection UX
- Tab bar showing active sessions (basic — no multi-tab management, that's Phase 4)

**Out of scope:**
- Multi-tab session management (Phase 4)
- SSH key authentication (v2)
- Dark/light theme toggle (Phase 4)
- Keyboard shortcuts like Ctrl+T/W (Phase 4)

</domain>

<decisions>
## Implementation Decisions

### Connection Flow & Password Handling
- **D-01:** Saved connections with stored password → connect immediately when clicked, no confirmation step. Terminal pane shows "Connecting..." state that transitions to live terminal.
- **D-02:** Saved connections without stored password → show inline password prompt in the terminal pane area (not modal, not sheet). User types password → connects.
- **D-03:** Quick-connect (user@host:port) → hits Enter → inline password prompt appears in terminal pane → connects. Password is ephemeral, never stored unless user saves later.
- **D-04:** Keyboard-interactive SSH prompts (TERM-03) render directly in the terminal — the SSH server's own prompts appear as typed input, masked with asterisks. No special frontend handling needed; backend pipes the challenge through.
- **D-05:** Connection errors (wrong password, host unreachable, auth failed) display in the terminal pane itself with a "Retry" button. Terminal area becomes the feedback zone.
- **D-06:** After quick-connect session ends or user disconnects, show a small banner: "Save this connection?" with Save/Dismiss buttons. Non-intrusive.
- **D-07:** Decrypted password lives in backend memory only for the WebSocket session duration. When session ends, password is gone. Reconnection requires re-fetching from encrypted storage or re-prompting user.

### Session Lifecycle & Reconnection (agent's discretion)
- Auto-reconnect vs. manual reconnect prompt on WebSocket disconnect (UI-04)
- Reconnection overlay design and retry behavior
- Session state persistence across reconnection attempts
- Connection timeout and cancellation UX

### Terminal Pane Layout & Interaction (agent's discretion)
- How tab bar shows active sessions (basic tab per session)
- Disconnect button placement in terminal header
- Visual treatment of connected vs. connecting vs. disconnected vs. error states
- Terminal pane sizing within the main area
- How the "No active sessions" empty state transitions when connecting

### Agent's Discretion
- Exact WebSocket message protocol extensions beyond spike's connect/resize/binary
- Frontend component architecture (hooks, context, component breakdown)
- Backend SSH session management (goroutine lifecycle, cleanup, concurrent sessions)
- Terminal theme and color configuration
- Error message wording
- Animation/transition timing for connecting states
- How to handle multiple simultaneous connections at backend level

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase 1 Spike Artifacts (validated patterns)
- `.planning/phases/01-wterm-spike/01-SPIKE-RESULT.md` — @wterm/react validation results, DECISION: PROCEED with all 8 tests passed
- `.planning/phases/01-wterm-spike/01-01-SUMMARY.md` — Spike implementation details, WebSocket protocol, established patterns
- `.planning/phases/01-wterm-spike/01-REVIEW.md` — 4 critical + 4 warning security findings from spike

### Phase 2 Context (connection management decisions)
- `.planning/phases/02-connection-mgmt/02-CONTEXT.md` — Sidebar layout, connection CRUD, quick-connect bar, data model, API, frontend architecture decisions

### Project Configuration
- `.planning/PROJECT.md` — Tech stack, constraints, key decisions
- `.planning/REQUIREMENTS.md` — TERM-01 through TERM-05, UI-04 requirements
- `.planning/ROADMAP.md` — Phase 3 success criteria and dependency on Phase 2

### Existing Codebase (must-read for integration)
- `be/internal/ssh/proxy.go` — Current WebSocket SSH proxy: ConnectMessage, ResizeMessage, bidirectional I/O, origin check, SSRF protection
- `be/internal/api/routes.go` — Route setup: `/api/*` for CRUD, `/ws` for WebSocket
- `be/cmd/server/main.go` — Server entry point, config, graceful shutdown
- `fe/src/features/terminal/terminal-spike.tsx` — Spike terminal component: wterm/react usage, WebSocket lifecycle, connect/resize/data handlers
- `fe/src/App.tsx` — App shell: sidebar, tab bar placeholder `{/* Tabs will go here in Phase 3 */}`, main content area
- `fe/src/stores/app-store.ts` — Zustand store: sidebarOpen, editingConnection, selectedTags (no session state yet)
- `fe/src/lib/api.ts` — Connection API client: CRUD, export/import
- `fe/src/features/connections/components/QuickConnect.tsx` — Quick-connect bar in sidebar

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`be/internal/ssh/proxy.go`**: Full WebSocket SSH proxy already works — handles connect message, SSH dial, PTY request, bidirectional I/O, resize. Needs enhancement for password-from-storage flow and reconnection, but core is production-ready.
- **`fe/src/features/terminal/terminal-spike.tsx`**: Proven @wterm/react integration — `useTerminal()` hook, `onData`/`onResize` callbacks, binary frame handling. Should be refactored into reusable hook + component, not used as-is.
- **`fe/src/stores/app-store.ts`**: Zustand store already set up. Extend with session state (active sessions, active tab).
- **`fe/src/lib/api.ts`**: Connection API client already fetches saved connections with encrypted passwords. Backend decrypts and returns password in GET response (for connecting).
- **shadcn/ui components**: Dialog, Sheet, Button, Input, ScrollArea, Badge all available in `fe/src/components/ui/`.

### Established Patterns
- **WebSocket protocol**: JSON text frames for control (connect, resize, connected, error), binary frames for SSH data. Established in spike, must continue.
- **Feature-based structure**: `fe/src/features/terminal/`, `fe/src/features/connections/`. New terminal components go in terminal feature.
- **Zustand for UI state, TanStack Query for server state**: Continue this pattern. Sessions are client-side state (Zustand).
- **Go backend layout**: `cmd/server/main.go` entry, `internal/api/` handlers, `internal/ssh/` SSH proxy, `internal/config/` config. Continue this layout.

### Integration Points
- **Sidebar → Terminal**: Clicking a connection in `ConnectionList` or submitting quick-connect in `QuickConnect` needs to trigger terminal session creation
- **Tab bar in App.tsx**: Currently placeholder — needs to render active session tabs
- **App store**: Needs session state (open sessions, active session ID, session metadata)
- **`/ws` WebSocket endpoint**: Already handles SSH proxy. May need session ID parameter or connection metadata.
- **Connection API GET response**: Backend needs to return decrypted password for saved connections so frontend can send it in WebSocket connect message (or backend fetches it server-side)

</code_context>

<specifics>
## Specific Ideas

- Connection flow prioritizes immediacy — saved connections with passwords connect instantly on click, no intermediate steps
- Terminal pane is the center of the experience — errors, prompts, connecting states all happen there, not in separate modals
- Quick-connect sessions are ephemeral by default but easy to save with a post-disconnect nudge

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 3-SSH Terminal*
*Context gathered: 2026-04-27*
