---
phase: 03-ssh-terminal
plan: 02
subsystem: frontend-terminal
tags: [terminal, websocket, ssh, zustand, react-hook]
dependency_graph:
  requires: ["03-01"]
  provides: ["useSSHSession", "TerminalPane", "session-store"]
  affects: ["fe/src/stores/app-store.ts"]
tech_stack:
  added: ["@wterm/react useTerminal hook", "WebSocket binary frames"]
  patterns: ["custom hook for WebSocket lifecycle", "Zustand session state management"]
key_files:
  created:
    - fe/src/features/terminal/types.ts
    - fe/src/features/terminal/useSSHSession.ts
    - fe/src/features/terminal/TerminalPane.tsx
  modified:
    - fe/src/stores/app-store.ts
decisions:
  - Password never stored in hook state — sent once in connect message and discarded (D-07)
  - initialConnect prop pattern for auto-connect on mount instead of useImperativeHandle
  - Auto-select first remaining session when active session is removed
metrics:
  duration: 3 min
  completed: 2026-04-27
  tasks: 2
  files: 4
---

# Phase 03 Plan 02: Frontend Terminal Infrastructure Summary

One-liner: useSSHSession hook manages full WebSocket lifecycle with @wterm/react terminal integration; TerminalPane renders 4 states; Zustand store extended with session CRUD.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Create session types + useSSHSession hook | 2e46e08 | types.ts, useSSHSession.ts |
| 2 | Create TerminalPane + extend app-store | 58f65dd | TerminalPane.tsx, app-store.ts |

## Key Changes

### Task 1: Session types + useSSHSession hook

Created `types.ts` with `SessionStatus`, `SSHSession`, and `ConnectOptions` types — the shared contracts for all terminal session management.

Created `useSSHSession` hook (~170 lines) that encapsulates the full WebSocket lifecycle:
- `connect(opts)` — creates WebSocket, sends connect message, adds session to store
- Handles text JSON messages: `connected`, `error`, `disconnected`
- Handles binary frames: converts ArrayBuffer to Uint8Array, writes to terminal
- `sendData(data)` — encodes user keystrokes and sends as binary WebSocket frame
- `sendResize(cols, rows)` — sends resize JSON message
- `disconnect()` — closes WebSocket, updates store status
- Cleanup on unmount via useEffect return
- Password never stored — sent once in connect message and discarded (D-07 / T-03-06)

### Task 2: TerminalPane + app-store extension

Extended `app-store.ts` with:
- `sessions: SSHSession[]` — tracks all open sessions
- `activeSessionId: string | null` — currently active session
- `addSession()` — adds session and auto-sets as active
- `removeSession()` — removes session, auto-selects first remaining if active was removed
- `updateSession()` — partial update for status changes
- `setActiveSession()` — switch active session

Created `TerminalPane` component (~120 lines) rendering 4 states:
- **connecting**: Spinner + "Connecting to {host}..."
- **connected**: Full-height @wterm/react Terminal with autoResize, cursorBlink, onData, onResize
- **error**: AlertTriangle icon + error message + Retry button
- **disconnected**: "Disconnected from {host}" + Reconnect button
- Accepts `initialConnect` prop for auto-connect on mount

## Deviations from Plan

None — plan executed exactly as written.

## Verification Results

- TypeScript compiles without errors (only pre-existing baseUrl deprecation warning)
- `types.ts` contains SSHSession, SessionStatus, ConnectOptions: 4 matches
- `useSSHSession.ts` contains connect/sendData/sendResize/disconnect: 35 matches
- `TerminalPane.tsx` handles all 4 states: 11 matches
- `app-store.ts` has session state management: 18 matches

## Threat Model Compliance

| Threat | Status | Notes |
|--------|--------|-------|
| T-03-06: Password in hook state | Mitigated | Password sent once in connect message, never stored in ref or state. ConnectOptions ref excludes password for saved connections. |
| T-03-07: Terminal binary flood | Accepted | @wterm/react handles rendering performance |

## Self-Check: PASSED

- [x] `fe/src/features/terminal/types.ts` exists
- [x] `fe/src/features/terminal/useSSHSession.ts` exists
- [x] `fe/src/features/terminal/TerminalPane.tsx` exists
- [x] `fe/src/stores/app-store.ts` updated with sessions
- [x] Commit 2e46e08 exists
- [x] Commit 58f65dd exists
