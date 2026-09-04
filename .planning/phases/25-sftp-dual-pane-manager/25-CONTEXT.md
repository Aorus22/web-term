# Phase 25: SFTP Dual-Pane Manager - Context

**Gathered:** 2026-09-05
**Status:** Ready for planning
**Mode:** Smart discuss (batch proposals accepted)

<domain>
## Phase Boundary

Phase 25 ports the dual-pane SFTP file manager to the desktop GPUI client at full interaction parity with the web app, speaking the Go backend SFTP REST API (`/api/sftp/...`).

- Requirements:
  - `SFTP-01`: Dual-pane view opens from the sidebar; each pane selects Local Filesystem or a saved host.
  - `SFTP-02`: Directory browsing with breadcrumbs, metadata columns (name, size, modified date), and sorting in both panes.
  - `SFTP-03`: Upload/download/delete/rename/new-folder work with progress indicators; transfers stream.
  - `SFTP-04`: Drag-and-drop works within/between panes and from the OS file manager.

</domain>

<decisions>
## Implementation Decisions

### Dual-Pane Layout & Source Selection (SFTP-01)
- **Navigation Entry:** Add persistent "📁 Files" item in the left sidebar navigation (`views/nav.rs`) that switches the main content area to the dual-pane SFTP manager.
- **Source Selection Per Pane:** Each pane header provides an independent dropdown allowing the user to select either "💻 Local Filesystem" (`connectionId=local`) or any saved SSH host from the database (`client.list_connections()`).
- **Layout:** Default 50/50 side-by-side flex split layout with a visual divider between Left Pane and Right Pane.

### Directory Browsing, Breadcrumbs & Metadata (SFTP-02)
- **Breadcrumbs & Navigation:** Interactive breadcrumb trail at the top of each pane allowing direct click navigation to any ancestor directory, complemented by an "Up ⬆" button, a refresh button, and an editable path input.
- **Metadata Columns & Sorting:** Columns for Name, Size (human-formatted B/KB/MB/GB), and Modified Date. Headers support click-to-sort (Name, Size, Date ascending/descending) with directories always sorted to the top.
- **Hidden Files:** Filter dotfiles by default with a quick "Show Hidden" toggle button in each pane toolbar.

### File Operations, Transfers & Progress (SFTP-03, SFTP-04)
- **Action Triggers:** Dual-layer action support: pane toolbar buttons (New Folder, Upload, Download, Delete, Refresh) plus right-click context menu (Download/Upload to opposite pane, Rename, Delete).
- **Transfer Queue & Drawer:** Collapsible bottom transfer drawer showing active streaming transfers, progress bars, transfer rates, and cancellation, communicating with `/api/sftp/transfers` and `/api/sftp/transfer/status`.
- **Drag-and-Drop:** Support drag-and-drop between panes (dropping file from source pane into target pane's directory initiates streaming transfer) as well as file drops from the host OS file manager.

### the agent's Discretion
- Modal styling for New Folder and Rename prompts adhering to GPUI theme variables.
- Debouncing of search / path filter inputs.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `be/internal/api/sftp.go`: Backend SFTP endpoints:
  - `GET /api/sftp/ls?connectionId=...&path=...` (returns `[]ssh.FileInfo`)
  - `GET /api/sftp/download?connectionId=...&path=...`
  - `POST /api/sftp/upload?connectionId=...&path=...`
  - `POST /api/sftp/mkdir?connectionId=...&path=...`
  - `DELETE /api/sftp/remove?connectionId=...&path=...`
  - `POST /api/sftp/rename?connectionId=...&oldPath=...&newPath=...`
  - `GET /api/sftp/transfers`
- `desktop-gpui/crates/backend-client/src/rest.rs`: REST client infrastructure for making HTTP calls.
- `desktop-gpui/crates/webterm/src/views/nav.rs`: Left navigation rail.
- `desktop-gpui/crates/webterm/src/app_state.rs`: Application state and active view router (`show_hosts_catalog`, active tab, etc.).

### Integration Points
- `desktop-gpui/crates/backend-client`:
  - Add SFTP REST client methods: `sftp_list`, `sftp_mkdir`, `sftp_rename`, `sftp_remove`, `sftp_download_file`, `sftp_upload_file`, `sftp_list_transfers`.
  - Add DTOs: `SftpFileInfo`, `SftpTransferStatus`.
- `desktop-gpui/crates/webterm`:
  - Add `SftpViewState` and dual-pane component (`views/sftp.rs`).
  - Wire navigation item in `views/nav.rs` and view switching in `app_state.rs`.

</code_context>

<specifics>
## Specific Ideas

- Ensure path separators are normalized cleanly between Windows backslashes (`\`) and SFTP forward slashes (`/`).
- Provide immediate visual feedback when starting transfers.

</specifics>

<deferred>
## Deferred Ideas

- In-app file viewer/editor -> Future enhancement.
- Remote-to-remote third-party FXP transfers -> Future enhancement.

</deferred>
