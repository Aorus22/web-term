# Phase 7: WebSocket Key Auth Integration - Context

**Gathered:** 2026-04-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can establish SSH connections using key-based authentication with passphrase support through the WebSocket proxy, while existing password-based connections continue working unchanged. This phase connects the backend key storage (Phase 5) and frontend key management UI (Phase 6) to the actual SSH connection flow. Requirements AUTH-03, AUTH-04, AUTH-05.

</domain>

<decisions>
## Implementation Decisions

### Passphrase Handshake Flow
- **D-01:** Frontend-initiated single-pass approach — frontend checks `has_passphrase` from key metadata before connecting. If true, prompts user for passphrase inline and sends it with the connect message. No multi-step WebSocket handshake needed.
- **D-02:** Extend ConnectMessage in `types.go` with three new fields: `auth_method` ("password" | "key", default "password"), `ssh_key_id` (optional string), `passphrase` (optional string, only sent for encrypted keys).
- **D-03:** Backend reads `auth_method` to decide auth path: "password" uses existing Password+KeyboardInteractive flow; "key" fetches SSHKey from DB, decrypts with SSHKeyAAD, uses passphrase (if provided) to parse the private key, creates ssh.Signer for auth.
- **D-04:** Backward-compatible — existing password connections omit or send `auth_method="password"`. No change to their flow.

### Passphrase Prompt UX
- **D-05:** New dedicated `PassphrasePrompt` component (not reusing PasswordPrompt). Shows key-specific info: key name, key type badge, and text like "Enter passphrase for [key name]".
- **D-06:** PassphrasePrompt uses same inline pattern as PasswordPrompt (centered form in TerminalPane area, not a modal).

### Reconnection & Passphrase Caching
- **D-07:** Passphrase cached per-session (per-tab) in a React ref within TerminalPane. Different tabs with the same key manage their own cache independently.
- **D-08:** Cache cleared when tab is closed or page is refreshed. Never persisted to disk, localStorage, or backend.
- **D-09:** Reconnects use cached passphrase without prompting. If cache is empty (e.g., page refresh), user is prompted again.

### Quick-connect with Key Auth
- **D-10:** Key auth is for saved connections only. Quick-connect bar remains password-only — no key selector added to NewTabView/QuickConnect.
- **D-11:** When a saved connection has `auth_method="key"`, the HostsPage click-to-connect flow handles the key auth path (checks has_passphrase, prompts if needed, sends passphrase with connect).

### Backend SSH Auth Logic
- **D-12:** When `auth_method="key"` and `connection_id` is provided: backend fetches Connection from DB, gets `ssh_key_id`, fetches SSHKey, decrypts with SSHKeyAAD, parses with `ssh.ParseRawPrivateKeyWithPassphrase` if passphrase provided (or `ssh.ParseRawPrivateKey` if not), creates `ssh.Signer` via `ssh.NewSignerFromKey`, uses it as auth method.
- **D-13:** If key decryption or parsing fails, return error via WebSocket so frontend can display it.

### the agent's Discretion
- Exact error messages for key auth failures
- Whether to add `auth_method` field to frontend ConnectOptions type or derive it from connection data
- How TerminalPane determines if a passphrase is needed (check connection's key has_passphrase via API vs pass it in ConnectOptions)
- Exact PassphrasePrompt styling and layout details
- Handling edge case: key deleted between connection save and connect attempt

</decisions>

<specifics>
## Specific Ideas

- Single-pass approach keeps WebSocket protocol simple — no new message types needed
- Per-tab passphrase cache is the right balance of UX and security
- Quick-connect stays simple — key auth is a saved-connection feature

</specifics>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` § "Connection Auth" (AUTH-03, AUTH-04, AUTH-05)

### Prior Phase Context
- `.planning/phases/05-backend-ssh-key-storage/05-CONTEXT.md` — SSHKey model structure, encryption with SSHKeyAAD, keys API endpoints, Connection model with auth_method/ssh_key_id fields
- `.planning/phases/06-frontend-ui-navigation/06-CONTEXT.md` — Frontend Connection type with auth_method/ssh_key_id, key selector in connection form, HostsPage click-to-connect flow

### Backend Core Files
- `be/internal/ssh/proxy.go` — WebSocket SSH proxy handler, password+keyboard-interactive auth flow to extend
- `be/internal/ssh/types.go` — ConnectMessage, ServerMessage, ResizeMessage structs to extend
- `be/internal/config/encryption.go` — DecryptWithAAD function for decrypting SSH keys (use SSHKeyAAD)
- `be/internal/api/keys.go` — SSHKey CRUD, extractPublicKey helper for key type handling
- `be/internal/db/models.go` — SSHKey model (has EncryptedKey, HasPassphrase fields) and Connection model (auth_method, ssh_key_id fields)
- `be/internal/api/routes.go` — Route setup with WebSocket handler

### Frontend Core Files
- `fe/src/features/terminal/useSSHSession.ts` — WebSocket lifecycle hook, connect message construction, session state management
- `fe/src/features/terminal/types.ts` — ConnectOptions and SSHSession interfaces to extend
- `fe/src/features/terminal/TerminalPane.tsx` — Terminal component with state-based rendering, PasswordPrompt integration point
- `fe/src/features/terminal/PasswordPrompt.tsx` — Existing inline prompt pattern to reference for PassphrasePrompt
- `fe/src/features/hosts/components/HostsPage.tsx` — Click-to-connect handler that needs key auth flow
- `fe/src/lib/api.ts` — API client with Connection and SSHKey types
- `fe/src/stores/app-store.ts` — Zustand store with session state

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **encryption.go DecryptWithAAD**: Decrypt SSH key blob using `config.DecryptWithAAD(key.EncryptedKey, cfg.EncryptionKey, []byte(config.SSHKeyAAD))`
- **keys.go extractPublicKey**: Already handles RSA/Ed25519/ECDSA type switching — pattern for creating ssh.Signer
- **PasswordPrompt**: Inline prompt pattern (centered form, KeyRound icon, password input, cancel/connect buttons) — reference for PassphrasePrompt
- **useSSHSession connect()**: Handles saved vs quick-connect flows, WebSocket message construction — extend for key auth

### Established Patterns
- **WebSocket protocol**: Text JSON for control messages (connect, resize, connected, error, disconnected), binary for SSH data
- **Saved connection flow**: Send connection_id → backend fetches from DB → decrypts password. Key auth follows same pattern but decrypts key instead of password.
- **TerminalPane state machine**: connecting → PasswordPrompt/spinner → connected (terminal) → error/disconnected. Key auth adds passphrase prompt before spinner.
- **Session reconnect**: lastOptionsRef stores ConnectOptions for retry — needs to include passphrase from cache

### Integration Points
- `proxy.go HandleWebSocket`: Add key auth branch after fetching Connection from DB — check auth_method, fetch SSHKey, decrypt, create Signer
- `types.go ConnectMessage`: Add AuthMethod, SSHKeyID, Passphrase fields
- `useSSHSession.ts connect()`: When connection has key auth, check has_passphrase, prompt if needed, include auth_method/ssh_key_id/passphrase in connect message
- `TerminalPane.tsx`: Add passphrase prompt state between connecting and connected states
- New component: `PassphrasePrompt.tsx` in `fe/src/features/terminal/`

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 07-websocket-key-auth-integration*
*Context gathered: 2026-04-28*
