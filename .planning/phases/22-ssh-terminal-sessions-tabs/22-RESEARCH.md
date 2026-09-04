# Phase 22: SSH Terminal Sessions & Tabs - Research

**Date:** 2026-09-04
**Domain:** WebSocket terminal transport, multi-tab lifecycle, session re-attachment, and reconnection resilience

## 1. WebSocket Protocol & Transport (`tokio-tungstenite`)

The Go backend exposes `GET /ws` upgraded via gorilla/websocket. Over loopback TCP (`ws://127.0.0.1:{port}/ws`), no TLS overhead is required.

### Dependency Configuration
```toml
tokio-tungstenite = { version = "=0.26.2", default-features = false, features = ["connect", "handshake"] }
```
- Avoids OpenSSL / native-tls dependencies completely.
- Operates directly inside the existing Tokio multi-thread runtime.

### Frame Sequencing & Handshake Contract
1. **Handshake Phase**:
   - Client sends Text Frame: `{"type": "connect", "connection_id": "<id>", "cols": 80, "rows": 24, "term": "xterm-256color"}`
   - Server responds with Text Frame: `{"type": "connected", "session_id": "<uuid>"}` (or `{"type": "error", "message": "<reason>"}`)
2. **Scrollback Replay**:
   - Client sends Text Frame: `{"type": "ready"}`
   - Server flushes stored buffer as Binary Frame(s) containing prior terminal output.
3. **Interactive Streaming**:
   - Server -> Client: Binary Frames containing raw PTY output bytes.
   - Client -> Server: Binary Frames containing raw keyboard / mouse escape bytes.
4. **Control Events**:
   - Client -> Server Text Frame: `{"type": "resize", "cols": 120, "rows": 40}`
   - Client -> Server Text Frame: `{"type": "disconnect"}`

## 2. Multi-Tab Architecture in GPUI

### Data Model
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionStatus {
    Connecting,
    Connected,
    Reconnecting { attempt: usize, error: Option<String> },
    Disconnected { reason: String },
}

pub struct TerminalTab {
    pub id: String,
    pub session_id: Option<String>,
    pub title: String,
    pub status: SessionStatus,
    pub terminal_view: Entity<TerminalView>,
    pub ws_handle: Option<TerminalWsHandle>,
}
```

### UI Tab Strip Design
- Horizontal bar rendered above the terminal view.
- Each tab item displays:
  - Status dot: Green for `Connected`, pulsing yellow for `Connecting`/`Reconnecting`, red for `Disconnected`.
  - Title string (e.g. `user@host:22` or `Local Terminal`).
  - Close button (`x`) with hover state.
- Right side of strip features a `+` button to open a new tab modal or prompt.
- Active tab highlighted with accent background and bottom indicator border.

## 3. Keyboard Shortcuts

Using GPUI's key listener:
- `Ctrl+T` / `Cmd+T`: New tab
- `Ctrl+W` / `Cmd+W`: Close active tab
- `Ctrl+Tab`: Next tab (wraps around)
- `Ctrl+Shift+Tab`: Previous tab (wraps around)
- `Alt+1` .. `Alt+9`: Switch directly to tab 1 through 9

## 4. Reconnection & Re-attachment Architecture

### Handling Unexpected Disconnects
- When `ws_read.next().await` yields `None` or `Err`:
  - Set tab status to `Reconnecting`.
  - Surface a subtle top banner in the terminal view: `"Connection lost. Reconnecting in 3s... [Reconnect Now]"`.
  - Use exponential backoff (1s, 2s, 4s, 8s, up to 15s).

### Session Re-attachment
- Backend `ManagedSession` persists in memory even when its WebSocket is dropped:
  - Re-connect with `{"type": "attach", "session_id": "<session_id>"}`.
  - Server verifies session exists, re-binds the socket, sends `{"type": "connected"}`, and on `ready` replays the buffer.
  - User's running applications (vim, htop, long builds) continue without interruption!

### App Restart Survival (TERM-07)
- Backend runs independently or retains sessions.
- Client persists open session IDs into desktop settings store (`settings.json`).
- Upon desktop app launch, query `GET /api/sessions` from backend.
- If previously saved sessions are active, client presents an option to re-attach or automatically restores tabs.
