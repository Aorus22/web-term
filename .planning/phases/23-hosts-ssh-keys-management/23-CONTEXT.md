# Phase 23: Hosts & SSH Keys Management - Context

**Gathered:** 2026-09-05
**Status:** Ready for planning
**Mode:** Smart Discuss (Autonomous)

<domain>
## Phase Boundary

Delivers full connection and SSH key management parity in GPUI over the Go backend REST API:
- Host cards browsing with tags, search/filtering, and kebab/action menus (Edit, Delete, Connect).
- Host connection CRUD modal (Label, Host, Port, Username, Tags, Password vs Key Auth).
- Quick-connect directly from Host cards and New Tab launcher.
- JSON Import/Export roundtrip compatible with WebTerm web app (`/api/connections/export`, `/api/connections/import`).
- SSH Key vault/pool browsing with fingerprint, key type (Ed25519/RSA/ECDSA), and creation/upload/delete.
- Passphrase prompt for encrypted keys with session-scoped in-memory caching (never stored).

</domain>

<decisions>
## Implementation Decisions

### Host Cards & Browsing
- Cards render in a responsive grid/list showing Label, Host (`user@host:port`), Auth Method badge, and Tags.
- Single-click "Connect" button directly opens a new SSH terminal tab using the existing `open_ssh_tab` infrastructure.
- Search input filters in real-time across label, host, username, and tags.

### Connection Management (CRUD)
- Connection modal supports both Create and Edit modes.
- Auth method selection: Toggle between "Password" and "SSH Key".
- When "SSH Key" is selected, dynamically lists available keys from the key pool.
- Saved connections speak the backend REST API (`GET/POST/PUT/DELETE /api/connections`).

### SSH Keys Pool
- SSH Keys view displays key name, fingerprint, algorithm/type, and creation timestamp.
- Upload/Add Key allows pasting PEM/OpenSSH private key text or reading from file; key bytes are base64-encoded to match backend `CreateKeyRequest`.
- Keys can be deleted with a confirmation prompt.

### Key Passphrase Handling
- For encrypted private keys, a modal prompts for passphrase when connecting.
- Passphrases are cached in an in-memory session map (`HashMap<String, String>`) during app runtime and never persisted to settings or disk.

### Import & Export
- "Export" triggers `GET /api/connections/export` and saves JSON file locally.
- "Import" accepts a JSON file or text payload, posts to `POST /api/connections/import`, and reports imported vs skipped count.

### The Agent's Discretion
- Layout density and UI styling follow the established GPUI JetBrains Mono / Zinc dark/light theme tokens.
- REST client methods in `webterm-backend-client` mirror existing endpoints faithfully with zero external crates.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `desktop-gpui/crates/backend-client/src/rest.rs`: `BackendClient` HTTP client with reqwest + tokio.
- `desktop-gpui/crates/backend-client/src/types.rs`: `Connection`, `SessionInfo`, `Settings` DTOs.
- `desktop-gpui/crates/webterm/src/app_state.rs`: `AppState`, `open_ssh_tab`, `client`.
- `desktop-gpui/crates/webterm/src/views/nav.rs`: Navigation sidebar with `View::Hosts` and `View::Keys`.

### Established Patterns
- Zero clippy warnings, exact pinned workspace dependencies.
- Dual-theme rendering (Dark `0x18181b` / `0x27272a` and Light `0xf8fafc` / `0xffffff`).
- Async background operations spawned via `TOKIO_RT` and bridged to GPUI context via `cx.spawn`.

### Integration Points
- Replace placeholder note in `render_content_pane` for `View::Hosts` and `View::Keys` with full views.
- Extend `webterm-backend-client` with Connection & SSH Key CRUD methods.

</code_context>

<specifics>
## Specific Ideas
- Support clicking a host card to immediately connect.
- Provide clear status toasts or inline feedback when adding/deleting connections or keys.

</specifics>

<deferred>
## Deferred Ideas
- SFTP dual-pane browsing is deferred to Phase 25.
- Local port forwarding rules are deferred to Phase 26.

</deferred>
