# Project Research Summary

**Project:** WebTerm v0.3.0 — SSH Key Authentication & Sidebar UI Redesign
**Domain:** Self-hosted web-based SSH terminal client
**Researched:** 2026-04-28
**Confidence:** HIGH

## Executive Summary

WebTerm v0.3.0 adds SSH key-based authentication and a sidebar UI redesign to an existing web SSH terminal that currently supports only password auth. This is a well-understood domain: the Go ecosystem's `golang.org/x/crypto/ssh` library provides all needed primitives (key parsing, passphrase handling, public key auth), and the existing AES-256-GCM encryption infrastructure is directly reusable for private key storage at rest. On the frontend, a 2-page sidebar navigation using Zustand state (no router needed) separates Hosts from SSH Keys, while the connection form gains an auth method toggle. The architecture is 3-layered: backend data layer (model + API), frontend UI layer (navigation + key management + connection form), and integration layer (WebSocket key auth + passphrase flow).

The recommended approach is a strict 3-phase build order driven by dependency analysis. Phase 1 establishes the backend foundation — `SSHKey` model, CRUD API, connection schema update — with zero frontend impact and testable via curl. Phase 2 builds all frontend UI — sidebar tabs, SSH Keys page, Hosts card layout, connection form auth toggle — consuming the Phase 1 API. Phase 3 is the integration capstone — extending the WebSocket protocol for key-based SSH auth and the passphrase round-trip prompt. This order ensures each phase is independently testable and that the hardest feature (passphrase flow over WebSocket) is built on a fully validated foundation.

The key risks are: (1) passphrase-protected key silent failure — `ssh.ParsePrivateKey()` returns `PassphraseMissingError` which must be explicitly type-asserted, not treated as a generic parse error; (2) private key material leakage via logs, errors, or API responses — the API must never return decrypted key material; (3) AES-256-GCM key reuse without domain separation — the existing encryption functions need AAD context strings to prevent ciphertext confusion between passwords and keys; and (4) the WebSocket passphrase round-trip adds protocol complexity that requires careful state machine handling in both frontend and backend.

## Key Findings

### Recommended Stack

**Zero new backend dependencies.** The existing `golang.org/x/crypto/ssh` v0.50.0 already provides `ParsePrivateKey`, `ParsePrivateKeyWithPassphrase`, `PublicKeys` (Signer→AuthMethod), `PassphraseMissingError` detection, and `FingerprintSHA256`. The existing AES-256-GCM encryption in `config.Encrypt()`/`config.Decrypt()` handles private key storage at rest identically to passwords. On the frontend, only two shadcn/ui components are added via CLI (`tabs`, `select`). No react-router, no file upload library, no form library — all handled by native APIs and existing patterns.

**Core technologies:**
- `golang.org/x/crypto/ssh` v0.50.0 (existing): SSH key parsing, passphrase handling, public key auth — all APIs verified present
- `config.Encrypt()`/`config.Decrypt()` (existing): AES-256-GCM for private key storage — reuse proven password encryption pattern
- shadcn/ui `Tabs` + `Select` (new): Sidebar navigation and auth method dropdown — standard components, CLI-installed
- Zustand `sidebarPage` state (existing store): Page switching without react-router — matches existing state-only architecture
- Native `<input type="file">` + `FileReader`: Key upload — PEM keys are 1-5KB text files, no library needed

### Expected Features

**Must have (table stakes) — all P1 for v0.3.0:**
- SSH key upload (PEM files via file picker or paste) — users expect to bring existing keys
- Encrypted key storage at rest (AES-256-GCM) — must match password encryption security bar
- Passphrase-protected key support — most serious SSH users encrypt private keys
- Per-connection auth method selection (Password vs SSH Key) — some hosts use keys, some passwords
- Key pool management (CRUD) — add, rename, delete keys without filesystem access
- 2-page sidebar navigation (Hosts / SSH Keys) — both are first-class, not buried in settings
- Hosts page with card layout — standard in connection managers (Termius, Tabby)

**Should have (competitive) — P2 for v0.3.x patches:**
- Key fingerprint display (SHA256) — verify correct key selected
- Key type badge (RSA/Ed25519/ECDSA) — identify weak keys visually
- Passphrase caching per session — reduce friction on reconnect
- Key usage indicator ("Used by N hosts") — prevent accidental deletion

**Defer (v2+):**
- In-browser key generation — edge case, `ssh-keygen` works fine
- Public key deployment (ssh-copy-id) — requires SFTP (out of scope)
- Agent forwarding — security risk, architecturally complex in web context
- SSH certificate support — enterprise feature

### Architecture Approach

The architecture follows the existing patterns exactly: GORM models + AutoMigrate for schema, standard `net/http` handlers for REST API, Zustand for frontend state, TanStack Query for API data fetching, and the established WebSocket JSON protocol for terminal communication. The key architectural change is extending the WebSocket proxy (`proxy.go`) to dispatch on auth method — resolving key credentials from the DB, detecting passphrase requirements, and implementing a round-trip prompt before SSH dial. No new architectural patterns or libraries are introduced.

**Major components:**
1. **SSHKey model + CRUD API** (`be/internal/db/models.go`, `be/internal/api/sshkeys.go`) — encrypted private key storage, key metadata (type, fingerprint, passphrase flag), full REST API
2. **Sidebar page navigation** (`fe/src/App.tsx`, `fe/src/stores/app-store.ts`) — Zustand `sidebarPage` state drives conditional rendering of HostsPage vs SSHKeysPage
3. **WebSocket key auth + passphrase flow** (`be/internal/ssh/proxy.go`, `fe/src/features/terminal/useSSHSession.ts`) — extends existing protocol with `passphrase-required`/`passphrase` message types, adds `ssh.PublicKeys()` auth branch

### Critical Pitfalls

1. **PassphraseMissingError silent failure** — Always try `ssh.ParsePrivateKey()` first, then `errors.As(err, &ssh.PassphraseMissingError{})` to detect encrypted keys. Never treat it as "invalid key." Test with encrypted Ed25519 key.
2. **Private key material leakage** — API must never return decrypted key PEM. Use `json:"-"` tag on `EncryptedKey` field. Never log key bytes. Zero `[]byte` slices after parsing. Sanitize all error messages.
3. **AES-256-GCM key reuse without domain separation** — Extend existing `Encrypt()`/`Decrypt()` with AAD context strings (`"webterm:password"` vs `"webterm:ssh-private-key"`) to prevent ciphertext confusion attacks.
4. **WebSocket protocol breaking change** — Add explicit `auth_method` field to `ConnectMessage`. Keep backward compatible (infer from fields if missing). Deploy backend before frontend.
5. **DB migration destroying existing connections** — Default `auth_method="password"`, nullable `ssh_key_id`. No cascading deletes. Test against populated DB.

## Implications for Roadmap

Based on combined research, the following 3-phase structure is recommended:

### Phase 1: Backend SSH Key Storage
**Rationale:** Pure backend foundation with zero frontend impact. Testable via curl. Everything else depends on keys existing in the database.
**Delivers:** `SSHKey` GORM model, encrypted key storage, CRUD API endpoints (`/api/ssh-keys`), connection schema update (`auth_method`, `ssh_key_id`), AES-256-GCM AAD context fix.
**Addresses:** Key upload, encrypted storage, key pool management API, per-connection auth method fields.
**Avoids:** Pitfall 2 (key leakage via API — `json:"-"` from the start), Pitfall 3 (AAD context built into encryption from day 1), Pitfall 5 (safe migration with defaults).
**Estimated scope:** ~165 lines across 5 backend files.

### Phase 2: Frontend UI — Navigation & Key Management
**Rationale:** Consumes Phase 1 API. Can be built and tested with real key data. Doesn't need WebSocket auth flow yet — connections still use password auth while UI for key selection is in place.
**Delivers:** Sidebar tab navigation (Hosts/SSH Keys), SSH Keys page with upload/rename/delete, Hosts page card layout replacing ConnectionList, ConnectionForm auth method toggle + key selector, `useSSHKeys` TanStack Query hooks.
**Uses:** shadcn/ui `Tabs` and `Select` components, Zustand `sidebarPage` state, existing `connectionsApi` pattern for new `sshKeysApi`.
**Implements:** Frontend half of all 7 P1 features. Backend key data is already accessible.
**Avoids:** Pitfall 7 (build new Hosts page before removing ConnectionList), Pitfall 8 (conditional rendering, not route-based unmounting), Pitfall 11 (React Query cache invalidation after key mutations).
**Estimated scope:** ~515 lines across 9 frontend files.

### Phase 3: WebSocket Key Auth Integration
**Rationale:** The integration capstone. Depends on both backend key storage (Phase 1) and frontend key selection UI (Phase 2). The passphrase prompt flow is the hardest feature — requires extending the WebSocket protocol with a round-trip message exchange before SSH dial.
**Delivers:** Key-based SSH auth in proxy.go (`ssh.PublicKeys(signer)`), passphrase round-trip protocol (`passphrase-required` → `passphrase` messages), PassphrasePrompt component, ConnectMessage extension with `ssh_key_id`, reconnection passphrase handling.
**Uses:** `ssh.ParsePrivateKey`, `ssh.ParsePrivateKeyWithPassphrase`, `ssh.PublicKeys` from existing Go library; extended WebSocket JSON protocol.
**Implements:** End-to-end SSH key authentication — the core value proposition of v0.3.0.
**Avoids:** Pitfall 1 (explicit `PassphraseMissingError` handling), Pitfall 4 (backward-compatible protocol extension with `auth_method` field), Pitfall 9 (passphrase prompt before reconnect, not after failure).
**Estimated scope:** ~120 lines across 6 files (3 backend, 3 frontend).

### Phase Ordering Rationale

- **Phase 1 → Phase 2 dependency:** Frontend SSH Keys page and ConnectionForm key selector need real API endpoints to test against. Building UI first would require mocking.
- **Phase 2 → Phase 3 dependency:** The WebSocket auth flow needs the frontend to already know which key a connection uses (`ssh_key_id` stored in connection form). Without the UI, there's no way to select a key.
- **Phase 1 and Phase 2 partial parallelism:** Phase 1 backend work is fully independent. Phase 2's frontend component scaffolding (HostsPage, SSHKeysPage layouts) could start in parallel with Phase 1, wiring to API only after Phase 1 completes. However, sequential is safer for a single developer.
- **Phase 3 is the integration test:** All the hardest pitfalls (passphrase flow, protocol extension, reconnect handling) live here. Building it on a validated foundation reduces risk.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 3:** The WebSocket passphrase round-trip is novel — no existing pattern in the codebase for bi-directional message exchange before SSH dial. The proxy.go `HandleWebSocket` function is already 252 lines; adding auth dispatch requires careful refactoring. Consider extracting `resolveAuthMethods()` helper as recommended in ARCHITECTURE.md anti-patterns section.

Phases with standard patterns (skip research-phase):
- **Phase 1:** GORM model + CRUD API mirrors existing `Connection` pattern exactly. Encryption reuse is straightforward. Well-documented Go SSH APIs.
- **Phase 2:** shadcn/ui component installation + TanStack Query hooks follow existing codebase patterns exactly. No novel patterns needed.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All APIs verified against pkg.go.dev and ui.shadcn.com. Zero new dependencies — all reuse existing libraries. |
| Features | HIGH | Feature list derived from competitor analysis (Termius, Tabby) and codebase integration point analysis. Clear P1/P2/P3 prioritization. |
| Architecture | HIGH | Full codebase analysis with file-level integration points. 3-phase build order validated against dependency graph. ~800 LOC estimate based on file-by-file analysis. |
| Pitfalls | HIGH | 16 pitfalls identified from codebase analysis and Go library behavior. Top 5 are critical with specific prevention code. |

**Overall confidence:** HIGH

### Gaps to Address

- **AES-256-GCM AAD context implementation:** The recommendation to add context strings to encryption is clear, but the exact function signature change (`EncryptWithContext` vs modifying `Encrypt`) should be decided during Phase 1 planning. The existing `connections.go` callers will need updating.
- **Export/import v2 format:** The current export format doesn't include keys. Phase 1 or Phase 3 should extend it, but the exact versioning strategy (`version: 2`) needs spec during planning.
- **Key deletion safety:** When a key is deleted but referenced by connections, the exact UX (prevent deletion? warn? cascade to password auth?) should be specified during Phase 2 planning.

## Sources

### Primary (HIGH confidence)
- `golang.org/x/crypto/ssh` v0.50.0 — pkg.go.dev — Verified: `ParsePrivateKey`, `ParsePrivateKeyWithPassphrase`, `PublicKeys`, `PassphraseMissingError`, `FingerprintSHA256`
- shadcn/ui — ui.shadcn.com — Verified: Tabs component, Select component installation and API
- WebTerm codebase — 11 Go files, 23+ TypeScript files analyzed directly — All integration points verified

### Secondary (MEDIUM confidence)
- Competitor analysis — Termius, Tabby, WebSSH — Feature comparison based on public documentation
- PROJECT.md — v0.3.0 milestone requirements and constraints

### Tertiary (LOW confidence)
- None — all findings verified against at least one primary source

---
*Research completed: 2026-04-28*
*Ready for roadmap: yes*
