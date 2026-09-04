# Parity Audit Matrix: Desktop GPUI vs Web Client

**Audit Date:** 2026-09-05  
**Audited Against:** `PROJECT.md` Validated Requirements (v0.2.0, v0.3.0, v0.4.0) & `REQUIREMENTS.md` (v0.5.0)  
**Status:** 100% Parity Verified (22/22 Web Features + 30/30 Desktop Features)

---

## 1. Web Application Validated Requirements Audit (v0.2.0 – v0.4.0)

Every validated requirement from the web app has been mapped and implemented natively in the GPUI desktop client.

| # | Web Requirement (`PROJECT.md`) | Web Phase | Desktop Crate & Module | Desktop Tests & Verification | Status |
|---|--------------------------------|-----------|------------------------|------------------------------|--------|
| 1 | **WebSocket SSH proxy** (browser → Go backend → SSH target) | v0.2.0 (Phase 01) | `desktop-gpui/crates/backend-client/src/ws.rs` | `tests/terminal_ws_test.rs`<br>Verifies WS connect request, handshake, binary streaming, and control frames. | ✓ Parity |
| 2 | **Terminal rendering** (vim, htop, tmux, curses, truecolor) | v0.2.0 (Phase 02) | `desktop-gpui/crates/terminal/src/` (`render.rs`, `colors.rs`, `view.rs`) | `tests/render_test.rs`, `tests/terminal_state_test.rs`<br>Verifies 24-bit RGB, alternate screens, ANSI styles, and box-drawing. | ✓ Parity |
| 3 | **Password-based SSH auth** with keyboard-interactive support | v0.2.0 (Phase 03) | `desktop-gpui/crates/backend-client/src/types.rs`, `webterm/src/views/hosts.rs` | `tests/connection_api_test.rs`<br>Connection form supports password fields and credentials pass to backend. | ✓ Parity |
| 4 | **Auto-resize terminal** synced with SSH PTY | v0.2.0 (Phase 03) | `desktop-gpui/crates/terminal/src/view.rs`, `webterm/src/views/terminal.rs` | `tests/terminal_state_test.rs`, `tests/terminal_ws_test.rs`<br>Window and element resize events send JSON resize frame (`{cols, rows}`). | ✓ Parity |
| 5 | **Save/edit/delete SSH connections** in SQLite | v0.2.0 (Phase 03) | `desktop-gpui/crates/backend-client/src/rest.rs`, `webterm/src/app_state.rs` | `tests/connection_api_test.rs`, `tests/hosts_view_test.rs`<br>Full CRUD for connections stored in backend SQLite database. | ✓ Parity |
| 6 | **Sidebar with connection list**, tags, quick-connect bar | v0.2.0 (Phase 03) | `desktop-gpui/crates/webterm/src/views/nav.rs`, `views/hosts.rs` | `tests/hosts_view_test.rs`<br>Collapsible navigation rail, tag filtering, search bar, and card grid. | ✓ Parity |
| 7 | **Export/import connections** as JSON | v0.2.0 (Phase 03) | `desktop-gpui/crates/backend-client/src/rest.rs`, `webterm/src/views/hosts.rs` | `tests/connection_api_test.rs`<br>Export and import endpoints roundtrip JSON connections identical to web format. | ✓ Parity |
| 8 | **Multi-tab SSH sessions** with status indicators | v0.2.0 (Phase 04) | `desktop-gpui/crates/webterm/src/tab_manager.rs`, `views/tab_bar.rs` | `tests/session_tab_test.rs`<br>Dynamic tab management with Connected (green), Connecting (amber), and Disconnected (gray/red) indicators. | ✓ Parity |
| 9 | **Dark/light theme** synced with terminal | v0.2.0 (Phase 04) | `desktop-gpui/crates/webterm/src/theme.rs`, `desktop-gpui/crates/terminal/src/colors.rs` | `tests/settings_theme_test.rs`, `tests/palette_update_test.rs`<br>Theme changes propagate to GPUI and terminal renderer palette live mid-session. | ✓ Parity |
| 10 | **Keyboard shortcuts** (Ctrl+T, Ctrl+W, Ctrl+Tab) | v0.2.0 (Phase 04) | `desktop-gpui/crates/webterm/src/main.rs`, `tab_manager.rs` | `tests/session_tab_test.rs`<br>Tab creation, closure, and cycling mapped to native GPUI keystroke actions. | ✓ Parity |
| 11 | **Reconnection handling** on WebSocket disconnect | v0.2.0 (Phase 04) | `desktop-gpui/crates/backend-client/src/ws.rs`, `webterm/src/app_state.rs` | `tests/terminal_ws_test.rs`, `tests/chaos_test.rs`<br>Exponential backoff reconnection with countdown banner and auto-reconnect. | ✓ Parity |
| 12 | **Clean UI** using styled component system | v0.2.0 (Phase 05) | `desktop-gpui/crates/webterm/src/theme.rs`, `gpui_component` | `tests/hosts_view_test.rs`, `tests/sftp_view_test.rs`<br>Pixel-perfect dark/light styling with consistent spacing, borders, and typography. | ✓ Parity |
| 13 | **SSH key-based authentication** (upload, connect, manage) | v0.3.0 (Phase 06) | `desktop-gpui/crates/backend-client/src/rest.rs`, `webterm/src/views/keys.rs` | `tests/keys_view_test.rs`, `tests/connection_api_test.rs`<br>Key pool view, file upload modal, fingerprint calculation, and delete actions. | ✓ Parity |
| 14 | **Sidebar navigation** (Hosts + SSH Keys) | v0.3.0 (Phase 07) | `desktop-gpui/crates/webterm/src/views/nav.rs` | `tests/nav_test.rs`<br>Sidebar rail switching between active views (`Hosts`, `Keys`, `SFTP`, `Forwards`, `Settings`). | ✓ Parity |
| 15 | **Hosts page** with card-based layout and kebab menus | v0.3.0 (Phase 07) | `desktop-gpui/crates/webterm/src/views/hosts.rs` | `tests/hosts_view_test.rs`<br>Host card grid displaying host, port, user, tags, quick connect, edit, and delete options. | ✓ Parity |
| 16 | **SSH Keys page** with key pool management | v0.3.0 (Phase 07) | `desktop-gpui/crates/webterm/src/views/keys.rs` | `tests/keys_view_test.rs`<br>Card list of registered private keys with type badges (RSA, Ed25519) and copy public key actions. | ✓ Parity |
| 17 | **Per-connection auth method selection** (password vs key) | v0.3.0 (Phase 07) | `desktop-gpui/crates/webterm/src/views/hosts.rs` | `tests/hosts_view_test.rs`<br>Radio toggle between Password and SSH Key with key dropdown selector in connection modal. | ✓ Parity |
| 18 | **SSH local port forwarding** (bind remote port to local) | v0.3.0 (Phase 08) | `desktop-gpui/crates/webterm/src/views/forwards.rs`, `backend-client/src/rest.rs` | `tests/forwards_api_test.rs`, `tests/forwards_view_test.rs`<br>Local and reverse port forward rules with start/stop lifecycle and port presets. | ✓ Parity |
| 19 | **Backend session persistence** (survive reloads / re-attach) | v0.3.0 (Phase 11) | `desktop-gpui/crates/backend-client/src/ws.rs`, `webterm/src/app_state.rs` | `tests/terminal_ws_test.rs`<br>Re-attachment to detached sessions via `WsConnectRequest::attach(session_id)`. | ✓ Parity |
| 20 | **Local terminal support** (spawn shell on host) | v0.4.0 (Phase 12) | `desktop-gpui/crates/backend-client/src/ws.rs`, `webterm/src/tab_manager.rs` | `tests/local_smoke_test.rs`<br>Spawns ConPTY on Windows or POSIX PTY on Linux with cwd inheritance and smoke test pass. | ✓ Parity |
| 21 | **Dual-pane SFTP file manager UI** | v0.4.0 (Phase 14) | `desktop-gpui/crates/webterm/src/views/sftp.rs` | `tests/sftp_view_test.rs`<br>Side-by-side local or remote filesystem panes with independent directory navigation. | ✓ Parity |
| 22 | **SFTP file operations** (list, upload, download, delete, rename) | v0.4.0 (Phase 15) | `desktop-gpui/crates/webterm/src/views/sftp.rs`, `backend-client/src/rest.rs` | `tests/sftp_operations_test.rs`, `tests/sftp_integration_test.rs`<br>Full file operations with streaming transfer progress drawer and drag-and-drop. | ✓ Parity |

---

## 2. Desktop v0.5.0 Requirements Audit Matrix

All 30 requirements for milestone v0.5.0 across Phases 20–27 are verified.

| REQ ID | Description | Phase | Implementation Evidence | Test Suite |
|--------|-------------|-------|-------------------------|------------|
| **FOUND-01** | Backend process lifecycle management (supervisor) | 20 | `webterm_supervisor::Supervisor`, `Child` monitoring | `tests/integration.rs` |
| **FOUND-02** | Dynamic port handshake (`BACKEND_PORT:<port>`) | 20 | `supervisor::parse_handshake_line`, dynamic `:0` port | `tests/integration.rs` |
| **FOUND-03** | Startup failure reporting with stderr ring buffer | 20 | `BackendStatus::Failed`, newest-wins 2000 char buffer | `tests/integration.rs` |
| **FOUND-04** | Desktop window geometry persistence | 20 | `webterm_settings::store`, `window_state.rs` | `tests/store_test.rs` |
| **TERM-01** | Native terminal rendering using Alacritty engine | 21 | `alacritty_terminal`, `webterm-terminal` crate | `tests/render_test.rs` |
| **TERM-02** | 24-bit TrueColor and ANSI 256 colors | 21 | `colors.rs` RGB conversion, styled quad batches | `tests/render_test.rs` |
| **TERM-03** | Text selection with mouse drag and clipboard copy | 21 | `mouse.rs` cell mapping, `cx.write_to_clipboard` | `tests/terminal_state_test.rs` |
| **TERM-04** | Scrollback history buffer inspection | 21 | `alacritty_terminal::grid::Scroll`, mouse wheel | `tests/terminal_state_test.rs` |
| **TERM-05** | Monospace font rendering with cell metrics | 21 | JetBrains Mono font asset, cell width/height cache | `tests/render_test.rs` |
| **TERM-06** | Alternate screen buffer support (vim, htop) | 21 | Alternate screen switching and cell updates | `tests/terminal_state_test.rs` |
| **SHELL-01** | SSH connection terminal tab session | 22 | `TerminalSessionManager`, `TerminalWsHandle` | `tests/terminal_ws_test.rs` |
| **SHELL-02** | Tab management (open, close, switch, cycle) | 22 | `TerminalTab`, Ctrl+T, Ctrl+W, Ctrl+Tab shortcuts | `tests/session_tab_test.rs` |
| **SHELL-03** | Visual connection status dots on tabs | 22 | Tab status dot: green, amber countdown, gray/red | `tests/session_tab_test.rs` |
| **SHELL-04** | Automatic reconnection with exponential backoff | 22 | Reconnect backoff (2s, 4s, 6s, 8s, 16s) with re-attach | `tests/terminal_ws_test.rs` |
| **HOSTS-01** | Create, edit, and delete SSH connections | 23 | REST endpoints in `backend-client`, modal dialogs | `tests/connection_api_test.rs` |
| **HOSTS-02** | Connection card grid with search, tags, kebab | 23 | `views/hosts.rs` card view with tag pills and menu | `tests/hosts_view_test.rs` |
| **HOSTS-03** | Quick-connect from New Tab modal | 23 | `views/new_tab.rs` one-click connection list | `tests/hosts_view_test.rs` |
| **HOSTS-04** | Export/import connections as JSON | 23 | Import/export modals and backend REST client | `tests/connection_api_test.rs` |
| **KEYS-01** | SSH key pool management (upload, view, delete) | 23 | `views/keys.rs` key cards, fingerprint display | `tests/keys_view_test.rs` |
| **KEYS-02** | Memory-only passphrase caching for encrypted keys | 23 | `passphrase_cache.rs` session-scoped cache | `tests/keys_view_test.rs` |
| **KEYS-03** | Per-connection auth method selection | 23 | Password vs SSH key radio selector in Host form | `tests/hosts_view_test.rs` |
| **TERM-05\*** | Local Shell launcher via cross-platform PTY | 24 | ConPTY on Windows, POSIX PTY on Linux | `tests/local_smoke_test.rs` |
| **SFTP-01** | Dual-pane file manager with local/remote sources | 25 | `views/sftp.rs` dual pane with source dropdowns | `tests/sftp_view_test.rs` |
| **SFTP-02** | Directory browsing, breadcrumbs, metadata, sort | 25 | Clickable breadcrumbs, column sorting, dotfile filter | `tests/sftp_view_test.rs` |
| **SFTP-03** | Upload, download, rename, delete, new folder | 25 | Streaming transfers, bottom progress drawer | `tests/sftp_operations_test.rs` |
| **SFTP-04** | Drag-and-drop within/between panes & OS drop | 25 | `.on_drag`, `.on_drop::<SftpDraggedItem>`, OS drop | `tests/sftp_integration_test.rs` |
| **FWD-01** | Port forwarding management (local and reverse) | 26 | `views/forwards.rs`, start/stop toggle, port presets | `tests/forwards_view_test.rs` |
| **SET-01** | Theme config, preferences, fixed Alacritty engine | 26 | `views/settings.rs`, mid-session palette sync | `tests/settings_theme_test.rs` |
| **QA-01** | Cross-platform (Windows & Linux), chaos, throughput | 27 | Multi-platform build, chaos tests, perf burst test | `tests/chaos_test.rs` |
| **QA-02** | Parity audit matrix execution vs PROJECT.md | 27 | Recorded in `PARITY-MATRIX.md`, parity test suite | `tests/parity_audit_test.rs` |
| **QA-03** | Single launchable bundle with loopback enforcement | 27 | Bundled packages (Win/Linux), `127.0.0.1` binding | `tests/loopback_test.rs` |

---

## 3. Conclusion

The desktop GPUI implementation preserves every user-facing capability, backend contract, and workflow of the web application while delivering:
1. Native startup and GPU-accelerated rendering through Alacritty and GPUI.
2. Direct local shell execution with platform-native PTYs without extra servers.
3. Enhanced dual-pane SFTP file management with OS file manager drag-and-drop.
4. Tighter local security with loopback binding (`127.0.0.1`), ephemeral dynamic ports, and environment-only secret transmission.
