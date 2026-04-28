# Requirements: WebTerm

**Defined:** 2026-04-28
**Core Value:** SSH ke server dari browser dengan pengalaman terminal yang smooth dan reliable

## v0.3.0 Requirements

Requirements for SSH Key Auth & UI Redesign milestone. Each maps to roadmap phases.

### Key Management

- [ ] **KEYS-01**: User can upload an existing private key file (PEM format) with a display name
- [ ] **KEYS-02**: User can view a list of all uploaded keys with metadata (name, key type, fingerprint, passphrase status)
- [ ] **KEYS-03**: User can rename a key's display name
- [ ] **KEYS-04**: User can delete a key from the pool (warned if connections reference it)
- [ ] **KEYS-05**: Private keys are encrypted at rest using AES-256-GCM with domain separation

### Connection Auth

- [ ] **AUTH-01**: User can select auth method (password or SSH key) when creating/editing a connection
- [ ] **AUTH-02**: User can select which key from the pool to use for a connection
- [ ] **AUTH-03**: User can connect to a server using SSH key authentication
- [ ] **AUTH-04**: User is prompted for passphrase when connecting with an encrypted key
- [ ] **AUTH-05**: Existing password-based connections continue to work unchanged

### UI Navigation

- [ ] **UI-01**: Sidebar shows 2 tabs: Hosts and SSH Keys
- [ ] **UI-02**: User sees hosts displayed as cards on the Hosts page with kebab menu (edit/delete/duplicate/connect)
- [ ] **UI-03**: User sees key pool management on the SSH Keys page (upload, list, rename, delete)
- [ ] **UI-04**: Connection add/edit form uses right-side sheet with auth method toggle and key selector
- [ ] **UI-05**: Quick-connect bar supports key-based connections

## v0.4.0 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Key Generation

- **KEYGEN-01**: User can generate new keypairs in-browser (RSA/Ed25519/ECDSA)
- **KEYGEN-02**: User can deploy public key to remote server

### Advanced

- **ADV-01**: User can export private keys from the pool
- **ADV-02**: Connection export/import includes SSH key references

## Out of Scope

| Feature | Reason |
|---------|--------|
| PuTTY PPK key format | PEM-only for v0.3.0, PPK conversion deferred |
| Key generation in-browser | Upload-only simpler for v0.3.0 |
| SSH agent forwarding | Security risk, complex WebSocket integration |
| Public key deployment to server | Out of scope for self-hosted single-user tool |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| KEYS-01 | Phase 5 | Pending |
| KEYS-02 | Phase 5 | Pending |
| KEYS-03 | Phase 5 | Pending |
| KEYS-04 | Phase 5 | Pending |
| KEYS-05 | Phase 5 | Pending |
| AUTH-01 | Phase 6 | Pending |
| AUTH-02 | Phase 6 | Pending |
| AUTH-03 | Phase 7 | Pending |
| AUTH-04 | Phase 7 | Pending |
| AUTH-05 | Phase 7 | Pending |
| UI-01 | Phase 6 | Pending |
| UI-02 | Phase 6 | Pending |
| UI-03 | Phase 6 | Pending |
| UI-04 | Phase 6 | Pending |
| UI-05 | Phase 6 | Pending |

**Coverage:**
- v0.3.0 requirements: 15 total
- Mapped to phases: 15
- Unmapped: 0

---
*Requirements defined: 2026-04-28*
*Last updated: 2026-04-28 after roadmap creation (v0.3.0)*
