# Phase 5: Backend SSH Key Storage - Context

**Gathered:** 2026-04-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Backend-only phase delivering SSH key encrypted storage and CRUD API. Users can upload PEM private keys which are stored encrypted at rest (AES-256-GCM), list/manage keys via REST API, and connections gain `auth_method` and `ssh_key_id` fields for future key-based auth. Zero frontend impact in this phase.

</domain>

<decisions>
## Implementation Decisions

### SSH Key Model Structure
- **D-01:** SSHKey model includes: id, name, encrypted_key, fingerprint, key_type, has_passphrase, created_at, updated_at
- **D-02:** Metadata (fingerprint, key_type, has_passphrase) extracted and stored at upload time for display without decryption
- **D-03:** Key types stored as standard names: "RSA", "Ed25519", "ECDSA" (not Go crypto library type strings)
- **D-04:** Parse full PEM on upload - extract public_key, comment, and key_size where available

### Encryption Domain Separation
- **D-05:** Use AAD domain separation: "webterm:password:v1" for passwords, "webterm:sshkey:v1" for SSH keys
- **D-06:** Same master encryption key, different AAD context strings passed to GCM for domain separation
- **D-07:** Do NOT use separate derived keys - AAD separation sufficient for v0.3.0

### API Design
- **D-08:** Flat REST endpoint structure: /api/keys (consistent with /api/connections)
- **D-09:** Upload via POST /api/keys with base64-encoded key in JSON body: {"name": "...", "key_base64": "..."}
- **D-10:** List keys returns full metadata only - never returns encrypted_key blob
- **D-11:** Response structure: id, name, fingerprint, key_type, has_passphrase, created_at, updated_at

### Connection Schema Migration
- **D-12:** auth_method field as string enum: "password" | "key"
- **D-13:** ssh_key_id field as foreign key to ssh_keys.id
- **D-14:** Foreign key with constraint, ON DELETE SET NULL (clear ssh_key_id)
- **D-15:** Default auth_method to "password" for all existing connections - backward compatible
- **D-16:** Migration uses GORM AutoMigrate with field additions - no data loss

### Key Validation on Upload
- **D-17:** Full parse validation using golang.org/x/crypto/ssh.ParseRawPrivateKey
- **D-18:** Validation detects key type, extracts fingerprint, and detects if passphrase-protected
- **D-19:** Reject malformed keys with 400 Bad Request and clear error message

### the agent's Discretion
- Fingerprint algorithm choice (MD5 vs SHA256) - agent decides during implementation
- Exact JSON response structure for errors - agent decides based on existing API patterns
- GORM model tags and constraints - agent decides based on existing Connection model pattern
- SQL index design for ssh_keys table - agent decides
- Transaction handling for upload+encryption - agent decides
</decisions>

<specifics>
## Specific Ideas

- "Base64 in JSON keeps API simple and testable with curl - just encode the PEM file"
- "Never expose encrypted key material in list - only metadata needed for UI"
- "Same as existing encryption pattern but with different AAD - keep it consistent"

</specifics>

<canonical_refs>
## Canonical References

### Requirements
- `.planning/REQUIREMENTS.md` § "Key Management" (KEYS-01 through KEYS-05)

### Encryption Pattern
- `be/internal/config/encryption.go` - Existing AES-256-GCM implementation to extend
- `be/internal/db/models.go` - Existing Connection model pattern to follow
- `be/internal/db/db.go` - Existing GORM migration pattern

### API Pattern
- `be/internal/api/connections.go` - CRUD endpoint structure to replicate
- `be/internal/api/routes.go` - Route registration pattern

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **encryption.go**: Extend Encrypt/Decrypt functions with AAD parameter support (currently uses nil AAD)
- **models.go**: Follow Connection struct pattern for SSHKey - UUID primary keys, BeforeCreate hooks
- **db.go**: AutoMigrate pattern for adding new table

### Established Patterns
- **GORM**: Model with `gorm:"primaryKey;type:varchar(36)"` for UUIDs
- **API**: Handler functions in connections.go pattern - JSON request/response
- **Encryption**: Base64-encoded ciphertext storage in database

### Integration Points
- New table `ssh_keys` alongside existing `connections` table
- New fields `auth_method` and `ssh_key_id` added to Connection model
- Foreign key relationship connections.ssh_key_id → ssh_keys.id

</code_context>

<deferred>
## Deferred Ideas

- SSH key export/download (encrypted blob) - deferred to v0.4.0
- PuTTY PPK key format support - deferred to backlog
- Key generation in-browser - explicitly out of scope per REQUIREMENTS.md
- Public key deployment to remote server - out of scope

</deferred>

---

*Phase: 05-backend-ssh-key-storage*
*Context gathered: 2026-04-28*
