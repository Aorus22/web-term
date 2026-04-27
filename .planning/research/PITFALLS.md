# Pitfalls Research

**Domain:** Web-based SSH terminal client (Go backend + Vite/React + @wterm/react + WebSocket proxy + SQLite)
**Researched:** 2026-04-27
**Confidence:** HIGH

## Critical Pitfalls

### Pitfall 1: SSH Session Zombie Leaks

**What goes wrong:**
When a browser tab closes, the network drops, or the WebSocket disconnects unexpectedly, the Go backend SSH session stays alive on the remote server. Over time, dozens of zombie `bash` processes accumulate on every server the user connects to. The remote server's process table fills up.

**Why it happens:**
WebSocket disconnects don't automatically trigger SSH session cleanup. The Go goroutine piping SSH ↔ WebSocket needs to detect the WebSocket close and call `session.Close()` + `client.Close()`. If the goroutine is blocked on SSH reads, it never detects the WebSocket close. This is a bidirectional problem — either side blocking prevents cleanup.

**How to avoid:**
- Use `context.Context` as the cancellation signal. Create a context that cancels on WebSocket close AND SSH close.
- Run SSH read and WebSocket write in separate goroutines, both selecting on `ctx.Done()`.
- Use `session.Signal(ssh.SIGKILL)` followed by `session.Close()` and `client.Close()` in a deferred cleanup function.
- For gorilla/websocket: The `readPump` goroutine detects close; signal the `writePump` via a channel. For coder/websocket: context cancellation handles this naturally.
- Add a global session registry with TTL-based cleanup that kills sessions inactive for > N minutes.

**Warning signs:**
- `ps aux | grep bash` on target servers shows stale sessions from the web terminal's source IP
- Go backend goroutine count (`runtime.NumGoroutine()`) steadily increases
- Memory usage climbs over time even with no active connections

**Phase to address:**
Phase 1 (WebSocket ↔ SSH proxy) — this is foundational. If you don't get cleanup right in the proxy layer, every subsequent feature (multi-tab, reconnection) builds on a leaky foundation.

---

### Pitfall 2: Terminal Resize Not Propagating End-to-End

**What goes wrong:**
The terminal renders at the wrong dimensions. `vim`, `htop`, `less`, and any curses-based application displays garbled output or wraps incorrectly. The user resizes the browser window and nothing happens — or worse, the remote PTY and the local terminal get out of sync.

**Why it happens:**
Resize requires a 3-hop propagation: browser terminal → WebSocket message → Go backend → SSH `WindowChange()`. Most implementations handle the first hop (FitAddon/ResizeObserver in the browser) but miss one of the subsequent hops. Common failure points:
- ResizeObserver fires but the WebSocket message is never sent or is malformed
- Go backend receives the resize message but calls `WindowChange()` with wrong column/row values (mixing up w/h or using pixel dimensions instead of character cells)
- ResizeObserver is not debounced, flooding the WebSocket with resize messages that overwhelm the SSH channel

**How to avoid:**
- Establish a typed JSON protocol for resize: `{"type":"resize","cols":80,"rows":24}` — always in character cells, never pixels.
- Debounce resize on the frontend (100ms is standard for xterm.js/wterm).
- In Go, call `session.WindowChange(rows, cols)` immediately on receiving the resize message — note the parameter order is `(h, w)` in the Go SSH library, not `(w, h)`.
- wterm has `ResizeObserver`-based auto-resize built into `@wterm/dom`. Verify it emits resize events you can hook into.
- Send initial dimensions as part of the connection handshake, not as a separate message after connect.

**Warning signs:**
- `stty size` in the terminal shows wrong values after resizing the browser
- `vim` displays with corrupted layout or wrong column count
- `top`/`htop` doesn't fill the terminal width

**Phase to address:**
Phase 1 (WebSocket ↔ SSH proxy) — resize must work from day one. It's part of the PTY setup (RequestPty needs initial dimensions) and must be maintained throughout the session.

---

### Pitfall 3: WebSocket Concurrent Write Panic (gorilla/websocket)

**What goes wrong:**
The Go backend panics with `concurrent write to websocket connection`. This is a fatal error that crashes the goroutine handling the SSH session.

**Why it happens:**
gorilla/websocket connections support exactly one concurrent reader and one concurrent writer. In a typical SSH proxy, you need at least two goroutines writing: one piping SSH stdout → WebSocket, and one sending ping frames for keepalive. If both try to write at the same time → panic.

**How to avoid:**
- **Option A (recommended): Use `coder/websocket` instead of gorilla/websocket.** The `github.com/coder/websocket` package explicitly supports concurrent writes, has first-class `context.Context` support, and is maintained by Coder (who build web-based terminals professionally). Zero dependencies.
- **Option B: If using gorilla/websocket**, implement a write pump pattern — all writes go through a buffered channel to a single writer goroutine. This is the pattern shown in gorilla/websocket's chat example. Ping messages also go through this pump.
- Never call `conn.WriteMessage()` from multiple goroutines with gorilla/websocket.

**Warning signs:**
- Intermittent panics in production that you can't reproduce locally
- Panics only happen under load (multiple tabs, long-running commands producing output)
- Stack trace shows `gorilla/websocket.(*Conn).WriteMessage` in multiple goroutines

**Phase to address:**
Phase 1 (WebSocket ↔ SSH proxy) — this is a library choice that affects the entire architecture. Decide on day one.

---

### Pitfall 4: Passwords Stored in Plaintext in SQLite

**What goes wrong:**
SSH passwords are stored unencrypted in the SQLite database. Anyone with file access to the database (or the server's filesystem) can read all saved SSH credentials. Since v1 has no auth, there's not even a login gate — if the app is exposed, the passwords are exposed.

**Why it happens:**
It's tempting to just store the password in a TEXT column for v1 and "add encryption later." But the passwords are the keys to production servers. Even for a self-hosted individual tool, a compromised laptop or backup means all server credentials are leaked.

**How to avoid:**
- **v1 minimum:** Encrypt passwords at rest using AES-GCM with a key derived from a user-provided master password or a server-side secret stored in an environment variable.
- Use Go's `crypto/aes` + `crypto/cipher` (standard library) — no external dependencies needed.
- Store only the ciphertext in SQLite, never the plaintext.
- Mark the password column in your schema as `BLOB` (for ciphertext), not `TEXT`.
- **Better for v1:** Don't store passwords at all. Prompt for the password each time the user connects (browser can auto-fill). Store only host/port/username.
- **For v2:** Implement SSH key management instead of passwords.

**Warning signs:**
- SQLite file contains readable password strings when viewed with `sqlite3` CLI
- No encryption/decryption functions in the Go codebase
- Database schema has `password TEXT` column

**Phase to address:**
Phase 2 (Connection management + SQLite) — the storage layer must be designed with encryption from the start. Retrofitting encryption into an existing schema and data migration is error-prone.

---

### Pitfall 5: wterm Library Maturity Risks

**What goes wrong:**
The @wterm/react library (v0.2.0, published days ago) has undiscovered bugs, missing features, or API changes that block development. You hit a bug in the escape sequence parser, find that `vim` doesn't work correctly, or discover the WebSocket transport layer doesn't support your use case. Since it's a new library, there are no Stack Overflow answers, no community patterns, and documentation is minimal.

**Why it happens:**
wterm is a Zig+WASM terminal emulator under active development by a small team (ctate). At v0.2.0, the API surface is still evolving. The official SSH example uses Node.js + ssh2 (not Go), meaning no reference implementation exists for the Go backend architecture. The PROJECT.md states this is a "first-time exploration" — the team has no prior experience with this library.

**How to avoid:**
- **Phase 0: Spike wterm BEFORE committing to the architecture.** Create a minimal prototype: `@wterm/react` component → WebSocket → Go backend → SSH to localhost. Verify that:
  - Basic shell interaction works (typing, output, escape sequences)
  - `vim`, `htop`, `less` render correctly (alternate screen buffer)
  - Resize works end-to-end
  - The library's WebSocket transport API is compatible with your Go backend protocol
- **Have a fallback plan:** If wterm doesn't work out, the architecture should allow swapping to xterm.js without rewriting the backend. Design the WebSocket protocol to be terminal-library-agnostic.
- **Monitor the wterm GitHub repo** for issues, breaking changes, and the rate of fixes.
- **Evaluate if `@wterm/core`'s built-in WebSocket transport** works with your Go backend or if you need a custom transport layer.

**Warning signs:**
- Escape sequence rendering issues (wrong colors, broken cursor positioning)
- Alternate screen buffer apps (`vim`, `less`) don't switch back cleanly
- Library API changes between minor versions
- Open GitHub issues with no response

**Phase to address:**
Phase 0 (Spike/exploration) — this must be the very first thing you do. If wterm can't handle basic SSH terminal use cases, you need to know before building anything on top of it.

---

### Pitfall 6: No Authentication = Open Proxy

**What goes wrong:**
Without authentication, anyone who can reach the web terminal's URL can: (a) see all saved SSH connections, (b) connect to any saved server, (c) use the web terminal as an SSH bounce point to attack other servers. If the app is exposed on the internet (even accidentally), it becomes an open relay.

**Why it happens:**
The PROJECT.md explicitly scopes out auth for v1 to focus on core terminal features. This is a reasonable trade-off IF the app is truly local-only. But developers often deploy "temporary" tools to a server for convenience, forgetting there's no auth.

**How to avoid:**
- **v1 hard requirement:** Bind the Go HTTP server to `127.0.0.1` only. Never bind to `0.0.0.0`.
- Add a startup check: if `HOST` env var is `0.0.0.0` and no auth is configured, print a loud warning to stderr.
- Document clearly in the README that v1 has NO authentication and must not be exposed to the internet.
- If you need remote access, use an SSH tunnel (`ssh -L 8080:localhost:8080`) or a VPN.
- **For v2 planning:** Design the auth system early enough that the connection storage schema doesn't need to change when multi-user support is added.

**Warning signs:**
- Go server binds to `0.0.0.0` by default
- No mention of network binding in configuration
- App deployed behind a reverse proxy without IP restrictions

**Phase to address:**
Phase 1 (Backend setup) — the HTTP server binding is a one-line configuration that should be set correctly from the start.

---

### Pitfall 7: SSH HostKeyCallback Mishandling

**What goes wrong:**
Using `ssh.InsecureIgnoreHostKey()` in production code silently accepts any host key, enabling man-in-the-middle attacks. Or, the host key verification is too strict — first connection always fails because the host key isn't in known_hosts, and there's no UI to accept it.

**Why it happens:**
The Go SSH library's `HostKeyCallback` has no middle ground — you either verify against a fixed key, check against a known_hosts file, or skip verification entirely. For a web-based terminal, the user needs to see "The authenticity of host X can't be established" and choose to trust it (like OpenSSH does on first connect). Most implementations skip this UX entirely.

**How to avoid:**
- **v1 minimum:** Implement host key verification with a local known_hosts-style storage in SQLite.
- On first connection to a new host, return the host key fingerprint to the frontend and display it to the user for confirmation (like SSH's "Are you sure you want to continue connecting?").
- Store the accepted host key in SQLite. On subsequent connections, verify against the stored key.
- If the host key changes, warn the user (this is a potential MITM indicator).
- **Never ship with `InsecureIgnoreHostKey()` as default** — even for development, use a known_hosts file.

**Warning signs:**
- `ssh.InsecureIgnoreHostKey()` appears in the codebase outside of test files
- No host key storage table in the SQLite schema
- No UI for host key verification/acceptance

**Phase to address:**
Phase 1 (WebSocket ↔ SSH proxy) — host key handling is part of the SSH connection establishment. Phase 2 (Connection storage) for persisting accepted keys.

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Store passwords in plaintext | Faster to implement, simpler CRUD | Credentials leaked if DB file is accessed | Never — use env-var-derived encryption at minimum |
| Skip resize debouncing | Simpler frontend code | Floods WebSocket + SSH channel with resize msgs, causes rendering glitches | Never — 100ms debounce is trivial to implement |
| Use gorilla/websocket without write pump | Simpler code initially | Random panics under concurrent writes | Never — use coder/websocket or implement write pump |
| Single goroutine for SSH↔WS (no context) | Fewer goroutines, simpler flow | SSH sessions leak on disconnect, no graceful shutdown | Never — context-based cancellation is essential |
| Skip alternate screen buffer handling | Works for basic shell | vim/htop/less break completely | Never — verify with @wterm/react early |
| No session timeout/cleanup | Simpler backend | Zombie SSH sessions accumulate on target servers | Acceptable for MVP only — add before public use |
| Binary WebSocket messages only (no protocol) | Simpler parsing | Can't add features (resize, heartbeat, reconnect) without breaking | Acceptable for spike — add typed protocol before multi-tab |
| No reconnection logic | Simpler WebSocket code | Every network blip kills the SSH session | Acceptable for v1 — add in v2 |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| **golang.org/x/crypto/ssh** | Setting `session.Stdout = os.Stdout` instead of using `StdinPipe()`/`StdoutPipe()` for bidirectional streaming | Use `session.StdinPipe()` for writes, pipe `session.Stdout` to an `io.Writer` that sends to WebSocket. Or use `session.Shell()` + `session.StdinPipe()` + direct Session read/write |
| **golang.org/x/crypto/ssh TerminalModes** | Forgetting to set `ssh.ECHO: 0` — causes doubled characters in the terminal | Set `ssh.ECHO: 0` (disable echo) since the terminal emulator handles local echo. Set `ssh.TTY_OP_ISPEED` and `ssh.TTY_OP_OSPEED` to 14400 |
| **golang.org/x/crypto/ssh WindowChange** | Calling with `(width, height)` instead of `(height, width)` — the parameter order is `(h, w)` | `session.WindowChange(rows, cols)` — rows first, cols second |
| **@wterm/core WebSocket transport** | Assuming the built-in WebSocket transport uses the same protocol as your Go backend | Verify the transport protocol (message format, resize messages) during spike. wterm's WebSocket transport may have its own protocol expectations |
| **SQLite from Go** | Using `database/sql` without `_ "modernc.org/sqlite"` driver import — compile error | Import `modernc.org/sqlite` (pure Go, no CGO needed) or `github.com/mattn/go-sqlite3` (CGO, more mature) |
| **gorilla/websocket** | Calling `conn.WriteMessage()` from multiple goroutines — panics with "concurrent write" | Use single write pump goroutine OR switch to `coder/websocket` which supports concurrent writes |
| **SSH session.StdinPipe()** | Calling `Write()` after `session.Close()` — pipe is broken, write returns error | Check error on every write. Close the write side with `stdinPipe.Close()` when WebSocket disconnects to signal EOF to the remote shell |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Unbuffered SSH→WebSocket pipe | Each byte from SSH triggers a separate WebSocket message (frame overhead: 2-14 bytes per frame for 1 byte of data) | Buffer SSH reads — read in chunks (e.g., `buf := make([]byte, 4096)`) and send accumulated data at ~16ms intervals or when buffer is full | Immediate — `cat large_file` creates thousands of tiny WebSocket frames |
| No WebSocket ping/pong keepalive | Connection silently dies after 30-60s of idle shell (reverse proxies/nginx kill idle WebSocket connections) | Send ping frames every 15-30s. Handle pong responses. coder/websocket handles this with `CloseRead()` | First time the shell is left idle behind a reverse proxy |
| Large scrollback buffer | Browser memory grows unbounded with long-running sessions (logs, build output) | Configure reasonable scrollback limit (1000-5000 lines). wterm has configurable ring buffer | Sessions running >1 hour with verbose output |
| SQLite WAL mode not enabled | Concurrent reads (connection list) and writes (host key storage) cause lock contention | Enable WAL mode: `PRAGMA journal_mode=WAL;` on database open | Unlikely at single-user scale, but good practice |
| Sync WebSocket writes in HTTP handler | Go HTTP handler goroutine blocks on WebSocket write, starving other requests | WebSocket handlers are long-lived by design — ensure they don't block the HTTP server's goroutine pool. The goroutine per connection model is expected. | Many concurrent connections (not a v1 concern) |

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| **Binding to 0.0.0.0 without auth** | Anyone on the network can use your SSH connections and see saved credentials | Bind to 127.0.0.1 only. Add startup warning if exposed |
| **Plaintext passwords in SQLite** | Database file theft = all server credentials compromised | Encrypt with AES-GCM using server secret. Or don't store passwords at all |
| **InsecureIgnoreHostKey()** | MITM attacks on SSH connections — attacker intercepts and proxies the connection | Implement known_hosts-style storage with user confirmation UI |
| **No CSRF/CORS on WebSocket upgrade** | Malicious websites can open WebSocket connections to your terminal | Check `Origin` header on WebSocket upgrade (coder/websocket does this by default via `AcceptOptions`) |
| **SSH credentials in URL/query params** | Credentials logged in browser history, server access logs, proxy logs | Send credentials only in the first WebSocket message after connection, never in the HTTP request |
| **No rate limiting on SSH connections** | Brute-force attacks against target servers through your proxy | Add connection rate limiting per target host. Even simple: max 3 connection attempts per host per minute |
| **No input sanitization before SSH** | Special characters in host/username could exploit the Go SSH library or downstream systems | Validate host (valid hostname/IP), port (1-65535), username (alphanumeric + limited chars). Reject before attempting connection |

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| **No connection status indicator** | User doesn't know if terminal is connecting, connected, or disconnected — appears frozen | Show clear status: "Connecting...", "Connected to user@host", "Disconnected — click to reconnect" |
| **Copy/paste doesn't work** | User can't copy commands or output — core terminal workflow broken | wterm's DOM rendering gives native copy/paste for free. Verify it works with your integration |
| **Tab close kills session without warning** | User accidentally closes tab, losing active SSH session and any running processes | Add `beforeunload` event handler: "You have an active SSH session. Close anyway?" |
| **No connection error feedback** | SSH connection fails silently — user sees blank terminal with no explanation | Display error messages: "Connection refused", "Authentication failed", "Host unreachable" — don't just show empty terminal |
| **Terminal font too small/large with no control** | Terminal is unreadable or wastes screen space | Provide font size control. Default to 14px monospace. Persist preference in localStorage |
| **Password shown in clear during input** | Shoulder-surfing risk when typing SSH passwords | Use password input type in connection form. Never display saved passwords — show dots |

## "Looks Done But Isn't" Checklist

- [ ] **Terminal rendering:** Often missing alternate screen buffer support — verify `vim`, `htop`, `less`, `top` render and exit cleanly
- [ ] **Terminal resize:** Often missing end-to-end propagation — verify `stty size` matches browser dimensions after resize
- [ ] **Connection cleanup:** Often missing SSH session cleanup on tab close — verify `ps aux` on target after closing tab shows no orphaned processes
- [ ] **Special keys:** Often missing Ctrl+C, Ctrl+D, Ctrl+Z, Ctrl+L, Tab, arrow keys — verify each sends correct escape sequence through the pipeline
- [ ] **Long-running commands:** Often broken — run `sleep 100`, close tab, verify the Go backend cleans up and the sleep process is terminated
- [ ] **Network interruption:** Often broken — disconnect wifi during active session, verify: (a) frontend shows disconnected state, (b) SSH session is cleaned up, (c) reconnecting creates fresh session
- [ ] **Multiple tabs to same host:** Often causes confusion — verify each tab gets its own SSH session and cleanup is independent
- [ ] **Large output:** Often causes lag — run `cat /dev/urandom | base64 | head -10000`, verify terminal doesn't freeze or drop data
- [ ] **Binary/special characters:** Often corrupted — run `cat /dev/urandom` (Ctrl+C quickly), verify terminal doesn't break

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| SSH session leaks | LOW | Add context-based cleanup. Change is isolated to the WebSocket handler. |
| Resize not propagating | LOW | Add resize message type to WebSocket protocol. Changes in frontend + backend message handler. |
| Wrong WebSocket library (gorilla without write pump) | MEDIUM | Switch to coder/websocket — API is similar but idiomatic. Requires refactoring all write calls. |
| Plaintext passwords in SQLite | HIGH | Must migrate: add encrypted column, encrypt existing passwords, drop plaintext column. Risk of data loss during migration. Better to not store at all. |
| wterm can't handle SSH terminal needs | MEDIUM | Swap to xterm.js. Frontend component changes but WebSocket protocol stays the same if designed correctly. |
| No auth on exposed server | LOW | Bind to 127.0.0.1. Add env var for bind address. Add auth system later. |
| Host key not verified | LOW | Add known_hosts table to SQLite, add verification UI. Can be added incrementally. |

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| SSH session zombie leaks | Phase 1: WebSocket↔SSH proxy | Close tab, check `ps aux` on target server — no orphaned processes |
| Terminal resize propagation | Phase 1: WebSocket↔SSH proxy | Resize browser, run `stty size` — values match terminal |
| WebSocket concurrent writes | Phase 1: WebSocket↔SSH proxy | Use `coder/websocket` — verified by library guarantees |
| Password storage encryption | Phase 2: Connection management | `sqlite3` CLI shows ciphertext, not plaintext |
| wterm library maturity | Phase 0: Spike | `vim`, `htop`, `less` all render correctly |
| No auth = open proxy | Phase 1: Backend setup | Server binds to 127.0.0.1 only |
| HostKeyCallback mishandling | Phase 1: WebSocket↔SSH proxy | First connection shows fingerprint prompt |
| Special key passthrough | Phase 1: WebSocket↔SSH proxy | Ctrl+C, Tab, arrow keys work in shell |
| Connection status UI | Phase 2: Frontend polish | Status indicator shows connecting/connected/disconnected |
| Copy/paste support | Phase 2: Frontend polish | Native browser copy/paste works in terminal |

## Sources

- **golang.org/x/crypto/ssh** — Official Go SSH package documentation (pkg.go.dev), verified `RequestPty()`, `WindowChange(h,w)`, `StdinPipe()`, `TerminalModes`, `InsecureIgnoreHostKey()` API signatures. HIGH confidence.
- **gorilla/websocket** — Context7 documentation confirms single-reader/single-writer concurrency constraint, write pump pattern. HIGH confidence.
- **coder/websocket** (github.com/coder/websocket) — pkg.go.dev documentation confirms concurrent write support, `context.Context` integration, `CloseRead()`, `NetConn()` wrapper. HIGH confidence.
- **@wterm/react, @wterm/core** — npm registry confirms v0.2.0 (published 2026-04-26), WASM/Zig-based, DOM renderer, WebSocket transport built-in. HIGH confidence on existence, LOW confidence on stability/maturity.
- **wterm SSH example** — GitHub vercel-labs/wterm/examples/ssh uses Node.js + ssh2 (not Go). README confirms WebSocket ↔ SSH bridge pattern, password and key auth support. HIGH confidence.
- **wterm.dev homepage** — Confirms features: VT100/VT220/xterm escape sequences, alternate screen buffer, dirty-row tracking, ResizeObserver auto-resize, WebSocket transport with reconnection. HIGH confidence.
- **xterm.js documentation** — Context7 docs on FitAddon, resize patterns, WebSocket integration, WebGL addon. Used as reference for terminal emulation patterns that wterm should also support. HIGH confidence.

---
*Pitfalls research for: Web-based SSH terminal client*
*Researched: 2026-04-27*
