# Phase 15: SFTP Operations & DND - Context

**Gathered:** 2026-05-18
**Status:** Ready for planning

<domain>
## Phase Boundary

Implement file operations (upload, download, delete, rename) with proper UI controls, inter-pane and OS drag-and-drop, transfer progress with SSE, and polished confirmation/error handling. Single-file operations only — no multi-select or folder transfer in this phase.

</domain>

<decisions>
## Implementation Decisions

### Upload & Download UX
- **D-01:** Per-pane toolbar with full action buttons: Upload, Download, New Folder, Delete, Rename. Toolbar sits above the file list alongside the existing source selector dropdown.
- **D-02:** Upload triggered via toolbar button (opens file picker) AND OS drag-in (drag files from Finder/Explorer into a pane) AND inter-pane drag-and-drop.
- **D-03:** Download via right-click context menu only. Double-click remains for folder navigation. Backend `/download` endpoint already returns Content-Disposition attachment.
- **D-04:** OS drag-in supported — handle browser drag-and-drop events to accept files from the OS file manager, using the existing multipart upload API.

### Transfer Progress & Feedback
- **D-05:** Inline progress bar inside a toast notification. Shows filename, progress percentage, and a fill bar. Toast auto-dismisses on completion.
- **D-06:** Backend reports transfer progress via SSE endpoint (`/api/sftp/transfer/{id}/progress`). Separate from the existing terminal WebSocket.
- **D-07:** Transfer flow: frontend initiates transfer → gets a transfer ID → subscribes to SSE for progress → shows progress toast → SSE completes → success/error toast.
- **D-08:** Backend must track bytes transferred during staging→write and emit progress events (percentage, bytes transferred, total bytes).

### Multi-file & Folder Support
- **D-09:** Single-file operations only. No Ctrl+click or Shift+click multi-select in this phase.
- **D-10:** No folder transfer. Users can only transfer individual files. Dragging a folder is a no-op or shows a "not supported" message.

### Operation Confirmations & Errors
- **D-11:** Delete confirmation uses shadcn AlertDialog (replaces current `window.confirm()`).
- **D-12:** Overwrite protection: before transfer/upload, check if destination file exists. If yes, show AlertDialog asking "File exists. Overwrite?"
- **D-13:** Error display uses destructive (red) variant of shadcn Toast. Auto-dismisses after a few seconds.
- **D-14:** Rename uses shadcn Dialog (small modal with input field), replacing `window.prompt()`.
- **D-15:** Success feedback: standard (non-destructive) toast confirming operation completed.

### Claude's Discretion
- Exact styling/layout of toolbar buttons (icon vs text vs both)
- SSE event format and polling granularity
- Toast positioning and auto-dismiss timing

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### SFTP Backend
- `be/internal/api/sftp.go` — Existing SFTP REST API (ls, download, upload, remove, rename, mkdir, transfer endpoints)
- `be/internal/ssh/fs.go` — FileSystem interface (List, Read, Write, Remove, Rename, Mkdir, Stat) and LocalFS/SFTPFS drivers
- `be/internal/ssh/client.go` — SSH/SFTP client creation
- `be/internal/ssh/staging.go` — StagingManager used by transfer endpoint

### SFTP Frontend
- `fe/src/features/sftp/components/DirectoryBrowser.tsx` — Main browser component with existing DND, context menu, and operations
- `fe/src/features/sftp/components/SFTPView.tsx` — Dual-pane resizable layout
- `fe/src/lib/api.ts` — `sftpApi` with all API wrappers including `transfer()` and `downloadUrl()`

### UI Components
- `fe/src/components/ui/` — shadcn components directory (AlertDialog, Dialog, Toast, ContextMenu, etc.)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `sftpApi` in `api.ts`: All backend API wrappers already exist (list, downloadUrl, upload, remove, rename, mkdir, transfer). Frontend just needs to call them from new UI elements.
- `ContextMenu` components: Already imported and wired in DirectoryBrowser.tsx with Cut/Copy/Paste/Delete/Rename handlers.
- `Drag-and-drop handlers`: `handleDragStart` and `handleDrop` already implemented in DirectoryBrowser.tsx for inter-pane transfer via `sftpApi.transfer()`.
- `StagingManager` in backend: Transfer endpoint already stages files server-side before writing to destination.
- shadcn UI library: AlertDialog, Dialog, Toast components available for confirmations and feedback.

### Established Patterns
- `useQuery`/`useQueryClient` from React Query for data fetching and cache invalidation (`refreshDir` pattern).
- `sftpClipboard` state in `useAppStore` (Zustand) for cross-pane cut/copy/paste.
- `window.confirm()`/`window.prompt()`/`alert()` for user interaction (to be replaced with shadcn components).

### Integration Points
- `DirectoryBrowser.tsx`: All new UI (toolbar, progress toasts, dialogs) integrates into this component.
- `SFTPView.tsx`: Two independent `DirectoryBrowser` instances need to communicate transfers (currently independent — may need shared state for progress).
- Backend `SFTPHandler.Transfer()`: Needs modification to accept a transfer ID, track progress, and emit SSE events.
- New SSE endpoint registration in `be/internal/api/routes.go`.

</code_context>

<specifics>
## Specific Ideas

No specific external references — standard file manager UX patterns.

</specifics>

<deferred>
## Deferred Ideas

- Multi-file selection (Ctrl/Shift+click) — potential future phase
- Recursive folder transfer — significant backend work, deferred to future phase

</deferred>

---

*Phase: 15-SFTP Operations & DND*
*Context gathered: 2026-05-18*
