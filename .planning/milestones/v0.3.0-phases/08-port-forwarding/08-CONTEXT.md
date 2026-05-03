# Phase 8: Port Forwarding - Context

**Gathered:** 2026-04-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can create SSH local port forwarding tunnels that bind a remote host port to a local port, managed via sheet UI with port conflict detection. Rules are persisted in SQLite, active tunnel state is in-memory. New "Port Forwards" sidebar page provides full CRUD and activation control. Backend uses separate SSH connections per forward, decoupled from terminal sessions.

</domain>

<decisions>
## Implementation Decisions

### Data Model & Persistence
- **D-01:** Hybrid persistence — forwarding rules stored in SQLite (like connections/keys), but active tunnel status is in-memory only. On server restart, rules are listed but tunnels are inactive and must be manually re-activated.
- **D-02:** Minimal fields per rule: name, connection_id (FK to connections), local_port (int), remote_port (int). Remote host is always `localhost` (relative to the SSH server). No bind_address or protocol fields.
- **D-03:** New `port_forwards` table with GORM AutoMigrate. Follows same pattern as `connections` and `ssh_keys` tables (UUID primary key, timestamps).

### Backend Tunnel Architecture
- **D-04:** Separate SSH connection per active forward — each forward dials its own SSH connection to the target server. Decoupled from terminal sessions. Uses same auth credentials (password or key) from the saved connection.
- **D-05:** REST API at `/api/forwards` with endpoints: GET (list with status), POST (create rule), DELETE (remove rule), POST `/api/forwards/{id}/start` (activate tunnel), POST `/api/forwards/{id}/stop` (deactivate tunnel). Follows existing API pattern.
- **D-06:** Tunnel implemented via `net.Listen` on local_port + `ssh.Client.Dial("tcp", "localhost:"+remote_port)` for each accepted connection. One goroutine per incoming connection through the tunnel. Standard Go SSH forwarding pattern.

### Forward UI Placement
- **D-07:** New sidebar page "Port Forwards" as third tab (Hosts, SSH Keys, Port Forwards). Uses same sidebar navigation pattern — icon + text label. `sidebarPage` state extended to `'forwards'`.
- **D-08:** Rules displayed as card list with toggle switch per rule. Each card shows: name, connection label, `localhost:local_port → remote_port`, on/off toggle, kebab menu (edit/delete). Active cards highlighted similar to active host cards.
- **D-09:** Create/edit via right-side Sheet form (same pattern as ConnectionForm and SSHKeyUploadSheet). Form fields: name input, connection dropdown (listing all saved connections), local_port number input, remote_port number input.

### Port Conflict & Lifecycle
- **D-10:** Port conflict detection via backend `net.Listen` attempt — when starting a forward, backend tries to bind the port. If EADDRINUSE, returns error to frontend which displays toast. Catches both other forwards AND system services using the port.
- **D-11:** Forwards keep alive independently — closing a terminal tab does NOT affect active forwards. Forward uses its own SSH connection, stays up until explicitly stopped.
- **D-12:** Frontend fetches forward status via GET `/api/forwards` on page load (each rule includes `active: true/false` + error message if any). Poll on page load is sufficient — no real-time WebSocket status stream needed.

### the agent's Discretion
- PortForward GORM model struct details and constraints
- Auto-suggest next available local port in create form
- Tunnel goroutine lifecycle management and cleanup on stop/error
- Exact error message wording for port conflicts and tunnel failures
- Card styling details (shadows, borders, active state)
- FAB + button placement on Port Forwards page
- Empty state design (no rules yet)
- Whether to show connection host details on forward cards
- Edit behavior (can edit inactive rules only, or also active?)
- Local port range validation (ephemeral port range? well-known ports?)

</decisions>

<specifics>
## Specific Ideas

- "hybrid aja. konfigurasi ada di sqlite. tapi ada status on / off nya yang itu in memory. jadi ketika server crash or something ya harus on off kan lagi" — rules persist, tunnels don't
- Remote host always localhost — covers the common case of forwarding services running on the remote machine
- Separate SSH connections for forwards — clean separation from terminal, forward survives tab close

</specifics>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/ROADMAP.md` § Phase 8 — Port Forwarding goal and success criteria

### Prior Phase Context
- `.planning/phases/05-backend-ssh-key-storage/05-CONTEXT.md` — Connection model with auth_method/ssh_key_id, SQLite/GORM patterns, API route registration
- `.planning/phases/06-frontend-ui-navigation/06-CONTEXT.md` — Sidebar navigation pattern, sheet UI pattern, card layout pattern, Zustand sidebarPage state
- `.planning/phases/07-websocket-key-auth-integration/07-CONTEXT.md` — SSH client auth flow (password + key), proxy.go connection handling pattern

### Backend Core Files
- `be/internal/ssh/proxy.go` — WebSocket SSH proxy handler, SSH dial + auth logic (both password and key paths) — reference for how SSH connections are established
- `be/internal/ssh/types.go` — Message type definitions (ConnectMessage, ServerMessage)
- `be/internal/db/models.go` — GORM model patterns (Connection, SSHKey), UUID primary keys, AutoMigrate
- `be/internal/api/routes.go` — Route registration with Go 1.22+ method routing pattern
- `be/internal/api/connections.go` — CRUD handler pattern to replicate for forwards
- `be/internal/config/encryption.go` — Decrypt/DecryptWithAAD for connection credentials (forward needs to auth with same creds)
- `be/internal/config/config.go` — Config struct, IsHostAllowed for SSRF protection

### Frontend Core Files
- `fe/src/stores/app-store.ts` — Zustand store with sidebarPage state (currently 'hosts' | 'keys' | 'new-tab'), needs 'forwards' added
- `fe/src/App.tsx` — Main layout with sidebar navigation, page rendering by sidebarPage state
- `fe/src/features/hosts/components/HostsPage.tsx` — Card grid pattern with kebab menus, active state highlighting, FAB + button
- `fe/src/features/ssh-keys/components/SSHKeysPage.tsx` — Another card grid page to reference for Port Forwards page
- `fe/src/features/ssh-keys/components/SSHKeyUploadSheet.tsx` — Right-side Sheet form pattern for create
- `fe/src/features/connections/components/ConnectionForm.tsx` — Right-side Sheet form with connection fields
- `fe/src/lib/api.ts` — API client pattern (connectionsApi, keysApi) — add forwardsApi
- `fe/src/components/ui/sheet.tsx` — shadcn Sheet component
- `fe/src/components/ui/card.tsx` — shadcn Card component

### Go SSH Library
- `golang.org/x/crypto/ssh` — Already imported. `ssh.Dial` for establishing connection, `client.Dial("tcp", addr)` for tunnel forwarding

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **GORM model pattern**: Connection and SSHKey models with UUID PK, BeforeCreate hooks, AutoMigrate — follow for PortForward model
- **API CRUD pattern**: connections.go handlers (List, Get, Create, Update, Delete) with JSON request/response — replicate for forwards
- **Route registration**: routes.go uses Go 1.22+ method routing — add /api/forwards endpoints
- **Sheet UI**: ConnectionForm and SSHKeyUploadSheet both use shadcn Sheet from right side — same for forward create/edit
- **Card grid pages**: HostsPage and SSHKeysPage both render card grids with FAB + button, kebab menus, empty states — same pattern for forwards
- **Zustand sidebarPage**: State already handles 'hosts' | 'keys' | 'new-tab' — extend to include 'forwards'
- **SSH auth logic in proxy.go**: Both password and key auth paths already implemented — forward's start endpoint reuses same auth resolution
- **Encryption**: DecryptWithAAD for key auth, Decrypt for password auth — forward needs to decrypt connection creds

### Established Patterns
- **WebSocket protocol**: Text JSON for control, binary for data — NOT used for forwards (REST API instead)
- **REST API pattern**: Fetch-based API client in api.ts, React Query hooks for data fetching
- **Feature-based structure**: `fe/src/features/<feature>/components/`, `fe/src/features/<feature>/hooks/`
- **GORM migrations**: AutoMigrate in db.go for adding new tables

### Integration Points
- New `port_forwards` SQLite table via GORM AutoMigrate
- New `/api/forwards` REST endpoints in routes.go
- New forward handler file: `be/internal/api/forwards.go`
- New feature directory: `fe/src/features/forwards/` with components and hooks
- `app-store.ts`: Add `'forwards'` to sidebarPage union type, add sidebar button
- `App.tsx`: Add `<PortForwardsPage />` rendering when sidebarPage === 'forwards'
- `api.ts`: Add `forwardsApi` with CRUD + start/stop methods
- Forward start logic: resolve connection credentials (password or key), dial SSH, open net.Listener, accept and forward connections

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 08-port-forwarding*
*Context gathered: 2026-04-28*
