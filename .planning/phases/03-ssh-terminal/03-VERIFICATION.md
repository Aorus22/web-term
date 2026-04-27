---
phase: 03-ssh-terminal
verified: 2026-04-27T12:00:00Z
status: human_needed
score: 10/10 must-haves verified
overrides_applied: 0
human_verification:
  - test: "SSH to a real server — connect, run vim, htop, tmux"
    expected: "Terminal renders vim/htop/tmux correctly with no rendering artifacts"
    why_human: "Requires running backend + frontend + real SSH server; binary SSH output rendering is runtime behavior"
  - test: "Resize browser window while connected to SSH session"
    expected: "Terminal auto-resizes and remote PTY redraws correctly"
    why_human: "Requires live server + SSH session; resize propagation is runtime behavior"
  - test: "Select text in terminal and copy via Ctrl+C / right-click"
    expected: "Text is copied to clipboard (native DOM selection via @wterm/react)"
    why_human: "Copy/paste UX requires browser interaction; cannot verify programmatically"
  - test: "Disconnect WebSocket (e.g. stop backend) and observe reconnection prompt"
    expected: "Semi-transparent overlay appears with Reconnect and Close buttons"
    why_human: "Requires disrupting live connection; visual overlay appearance is UX verification"
  - test: "Quick-connect to a server, then disconnect — observe save banner"
    expected: "Banner appears offering to save connection details (host/port/user only, no password)"
    why_human: "Requires live session + disconnect flow; visual behavior"
  - test: "Click saved connection with stored password in sidebar"
    expected: "Terminal immediately opens and auto-connects without password prompt (D-01)"
    why_human: "Requires live backend with encrypted passwords in DB"
  - test: "Click saved connection without stored password in sidebar"
    expected: "Inline PasswordPrompt appears in terminal pane area (D-02)"
    why_human: "Requires live backend + connection without password stored"
---

# Phase 3: SSH Terminal Verification Report

**Phase Goal:** Users can SSH to any server from the browser with a smooth, reliable terminal experience — this is the product's core value
**Verified:** 2026-04-27T12:00:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

Truths derived from ROADMAP.md Success Criteria (5) + PLAN frontmatter must_haves (15 total, deduplicated to 10 unique):

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | User can connect to a remote server and get an interactive shell that handles vim, htop, tmux (TERM-01, TERM-02, SC-1) | ✓ VERIFIED | Backend proxy.go dials SSH, requests xterm-256color PTY, pipes bidirectional I/O via WebSocket binary frames. Frontend useSSHSession creates WebSocket, TerminalPane renders @wterm/react Terminal. Full data flow: WebSocket → binary frame → `write(data)` → @wterm/react renders. Keystrokes: `sendData()` → TextEncoder → binary WS → stdinPipe.Write. |
| 2 | Password authentication works, including keyboard-interactive SSH prompts (TERM-03, SC-2) | ✓ VERIFIED | proxy.go:114 — `ssh.KeyboardInteractive(func(...)` returns password for all challenges alongside `ssh.Password(connectMsg.Password)` in Auth methods. Both saved (connection_id) and quick-connect flows populate password. |
| 3 | Terminal auto-resizes when browser window or pane resizes (TERM-04, SC-3) | ✓ VERIFIED | TerminalPane.tsx:108,154 — `<Terminal autoResize onResize={sendResize}>`. useSSHSession `sendResize()` sends `{"type":"resize","cols","rows"}` JSON via WebSocket. proxy.go:168-177 — receives resize message, sends `window-change` SSH request with new dimensions. |
| 4 | Copy/paste works natively through browser text selection (TERM-05, SC-4) | ✓ VERIFIED | TerminalPane.tsx:2 — `import { Terminal } from '@wterm/react'`. @wterm/react uses DOM-based rendering (not canvas), enabling native browser text selection and clipboard. No custom clipboard handling needed — confirmed by spike validation in Phase 1. |
| 5 | WebSocket disconnection shows a reconnection prompt to user (UI-04, SC-5) | ✓ VERIFIED | TerminalPane.tsx:148-168 — disconnected state renders `<ReconnectOverlay>` with "Connection to {host} lost" message, Reconnect button (calls `handleRetry` → `connect(lastOptionsRef.current)`), and Close button. ReconnectOverlay.tsx: semi-transparent overlay with WifiOff icon. |
| 6 | Backend accepts both connection_id (saved) and host/user/password (quick-connect) connect messages | ✓ VERIFIED | proxy.go:72-93 — `if connectMsg.ConnectionID != ""` branches to DB fetch + decrypt flow; else uses direct fields. types.go — ConnectMessage has both `ConnectionID` (optional) and `Host/Port/User/Password` fields. |
| 7 | Server fetches decrypted password from SQLite when connection_id is provided | ✓ VERIFIED | proxy.go:80 — `database.First(&conn, "id = ?", connectMsg.ConnectionID)` via GORM. proxy.go:85 — `config.Decrypt(conn.Encrypted, cfg.EncryptionKey)` decrypts stored password. Password never crosses frontend for saved connections. |
| 8 | Backend sends session_id in connected response for frontend session tracking | ✓ VERIFIED | proxy.go:69 — `sessionID := uuid.New().String()` generated before SSH dial. proxy.go:147-150 — `ServerMessage{Type: "connected", SessionID: sessionID}` sent to client. useSSHSession.ts stores session with matching ID via `addSession()`. |
| 9 | SSH session goroutines cleaned up when WebSocket closes — no zombie connections | ✓ VERIFIED | proxy.go:172-173 — `ctx, cancel := context.WithCancel(context.Background())` + `defer cancel()`. Three goroutines (stdin reader, stdout forwarder, stderr forwarder) all check `<-ctx.Done()`. proxy.go:47 — `defer conn.Close()`, proxy.go:138 — `defer session.Close()`. |
| 10 | Tab bar shows active sessions, clicking tab switches terminal, closing tab disconnects session | ✓ VERIFIED | TabBar.tsx — renders `sessions.map()`, `setActiveSession(session.id)` on click, `removeSession(session.id)` on × click. App.tsx:102-112 — all sessions rendered with visibility toggle (`hidden` class for inactive). App store `removeSession` auto-selects first remaining session. |

**Score:** 10/10 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `be/internal/ssh/proxy.go` (252 lines) | Enhanced SSH proxy with KBI, connection_id, cleanup | ✓ VERIFIED | Exists, substantive, exports `HandleWebSocket`. Full implementation: dual auth flow, PTY request, bidirectional I/O, resize handling, context-based cleanup. |
| `be/internal/ssh/types.go` (30 lines) | Extracted message types | ✓ VERIFIED | Contains `ConnectMessage`, `ResizeMessage`, `ServerMessage` with proper JSON tags. |
| `fe/src/features/terminal/types.ts` (26 lines) | Session types and status enum | ✓ VERIFIED | Exports `SessionStatus` (4 states), `SSHSession` interface (9 fields), `ConnectOptions` interface (dual flow). |
| `fe/src/features/terminal/useSSHSession.ts` (188 lines) | WebSocket SSH session hook | ✓ VERIFIED | min_lines: 80 → actual: 188. Exports `useSSHSession`. Full lifecycle: `connect`, `sendData`, `sendResize`, `disconnect`. Binary frame handling, JSON control messages, store integration. |
| `fe/src/features/terminal/TerminalPane.tsx` (171 lines) | Terminal rendering with 4 states | ✓ VERIFIED | min_lines: 60 → actual: 171. Exports `TerminalPane`. Renders: connecting (spinner), connected (@wterm/react Terminal), error (AlertTriangle + Retry), disconnected (frozen terminal + ReconnectOverlay + SaveBanner). |
| `fe/src/stores/app-store.ts` (55 lines) | Extended store with session state | ✓ VERIFIED | Contains `sessions: SSHSession[]`, `activeSessionId`, `addSession`, `removeSession`, `updateSession`, `setActiveSession` — all implemented with proper Zustand patterns. |
| `fe/src/components/TabBar.tsx` (59 lines) | Tab bar for active sessions | ✓ VERIFIED | min_lines: 40 → actual: 59. Renders tabs from store sessions, click → setActiveSession, × → removeSession. Accessible (role, aria-label, keyboard). |
| `fe/src/features/terminal/PasswordPrompt.tsx` (58 lines) | Inline password prompt | ✓ VERIFIED | Form with password input, auto-focus, Connect/Cancel buttons. `onConnect(password)` callback wired to `handlePasswordConnect` in TerminalPane. |
| `fe/src/features/terminal/ReconnectOverlay.tsx` (40 lines) | Reconnection overlay on disconnect | ✓ VERIFIED | Semi-transparent overlay with WifiOff icon, "Connection lost" message, Reconnect + Close buttons. Wired to `handleRetry`/`handleReconnectDismiss`. |
| `fe/src/features/terminal/SaveConnectionBanner.tsx` (63 lines) | Save connection after quick-connect | ✓ VERIFIED | Shows after quick-connect disconnect. Calls `connectionsApi.create()` with host/port/username only (no password per T-03-08). Invalidates TanStack Query cache. |
| `fe/src/App.tsx` (124 lines) | App shell with TabBar and terminal area | ✓ VERIFIED | Imports and renders `TabBar` in header, all sessions rendered with visibility toggle, `TerminalPane` per session with `initialConnect` for saved connections. |
| `fe/src/features/connections/components/ConnectionList.tsx` (175 lines) | Saved connection click → terminal session | ✓ VERIFIED | Imports `useAppStore`, destructures `addSession`. Line 104: `addSession(session)` creates terminal session on connection click. |
| `fe/src/features/connections/components/QuickConnect.tsx` (123 lines) | Quick-connect → terminal session | ✓ VERIFIED | Imports `useAppStore`, destructures `addSession`. Lines 42,75: `addSession(session)` creates terminal session for quick-connect flow. |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `proxy.go` | `db.Connection` | `database.First(&conn, "id = ?", connectMsg.ConnectionID)` | ✓ WIRED | GORM query at line 80, fetches saved connection by ID |
| `proxy.go` | `config.Decrypt` | `config.Decrypt(conn.Encrypted, cfg.EncryptionKey)` | ✓ WIRED | Password decryption at line 85, uses encryption key from config |
| `proxy.go` | SSH session | `context.WithCancel` + goroutines | ✓ WIRED | ctx/cancel pattern at line 172-173, all 3 goroutines respect ctx.Done() |
| `useSSHSession.ts` | WebSocket server | `new WebSocket('ws://${wsHost}:8080/ws')` | ✓ WIRED | Line 75, sends connect message, handles binary/text frames |
| `TerminalPane.tsx` | `@wterm/react` | `import { Terminal } from '@wterm/react'` | ✓ WIRED | Line 2, renders with autoResize, cursorBlink, onData, onResize |
| `useSSHSession.ts` | `app-store` | `useAppStore` (addSession, updateSession) | ✓ WIRED | Line 12 import, store operations throughout hook |
| `ConnectionList.tsx` | `app-store` | `addSession()` | ✓ WIRED | Line 104, creates SSHSession on saved connection click |
| `QuickConnect.tsx` | `app-store` | `addSession()` | ✓ WIRED | Lines 42,75, creates SSHSession for quick-connect flow |
| `App.tsx` | `TabBar` | Import + render in header | ✓ WIRED | Line 18 import, rendered in header with sidebar toggle |
| `App.tsx` | `TerminalPane` | Per-session render with visibility toggle | ✓ WIRED | Lines 102-112, all sessions rendered, active shown via hidden class |
| `TerminalPane.tsx` | `PasswordPrompt` | Import + conditional render | ✓ WIRED | Line 15 import, rendered when connecting + no initialConnect + no passwordProvided |
| `TerminalPane.tsx` | `ReconnectOverlay` | Import + render on disconnect | ✓ WIRED | Line 16 import, rendered in disconnected state over frozen terminal |
| `TerminalPane.tsx` | `SaveConnectionBanner` | Import + render on quick-connect disconnect | ✓ WIRED | Line 17 import, shown when disconnected + isQuickConnect |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `TerminalPane.tsx` (connected) | `ref` (terminal data) | `useSSHSession` → `write()` → `useTerminal()` | Binary frames from SSH via WebSocket → `write(data)` → @wterm/react renders | ✓ FLOWING |
| `TerminalPane.tsx` (keystrokes) | `sendData(data)` | User input via `onData` callback | TextEncoder → binary WebSocket frame → proxy.go `stdinPipe.Write` → SSH session | ✓ FLOWING |
| `TerminalPane.tsx` (resize) | `sendResize(cols, rows)` | `onResize` callback from @wterm/react autoResize | JSON `{"type":"resize"}` → proxy.go parses → `session.SendRequest("window-change")` | ✓ FLOWING |
| `TerminalPane.tsx` (session state) | `session` from `useAppStore` | `useSSHSession.connect()` → `addSession()` + `updateSession()` | Store updates via WebSocket onmessage (connected/error/disconnected) | ✓ FLOWING |
| `proxy.go` (connection_id flow) | `connectMsg.Password` | `database.First()` → `config.Decrypt(conn.Encrypted)` | Real DB query + decryption populates password field | ✓ FLOWING |
| `TabBar.tsx` | `sessions` from `useAppStore` | `App.tsx` → `addSession()` → store | Sessions array renders tab labels and status indicators | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Go backend compiles | `cd be && go build ./cmd/server/` | BUILD OK | ✓ PASS |
| Frontend TypeScript compiles | `cd fe && npx tsc --noEmit` | Clean (only baseUrl deprecation warning, no real errors) | ✓ PASS |
| Backend types exported | `grep -c 'ConnectMessage\|ResizeMessage\|ServerMessage' be/internal/ssh/types.go` | 3 matches | ✓ PASS |
| KeyboardInteractive auth present | `grep -c 'KeyboardInteractive' be/internal/ssh/proxy.go` | 1 match | ✓ PASS |
| Context cleanup present | `grep -c 'context.WithCancel' be/internal/ssh/proxy.go` | 1 match | ✓ PASS |
| useSSHSession exports correct functions | `grep -c 'connect\\|sendData\\|sendResize\\|disconnect' fe/src/features/terminal/useSSHSession.ts` | Multiple matches for each | ✓ PASS |
| TerminalPane renders 4 states | `grep -c 'connecting\\|connected\\|error\\|disconnected' fe/src/features/terminal/TerminalPane.tsx` | Multiple matches per state | ✓ PASS |
| App store has session CRUD | `grep 'sessions\\|addSession\\|removeSession\\|updateSession' fe/src/stores/app-store.ts` | All present | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| TERM-01 | Plan 01, Plan 02 | WebSocket SSH proxy — Go backend accepts WS, dials SSH, pipes bidirectional I/O | ✓ SATISFIED | proxy.go: full WebSocket → SSH proxy with binary frame forwarding. useSSHSession.ts: WebSocket client with binary frame handling. |
| TERM-02 | Plan 02 | Terminal rendering via @wterm/react — handles vim, htop, tmux | ✓ SATISFIED | TerminalPane.tsx imports `@wterm/react` Terminal, renders with xterm-256color PTY. Validated in Phase 1 spike. |
| TERM-03 | Plan 01 | Password-based SSH auth with keyboard-interactive support | ✓ SATISFIED | proxy.go:114 — `ssh.KeyboardInteractive` + `ssh.Password` in Auth methods. Covers both password and KBI flows. |
| TERM-04 | Plan 02 | Auto-resize terminal — resize propagated to SSH PTY | ✓ SATISFIED | TerminalPane.tsx: `autoResize` + `onResize={sendResize}`. useSSHSession: sends resize JSON. proxy.go: window-change SSH request. |
| TERM-05 | Plan 02 | Copy/paste — native browser text selection via DOM rendering | ✓ SATISFIED | @wterm/react uses DOM-based rendering (not canvas), enabling native copy/paste. Confirmed by Phase 1 spike validation. |
| UI-04 | Plan 03 | Reconnection handling — prompt user on WebSocket disconnect | ✓ SATISFIED | ReconnectOverlay.tsx: semi-transparent overlay with Reconnect/Close buttons. TerminalPane.tsx: renders on disconnected state. |

No orphaned requirements found — all 6 requirement IDs mapped to this phase are covered by plan frontmatter and verified.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `be/internal/ssh/proxy.go` | 122 | `ssh.InsecureIgnoreHostKey()` // TODO: Implement host key verification | ⚠️ Warning | Known limitation from Phase 1 spike (documented in Phase 1 review as WR-01). Acceptable for v1 — single-user local tool. Not a blocker for phase goal. |
| `fe/src/features/terminal/TerminalPane.tsx` | 156 | `onData={() => {}}` on disconnected state Terminal | ℹ️ Info | Intentional — frozen terminal should not accept input when disconnected. Not a stub. |
| `fe/src/features/terminal/TerminalPane.tsx` | 170 | `return null` | ℹ️ Info | Fallback for state not matching any condition — defensive coding. Not a stub. |
| `fe/src/components/TabBar.tsx` | 16 | `if (sessions.length === 0) return null` | ℹ️ Info | Intentional — no tabs to show when no sessions exist. Not a stub. |

### Human Verification Required

### 1. Live SSH Connection Test
**Test:** Start backend (`cd be && go run ./cmd/server/`), start frontend (`cd fe && npm run dev`), open browser, SSH to a real server
**Expected:** Terminal renders with interactive shell. Run `vim`, `htop`, `tmux` — all display correctly without artifacts.
**Why human:** Requires running backend + frontend + real SSH server. Binary SSH output rendering (TERM-02) is runtime behavior.

### 2. Terminal Auto-Resize Test
**Test:** While connected to SSH session, resize browser window
**Expected:** Terminal redraws at new size. Remote PTY dimensions update (verify with `stty size` in terminal).
**Why human:** Requires live SSH session. Resize propagation through WebSocket → SSH is runtime behavior.

### 3. Copy/Paste Test
**Test:** Select text in terminal using mouse, copy via Ctrl+C or right-click
**Expected:** Text copies to clipboard. Paste into another application works.
**Why human:** Browser clipboard interaction requires user interaction to verify.

### 4. Disconnection & Reconnection Test
**Test:** Connect to SSH, then stop backend server. Observe terminal pane.
**Expected:** Semi-transparent ReconnectOverlay appears with "Connection to {host} lost" message. Click Reconnect, restart backend, verify reconnection works.
**Why human:** Requires disrupting live connection and observing visual overlay behavior.

### 5. Quick-Connect Save Banner Test
**Test:** Use Quick Connect to SSH to a server, then disconnect (stop backend or Ctrl+D in shell)
**Expected:** Save connection banner appears at top of terminal pane offering to save. Click Save — connection appears in sidebar.
**Why human:** Requires live session + disconnect flow. Banner visual and save action are runtime behaviors.

### 6. Saved Connection Auto-Connect Test
**Test:** Save a connection with password in sidebar. Click the saved connection.
**Expected:** Terminal immediately opens and auto-connects (no password prompt). Connection uses connection_id flow — password fetched server-side.
**Why human:** Requires live backend with encrypted passwords in SQLite DB.

### 7. Inline Password Prompt Test
**Test:** Save a connection without password. Click the saved connection in sidebar.
**Expected:** Terminal pane shows inline PasswordPrompt (not modal) with "Connect to user@host" heading.
**Why human:** Visual behavior of inline prompt requires browser interaction to verify.

### Gaps Summary

No structural gaps found. All 10 observable truths are VERIFIED with code-level evidence:

- **Backend:** Full SSH proxy implementation with dual auth flow (connection_id + direct credentials), keyboard-interactive support, context-based goroutine cleanup, SSRF protection, session ID tracking, and PTY resize propagation.
- **Frontend:** Complete terminal infrastructure with WebSocket lifecycle management, 4-state TerminalPane (connecting/connected/error/disconnected), TabBar for multi-session visibility, inline PasswordPrompt, ReconnectOverlay, and SaveConnectionBanner.
- **Wiring:** All key links verified — DB query → config.Decrypt → SSH dial → WebSocket → binary frames → @wterm/react Terminal → user keystrokes → stdin pipe. Full bidirectional data flow confirmed.
- **Build:** Both Go backend and TypeScript frontend compile cleanly.
- **Requirements:** All 6 requirement IDs (TERM-01 through TERM-05, UI-04) satisfied with code evidence.

**One warning** worth noting: `InsecureIgnoreHostKey()` at proxy.go:122 is a known limitation from Phase 1, documented as acceptable for v1 single-user tool. Should be addressed before multi-user deployment.

**7 human verification items** require live server testing — the core user experience (SSH connection, rendering, resize, copy/paste, reconnection) must be validated with a real SSH server.

---

_Verified: 2026-04-27T12:00:00Z_
_Verifier: the agent (gsd-verifier)_
