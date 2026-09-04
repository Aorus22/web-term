# Plan Summary: 22-03 - Reconnection UX & Session Survival Across Restarts

## Execution Summary
- **Phase:** 22-ssh-terminal-sessions-tabs
- **Plan:** 03
- **Status:** Complete
- **Requirements Addressed:** `TERM-06`, `TERM-07`

## Accomplishments
1. **Reconnection Alert Banner UI (`reconnect_banner.rs`):**
   - Implemented `render_reconnect_banner` providing contextual recovery UI above the terminal grid for active tabs in non-connected states.
   - Amber alert banner for `SessionStatus::Reconnecting`: indicates countdown timer, attempt count (`1..5`), "Reconnect Now" button for instant retry, and "Close Tab" button.
   - Red alert banner for `SessionStatus::Disconnected`: displays disconnection reason with "Reconnect" and "Close Tab" actions.
   - Theme-adaptive color schemes for both dark and light themes.

2. **Drop Detection & Automated Exponential Backoff (`session.rs`, `app_state.rs`):**
   - `TerminalSessionManager::attach_handle` detects unexpected WebSocket closure when the inbound PTY reader pump terminates without manual user disconnection.
   - Signals `AppState::on_tab_dropped`, which initiates an automated reconnection loop with exponential backoff (`2s`, `4s`, `6s`, `8s`, up to `16s`).
   - Re-attaches to existing sessions on the backend via `client.attach_terminal(&session_id)`, which re-links the PTY and triggers scrollback replay via the backend's `"ready"` protocol.
   - Limits retry attempts to 5 before transitioning to `Disconnected(Some(reason))`.

3. **Session Survival Across Application Restarts (`settings/lib.rs`, `app_state.rs`):**
   - Added `SavedSessionTab` struct and `open_sessions: Vec<SavedSessionTab>` to `DesktopSettings`.
   - Open tabs are automatically persisted to `settings.json` upon tab open, connect, and close.
   - On supervisor `Ready`, `restore_sessions_or_default` queries `client.list_sessions().await`:
     - Cross-references detached backend sessions with saved tabs, or discovers any active/detached backend sessions.
     - Restores tabs in `Connecting` status and re-attaches to existing backend sessions without killing remote shells or losing history.
     - Falls back to opening a fresh local shell tab if no backend sessions exist.

4. **Testing & Workspace Verification:**
   - Created `desktop-gpui/crates/webterm/tests/reconnect_test.rs`:
     - Verified backoff schedule progression across retry attempts.
     - Verified `SavedSessionTab` serialization and deserialization roundtrip.
     - Verified detached session matching and restoration selection logic.
   - Full workspace test suite passing: 52/52 tests pass.
   - Clean `cargo clippy --workspace --manifest-path desktop-gpui/Cargo.toml -- -D warnings` with 0 warnings.

## Files Created / Modified
- `desktop-gpui/crates/settings/src/lib.rs`
- `desktop-gpui/crates/webterm/src/session.rs`
- `desktop-gpui/crates/webterm/src/app_state.rs`
- `desktop-gpui/crates/webterm/src/views/reconnect_banner.rs`
- `desktop-gpui/crates/webterm/src/views/mod.rs`
- `desktop-gpui/crates/webterm/src/views/nav.rs`
- `desktop-gpui/crates/webterm/tests/reconnect_test.rs`

## Next Steps
Proceed to Plan 22-04: Local terminal tab baseline verification, New Tab launcher modal / menu, and Phase 22 verification & wrap-up (`TERM-05`).
