---
phase: 10-tab-plus-button-popover
plan: 01
subsystem: backend
tags: [go, websocket, ssh, cwd]

# Dependency graph
requires: []
provides:
  - ConnectMessage.Cwd field for specifying initial working directory on connect
  - CwdResponseMessage type for responding to get-cwd WebSocket requests
  - get-cwd WebSocket message handler with stdout marker-based interception
  - cwd injection after Shell() with path sanitization
affects: [10-tab-plus-button-popover-frontend]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "stdout marker interception: buffer output until sentinel marker found, then flush"
    - "space-prefixed cd command to avoid shell history"

key-files:
  created:
    - be/internal/ssh/types_test.go
  modified:
    - be/internal/ssh/types.go
    - be/internal/ssh/proxy.go

key-decisions:
  - "Space-prefixed cd command (cd) to prevent it appearing in shell history"
  - "Single-quote wrapping for cwd path in cd command to handle spaces in paths"
  - "Path sanitization: reject paths containing single quotes or newlines to prevent injection"
  - "Marker-based stdout interception (__WTERM_CWD_END__) for get-cwd parsing"

patterns-established:
  - "stdout buffer interception: when cwdPending is true, accumulate stdout reads until sentinel marker, then parse path and flush"

requirements-completed: [TAB-01, TAB-02]

# Metrics
duration: 2min
completed: 2026-04-29
---

# Phase 10 Plan 01: Backend cwd Support Summary

**Backend cwd tracking and injection: ConnectMessage.Cwd field, get-cwd WebSocket handler with stdout marker interception, and path-sanitized cd injection after Shell()**

## Performance

- **Duration:** 2 min
- **Started:** 2026-04-29T02:38:40Z
- **Completed:** 2026-04-29T02:40:31Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Added `Cwd` field to `ConnectMessage` with `omitempty` tag for duplicate tab directory passing
- Added `CwdResponseMessage` struct for structured cwd JSON responses
- Implemented `get-cwd` WebSocket message handler with pwd marker injection and stdout interception
- Implemented cwd injection after `Shell()` with security sanitization (T-10-01)
- TDD approach: failing tests first (RED), then implementation (GREEN)

## Task Commits

Each task was committed atomically:

1. **Task 1: Add cwd field to ConnectMessage and get-cwd message types** - `9c4bd36` (test), `b245b9e` (feat)
2. **Task 2: Implement get-cwd handler and cwd injection in SSH proxy** - `b773d2a` (feat)

_Note: TDD tasks have multiple commits (test → feat)_

## Files Created/Modified
- `be/internal/ssh/types.go` - Added Cwd field to ConnectMessage, added CwdResponseMessage struct
- `be/internal/ssh/proxy.go` - Added cwd injection after Shell(), get-cwd handler, stdout marker interception
- `be/internal/ssh/types_test.go` - Tests for Cwd field serialization and CwdResponseMessage

## Decisions Made
- Space-prefixed `cd` command to avoid shell history: ` cd '/path'` instead of `cd '/path'`
- Single-quote wrapping for paths to handle spaces correctly in shell commands
- Path sanitization rejects single quotes and newlines (T-10-01 injection mitigation)
- `__WTERM_CWD_END__` marker for reliable stdout output parsing — won't appear in normal terminal output
- Buffering stdout while waiting for marker (acceptable UX: brief pause since "Duplicate" is an explicit action)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Backend is ready for frontend integration: frontend can send `cwd` in connect message and `get-cwd` control message
- No blockers for next plan

## Threat Flags

| Flag | File | Description |
|------|------|-------------|
| threat_flag: cwd_injection | be/internal/ssh/proxy.go | cwd field injected into shell stdin via cd command — mitigated by rejecting single quotes and newlines |

## Self-Check: PASSED

- All created/modified files exist
- All commits verified in git log
- `go build ./...` passes
- `go vet ./...` passes
- `go test ./...` passes (all 4 new tests + all existing tests)

---
*Phase: 10-tab-plus-button-popover*
*Completed: 2026-04-29*
