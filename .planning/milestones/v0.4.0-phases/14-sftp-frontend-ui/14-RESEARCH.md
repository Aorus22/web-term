# Phase 14: SFTP Frontend UI - Research

**Researched:** 2026-05-04
**Domain:** Frontend / SFTP UI
**Confidence:** HIGH

## Summary
Phase 14 focuses on creating the visual framework for a dual-pane SFTP manager. The backend already provides the necessary REST API (`/api/sftp/*`) for listing, downloading, uploading, and managing files on both local and remote (SSH) filesystems. The UI will follow a "Termius-inspired" dual-pane layout using `shadcn/ui` components.

**Primary recommendation:** Use `react-resizable-panels` for the split-pane layout and modularize the directory browser into a reusable component that can be instantiated for both the left and right panes.

## Standard Stack
- **Layout:** `react-resizable-panels` (Standard dependency for shadcn/ui Resizable component).
- **Icons:** `lucide-react` (already in project) for file/folder icons.
- **UI Components:** `shadcn/ui` (Table, ScrollArea, Button, DropdownMenu, Select).
- **State Management:** `zustand` (existing `app-store.ts` or a new specialized `sftp-store.ts`).

## Architecture Patterns
### Dual-Pane Layout
- A container component `SFTPView` will render two `SFTPPane` components separated by a resizable handle.
- Each `SFTPPane` will maintain its own state: `source` (Local vs ConnectionID), `currentPath`, and `selectedFiles`.

### Component Responsibilities
- `SFTPView`: Orchestrates the layout and global SFTP state.
- `SFTPPane`: Manages navigation, path history, and the display of a single filesystem.
- `FileList`: A tabular view of `FileInfo` objects with sorting and double-click navigation.
- `PathBreadcrumbs`: Interactive navigation for the current path.

## Common Pitfalls
- **Path Separators:** Ensure the frontend handles both `/` (Unix/SFTP) and `\` (Windows local) if the backend host is Windows, though the backend's `SFTPHandler` attempts to normalize. Recommendation: Always use `/` for SFTP and let the Go backend handle OS specifics.
- **Selection State:** Managing selections across two panes can be tricky; keep selection state local to each pane.
