---
phase: 01-wterm-spike
plan: 02
subsystem: terminal
tags: [wterm, react, validation, spike]

requires:
  - phase: 01-01
    provides: Go WebSocket SSH proxy + React @wterm/react terminal
provides:
  - Spike validation results for all 8 tests
  - Go/no-go decision: PROCEED with @wterm/react
affects: [02-production-build]

tech-stack:
  added: []
  patterns: []

key-files:
  created:
    - .planning/phases/01-wterm-spike/01-SPIKE-RESULT.md
  modified: []

key-decisions:
  - "DECISION: PROCEED with @wterm/react — all critical tests passed, color issues are cosmetic only"
  - "@wterm/react v0.1.9 downgraded from v0.2.0 due to npm publishing bug"

patterns-established: []

requirements-completed:
  - TERM-02
  - TERM-03
  - TERM-04
  - TERM-05

duration: 15min
completed: 2026-04-27
---

# Phase 01 Plan 02: Spike Validation Summary

**Human validation of @wterm/react spike — all 8 tests passed, DECISION: PROCEED**

## Performance

- **Duration:** ~15 min (including debugging)
- **Tasks:** 2 (1 human-verify checkpoint, 1 auto documentation)
- **Files modified:** 1

## Accomplishments
- All 8 validation tests passed: basic connection, vim, htop, tmux, resize, copy/paste, curses apps, long output
- Documented go/no-go decision: PROCEED with @wterm/react
- Identified minor color accuracy issue (cosmetic, not blocking)

## Task Commits

1. **Task 1: Human validation** — manual testing of spike app
2. **Task 2: Document decision** - `d342843` (docs)

## Issues Encountered
- **WebSocket disconnection on connect:** Root cause was malformed SSH `window-change` payload (2 uint32s instead of required 4). Fixed in plan 01-01 commits.
- **Vite proxy not forwarding messages:** Bypassed by connecting directly to Go backend on port 8080.
- **Color accuracy slightly off:** Especially visible in tmux. Cosmetic only, readable. Likely @wterm/react theme mapping issue.

## Deviations from Plan

### Auto-fixed Issues

**1. Bug fix: window-change SSH payload format**
- **Found during:** Task 1 (human validation)
- **Issue:** SSH window-change request only sent 2 uint32s (cols, rows) instead of required 4 (cols, rows, width, height)
- **Fix:** Added Width and Height fields (set to 0) to the struct
- **Files modified:** be/main.go
- **Committed in:** `5eff605`

**2. Bug fix: concurrent WebSocket writes**
- **Found during:** Task 1 (human validation)
- **Issue:** stdout and stderr goroutines wrote to WebSocket concurrently — gorilla/websocket requires serialized writes
- **Fix:** Added sync.Mutex wrapper for all WebSocket writes
- **Files modified:** be/main.go
- **Committed in:** `b190029`

**3. Config change: bind to 0.0.0.0**
- **Found during:** Task 1 (user request)
- **Issue:** Default 127.0.0.1 binding prevented access from other devices
- **Fix:** Changed Go backend and Vite dev server to bind 0.0.0.0
- **Files modified:** be/main.go, fe/vite.config.ts
- **Committed in:** `812b5f1`

**4. Fix: direct WebSocket connection**
- **Found during:** Task 1 (Vite proxy not forwarding)
- **Fix:** Client connects directly to Go backend port 8080 instead of through Vite proxy
- **Files modified:** fe/src/App.tsx
- **Committed in:** `ec9629e`

---

**Total deviations:** 4 auto-fixed (3 bug fixes, 1 config change)
**Impact on plan:** All fixes necessary for the spike to function. No scope creep.

## Next Phase Readiness
- @wterm/react validated for production use
- Go WebSocket SSH proxy pattern proven
- Phase 2 can build on this architecture directly
- Color accuracy issue should be investigated in Phase 2 (theme configuration)

---
*Phase: 01-wterm-spike*
*Completed: 2026-04-27*
