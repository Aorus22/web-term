---
phase: 18-terminal-engine-selector
plan: 03
subsystem: terminal
tags: [terminal, engine, UI, settings]
requires: [ENGINE-01, ENGINE-02, ENGINE-03]
provides: TerminalWrapper in TerminalPane, engine selector in SettingsPage
affects: [fe/src/features/terminal/TerminalPane.tsx, fe/src/features/settings/components/SettingsPage.tsx]
tech-stack:
  added: []
  patterns: ["Engine-based conditional rendering"]
key-files:
  modified:
    - path: "fe/src/features/terminal/TerminalPane.tsx"
      description: "Uses TerminalWrapper with engine from settings"
    - path: "fe/src/features/settings/components/SettingsPage.tsx"
      description: "Added terminal engine dropdown"
key-decisions:
  - "TerminalPane passes settings.terminal_engine to TerminalWrapper"
  - "Settings shows 'wterm (Experimental)' and 'xterm.js (Stable)' options"
  - "Engine switches immediately in both connected and disconnected states"
requirements-completed: [ENGINE-01, ENGINE-02, ENGINE-03]
---
# Phase 18 Plan 3: UI Integration Summary

## Overview
Integrated the terminal engine abstraction into the main terminal pane and provided a UI in the Settings page for users to switch between engines.

## Tasks Executed
1. **Integrate TerminalWrapper into TerminalPane** - Replaced direct @wterm/react Terminal with TerminalWrapper that accepts engine prop from settings
2. **Add Terminal Engine Selector to SettingsPage** - Added select dropdown in Terminal section with wterm (Experimental) and xterm.js (Stable) options

## Files Modified
- `fe/src/features/terminal/TerminalPane.tsx` - Uses TerminalWrapper with engine from settings
- `fe/src/features/settings/components/SettingsPage.tsx` - Added terminal engine dropdown

## Verification
- TypeScript compiles: ✓
- Both connected and disconnected states use TerminalWrapper: ✓

## Checkpoint: Human Verification Required
**What was built:** Terminal engine switching functionality.

**How to verify:**
1. Open the application
2. Go to Settings
3. Change "Terminal Engine" to "xterm.js"
4. Open a terminal tab and verify it renders correctly (xterm.js look)
5. Switch back to "wterm" in settings and verify the terminal updates
6. Verify mouse support (clicking, scrolling) works in both engines

## Deviations from Plan
None - plan executed exactly as written.

**Total deviations:** 0

**Impact:** None - all acceptance criteria met.

## Next Steps
Phase 18 complete - all 3 plans executed.