# Milestones

- ✅ **v0.2.0 — MVP** (shipped 2026-04-28)
- ✅ **v0.3.0 — SSH Key Auth & UI Redesign** (shipped 2026-04-29)

---

## v0.3.0 — SSH Key Auth & UI Redesign
**Goal:** Add SSH key-based authentication + redesign sidebar into a 2-page navigation.

### Key Accomplishments
- **Backend Key Storage:** Secure AES-256-GCM encrypted storage for SSH private keys.
- **Frontend Navigation:** Redesigned sidebar with "Hosts" and "SSH Keys" views.
- **SSH Key Auth:** Full support for key-based authentication, including passphrase-protected keys.
- **Port Forwarding:** Basic local port forwarding support.
- **Settings Page:** Centralized configuration for themes and UI preferences.
- **Session Persistence:** Backend-managed SSH sessions that survive browser reloads.

### Stats
- **Phases:** 7 (05-11)
- **Plans:** 16
- **Duration:** ~4 hours active development
- **Commits:** ~45

---

## v0.2.0 — MVP

**Shipped:** 2026-04-28
**Phases:** 4 | **Plans:** 10 | **LOC:** ~5,007

### Key Accomplishments

1. Validated @wterm/react handles SSH workloads (vim, htop, tmux, resize) — go/no-go decision: PROCEED
2. Built full connection management: Go REST API + SQLite with AES-256-GCM encryption, React CRUD UI with shadcn
3. Delivered core SSH terminal: keyboard-interactive auth, useSSHSession hook, TerminalPane with 4 states, reconnection UX
4. Multi-tab sessions with TabBar status dots, keyboard shortcuts (Ctrl+T/W/Tab), dark/light theme synced with terminal
5. Clean Vercel-inspired UI with connection sidebar, quick-connect bar, export/import, tag filtering

### Stats

- Timeline: 2 days (2026-04-27 → 2026-04-28)
- Commits: 51
- Files: 53 source files
- Languages: Go, TypeScript, CSS
- Avg plan duration: ~14 min
