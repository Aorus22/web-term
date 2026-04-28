# Architecture: SSH Key Auth & UI Redesign Integration

**Project:** WebTerm v0.3.0
**Researched:** 2026-04-28
**Confidence:** HIGH (based on full codebase analysis, verified Go SSH library APIs)

## Current Architecture Summary

```
Browser (React)                    Go Backend                     SSH Target
┌──────────────────┐              ┌──────────────────┐           ┌──────────┐
│ App.tsx           │              │ main.go          │           │          │
│  ├─ Sidebar       │   REST      │  ├─ routes.go    │           │          │
│  │   ├─ TagFilter │◄───────────►│  │   ├─ CRUD     │  SQLite   │          │
│  │   └─ ConnList  │             │  │   └─ Export   │◄────────►│          │
│  ├─ TabBar        │             │  └─ /ws handler  │           │          │
│  └─ TerminalPanes │  WebSocket  │      (proxy.go)  │  SSH      │          │
│      └─ useSSH    │◄───────────►│      password    │──────────►│          │
│         Session   │             │      keyboard-   │           │          │
│                   │             │      interactive │           │          │
└──────────────────┘              └──────────────────┘           └──────────┘
```

**Key files examined:**
- `be/internal/ssh/proxy.go` — WebSocket handler, password-only auth (lines 110-124)
- `be/internal/ssh/types.go` — `ConnectMessage`, `ServerMessage`, `ResizeMessage`
- `be/internal/db/models.go` — `Connection` model (password-only, no auth_method)
- `be/internal/config/encryption.go` — AES-256-GCM encrypt/decrypt (reusable for keys)
- `be/internal/api/connections.go` — CRUD with password encryption
- `fe/src/App.tsx` — No router, sidebar is inline component tree
- `fe/src/features/terminal/useSSHSession.ts` — WebSocket lifecycle, connect message builder
- `fe/src/features/terminal/TerminalPane.tsx` — Session state machine (connecting→connected→error→disconnected)
- `fe/src/features/connections/components/ConnectionForm.tsx` — Password-only form

---

## New Architecture

```
Browser (React)                         Go Backend                        SSH Target
┌───────────────────────────┐          ┌──────────────────────────┐      ┌──────────┐
│ App.tsx                    │          │ main.go                  │      │          │
│  ├─ Sidebar               │          │  ├─ routes.go            │      │          │
│  │   ├─ Nav: Hosts|Keys   │  REST    │  │   ├─ Connections CRUD │      │          │
│  │   ├─ HostsPage (cards) │◄────────►│  │   ├─ SSHKeys CRUD   │      │          │
│  │   └─ SSHKeysPage       │          │  │   └─ Export         │      │          │
│  ├─ TabBar                │          │  └─ /ws handler          │      │          │
│  └─ TerminalPanes         │ WebSocket│      (proxy.go)          │      │          │
│      └─ useSSHSession     │◄────────►│      password OR         │ SSH  │          │
│         + passphrase flow │          │      publickey auth      │─────►│          │
│                           │          │      + passphrase prompt │      │          │
└───────────────────────────┘          └──────────────────────────┘      └──────────┘
```

---

## Integration Points (New vs Modified)

### Backend — New Components

#### 1. SSHKey Model (`be/internal/db/models.go` — APPEND)

```go
type SSHKey struct {
    ID            string    `json:"id" gorm:"primaryKey;type:varchar(36)"`
    Name          string    `json:"name" gorm:"not null"`
    PrivateKey    string    `json:"private_key,omitempty" gorm:"-"`     // Never sent to client
    EncryptedKey  string    `json:"-" gorm:"column:private_key"`        // AES-256-GCM encrypted PEM
    PublicKey     string    `json:"public_key" gorm:"type:text"`        // Plaintext for display
    KeyType       string    `json:"key_type" gorm:"type:varchar(20)"`   // "RSA", "Ed25519", "ECDSA"
    Fingerprint   string    `json:"fingerprint" gorm:"type:varchar(64)"` // SHA256 fingerprint
    HasPassphrase bool      `json:"has_passphrase"`                     // Detected on upload
    CreatedAt     time.Time `json:"created_at"`
    UpdatedAt     time.Time `json:"updated_at"`
}

func (k *SSHKey) BeforeCreate(tx *gorm.DB) error {
    if k.ID == "" {
        k.ID = uuid.New().String()
    }
    return nil
}
```

**Key decisions:**
- Reuse existing `config.Encrypt()`/`config.Decrypt()` — same AES-256-GCM, same encryption key
- `HasPassphrase` detected at upload time by trying `ssh.ParsePrivateKey()` and checking for `ssh.PassphraseMissingError`
- `Fingerprint` computed via `ssh.NewPublicKey()` → `ssh.MarshalAuthorizedKey()` → SHA256 hash — allows visual key verification
- `PublicKey` stored in plaintext — it's public, needed for display. Computed from private key at upload
- `EncryptedKey` stores the full PEM block (including `ENCRYPTED` header if passphrase-protected) — the backend decrypts the AES layer but the SSH key itself may still need a passphrase

#### 2. SSHKey API Handler (`be/internal/api/sshkeys.go` — NEW FILE)

| Route | Method | Purpose |
|-------|--------|---------|
| `/api/ssh-keys` | GET | List all keys (name, type, fingerprint, has_passphrase only) |
| `/api/ssh-keys/{id}` | GET | Get key details (still no private key material) |
| `/api/ssh-keys` | POST | Upload key (PEM content in body, auto-detect type/passphrase) |
| `/api/ssh-keys/{id}` | PUT | Rename key |
| `/api/ssh-keys/{id}` | DELETE | Delete key (check if any connection references it) |

**Upload flow:**
1. Client sends `{ "name": "my-key", "private_key": "-----BEGIN..." }`
2. Backend parses PEM → detect key type via `ssh.ParsePrivateKey()`
3. If `PassphraseMissingError` → set `has_passphrase = true`, extract public key from error's `PublicKey` field
4. If success → set `has_passphrase = false`, extract public key from signer
5. Compute fingerprint: `ssh.FingerprintSHA256(publicKey)`
6. Encrypt PEM with `config.Encrypt(pemContent, cfg.EncryptionKey)`
7. Store in DB

#### 3. DB Migration (`be/internal/db/db.go` — MODIFY)

```go
func Init(dbPath string) (*gorm.DB, error) {
    // ... existing code ...
    if err := db.AutoMigrate(&Connection{}, &SSHKey{}); err != nil {  // Add SSHKey
        return nil, err
    }
    return db, nil
}
```

GORM's AutoMigrate handles additive schema changes (adding columns to existing `connections` table, creating new `ssh_keys` table). No destructive migration needed.

---

### Backend — Modified Components

#### 4. Connection Model Update (`be/internal/db/models.go` — MODIFY)

```go
type Connection struct {
    // ... existing fields ...
    AuthMethod string  `json:"auth_method" gorm:"default:password;type:varchar(10)"` // "password" or "key"
    SSHKeyID   *string `json:"ssh_key_id,omitempty" gorm:"type:varchar(36)"`          // FK to SSHKey
}
```

**Migration safety:** `default:password` ensures existing connections keep working without any data migration.

#### 5. WebSocket Proxy Auth (`be/internal/ssh/proxy.go` — MODIFY, lines 76-124)

This is the **most critical integration point.** The current auth block (lines 110-124):

```go
// CURRENT — password-only
sshConfig := &ssh.ClientConfig{
    User: connectMsg.User,
    Auth: []ssh.AuthMethod{
        ssh.Password(connectMsg.Password),
        ssh.KeyboardInteractive(func(user, instruction string, questions []string, echos []bool) ([]string, error) {
            answers := make([]string, len(questions))
            for i := range answers { answers[i] = connectMsg.Password }
            return answers, nil
        }),
    },
    HostKeyCallback: ssh.InsecureIgnoreHostKey(),
    Timeout:         10 * time.Second,
}
```

**New flow — dispatch on auth method:**

```go
// NEW — supports both password and key auth
var authMethods []ssh.AuthMethod

if connectMsg.SSHKeyID != "" {
    // Key-based auth: fetch encrypted key from DB
    var key db.SSHKey
    if err := database.First(&key, "id = ?", connectMsg.SSHKeyID).Error; err != nil {
        sendWSError(wsWrite, "SSH key not found")
        return
    }
    decryptedPEM, err := config.Decrypt(key.EncryptedKey, cfg.EncryptionKey)
    if err != nil {
        sendWSError(wsWrite, "Failed to decrypt SSH key")
        return
    }
    signer, err := ssh.ParsePrivateKey([]byte(decryptedPEM))
    if err != nil {
        if _, ok := err.(*ssh.PassphraseMissingError); ok {
            // Need passphrase — request from client
            // (see passphrase flow below)
        }
        sendWSError(wsWrite, "Failed to parse SSH key")
        return
    }
    authMethods = []ssh.AuthMethod{ssh.PublicKeys(signer)}
} else {
    // Password auth (existing flow)
    authMethods = []ssh.AuthMethod{
        ssh.Password(connectMsg.Password),
        ssh.KeyboardInteractive(/* ... existing ... */),
    }
}

sshConfig := &ssh.ClientConfig{
    User:            connectMsg.User,
    Auth:            authMethods,
    HostKeyCallback: ssh.InsecureIgnoreHostKey(),
    Timeout:         10 * time.Second,
}
```

#### 6. Passphrase Prompt Flow (`be/internal/ssh/types.go` — MODIFY + `proxy.go` — MODIFY)

New message types needed for the passphrase round-trip:

```go
// Add to types.go

// Server → Client: key requires passphrase
// Type: "passphrase-required"
type PassphraseRequiredMessage struct {
    Type    string `json:"type"`    // "passphrase-required"
    KeyName string `json:"key_name"` // Display name of the key
}

// Client → Server: user provided passphrase
// Type: "passphrase"
type PassphraseMessage struct {
    Type       string `json:"type"`       // "passphrase"
    Passphrase string `json:"passphrase"` // User-provided passphrase
}
```

**Flow:**
1. Backend tries `ssh.ParsePrivateKey(decryptedPEM)`
2. Gets `*ssh.PassphraseMissingError` → sends `{"type": "passphrase-required", "key_name": "my-key"}`
3. Frontend shows passphrase prompt (reuse PasswordPrompt pattern)
4. User enters passphrase → frontend sends `{"type": "passphrase", "passphrase": "..."}`
5. Backend calls `ssh.ParsePrivateKeyWithPassphrase(decryptedPEM, []byte(passphrase))`
6. On success → proceed with `ssh.PublicKeys(signer)` and normal session
7. On failure → send error, frontend shows retry or cancel

**WebSocket read loop modification:** The proxy needs to handle the initial passphrase exchange *before* entering the main I/O forwarding loop. Currently the connect message is read once (line 57), then the proxy enters forwarding goroutines. The passphrase flow needs a second read *before* forwarding starts.

```go
// Pseudocode for passphrase flow insertion point
// After parsing connectMsg and resolving credentials, BEFORE dialing SSH:

if needsPassphrase {
    // Send passphrase-required to client
    wsWrite(TextMessage, json.Marshal(PassphraseRequiredMessage{...}))
    
    // Read passphrase response (blocking, before SSH dial)
    _, msgData, err := conn.ReadMessage()
    // Parse passphrase message
    // Try ParsePrivateKeyWithPassphrase
    // If fail, send error and return (client can retry entire connection)
}
```

#### 7. ConnectMessage Update (`be/internal/ssh/types.go` — MODIFY)

```go
type ConnectMessage struct {
    Type         string `json:"type"`
    Host         string `json:"host"`
    Port         int    `json:"port"`
    User         string `json:"user"`
    Password     string `json:"password,omitempty"`
    ConnectionID string `json:"connection_id,omitempty"`
    SSHKeyID     string `json:"ssh_key_id,omitempty"`  // NEW: key ID (for saved or quick-connect)
    Rows         int    `json:"rows,omitempty"`
    Cols         int    `json:"cols,omitempty"`
}
```

#### 8. Routes Update (`be/internal/api/routes.go` — MODIFY)

```go
func SetupRoutes(mux *http.ServeMux, db *gorm.DB, cfg *config.Config) {
    h := &ConnectionHandler{DB: db, Cfg: cfg}
    kh := &SSHKeyHandler{DB: db, Cfg: cfg}  // NEW

    // Existing connection routes
    mux.HandleFunc("GET /api/connections", h.ListConnections)
    // ... existing ...

    // NEW SSH key routes
    mux.HandleFunc("GET /api/ssh-keys", kh.ListKeys)
    mux.HandleFunc("GET /api/ssh-keys/{id}", kh.GetKey)
    mux.HandleFunc("POST /api/ssh-keys", kh.CreateKey)
    mux.HandleFunc("PUT /api/ssh-keys/{id}", kh.UpdateKey)
    mux.HandleFunc("DELETE /api/ssh-keys/{id}", kh.DeleteKey)

    // WebSocket
    mux.HandleFunc("GET /ws", ssh.HandleWebSocket(db, cfg))
}
```

#### 9. Connection CRUD Update (`be/internal/api/connections.go` — MODIFY)

- **CreateConnection:** Handle `auth_method` and `ssh_key_id` fields; validate key exists if auth_method is "key"; still encrypt password if provided
- **UpdateConnection:** Same validation; allow switching auth method
- **GetConnection:** Include `auth_method` and `ssh_key_id` in response
- **ListConnections:** Include `auth_method` in response (front-end needs this for display)

---

### Frontend — New Components

#### 10. SSH Keys API Client (`fe/src/lib/api.ts` — APPEND)

```typescript
const KEYS_BASE = `${import.meta.env.VITE_API_URL || 'http://localhost:8080'}/api/ssh-keys`

export interface SSHKey {
  id: string
  name: string
  public_key: string
  key_type: string        // "RSA", "Ed25519", "ECDSA"
  fingerprint: string     // SHA256:xxx
  has_passphrase: boolean
  created_at: string
  updated_at: string
}

export const sshKeysApi = {
  list: (): Promise<SSHKey[]> => fetch(KEYS_BASE).then(r => r.json()),
  get: (id: string): Promise<SSHKey> => fetch(`${KEYS_BASE}/${id}`).then(r => r.json()),
  create: (data: { name: string; private_key: string }): Promise<SSHKey> =>
    fetch(KEYS_BASE, { method: 'POST', headers: {'Content-Type': 'application/json'}, body: JSON.stringify(data) }).then(r => r.json()),
  update: (id: string, data: { name: string }): Promise<SSHKey> =>
    fetch(`${KEYS_BASE}/${id}`, { method: 'PUT', headers: {'Content-Type': 'application/json'}, body: JSON.stringify(data) }).then(r => r.json()),
  delete: (id: string): Promise<void> =>
    fetch(`${KEYS_BASE}/${id}`, { method: 'DELETE' }).then(r => { if (!r.ok) throw new Error('Delete failed') }),
}
```

#### 11. SSH Keys Hooks (`fe/src/features/ssh-keys/hooks/useSSHKeys.ts` — NEW FILE)

Standard TanStack Query hooks mirroring the `useConnections` pattern:
- `useSSHKeys()` — list
- `useCreateSSHKey()` — create (upload)
- `useUpdateSSHKey()` — rename
- `useDeleteSSHKey()` — delete

#### 12. SSH Keys Page (`fe/src/features/ssh-keys/components/SSHKeysPage.tsx` — NEW FILE)

Key pool management UI:
- List of SSH keys with columns: name, type, fingerprint, passphrase badge, actions
- Upload button → dialog with file picker + name input
- Per-key actions: rename (inline edit), delete (with confirmation)
- Empty state: "No SSH keys uploaded yet"

---

### Frontend — Modified Components

#### 13. Connection Type Update (`fe/src/lib/api.ts` — MODIFY)

```typescript
export interface Connection {
  // ... existing fields ...
  auth_method: 'password' | 'key'    // NEW
  ssh_key_id?: string                 // NEW
}
```

#### 14. App Sidebar → Page Navigation (`fe/src/App.tsx` — MODIFY)

**No react-router needed.** The current app uses Zustand for all state and has no routing. Adding a full router for 2 pages is over-engineering. Use Zustand state for page switching:

```typescript
// app-store.ts addition
sidebarPage: 'hosts' | 'keys'
setSidebarPage: (page: 'hosts' | 'keys') => void
```

Sidebar structure changes from:
```
┌─────────────────────┐
│ WebTerm       [+]   │
│ ─────────────────── │
│ [Tag Filter]        │
│ Connection 1        │
│ Connection 2        │
│ Connection 3        │
└─────────────────────┘
```

To:
```
┌─────────────────────┐
│ WebTerm       [+]   │
│ ─────────────────── │
│ [Hosts] [SSH Keys]  │  ← tab-style nav buttons
│ ─────────────────── │
│ (page content)      │
│  Hosts: card grid   │
│  Keys: key list     │
└─────────────────────┘
```

The sidebar remains the same width (`w-[280px]`). The page content replaces the `TagFilter` + `ConnectionList` section.

#### 15. Hosts Page in Sidebar (`fe/src/features/connections/components/HostsPage.tsx` — NEW FILE)

Card-based host layout inside the sidebar, replacing the flat `ConnectionList`. Each card shows:
- Host label + username@host
- Auth method indicator (key icon vs password icon)
- Kebab menu (edit, duplicate, delete, copy host:port)
- Click to connect

This is a **compact card grid** inside the 280px sidebar, not the full-page NewTabView cards. Cards stack vertically (1 column at 280px width).

#### 16. ConnectionForm Auth Method Toggle (`fe/src/features/connections/components/ConnectionForm.tsx` — MODIFY)

**Changes to the form:**
1. Add auth method toggle (RadioGroup or Tabs): "Password" | "SSH Key"
2. When "Password" selected: show existing password input (current behavior)
3. When "SSH Key" selected:
   - Show SSH key selector dropdown (populated from `useSSHKeys()`)
   - Hide password field
   - Show "No keys? Upload one" link that switches to SSH Keys page
4. Validation: if auth_method is "key", ssh_key_id is required
5. On submit: include `auth_method` and `ssh_key_id` in payload

#### 17. ConnectOptions Type Update (`fe/src/features/terminal/types.ts` — MODIFY)

```typescript
export interface ConnectOptions {
  connectionId?: string
  host?: string
  port?: number
  username?: string
  password?: string
  sshKeyId?: string       // NEW: for quick-connect with key
  rows?: number
  cols?: number
}
```

#### 18. WebSocket Connect Message Update (`fe/src/features/terminal/useSSHSession.ts` — MODIFY)

The `connect()` callback builds the connect message (lines 86-101). Update to include `ssh_key_id`:

```typescript
// For saved connections with key auth
const connectMsg = opts.connectionId
  ? JSON.stringify({
      type: 'connect',
      connection_id: opts.connectionId,
      ssh_key_id: opts.sshKeyId,  // NEW
      rows, cols,
    })
  : JSON.stringify({
      type: 'connect',
      host: opts.host,
      port: opts.port ?? 22,
      user: opts.username,
      password: opts.password,
      ssh_key_id: opts.sshKeyId,  // NEW
      rows, cols,
    })
```

#### 19. Passphrase Prompt in TerminalPane (`fe/src/features/terminal/TerminalPane.tsx` + `useSSHSession.ts` — MODIFY)

New handling in WebSocket `onmessage` for `passphrase-required` message type:
1. `useSSHSession.ts`: On receiving `{"type": "passphrase-required"}`, update session status to `'passphrase-required'`
2. `TerminalPane.tsx`: When `session.status === 'passphrase-required'`, show `PassphrasePrompt` component (reuse `PasswordPrompt` with different copy)
3. `PassphrasePrompt` on submit → send `{"type": "passphrase", "passphrase": "..."}` via WebSocket
4. Backend responds with `connected` or `error`

**Session status type update** (`fe/src/features/terminal/types.ts`):
```typescript
export type SessionStatus = 'connecting' | 'connected' | 'disconnected' | 'error' | 'passphrase-required'
```

**PassphrasePrompt component** — can be a variant of the existing `PasswordPrompt` with:
- Title: "Enter Passphrase for [key name]"
- Description: "This SSH key is passphrase-protected"
- Different icon (KeyRound already used, works fine)

#### 20. App Store Update (`fe/src/stores/app-store.ts` — MODIFY)

```typescript
interface AppState {
  // ... existing ...
  sidebarPage: 'hosts' | 'keys'       // NEW
  setSidebarPage: (page: 'hosts' | 'keys') => void  // NEW
}
```

---

## Data Flow Changes

### Current Flow: Password Auth via Saved Connection

```
1. User clicks connection in sidebar
2. React: addSession({connectionId: "abc-123", ...})
3. TerminalPane mounts → useSSHSession.connect({connectionId: "abc-123"})
4. WebSocket opens → sends {"type":"connect", "connection_id":"abc-123", "rows":24, "cols":80}
5. Go proxy: fetches Connection from DB, decrypts password
6. Go proxy: ssh.Dial() with ssh.Password()
7. Go proxy: sends {"type":"connected", "session_id":"..."}
8. Bidirectional I/O begins
```

### New Flow: Key Auth via Saved Connection (No Passphrase)

```
1-4. (same as above, connect message now includes ssh_key_id implicitly via connection)
5. Go proxy: fetches Connection from DB → sees auth_method="key", ssh_key_id="key-456"
6. Go proxy: fetches SSHKey from DB → decrypts private key PEM
7. Go proxy: ssh.ParsePrivateKey(decryptedPEM) → signer
8. Go proxy: ssh.Dial() with ssh.PublicKeys(signer)
9-10. (same as above)
```

### New Flow: Key Auth with Passphrase Prompt

```
1-6. (same as above)
7. Go proxy: ssh.ParsePrivateKey(decryptedPEM) → *ssh.PassphraseMissingError
8. Go proxy: sends {"type":"passphrase-required", "key_name":"my-prod-key"}
9. React: session status → "passphrase-required"
10. TerminalPane: renders PassphrasePrompt
11. User enters passphrase → WebSocket sends {"type":"passphrase", "passphrase":"secret"}
12. Go proxy: ssh.ParsePrivateKeyWithPassphrase(decryptedPEM, []byte("secret")) → signer
13. Go proxy: ssh.Dial() with ssh.PublicKeys(signer)
14-15. (same as above)
```

### New Flow: Quick Connect with Key (New Feature)

```
1. User fills QuickConnect or ConnectionForm with host + username + selects SSH key
2. React: addSession({host, port, username, sshKeyId: "key-456", ...})
3. TerminalPane mounts → useSSHSession.connect({host, port, username, sshKeyId: "key-456"})
4. WebSocket opens → sends {"type":"connect", "host":"10.0.0.1", "user":"root", "ssh_key_id":"key-456", ...}
5. Go proxy: fetches SSHKey from DB, decrypts, creates signer
6. Go proxy: ssh.Dial() with ssh.PublicKeys(signer) and connectMsg.User
7. (proceeds as normal)
```

---

## Component Dependency Graph

```
                        ┌─────────────────────┐
                        │  be/db/models.go     │
                        │  + SSHKey model       │
                        │  + Connection update  │
                        └──────────┬───────────┘
                                   │
                    ┌──────────────┼───────────────┐
                    │              │               │
           ┌────────▼──────┐  ┌───▼───────────┐  │
           │ be/api/       │  │ be/db/db.go   │  │
           │ sshkeys.go    │  │ + AutoMigrate  │  │
           │ (NEW)         │  │   SSHKey       │  │
           └────────┬──────┘  └───────────────┘  │
                    │                             │
           ┌────────▼──────┐                     │
           │ be/api/       │                     │
           │ routes.go     │◄────────────────────┘
           │ + key routes   │
           └────────┬──────┘
                    │
           ┌────────▼──────────────────────┐
           │ be/ssh/proxy.go               │
           │ + key auth dispatch           │
           │ + passphrase prompt flow      │
           │ + ssh.PublicKeys() integration│
           └──────────┬───────────────────┘
                      │
    ┌─────────────────┼──────────────────┐
    │                 │                  │
┌───▼──────┐  ┌──────▼───────┐  ┌───────▼──────┐
│ fe/lib/  │  │ fe/stores/   │  │ fe/features/ │
│ api.ts   │  │ app-store.ts │  │ terminal/    │
│ +SSHKey  │  │ +sidebarPage │  │ types.ts     │
│ +api     │  │              │  │ +passphrase  │
└──┬───────┘  └──────┬───────┘  │ +ssh_key_id  │
   │                 │          └──────┬────────┘
   │        ┌────────▼─────────┐      │
   │        │ fe/App.tsx       │      │
   │        │ +page navigation │      │
   │        └───┬──────────┬───┘      │
   │            │          │          │
   │   ┌────────▼──┐  ┌───▼────────┐ │
   │   │ HostsPage │  │ SSHKeysPage│ │
   │   │ (cards)   │  │ (key pool) │ │
   │   └─────┬─────┘  └────────────┘ │
   │         │                       │
   │  ┌──────▼──────────────┐       │
   │  │ ConnectionForm       │       │
   │  │ +auth method toggle  │       │
   │  │ +key selector        │       │
   │  └─────────────────────┘       │
   │                                │
   │  ┌─────────────────────────────▼──┐
   │  │ useSSHSession.ts               │
   │  │ +passphrase-required handling   │
   │  │ +ssh_key_id in connect msg     │
   │  └────────────────────────────────┘
   │
   │  ┌────────────────────────────────┐
   └─►│ useSSHKeys.ts (NEW)            │
      │ +TanStack Query hooks           │
      └────────────────────────────────┘
```

---

## Build Order (Phase Dependencies)

### Phase 1: Backend SSH Key Storage (Independent)

**What:** All backend data layer changes — model, API, encryption.

| Component | File | Action |
|-----------|------|--------|
| SSHKey model | `be/internal/db/models.go` | APPEND |
| DB migration | `be/internal/db/db.go` | MODIFY |
| SSHKey CRUD | `be/internal/api/sshkeys.go` | NEW |
| Connection update | `be/internal/db/models.go` | MODIFY |
| Routes update | `be/internal/api/routes.go` | MODIFY |
| Connection handler | `be/internal/api/connections.go` | MODIFY |

**Why first:** Zero frontend impact. Pure backend. Can be tested with curl. Sets up the data layer that everything else depends on.

**Dependencies:** None. Uses existing `config.Encrypt/Decrypt`.

### Phase 2: Frontend UI Navigation + Key Management (Depends on Phase 1 API)

**What:** Sidebar redesign, SSH Keys page, ConnectionForm update.

| Component | File | Action |
|-----------|------|--------|
| SSHKey API client | `fe/src/lib/api.ts` | APPEND |
| useSSHKeys hooks | `fe/src/features/ssh-keys/hooks/useSSHKeys.ts` | NEW |
| SSH Keys page | `fe/src/features/ssh-keys/components/SSHKeysPage.tsx` | NEW |
| Sidebar navigation | `fe/src/App.tsx` | MODIFY |
| App store | `fe/src/stores/app-store.ts` | MODIFY |
| Hosts page (cards) | `fe/src/features/connections/components/HostsPage.tsx` | NEW |
| Connection form | `fe/src/features/connections/components/ConnectionForm.tsx` | MODIFY |
| Connection type | `fe/src/lib/api.ts` | MODIFY |

**Why second:** Needs Phase 1 API endpoints to actually list/create/delete keys. The ConnectionForm key selector needs real data. But doesn't need the WebSocket auth flow yet — connections can still use password auth while the UI for selecting key auth is in place.

**Dependencies:** Phase 1 (API endpoints for SSH keys)

### Phase 3: SSH Key Auth Flow (Depends on Phase 1 + 2)

**What:** WebSocket key auth integration, passphrase prompt flow.

| Component | File | Action |
|-----------|------|--------|
| ConnectMessage type | `be/internal/ssh/types.go` | MODIFY |
| Passphrase types | `be/internal/ssh/types.go` | APPEND |
| WebSocket proxy | `be/internal/ssh/proxy.go` | MODIFY (key dispatch) |
| ConnectOptions type | `fe/src/features/terminal/types.ts` | MODIFY |
| useSSHSession | `fe/src/features/terminal/useSSHSession.ts` | MODIFY |
| TerminalPane | `fe/src/features/terminal/TerminalPane.tsx` | MODIFY |
| PassphrasePrompt | `fe/src/features/terminal/PassphrasePrompt.tsx` | MODIFY |

**Why last:** This is the integration layer that ties backend key storage + frontend UI selection → actual SSH connection. Most complex because it involves the WebSocket protocol extension (passphrase round-trip). Depends on both the backend having key data and the frontend having the key selector UI.

**Dependencies:** Phase 1 (encrypted key storage + decryption) + Phase 2 (key selection in connection form, connection stores ssh_key_id)

---

## Patterns to Follow

### Pattern: Encryption Reuse
**What:** Use existing `config.Encrypt()`/`config.Decrypt()` for private key PEM storage, exactly like passwords.
**When:** Any sensitive data stored in SQLite (private keys, passphrases — but passphrases should NOT be stored)
**Why:** Same AES-256-GCM, same encryption key, same proven code. Don't build a second encryption system.

### Pattern: WebSocket Protocol Extension
**What:** Extend the existing text-frame JSON protocol with new message types (`passphrase-required`, `passphrase`).
**When:** Adding interactive flows to the WebSocket connection.
**Why:** The protocol already uses typed JSON messages (`connect`, `resize`, `connected`, `error`, `disconnected`). Extending with new types follows the established pattern.

### Pattern: State-Based Page Navigation
**What:** Use Zustand state (`sidebarPage: 'hosts' | 'keys'`) instead of react-router.
**When:** Simple 2-page navigation within a sidebar.
**Why:** No new dependency. App already uses Zustand for all state. react-router for 2 pages in a sidebar is overkill. The sidebar isn't a URL-addressable resource.

### Pattern: TanStack Query for API State
**What:** Follow the existing `useConnections` pattern for SSH key hooks.
**When:** Any new API data fetching.
**Why:** Consistent with existing code. Automatic caching, refetching, invalidation.

---

## Anti-Patterns to Avoid

### Anti-Pattern: Storing Passphrases
**What:** Saving the user's key passphrase in the database for auto-connect.
**Why bad:** Defeats the purpose of passphrase-protected keys. If someone gets DB access, they get both key and passphrase.
**Instead:** Store `has_passphrase = true` flag. Prompt user for passphrase each time they connect. This is how every SSH agent works.

### Anti-Pattern: Sending Private Key Material to Frontend
**What:** Returning decrypted private key PEM in API responses.
**Why bad:** Private key material should never leave the backend. The frontend doesn't need it — the Go backend makes the SSH connection.
**Instead:** API returns only metadata (name, type, fingerprint, has_passphrase). All key handling happens server-side in the WebSocket proxy.

### Anti-Pattern: Adding react-router for 2 Pages
**What:** Installing react-router for sidebar page switching.
**Why bad:** Adds ~50KB dependency for 2 states in a sidebar. No URL-addressable routes needed. Breaks the existing Zustand-only state management pattern.
**Instead:** `sidebarPage` state in Zustand. Conditional rendering in sidebar.

### Anti-Pattern: Tight Coupling in WebSocket Proxy
**What:** Adding DB queries and key parsing directly in the `HandleWebSocket` function which is already complex (252 lines).
**Why bad:** The proxy function handles WebSocket lifecycle, SSH connection, PTY, and bidirectional I/O. Adding key auth logic inline makes it unmaintainable.
**Instead:** Extract auth resolution into a helper function:
```go
func resolveAuthMethods(database *gorm.DB, cfg *config.Config, msg *ConnectMessage, wsWrite func(int, []byte) error) ([]ssh.AuthMethod, error)
```

---

## Scalability Considerations

| Concern | Current (v0.2) | With SSH Keys | Future |
|---------|----------------|---------------|--------|
| Key storage | N/A | ~2-10 keys per user, ~4KB each encrypted | Fine for single-user |
| Connection resolution | 1 DB query (connection) | 2 DB queries (connection + key) | Add caching if needed |
| WebSocket message types | 5 types | 7 types (+passphrase-required, +passphrase) | Stable protocol |
| Passphrase round-trip | N/A | 1 extra WebSocket exchange | Constant overhead |

No scalability concerns for the self-hosted, single-user target. The additional DB query for key resolution is negligible.

---

## Edge Cases to Handle

1. **Key deleted while connection references it:** Connection's `ssh_key_id` becomes a dangling reference. Backend should return a clear error ("SSH key not found, please update connection auth settings"). Frontend should show the key selector with a warning.

2. **Key with passphrase — wrong passphrase:** Backend sends error. Frontend should allow retry (re-show passphrase prompt) or cancel.

3. **Key with passphrase — connection retry:** Each reconnection should prompt for passphrase again. Don't cache passphrases in memory on the backend.

4. **Mixed auth in connection form:** Switching from "key" back to "password" should clear `ssh_key_id`. Switching from "password" to "key" should make password optional.

5. **Quick-connect with key:** User types `user@host` and selects a key from a dropdown. The `sshKeyId` needs to be part of the session state for reconnection to work.

6. **Export/import with key references:** Exported connections include `ssh_key_id` but keys are exported separately. Import should handle missing keys gracefully (set auth_method back to "password" or mark as broken).

---

## File Change Summary

| File | Action | Phase | LOC Impact |
|------|--------|-------|------------|
| `be/internal/db/models.go` | MODIFY | 1 | +25 lines (SSHKey model, Connection fields) |
| `be/internal/db/db.go` | MODIFY | 1 | +1 line (AutoMigrate SSHKey) |
| `be/internal/api/sshkeys.go` | NEW | 1 | ~120 lines (CRUD handler) |
| `be/internal/api/routes.go` | MODIFY | 1 | +6 lines (key routes) |
| `be/internal/api/connections.go` | MODIFY | 1 | +15 lines (auth_method handling) |
| `be/internal/ssh/types.go` | MODIFY | 3 | +15 lines (SSHKeyID, passphrase messages) |
| `be/internal/ssh/proxy.go` | MODIFY | 3 | +60 lines (key auth dispatch, passphrase flow) |
| `fe/src/lib/api.ts` | MODIFY | 2 | +30 lines (SSHKey type + API) |
| `fe/src/features/ssh-keys/hooks/useSSHKeys.ts` | NEW | 2 | ~50 lines |
| `fe/src/features/ssh-keys/components/SSHKeysPage.tsx` | NEW | 2 | ~200 lines |
| `fe/src/stores/app-store.ts` | MODIFY | 2 | +4 lines (sidebarPage) |
| `fe/src/App.tsx` | MODIFY | 2 | ~30 lines changed (page nav) |
| `fe/src/features/connections/components/HostsPage.tsx` | NEW | 2 | ~180 lines |
| `fe/src/features/connections/components/ConnectionForm.tsx` | MODIFY | 2 | +50 lines (auth toggle, key selector) |
| `fe/src/features/terminal/types.ts` | MODIFY | 3 | +2 lines (sshKeyId, passphrase-required status) |
| `fe/src/features/terminal/useSSHSession.ts` | MODIFY | 3 | +20 lines (passphrase handling, sshKeyId) |
| `fe/src/features/terminal/TerminalPane.tsx` | MODIFY | 3 | +15 lines (passphrase-required state) |
| `fe/src/features/terminal/PasswordPrompt.tsx` | MODIFY | 3 | +10 lines (passphrase variant) |

**Total estimated:** ~800 lines of new/modified code across 17 files (8 backend, 9 frontend).

---

## Sources

- `golang.org/x/crypto/ssh` — `ParsePrivateKey`, `ParsePrivateKeyWithPassphrase`, `PublicKeys`, `PassphraseMissingError` — verified via `go doc` (HIGH confidence)
- `be/internal/config/encryption.go` — AES-256-GCM encrypt/decrypt, reuse for key storage (HIGH confidence — codebase analysis)
- `be/internal/ssh/proxy.go` — Current WebSocket flow, auth insertion point at lines 110-124 (HIGH confidence — codebase analysis)
- `fe/src/App.tsx` — No router, Zustand-only state, sidebar is inline (HIGH confidence — codebase analysis)
- `fe/package.json` — No react-router dependency (HIGH confidence — verified)
