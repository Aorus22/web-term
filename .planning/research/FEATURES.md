# Feature Research

**Domain:** Web-based SSH client — SSH key authentication + sidebar UI redesign
**Researched:** 2026-04-28
**Confidence:** HIGH
**Scope:** v0.3.0 milestone ONLY. Features already built in v0.2.0 are NOT re-researched.

## Feature Landscape

### Table Stakes (Users Expect These)

Features users assume exist in a web SSH client that supports key auth. Missing = broken experience.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Upload existing private keys** | Users have keys already; generating new ones in-browser is unusual | MEDIUM | Accept PEM-encoded files via file picker or paste. Support RSA, ECDSA, Ed25519. Go's `ssh.ParsePrivateKey()` handles all OpenSSH/PKCS#1/PKCS#8 formats natively. |
| **Encrypted key storage at rest** | Passwords already encrypted with AES-256-GCM; keys must match that bar | LOW | Reuse existing `config.Encrypt()`/`config.Decrypt()` with same `EncryptionKey`. Store PEM blob (base64 ciphertext) in new `ssh_keys` table. |
| **Passphrase-protected key support** | Most serious SSH users encrypt their private keys; silently failing = confusing errors | HIGH | Go's `ssh.ParsePrivateKey()` returns `PassphraseMissingError` for encrypted keys. Must detect this, prompt user for passphrase, then use `ssh.ParsePrivateKeyWithPassphrase(pemBytes, []byte(passphrase))`. **Critical design decision:** where and when to prompt (see dependency notes). |
| **Per-connection auth method selection** | Users need to pick password OR key per host; some hosts use both | MEDIUM | Add `auth_method` field ("password" or "key") and optional `ssh_key_id` foreign key to `connections` table. ConnectionForm needs auth method toggle + key selector dropdown. |
| **Key pool management (CRUD)** | Users accumulate keys; need to add/rename/delete without touching filesystem | MEDIUM | New `ssh_keys` table + API endpoints. SSH Keys page in sidebar: list keys by name/type, upload button, delete, rename. |
| **Hosts page with card layout** | Card-based host list is standard in connection managers (Termius, Tabby) | MEDIUM | Replace current `ConnectionList` list view with card grid. Each card shows label, host, tags, status. Kebab menu (DropdownMenu) for edit/delete/duplicate. Click to connect. |
| **2-page sidebar navigation** | Separate hosts from keys so both are first-class, not buried in settings | LOW | Tabs at sidebar top: "Hosts" / "SSH Keys". Uses shadcn Tabs component (already available). Sidebar content switches between Hosts cards and SSH Keys list. |

### Differentiators (Competitive Advantage)

Features that go beyond table stakes and add real value for self-hosted users.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Key fingerprint display** | Users verify they selected the right key by seeing the fingerprint (SHA256 hash of public key) | LOW | Parse public key from private key via `signer.PublicKey()`, compute `ssh.FingerprintSHA256()`. Show on key list and during key selection in connection form. |
| **Key type badge** | Visual indicator of key algorithm (RSA-2048, Ed25519, ECDSA-P256) | LOW | Extract algorithm from `signer.PublicKey().Type()`. Display as badge next to key name. Helps users identify weak keys (e.g., RSA-1024). |
| **Passphrase caching per session** | Avoid re-prompting for passphrase on every reconnect within same browser session | MEDIUM | Cache decrypted signer in-memory (Zustand store or React state) keyed by `ssh_key_id`. Clear on tab close or explicit lock. Never persist decrypted keys. Security tradeoff: convenience vs. security. |
| **Drag-and-drop key upload** | Paste or drag PEM file directly into upload area | LOW | shadcn doesn't have dropzone component. Use native HTML5 drag/drop + file reader. Or simpler: textarea paste + file picker button. |
| **Key usage indicator** | Show which hosts use each key (e.g., "Used by 3 hosts") | MEDIUM | Query connections with `ssh_key_id = X`. Display count on key card. Prevents accidental deletion of in-use keys (or warn before delete). |

### Anti-Features (Commonly Requested, Often Problematic)

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| **In-browser key generation** | "I don't have a key yet, make me one" | Adds significant complexity (crypto in browser, format selection, download flow). Most SSH users already have keys. Edge case for v1. | Tell user to run `ssh-keygen` locally and upload. Add generation in v2 if demanded. |
| **Public key deployment (ssh-copy-id)** | "Upload key to server for me" | Requires a separate credential to deploy the key (chicken-and-egg). SFTP/SCP not built yet. Security surface. | Document `ssh-copy-id` usage. Defer to v2 with SFTP support. |
| **Agent forwarding** | "Use my key to hop to another server" | Major security risk — compromised remote host can use your agent. Web context makes this architecturally complex. | Explicitly do not support. Document security rationale. |
| **Key sharing between users** | "Team uses same key" | v1 is single-user, no auth system. Sharing requires multi-user infrastructure that doesn't exist. | Defer to team/shared milestone (explicitly out of scope in PROJECT.md). |
| **Never-expiring passphrase cache** | "Remember my passphrase forever" | Defeats the purpose of passphrase protection. Browser storage is not secure enough for decrypted keys. | Session-only cache that clears on tab close. Acceptable UX compromise. |
| **Storing keys in browser localStorage** | "Keys survive page refresh" | localStorage is accessible to any JS in the page (XSS = key theft). AES-256-GCM encrypted storage on server is safer. | Store encrypted on server. Prompt passphrase per session only. |

## Feature Dependencies

```
[SSH Key DB Model + API]
    │
    ├──requires──> [New ssh_keys table in SQLite]
    │
    └──enables──> [Key Pool Management UI]
                     │
                     └──requires──> [SSH Keys Page (sidebar tab)]
                                      │
                                      └──requires──> [2-Page Sidebar Nav (Tabs)]

[Per-Connection Auth Method Selection]
    │
    ├──requires──> [SSH Key DB Model + API]
    │                  (key_id FK in connections table)
    │
    ├──requires──> [ConnectionForm auth toggle]
    │                  (password vs key selector)
    │
    └──requires──> [Backend SSH key auth in proxy]
                       │
                       ├──requires──> [Passphrase prompt flow]
                       │                  (detect PassphraseMissingError → prompt → retry)
                       │
                       └──requires──> [WebSocket message extension]
                                          (connection_id flow sends key_id, backend loads key)

[Hosts Page Card Layout]
    │
    ├──requires──> [2-Page Sidebar Nav (Tabs)]
    │
    └──requires──> [Kebab menu component]
                       (DropdownMenu already in shadcn)

[Key Fingerprint Display]
    │
    └──enhances──> [Key Pool Management UI]

[Passphrase Caching]
    │
    └──enhances──> [Passphrase prompt flow]
                       (reduces friction on reconnect)
```

### Dependency Notes

- **Key DB Model is the foundation:** Everything depends on the `ssh_keys` table and CRUD API existing first. This must be Phase 1.
- **2-Page Sidebar Nav unblocks both UIs:** The Tabs component separates Hosts and SSH Keys pages. Without it, you can't navigate to keys. Phase 1 alongside DB model.
- **Backend key auth depends on DB model but NOT on UI:** The proxy.go changes (add `ssh.PublicKeys()` auth method) can be built as soon as keys are in the DB. The passphrase prompt flow is the hardest part.
- **Passphrase prompt is the hardest feature:** It requires a round-trip: WebSocket connect → backend detects encrypted key → sends `passphrase_required` message to frontend → user types passphrase → frontend sends `passphrase` message → backend retries with `ParsePrivateKeyWithPassphrase()`. This changes the WebSocket protocol.
- **Card layout is independent of key auth:** The Hosts page card redesign can happen in parallel with key auth backend work. They're orthogonal features that share only the sidebar Tabs container.
- **Export/import needs extension:** Existing connection export/import must handle new `auth_method` and `ssh_key_id` fields. Keys themselves should NOT be exported by default (security risk).

## MVP Definition

### Launch With (v0.3.0)

Minimum viable SSH key auth + sidebar redesign.

- [ ] **ssh_keys DB table + CRUD API** — Foundation for everything else. Fields: id, name, private_key (encrypted PEM), key_type, fingerprint, has_passphrase, created_at, updated_at.
- [ ] **Upload existing private keys** — File picker or paste. Parse with `ssh.ParsePrivateKey()` to validate. Store encrypted PEM in DB.
- [ ] **Passphrase-protected key support** — Detect `PassphraseMissingError`, extend WebSocket protocol with `passphrase_required`/`passphrase` messages, use `ssh.ParsePrivateKeyWithPassphrase()`.
- [ ] **Per-connection auth method selection** — `auth_method` + `ssh_key_id` fields on connections. Toggle in ConnectionForm. Backend resolves key on connect.
- [ ] **2-page sidebar navigation** — Tabs ("Hosts" / "SSH Keys") at sidebar top. Content area switches between pages.
- [ ] **Hosts page card layout** — Card grid replacing current list. Click to connect. Kebab menu for edit/delete/duplicate.
- [ ] **SSH Keys page** — Key list with name, type badge, fingerprint, delete button. Upload button.

### Add After Validation (v0.3.x)

- [ ] **Passphrase caching per session** — In-memory signer cache. Trigger: users complain about re-entering passphrase on reconnect.
- [ ] **Key usage indicator** — "Used by N hosts" on key card. Trigger: users accidentally delete in-use keys.
- [ ] **Key rename** — Trigger: users want better naming than the auto-generated name from comment field.
- [ ] **Duplicate host** — Kebab menu option. Trigger: users manually re-entering similar hosts.

### Future Consideration (v2+)

- [ ] **In-browser key generation** — Defer: edge case, `ssh-keygen` works fine.
- [ ] **Public key deployment (ssh-copy-id equivalent)** — Defer: requires SFTP (out of scope).
- [ ] **Agent forwarding** — Defer: security risk, architecturally complex.
- [ ] **SSH certificate support** — Defer: enterprise feature, not needed for individual developer use case.
- [ ] **Host key verification/pinning** — Currently using `InsecureIgnoreHostKey()`. Should add TOFU (Trust On First Use) in v2.

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| ssh_keys DB + API | HIGH | LOW | P1 |
| 2-Page Sidebar Nav (Tabs) | HIGH | LOW | P1 |
| Upload existing private keys | HIGH | MEDIUM | P1 |
| Passphrase-protected key support | HIGH | HIGH | P1 |
| Per-connection auth method selection | HIGH | MEDIUM | P1 |
| Backend SSH key auth in proxy | HIGH | MEDIUM | P1 |
| Hosts page card layout | MEDIUM | MEDIUM | P1 |
| SSH Keys page (key pool management) | HIGH | MEDIUM | P1 |
| Key fingerprint display | MEDIUM | LOW | P2 |
| Key type badge | LOW | LOW | P2 |
| Passphrase caching per session | MEDIUM | MEDIUM | P2 |
| Key usage indicator | LOW | MEDIUM | P2 |
| Duplicate host | LOW | LOW | P2 |
| Drag-and-drop key upload | LOW | LOW | P3 |
| In-browser key generation | LOW | HIGH | P3 |

**Priority key:**
- P1: Must have for v0.3.0 launch
- P2: Should have, add in v0.3.x patches
- P3: Nice to have, future consideration

## Competitor Feature Analysis

| Feature | Termius | Tabby (Terminus) | WebSSH (browser) | Our Approach |
|---------|---------|-------------------|-------------------|--------------|
| Key upload | File + paste + generate | File + agent | File upload | File picker + paste (no generation) |
| Key storage | Encrypted cloud sync | Local encrypted | Local | Server-side AES-256-GCM in SQLite |
| Passphrase handling | Prompt once, cache in memory | Agent-based | Not supported | Prompt per connect, optional session cache |
| Key per connection | Yes, dropdown selector | Yes | No | Yes, dropdown in connection form |
| Key management UI | Separate page with cards | Settings panel | None | Dedicated sidebar tab with key pool |
| Host list layout | Card grid | List + tree | List | Card grid with kebab menus |
| Fingerprint display | Yes (SHA256) | Yes | No | Yes (SHA256 via `ssh.FingerprintSHA256`) |

## Integration Points with Existing Codebase

These are the specific files and interfaces that need modification:

| Area | Existing Code | Change Needed |
|------|---------------|---------------|
| **DB Model** | `be/internal/db/models.go` — `Connection` struct | Add `ssh_keys` model. Add `auth_method`, `ssh_key_id` fields to `Connection`. |
| **Encryption** | `be/internal/config/encryption.go` — `Encrypt()`/`Decrypt()` | Reuse as-is for encrypting private key PEM blobs. |
| **SSH Proxy** | `be/internal/ssh/proxy.go` — `HandleWebSocket()` | Add key-based auth branch: fetch key from DB, decrypt, parse with `ssh.ParsePrivateKey()` or `ssh.ParsePrivateKeyWithPassphrase()`, use `ssh.PublicKeys(signer)` as auth method. Add `passphrase_required` server message. |
| **SSH Types** | `be/internal/ssh/types.go` — `ConnectMessage` | Add `key_id` field. Add `PassphraseMessage` type. Add `PassphraseRequiredMessage` server type. |
| **API Routes** | `be/internal/api/routes.go` | Add `/api/ssh-keys` CRUD routes. |
| **API Client** | `fe/src/lib/api.ts` — `Connection` interface | Add `sshKeysApi` object. Add `auth_method`, `ssh_key_id` to `Connection` interface. |
| **Connection Form** | `fe/src/features/connections/components/ConnectionForm.tsx` | Add auth method toggle (radio/select). Add key selector dropdown when "key" chosen. |
| **Sidebar** | `fe/src/App.tsx` — sidebar `<aside>` | Wrap content in shadcn Tabs. "Hosts" tab → card grid. "SSH Keys" tab → key pool page. |
| **Connection List** | `fe/src/features/connections/components/ConnectionList.tsx` | Refactor into card-based layout with kebab menus. |
| **SSH Session Hook** | `fe/src/features/terminal/useSSHSession.ts` — `connect()` | Handle `passphrase_required` server message → open passphrase prompt → send passphrase back. |
| **Store** | `fe/src/stores/app-store.ts` | Add SSH key state (key list, passphrase cache). Add `sidebarPage` state for tab switching. |
| **Export/Import** | `fe/src/features/connections/components/ExportImport.tsx` + `be/internal/api/export.go` | Handle new `auth_method`/`ssh_key_id` fields. Skip key material in export (security). |

## Sources

- **Go x/crypto/ssh package docs** — pkg.go.dev/golang.org/x/crypto/ssh (ParsePrivateKey, ParsePrivateKeyWithPassphrase, PublicKeys, PassphraseMissingError) — HIGH confidence
- **Existing codebase analysis** — All source files in be/ and fe/ examined directly — HIGH confidence
- **shadcn/ui component library** — Tabs, Card, DropdownMenu, Dialog components available — HIGH confidence
- **Competitor analysis** — Termius, Tabby, WebSSH feature comparison based on public documentation — MEDIUM confidence
- **PROJECT.md** — v0.3.0 milestone requirements and constraints — HIGH confidence

---
*Feature research for: WebTerm v0.3.0 SSH Key Auth & UI Redesign*
*Researched: 2026-04-28*
