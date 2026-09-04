# Requirements: Milestone v0.5.0 — Desktop GPUI Client

**Defined:** 2026-09-04
**Core Value:** SSH ke server dari browser (kini juga desktop) dengan pengalaman terminal yang smooth dan reliable — save connections, multi-tab, SFTP file management, dan UI yang clean.

## v1 Requirements

Requirements for the desktop client. Each maps to roadmap phases.

### Desktop Shell

- [x] **SHELL-01**: User can launch WebTerm as a native desktop app that starts the Go backend automatically and reaches a ready window
- [x] **SHELL-02**: User sees a visible startup state and a clear error if the backend fails to start
- [x] **SHELL-03**: User can navigate Hosts, SSH Keys, SFTP, and Settings via the sidebar (2-page navigation parity)
- [x] **SHELL-04**: User can open, switch, and close multiple terminal tabs with status indicators
- [x] **SHELL-05**: User's dark/light theme applies to the whole app and the terminal, persisting across restarts
- [x] **SHELL-06**: User's window size/position persists across restarts

### Terminal

- [x] **TERM-01**: User can work in SSH sessions rendered by the alacritty engine (vim, htop, tmux, curses apps, truecolor)
- [x] **TERM-02**: User can select text with the mouse and copy it, and paste into the terminal
- [x] **TERM-03**: User can scroll through scrollback history
- [x] **TERM-04**: Terminal resizes with the window and keeps the PTY grid in sync
- [x] **TERM-05**: User can open a local terminal tab (shell on the desktop host), first-class in New Tab
- [x] **TERM-06**: Terminal sessions reconnect after a WebSocket drop, with reconnection UX
- [x] **TERM-07**: Terminal sessions survive an app restart via backend session re-attach
- [x] **TERM-08**: Keyboard shortcuts for new tab, close tab, and tab cycling (desktop conventions)

### Hosts

- [x] **HOSTS-01**: User can create, edit, and delete SSH connections
- [x] **HOSTS-02**: User can browse connections as cards with kebab menus, tags, and search/filter
- [x] **HOSTS-03**: User can quick-connect from the New Tab view
- [x] **HOSTS-04**: User can export/import connections as JSON, roundtripping with the web app

### SSH Keys

- [x] **KEYS-01**: User can upload and manage SSH keys in the key pool
- [x] **KEYS-02**: User is prompted for key passphrases with session-scoped caching (never stored)
- [x] **KEYS-03**: User can choose password vs key authentication per connection

### SFTP

- [ ] **SFTP-01**: User can open the dual-pane SFTP manager with a local or remote source per pane
- [ ] **SFTP-02**: User can browse directories with breadcrumbs, file metadata, and sorting
- [ ] **SFTP-03**: User can upload, download, delete, rename, and create folders with progress indicators
- [ ] **SFTP-04**: User can drag-and-drop files within/between panes and from the OS file manager

### Forwarding & Settings

- [ ] **FWD-01**: User can create and manage local port forwarding rules
- [ ] **SET-01**: User can configure themes and desktop preferences (e.g., backend path override); no terminal engine selector — the desktop engine is fixed to alacritty

### Quality & Packaging

- [ ] **QA-01**: App runs on Windows 10/11 and Linux (X11/Wayland)
- [ ] **QA-02**: A parity audit verifies every validated v0.2–v0.4 requirement against the desktop app
- [ ] **QA-03**: App ships as a single launchable bundle with the backend bound to loopback only

## Future Requirements

Deferred to future milestones. Tracked but not in current roadmap.

### Desktop Native Extras

- **DESK-01**: Command palette for fast actions
- **DESK-02**: System tray with close-to-tray behavior
- **DESK-03**: Global hotkey to summon the window
- **DESK-04**: Auto-updater

### Platform

- **PLAT-01**: macOS target

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Authentication/user login | v1 stays auth-free (PROJECT.md decision) |
| Team/shared credentials | Individual developer first |
| Mobile | Desktop milestone; unchanged |
| Mosh protocol | UDP, incompatible with WebSocket proxy |
| Plugin/extension system | No demand yet (unchanged) |
| Native Rust SSH implementation (russh) | Go backend locked as the single SSH/SFTP implementation |
| Terminal engine selector | Desktop engine fixed to alacritty_terminal (Zed-proven) |
| Native S3/object storage in SFTP | Parked Active item from v0.4; not part of desktop parity |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| SHELL-01 | Phase 20 | Satisfied |
| SHELL-02 | Phase 20 | Satisfied |
| SHELL-03 | Phase 20 | Satisfied |
| SHELL-05 | Phase 20 | Satisfied |
| SHELL-06 | Phase 20 | Satisfied |
| TERM-01 | Phase 21 | Satisfied |
| TERM-02 | Phase 21 | Satisfied |
| TERM-03 | Phase 21 | Satisfied |
| TERM-04 | Phase 21 | Satisfied |
| TERM-05 | Phase 22 | Satisfied |
| TERM-06 | Phase 22 | Satisfied |
| TERM-07 | Phase 22 | Satisfied |
| TERM-08 | Phase 22 | Satisfied |
| SHELL-04 | Phase 22 | Satisfied |
| HOSTS-01 | Phase 23 | Satisfied |
| HOSTS-02 | Phase 23 | Satisfied |
| HOSTS-03 | Phase 23 | Satisfied |
| HOSTS-04 | Phase 23 | Satisfied |
| KEYS-01 | Phase 23 | Satisfied |
| KEYS-02 | Phase 23 | Satisfied |
| KEYS-03 | Phase 23 | Satisfied |
| SFTP-01 | Phase 25 | Pending |
| SFTP-02 | Phase 25 | Pending |
| SFTP-03 | Phase 25 | Pending |
| SFTP-04 | Phase 25 | Pending |
| FWD-01 | Phase 26 | Pending |
| SET-01 | Phase 26 | Pending |
| QA-01 | Phase 27 | Pending |
| QA-02 | Phase 27 | Pending |
| QA-03 | Phase 27 | Pending |

**Coverage:**
- v1 requirements: 30 total
- Mapped to phases: 30
- Unmapped: 0 ✓

---
*Requirements defined: 2026-09-04*
*Last updated: 2026-09-05 after Phase 23 completion*
