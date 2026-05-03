---
phase: 10-tab-plus-button-popover
verified: 2026-04-29T03:00:00Z
status: human_needed
score: 4/4 must-haves verified
overrides_applied: 0
gaps: []
human_verification:
  - test: "Connect to an SSH host, navigate to a directory (e.g., /tmp), and click Duplicate."
    expected: "A new tab opens in /tmp."
    why_human: "Portability of the backend's 'readlink /proc/<pid>/cwd' approach needs verification across different remote OS types (Linux, macOS, BSD)."
  - test: "Open the plus button popover and click outside it."
    expected: "The popover closes."
    why_human: "UI interaction feel and outside-click behavior require manual verification."
---

# Phase 10: Tab Plus Button Popover Verification Report

**Phase Goal:** Add a plus button popover on the tab bar with two options — "Duplicate" and "New Connection".
**Verified:** 2026-04-29
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| #   | Truth   | Status     | Evidence       |
| --- | ------- | ---------- | -------------- |
| 1   | Plus button visible on tab bar | ✓ VERIFIED | `fe/src/components/TabBar.tsx` renders `<Plus>` button. |
| 2   | Popover opens with "Duplicate" and "New Connection" | ✓ VERIFIED | `TabBar.tsx` uses `Popover` component with two buttons. |
| 3   | "Duplicate" creates new tab with same host + CWD | ✓ VERIFIED | `handleDuplicate` in `TabBar.tsx` queries CWD and calls `duplicateSession`. |
| 4   | "New Connection" opens new connection flow | ✓ VERIFIED | `New Connection` button calls `setActiveSession(null)` and `setSidebarPage('new-tab')`. |
| 5   | Popover closes on select or outside click | ✓ VERIFIED | `setPlusPopoverOpen(false)` called on click; `Popover` handles outside click. |

**Score:** 4/4 truths verified

### Required Artifacts

| Artifact | Expected    | Status | Details |
| -------- | ----------- | ------ | ------- |
| `fe/src/components/TabBar.tsx` | Plus button + Popover | ✓ VERIFIED | Implements the UI and duplication logic. |
| `fe/src/features/terminal/useSSHSession.ts` | getCwd and wsMap | ✓ VERIFIED | Exposes WebSocket access and CWD query functionality. |
| `fe/src/stores/app-store.ts` | duplicateSession action | ✓ VERIFIED | Handles store updates for duplicated sessions. |
| `be/internal/ssh/proxy.go` | get-cwd handler | ✓ VERIFIED | Implements `get-cwd` message handling using `readlink`. |

### Key Link Verification

| From | To  | Via | Status | Details |
| ---- | --- | --- | ------ | ------- |
| `TabBar.tsx` | `useSSHSession.ts` | `getSessionWebSocket` | ✓ WIRED | Correctly retrieves the active session's WebSocket. |
| `TabBar.tsx` | `app-store.ts` | `duplicateSession` | ✓ WIRED | Correctly triggers the session duplication in store. |
| `useSSHSession.ts` | Backend | WebSocket "get-cwd" | ✓ WIRED | Sends the request and handles the "cwd" response. |
| `TerminalPane.tsx` | `useSSHSession.ts` | `connect(opts)` | ✓ WIRED | Passes `session.cwd` to the connection options. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| `TabBar.tsx` | `cwd` | `get-cwd` message | ✓ FLOWING | Uses `readlink` on remote shell PID. |
| `TerminalPane.tsx` | `initial CWD` | `ConnectMessage.Cwd` | ✓ FLOWING | Backend executes `cd '...' && exec ...` |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| Backend CWD retrieval | grep in `proxy.go` | Uses `readlink /proc/%d/cwd` | ✓ PASS |
| Connection with CWD | grep in `proxy.go` | Uses `cd '%s' && exec -l $SHELL` | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ---------- | ----------- | ------ | -------- |
| TAB-01 | 10-02 | Plus button visible | ✓ SATISFIED | TabBar.tsx implementation. |
| TAB-02 | 10-02 | Duplicate logic | ✓ SATISFIED | handleDuplicate and duplicateSession wired. |
| TAB-03 | 10-02 | New Connection logic | ✓ SATISFIED | Directs to new-tab view. |
| TAB-04 | 10-02 | Popover behavior | ✓ SATISFIED | Shadcn Popover integration. |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| `fe/src/components/TabBar.tsx` | 64 | Potential listener leak | ℹ️ INFO | If `get-cwd` times out, the message listener is not removed. Minimal impact. |
| `be/internal/ssh/proxy.go` | - | Summary Discrepancy | ⚠️ WARNING | 10-01-SUMMARY claims marker-based interception, but code uses `readlink`. `readlink` is better for UX but less portable. |

### Human Verification Required

1. **Duplicate with CWD**: Verify that navigating to a subdirectory on a Linux remote and duplicating results in the new tab starting in that same directory.
2. **Mac/BSD Remote Support**: Verify behavior when connected to a non-Linux remote (where `/proc` may not exist). Expected: Fallback to home directory (`/` in code).
3. **UI/UX**: Verify that the popover appears correctly and closes as expected.

### Gaps Summary

The core functionality is well-implemented and correctly wired. The duplication logic uses a `readlink`-based approach on the backend which provides a cleaner user experience than marker-based interception (no visible output), though it may be less portable across remote operating systems. A minor technical discrepancy was noted between the `10-01-SUMMARY.md` claims and the final implementation.

---
_Verified: 2026-04-29_
_Verifier: gsd-verifier_
