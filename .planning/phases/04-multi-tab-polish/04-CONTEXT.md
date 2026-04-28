# Phase 4: Multi-Tab & Polish - Context

**Gathered:** 2026-04-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can work with multiple SSH sessions simultaneously with a polished, keyboard-driven workflow and dark/light theme toggle. This phase enhances the existing tab bar with connection status indicators and close confirmation, adds a theme toggle with terminal color sync, and implements keyboard shortcuts (Ctrl+T/W/Tab) with terminal focus detection.

**Requirements covered:** TAB-01, TAB-02, TAB-03, TAB-04, UI-02, UI-03

**In scope:**
- Tab close confirmation for connected sessions (TAB-03)
- Connection status dots on tabs (TAB-04)
- Dark/light theme toggle with system preference detection (UI-02)
- Terminal theme sync with app theme
- Keyboard shortcuts: Ctrl+T (new tab), Ctrl+W (close tab), Ctrl+Tab (switch tab) (UI-03)
- useSSHSession unmount cleanup (fix zombie WebSocket from Phase 3)

**Out of scope:**
- SSH key authentication (v2)
- Authentication system (v2)
- Mobile responsive design (v2)
- Session recording/playback (v2)

</domain>

<decisions>
## Implementation Decisions

### Tab Close & Session Cleanup
- **D-01:** Add useEffect cleanup in useSSHSession that calls disconnect() when the hook unmounts. When TerminalPane unmounts (because removeSession removes it from the array), the cleanup effect fires and closes the WebSocket. TabBar flow stays simple: removeSession → unmount → cleanup.
- **D-02:** Show disconnect confirmation tooltip only for connected tabs. Connecting, disconnected, and error tabs close immediately without confirmation. Per UI-SPEC: connected tab × click → tooltip with "Disconnect from {host}?" → "Disconnect" / "Cancel" buttons.
- **D-03:** Confirmation tooltip uses shadcn `tooltip` component. Only shows for connected sessions. Non-connected tabs close immediately on × click (same as current behavior).

### Terminal Theme Sync
- **D-04:** Use @wterm/react built-in themes as-is. Dark mode: no theme prop (default is dark). Light mode: pass `theme="light"`. Built-in themes have appropriate colors that are close enough to the app palette — no need for custom CSS variable overrides.
- **D-05:** @wterm/react `theme` prop is a string that becomes CSS class `theme-{name}`. Default dark theme: `--term-bg: #1e1e1e`, `--term-fg: #d4d4d4`. Built-in light theme: `--term-bg: #fafafa`, `--term-fg: #383a42`. Theme changes are reactive (prop-driven).
- **D-06:** Override `.wterm` default styling (padding: 12px, border-radius: 8px, box-shadow) in index.css to remove card-style presentation since terminal fills the main content area. Terminal should have no padding, no border-radius, no box-shadow.

### Theme Toggle (UI-02)
- **D-07:** `useTheme` hook manages system preference detection (`matchMedia`), user preference persistence (`localStorage` key `webterm-theme`), `.dark` class toggling on `<html>`. Per UI-SPEC.
- **D-08:** ThemeToggle component: icon button with dropdown (Light/Dark/System). Uses shadcn `dropdown-menu`. Placed at right end of header bar.
- **D-09:** Dark/light CSS variables already defined in `index.css` with `@custom-variant dark (&:is(.dark *))`. No CSS changes needed for app theme — only `.wterm` overrides for terminal presentation.

### Keyboard Shortcuts (UI-03)
- **D-10:** Shortcuts work everywhere except when terminal content has focus (`.wterm` class check on `document.activeElement`). Per UI-SPEC: when terminal is focused, Ctrl+T/W/Tab pass through to the shell.
- **D-11:** Shortcuts are always active regardless of session state — Ctrl+T works even during connecting, password prompt, error state, or when no sessions exist. Simple mental model.
- **D-12:** Ctrl+T focuses quick-connect input in sidebar. Ctrl+W closes the active tab (with confirmation if connected). Ctrl+Tab / Ctrl+Shift+Tab cycles tabs forward/backward.
- **D-13:** Show a one-time toast on first shortcut use: "Tip: Use Ctrl+T to open, Ctrl+W to close, Ctrl+Tab to switch tabs (works when terminal is not focused)". Track with localStorage flag.

### Agent's Discretion
- Exact animation timing for tab close confirmation tooltip
- Error message wording for edge cases
- Toast component implementation (shadcn sonner vs custom)
- Tab cycling behavior when only 0 or 1 tabs open
- Whether to persist sidebar open/close state in localStorage alongside theme

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase 4 UI Design Contract (APPROVED — must-read)
- `.planning/phases/04-multi-tab-polish/04-UI-SPEC.md` — Detailed layout, spacing, colors, interaction specs, component inventory, file changes summary. ALL visual and interaction decisions are locked here.

### Prior Phase Context (integration decisions)
- `.planning/phases/03-ssh-terminal/03-CONTEXT.md` — Session lifecycle, WebSocket protocol, terminal integration, tab bar basics
- `.planning/phases/02-connection-mgmt/02-CONTEXT.md` — Sidebar layout, shadcn setup, data model, frontend architecture

### Project Configuration
- `.planning/PROJECT.md` — Tech stack, constraints, key decisions
- `.planning/REQUIREMENTS.md` — TAB-01 through TAB-04, UI-02, UI-03 requirements
- `.planning/ROADMAP.md` — Phase 4 success criteria and dependency on Phase 3

### @wterm/react Theme System (discovered during discussion)
- `fe/node_modules/@wterm/dom/src/terminal.css` — CSS variables: `--term-fg`, `--term-bg`, `--term-cursor`, plus 16 ANSI colors. Built-in themes: `.wterm.theme-light`, `.wterm.theme-solarized-dark`, `.wterm.theme-monokai`.
- `fe/node_modules/@wterm/react/dist/Terminal.d.ts` — `theme?: string` prop → CSS class `theme-{name}`

### Existing Codebase (must-read for integration)
- `fe/src/App.tsx` — App shell: sidebar, header with TabBar, session rendering with visibility toggle
- `fe/src/components/TabBar.tsx` — Current tab bar: click to switch, × to close, no status dots, no confirmation
- `fe/src/features/terminal/TerminalPane.tsx` — Terminal component: 4 states (connecting, connected, error, disconnected), uses `<Terminal>` with no theme prop
- `fe/src/features/terminal/useSSHSession.ts` — WebSocket lifecycle hook: connect, sendData, sendResize, disconnect. **Missing unmount cleanup** (D-01 fixes this)
- `fe/src/stores/app-store.ts` — Zustand store: sessions, activeSessionId, add/remove/update/setActive
- `fe/src/index.css` — Dark/light CSS variables already defined with Tailwind v4 `@custom-variant dark`

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`TabBar.tsx`**: Functional tab bar already exists. Needs enhancement: status dots (6px Circle icon), close confirmation tooltip for connected tabs, keyboard shortcut hints on hover. Structure is good — enhance, don't rewrite.
- **`useSSHSession`**: Complete WebSocket lifecycle hook. Returns `{ ref, connect, sendData, sendResize, disconnect, write, focus }`. Needs one addition: useEffect cleanup calling `disconnect()` on unmount (D-01).
- **`app-store.ts`**: Session management fully functional — `sessions`, `activeSessionId`, `addSession`, `removeSession`, `updateSession`, `setActiveSession`. No changes needed for multi-tab.
- **`TerminalPane.tsx`**: Renders 4 session states. Uses `<Terminal ref={ref} autoResize cursorBlink onData onResize />` — needs `theme` prop added for theme sync.
- **shadcn components**: `dropdown-menu`, `button`, `tooltip` already installed — needed for ThemeToggle and tab close confirmation.
- **`index.css`**: `:root` and `.dark` CSS variables already defined. `@custom-variant dark (&:is(.dark *))` already set up.

### Established Patterns
- **Visibility toggle for sessions**: `sessions.map()` with `hidden` class on inactive sessions — all sessions stay mounted, only active is visible. Must continue this pattern.
- **Feature-based structure**: `fe/src/features/terminal/`, `fe/src/features/connections/`, `fe/src/components/`, `fe/src/hooks/`.
- **Zustand for UI state, TanStack Query for server state**: Continue. Theme is UI state (could be Zustand or standalone hook — UI-SPEC says standalone `useTheme` hook).
- **lucide-react for icons**: Already in use. `Circle` icon for status dots, `Sun`/`Moon`/`Monitor` for theme toggle.

### Integration Points
- **TabBar → Session cleanup**: `removeSession(id)` triggers unmount → `useSSHSession` cleanup fires → WebSocket closed. Connected tabs get confirmation tooltip first.
- **useTheme → TerminalPane**: Theme hook provides resolved theme ('light'|'dark'), TerminalPane passes `theme="light"` or omits prop for dark to `<Terminal>`.
- **useTheme → `<html>` class**: `.dark` class toggled on `<html>` — existing CSS variables handle app theme automatically.
- **useKeyboardShortcuts → app-store**: Ctrl+T focuses quick-connect, Ctrl+W calls `removeSession(activeSessionId)`, Ctrl+Tab calls `setActiveSession(nextId)`.

</code_context>

<specifics>
## Specific Ideas

- @wterm/react terminal has card-style styling (padding, border-radius, box-shadow) designed for standalone display — must override to fill content area seamlessly
- Built-in `theme="light"` is clean enough — no need to fight the library with custom CSS overrides
- Tab close confirmation is a tooltip, not a modal — lightweight and non-blocking
- Keyboard shortcuts are simple: check `document.activeElement` against `.wterm` class, skip if terminal has focus

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 04-multi-tab-polish*
*Context gathered: 2026-04-28*
