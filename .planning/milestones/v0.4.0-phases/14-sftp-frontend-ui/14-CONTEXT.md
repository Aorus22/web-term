# Phase 14: SFTP Frontend UI - Context

**Gathered:** 2026-05-04

## Decisions
- **D-01:** Use `shadcn/ui`'s Resizable component (based on `react-resizable-panels`) for the dual-pane split.
- **D-02:** Implement a reusable `DirectoryBrowser` component to ensure consistency between the left and right panes.
- **D-03:** "SFTP" will be added as a primary navigation item in the sidebar, opening a view that persists independently of terminal sessions.
- **D-04:** Use `lucide-react` for file type icons (folder, file, image, archive, etc.).

## Agent's Discretion
- Precise layout of the file metadata columns (Name, Size, ModTime).
- The specific implementation of the "Source Selector" (dropdown vs combo box).
- Whether to use a new Zustand store or keep SFTP state local to the components.

## Deferred Ideas
- **Drag-and-Drop:** Inter-pane drag-and-drop is deferred to Phase 15.
- **File Operations:** Upload/Download/Delete/Rename logic (UI triggers) are deferred to Phase 15, though basic listing and navigation (double-click) are in scope for Phase 14.
