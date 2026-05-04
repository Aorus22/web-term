# Roadmap: WebTerm v0.4.0 — Local Terminal & SFTP

## Overview
Milestone v0.4.0 introduces local terminal access and a dual-pane SFTP file manager.

## Milestones
- ✅ **v0.2.0 MVP** - [Archive](milestones/v0.2.0-ROADMAP.md) (shipped 2026-04-28)
- ✅ **v0.3.0 SSH Key Auth & UI Redesign** - [Archive](milestones/v0.3.0-ROADMAP.md) (shipped 2026-04-29)
- 🏗️ **v0.4.0 Local Terminal & SFTP** - (Target: 2026-05-05)

## Current Objective
Implement local shell support and a Termius-inspired dual-pane SFTP manager.

## Progress
**Execution Order:** Phases 12 → 13 → 14 → 15 → 16 → 17

| Phase | Milestone | Description | Status | Target |
|-------|-----------|-------------|--------|--------|
| **12. Local Terminal Foundation** | v0.4.0 | Backend PTY spawning + Frontend "Local Terminal" option in New Tab. | ✅ Shipped | 2026-05-03 |
| **13. SFTP Backend Core** | v0.4.0 | `pkg/sftp` integration, Local FS driver, and REST API for file operations. | ✅ Shipped | 2026-05-03 |
| **14. SFTP Frontend UI** | v0.4.0 | Sidebar nav, dual-pane layout, and basic directory browsing (Local & Remote). | ✅ Shipped | 2026-05-04 |
| **15. SFTP Operations & DND** | v0.4.0 | File upload/download/delete/rename and inter-pane drag-and-drop. | 🏗️ Planned | 2026-05-05 |
| **16. Polish & UI Cohesion** | v0.4.0 | Breadcrumbs, keyboard shortcuts, progress indicators, and visual polish. | 🏗️ Planned | 2026-05-05 |
| **17. Terminal Theme Sync** | v0.4.0 | Sync terminal theme selection from settings to overall application theme for cohesive look. | 🏗️ Planned | 2026-05-05 |

## Phase Details

### Phase 12: Local Terminal Foundation
- **Goal:** Enable users to open a terminal session on the machine running the WebTerm backend.
- **Requirements:** [REQ-PTY-INTEGRATION, REQ-LOCAL-WS-HANDLER, REQ-LOCAL-UI-OPTION]
- **Tasks:**
    - Integrate `creack/pty` in Go backend.
    - Add `ConnectionID: "local"` support to WebSocket handler.
    - Update `NewTabView.tsx` to show "Local Terminal" as the primary option.
    - Ensure terminal resizing and I/O work for local PTY.

### Phase 13: SFTP Backend Core
- **Goal:** Provide a unified API for interacting with local and remote filesystems.
- **Plans:** 1 plan
- **Plan list:**
    - [ ] 13-01-PLAN.md — Implement FileSystem drivers and SFTP REST API.
- **Requirements:** [SFTP-01, SFTP-02, SFTP-03, SFTP-04]
- **Tasks:**
    - Add `pkg/sftp` dependency.
    - Implement a `FileSystem` interface in Go with `Local` and `SFTP` implementations.
    - Create REST endpoints for `list`, `upload`, `download`, `delete`, `rename`.
    - Handle authentication by reusing existing SSH client sessions.

### Phase 14: SFTP Frontend UI
- **Goal:** Create the visual framework for the SFTP manager.
- **Plans:** 2 plans
- **Plan list:**
    - [ ] 14-01-PLAN.md — Setup navigation, API extensions, and dual-pane resizable layout.
    - [x] 14-02-PLAN.md — Implement directory listing, navigation logic, and source selection.
- **Requirements:** [SFTP-01, SFTP-02, SFTP-03, SFTP-04]
- **Tasks:**
    - Add "SFTP" to sidebar navigation.
    - Create `SFTPView` component with a dual-pane `SplitLayout`.
    - Implement `DirectoryBrowser` component with source selector (Local vs Hosts).
    - Basic directory listing and navigation (double-click to enter folder).

### Phase 15: SFTP Operations & DND
- **Goal:** Enable file management and inter-host transfers.
- **Tasks:**
    - Implement file upload (multipart) and download (stream).
    - Add UI for Delete and Rename (with confirmation).
    - Implement drag-and-drop between panes to trigger transfers.
    - Handle large file transfers with chunked/streamed backend processing.

### Phase 16: Polish & UI Cohesion
- **Goal:** Finalize the UX and visual design of the SFTP manager with breadcrumbs, shortcuts, and progress tracking.
- **Plans:** 3 plans
- **Plan list:**
    - [ ] 16-01-PLAN.md — Implement interactive breadcrumbs and file-type icons.
    - [ ] 16-02-PLAN.md — Implement keyboard shortcuts (Enter, Backspace, Delete, etc.).
    - [ ] 16-03-PLAN.md — Create a centralized Transfer Manager for progress tracking.
- **Requirements:** [SFTP-01, SFTP-04]
- **Tasks:**
    - Add interactive breadcrumbs for path navigation.
    - Implement keyboard shortcuts (Delete, F5, Ctrl+C/V).
    - Add a "Transfer Queue" or toast-based progress indicators.
    - Visual polish: high-quality icons, row highlighting, and transition animations.

### Phase 17: Terminal Theme Sync
- **Goal:** When user selects a terminal theme in settings, apply the corresponding theme colors to the overall application UI for a cohesive visual experience.
- **Tasks:**
    - Map terminal theme palettes to app-level CSS variables.
    - Update settings UI to trigger app theme change on terminal theme selection.
    - Ensure all app components (sidebar, tabs, panels) respond to the synced theme.
    - Handle edge cases (custom themes, fallback colors).
