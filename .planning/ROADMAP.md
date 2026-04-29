# Roadmap: WebTerm

## Overview

WebTerm v0.3.0 adds SSH key-based authentication and a sidebar UI redesign. Three phases: backend key storage foundation → frontend navigation & key management UI → WebSocket key auth integration. Each phase is independently testable and builds on the previous.

## Milestones

- ✅ **v0.2.0 MVP** - Phases 1-4 (shipped 2026-04-28)
- ✅ **v0.3.0 SSH Key Auth & UI Redesign** - Phases 5-8 (shipped 2026-04-28)

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

### ✅ v0.3.0 SSH Key Auth & UI Redesign (Shipped 2026-04-28)

**Milestone Goal:** SSH key-based authentication + sidebar redesigned into 2-page navigation (Hosts & SSH Keys) + port forwarding

- [x] **Phase 5: Backend SSH Key Storage** - SSH key model, encrypted storage, CRUD API, connection schema update
- [x] **Phase 6: Frontend UI & Navigation** - Sidebar tabs, hosts card page, SSH keys page, connection form auth toggle
- [x] **Phase 7: WebSocket Key Auth Integration** - Key-based SSH auth, passphrase round-trip, backward compatibility
- [x] **Phase 8: Port Forwarding** - SSH local port forwarding, sheet UI for managing forwards, port conflict detection

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
**Plans:** 2 plans

Plans:
- [x] 07-01-PLAN.md — Backend key auth: extend ConnectMessage + implement ssh.Signer auth in proxy
- [x] 07-02-PLAN.md — Frontend key auth: PassphrasePrompt, HostsPage flow, passphrase caching & reconnect

### Phase 8: Port Forwarding
**Goal**: Users can create SSH local port forwarding tunnels that bind a remote host port to a local port, managed via sheet UI with port conflict detection
**Depends on**: Phase 7
**Success Criteria** (what must be TRUE):
  1. User can create a port forwarding rule by selecting an SSH connection, specifying a remote target host:port, and a local bind port via a sheet UI
  2. Active port forwards are listed and can be stopped/removed from the UI
  3. When saving a port forward with a local port already in use, the user sees a toast error and the forward is not created
  4. The Go proxy establishes SSH local port forwarding tunnels and exposes them on the specified localhost ports
**Plans:** 2 plans

Plans:
- [x] 08-01-PLAN.md — Backend: PortForward model, CRUD API, tunnel manager with SSH dial
- [x] 08-02-PLAN.md — Frontend: Port Forwards page, form sheet, sidebar integration, port conflict toast

### Phase 9: Settings Page
**Goal**: Settings page with justify-between layout (label left, value right), dark/light mode switcher, terminal emulator type selection (xterm, xterm-256-color, vanilla), and terminal color theme presets (Monokai, Solarized, Dracula, Nord, etc. — like Termius)
**Depends on**: Phase 4 (v0.2.0 complete)
**Success Criteria** (what must be TRUE):
1. Settings page displays options in justify-between layout — label on the left, control on the right
2. Dark mode / light mode switcher toggles app theme and persists preference
3. Terminal emulator type selector offers options like xterm, xterm-256-color, vanilla (like Termius)
4. Terminal color theme picker shows preset themes (Monokai, Solarized Dark/Light, Dracula, Nord, GitHub, One Dark, etc.) with live preview
5. Selected terminal theme and emulator type are applied to the terminal component on connection
**Plans**: 3 plans

Plans:
- [x] 09-01-PLAN.md — Backend settings API (key-value model + handlers) and dynamic terminal type in SSH proxy
- [x] 09-02-PLAN.md — Frontend settings infrastructure, SettingsPage UI, sidebar integration, ThemeToggle removal
- [x] 09-03-PLAN.md — Terminal settings integration (theme, font, cursor, scrollback, terminal type)

### Phase 10: Tab Plus Button Popover
**Goal**: Add a plus button popover on the tab bar with two options — "Duplicate" (opens same connection at same working directory) and "New Connection" (opens a new connection tab like current new tab behavior)
**Depends on**: Phase 4 (multi-tab sessions)
**Success Criteria** (what must be TRUE):
1. A plus (+) button is visible on the tab bar; clicking it opens a popover with "Duplicate" and "New Connection" options
2. "Duplicate" creates a new tab connected to the same host AND navigates to the same working directory as the active tab
3. "New Connection" opens a new connection tab with behavior identical to the current new tab flow
4. Popover closes after selecting an option or clicking outside
**Plans:** 2 plans

Plans:
- [ ] 10-01-PLAN.md — Backend cwd support: ConnectMessage.Cwd field, get-cwd WebSocket handler with stdout interception
- [ ] 10-02-PLAN.md — Frontend plus button popover with Duplicate (same connection + directory) and New Connection options

## Progress

**Execution Order:**
Phases execute in numeric order: 5 → 6 → 7 → 8 → 9

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. wterm Spike | v0.2.0 | 2/2 | Complete | 2026-04-27 |
| 2. Connection Management | v0.2.0 | 2/2 | Complete | 2026-04-27 |
| 3. SSH Terminal | v0.2.0 | 3/3 | Complete | 2026-04-28 |
| 4. Multi-tab & Polish | v0.2.0 | 3/3 | Complete | 2026-04-28 |
| 5. Backend SSH Key Storage | v0.3.0 | 2/2 | Complete | 2026-04-28 |
| 6. Frontend UI & Navigation | v0.3.0 | 3/3 | Complete | 2026-04-28 |
| 7. WebSocket Key Auth Integration | v0.3.0 | 2/2 | Complete | 2026-04-28 |
| 8. Port Forwarding | v0.3.0 | 2/2 | Complete | 2026-04-28 |
| 9. Settings Page | v0.3.0 | 3/3 | Complete | 2026-04-28 |
| 10. Tab Plus Button Popover | — | 0/2 | Planned | — |
