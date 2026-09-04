---
phase: 18-terminal-engine-selector
reviewed: 2026-05-05T16:45:00Z
depth: standard
files_reviewed: 10
files_reviewed_list:
  - be/internal/api/settings.go
  - fe/package.json
  - fe/src/features/terminal/types.ts
  - fe/src/features/terminal/useSSHSession.ts
  - fe/src/features/terminal/TerminalPane.tsx
  - fe/src/features/terminal/components/WTermEngine.tsx
  - fe/src/features/terminal/components/XTermEngine.tsx
  - fe/src/features/terminal/components/TerminalWrapper.tsx
  - fe/src/features/settings/hooks/useSettings.ts
  - fe/src/features/settings/components/SettingsPage.tsx
findings:
  critical: 0
  warning: 6
  info: 3
  total: 9
status: issues_found
---

# Phase 18: Code Review Report

**Reviewed:** 2026-05-05T16:45:00Z
**Depth:** standard
**Files Reviewed:** 10
**Status:** issues_found

## Summary

This phase implements a terminal engine selector allowing users to choose between wterm (experimental) and xterm.js (stable). The implementation includes backend settings validation and frontend terminal engine switching. No critical security vulnerabilities were found. Several code quality issues were identified, primarily around incomplete prop propagation and debug artifacts in production code.

## Critical Issues

No critical issues found.

## Warnings

### WR-01: Incomplete cursor blink setting propagation to WTermEngine

**File:** fe/src/features/terminal/components/WTermEngine.tsx:65
**Issue:** The `cursorBlink` prop is passed to TerminalWrapper but is hardcoded to `true` in WTermEngine, ignoring user settings. The cursorBlink setting from settings page has no effect on WTermEngine.
**Fix:**
```tsx
// Change line 65 from:
cursorBlink={true}
// To:
cursorBlink={cursorBlink ?? true}
```
Or pass cursorBlink through the component props.

### WR-02: Terminal theme prop ignored in XTermEngine

**File:** fe/src/features/terminal/components/XTermEngine.tsx:61-84
**Issue:** The `theme` prop is accepted but a hardcoded theme object is always used instead. User's terminal color theme selection is ignored when using xterm engine.
**Fix:** Use the theme prop to select from predefined themes or implement theme mapping:
```tsx
// Example: Map theme prop to xterm theme colors
const getThemeColors = (themeName: string) => {
  const themes: Record<string, object> = {
    default: { background: '#000000', foreground: '#cccccc', ... },
    // other themes
  }
  return themes[themeName] || themes.default
}
```

### WR-03: Missing cursor style support in XTermEngine

**File:** fe/src/features/terminal/components/XTermEngine.tsx:57-84
**Issue:** The `cursorStyle` prop is passed to TerminalWrapper but XTermEngine does not implement it. Only block cursor is available regardless of user settings.
**Fix:** Add cursorStyle to Terminal options:
```tsx
const terminal = new Terminal({
  // ... existing options
  cursorStyle: cursorStyle || 'block',  // 'block' | 'underline' | 'bar'
})
```

### WR-04: Race condition in WTermEngine terminal ready handling

**File:** fe/src/features/terminal/components/WTermEngine.tsx:60-62
**Issue:** The condition `terminalRef.current === null` may never be true if React's timing causes it to be set earlier, causing handleReady to not be called.
**Fix:**
```tsx
// Change condition to always call handleReady when element is available
ref={(el) => {
  internalRef.current = el
  if (el) {
    handleReady(el)
  }
}}
```

### WR-05: Debug console.log statements in production code

**File:** fe/src/features/terminal/TerminalPane.tsx:38
**File:** fe/src/features/terminal/useSSHSession.ts:65, 254, 298, 335, 344, 358, 366, 396, 410
**Issue:** Multiple console.log statements remain in production code. While useful for debugging, they should be removed or replaced with proper logging in production.
**Fix:** Remove console.log statements or wrap in development-only condition:
```tsx
if (import.meta.env.DEV) {
  console.log(`[TerminalPane:${sessionId}] Render. Status: ${session?.status}`)
}
```

### WR-06: Unsafe type assertion in TerminalPane

**File:** fe/src/features/terminal/TerminalPane.tsx:172, 191
**Issue:** Non-null assertions (`session!`) are used without proper null checks. If session is null at these points, this would cause runtime errors.
**Fix:**
```tsx
// Before using session properties, add null check
if (!session) return

const handlePasswordConnect = (password: string) => {
  setPasswordProvided(true)
  const opts: ConnectOptions = {
    type: 'ssh',
    host: session.host,  // Now safe
    // ...
  }
}
```

## Info

### IN-01: Unused hook dependency in useEffect

**File:** fe/src/features/terminal/TerminalPane.tsx:126
**Issue:** The useEffect has `session?.status` in dependency array but not `session` itself. While this works due to eslint disable, the dependency is incomplete.
**Fix:** Consider including `session` in dependencies or explicitly document why only status is needed.

### IN-02: Type use in WTermEngine

**File:** fe/src/features/terminal/components/WTermEngine.tsx:19, 39
**Issue:** Uses `any` type for internalRef and terminal instance, losing TypeScript safety.
**Fix:** Use proper types from @wterm/react if available, or define interface:
```tsx
// If @wterm/react exports types:
import type { TerminalInstance } from '@wterm/react'
const internalRef = useRef<TerminalInstance | null>(null)
```

### IN-03: Global WebSocket map without cleanup guarantee

**File:** fe/src/features/terminal/useSSHSession.ts:7-11
**Issue:** Global wsMap is used for cross-component state sharing but relies on component cleanup. If cleanup fails or is interrupted, entries may remain in map.
**Fix:** Consider using a WeakMap or adding periodic cleanup mechanism, or document this as a known trade-off.

---

_Reviewed: 2026-05-05T16:45:00Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_