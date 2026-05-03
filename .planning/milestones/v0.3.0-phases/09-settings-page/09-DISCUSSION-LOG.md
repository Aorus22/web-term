# Phase 9: Settings Page - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-28
**Phase:** 09-settings-page
**Areas discussed:** Settings Navigation, Terminal Color Themes, Terminal Type Selection, Persistence & Scope, Terminal Font Settings, Cursor Style & Blink, Scrollback Buffer

---

## Settings Navigation

| Option | Description | Selected |
|--------|-------------|----------|
| Sidebar item at bottom | New sidebar item di bawah Port Forwards dengan icon Settings/Gear. Mirip Termius. | ✓ |
| TabBar gear icon | Gear icon di sebelah ThemeToggle di TabBar. Klik buka settings. | |
| Both — sidebar + tabbar | Sidebar item + shortcut gear icon di TabBar. | |

| Option | Description | Selected |
|--------|-------------|----------|
| Move to Settings page | ThemeToggle dihapus dari TabBar. Dark/light/system switcher ada di Settings page only. | ✓ |
| Keep both places | Tetap di TabBar untuk quick access + settings page mirror. | |
| TabBar only, settings just shows | Settings page cuma display current theme, toggle tetep di TabBar. | |

| Option | Description | Selected |
|--------|-------------|----------|
| Grouped sections | Sections kayak "Appearance", "Terminal" dengan headers. Mirip iOS Settings. | ✓ |
| Flat list semua | Satu list justify-between tanpa grouping. | |

**Notes:** User wants clean TabBar without ThemeToggle. Settings as a proper page with sections.

---

## Terminal Color Themes

| Option | Description | Selected |
|--------|-------------|----------|
| Popular presets only | 10-15 theme populer: Monokai, Solarized, Dracula, Nord, etc. | ✓ |
| Presets + custom editor | Presets + user bisa edit 16 ANSI colors. Mirip Termius full editor. | |
| Minimal presets (5-8) | Cuma yang paling populer. | |

| Option | Description | Selected |
|--------|-------------|----------|
| Grid of theme cards | Grid cards dengan mini color swatch preview + nama. Mirip VS Code theme picker. | ✓ |
| Dropdown with color preview | Dropdown dengan mini swatch per option. | |
| Dropdown nama saja | Simple dropdown tanpa preview. | |

| Option | Description | Selected |
|--------|-------------|----------|
| Independent | Terminal theme independent dari app dark/light. Nord bisa di light mode app. | ✓ |
| Linked — auto-switch | Terminal theme auto-follow app dark/light. | |
| Default follows app, overrideable | Default follow app, user bisa override. | |

**Notes:** User explicitly wants terminal themes like Termius — independent from app theme, grid display with color preview.

---

## Terminal Type Selection

| Option | Description | Selected |
|--------|-------------|----------|
| Standard set | xterm, xterm-256-color, xterm-color, vt100, vt220, ansi, linux, screen-256color. | ✓ |
| Minimal set | xterm, xterm-256-color, vt100 saja. | |
| Full list + custom | Semua standard + custom TERM input. | |

| Option | Description | Selected |
|--------|-------------|----------|
| Global default only | Satu setting untuk semua koneksi. | ✓ |
| Global default + per-connection override | Default di settings, tiap connection bisa override. | |

**Notes:** User wants standard set only, global default — no per-connection override.

---

## Persistence & Scope

| Option | Description | Selected |
|--------|-------------|----------|
| localStorage only | Sama kayak theme sekarang. No backend changes. | |
| Backend SQLite API | Settings disimpan di backend via API. Persist across browser clear. | ✓ |
| localStorage + sync to backend | Primary localStorage, sync ke backend. | |

| Option | Description | Selected |
|--------|-------------|----------|
| All in backend | Semua settings termasuk terminal theme choice di SQLite. | ✓ |
| Backend + theme color data cached locally | Choice di backend, definisi warna hard-coded di frontend. | |

**Notes:** User wants all settings in backend SQLite. Theme color definitions (16 ANSI values) hard-coded in frontend, only selected theme name stored in backend.

---

## Terminal Font Settings

| Option | Description | Selected |
|--------|-------------|----------|
| Popular monospace fonts | JetBrains Mono, Fira Code, Source Code Pro, Cascadia Code, etc. | ✓ |
| System monospace only | Cuma system monospace. Simple. | |
| System + web fonts lazy-loaded | System default + Google Fonts lazy-load. | |

| Option | Description | Selected |
|--------|-------------|----------|
| 12-24px, default 14px | Range cukup untuk terminal. | |
| 10-32px, default 14px | Range lebih besar untuk accessibility. | ✓ |

| Option | Description | Selected |
|--------|-------------|----------|
| Row item → opens modal | Row 'Font' dengan value, klik → modal dengan selector + slider + preview. | ✓ |
| Inline in settings page | Font selector dan slider langsung di page. | |

| Option | Description | Selected |
|--------|-------------|----------|
| Sample terminal text | Preview dengan `ls -la` result + ANSI colors. Mirip iTerm2. | ✓ |
| Just alphabet + numbers | Simple preview kayak Google Fonts. | |

**Notes:** User specifically described the UX: "saat diklik muncul modal yang atas ada selector font yang bawah ada input yang bisa digeser geser dan ada previewnya". Font settings in modal with live terminal sample preview.

---

## Cursor Style & Blink

| Option | Description | Selected |
|--------|-------------|----------|
| Block, underline, bar | Tiga opsi classic. | ✓ |
| Block + bar only | Dua yang paling umum. | |

| Option | Description | Selected |
|--------|-------------|----------|
| Blink ON by default | Current behavior. User bisa toggle. | ✓ |
| Blink OFF by default | Steady cursor default. | |

**Notes:** User wants all three cursor styles with blink ON as default.

---

## Scrollback Buffer

| Option | Description | Selected |
|--------|-------------|----------|
| Preset options | Dropdown: 1000 (default), 5000, 10000, 50000, Unlimited. | ✓ |
| Number input | Input field angka bebas. | |
| Slider dengan preset ticks | Slider dari 1000 sampai 100000. | |

**Notes:** Preset dropdown for simplicity.

---

## Agent's Discretion

- Web font loading strategy (Google Fonts CDN vs self-hosted)
- Theme card grid layout columns
- Font modal exact layout and spacing
- Sample terminal text content for font preview
- Settings API error handling
- Migration of existing localStorage theme to backend
- Transition animation for live theme switching

## Deferred Ideas

None — all discussed items stayed within phase scope.
