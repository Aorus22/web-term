# Roadmap: WebTerm

## Overview

WebTerm v0.3.0 adds SSH key-based authentication and a sidebar UI redesign. Three phases: backend key storage foundation → frontend navigation & key management UI → WebSocket key auth integration. Each phase is independently testable and builds on the previous.

## Milestones

- ✅ **v0.2.0 MVP** - Phases 1-4 (shipped 2026-04-28)
- 🚧 **v0.3.0 SSH Key Auth & UI Redesign** - Phases 5-7 (in progress)

## Phases

<details>
<summary>✅ v0.2.0 MVP (Phases 1-4) - SHIPPED 2026-04-28</summary>

### Phase 1: wterm Spike
**Goal**: Validate @wterm/react handles real SSH workloads
**Plans**: 2 plans

Plans:
- [x] 01-01: Terminal spike
- [x] 01-02: Go WebSocket SSH proxy

### Phase 2: Connection Management
**Goal**: Full CRUD connection management with encrypted storage
**Plans**: 2 plans

Plans:
- [x] 02-01: Backend REST API + SQLite
- [x] 02-02: Frontend connection UI

### Phase 3: SSH Terminal
**Goal**: Core SSH terminal with keyboard-interactive auth
**Plans**: 3 plans

Plans:
- [x] 03-01: SSH session hook
- [x] 03-02: Terminal pane with states
- [x] 03-03: Keyboard-interactive auth

### Phase 4: Multi-tab & Polish
**Goal**: Multi-tab sessions, keyboard shortcuts, theme, clean UI
**Plans**: 3 plans

Plans:
- [x] 04-01: Multi-tab sessions
- [x] 04-02: Keyboard shortcuts & theme
- [x] 04-03: UI polish & export/import

</details>

### 🚧 v0.3.0 SSH Key Auth & UI Redesign (In Progress)

**Milestone Goal:** SSH key-based authentication + sidebar redesigned into 2-page navigation (Hosts & SSH Keys)

- [x] **Phase 5: Backend SSH Key Storage** - SSH key model, encrypted storage, CRUD API, connection schema update
- [x] **Phase 6: Frontend UI & Navigation** - Sidebar tabs, hosts card page, SSH keys page, connection form auth toggle
- [ ] **Phase 7: WebSocket Key Auth Integration** - Key-based SSH auth, passphrase round-trip, backward compatibility

## Phase Details

### Phase 5: Backend SSH Key Storage
**Goal**: SSH keys can be securely stored, encrypted at rest, and managed via REST API — backend foundation with zero frontend impact
**Depends on**: Phase 4 (v0.2.0 complete)
**Requirements**: KEYS-01, KEYS-02, KEYS-03, KEYS-04, KEYS-05
**Success Criteria** (what must be TRUE):
1. User can upload a PEM private key via API and it is stored encrypted (AES-256-GCM with domain separation) in the database
2. User can list all keys with metadata (name, key type, fingerprint, passphrase status) via API — decrypted key material never exposed
3. User can rename and delete keys via API, with a warning response when deleting a key referenced by connections
4. Connection schema supports `auth_method` and `ssh_key_id` fields with safe migration (defaults to password, no data loss)
**Plans**: 2 plans

Plans:
- [x] 05-01-PLAN.md — SSH key model, encrypted storage, and CRUD API
- [x] 05-02-PLAN.md — Connection schema migration and encryption AAD context

### Phase 6: Frontend UI & Navigation
**Goal**: Users navigate between Hosts and SSH Keys pages, manage keys visually, and configure auth method per connection
**Depends on**: Phase 5
**Requirements**: UI-01, UI-02, UI-03, UI-04, UI-05, AUTH-01, AUTH-02
**Success Criteria** (what must be TRUE):
  1. Sidebar shows two tabs — clicking "Hosts" shows card-based host list, clicking "SSH Keys" shows key pool management page
  2. User can upload, view, rename, and delete SSH keys from the SSH Keys page (all operations reflect immediately in UI)
  3. Hosts page displays connections as cards with kebab menu actions (edit, delete, duplicate, connect)
  4. Connection add/edit form includes auth method toggle (password vs SSH key) and key selector dropdown when key is chosen
  5. Quick-connect bar supports initiating connections that use key-based auth
**Plans**: 3 plans
**UI hint**: yes

Plans:
- [x] 06-01-PLAN.md — Sidebar navigation + API layer + SSH Keys feature
- [x] 06-02-PLAN.md — Hosts card page with kebab menus
- [x] 06-03-PLAN.md — Connection form auth method and key selector

### Phase 7: WebSocket Key Auth Integration
**Goal**: Users can establish SSH connections using key-based authentication with passphrase support, while existing password connections work unchanged
**Depends on**: Phase 6
**Requirements**: AUTH-03, AUTH-04, AUTH-05
**Success Criteria** (what must be TRUE):
  1. User can connect to a server using SSH key authentication (key selected in connection settings)
  2. User is prompted for passphrase when connecting with an encrypted key — passphrase is sent securely over WebSocket and never stored
  3. Existing password-based connections continue to work without any changes
**Plans**: TBD

Plans:
- [ ] 07-01: WebSocket key-based SSH auth and passphrase round-trip
- [ ] 07-02: Reconnection handling and backward compatibility verification

## Progress

**Execution Order:**
Phases execute in numeric order: 5 → 6 → 7

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. wterm Spike | v0.2.0 | 2/2 | Complete | 2026-04-27 |
| 2. Connection Management | v0.2.0 | 2/2 | Complete | 2026-04-27 |
| 3. SSH Terminal | v0.2.0 | 3/3 | Complete | 2026-04-28 |
| 4. Multi-tab & Polish | v0.2.0 | 3/3 | Complete | 2026-04-28 |
| 5. Backend SSH Key Storage | v0.3.0 | 0/2 | Not started | - |
| 6. Frontend UI & Navigation | v0.3.0 | 0/3 | Not started | - |
| 7. WebSocket Key Auth Integration | v0.3.0 | 0/2 | Not started | - |
