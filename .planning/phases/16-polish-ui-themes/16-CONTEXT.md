# Phase 16: Polish & UI Cohesion - Context

Phase 16 focuses on elevating the SFTP manager from "functional" to "native-feeling" by adding navigation aids, shortcuts, and improved feedback.

## Current State
- Dual-pane SFTP manager implemented (`DirectoryBrowser.tsx`).
- Basic file operations (Upload, Download, Rename, Delete, Mkdir, Drag-and-Drop) are working.
- Navigation is double-click based, with a ".." entry for parent directories.
- Progress feedback uses `sonner` toasts with SSE tracking.
- Navigation path is shown as a plain string.

## Requirements (from ROADMAP)
- **Interactive Breadcrumbs:** Users should be able to click parts of the path to navigate back.
- **Keyboard Shortcuts:**
  - `Enter`: Open folder / Download file.
  - `Backspace`: Go to parent directory.
  - `Delete`: Trigger delete dialog.
  - `F5` / `Ctrl+R`: Refresh current pane.
  - `Ctrl+C` / `Ctrl+X` / `Ctrl+V`: Copy / Cut / Paste (using existing SFTP clipboard).
- **Refined Progress Indicators:** Beyond toasts, provide a more persistent view of active/historical transfers.
- **Visual Polish:**
  - Improved icons for common file types (images, archives, config files).
  - Row highlighting and selected state refinement.
  - Smooth transitions during navigation.

## Decisions

### D-16-01: Breadcrumb Implementation
We will replace the static path string in `DirectoryBrowser.tsx` with a custom breadcrumb component. It should split the path by `/` and allow clicking any segment.

### D-16-02: Keyboard Shortcuts Hook
Create a local hook `useSFTPShortcuts` inside `fe/src/features/sftp/hooks/` to handle pane-specific shortcuts. This avoids bloating the global keyboard shortcut logic and allows shortcuts to be scoped to the active pane.

### D-16-03: Transfer Progress Panel
Instead of just toasts, we will add a collapsible "Transfers" panel at the bottom of the SFTP view or a dedicated popover/status bar. This will track all active SSE streams managed by `useSFTPTransfer`.

### D-16-04: File Type Icons
Extend the icon logic to use specific icons for:
- Folders (already exists)
- Images (file-image)
- Archives (file-archive)
- Code/Text (file-code/file-text)
- Default (file)

## Discretion
- **Animations:** Use Tailwind's built-in transitions or `tw-animate-css` for subtle entry/exit animations of list items during navigation.
- **Status Bar:** A small status bar at the bottom of each pane showing "X items, Y MB total" could be a nice addition if context allows.
