# Roadmap: WebTerm v0.5.0 — Desktop GPUI Client

## Overview

Milestone v0.5.0 ships a native desktop WebTerm built with GPUI (Rust) that reuses the existing Go backend as a locally-spawned process, reaching full feature parity with the web app on Windows and Linux. The terminal renders via alacritty_terminal — the engine proven in Zed.

## Milestones

- ✅ **v0.2.0 MVP** - [Archive](milestones/v0.2.0-ROADMAP.md) (shipped 2026-04-28)
- ✅ **v0.3.0 SSH Key Auth & UI Redesign** - [Archive](milestones/v0.3.0-ROADMAP.md) (shipped 2026-04-29)
- ✅ **v0.4.0 Local Terminal & SFTP** - [Archive](milestones/v0.4.0-ROADMAP.md) (shipped 2026-05-18)
- 🏗️ **v0.5.0 Desktop GPUI Client** - Phases 20-27

## Current Objective

Build the GPUI desktop client: app shell + backend integration, terminal rendering (alacritty engine), SSH/local sessions, hosts & keys management, SFTP dual-pane, forwarding & settings, and a parity-audit close-out.

## Progress

**Execution Order:** Phases 20 → 21 → 22 → 23 → 24 → 25 → 26 → 27

| Phase | Milestone | Description | Status | Target |
|-------|-----------|-------------|--------|--------|
| **20. Desktop Foundation & Backend Integration** | v0.5.0 | Cargo workspace, pinned deps + CI, GPUI app shell, backend supervisor + client, startup UX. | ✓ Complete | 2026-09-08 |
| **21. Terminal Rendering (Alacritty Engine)** | v0.5.0 | gpui-terminal spike with go/no-go; alacritty_terminal state + GPUI render; selection, scrollback, truecolor. | ✓ Complete | 2026-09-10 |
| **22. SSH Terminal Sessions & Tabs** | v0.5.0 | SSH connect (password/key), multi-tab with status dots, resize sync, shortcuts, reconnection + re-attach. | ✓ Complete | 2026-09-14 |
| **23. Hosts & SSH Keys Management** | v0.5.0 | Host cards, kebab menus, tags, search, quick-connect, import/export; key pool, passphrase flow, auth method. | ○ Not started | 2026-09-16 |
| **24. Local Terminal (Cross-Platform PTY)** | v0.5.0 | Local terminal tab on Linux (backend path) + Windows (ConPTY decision with go/no-go). | ○ Not started | 2026-09-18 |
| **25. SFTP Dual-Pane Manager** | v0.5.0 | Dual-pane browsing, local/remote sources, file ops with streaming, drag-and-drop incl. OS DnD. | ○ Not started | 2026-09-22 |
| **26. Port Forwarding, Settings & Theme Polish** | v0.5.0 | Forward rule management, settings page (desktop scope), theme system with terminal palette sync, window state. | ○ Not started | 2026-09-24 |
| **27. Parity Audit & Desktop Hardening** | v0.5.0 | REQ-by-REQ audit vs web app, reconnection chaos tests, performance pass, packaging for Windows + Linux. | ○ Not started | 2026-09-28 |

## Phase Details

### Phase 20: Desktop Foundation & Backend Integration

- **Goal:** Stand up the GPUI application and make the existing Go backend a locally-managed child process with one-click startup.
- **Depends on:** Nothing (first phase of the milestone)
- **Requirements:** [SHELL-01, SHELL-02, SHELL-03, SHELL-05, SHELL-06]
- **Success Criteria** (what must be TRUE):
  1. Running `webterm` (desktop binary) opens a native window on Windows and Linux without a manually started backend
  2. The window shows the shell skeleton (sidebar nav with Hosts / SSH Keys / SFTP / Settings) and a visible startup state that surfaces backend failure errors
  3. Dark/light theme selection renders across the shell and persists across restarts
  4. Window size and position persist across restarts
  5. Dependency pins are exact, Cargo.lock is committed, and CI builds both target platforms
- **Plans:** TBD

Plans:

- [x] 20-01-PLAN.md
- [x] 20-02-PLAN.md
- [x] 20-03-PLAN.md
- [x] 20-04-PLAN.md
- [x] 20-01: Workspace scaffold, dependency pins, CI matrix, tracing/logging
- [x] 20-02: Backend supervisor (spawn/ephemeral port/health/kill) + stale-instance handling
- [x] 20-03: GPUI app shell: root entity, sidebar navigation, startup state, theme application
- [x] 20-04: Desktop settings store (window state, theme, backend path) + persistence

### Phase 21: Terminal Rendering (Alacritty Engine)

- **Goal:** Render a terminal grid in GPUI through alacritty_terminal, proving selection, scrollback, and truecolor work — with an explicit go/no-go on the gpui-terminal crate.
- **Depends on:** Phase 20
- **Requirements:** [TERM-01, TERM-02, TERM-03, TERM-04]
- **Success Criteria** (what must be TRUE):
  1. A terminal view renders PTY byte streams (local test source) with correct colors, bold/italic/underline, and 24-bit truecolor
  2. Mouse selection + copy and paste work in real workflows
  3. Scrollback works (reviewable history after full-screen output)
  4. Resizing the view keeps the grid in sync (resize callback fires with cols/rows)
  5. Spike verdict recorded: gpui-terminal validated OR vendored Zed-pattern fallback implemented behind the terminal crate boundary
- **Plans:** 3/3 plans executed

Plans:

- [x] 21-01-PLAN.md
- [x] 21-02-PLAN.md
- [x] 21-03-PLAN.md
- [x] 21-01: Timeboxed spike on gpui-terminal (selection + scrollback against vim/htop) with go/no-go report
- [x] 21-02: Terminal state layer (alacritty_terminal Term, parser, scrollback) behind the crates/terminal boundary
- [x] 21-03: GPUI renderer (glyph batching, palette, cursor) + input mapping (keystroke → escape sequences)

### Phase 22: SSH Terminal Sessions & Tabs

- **Goal:** Wire real SSH sessions into the terminal: connect, tab lifecycle, resilience.
- **Depends on:** Phase 21
- **Requirements:** [SHELL-04, TERM-05, TERM-06, TERM-07, TERM-08]
- **Success Criteria** (what must be TRUE):
  1. User connects to a saved host over SSH (password or key) and works in vim/htop/tmux in a desktop tab
  2. Multiple tabs open/switch/close with per-session status indicators
  3. WS drop triggers reconnection UX; killing the backend process mid-session is distinguishable and recoverable (app restart re-attaches sessions)
  4. Keyboard shortcuts for new/close/cycle tabs follow desktop conventions
- **Plans:** TBD

Plans:

- [x] 22-01-PLAN.md
- [x] 22-02-PLAN.md
- [x] 22-03-PLAN.md
- [x] 22-04-PLAN.md
- [x] 22-01: backend-client WS terminal channel (framing matched to backend protocol, contract tests)
- [x] 22-02: Session manager + tab strip with status dots and shortcuts
- [x] 22-03: Reconnection UX + session re-attach across app restart
- [x] 22-04: Local terminal tab baseline via backend WS (POSIX path; full cross-platform PTY finalization in Phase 24)

### Phase 23: Hosts & SSH Keys Management

- **Goal:** Full connection and key management parity in GPUI, speaking the existing REST API.
- **Depends on:** Phase 22 (uses backend-client REST surface; can overlap Phase 22 in planning)
- **Requirements:** [HOSTS-01, HOSTS-02, HOSTS-03, HOSTS-04, KEYS-01, KEYS-02, KEYS-03]
- **Success Criteria** (what must be TRUE):
  1. User can create/edit/delete connections and browse them as cards with kebab menus, tags, and search
  2. Quick-connect works from the New Tab view; export/import JSON roundtrips with the web app
  3. Key pool supports upload/management; passphrase prompts cache session-scoped (never stored); per-connection auth method selectable
- **Plans:** 4/4 plans executed

Plans:

- [x] 23-01-PLAN.md
- [x] 23-02-PLAN.md
- [x] 23-03-PLAN.md
- [x] 23-04-PLAN.md
- [x] 23-01: REST Client Surface for Connections and Keys (HOSTS-01, HOSTS-04, KEYS-01)
- [x] 23-02: Hosts View, Card Grid, Search & Tag Filter, Quick-Connect (HOSTS-02, HOSTS-03)
- [x] 23-03: Connection Modal (Create/Edit) & JSON Import/Export (HOSTS-01, HOSTS-04, KEYS-03)
- [x] 23-04: SSH Keys View, Key Upload Modal, and Session Passphrase Prompt (KEYS-01, KEYS-02)

### Phase 24: Local Terminal (Cross-Platform PTY)

- **Goal:** Local terminal as a first-class New Tab option on both target platforms, resolving the Windows ConPTY decision explicitly.
- **Depends on:** Phase 22
- **Requirements:** [TERM-05]
- **Success Criteria** (what must be TRUE):
  1. Local terminal opens from New Tab as a first-class option on Linux (backend PTY path)
  2. Local terminal opens on Windows via the chosen ConPTY path (Go-side extension or Rust-side portable-pty), with a recorded decision
  3. A Windows CI smoke test exercises the local terminal path
- **Plans:** 3/3 plans executed
- [x] 24-01-PLAN.md
- [x] 24-02-PLAN.md
- [x] 24-03-PLAN.md

Plans:

- [x] 24-01: ConPTY spike + decision record (Go wrapper vs portable-pty bypass)
- [x] 24-02: Implement chosen path + New Tab ordering parity
- [x] 24-03: Windows CI smoke test for local terminal

### Phase 25: SFTP Dual-Pane Manager

- **Goal:** Port the dual-pane SFTP manager to GPUI at full interaction parity.
- **Depends on:** Phase 23 (shares connection state + REST client)
- **Requirements:** [SFTP-01, SFTP-02, SFTP-03, SFTP-04]
- **Success Criteria** (what must be TRUE):
  1. Dual-pane view opens from the sidebar; each pane selects Local Filesystem or a saved host
  2. Directory browsing with breadcrumbs, metadata columns, and sorting in both panes
  3. Upload/download/delete/rename/new-folder work with progress indicators; transfers stream
  4. Drag-and-drop works within/between panes and from the OS file manager (Windows Explorer + Linux)
- **Plans:** 3/3 plans executed

Plans:

- [x] 25-01-PLAN.md
- [x] 25-02-PLAN.md
- [x] 25-03-PLAN.md
- [x] 25-01: Dual-pane layout + directory browsers (source selection, breadcrumbs, metadata, sorting)
- [x] 25-02: File operations (upload/download/delete/rename/new folder) with transfer progress
- [x] 25-03: Drag-and-drop (inter-pane + OS DnD) + context menus + keyboard shortcuts

### Phase 26: Port Forwarding, Settings & Theme Polish

- **Goal:** Close out remaining parity surfaces: forwarding rules, settings, and cohesive theming.
- **Depends on:** Phase 23
- **Requirements:** [FWD-01, SET-01]
- **Success Criteria** (what must be TRUE):
  1. User can create and manage local port forwarding rules from the desktop app
  2. Settings page offers themes and desktop preferences (backend path override); no terminal engine selector
  3. Theme change applies to app and terminal palette live, mid-session
- **Plans:** 2/2 plans executed
- [x] 26-01-PLAN.md
- [x] 26-02-PLAN.md

Plans:

- [x] 26-01: Port forwarding management UI (FWD-01)
- [x] 26-02: Settings page (desktop scope) + live theme application incl. terminal palette sync (SET-01)

### Phase 27: Parity Audit & Desktop Hardening

- **Goal:** Verify full parity against the web app's validated requirements and harden for daily-driver use.
- **Depends on:** Phases 20-26
- **Requirements:** [QA-01, QA-02, QA-03]
- **Success Criteria** (what must be TRUE):
  1. Every validated v0.2-v0.4 REQ is verified against the desktop app in a recorded audit matrix
  2. Reconnection chaos tests pass (WS drop vs backend kill vs app restart)
  3. Performance holds under heavy terminal output (no UI stall, bounded memory)
  4. The app ships as a single launchable bundle per platform with the backend loopback-only
- **Plans:** 3/3 plans executed
- [x] 27-01-PLAN.md
- [x] 27-02-PLAN.md
- [x] 27-03-PLAN.md

Plans:

- [x] 27-01: Parity audit matrix execution vs PROJECT.md validated list (QA-02)
- [x] 27-02: Chaos + performance testing pass (QA-01)
- [x] 27-03: Packaging/installers (Windows + Linux) + loopback enforcement verification (QA-03)

## Progress

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 20. Desktop Foundation & Backend Integration | 4/4 | Complete | 2026-09-04 |
| 21. Terminal Rendering (Alacritty Engine) | 3/3 | Complete | 2026-09-04 |
| 22. SSH Terminal Sessions & Tabs | 4/4 | Complete | 2026-09-05 |
| 23. Hosts & SSH Keys Management | 4/4 | Complete | 2026-09-05 |
| 24. Local Terminal (Cross-Platform PTY) | 3/3 | Complete | 2026-09-05 |
| 25. SFTP Dual-Pane Manager | 3/3 | Complete | 2026-09-05 |
| 26. Port Forwarding, Settings & Theme Polish | 2/2 | Complete | 2026-09-05 |
| 27. Parity Audit & Desktop Hardening | 3/3 | Complete | 2026-09-05 |

**Validation:** 30 requirements mapped across 8 phases — coverage complete, every REQ mapped to exactly one phase ✓ (TERM-05 primary mapping: Phase 24)

---
*Last updated: 2026-09-04 at milestone v0.5.0 roadmap creation*
