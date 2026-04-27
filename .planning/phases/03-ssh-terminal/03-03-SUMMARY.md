---
phase: 03-ssh-terminal
plan: 03
subsystem: frontend-integration
tags: [tab-bar, password-prompt, reconnect, save-banner, session-wiring, quick-connect]
dependency_graph:
  requires:
    - "03-01 (Backend SSH proxy with connection_id protocol)"
    - "03-02 (useSSHSession hook, TerminalPane, session store)"
  provides:
    - "Fully wired SSH terminal sessions from sidebar and quick-connect"
    - "TabBar, PasswordPrompt, ReconnectOverlay, SaveConnectionBanner components"
  affects:
    - "fe/src/App.tsx"
    - "fe/src/features/connections/components/ConnectionList.tsx"
    - "fe/src/features/connections/components/QuickConnect.tsx"
    - "fe/src/features/terminal/TerminalPane.tsx"
tech_stack:
  added:
    - "Lucide X/WifiOff/Bookmark/KeyRound icons"
    - "TanStack Query invalidation for save banner"
  patterns:
    - "All-sessions-rendered with visibility toggle (no tab-remount disconnect)"
    - "Inline password prompt (not modal) for D-02/D-03"
    - "State machine in TerminalPane: connecting→connected→error/disconnected with overlays"
key_files:
  created:
    - fe/src/components/TabBar.tsx
    - fe/src/features/terminal/PasswordPrompt.tsx
    - fe/src/features/terminal/ReconnectOverlay.tsx
    - fe/src/features/terminal/SaveConnectionBanner.tsx
  modified:
    - fe/src/App.tsx
    - fe/src/features/connections/components/ConnectionList.tsx
    - fe/src/features/connections/components/QuickConnect.tsx
    - fe/src/features/terminal/TerminalPane.tsx
decisions:
  - "All TerminalPanes rendered simultaneously with visibility toggle — keeps WebSocket connections alive when switching tabs (v1 approach before Phase 4 multi-tab management)"
  - "Inline password prompt instead of modal — consistent with terminal pane area UX (D-02/D-03)"
  - "SaveConnectionBanner does NOT include password — user must re-enter later if saving (T-03-08)"
  - "Quick-connect input validates host/username — rejects empty strings and spaces (T-03-09)"
metrics:
  duration: 55 min
  completed: 2026-04-27
  tasks: 2
  files: 8
---

# Phase 03 Plan 03: Frontend Integration Summary

One-liner: Sidebar connections and quick-connect create terminal sessions with tab management; TerminalPane state machine handles password prompts, error retry, reconnection overlay, and save-connection banner.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Create TabBar + PasswordPrompt + ReconnectOverlay + wire sidebar and quick-connect | 94fc424 | TabBar.tsx (new), PasswordPrompt.tsx (new), ReconnectOverlay.tsx (new), ConnectionList.tsx, QuickConnect.tsx, App.tsx |
| 2 | Wire save-connection banner + error handling + reconnection flow | c94a9e5 | SaveConnectionBanner.tsx (new), TerminalPane.tsx |

## Key Changes

### Task 1: TabBar, PasswordPrompt, ReconnectOverlay + wiring

**TabBar** (`components/TabBar.tsx`): Reads sessions from Zustand store, renders horizontal tabs with label + close button. Active tab has distinct `bg-background border-border` style. Clicking a tab switches active session; clicking × removes session (triggers disconnect). Returns null when no sessions exist.

**PasswordPrompt** (`terminal/PasswordPrompt.tsx`): Inline password entry form centered in terminal pane area. Shows "Connect to user@host" heading, password input, Connect/Cancel buttons. Enter key submits. Per D-02/D-03: inline, not modal.

**ReconnectOverlay** (`terminal/ReconnectOverlay.tsx`): Semi-transparent backdrop-blur overlay on disconnected terminal. Shows "Connection to host lost" + Reconnect + Close buttons. Per UI-04: clear reconnection prompt.

**ConnectionList wiring**: Clicking a connection checks for existing session (reuses it) or creates a new SSHSession with `connectionId`, then sets it active. TerminalPane auto-connects via `initialConnect` prop for saved connections with stored password (D-01).

**QuickConnect wiring**: Parses `user@host[:port]` format, validates input (T-03-09: rejects empty/spaces), creates session with `isQuickConnect: true`. TerminalPane shows PasswordPrompt since no `initialConnect` is passed (D-03). Saved connection dropdown selections route through ConnectionList flow.

**App.tsx**: Replaced tab placeholder with `<TabBar />`. Content area renders ALL sessions as `absolute inset-0` divs with `hidden` class on inactive sessions — keeps WebSocket connections alive when switching tabs. Falls back to "No active sessions" placeholder when empty.

### Task 2: SaveConnectionBanner + TerminalPane state machine

**SaveConnectionBanner** (`terminal/SaveConnectionBanner.tsx`): Banner at top of terminal pane after quick-connect disconnect (D-06). Save button calls `connectionsApi.create` with label/host/port/username only — no password (T-03-08). Invalidates TanStack Query cache on success. Ghost variant dismiss button.

**TerminalPane state machine**: Full 5-state rendering:
- `connecting` + no `initialConnect` + no `passwordProvided` → PasswordPrompt (D-02/D-03)
- `connecting` + has `initialConnect` or `passwordProvided` → Spinner (auto-connect via useEffect)
- `connected` → Full @wterm/react Terminal
- `error` → AlertTriangle + error message + Retry button (D-05)
- `disconnected` → Frozen terminal + ReconnectOverlay (UI-04) + optional SaveConnectionBanner for quick-connect (D-06)

## Deviations from Plan

None — plan executed exactly as written.

## Verification Results

```
✓ TypeScript compiles without errors (only pre-existing baseUrl deprecation warning)
✓ TabBar component: 1 export found
✓ PasswordPrompt component: 2 references found
✓ ReconnectOverlay component: 2 references found
✓ SaveConnectionBanner component: 2 references found
✓ ConnectionList uses addSession: 2 references
✓ QuickConnect uses addSession: 3 references
✓ App.tsx uses TabBar/TerminalPane: 4 references
```

## Threat Model Compliance

| Threat | Mitigation | Status |
|--------|-----------|--------|
| T-03-08 | Save banner does NOT send ephemeral password | ✓ Implemented — SaveConnectionBanner only sends host/port/username |
| T-03-09 | Validate parsed host/username | ✓ Implemented — rejects empty strings and spaces in QuickConnect |

## Self-Check: PASSED

- [x] `fe/src/components/TabBar.tsx` exists
- [x] `fe/src/features/terminal/PasswordPrompt.tsx` exists
- [x] `fe/src/features/terminal/ReconnectOverlay.tsx` exists
- [x] `fe/src/features/terminal/SaveConnectionBanner.tsx` exists
- [x] `fe/src/features/terminal/TerminalPane.tsx` exists and updated
- [x] `fe/src/features/connections/components/ConnectionList.tsx` updated
- [x] `fe/src/features/connections/components/QuickConnect.tsx` updated
- [x] `fe/src/App.tsx` updated
- [x] Commit 94fc424 exists
- [x] Commit c94a9e5 exists
