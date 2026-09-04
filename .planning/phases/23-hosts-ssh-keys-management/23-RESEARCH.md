# Phase 23 Research: Hosts & SSH Keys Management

## Overview
Phase 23 brings full connection and SSH key management parity to the GPUI desktop client, speaking the existing Go backend REST API (`/api/connections`, `/api/connections/export`, `/api/connections/import`, `/api/keys`).

## 1. Backend REST API Analysis

### Connections API (`be/internal/api/connections.go`, `be/internal/api/export.go`)
1. **GET `/api/connections`**:
   - Returns JSON array of `db.Connection` (passwords omitted).
   - Fields: `id`, `label`, `host`, `port`, `username`, `tags` (array of strings), `auth_method` (`"password"` | `"key"`), `ssh_key_id` (nullable string), `created_at`, `updated_at`.
2. **GET `/api/connections/{id}`**:
   - Returns single connection with decrypted password for edit forms.
3. **POST `/api/connections`**:
   - Body: `{"label": "...", "host": "...", "port": 22, "username": "...", "password": "...", "tags": [...], "auth_method": "password"|"key", "ssh_key_id": "..."}`
   - Status 201 on success; returns created `db.Connection`.
4. **PUT `/api/connections/{id}`**:
   - Body: same fields as create. Updates existing connection.
5. **DELETE `/api/connections/{id}`**:
   - Returns Status 204 No Content.
6. **GET `/api/connections/export`**:
   - Returns all connections as a JSON array attachment.
7. **POST `/api/connections/import`**:
   - Body: JSON array of connections.
   - Response: `{"imported": int, "skipped": int}`.

### SSH Keys API (`be/internal/api/keys.go`)
1. **GET `/api/keys`**:
   - Returns JSON array of `db.SSHKey`.
   - Fields: `id`, `name`, `key_type` (`"RSA"`, `"Ed25519"`, `"ECDSA"`, `"Unknown"`), `fingerprint`, `created_at`, `updated_at`.
2. **GET `/api/keys/{id}`**:
   - Returns single `db.SSHKey`.
3. **POST `/api/keys`**:
   - Body: `{"name": "...", "key_base64": "..."}` where `key_base64` is base64-encoded raw private key bytes (PEM format).
   - Backend parses key, computes fingerprint, and encrypts with AES-256 before saving.
   - Status 201 on success; returns created `db.SSHKey`.
4. **DELETE `/api/keys/{id}`**:
   - Returns Status 204 No Content.

## 2. Desktop GPUI Architecture

### Crate Dependencies & Boundaries
- `crates/backend-client`:
  - Add DTOs: `SshKey`, `CreateKeyRequest`, `CreateConnectionRequest`, `UpdateConnectionRequest`, `ImportResult`.
  - Add methods to `BackendClient`:
    - `get_connection`, `create_connection`, `update_connection`, `delete_connection`, `export_connections`, `import_connections`.
    - `list_keys`, `get_key`, `create_key`, `delete_key`.
  - Add integration contract tests verifying all endpoints against a mock server.

- `crates/webterm`:
  - `AppState`:
    - `connections: Vec<Connection>`: cached connection list.
    - `ssh_keys: Vec<SshKey>`: cached SSH keys list.
    - `search_query: String`: filter text for host cards.
    - `selected_tag: Option<String>`: tag filter.
    - `show_connection_modal: Option<ConnectionModalState>`: Create or Edit state.
    - `show_key_modal: bool`: Add SSH key modal state.
    - `show_passphrase_modal: Option<PassphrasePromptState>`: Modal when connecting with encrypted key.
    - `passphrase_cache: HashMap<String, String>`: in-memory session cache mapping `ssh_key_id -> passphrase`.
  - Views:
    - `views/hosts.rs`: Host cards grid, search input, tag chips, "+ Add Host", "Export", "Import".
    - `views/connection_modal.rs`: Add/Edit connection form modal.
    - `views/keys.rs`: SSH Keys pool view with cards, fingerprint, type badge, "+ Add Key", and delete.
    - `views/key_modal.rs`: Add SSH Key modal (name, private key text/file).
    - `views/passphrase_modal.rs`: Session passphrase prompt modal.

## 3. Plan Phasing
- **23-01**: `webterm-backend-client` API surface & contract tests (Connection CRUD, Export/Import, Keys CRUD).
- **23-02**: Hosts View — Connection cards grid/list, tag filtering, real-time search, quick-connect (`HOSTS-02`, `HOSTS-03`).
- **23-03**: Connection Modal (Create/Edit) & Import/Export JSON dialog (`HOSTS-01`, `HOSTS-04`, `KEYS-03`).
- **23-04**: SSH Keys View, Key Upload Modal, and Session Passphrase Prompt (`KEYS-01`, `KEYS-02`).
