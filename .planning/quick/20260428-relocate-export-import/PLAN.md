# Plan: Relocate Export/Import and Fix NewTabView Spacing

Move the `ExportImport` component from the `NewTabView` to the `HostsPage` and increase top spacing for the welcome section.

## Tasks
- [ ] Remove `ExportImport` and its container from `fe/src/components/NewTabView.tsx`
- [ ] Add `ExportImport` to the header or utility section of `fe/src/features/hosts/components/HostsPage.tsx`
- [ ] Increase top padding/margin for the "Welcome to WebTerm" section in `fe/src/components/NewTabView.tsx`
- [ ] Verify layout and spacing
