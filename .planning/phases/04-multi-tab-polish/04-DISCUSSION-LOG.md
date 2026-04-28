# Phase 4: Multi-Tab & Polish - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-28
**Phase:** 04-multi-tab-polish
**Areas discussed:** Tab close cleanup path, Terminal theme sync mechanism, Keyboard shortcut edge cases

---

## Tab Close Cleanup Path

| Option | Description | Selected |
|--------|-------------|----------|
| useSSHSession unmount cleanup | Add useEffect cleanup in useSSHSession that calls disconnect() on unmount. Tab close: removeSession → unmount → cleanup effect → WebSocket closed. React-idiomatic. | ✓ |
| Explicit disconnect-before-remove in TabBar | TabBar shows confirmation, then calls closeTab(id) that disconnects first, then removes. Requires access to disconnect from outside hook. | |
| Disconnect ref registry in store | Store disconnect ref per session in app store. TerminalPane registers on mount, TabBar calls before removeSession. More complex. | |

**User's choice:** useSSHSession unmount cleanup
**Notes:** Also decided: only connected tabs get confirmation tooltip. Connecting/disconnected/error tabs close immediately.

---

## Terminal Theme Sync Mechanism

| Option | Description | Selected |
|--------|-------------|----------|
| Use built-in themes as-is | @wterm/react has theme="light" (built-in) and default dark. No custom overrides needed. Minimal work, zero risk. | ✓ |
| Custom CSS variable overrides | Override --term-bg and --term-fg to match app's oklch() colors exactly. More consistent but adds maintenance burden. | |

**User's choice:** Use built-in themes as-is
**Notes:** Discovered @wterm/react `theme` prop is a string → CSS class `theme-{name}`. Default dark theme is already appropriate. Built-in light theme (`--term-bg: #fafafa`) is clean enough. Also noted `.wterm` default has card-style (padding, border-radius, box-shadow) that needs overriding for full-area terminal display.

---

## Keyboard Shortcut Edge Cases

| Option | Description | Selected |
|--------|-------------|----------|
| Shortcuts always active | Ctrl+T always opens, Ctrl+W always closes, Ctrl+Tab always cycles. Simple mental model. Only skipped when .wterm has focus. | ✓ |
| Smart context-aware shortcuts | Ctrl+W skips when connecting. Ctrl+Tab skips with 1 tab. More nuanced but harder to predict. | |

**User's choice:** Shortcuts always active
**Notes:** Simple rule: shortcuts work everywhere except when terminal content (`.wterm` class) has focus. This includes during password prompts, connecting states, error states, and empty sessions. Per UI-SPEC: show one-time toast explaining shortcuts.

---

## Claude's Discretion

- Animation timing for tab close tooltip
- Toast implementation details
- Tab cycling edge cases (0 or 1 tab)
- Whether to persist sidebar state in localStorage

## Deferred Ideas

None — discussion stayed within phase scope.
