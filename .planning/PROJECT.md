# WebTerm

## What This Is

Self-hosted web-based SSH client. Browser-based terminal with connection management, multi-tab sessions, dark/light theme, and keyboard shortcuts. Built with Go backend (WebSocket SSH proxy) and React frontend (@wterm/react + shadcn).

## Core Value

SSH ke server dari browser dengan pengalaman terminal yang smooth dan reliable — save connections, multi-tab, dan UI yang clean.

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

### Active

- [ ] SSH local port forwarding (bind remote port to localhost)
- [ ] Port forwarding sheet UI with connection selector
- [ ] Port conflict detection with toast error

### Out of Scope

- **Authentication/User login** — v1 tanpa auth, fokus ke fitur terminal dulu
- **SFTP/File transfer** — ditunda ke v2
- **Team/Shared credentials** — target individual developer dulu
- **SSH key management** — v1 pakai password-based auth
- **Mobile responsive** — desktop-first untuk v1
- **Mosh protocol** — requires UDP, incompatible with WebSocket proxy
- **Plugin/extension system** — premature abstraction, no demand yet
- **Native desktop/mobile apps** — web-first, browser is the platform

## Context

- **Shipped:** v0.2.0 on 2026-04-28
- **LOC:** ~5,007 lines across 53 source files (Go + TypeScript + CSS)
- **Tech stack:** Go backend (gorilla/websocket, golang.org/x/crypto/ssh), Vite + React + shadcn frontend, @wterm/react terminal, SQLite
- **Architecture:** WebSocket proxy pattern — browser ↔ Go backend ↔ SSH target
- **Timeline:** 2 days (2026-04-27 → 2026-04-28), 51 commits, 10 plans across 4 phases

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
| Password-only SSH auth | Simpler for v1 | ✓ Good — keyboard-interactive supported |
| Theme in localStorage | Persist preference without backend | ✓ Good |
| QuickConnect in NewTabView | Simplify sidebar, reduce clutter | ✓ Good — cleaner architecture |
| TabBar status dots | Visual session status at a glance | ✓ Good |
| Passphrase caching via ref | Session-scoped, never stored, auto-cleared on disconnect | ✓ Good — secure UX without repeated prompts |

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
## Current Milestone: v0.3.0 SSH Key Auth & UI Redesign

**Goal:** Add SSH key-based authentication + redesign sidebar into a 2-page navigation (Hosts & SSH Keys).

**Target features:**
- Sidebar → 2-page nav: "Hosts" and "SSH Keys" menu items
- Hosts page: Card-based layout showing all hosts, kebab menu per card (edit/delete/duplicate), click to connect. Add/edit via right-side sheet (existing pattern)
- SSH Keys page: Key pool management — upload, list, rename, delete keys
- SSH Key auth: Upload existing private keys, encrypted at rest (AES-256-GCM), passphrase-protected key support (prompt on connect)
- Per-connection auth: When adding/editing a host, select auth method (password vs key) and which key to use

---

*Last updated: 2026-04-28 after Phase 7 complete*
