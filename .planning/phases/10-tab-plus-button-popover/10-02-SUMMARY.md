---
phase: 10-tab-plus-button-popover
plan: 02
subsystem: frontend
tags: [react, typescript, popover, websocket, cwd]

# Dependency graph
requires:
  - 10-01 (backend cwd support, get-cwd handler)
provides:
  - Plus button popover with Duplicate and New Connection options
  - duplicateSession store action for creating duplicate tabs
  - getCwd function for querying current working directory over WebSocket
  - getSessionWebSocket() for cross-component WebSocket access
  - cwd field on ConnectOptions and SSHSession for duplicate tab flow
affects: []

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Global wsMap pattern for cross-component WebSocket access (module-level Map<string, WebSocket>)"
    - "addEventListener-based get-cwd message listener (non-destructive to existing onmessage)"
    - "Popover with controlled open state for menu UX"

key-files:
  created: []
  modified:
    - fe/src/features/terminal/types.ts
    - fe/src/features/terminal/useSSHSession.ts
    - fe/src/features/terminal/TerminalPane.tsx
    - fe/src/stores/app-store.ts
    - fe/src/components/TabBar.tsx

key-decisions:
  - "Module-level wsMap for WebSocket access from TabBar (avoiding Zustand store complexity for non-serializable WebSocket objects)"
  - "addEventListener-based get-cwd listener instead of wrapping onmessage (non-destructive, doesn't interfere with existing message handling)"
  - "Duplicate session auto-connect in TerminalPane via cwd field on SSHSession (leverages existing auto-connect pattern)"
  - "Duplicate falls back to no-cwd on timeout (graceful degradation — starts in home directory)"

patterns-established:
  - "Cross-component WebSocket access via module-level Map export"

requirements-completed: [TAB-03, TAB-04]

# Metrics
duration: 2min
completed: 2026-04-29
---

# Phase 10 Plan 02: Plus Button Popover with Duplicate/New Connection Summary

**Tab bar plus button popover with Duplicate (same host + directory) and New Connection options, backed by WebSocket get-cwd and store-based duplicateSession action**

## Performance

- **Duration:** 2 min
- **Started:** 2026-04-29T02:41:59Z
- **Completed:** 2026-04-29T02:43:52Z
- **Tasks:** 2 (Task 3 is human-verify checkpoint)
- **Files modified:** 5

## Accomplishments
- Added `cwd?: string` field to `ConnectOptions` and `SSHSession` interfaces
- Implemented `getCwd()` function in `useSSHSession` using addEventListener-based message interception
- Added module-level `wsMap` and `getSessionWebSocket()` export for cross-component WebSocket access
- Replaced Plus button with Popover containing Duplicate and New Connection options
- Added `duplicateSession` action to app store that copies all session fields and sets cwd
- Updated TerminalPane auto-connect to handle duplicated sessions with cwd (three paths: saved-connection, key-auth, and duplicate)
- Popover closes on option select and outside click (Popover default behavior)

## Task Commits

1. **Task 1: Add cwd to ConnectOptions and get-cwd handler to useSSHSession** - `aa0db2e`
2. **Task 2: Replace Plus button with popover and implement duplicate/new connection** - `fb856d7`

## Files Created/Modified
- `fe/src/features/terminal/types.ts` - Added cwd to ConnectOptions and SSHSession
- `fe/src/features/terminal/useSSHSession.ts` - Added getCwd(), wsMap, getSessionWebSocket(), cwd in connect message
- `fe/src/features/terminal/TerminalPane.tsx` - Added duplicate session auto-connect path with cwd
- `fe/src/stores/app-store.ts` - Added duplicateSession action, imported generateId
- `fe/src/components/TabBar.tsx` - Replaced Plus button with Popover, added handleDuplicate

## Decisions Made
- Module-level `wsMap` instead of Zustand store for WebSocket references (WebSocket is non-serializable)
- `addEventListener`/`removeEventListener` for get-cwd listener (avoids wrapping/replacing onmessage handler)
- Graceful fallback on get-cwd timeout — duplicate without cwd, starts in home directory
- Three auto-connect paths in TerminalPane: initialConnect (saved), key-auth (no passphrase), and duplicate (has cwd)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Full feature complete: plus button popover with Duplicate and New Connection
- Pending human verification checkpoint (Task 3)
- No blockers

## Self-Check: PASSED

- All modified files exist
- All commits verified in git log
- TypeScript compiles without errors (only tsconfig deprecation warning)

---
*Phase: 10-tab-plus-button-popover*
*Completed: 2026-04-29*
