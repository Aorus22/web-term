# Phase 9: Settings Page - Context

**Gathered:** 2026-04-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Settings page for configuring app and terminal preferences: dark/light mode, terminal color theme presets, terminal emulator type (TERM env var), terminal font + font size, cursor style, and scrollback buffer. New sidebar navigation item. All settings persisted in backend SQLite API.

</domain>

<decisions>
## Implementation Decisions

### Settings Navigation
- **D-01:** Settings page accessible via new sidebar item at the bottom (below Port Forwards) with Settings/Gear icon. `sidebarPage` state extended with 'settings' value.
- **D-02:** ThemeToggle removed from TabBar. Dark/light/system switcher lives in Settings page only. TabBar becomes cleaner — only sidebar toggle + tabs remain in header.
- **D-03:** Settings page uses grouped sections layout (like iOS Settings). Sections: "Appearance" (theme mode, terminal color theme), "Terminal" (terminal type, font, cursor, scrollback).
- **D-04:** Each setting row uses justify-between layout — label on the left, current value/control on the right.

### Terminal Color Themes
- **D-05:** Provide 10-15 popular preset themes: Monokai, Solarized Dark, Solarized Light, Dracula, Nord, One Dark, GitHub Dark, GitHub Light, Tokyo Night, Catppuccin, Gruvbox Dark, Ayu Dark, Ayu Light.
- **D-06:** Theme selector displays as a grid of theme cards. Each card shows mini color swatch preview (4-6 colors from the theme) + theme name. Click to select.
- **D-07:** Terminal color theme is independent from app dark/light mode. User can use Nord terminal theme while app is in light mode, etc. App dark/light only affects UI shell (sidebar, header, cards), not terminal content.
- **D-08:** Terminal theme definitions (16 ANSI color values) are hard-coded in frontend as theme preset data. Only the selected theme name is stored in backend.

### Terminal Type Selection
- **D-09:** Standard set of terminal types: xterm, xterm-256-color, xterm-color, vt100, vt220, ansi, linux, screen-256color.
- **D-10:** Terminal type is a global default only — applies to all SSH connections. Default value: xterm-256-color.
- **D-11:** Selected terminal type is sent as TERM env variable when establishing SSH connection (via WebSocket ConnectMessage).

### Terminal Font Settings
- **D-12:** Font selector offers popular monospace fonts: JetBrains Mono, Fira Code, Source Code Pro, Cascadia Code, IBM Plex Mono, MesloLGS, Menlo, Consolas. Web fonts lazy-loaded from Google Fonts when selected.
- **D-13:** Font size slider range: 10-32px, default 14px.
- **D-14:** Font setting row in Settings page shows current font name + size. Clicking opens a modal with: font selector dropdown at top, font size slider below, live preview at bottom.
- **D-15:** Live preview in font modal shows sample terminal output (e.g., `ls -la` result with ANSI colors) using the selected font and size — not just alphabet characters.

### Cursor Style
- **D-16:** Three cursor style options: block (▮), underline (_), bar (|).
- **D-17:** Cursor blink toggle available, default ON (matches current `cursorBlink` behavior).

### Scrollback Buffer
- **D-18:** Scrollback buffer size uses preset dropdown options: 1000 (default), 5000, 10000, 50000, Unlimited.
- **D-19:** Scrollback setting applies to all terminal instances globally.

### Persistence
- **D-20:** All settings stored in backend SQLite via new `/api/settings` API endpoint. One source of truth.
- **D-21:** Settings include: app theme mode, terminal color theme name, terminal type, font family, font size, cursor style, cursor blink, scrollback buffer size.
- **D-22:** Backend settings model: key-value pairs (simple `settings` table with `key` TEXT PRIMARY KEY, `value` TEXT).
- **D-23:** Frontend loads settings on app startup from API. Caches in Zustand store. Updates via API mutation + local state sync.

### Agent's Discretion
- Exact web font loading strategy (Google Fonts CDN vs self-hosted)
- Theme card grid layout (2 columns? 3? responsive?)
- Font modal exact layout and spacing
- Sample terminal text content for font preview
- Settings API error handling (fallback to localStorage?)
- Migration of existing `webterm-theme` localStorage value to backend on first load
- Transition animation when switching terminal themes live

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Existing Theme Implementation
- `fe/src/hooks/use-theme.ts` — Current theme hook (localStorage-based, dark/light/system). Needs refactoring to use backend API.
- `fe/src/components/ThemeToggle.tsx` — Current ThemeToggle in TabBar. Will be REMOVED and replaced with Settings page switcher.
- `fe/src/index.css` — CSS variables for light/dark themes (oklch color system, `.dark` class toggle).

### Terminal Component
- `fe/src/features/terminal/TerminalPane.tsx` — Terminal rendering with `@wterm/react`. Currently accepts `theme` prop ('light' or undefined). Needs extension for: terminal color themes, font family/size, cursor style, scrollback, TERM env var.
- `fe/src/features/terminal/useSSHSession.ts` — WebSocket SSH session hook. ConnectMessage needs `term` field for terminal type.
- `fe/src/features/terminal/types.ts` — SSH session types including ConnectOptions.

### Navigation & Layout
- `fe/src/App.tsx` — Main layout with sidebar + content area. Needs 'settings' sidebar item and SettingsPage component.
- `fe/src/stores/app-store.ts` — Zustand store with `sidebarPage` state. Needs 'settings' added to type union.

### Backend Patterns
- `be/internal/api/routes.go` — Route registration pattern for new `/api/settings` endpoints.
- `be/internal/db/models.go` — Model definitions. New Settings model (key-value).
- `be/internal/api/keys.go` — Example CRUD API pattern to follow for settings API.

### UI Components (reusable)
- `fe/src/components/ui/switch.tsx` — shadcn Switch for toggle settings (cursor blink)
- `fe/src/components/ui/select.tsx` — shadcn Select for dropdown settings (terminal type, scrollback)
- `fe/src/components/ui/dialog.tsx` — shadcn Dialog for font settings modal
- `fe/src/components/ui/slider.tsx` — (needs check if installed) for font size slider

### Prior Phase Context
- `.planning/phases/06-frontend-ui-navigation/06-CONTEXT.md` — Sidebar navigation pattern, page-based content rendering, Zustand sidebarPage state

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **useTheme hook** (`fe/src/hooks/use-theme.ts`): Current localStorage-based theme management. Needs rewriting to use backend API while keeping the same interface for consumers.
- **@wterm/react Terminal** (`fe/src/features/terminal/TerminalPane.tsx`): Supports `theme` prop for light/dark, `cursorBlink` prop, `autoResize`. Will need to accept: font family, font size, cursor style, scrollback, custom color theme.
- **shadcn Switch** (`fe/src/components/ui/switch.tsx`): Ready for toggle settings.
- **shadcn Select** (`fe/src/components/ui/select.tsx`): Ready for dropdown settings.
- **shadcn Dialog** (`fe/src/components/ui/dialog.tsx`): Ready for font settings modal.
- **Zustand store pattern** (`fe/src/stores/app-store.ts`): Established pattern for app-wide state. Settings can live here as cached frontend state.
- **React Query hooks** (`fe/src/features/connections/hooks/useConnections.ts`): Pattern for API data fetching with `useQuery`/`useMutation`.

### Established Patterns
- **Feature-based directory structure**: `fe/src/features/{feature}/components/` and `fe/src/features/{feature}/hooks/`
- **Backend CRUD API**: Routes in `routes.go`, handlers in `api/`, models in `db/models.go`
- **Sidebar page routing**: `sidebarPage` Zustand state controls main content area rendering
- **CSS variable theming**: `.dark` class on `<html>` toggles oklch color variables

### Integration Points
- **Sidebar** (`App.tsx`): Add 'settings' button at bottom of nav, before closing `</nav>`
- **TabBar header** (`App.tsx:124`): Remove `<ThemeToggle />` from header
- **TerminalPane**: Extend Terminal props with font, cursor, scrollback, color theme options
- **useSSHSession**: Add `term` field to ConnectMessage for terminal type (TERM env var)
- **Backend API**: New `/api/settings` GET/PUT endpoints with Settings model
- **App startup**: Load settings from API, populate Zustand cache

</code_context>

<specifics>
## Specific Ideas

- "justify between — label sebelah kiri dan value sebelah kanan" — Settings row layout pattern
- "switcher darkmode lightmode" — toggle/selector for app theme in settings
- "terminal theme kayak termius" — grid of color theme cards with preview, independent from app theme
- "modal yang atas ada selector font yang bawah ada input yang bisa digeser geser" — font settings modal with dropdown + slider
- "preview kayak gimana bentuknya nanti di dalam terminal" — live terminal sample text in font preview modal
- "xterm xterm-256 vanilla" — terminal type options like Termius
- "banyak macemnya kayak yang termius" — multiple terminal color theme presets

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 09-settings-page*
*Context gathered: 2026-04-28*
