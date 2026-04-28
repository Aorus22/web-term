---
status: complete
date: 2026-04-28
---
# Summary: Fix UI Separation

Separated New Tab View from Hosts Page View to respect v0.3.0 architectural separation.

## Changes
- Updated `app-store.ts` to include `new-tab` as a valid sidebar page.
- Reverted `HostsPage.tsx` to a clean management-only view (Grid, Header, Filter).
- Updated `NewTabView.tsx` to restore the "Landing" experience (Welcome Hero, QuickConnect).
- Updated `App.tsx` and `TabBar.tsx` navigation logic to distinguish between starting a new session (New Tab) and managing existing hosts (Hosts Page).
- Fixed a regression where multiple sessions couldn't be opened for the same host.
- Set `new-tab` as the default starting page for a "fresh" experience.

## Verification
- Lint checks passed for modified files.
- Logic confirms separation of concerns between management and usage.
