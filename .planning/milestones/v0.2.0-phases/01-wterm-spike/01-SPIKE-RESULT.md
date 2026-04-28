---
phase: 01-wterm-spike
date: 2026-04-27
tester: aorus
---

# Phase 1 Spike Result

## Validation Results

| Test | Description | Result | Notes |
|------|-------------|--------|-------|
| V-01 | Basic connection | PASS | SSH to localhost works, shell prompt appears |
| V-02 | vim | PASS | Editor renders correctly, typing works, clean exit |
| V-03 | htop | PASS | Dashboard renders with colors and bars |
| V-04 | tmux | PASS | Splits, pane navigation, detach/reattach all work |
| V-05 | Resize | PASS | Terminal redraws on browser resize |
| V-06 | Copy/paste | PASS | DOM-based selection works |
| V-07 | curses apps | PASS | nano and similar apps render correctly |
| V-08 | Long output | PASS | Scrollback works, no corruption |

## Issues Found

1. **Colors slightly off (minor):** Terminal colors are not fully accurate, especially in tmux. Readable but not pixel-perfect. Likely a 256-color support or theme mapping issue in @wterm/react v0.1.9.
2. **@wterm/react version downgrade:** Had to use v0.1.9 instead of v0.2.0 due to a `workspace:*` publishing bug in the npm package.

## DECISION: PROCEED with @wterm/react

All critical validation tests (V-01 through V-06) passed. The color issues are cosmetic and do not affect functionality. @wterm/react handles real SSH workloads including vim, htop, tmux, and curses applications.

## Rationale

@wterm/react successfully handles all critical terminal workloads. The DOM-based rendering provides native copy/paste support. Auto-resize via ResizeObserver works correctly. Color accuracy issues are minor and likely fixable with theme configuration or a future library update.

## Implications for Phase 2+

- **Phase 2** uses @wterm/react (v0.1.9 or newer when the publishing bug is fixed)
- WebSocket transport can be built directly into the existing Go proxy pattern
- Terminal themes may need custom color mapping for better accuracy
- No need to fall back to xterm.js — @wterm/react meets all requirements
