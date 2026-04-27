# Requirements: WebTerm

**Defined:** 2026-04-27
**Core Value:** Bisa SSH ke server dari browser dengan pengalaman terminal yang smooth dan reliable

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### Terminal & SSH Connection

- [ ] **TERM-01**: WebSocket SSH proxy — Go backend accepts WebSocket connection, dials SSH to target server, pipes bidirectional I/O
- [ ] **TERM-02**: Terminal rendering via @wterm/react — handles vim, htop, tmux, curses apps correctly
- [ ] **TERM-03**: Password-based SSH authentication with keyboard-interactive support
- [ ] **TERM-04**: Auto-resize terminal — terminal resizes when browser window/pane resizes, SSH PTY resize propagated
- [ ] **TERM-05**: Copy/paste — native browser text selection and clipboard via DOM rendering

### Connection Management

- [ ] **CONN-01**: Save SSH connections — store host, port, username, label in SQLite
- [ ] **CONN-02**: List saved connections in sidebar
- [ ] **CONN-03**: Edit saved connections
- [ ] **CONN-04**: Delete saved connections
- [ ] **CONN-05**: Quick-connect bar — type `user@host:port` and connect immediately without saving
- [ ] **CONN-06**: Connection grouping/tags — organize connections with tags/labels
- [ ] **CONN-07**: Export/import connections as JSON backup

### Multi-Tab Sessions

- [ ] **TAB-01**: Open multiple SSH sessions in tabs within single browser window
- [ ] **TAB-02**: Per-tab independent WebSocket connection and SSH session lifecycle
- [ ] **TAB-03**: Close tab cleanly disconnects SSH session (no zombie processes)
- [ ] **TAB-04**: Connection status indicator per tab — connected/connecting/disconnected

### UI & UX

- [ ] **UI-01**: Clean, modern UI using shadcn — minimalist Vercel-inspired design
- [ ] **UI-02**: Dark/light theme — system preference detection + manual toggle, synced with terminal theme
- [ ] **UI-03**: Keyboard shortcuts — Ctrl+T (new tab), Ctrl+W (close tab), Ctrl+Tab (switch tab)
- [ ] **UI-04**: Reconnection handling — auto-reconnect on WebSocket disconnect or prompt user

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Authentication

- **AUTH-01**: Basic authentication system (login/register)
- **AUTH-02**: OAuth login (GitHub, Google)

### SSH Advanced

- **SSH-01**: SSH key management — upload, store encrypted, use for authentication
- **SSH-02**: SFTP/file transfer panel
- **SSH-03**: Port forwarding UI
- **SSH-04**: Jump host / bastion support

### Collaboration

- **COLLAB-01**: Multi-user support with RBAC
- **COLLAB-02**: Shared connection vaults

### Other

- **MOBILE-01**: Mobile responsive design
- **REC-01**: Session recording and playback
- **AI-01**: AI-powered command autocomplete

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Mosh protocol | Requires UDP, incompatible with WebSocket proxy |
| Plugin/extension system | Premature abstraction, no demand yet |
| Native desktop/mobile apps | Web-first, browser is the platform |
| Team features | Individual developer target for v1 |
| Terminal multiplexing | Use tmux inside terminal instead |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| TERM-01 | Phase 1 (validated), Phase 3 | Validated |
| TERM-02 | Phase 1 (validated), Phase 3 | Validated |
| TERM-03 | Phase 1 (validated), Phase 3 | Validated |
| TERM-04 | Phase 1 (validated), Phase 3 | Validated |
| TERM-05 | Phase 1 (validated), Phase 3 | Validated |
| CONN-01 | Phase 2 | Pending |
| CONN-02 | Phase 2 | Pending |
| CONN-03 | Phase 2 | Pending |
| CONN-04 | Phase 2 | Pending |
| CONN-05 | Phase 2 | Pending |
| CONN-06 | Phase 2 | Pending |
| CONN-07 | Phase 2 | Pending |
| TAB-01 | Phase 4 | Pending |
| TAB-02 | Phase 4 | Pending |
| TAB-03 | Phase 4 | Pending |
| TAB-04 | Phase 4 | Pending |
| UI-01 | Phase 2 | Pending |
| UI-02 | Phase 4 | Pending |
| UI-03 | Phase 4 | Pending |
| UI-04 | Phase 3 | Pending |

**Coverage:**
- v1 requirements: 20 total
- Mapped to phases: 20
- Unmapped: 0 ✓

---
*Requirements defined: 2026-04-27*
*Last updated: 2026-04-27 after Phase 1 completion*
