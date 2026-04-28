# Domain Pitfalls

**Domain:** Adding SSH key authentication and 2-page UI navigation to an existing web-based SSH terminal (WebTerm v0.3.0)
**Researched:** 2026-04-28
**Codebase analyzed:** 53 source files, ~5,007 LOC (Go backend + React frontend)
**Confidence:** HIGH (codebase fully analyzed, Go x/crypto/ssh v0.50.0 API verified via pkg.go.dev)

---

## Critical Pitfalls

Mistakes that cause rewrites, data loss, or security breaches.

### Pitfall 1: Passphrase-Encrypted Key Silent Failure on ParsePrivateKey

**What goes wrong:** Calling `ssh.ParsePrivateKey()` on an encrypted key returns a `*PassphraseMissingError` — but developers often treat it as a generic parse error ("invalid key format") and show the user "key is invalid" instead of "this key needs a passphrase."

**Why it happens:** `ParsePrivateKey` returns `error` interface. The `PassphraseMissingError` type is not widely known and requires a type assertion to detect. The error message string doesn't explicitly say "try ParsePrivateKeyWithPassphrase" — it just says the key needs a passphrase.

**Consequences:** Users with encrypted keys (very common — `ssh-keygen -p` encrypts by default) upload a valid key but get told it's broken. They re-generate keys, try different formats, or abandon the feature.

**Prevention:**
1. Always try `ssh.ParsePrivateKey()` first
2. Check `errors.As(err, &ssh.PassphraseMissingError{})` — if true, the key is valid but encrypted
3. Only then try `ssh.ParsePrivateKeyWithPassphrase(keyBytes, []byte(passphrase))`
4. If that fails with `x509.IncorrectPasswordError`, the passphrase is wrong — tell the user explicitly
5. Store whether a key is passphrase-protected in the database (`has_passphrase bool`) so the UI knows to prompt before connecting

**Detection:** Test with an encrypted Ed25519 key and verify the error path produces a passphrase prompt, not a "bad key" error.

**Codebase impact:** `be/internal/ssh/proxy.go` line 110-124 currently hardcodes `ssh.Password()` auth. The new key-based auth path must handle the passphrase flow before constructing `ssh.ClientConfig.Auth`.

```go
// Correct pattern:
signer, err := ssh.ParsePrivateKey(keyBytes)
if err != nil {
    var passErr *ssh.PassphraseMissingError
    if errors.As(err, &passErr) {
        // Key is encrypted — need passphrase from user
        signer, err = ssh.ParsePrivateKeyWithPassphrase(keyBytes, []byte(passphrase))
        if err != nil {
            // Check for x509.IncorrectPasswordError
            return fmt.Errorf("wrong passphrase")
        }
    } else {
        return fmt.Errorf("invalid private key: %w", err)
    }
}
```

### Pitfall 2: Private Key Material Leaked via Logs, Errors, or API Responses

**What goes wrong:** The decrypted private key PEM bytes end up in log output, error messages, API response bodies, or the frontend JavaScript console.

**Why it happens:**
- `log.Printf("Key parsed: %s", keyBytes)` — accidentally logs key material
- Error wrapping includes key bytes: `fmt.Errorf("failed to parse key %s: %w", keyBytes, err)`
- API endpoints return the decrypted key in GET responses (like `GetConnection` currently returns decrypted password)
- Frontend `console.error()` dumps the full request body including key data

**Consequences:** Private SSH keys in logs = complete server access compromise. Keys are long-lived credentials — a leaked key means every server that key accesses is compromised.

**Prevention:**
1. **Never log the key PEM bytes** — only log key metadata (type, fingerprint, ID, length)
2. **API must never return decrypted private keys** — the `GetSSHKey` endpoint should return metadata (name, type, fingerprint, has_passphrase) but NEVER the decrypted key material
3. **Use `[]byte` not `string` for key material** — `[]byte` can be zeroed after use; Go strings are immutable and linger in memory
4. **Strip key from error messages** — errors should reference key ID, not key content
5. **Zero key bytes after use** — after `ssh.ParsePrivateKey()` succeeds, zero the input `[]byte` slice: `for i := range keyBytes { keyBytes[i] = 0 }`

**Codebase impact:** The existing pattern in `connections.go:46-50` decrypts and returns passwords via API. The SSH key endpoints must NOT follow this pattern. The `Encrypted` field uses `json:"-"` tag correctly — replicate this for keys.

```go
// SSHKey model — correct pattern:
type SSHKey struct {
    ID              string `json:"id"`
    Name            string `json:"name"`
    KeyType         string `json:"key_type"`         // "RSA", "Ed25519", "ECDSA"
    Fingerprint     string `json:"fingerprint"`       // SHA256:xxx — safe to display
    HasPassphrase   bool   `json:"has_passphrase"`
    EncryptedKey    string `json:"-" gorm:"column:private_key"` // NEVER in JSON
    PublicKeyString string `json:"public_key"`        // OK to show
    // ...
}
```

### Pitfall 3: Reusing the Same AES-256-GCM Key for Passwords and Private Keys

**What goes wrong:** The existing `config.EncryptionKey` (32-byte key from `WEBTERM_ENCRYPTION_KEY` env var) is used for password encryption. If the same key is reused for private key encryption without domain separation, a ciphertext confusion attack becomes possible — an attacker could swap encrypted password and encrypted key fields.

**Why it happens:** It seems DRY — "we already have an encryption key, just reuse it." But AES-GCM with the same key and different data types but no type-tagging in the ciphertext creates confusion vulnerabilities.

**Consequences:** If the database is dumped, encrypted passwords could potentially be substituted into the key field or vice versa during a controlled attack.

**Prevention:**
1. Use **different AEAD nonce spaces** per data type — add a context string to the GCM additional data (AAD) parameter
2. The simplest fix: extend the existing `Encrypt`/`Decrypt` functions to accept an optional `context` string passed as GCM additional data
3. Or use separate derived keys: `subkey_password = HKDF(master, "passwords")`, `subkey_keys = HKDF(master, "ssh-keys")`

**Codebase impact:** `be/internal/config/encryption.go` uses `gcm.Seal(nonce, nonce, plaintext, nil)` — the `nil` additional data parameter should carry a type context.

```go
// Fix: add context parameter
func EncryptWithContext(plaintext string, key []byte, context string) (string, error) {
    // ... same as current Encrypt but:
    ciphertext := gcm.Seal(nonce, nonce, []byte(plaintext), []byte(context))
    // ...
}

// Usage:
encryptedPassword, _ := EncryptWithContext(password, key, "webterm:password")
encryptedKey, _ := EncryptWithContext(keyPEM, key, "webterm:ssh-private-key")
```

### Pitfall 4: WebSocket Protocol Breaking Change During Auth Method Migration

**What goes wrong:** The `ConnectMessage` struct in `types.go` currently has `Password` field. Adding key-based auth means adding `KeyID` and `Passphrase` fields. If the frontend sends the new fields but the backend hasn't been updated (or vice versa), connections fail silently or with confusing errors.

**Why it happens:** The WebSocket connect message has no version field. Frontend and backend are deployed independently (Go binary vs Vite build). During development, both change rapidly. The JSON unmarshaling is lenient (unknown fields ignored), but the auth logic assumes specific field combinations.

**Consequences:** A half-deployed update where frontend sends `key_id` but backend ignores it and tries `ssh.Password("")` — connection hangs for 10 seconds then times out with no useful error.

**Prevention:**
1. **Add an `auth_method` field** to `ConnectMessage` — explicit `"password"` or `"key"` — no guessing
2. **Backend must validate the combination** — if `auth_method` is `"key"` but `key_id` is empty, return error immediately
3. **Keep backward compatibility** — if `auth_method` is missing, infer from available fields (current behavior)
4. **Test the transition** — deploy backend first, then frontend (backend accepts both old and new format)

**Codebase impact:** `be/internal/ssh/types.go` ConnectMessage struct needs new fields. `be/internal/ssh/proxy.go` lines 76-95 need to handle key-based credential resolution alongside password-based.

```go
type ConnectMessage struct {
    Type         string `json:"type"`
    Host         string `json:"host"`
    Port         int    `json:"port"`
    User         string `json:"user"`
    Password     string `json:"password,omitempty"`
    ConnectionID string `json:"connection_id,omitempty"`
    AuthMethod   string `json:"auth_method,omitempty"` // NEW: "password" | "key"
    KeyID        string `json:"key_id,omitempty"`      // NEW: SSH key ID
    Passphrase   string `json:"passphrase,omitempty"`  // NEW: for encrypted keys
    Rows         int    `json:"rows,omitempty"`
    Cols         int    `json:"cols,omitempty"`
}
```

### Pitfall 5: Database Migration Destroys Existing Connections

**What goes wrong:** Adding an `AuthMethod` column and an optional `SSHKeyID` foreign key to the `Connection` model. If the migration sets a non-null default or requires a foreign key constraint, existing connection records become invalid.

**Why it happens:** GORM's `AutoMigrate` adds columns but doesn't set defaults for existing rows. If the new `auth_method` column has a NOT NULL constraint without a default, SQLite will reject the migration. If `ssh_key_id` has a foreign key constraint and some key gets deleted, cascading deletes could wipe connections.

**Consequences:** Existing users' saved connections disappear on upgrade, or the application fails to start with a migration error.

**Prevention:**
1. `auth_method` column must default to `"password"` for existing rows
2. `ssh_key_id` must be nullable — existing connections don't use keys
3. Do NOT use foreign key cascading delete — soft-link by ID, check existence at query time
4. Run `AutoMigrate` BEFORE any query — ensure `db.go:Init()` includes new models
5. Test migration against a populated SQLite database (not just empty)

**Codebase impact:** `be/internal/db/models.go` needs `AuthMethod` and `SSHKeyID` fields on `Connection`. `be/internal/db/db.go:25` `AutoMigrate` must include `&SSHKey{}`.

---

## Moderate Pitfalls

### Pitfall 6: Key Format Support Gaps (PuTTY .ppk, PKCS#8 DER)

**What goes wrong:** Users try to upload PuTTY-format keys (`.ppk`) or binary DER-encoded keys. `ssh.ParsePrivateKey()` only supports PEM-encoded keys (PKCS#1 RSA, PKCS#8, OpenSSH format). PuTTY keys fail silently.

**Prevention:**
1. Document supported formats clearly: "OpenSSH, PEM-encoded RSA/Ed25519/ECDSA"
2. Validate the uploaded key immediately on upload — try `ssh.ParsePrivateKey()` and report the specific error
3. If the PEM block type is "RSA PRIVATE KEY" but parsing fails, suggest converting with `ssh-keygen -i`
4. Consider adding `.ppk` → PEM conversion on the backend using a library, or reject with a helpful error message
5. Set a reasonable max key size (e.g., 1MB) — some users paste entire `authorized_keys` files

### Pitfall 7: Sidebar Removal Breaks Session Connection Flow

**What goes wrong:** The current sidebar (`ConnectionList`) is the primary way to open saved connections. The redesign moves this to a card-based "Hosts" page. During the transition, the `ConnectionList` component is deleted but the new Hosts page isn't fully wired — users can't connect to anything.

**Prevention:**
1. Build the Hosts page FIRST, wire it completely, THEN remove the sidebar
2. The `ConnectionList` component in the sidebar and the new `HostsPage` card grid can coexist temporarily
3. Ensure the Hosts page card `onClick` handler calls the same `handleConnect` pattern as `NewTabView.tsx:23-37`
4. Keep `QuickConnect` accessible from the header/tab bar, not just the sidebar

### Pitfall 8: Sidebar State Lost During Page Navigation

**What goes wrong:** Moving from a single-page sidebar to a 2-page router (Hosts / SSH Keys). When user navigates between pages, the current sidebar state (scroll position, selected tags, form open/close) is lost because the component unmounts.

**Prevention:**
1. Use conditional rendering (not router-based unmounting) for the sidebar pages — both pages render but only one is visible
2. Or use Zustand store to persist page-specific state (selected filter, scroll position)
3. The sidebar page navigation should be tab-like — CSS `hidden`/`visible`, not component mount/unmount

**Codebase impact:** `fe/src/App.tsx` currently renders `ConnectionList` directly in the sidebar. The new pattern should keep both `HostsPage` and `SSHKeysPage` conditionally rendered based on a `sidebarPage` state in the store.

### Pitfall 9: Passphrase Not Available During Reconnection

**What goes wrong:** The `ReconnectOverlay` in `fe/src/features/terminal/ReconnectOverlay.tsx` calls `onReconnect` which re-triggers the WebSocket connect flow. For key-auth connections with encrypted keys, the passphrase is needed again but was never stored. The reconnect fails with "passphrase required" — but there's no UI to prompt for it.

**Prevention:**
1. Detect that the connection uses an encrypted key before reconnect
2. Show the passphrase prompt BEFORE attempting WebSocket reconnection
3. Do NOT store passphrases — always re-prompt on reconnect
4. Add `needsPassphrase` flag to the `SSHSession` type so the reconnect overlay knows to prompt

**Codebase impact:** `fe/src/features/terminal/types.ts` needs a field like `authMethod: 'password' | 'key'` and `keyHasPassphrase?: boolean` on `SSHSession`. The reconnect flow in `TerminalPane.tsx` must check these before calling `connect()`.

### Pitfall 10: Export/Import Doesn't Handle SSH Keys

**What goes wrong:** The existing `ExportConnections` endpoint in `be/internal/api/export.go` serializes `Connection` records. After adding SSH keys, the export contains `ssh_key_id` references but not the actual keys. Importing on another instance fails — connections reference key IDs that don't exist.

**Prevention:**
1. Extend export to include SSH keys alongside connections
2. Import must create keys first, then map old key IDs to new IDs before creating connections
3. Export format should be versioned: `{ "version": 2, "connections": [...], "ssh_keys": [...] }`
4. Maintain backward compatibility — importing a v1 export (no keys) still works

### Pitfall 11: Frontend State Gets Out of Sync with Backend Key Store

**What goes wrong:** User uploads a key (POST), but the React Query cache isn't invalidated. The SSH Keys page still shows the old list. User tries to use the new key in a connection form, but the dropdown doesn't include it.

**Prevention:**
1. After key upload, invalidate the React Query cache for the keys list: `queryClient.invalidateQueries({ queryKey: ['ssh-keys'] })`
2. The connection form's key dropdown should use `useQuery` (not one-time fetch) so it auto-updates
3. Follow the same pattern as existing `useConnections` hook which uses `useQuery` with proper cache keys

---

## Minor Pitfalls

### Pitfall 12: Large Key File Upload Causes Memory Pressure

**What goes wrong:** Users upload very large files (not just keys — some paste entire certificates or accidentally upload wrong files). The backend reads the entire body into memory, encrypts it, and stores it in SQLite.

**Prevention:**
1. Set `http.MaxBytesReader(w, r.Body, 1<<20)` (1MB max) on key upload endpoint
2. Validate PEM structure before encryption — reject non-PEM data early
3. Private keys should never exceed ~20KB even for 16K-bit RSA keys

### Pitfall 13: Key Fingerprint Collision in Display

**What goes wrong:** Two keys with the same name but different content confuse users. The SSH keys page shows only the name.

**Prevention:**
1. Always display the fingerprint alongside the key name (e.g., "Production Key (SHA256:abc...xyz)")
2. Use `ssh.FingerprintSHA256(signer.PublicKey())` on upload to generate and store the fingerprint
3. Reject duplicate keys (same fingerprint) with a clear message

### Pitfall 14: Keyboard Shortcuts Conflict with New Page Navigation

**What goes wrong:** Adding keyboard navigation for the sidebar pages (e.g., Ctrl+1 for Hosts, Ctrl+2 for SSH Keys) conflicts with existing shortcuts in `use-keyboard-shortcuts.ts`.

**Prevention:**
1. Check the existing shortcut map before assigning new ones
2. Current shortcuts: Ctrl+T (new tab), Ctrl+W (close tab), Ctrl+Tab/Shift+Tab (next/prev tab)
3. Sidebar page nav can use simpler keys or avoid shortcuts entirely — it's infrequent navigation

### Pitfall 15: Connection Form Auth Method Toggle Resets Form State

**What goes wrong:** The `ConnectionForm` component in `fe/src/features/connections/components/ConnectionForm.tsx` uses a single `formData` state. Toggling from "password" to "key" auth method clears the password field but also might accidentally clear other fields due to state management bugs.

**Prevention:**
1. Use separate state sections: `authMethod: 'password' | 'key'`, `passwordFields: {...}`, `keyFields: {...}`
2. When toggling auth method, only show/hide the relevant fields — don't clear unrelated fields
3. Keep password value in state even when hidden (in case user toggles back) — but don't send it if auth is "key"

### Pitfall 16: Theme Toggle Position Shifts with New Navigation

**What goes wrong:** The header bar layout changes with the new sidebar navigation. The `ThemeToggle` component currently sits in the header `flex` row. Adding page navigation icons shifts its position.

**Prevention:**
1. Keep `ThemeToggle` at `ml-auto` or fixed right position in the header
2. Test both sidebar open/collapsed states with the new navigation elements
3. The header structure: `[sidebar-toggle] [page-nav-icons?] [tabs...] [theme-toggle]`

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation | Phase to Address |
|-------------|---------------|------------|------------------|
| SSH key upload API | Key material in error responses (Pitfall 2) | Sanitize all error messages, never include key bytes | Backend SSH key CRUD phase |
| Key storage encryption | Same encryption key for passwords and keys (Pitfall 3) | Add AAD context to AES-GCM encryption | Backend SSH key storage phase |
| Key parsing | PassphraseMissingError mishandled (Pitfall 1) | Try ParsePrivateKey, catch PassphraseMissingError, use ParsePrivateKeyWithPassphrase | Backend SSH auth phase |
| WebSocket connect message | Breaking protocol change (Pitfall 4) | Add `auth_method` field, keep backward compatible | Backend SSH auth phase |
| DB migration | Existing connections broken (Pitfall 5) | Default `auth_method="password"`, nullable `ssh_key_id` | Backend DB model phase |
| Sidebar → Hosts page | Connection flow broken during transition (Pitfall 7) | Build new first, remove old after verification | Frontend Hosts page phase |
| Page navigation | State lost on page switch (Pitfall 8) | Conditional rendering, not route-based unmounting | Frontend navigation phase |
| Passphrase prompt on connect | No passphrase prompt during reconnect (Pitfall 9) | Add `keyHasPassphrase` to session type, prompt before reconnect | Frontend terminal phase |
| Connection form auth toggle | Form state reset on toggle (Pitfall 15) | Separate auth state sections, don't clear hidden fields | Frontend connection form phase |
| Export/import with keys | Key references without key data (Pitfall 10) | Version export format, include keys, map IDs on import | Backend export phase |
| Key format validation | Rejection of valid PuTTY keys (Pitfall 6) | Clear error messages, suggest conversion command | Backend key upload phase |

## Sources

- Go x/crypto/ssh v0.50.0 package documentation — pkg.go.dev (HIGH confidence)
- WebTerm codebase analysis — 11 Go files, 23+ TypeScript files read directly (HIGH confidence)
- SSH protocol key format support: RSA (PKCS#1), ECDSA, Ed25519, DSA, PKCS#8, OpenSSH — confirmed via Go docs
- `PassphraseMissingError` type with `PublicKey` field — confirmed in Go x/crypto/ssh v0.50.0
- `ParseRawPrivateKeyWithPassphrase` returns `x509.IncorrectPasswordError` on wrong passphrase — confirmed via Go docs
