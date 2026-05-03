---
phase: quick
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - fe/src/components/ui/context-menu.tsx
  - fe/src/stores/app-store.ts
  - fe/src/features/sftp/components/DirectoryBrowser.tsx
autonomous: true
requirements: []
must_haves:
  truths:
    - User can right click to open context menu on SFTP items
    - Context menu has Cut, Copy, Paste, Rename, Delete
    - User can drag and drop items between two SFTP panes
  artifacts:
    - path: fe/src/components/ui/context-menu.tsx
      provides: shadcn context menu component
    - path: fe/src/stores/app-store.ts
      provides: clipboard state for sftp
    - path: fe/src/features/sftp/components/DirectoryBrowser.tsx
      provides: drag and drop + context menu logic
  key_links:
    - from: fe/src/features/sftp/components/DirectoryBrowser.tsx
      to: fe/src/lib/api.ts
      via: sftpApi calls for upload/download/rename/remove
---

<objective>
Implement Context Menu (Cut, Copy, Paste, Rename, Delete) and Drag & Drop file transfer across split panes in the SFTP Directory Browser.
Purpose: To enable file management and file transfers between local and remote connections.
Output: Working right-click context menu and HTML5 drag-and-drop between panes.
</objective>

<execution_context>
@$HOME/.gemini/get-shit-done/workflows/execute-plan.md
</execution_context>

<context>
@fe/src/features/sftp/components/DirectoryBrowser.tsx
@fe/src/stores/app-store.ts
@fe/src/lib/api.ts
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add shadcn context-menu component</name>
  <files>fe/src/components/ui/context-menu.tsx</files>
  <action>
    Run `cd fe && npx shadcn@latest add context-menu` or use equivalent CLI tools to add the context-menu component. Wait for it to finish and confirm the file is created.
  </action>
  <verify>
    <automated>test -f fe/src/components/ui/context-menu.tsx</automated>
  </verify>
  <done>Component exists.</done>
</task>

<task type="auto">
  <name>Task 2: Add SFTP clipboard state to app-store.ts</name>
  <files>fe/src/stores/app-store.ts</files>
  <action>
    Add `sftpClipboard` state to `app-store.ts` for cross-pane operations.
    Add interface:
    ```typescript
    export interface SFTPClipboard {
      action: 'cut' | 'copy';
      connectionId: string;
      path: string;
      fileName: string;
      isDir: boolean;
    }
    ```
    Add to AppState:
    ```typescript
    sftpClipboard: SFTPClipboard | null
    setSftpClipboard: (clipboard: SFTPClipboard | null) => void
    ```
    Implement the initial state (`sftpClipboard: null`) and setter in `useAppStore`.
  </action>
  <verify>
    <automated>grep -q "sftpClipboard" fe/src/stores/app-store.ts</automated>
  </verify>
  <done>Store updated with clipboard state.</done>
</task>

<task type="auto">
  <name>Task 3: Implement Context Menu and DND in DirectoryBrowser</name>
  <files>fe/src/features/sftp/components/DirectoryBrowser.tsx</files>
  <action>
    1. Import `ContextMenu`, `ContextMenuTrigger`, `ContextMenuContent`, `ContextMenuItem`, `ContextMenuSeparator` from `@/components/ui/context-menu`.
    2. Import `useAppStore` to access `sftpClipboard` and `setSftpClipboard`.
    3. Update the `<tr>` elements rendering the files to be wrapped in `<ContextMenu>` and `<ContextMenuTrigger asChild>`. Note: do not apply it to the `..` virtual row.
    4. Add `<ContextMenuContent>` containing items: Cut, Copy, Paste, Rename, Delete. Disable Paste if `sftpClipboard` is null.
    5. **Paste Logic**: If pasting within the same `connectionId`, use `sftpApi.rename` (if action was 'cut'). If different connection, fetch from `sftpApi.downloadUrl(clipboard.connectionId, sourcePath)`, convert response to `Blob`, create a `File` object from it, and use `sftpApi.upload(selectedConnection, targetPath, file)`. If action was 'cut', follow up by calling `sftpApi.remove` on the source. Use `useMutation` and invalidate `['sftp']` queries.
    6. **Drag and Drop**: Make `<tr>` elements `draggable={true}`.
       - `onDragStart`: `e.dataTransfer.setData('application/json', JSON.stringify({ connectionId: selectedConnection, path: fullPath, fileName: file.name, isDir: file.isDir, action: 'copy' }))`
       - Add `onDragOver={(e) => e.preventDefault()}` on the `<ScrollArea>` or `<tbody>` to allow drops.
       - `onDrop`: Parse JSON, handle transfer logic similar to Paste.
    7. **Rename/Delete**: Use `window.prompt` for Rename and `window.confirm` for Delete to keep things simple for this MVP.
  </action>
  <verify>
    <automated>grep -q "ContextMenu" fe/src/features/sftp/components/DirectoryBrowser.tsx</automated>
  </verify>
  <done>Table items have context menu and support drag-and-drop file transfers.</done>
</task>

</tasks>

<success_criteria>
- Right clicking an item shows Cut, Copy, Paste, Rename, Delete.
- Dragging a file from one DirectoryBrowser instance to another successfully copies/moves the file.
- `app-store.ts` handles the clipboard state correctly.
</success_criteria>