# WebTerm

## What This Is

Web-based SSH client yang memungkinkan developer mengakses dan mengelola koneksi SSH ke server langsung dari browser. Mirip Termius tapi self-hosted, dengan UI minimalis ala Vercel. Target pengguna: developer individual yang butuh akses terminal dari mana saja tanpa install client.

## Core Value

Bisa SSH ke server dari browser dengan pengalaman terminal yang smooth dan reliable — save connections, multi-tab, dan UI yang clean.

## Requirements

### Validated

- [x] User bisa menyimpan koneksi SSH (host, port, username) — Validated in Phase 2
- [x] User bisa connect ke server via SSH langsung dari browser — Validated in Phase 3
- [x] Terminal emulator yang smooth menggunakan @wterm.dev — Validated in Phase 3
- [x] Koneksi SSH via WebSocket proxy (browser → Go backend → SSH target) — Validated in Phase 3
- [x] Data koneksi disimpan server-side di SQLite — Validated in Phase 2
- [x] UI minimalis dan clean ala Vercel menggunakan shadcn — Validated in Phase 2
- [x] Frontend dibangun dengan Vite + React + shadcn — Validated in Phase 2

### Active

- [ ] Multi-tab — buka beberapa sesi SSH sekaligus

### Out of Scope

- **Authentication/User login** — v1 tanpa auth, fokus ke fitur terminal dulu
- **SFTP/File transfer** — ditunda ke v2
- **Team/Shared credentials** — target individual developer dulu
- **SSH key management** — v1 pakai password-based auth
- **Mobile responsive** — desktop-first untuk v1

## Context

- **Backend**: Golang — dipilih untuk performance dan native SSH library support
- **Frontend**: Vite + React + shadcn — modern stack, fast dev experience
- **Terminal library**: @wterm.dev by Vercel — pengganti xterm.js, belum pernah dipakai sebelumnya (first-time exploration)
- **Architecture**: WebSocket proxy pattern — browser gak bisa langsung SSH, jadi Go backend jadi perantara
- **Database**: SQLite — simple, server-side storage untuk connection data
- **Directory structure**: `be/` (Go backend) dan `fe/` (Vite/React frontend) sudah disiapkan
- **UI inspiration**: Clean minimalis ala Vercel, seimbang antara fungsional dan estetika

## Constraints

- **Tech Stack**: Go backend, Vite/React/shadcn frontend — sudah decided
- **Terminal Library**: @wterm.dev — first-time use, perlu exploration phase
- **Storage**: SQLite — no external DB dependency
- **No Auth**: v1 tanpa authentication system
- **SSH Auth**: Password-based only untuk v1 (no key management)

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| @wterm.dev over xterm.js | Library by Vercel, pengganti xterm.js yang lebih modern | — Pending |
| WebSocket proxy pattern | Browser tidak bisa langsung SSH, butuh perantara | — Pending |
| SQLite | Simple, no setup, cukup untuk individual use case | — Pending |
| No auth for v1 | Fokus ke core fitur terminal dulu, auth nanti | — Pending |
| Password-only SSH auth | Simpler untuk v1, key management complex | — Pending |

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

---
*Last updated: 2026-04-27 after Phase 3 completion*
