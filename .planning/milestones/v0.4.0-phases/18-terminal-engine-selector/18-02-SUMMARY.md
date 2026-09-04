---
phase: 18-terminal-engine-selector
plan: 02
subsystem: terminal
tags: [terminal, engine, wterm, xterm]
requires: [ENGINE-03]
provides: WTermEngine, XTermEngine, TerminalWrapper components
affects: [fe/src/features/terminal/components/*]
tech-stack:
  added: ["@xterm/xterm", "@xterm/addon-fit", "@xterm/addon-web-links", "@xterm/addon-webgl"]
  patterns: ["Engine abstraction", "useImperativeHandle for ref forwarding"]
key-files:
  created:
    - path: "fe/src/features/terminal/components/WTermEngine.tsx"
      description: "Wrapper for @wterm/react with TerminalHandle interface"
    - path: "fe/src/features/terminal/components/XTermEngine.tsx"
      description: "xterm.js implementation with addons"
    - path: "fe/src/features/terminal/components/TerminalWrapper.tsx"
      description: "Engine selector component switching between WTerm and XTerm"
key-decisions:
  - "Both engines implement TerminalHandle interface for consistent API"
  - "XTermEngine handles mouse tracking natively (no useTerminalMouse needed)"
  - "TerminalWrapper uses forwardRef to expose unified interface"
requirements-completed: [ENGINE-03]
---
# Phase 18 Plan 2: Terminal Engine Abstraction Summary

## Overview
Implemented the specific terminal engine components (WTerm and XTerm) and a wrapper that abstracts them for use in the main terminal pane.

## Tasks Executed
1. **Create WTermEngine Component** - forwardRef component wrapping @wterm/react, uses useTerminalMouse for mouse tracking, exposes write/focus via useImperativeHandle
2. **Create XTermEngine Component** - forwardRef component using @xterm/xterm with FitAddon, WebLinksAddon, WebglAddon. Native mouse handling.
3. **Create TerminalWrapper Component** - Engine selector that conditionally renders WTermEngine or XTermEngine based on engine prop

## Files Created
- `fe/src/features/terminal/components/WTermEngine.tsx` - @wterm/react wrapper
- `fe/src/features/terminal/components/XTermEngine.tsx` - xterm.js implementation
- `fe/src/features/terminal/components/TerminalWrapper.tsx` - Engine selector

## Verification
- TypeScript compiles: ✓ (deprecation warnings only)
- Both engines implement TerminalHandle interface: ✓

## Deviations from Plan
None - plan executed exactly as written.

**Total deviations:** 0

**Impact:** None - all acceptance criteria met.

## Next Steps
Ready for Plan 18-03: UI Integration into TerminalPane and Settings page