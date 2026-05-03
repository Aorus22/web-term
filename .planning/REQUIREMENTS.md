# Requirements: Milestone v0.4.0 — Local Terminal & SFTP

## Vision
Expand WebTerm from a pure SSH client into a versatile sysadmin tool by adding local shell access and a high-quality dual-pane SFTP file manager.

## 1. Local Terminal Support

### 1.1 Backend (Go)
- **PTY Spawning:** Ability to spawn a local shell (bash/zsh on Linux, cmd/powershell on Windows) using a PTY library.
- **WebSocket Integration:** Extend the existing WebSocket proxy to handle local PTY I/O.
- **Environment:** Respect environment variables and current working directory of the backend process.

### 1.2 Frontend (React)
- **New Tab View:** "Local Terminal" must be the first and most prominent option.
- **Connection Type:** Support a "local" connection type that doesn't require SSH credentials.
- **UI:** Reuse the existing `@wterm/react` TerminalPane for local sessions.

## 2. Dual-Pane SFTP Manager

### 2.1 Backend (Go)
- **SFTP Client:** Use `pkg/sftp` to establish SFTP sessions over existing SSH connections.
- **Local FS Driver:** Implement a filesystem driver for the local host.
- **APIs:**
    - `GET /api/sftp/list?connection_id=X&path=Y`: List directory contents.
    - `POST /api/sftp/upload`: Handle file uploads.
    - `GET /api/sftp/download`: Handle file downloads.
    - `DELETE /api/sftp/file`: Delete file/folder.
    - `PATCH /api/sftp/rename`: Rename/move file.

### 2.2 Frontend (React)
- **Navigation:** New "SFTP" item in the sidebar.
- **Layout:** Dual-pane (left/right) split view.
- **Source Selection:** Each pane has a dropdown to select the source:
    - "Local Filesystem"
    - List of saved SSH hosts.
- **Directory Browser:**
    - Breadcrumb navigation.
    - File list with icons (folder vs file type).
    - Metadata columns: Name, Size, Modified Date.
    - Sorting by name/size/date.
- **File Operations:**
    - Toolbar for Upload, Download, New Folder, Delete.
    - Context menu (Right-click) for common actions.
- **Inter-Pane Operations:**
    - Drag-and-drop file(s) from one pane to another (initiates transfer).
    - Transfer progress indicators/toasts.

## 3. UI/UX Polish
- **Termius-inspired Design:** Clean cards, subtle gradients, high-quality icons (Lucide/Radix).
- **Responsive Layout:** SFTP panes should resize gracefully.
- **Keyboard Shortcuts:** Support common file manager shortcuts (e.g., F5 to refresh, Delete to delete).

## 4. Technical Constraints
- **Performance:** Directory listing should be fast/cached.
- **Security:** SFTP sessions must use the same credentials/keys as SSH sessions.
- **Concurrency:** Support multiple simultaneous file transfers.
