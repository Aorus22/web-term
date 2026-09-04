# Phase 22: SSH Terminal Sessions & Tabs - Context

**Gathered:** 2026-09-04
**Status:** Ready for planning
**Mode:** Autonomous (domain analysis + architectural decisions)

<domain>
## Phase Boundary

Wire live backend terminal sessions into the desktop application:
1. Bidirectional WebSocket terminal transport (`webterm-backend-client`) speaking the backend's binary PTY byte stream + JSON control protocol.
2. Multi-tab manager and UI tab strip in GPUI: open, switch, close, per-tab session state indicators (connecting, connected, disconnected).
3. Reconnection resilience: visual reconnection banners, retry actions, and session re-attachment across WebSocket disconnects and application restarts.
4. Desktop tab shortcuts: Ctrl+T (new tab), Ctrl+W (close tab), Ctrl+Tab / Ctrl+Shift+Tab (cycle tabs), Alt+1..9 (direct tab jump).
5. Local terminal session baseline via backend `/ws` (`connectMsg.ConnectionID = "local"`), establishing the desktop tab workflow ahead of Phase 24's deep ConPTY spike.

Phase 22 builds on Phase 21's verified `TerminalView` and `Terminal` engine. Full connection catalog CRUD and SSH key vault are reserved for Phase 23.

</domain>

<decisions>
## Implementation Decisions

### D-01: WebSocket Client Transport (`webterm-backend-client`)
- Integrate `tokio-tungstenite` (with `connect` feature over unencrypted loopback TCP) into `webterm-backend-client`.
- Connect to `ws://127.0.0.1:{port}/ws`.
- Framing protocol:
  - Initial handshake: Client sends JSON `{"type": "connect", ...}` or `{"type": "attach", "session_id": "..."}`.
  - Server replies: `{"type": "connected", "session_id": "..."}`.
  - Client sends `{"type": "ready"}` to trigger scrollback buffer transmission.
  - Inbound binary frames: piped directly into `Terminal::process_bytes`.
  - Outbound user input bytes: sent as binary WebSocket frames.
  - Outbound control commands: sent as JSON text frames (e.g. `{"type": "resize", "cols": N, "rows": M}`).

### D-02: Session & Tab Orchestration (`TerminalSessionManager`)
- Maintain a collection of active tabs/sessions in `TerminalSessionManager`:
  - `SessionId`: unique backend session identifier.
  - `TabId`: internal UI tab identifier.
  - `Title`: host name or custom session title.
  - `Status`: `Connecting`, `Connected`, `Reconnecting`, `Disconnected(String)`.
  - `TerminalView`: GPUI entity holding the terminal grid and renderer.
- Selected tab displays full size; inactive tabs remain in memory preserving grid state and background updates.

### D-03: Tab Strip UI & Visual Indicators
- Tab strip renders at the top of the terminal content pane:
  - Status dot: Green (connected), Yellow pulsing (connecting/reconnecting), Red/Gray (disconnected).
  - Tab title with close 'x' icon.
  - Plus '+' button for opening a new session/connection selector.
  - Active tab highlighted with theme border/background.

### D-04: Keyboard Shortcuts
- Support standard desktop shortcuts:
  - `Ctrl+T` / `Cmd+T`: Open new tab
  - `Ctrl+W` / `Cmd+W`: Close active tab
  - `Ctrl+Tab`: Cycle to next tab
  - `Ctrl+Shift+Tab`: Cycle to previous tab
  - `Alt+1` through `Alt+9`: Switch directly to tab 1..9

### D-05: Reconnection & Session Re-attachment
- If the WebSocket connection drops unexpectedly (network glitch or backend reload):
  - Mark session status as `Reconnecting`.
  - Surface non-intrusive reconnection banner with retry countdown and manual "Reconnect Now" button.
  - Reconnect using `{"type": "attach", "session_id": "<existing_id>"}`.
  - Backend re-links the session, streams the scrollback buffer via `ready`, and user continues work without lost history or killed shell process.
- Persist active session IDs in `settings.json` so app restart can offer one-click re-attachment to detached backend sessions.

</decisions>

<code_context>
## Existing Code Insights

- Backend routes (`be/internal/api/routes.go`): `/ws` handles WebSocket connections.
- Backend proxy (`be/internal/ssh/proxy.go`): Handles `connect`, `attach`, `ready`, `resize`, `disconnect`, `get-cwd`.
- Backend session manager (`be/internal/ssh/session_manager.go`): Tracks detached sessions in memory.
- Desktop client (`webterm-terminal`): `TerminalView` already has `write_to_pty`, `process_output`, `with_input_callback`, `with_resize_callback`.
- Desktop shell (`desktop-gpui/crates/webterm`): Navigation shell ready to embed tab strip and session views.

</code_context>

<specifics>
## Specific Requirements

- **SHELL-04**: User can open, switch, and close multiple terminal tabs with status indicators.
- **TERM-05**: User can open a local terminal tab (shell on the desktop host), first-class in New Tab.
- **TERM-06**: Terminal sessions reconnect after a WebSocket drop, with reconnection UX.
- **TERM-07**: Terminal sessions survive an app restart via backend session re-attach.
- **TERM-08**: Keyboard shortcuts for new tab, close tab, and tab cycling (desktop conventions).

</specifics>

<deferred>
## Deferred Ideas

- Saved host catalog management with tags/search: Phase 23.
- Windows ConPTY direct native spawning bypass: Phase 24.
- Dual-pane SFTP tab: Phase 25.
- Port forwarding UI tab: Phase 26.

</deferred>
