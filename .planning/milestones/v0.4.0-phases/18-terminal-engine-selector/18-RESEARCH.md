# Phase 18: Terminal Engine Selector - Research

**Researched:** 2026-05-06
**Domain:** Frontend / Terminal Emulation
**Confidence:** HIGH

## Summary

The goal of this phase is to allow users to switch between the current `@wterm/react` engine and `xterm.js` via the Settings page. This provides a fallback for users who might encounter issues with the experimental `@wterm/react` engine or who prefer the established features and stability of `xterm.js`.

The implementation will involve creating a `TerminalWrapper` component that abstracts the underlying engine, updating the settings persistence layer to store the engine choice, and integrating `xterm.js` with its standard addons (`FitAddon`).

**Primary recommendation:** Introduce a `TerminalEngine` abstraction that maps generic terminal events (`onData`, `onResize`, `write`) to engine-specific implementations, ensuring that `useSSHSession` remains decoupled from the specific rendering library.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Terminal Rendering | Browser / Client | — | Purely a client-side concern handled by the terminal library. |
| Engine Abstraction | Browser / Client | — | Frontend component architecture for swapping engines. |
| Settings Persistence | API / Backend | Browser / Client | User preferences are stored in the database but managed via the Settings UI. |
| WebSocket Transport | API / Backend | Browser / Client | The backend manages the SSH/PTY, the frontend handles the WebSocket client. |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| @xterm/xterm | 6.0.0 | Terminal Emulation | Industry standard for web-based terminals. |
| @xterm/addon-fit | 0.11.0 | Auto-resizing | Essential for fitting terminal to container. |
| @wterm/react | 0.1.9 | Primary Terminal | Current modern engine with native DOM rendering. |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|--------------|
| @xterm/addon-web-links | 0.11.0 | Clickable Links | To match feature parity with wterm. |
| @xterm/addon-webgl | 0.11.0 | GPU Acceleration | For high-performance rendering in xterm.js. |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| @xterm/xterm | term.js | term.js is deprecated; xterm.js is the successor. |
| Manual Fit | @xterm/addon-fit | FitAddon is well-tested and handles edge cases for font measurement. |

**Installation:**
```bash
npm install @xterm/xterm @xterm/addon-fit @xterm/addon-web-links @xterm/addon-webgl
```

## Architecture Patterns

### System Architecture Diagram
The `TerminalPane` uses `TerminalWrapper`, which selects the engine based on settings. Both engines receive data from the same `useSSHSession` hook.

```
[ TerminalPane ]
      |
      +-- [ useSSHSession ] <--> [ WebSocket / Backend ]
      |
      +-- [ TerminalWrapper ]
                |
        +-------+-------+
        |               |
 [ WTermEngine ] [ XTermEngine ]
```

### Recommended Project Structure
```
fe/src/features/terminal/
├── components/
│   ├── TerminalWrapper.tsx   # Abstraction layer
│   ├── WTermEngine.tsx       # @wterm/react implementation
│   └── XTermEngine.tsx       # xterm.js implementation
└── hooks/
    └── useTerminalEngine.ts  # Shared logic for engine management
```

### Pattern 1: Unified Terminal Handle
To allow `useSSHSession` (or its consumer) to interact with the terminal without knowing the implementation, we define a common handle:

```typescript
export interface TerminalHandle {
  write: (data: Uint8Array | string) => void;
  focus: () => void;
  clear: () => void;
}
```

### Anti-Patterns to Avoid
- **Leaky Abstractions:** Do not pass `@wterm/react` specific hooks (like `useTerminal`) to the generic `useSSHSession` hook.
- **Redundant Mouse Logic:** `xterm.js` handles xterm-style mouse tracking internally; do not apply `useTerminalMouse` to the `XTermEngine`.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Font measurement | Manual canvas measure | `FitAddon` | Handles sub-pixel rendering and font loading delays better. |
| Mouse Tracking | Event listeners | `xterm.js` Internal | `xterm.js` has robust support for 1000/1002/1003/1006 mouse protocols. |
| Link Detection | Regex matching | `WebLinksAddon` | Handles complex URLs and terminal-specific escaping. |

## Common Pitfalls

### Pitfall 1: Font Loading Delays
**What goes wrong:** Terminal fits before fonts are loaded, leading to incorrect column/row calculations.
**How to avoid:** Use `document.fonts.ready` or a slight delay before the initial `fit()`.

### Pitfall 2: Resize Loops
**What goes wrong:** `FitAddon.fit()` triggers an internal resize, which might trigger an external `ResizeObserver`, causing a loop.
**How to avoid:** Debounce the `onResize` callback and only send to the backend if dimensions actually changed.

### Pitfall 3: Reconnection State
**What goes wrong:** Switching engines while a session is active will lose the local terminal buffer.
**How to avoid:** Since the backend supports session attachment (`type: attach`), the UI should ideally trigger a re-attach (re-opening the WebSocket) when the engine is swapped. This ensures the terminal is populated with the latest session state from the PTY.

## Code Examples

### XTermEngine Implementation
```typescript
// Source: [VERIFIED: xterm.js documentation]
import { useEffect, useRef, useImperativeHandle, forwardRef } from 'react';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import '@xterm/xterm/css/xterm.css';

export const XTermEngine = forwardRef((props, ref) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const termRef = useRef<Terminal | null>(null);

  useEffect(() => {
    const term = new Terminal({
      cursorBlink: props.cursorBlink,
      fontFamily: props.fontFamily,
      fontSize: props.fontSize,
      theme: props.theme,
    });
    
    const fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    
    term.open(containerRef.current!);
    fitAddon.fit();
    
    term.onData(props.onData);
    term.onResize(({ cols, rows }) => props.onResize(cols, rows));
    
    termRef.current = term;
    
    return () => term.dispose();
  }, [props.cursorBlink, props.fontFamily, props.fontSize, props.theme]);

  useImperativeHandle(ref, () => ({
    write: (data) => termRef.current?.write(data),
    focus: () => termRef.current?.focus(),
  }));

  return <div ref={containerRef} style={props.style} className={props.className} />;
});
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `xterm` package | `@xterm/xterm` | v5.3.0+ | Moved to scoped packages on npm. |
| Hand-rolled Fit | `addon-fit` | — | More reliable across browsers. |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `xterm.js` 6.0.0 is stable for this project | Standard Stack | Minor API changes might exist. |
| A2 | `@wterm/react` 0.1.9 is the preferred default | Summary | User might prefer xterm.js as default. |

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Go | Settings Backend | ✓ | 1.26 | — |
| Node.js | Frontend Build | ✓ | 24.14 | — |
| npm | Dependency Mgmt | ✓ | 11.11 | — |

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Vitest |
| Config file | `fe/vite.config.ts` |
| Quick run command | `npm test` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command |
|--------|----------|-----------|-------------------|
| ENGINE-01 | Toggle engine in settings | UI | Manual UAT |
| ENGINE-02 | xterm.js rendering | Smoke | `npm run build` (check bundles) |
| ENGINE-03 | Theme parity | Visual | Manual UAT |

## Security Domain

### Applicable ASVS Categories
| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | yes | Validate `terminal_engine` whitelist in Go backend. |

### Known Threat Patterns for Terminal
| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Settings Injection | Tampering | Whitelist allowed setting keys in the backend. |

## Sources

### Primary (HIGH confidence)
- `@xterm/xterm` Documentation - API and Addon usage.
- `be/internal/api/settings.go` - Backend settings implementation.
- `fe/src/features/terminal/TerminalPane.tsx` - Current terminal integration.

### Secondary (MEDIUM confidence)
- Community patterns for React + xterm.js integration.

## Metadata
**Confidence breakdown:**
- Standard stack: HIGH
- Architecture: HIGH
- Pitfalls: HIGH

**Research date:** 2026-05-06
**Valid until:** 2026-06-06
