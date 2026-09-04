# Phase 15: SFTP Operations & DND - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-18
**Phase:** 15-SFTP Operations & DND
**Areas discussed:** Upload & Download UX, Transfer Progress & Feedback, Multi-file & Folder Support, Operation Confirmations & Errors

---

## Upload & Download UX

| Option | Description | Selected |
|--------|-------------|----------|
| Toolbar button + drag-and-drop | Per-pane toolbar with Upload button (file picker) plus existing drag-and-drop. Standard file manager UX. | ✓ |
| Drag-and-drop only | No upload button — rely purely on inter-pane drag-and-drop and paste. | |
| Upload button in top bar only | Single shared upload button at SFTPView level. | |

| Option | Description | Selected |
|--------|-------------|----------|
| Double-click to download files | Double-click a file = download via browser save dialog. Folders navigate. | |
| Right-click download only | Download only via context menu. Double-click does nothing for files. | ✓ |
| Toolbar download button | Select file(s), then click Download in toolbar. | |

| Option | Description | Selected |
|--------|-------------|----------|
| Upload + New Folder only | Minimal toolbar. Delete/rename stay in context menu. | |
| Full toolbar | Upload, Download, New Folder, Delete, Rename all in toolbar. | ✓ |
| No toolbar actions | All operations via context menu and drag-and-drop only. | |

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, OS drag-in supported | Handle browser drag-and-drop events to accept files from the OS. | ✓ |
| No, file picker only | Only upload via the file picker button. | |

**User's choice:** Full toolbar with OS drag-in support, right-click download only.
**Notes:** User wants all operations accessible via toolbar for discoverability, not just context menu.

---

## Transfer Progress & Feedback

| Option | Description | Selected |
|--------|-------------|----------|
| Toast notifications only | Start/end toasts, no progress bar. Simple. | |
| Inline progress bar in toast | Toast with progress bar that fills as transfer proceeds. | ✓ |
| Dedicated transfer queue panel | Bottom panel listing all transfers with progress bars. | |

| Option | Description | Selected |
|--------|-------------|----------|
| Keep simple — no progress | Operations return success/failure only. | |
| Add progress via SSE/WebSocket | Backend emits progress events during transfer. | ✓ |

| Option | Description | Selected |
|--------|-------------|----------|
| SSE endpoint | Separate /api/sftp/transfer/{id}/progress SSE stream. | ✓ |
| Shared WebSocket channel | Multiplex onto existing terminal WebSocket. | |
| You decide | Let planner/researcher pick. | |

**User's choice:** Inline progress bar in toast, SSE endpoint for backend progress.
**Notes:** User wants real-time progress feedback. SSE chosen over WebSocket for simplicity and separation of concerns.

---

## Multi-file & Folder Support

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, multi-select | Ctrl+click for individual, Shift+click for range. | |
| No, single file only | One file at a time. Simpler implementation. | ✓ |

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, recursive folder transfer | Backend walks folder tree, transfers all files. | |
| Files only, no folder transfer | Users can only transfer individual files. | ✓ |
| Folders for remote-to-remote only | Folder transfer between remote hosts only. | |

**User's choice:** Single-file operations only, no folder transfer.
**Notes:** Keeping scope focused. Multi-select and folder transfer deferred as future enhancements.

---

## Operation Confirmations & Errors

| Option | Description | Selected |
|--------|-------------|----------|
| shadcn AlertDialog | Delete confirmations using AlertDialog component. Consistent with design system. | ✓ |
| Keep window.confirm() | Native browser confirmation dialogs. | |
| Inline undo pattern | No confirmation, toast with Undo button. | |

| Option | Description | Selected |
|--------|-------------|----------|
| Ask before overwrite | AlertDialog asking "File exists. Overwrite?" | ✓ |
| Auto-rename | Automatically rename the new file. | |
| Silent overwrite | Just overwrite without asking. | |

| Option | Description | Selected |
|--------|-------------|----------|
| Destructive toast | Red/destructive variant of shadcn Toast. Auto-dismisses. | ✓ |
| Error banner in pane | Inline error banner at top of pane. | |
| Modal error dialog | shadcn Dialog popup for errors. | |

| Option | Description | Selected |
|--------|-------------|----------|
| Inline rename | Filename cell becomes editable input field in table row. | |
| shadcn Dialog | Small modal dialog with input field. | ✓ |
| Keep window.prompt() | Native prompt. | |

**User's choice:** shadcn AlertDialog for delete/overwrite, destructive toast for errors, shadcn Dialog for rename.

---

## Claude's Discretion

- Toolbar button styling (icon vs text vs both)
- SSE event format and polling granularity
- Toast positioning and auto-dismiss timing

## Deferred Ideas

- Multi-file selection (Ctrl/Shift+click) — potential future phase
- Recursive folder transfer — significant backend work, deferred
