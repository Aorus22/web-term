# Phase 09 Plan 03: Integrate settings with TerminalPane Summary

Wired terminal settings (theme, font, cursor, type) from the settings store into the `TerminalPane` and `useSSHSession` hook, ensuring the live terminal experience reflects user preferences.

## Key Changes

### Terminal Integration (`fe/src/features/terminal/TerminalPane.tsx`)
- Imported and utilized `useSettings` hook to access application settings.
- Derived terminal properties (theme, cursor blink, font family, font size) from settings.
- Applied settings-derived props to both connected and disconnected `Terminal` component instances.
- Used `cn` utility to apply cursor style classes (`cursor-underline`, `cursor-bar`) and `has-scrollback` class.
- Applied font family and font size via inline CSS `style` prop.

### Connection Flow (`fe/src/features/terminal/useSSHSession.ts` & `types.ts`)
- Added `term` field to `ConnectOptions` interface in `types.ts`.
- Updated `useSSHSession` to include the `term` value in the WebSocket `connect` message for both saved and quick-connect flows.
- Updated `TerminalPane` to pass `settings.terminal_type` in all `connect` calls (auto-connect, retry, password/passphrase connect).

## Deviations from Plan
- **Prompts UI**: The plan mentioned applying settings to "terminal prompt" states. However, `PasswordPrompt` and `PassphrasePrompt` in the current implementation are standard React components that do not use the `@wterm/react` `Terminal` component. Settings were applied to all active `Terminal` component instances (Connected and Disconnected/Frozen states).

## Verification Results

### Automated Tests
- Ran `npx tsc --noEmit` in the frontend directory.
- Results: TypeScript compilation successful (ignoring a pre-existing deprecated `baseUrl` warning in `tsconfig.json`).

## Self-Check: PASSED
- [x] `ConnectOptions.term` field exists and is sent in WebSocket connect messages.
- [x] `TerminalPane` reads settings from `useSettings` hook.
- [x] Font family/size applied via inline CSS style.
- [x] Color theme applied via Terminal `theme` prop.
- [x] Cursor style applied via `className`.
- [x] Cursor blink respects settings toggle.
- [x] Connected and Disconnected terminal states apply settings consistently.
- [x] TypeScript compiles cleanly (minus unrelated `tsconfig` warning).
