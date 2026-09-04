# Phase 25 Research: SFTP Dual-Pane Manager

## Executive Summary
Phase 25 implements a full dual-pane SFTP manager in GPUI (`views/sftp.rs`) adhering to requirements `SFTP-01` through `SFTP-04`. It enables side-by-side file browsing between local filesystem and remote SSH servers (or remote-to-remote), interactive breadcrumb navigation, sorting, file CRUD operations, streaming transfers with progress indicators, and drag-and-drop file transfers.

---

## 1. Backend REST API Analysis

The Go backend (`be/internal/api/sftp.go`) exposes a complete set of SFTP and local filesystem endpoints:

| Endpoint | Method | Parameters | Response | Description |
|---|---|---|---|---|
| `/api/sftp/ls` | GET | `connectionId`, `path` | `[]SftpFileInfo` | Lists directory contents with metadata |
| `/api/sftp/mkdir` | POST | `connectionId`, `path` | 201 Created | Creates directory |
| `/api/sftp/rename` | POST | `connectionId`, `oldPath`, `newPath` | 200 OK | Renames or moves a file/folder |
| `/api/sftp/remove` | DELETE | `connectionId`, `path` | 204 No Content | Deletes file or directory |
| `/api/sftp/download` | GET | `connectionId`, `path` | Binary stream | Downloads file content |
| `/api/sftp/upload` | POST | `connectionId`, `path` | `{"transferId": "..."}` | Multipart file upload; returns tracking ID |
| `/api/sftp/transfers` | GET | - | `[]SftpTransferStatus` | List all active/recent transfers |
| `/api/sftp/transfer/status` | GET | `transferId` | `SftpTransferStatus` | Detailed transfer progress |

### Local Filesystem vs Remote Hosts
- When `connectionId` is `"local"` or empty, the backend routes requests to `ssh.LocalFS`, exposing the local filesystem.
- When `connectionId` is a UUID, the backend connects via SSH/SFTP using the saved database credentials.

---

## 2. Desktop GPUI Client Architecture

### Crate Dependencies
1. **`crates/backend-client`:**
   - DTOs: `SftpFileInfo` (`name`, `size`, `mode`, `mod_time`, `is_dir`), `SftpTransferStatus` (`id`, `bytes_transferred`, `total_bytes`, `status`, `error`).
   - Client Methods on `BackendClient`:
     - `sftp_list(connection_id: &str, path: &str) -> Result<Vec<SftpFileInfo>>`
     - `sftp_mkdir(connection_id: &str, path: &str) -> Result<()>`
     - `sftp_rename(connection_id: &str, old_path: &str, new_path: &str) -> Result<()>`
     - `sftp_remove(connection_id: &str, path: &str) -> Result<()>`
     - `sftp_upload(connection_id: &str, path: &str, filename: &str, data: Vec<u8>) -> Result<String>`
     - `sftp_download(connection_id: &str, path: &str) -> Result<Vec<u8>>`
     - `sftp_list_transfers() -> Result<Vec<SftpTransferStatus>>`

2. **`crates/webterm`:**
   - State Structure:
     - `SftpPaneState`:
       - `connection_id: String` ("local" or UUID)
       - `current_path: String`
       - `entries: Vec<SftpFileInfo>`
       - `selected_indices: HashSet<usize>`
       - `sort_column: SftpSortColumn` (Name, Size, Date)
       - `sort_ascending: bool`
       - `show_hidden: bool`
       - `search_filter: String`
       - `loading: bool`
       - `error_message: Option<String>`
     - `SftpManager`:
       - `left_pane: SftpPaneState`
       - `right_pane: SftpPaneState`
       - `active_pane: SftpActivePane` (Left or Right)
       - `transfer_drawer_open: bool`
       - `active_transfers: Vec<SftpTransferStatus>`
       - `modal_state: Option<SftpModalState>` (NewFolder, Rename, DeleteConfirm)
   - Views:
     - `views/sftp.rs`: Main dual-pane view rendering left/right panes, toolbar, breadcrumbs, table, and transfer drawer.
     - `views/nav.rs`: Adds `"📁 Files"` navigation button.

---

## 3. Interaction Patterns & Verification Strategy
- Breadcrumbs: Clicking any segment navigates directly to that path.
- Table: Double-clicking a directory navigates into it.
- Action: Selecting files and clicking "Transfer ➔" initiates download/upload between left and right panes.
- Test Coverage:
  - Unit tests in `backend-client` validating REST API calls against mock server.
  - View logic tests in `webterm` testing breadcrumbs parsing, file sorting, filtering, and transfer state updates.
  - End-to-end integration test against running supervisor backend.
