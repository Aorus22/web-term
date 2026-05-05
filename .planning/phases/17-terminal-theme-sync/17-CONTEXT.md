# Phase 17: Terminal Theme Sync Context

## Goal
Synchronize the application's visual theme (sidebar, tabs, panels) with the selected terminal theme.

## Current State
- Terminal themes are defined in `fe/src/features/settings/data/terminal-themes.ts`.
- App theme (Light/Dark/System) is managed via `fe/src/hooks/use-theme.ts`.
- User settings including `terminal_color_theme` are managed via `fe/src/features/settings/hooks/useSettings.ts`.
- UI uses Tailwind/Shadcn variables defined in `fe/src/index.css`.

## Requirements
- When a user selects a terminal theme, the application's background, foreground, and accent colors should adapt to match that theme.
- The sidebar, header, and content areas should all reflect the terminal's color palette.
- Maintain accessibility (contrast) across all themes.
- Support smooth transitions when switching themes.

## Proposed Mapping
Terminal themes provide a 16-color palette + background/foreground/cursor.

| App Variable | Terminal Color | Rationale |
|--------------|----------------|-----------|
| `--background` | `background` | Primary background color |
| `--foreground` | `foreground` | Primary text color |
| `--card` | `background` | Card background |
| `--popover` | `background` | Popover background |
| `--primary` | `blue` (color 4) | Primary action color |
| `--primary-foreground`| calculated | Contrast with primary |
| `--secondary` | `black` (color 0) | Secondary background/elements |
| `--muted` | `black` (color 0) | Muted backgrounds |
| `--muted-foreground` | `brightBlack` (8) | Muted text |
| `--accent` | `brightBlack` (8) | Hover/Active states |
| `--border` | `brightBlack` (8) | Component borders |
| `--sidebar` | `background` | Sidebar background |
| `--sidebar-foreground` | `foreground` | Sidebar text |
| `--sidebar-accent` | `brightBlack` (8) | Sidebar active item |

## Implementation Strategy
- Create an `AppThemeProvider` that subscribes to both `useTheme` (for mode) and `useSettings` (for terminal theme).
- Calculate the CSS variables and apply them to `document.documentElement` via inline styles.
- Since inline styles have highest specificity, they will override the values in `index.css`.
- Determine if `resolvedTheme` (light/dark) should be forced based on the terminal theme's background luminance.

## Accessibility
- Use a utility to determine if a color is light or dark (luminance check).
- Ensure `primary-foreground` is either white or black based on the `primary` (blue) color's luminance.
