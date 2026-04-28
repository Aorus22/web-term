---
phase: 04-multi-tab-polish
plan: 02
subsystem: fe
tags: [sidebar, connection-form, new-tab]
requires: []
provides: [TAB-01, TAB-02]
affects: [fe/src/App.tsx, fe/src/features/connections/components/ConnectionForm.tsx]
tech-stack: [react, tailwind, shadcn]
key-files: [fe/src/components/NewTabView.tsx, fe/src/App.tsx, fe/src/features/connections/components/ConnectionForm.tsx]
decisions:
  - Moved QuickConnect and ExportImport from sidebar to NewTabView (TAB-01)
  - Simplified sidebar to only show connections and tags (D-01, D-02, D-03)
  - Fixed ConnectionForm padding and spacing for better UX (D-05, D-06)
metrics:
  duration: 20 min
  completed_date: 2026-04-28
---

# Phase 04 Plan 02: Sidebar Cleanup & Connection Form Polish Summary

Implemented sidebar cleanup, created the New Tab selection view, and polished the ConnectionForm to improve the overall user experience and reduce redundancy.

## Key Changes

### New Tab View (TAB-01, TAB-02)
- Created `fe/src/components/NewTabView.tsx` which provides a rich selection view when no sessions are active.
- Integrated `QuickConnect` into the top of the `NewTabView`.
- Added a grid of saved connection cards with hover effects and tag display.
- Integrated `ExportImport` into the footer of the `NewTabView` to maintain its accessibility after removal from the sidebar.
- Clicking a connection card in `NewTabView` immediately starts a new SSH session.

### Sidebar Cleanup (D-01, D-02, D-03)
- Removed `QuickConnect` and `ExportImport` from the sidebar in `fe/src/App.tsx`.
- Simplified sidebar layout to focus on the connection list and tag filters.
- Cleaned up `App.tsx` imports and removed redundant empty state code.

### Connection Form Polish (D-05, D-06)
- Updated `fe/src/features/connections/components/ConnectionForm.tsx` to use `p-6` padding on the form container (previously `py-6`).
- Verified use of `space-y-6` for consistent vertical spacing between input groups.
- Confirmed the sheet opens from the right (`side="right"`) as requested.

## Deviations from Plan

### Auto-added functionality
**1. [Rule 2 - Missing Functionality] Moved ExportImport to NewTabView**
- **Found during:** Task 1
- **Issue:** Removing `ExportImport` from the sidebar without an alternative would make connection management difficult.
- **Fix:** Added `ExportImport` to the footer of `NewTabView`.
- **Files modified:** `fe/src/components/NewTabView.tsx`
- **Commit:** [hash from Task 1]

## Threat Flags
None.

## Self-Check: PASSED
- [x] NewTabView created and integrated: YES
- [x] Sidebar cleaned: YES
- [x] ConnectionForm polished: YES
- [x] TypeScript compiles (modulo deprecation warnings): YES
- [x] Commits made: YES
