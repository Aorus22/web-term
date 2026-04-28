# Project Retrospective

*A living document updated after each milestone. Lessons feed forward into future planning.*

## Milestone: v0.2.0 — MVP

**Shipped:** 2026-04-28
**Phases:** 4 | **Plans:** 10

### What Was Built
- WebSocket SSH proxy (Go backend) with keyboard-interactive auth and session management
- Connection management: REST API + SQLite (encrypted passwords) + React CRUD UI with shadcn
- Browser-based terminal using @wterm/react with resize, copy/paste, reconnection UX
- Multi-tab sessions with status indicators, keyboard shortcuts, dark/light theme

### What Worked
- Spike-first approach (Phase 1) validated terminal library before committing — zero rework
- Wave-based parallelization in Phases 3 and 4 — independent plans ran concurrently
- Vertical slice pattern (backend + frontend together per feature) — end-to-end working at each phase boundary
- shadcn component library — rapid UI development with consistent design

### What Was Inefficient
- Minor: crypto.randomUUID needed fallback for non-HTTPS contexts (caught in verification)
- Phase 2 had a combined backend+frontend commit instead of separate plan-level commits

### Patterns Established
- WebSocket proxy pattern for browser-to-SSH communication
- Zustand stores for frontend state management (app-store pattern)
- connection_id protocol for server-side password fetch (avoids sending passwords over WebSocket)
- AES-256-GCM encryption for stored connection passwords

### Key Lessons
1. Spike before commit — validating @wterm/react saved potential xterm.js migration cost
2. Keyboard-interactive SSH auth is essential for real-world server compatibility (not just simple password)
3. Secure-context fallbacks needed even for localhost development (some browsers restrict crypto APIs)

### Cost Observations
- Total execution time: ~2.33 hours across 10 plans
- Avg plan duration: 14 min
- Notable: Phase 1 spike was fastest path to risk resolution

---

## Cross-Milestone Trends

### Process Evolution

| Milestone | Phases | Plans | Key Change |
|-----------|--------|-------|------------|
| v0.2.0 | 4 | 10 | Spike-first validation, wave-based parallelization |

### Cumulative Quality

| Milestone | Source Files | LOC | Commits |
|-----------|-------------|-----|---------|
| v0.2.0 | 53 | ~5,007 | 51 |

### Top Lessons (Verified Across Milestones)

1. Spike before commit — validate key dependencies before building on them
2. Vertical slices deliver working software at each phase boundary
