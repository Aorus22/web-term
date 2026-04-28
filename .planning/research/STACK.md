# Technology Stack

**Project:** WebTerm v0.3.0 — SSH Key Auth & UI Redesign
**Researched:** 2026-04-28

## Executive Summary

**This milestone requires zero new backend dependencies and only two new shadcn/ui components on the frontend.** The existing `golang.org/x/crypto/ssh` (v0.50.0) already provides all SSH key parsing, passphrase handling, and public key authentication APIs needed. The existing AES-256-GCM encryption infrastructure is reused directly for private key storage at rest. On the frontend, the sidebar navigation is a state-driven tab switcher (no router needed), and the key upload uses native browser APIs.

## Recommended Stack Changes

### Backend (Go) — No New Dependencies

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `golang.org/x/crypto/ssh` | v0.50.0 (existing) | SSH key auth: `ParsePrivateKey`, `ParsePrivateKeyWithPassphrase`, `PublicKeys` | Already in `go.mod`. All key-based auth primitives are available: unencrypted key parsing, passphrase-protected key parsing, `Signer` → `AuthMethod` conversion, `PassphraseMissingError` detection, `FingerprintSHA256` for display. **Verified via pkg.go.dev official docs.** |
| `config.Encrypt/Decrypt` | existing (AES-256-GCM) | Private key encryption at rest | Same proven pattern used for password storage. PEM-encoded private keys are just strings — encrypt with existing `Encrypt()`, store in DB, decrypt with `Decrypt()` on connect. No size concerns: typical PEM keys are 1-5KB, well within AES-256-GCM practical limits. |
| GORM + SQLite | existing | `SSHKey` model storage | New `SSHKey` GORM model alongside existing `Connection` model. AutoMigrate handles schema creation. Same DB, same driver. |

#### Key APIs Used (all from existing `golang.org/x/crypto/ssh`)

```go
// Unencrypted key → Signer
signer, err := ssh.ParsePrivateKey(pemBytes)

// Encrypted key → Signer (when PassphraseMissingError is returned)
signer, err := ssh.ParsePrivateKeyWithPassphrase(pemBytes, []byte(passphrase))

// Signer → AuthMethod (for ssh.ClientConfig.Auth)
authMethod := ssh.PublicKeys(signer)

// Detect encrypted key (enables passphrase prompt flow)
var passErr *ssh.PassphraseMissingError
if errors.As(err, &passErr) { /* key needs passphrase */ }

// Key fingerprint for display
fingerprint := ssh.FingerprintSHA256(signer.PublicKey())
```

**Confidence:** HIGH — verified against pkg.go.dev for `golang.org/x/crypto/ssh` v0.50.0.

### Frontend (React) — Two New shadcn/ui Components

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `@shadcn/ui tabs` | latest (via CLI) | Sidebar 2-page navigation (Hosts / SSH Keys) | Standard shadcn/ui component built on Radix Tabs. Provides `Tabs`, `TabsList`, `TabsTrigger`, `TabsContent`. Install: `npx shadcn@latest add tabs`. **Verified via ui.shadcn.com/docs/components/tabs.** |
| `@shadcn/ui select` | latest (via CLI) | Auth method selector in connection form (Password vs SSH Key, which key) | Standard shadcn/ui component built on Radix Select. Provides `Select`, `SelectTrigger`, `SelectContent`, `SelectItem`, `SelectGroup`. Install: `npx shadcn@latest add select`. **Verified via ui.shadcn.com/docs/components/select.** |

#### No New npm Packages Required

| What | Why NOT Needed | Instead Use |
|------|----------------|-------------|
| react-router-dom | Sidebar navigation is tab-like state switching, not URL-based routing. App has no URL structure — it's a single-page terminal client with a sidebar. | Zustand state: `sidebarPage: 'hosts' \| 'keys'` controls which view renders in the sidebar. Same pattern as existing `sidebarOpen` state. |
| File upload library | SSH private keys are small PEM text files (1-5KB). No progress bars, chunking, or drag-drop needed. | Native `<input type="file" accept=".pem,.key">` + `FileReader.readAsText()`. Standard browser API, zero dependencies. |
| Form library | Existing connection form uses controlled React state directly. Adding auth method toggle + key selector follows same pattern. | Existing pattern: controlled state + `connectionsApi.create/update`. |

**Confidence:** HIGH — verified current component inventory, verified no router dependency, verified shadcn/ui tabs and select docs.

## Installation

```bash
# Frontend only — add two shadcn/ui components
cd fe
npx shadcn@latest add tabs
npx shadcn@latest add select

# Backend — nothing to install
# All Go dependencies already in go.mod
```

## Internal Components to Build

### Backend — New/Modified Files

| Component | Type | Description |
|-----------|------|-------------|
| `db/models.go` → `SSHKey` struct | New model | `ID`, `Name`, `EncryptedKey` (AES-256-GCM), `PublicKeyFingerprint`, `KeyType` (rsa/ed25519/ecdsa), `HasPassphrase` bool, `CreatedAt`, `UpdatedAt` |
| `db/models.go` → `Connection` modification | Schema change | Add `AuthMethod` (`"password"` \| `"key"`) and `SSHKeyID` (nullable FK to `SSHKey`) |
| `api/keys.go` | New handler | CRUD: `ListKeys`, `GetKey` (returns metadata only, never decrypted key), `CreateKey` (upload + encrypt + parse metadata), `UpdateKey` (rename), `DeleteKey` |
| `api/routes.go` | Route additions | `GET/POST/PUT/DELETE /api/keys` + `GET /api/keys/{id}` |
| `ssh/proxy.go` → `HandleWebSocket` | Auth logic change | When `ConnectionID` references a key-auth connection: decrypt key from DB → parse with `ParsePrivateKey` or `ParsePrivateKeyWithPassphrase` → use `ssh.PublicKeys(signer)` as AuthMethod |
| `ssh/types.go` → `ConnectMessage` | Message extension | Add `KeyID string` and `Passphrase string` fields for key-auth WebSocket connections |
| `db/db.go` → `Init` | Migration | Add `&SSHKey{}` to `AutoMigrate` call |

### Frontend — New/Modified Files

| Component | Type | Description |
|-----------|------|-------------|
| `components/ui/tabs.tsx` | New (shadcn) | Installed via CLI — sidebar navigation |
| `components/ui/select.tsx` | New (shadcn) | Installed via CLI — auth method dropdown in connection form |
| `features/keys/components/SSHKeysPage.tsx` | New | Key pool management: upload, list (with fingerprint + type), rename, delete |
| `features/keys/components/KeyUploadDialog.tsx` | New | File upload with name input, passphrase option |
| `features/connections/components/HostsPage.tsx` | New | Card-based host layout with kebab menus (replaces current sidebar list) |
| `lib/api.ts` | Extended | Add `keysApi` (list, create with FormData, update, delete) + extend `Connection` interface with `auth_method` and `ssh_key_id` |
| `stores/app-store.ts` | Extended | Add `sidebarPage: 'hosts' \| 'keys'` state + `setSidebarPage` action |
| App.tsx sidebar section | Modified | Replace current sidebar with `Tabs` containing `HostsPage` and `SSHKeysPage` |
| `features/connections/components/ConnectionForm.tsx` | Modified | Add auth method selector (`Select`) + key picker dropdown |

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| Sidebar nav | Zustand state + `Tabs` component | react-router-dom with `<Outlet>` | Overkill. App is single-page with no URL structure. Router adds ~15KB bundle for zero benefit. The sidebar is a tab switcher, not route-based navigation. |
| Key upload | Native `<input type="file">` + FileReader | react-dropzone, uppy | Private keys are tiny text files. No need for drag-drop UX, progress bars, or multi-file upload. Native API is simpler and zero-dependency. |
| Key storage encryption | Existing AES-256-GCM | HashiCorp Vault, age encryption | Self-hosted single-user app. Vault requires running a separate service. age adds a new dependency for something the existing crypto already handles. |
| Auth method UI | shadcn `Select` | shadcn `RadioGroup` or custom toggle | Select scales better if more auth methods are added later (e.g., certificate-based). Also handles the "which key" dropdown naturally when nested in `SelectGroup`. |
| Key format | PEM (OpenSSH-compatible) | PKCS#8, PuTTY PPK | PEM is the universal standard. OpenSSH, OpenSSL, and all tools use PEM. PuTTY users can convert with `puttygen`. Don't add format complexity. |

## Sources

- **golang.org/x/crypto/ssh v0.50.0** — Official Go package docs: https://pkg.go.dev/golang.org/x/crypto/ssh — Verified `ParsePrivateKey`, `ParsePrivateKeyWithPassphrase`, `PublicKeys`, `PassphraseMissingError`, `Signer` interface, `FingerprintSHA256`. All present in v0.50.0.
- **shadcn/ui Tabs** — Official docs: https://ui.shadcn.com/docs/components/tabs — Verified installation (`npx shadcn@latest add tabs`) and API (`Tabs`, `TabsList`, `TabsTrigger`, `TabsContent`).
- **shadcn/ui Select** — Official docs: https://ui.shadcn.com/docs/components/select — Verified installation (`npx shadcn@latest add select`) and composition API.
- **Existing codebase** — `be/go.mod`, `be/internal/ssh/proxy.go`, `be/internal/config/encryption.go`, `be/internal/db/models.go`, `fe/package.json`, `fe/src/stores/app-store.ts`, `fe/src/lib/api.ts`, `fe/src/App.tsx` — All analyzed for current patterns and integration points.
