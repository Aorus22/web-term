---
status: complete
date: 2026-04-28
---
# Summary: Relocate Export/Import and Fix NewTabView Spacing

Successfully moved the `ExportImport` component to the `HostsPage` and improved the vertical layout of the `NewTabView`.

## Changes
- **NewTabView.tsx**:
    - Removed `ExportImport` and its container.
    - Adjusted main container alignment to `justify-start` and added `pt-24` to prevent the "Welcome" hero from hugging the header.
- **HostsPage.tsx**:
    - Imported and added the `ExportImport` component to the header section next to the "Hosts" title.
- **Build**: Verified that the changes do not break the build.

## Verification
- `npm run build` passed.
- Visual logic confirmed: Landing page is cleaner with better vertical rhythm; management utilities are consolidated on the management page.
