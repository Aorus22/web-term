# WebTerm

## What This Is

Self-hosted web-based SSH client and file manager. Browser-based terminal with connection management, multi-tab sessions, dark/light theme, local terminal support, and SFTP file management. Built with Go backend (WebSocket SSH proxy) and React frontend (@wterm/react + shadcn).

## Core Value

SSH ke server dari browser dengan pengalaman terminal yang smooth dan reliable — save connections, multi-tab, SFTP file management, dan UI yang clean.

## Requirements

### Validated

- ✓ WebSocket SSH proxy (browser → Go backend → SSH target) — v0.2.0
- ✓ Terminal rendering via @wterm/react (vim, htop, tmux, curses) — v0.2.0
- ✓ Password-based SSH auth with keyboard-interactive support — v0.2.0
- ✓ Auto-resize terminal synced with SSH PTY — v0.2.0
- ✓ Save/edit/delete SSH connections in SQLite — v0.2.0
- ✓ Sidebar with connection list, tags, quick-connect bar — v0.2.0
- ✓ Export/import connections as JSON — v0.2.0
- ✓ Multi-tab SSH sessions with status indicators — v0.2.0
- ✓ Dark/light theme synced with terminal — v0.2.0
- ✓ Keyboard shortcuts (Ctrl+T/W/Tab) — v0.2.0
- ✓ Reconnection handling on WebSocket disconnect — v0.2.0
- ✓ Clean UI using shadcn — v0.2.0
- ✓ SSH key-based authentication (upload, connect, manage keys) — v0.3.0
- ✓ Sidebar redesign into 2-page navigation (Hosts + SSH Keys) — v0.3.0
- ✓ Hosts page with card-based layout and kebab menus — v0.3.0
- ✓ SSH Keys page with key pool management — v0.3.0
- ✓ Per-connection auth method selection (password vs key) — v0.3.0
- ✓ SSH local port forwarding (bind remote port to localhost) — v0.3.0
- ✓ Backend session persistence (survive reloads and re-attach) — v0.3.0
- ✓ Local terminal support (spawn shell on backend host) — v0.4.0
- ✓ Dual-pane SFTP file manager UI — v0.4.0
- ✓ SFTP file operations (list, upload, download, delete, rename) — v0.4.0

### Active

- [ ] S3 / Object Storage integration in SFTP manager
- [ ] SSH Agent forwarding support
- [ ] Bulk connection edit/management

### Out of Scope

- **Authentication/User login** — v1 tanpa auth, fokus ke fitur terminal dulu
- **Team/Shared credentials** — target individual developer dulu
- **Mobile responsive** — desktop-first untuk v1
- **Mosh protocol** — requires UDP, incompatible with WebSocket proxy
- **Plugin/extension system** — premature abstraction, no demand yet
- **Native desktop/mobile apps** — web-first, browser is the platform

## Context

- **Shipped:** v0.3.0 on 2026-04-29
- **LOC:** ~7,200 lines across 75 source files (Go + TypeScript + CSS)
- **Tech stack:** Go backend (gorilla/websocket, golang.org/x/crypto/ssh, pkg/sftp), Vite + React + shadcn frontend, @wterm/react terminal, SQLite
- **Architecture:** WebSocket proxy pattern — browser ↔ Go backend ↔ SSH/Local target
- **Timeline:** 3 days (2026-04-27 → 2026-04-29), 96 commits, 26 plans across 11 phases

## Constraints

- **Tech Stack:** Go backend, Vite/React/shadcn frontend — locked
- **Terminal Library:** @wterm/react — validated, working well
- **Storage:** SQLite — no external DB dependency
- **No Auth:** v1 without authentication system
- **SSH Auth:** Password and SSH key-based auth supported

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| @wterm/react over xterm.js | Modern Vercel library, better DX | ✓ Good — handles vim/htop/tmux cleanly |
| WebSocket proxy pattern | Browser cannot SSH directly | ✓ Good — stable bidirectional I/O |
| SQLite | Simple, no setup, sufficient for single-user | ✓ Good — AES-256-GCM encryption for passwords |
| No auth for v1 | Focus on core terminal features | ✓ Good — appropriate for self-hosted single-user |
| Theme in localStorage | Persist preference without backend | ✓ Good |
| QuickConnect in NewTabView | Simplify sidebar, reduce clutter | ✓ Good — cleaner architecture |
| TabBar status dots | Visual session status at a glance | ✓ Good |
| Passphrase caching via ref | Session-scoped, never stored, auto-cleared on disconnect | ✓ Good — secure UX without repeated prompts |
| Backend Session Re-attach | Uses a session manager to map IDs to active SSH connections | ✓ Good — handles reloads seamlessly |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

## Current Milestone: v0.4.0 Local Terminal & SFTP

**Goal:** Add support for local terminal sessions and a dual-pane SFTP file manager.

**Target features:**
- **Local Terminal:**
    - Option in "New Tab" view to open a local shell on the backend host.
    - Local terminal should be the first/primary option in New Tab.
    - Full PTY support (ANSI colors, window resizing).
- **Dual-Pane SFTP Manager:**
    - New "SFTP" navigation item in the sidebar.
    - Split-screen view allowing two independent directory browsers.
    - Each pane can select a "Source": Local Filesystem or Remote Host (via SSH/SFTP).
    - File operations: List, Upload, Download, Delete, Rename.
    - Drag-and-drop support between panes.
    - High-quality UI inspired by Termius.

---

*Last updated: 2026-05-03 after Milestone v0.3.0 complete*
