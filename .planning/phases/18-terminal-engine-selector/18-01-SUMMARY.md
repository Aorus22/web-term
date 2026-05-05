---
phase: 18-terminal-engine-selector
plan: 01
subsystem: terminal
tags: [terminal, engine, settings, xterm]
requires: [ENGINE-01, ENGINE-02]
provides: terminal_engine setting, TerminalHandle abstraction, xterm.js dependencies
affects: [be/internal/api/settings.go, fe/package.json, fe/src/features/terminal/*, fe/src/features/settings/*]
tech-stack:
  added: ["@xterm/xterm", "@xterm/addon-fit", "@xterm/addon-web-links", "@xterm/addon-webgl"]
  patterns: ["Engine abstraction pattern", "TerminalHandle interface"]
key-files:
  created:
    - path: "be/internal/api/settings.go"
      description: "Added terminal_engine setting with wterm/xterm validation"
    - path: "fe/src/features/terminal/types.ts"
      description: "Added TerminalHandle interface for engine-agnostic terminals"
    - path: "fe/src/features/terminal/useSSHSession.ts"
      description: "Decoupled from @wterm/react, uses local ref for terminal handle"
    - path: "fe/src/features/settings/hooks/useSettings.ts"
      description: "Added terminal_engine to AppSettings with default 'wterm'"
key-decisions:
  - "Used @xterm namespace packages (v5.x) instead of xterm (v4.x) for modern API"
  - "Refactored useSSHSession to use local useRef instead of @wterm/react useTerminal hook"
  - "Added terminal_engine to backend defaultSettings with whitelist validation"
requirements-completed: [ENGINE-01, ENGINE-02]
---
# Phase 18 Plan 1: Terminal Engine Foundation Summary

## Overview
Implemented the foundation for multiple terminal engines by updating backend settings, installing xterm.js dependencies, and decoupling the SSH session logic from specific terminal implementations.

## Tasks Executed
1. **Update Backend Settings** - Added `terminal_engine` setting with validation allowing only "wterm" or "xterm"
2. **Install Frontend Dependencies** - Added @xterm/xterm, @xterm/addon-fit, @xterm/addon-web-links, @xterm/addon-webgl
3. **Define Terminal Abstraction** - Created TerminalHandle interface and refactored useSSHSession to use it

## Files Modified
- `be/internal/api/settings.go` - Added terminal_engine to defaultSettings + validation
- `fe/package.json` - Added xterm.js dependencies
- `fe/src/features/terminal/types.ts` - Added TerminalHandle interface
- `fe/src/features/terminal/useSSHSession.ts` - Removed @wterm/react dependency, uses local ref
- `fe/src/features/settings/hooks/useSettings.ts` - Added terminal_engine to AppSettings

## Verification
- Go backend compiles: ✓
- TypeScript frontend compiles: ✓ (deprecation warnings only)
- npm dependencies installed: ✓

## Deviations from Plan
None - plan executed exactly as written.

**Total deviations:** 0

**Impact:** None - all acceptance criteria met.

## Next Steps
Ready for Plan 18-02: Terminal Engine Abstraction components (WTermEngine, XTermEngine, TerminalWrapper)