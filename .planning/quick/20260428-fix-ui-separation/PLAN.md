# Plan: Fix UI Separation between New Tab and Hosts Page

The recent refactor conflated the New Tab experience with the Hosts management page. The user wants them to remain distinct.

## User Constraints
- New Tab View and Host List View must be different.
- Do not equate them in implementation.

## Strategy
1.  **AppState Update**: Add 'new-tab' as a valid `sidebarPage` in Zustand store.
2.  **HostsPage Reversion**: Revert `HostsPage.tsx` to be a focused host management view (Grid of cards, Header, Filter, FAB) without the "Welcome" hero or `QuickConnect`.
3.  **NewTabView Restoration**: Ensure `NewTabView.tsx` serves as the landing experience (Welcome Hero, QuickConnect, Saved Connections Grid).
4.  **Navigation Integration**:
    *   Update TabBar's "+" button to set `sidebarPage` to 'new-tab' and `activeSessionId` to `null`.
    *   Update Sidebar buttons to set `activeSessionId` to `null` when switching to management pages.
    *   Update `App.tsx` to render `NewTabView` when `sidebarPage === 'new-tab'`.

## Tasks
- [ ] Update `AppState` interface and store in `fe/src/stores/app-store.ts`
- [ ] Revert `fe/src/features/hosts/components/HostsPage.tsx` to management-only view
- [ ] Update `fe/src/App.tsx` to handle 'new-tab' page and navigation logic
- [ ] Update `fe/src/components/TabBar.tsx` to trigger 'new-tab' page
- [ ] Verify separation and navigation flow

