---
phase: 11-backend-session-persistence
plan: 02
subsystem: frontend
tags: [react, zustand, websocket, persistence]

# Dependency graph
requires:
  - 11-01 (backend session manager, attach protocol, sessions API)
provides:
  - Session rehydration from backend on app startup
  - Automatic re-attachment to active terminal streams
  - UI state for session recovery
affects:
  - App startup flow
  - Terminal connection lifecycle

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Backend session discovery via REST on mount"
    - "Auto-attach protocol for 'detached' sessions"
    - "Loader-based recovery UI"

key-files:
  created: []
  modified:
    - fe/src/features/terminal/types.ts
    - fe/src/lib/api.ts
    - fe/src/stores/app-store.ts
    - fe/src/App.tsx
    - fe/src/features/terminal/useSSHSession.ts
    - fe/src/features/terminal/TerminalPane.tsx

key-decisions:
  - "Use 'detached' status to represent sessions known to backend but not yet attached locally"
  - "Trigger rehydration as soon as backend port is discovered in Tauri"
  - "Implicit re-attach via useEffect in TerminalPane when a session tab is rendered"

patterns-established:
  - "Session rehydration from backend"

requirements-completed: [PERSIST-01, PERSIST-02]

# Metrics
duration: 15min
completed: 2026-04-29
---

# Phase 11 Plan 02: Frontend Re-attachment & UI Recovery Summary

**Implemented the frontend logic to discover, rehydrate, and re-attach to active SSH sessions from the backend, ensuring terminal sessions survive browser refreshes.**

## Accomplishments
- Added `detached` status to `SessionStatus` to represent recovered but unattached sessions.
- Implemented `sessionsApi.list()` to fetch active sessions from the Go backend.
- Added `rehydrateSessions` action to app store to populate the UI with backend-tracked sessions.
- Integrated hydration into `App.tsx` startup sequence (post-port discovery).
- Extended `useSSHSession` with `attach()` function for the re-bind protocol.
- Updated `TerminalPane.tsx` to automatically trigger `attach()` and show a "Resuming session..." indicator.

## Files Modified
- `fe/src/features/terminal/types.ts` - Added `detached` status.
- `fe/src/lib/api.ts` - Added `sessionsApi.list()`.
- `fe/src/stores/app-store.ts` - Added `rehydrateSessions` action.
- `fe/src/App.tsx` - Trigger hydration on mount.
- `fe/src/features/terminal/useSSHSession.ts` - Added `attach()` protocol.
- `fe/src/features/terminal/TerminalPane.tsx` - Auto-attach logic and UI.

## Verification Results
- **Manual Verification**: Reloading the app successfully restores active terminal tabs.
- **Scrollback**: Re-attached terminals receive the last 64KB of output from the backend buffer.
- **Tauri Support**: Works correctly within the Tauri 2.0 environment.

## Next Step
Phase 11 Verification: `/gsd-verify-phase 11`
