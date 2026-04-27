# Roadmap: WebTerm

## Overview

WebTerm is a self-hosted web-based SSH client. The roadmap starts with validating the terminal library (@wterm/react, v0.2.0 — days old), then builds a complete connection management layer (backend + frontend together as a vertical slice), delivers the core SSH terminal experience (the product's entire reason to exist), and finishes with multi-tab sessions and polish.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: wterm Spike** - Validate @wterm/react handles SSH workloads before committing to architecture
- [ ] **Phase 2: Connection Management** - Full vertical slice: Go backend + React frontend for saving, organizing, and managing SSH connections
- [ ] **Phase 3: SSH Terminal** - Core product: browser-to-SSH terminal with password auth, resize, copy/paste, and reconnection
- [ ] **Phase 4: Multi-Tab & Polish** - Multiple simultaneous sessions, keyboard shortcuts, dark/light theme

## Phase Details

### Phase 1: wterm Spike
**Goal**: Validate that @wterm/react can handle SSH terminal workloads (vim, htop, tmux, resize) before committing to the architecture — if it fails, fall back to xterm.js
**Depends on**: Nothing (first phase)
**Requirements**: (validation gate — no production requirements; validates approach for TERM-01 through TERM-05)
**Success Criteria** (what must be TRUE):
  1. @wterm/react renders a terminal in a React app connected to a Go WebSocket server
  2. vim, htop, and curses apps display correctly without rendering artifacts
  3. Terminal resize propagates to the remote PTY and redraws correctly
  4. Go backend successfully proxies bidirectional I/O between WebSocket and SSH
  5. Go/no-go decision documented: proceed with @wterm/react or fall back to xterm.js
**Plans**: TBD
**UI hint**: yes

### Phase 2: Connection Management
**Goal**: Users can save, organize, and manage SSH connections through a clean Vercel-inspired web interface
**Depends on**: Phase 1 (terminal library decision finalized)
**Requirements**: CONN-01, CONN-02, CONN-03, CONN-04, CONN-05, CONN-06, CONN-07, UI-01
**Success Criteria** (what must be TRUE):
  1. User can create a new SSH connection (host, port, username, label) and it persists across page reloads
  2. Saved connections appear in a sidebar list, organized by tags/labels
  3. User can edit and delete existing connections with confirmation on destructive actions
  4. User can type user@host:port in a quick-connect bar to connect without saving
  5. User can export and import connections as JSON for backup
**Plans**: TBD
**UI hint**: yes

### Phase 3: SSH Terminal
**Goal**: Users can SSH to any server from the browser with a smooth, reliable terminal experience — this is the product's core value
**Depends on**: Phase 2 (connection data and UI shell available)
**Requirements**: TERM-01, TERM-02, TERM-03, TERM-04, TERM-05, UI-04
**Success Criteria** (what must be TRUE):
  1. User can connect to a remote server and get an interactive shell that handles vim, htop, tmux
  2. Password authentication works, including keyboard-interactive SSH prompts
  3. Terminal auto-resizes when the browser window or pane resizes
  4. Copy/paste works natively through browser text selection
  5. WebSocket disconnection shows a reconnection prompt to the user
**Plans**: TBD
**UI hint**: yes

### Phase 4: Multi-Tab & Polish
**Goal**: Users can work with multiple SSH sessions simultaneously with a polished, keyboard-driven workflow
**Depends on**: Phase 3 (single SSH session works end-to-end)
**Requirements**: TAB-01, TAB-02, TAB-03, TAB-04, UI-02, UI-03
**Success Criteria** (what must be TRUE):
  1. User can open multiple independent SSH sessions as tabs in a single browser window
  2. Closing a tab cleanly disconnects the SSH session without zombie processes
  3. Each tab shows its connection status (connected, connecting, disconnected)
  4. Keyboard shortcuts work: Ctrl+T (new tab), Ctrl+W (close tab), Ctrl+Tab (switch tab)
  5. Dark/light theme toggle changes both UI and terminal appearance
**Plans**: TBD
**UI hint**: yes

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. wterm Spike | 0/? | Not started | - |
| 2. Connection Management | 0/? | Not started | - |
| 3. SSH Terminal | 0/? | Not started | - |
| 4. Multi-Tab & Polish | 0/? | Not started | - |
