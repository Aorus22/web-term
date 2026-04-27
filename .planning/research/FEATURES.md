# Feature Landscape

**Domain:** Web-based SSH terminal client (self-hosted, individual developer target)
**Researched:** 2026-04-27

## Competitor Feature Matrix

Based on analysis of Termius, WebSSH, Shellngn, WebSSH2, ttyd, Next Terminal, Apache Guacamole, and Wetty.

| Feature | Termius (Free) | WebSSH2 | ttyd | Next Terminal | Guacamole | WebTerm v1 Target |
|----------|---------------|---------|------|---------------|-----------|-------------------|
| SSH protocol | ✅ | ✅ | ❌ (local only) | ✅ | ✅ | ✅ |
| SFTP/File transfer | ✅ | ❌ | ZMODEM | ❌ | ✅ | ❌ (v2) |
| Save connections | ✅ (local vault) | Config file | CLI flags | ✅ DB | ✅ DB | ✅ SQLite |
| Multi-tab sessions | ✅ | ❌ | Multiple clients | ✅ | ✅ | ✅ |
| Terminal emulator | Native | xterm.js | xterm.js (WebGL2) | guacamole proto | guacamole proto | @wterm/react |
| Port forwarding | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ (v2) |
| Snippets | ✅ (Pro) | ❌ | ❌ | ❌ | ❌ | ❌ (v2) |
| Session recording | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ (v2) |
| Auth system | ✅ | Basic HTTP | Basic/Credential | ✅ RBAC | ✅ | ❌ (v1 no auth) |
| SSH key auth | ✅ | ✅ | ❌ | ✅ | ✅ | ❌ (v1 password) |
| Password auth | ✅ | ✅ | N/A | ✅ | ✅ | ✅ |
| Mobile responsive | ✅ | Partial | ✅ | ✅ | ✅ | ❌ (desktop v1) |
| Cross-device sync | ✅ (Pro) | ❌ | ❌ | ❌ | ❌ | ❌ |
| Host grouping | ✅ | ❌ | ❌ | ✅ | ✅ | ❌ (v2) |
| Custom themes | ✅ | Partial | ✅ CSS | ❌ | ❌ | ✅ (built-in) |
| Self-hosted | ❌ (cloud app) | ✅ | ✅ | ✅ | ✅ | ✅ |
| WebSocket transport | N/A (native) | ✅ socket.io | ✅ | ✅ | Own protocol | ✅ |

**Sources:** Termius pricing page (termius.com/pricing, verified 2026-04-27), WebSSH2 GitHub README, ttyd GitHub README, Next Terminal GitHub, Apache Guacamole official site. **Confidence: HIGH**

---

## Table Stakes

Features users expect. Missing any = product feels broken or incomplete for the "SSH from browser" use case.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **SSH connection** | Core value proposition — this IS the product | High | WebSocket proxy pattern: browser → Go backend → SSH target. Go `golang.org/x/crypto/ssh` library is mature and well-documented. **Confidence: HIGH** |
| **Password authentication** | Most basic SSH auth method; every SSH client supports it | Med | Implement via SSH library auth callback. Must handle password prompts correctly (keyboard-interactive). **Confidence: HIGH** |
| **Save connections** | Re-typing host/port/user every time is unacceptable | Low | CRUD API over SQLite. Store: host, port, username, label, optional color/tag. No passwords stored in v1 (user enters each session). **Confidence: HIGH** |
| **Multi-tab terminal** | Developers routinely SSH into multiple servers simultaneously | Med | Browser tab is not sufficient — users need in-app tabs with independent sessions, each with its own WebSocket connection. Tab bar UI pattern similar to browser tabs. **Confidence: HIGH** |
| **Terminal emulator fidelity** | Must handle vim, htop, tmux, curses apps correctly | Med | @wterm/react (by Vercel Labs) provides DOM-based rendering with VT100/VT220/xterm escape sequence support, alternate screen buffer, 24-bit color, scrollback. Only 52 commits, v0.2.0 — **very new, moderate risk**. **Confidence: MEDIUM** — library is young, edge cases likely |
| **Copy/paste** | Non-negotiable for any terminal | Low | @wterm uses DOM rendering → native browser text selection and clipboard work out of the box. This is a major advantage over xterm.js canvas/WebGL rendering. **Confidence: HIGH** |
| **Auto-resize terminal** | Terminal must resize when window/pane resizes | Low | @wterm supports `ResizeObserver`-based auto-resize. Backend must handle `pty.Resize()` on resize events via WebSocket message. **Confidence: HIGH** |
| **Connection status indicator** | Users must know if connected, connecting, or disconnected | Low | Status bar or tab indicator: green/yellow/red. WebSocket lifecycle events drive state. **Confidence: HIGH** |
| **Reconnection** | Network interruptions happen; auto-reconnect is expected | Med | @wterm has built-in WebSocket reconnection support. Must re-establish SSH session (or prompt for reconnect). **Confidence: HIGH** |
| **Clean, modern UI** | Target is developer who values aesthetics (Vercel-inspired) | Med | shadcn/ui provides the component library. Minimalist layout: sidebar for connections, tab bar for sessions, terminal fills remaining space. **Confidence: HIGH** |

---

## Differentiators

Features that set the product apart. Not expected in a v1 web SSH client, but valued by power users.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Self-hosted, single binary** | Run anywhere — no cloud account, no subscription. `docker run` or download binary. | Low | Go binary + embed React build. SQLite embedded. This IS the primary differentiator vs Termius (SaaS). **Confidence: HIGH** |
| **DOM-based terminal (no canvas)** | Native text selection, browser find (Ctrl+F), accessibility (screen readers), standard clipboard. Most competitors use xterm.js with canvas rendering. | N/A (lib choice) | @wterm renders to DOM nodes instead of canvas/WebGL. Unique advantage — no other web SSH client does this yet. **Confidence: HIGH** |
| **Zero-config startup** | No database setup, no config files. Run binary → open browser → start SSHing. | Low | SQLite embedded, default port, auto-open browser. Frictionless first-run experience. **Confidence: HIGH** |
| **Connection grouping/tags** | Organize servers by project, environment (prod/staging/dev). Termius charges for this in Pro. | Low-Med | Simple tag/color system on connections. Filter sidebar by tag. SQLite-friendly. **Confidence: MEDIUM** — depends on UX design complexity |
| **Quick-connect bar** | Type `user@host:port` and connect immediately without saving. Power-user feature. | Low | URL-bar style input. Parse format, create ephemeral session. **Confidence: HIGH** |
| **Dark/light theme** | Developer expectation. Terminal and UI must respect system preference. | Low | shadcn supports theme switching. @wterm has built-in themes (Default, Solarized Dark, Monokai, Light). Sync UI theme with terminal theme. **Confidence: HIGH** |
| **Keyboard shortcuts** | Power users live in keyboard. New tab (Ctrl+T), close tab (Ctrl+W), switch tab (Ctrl+1-9). | Low | Standard keyboard shortcut pattern. Must not conflict with terminal input (only capture when focus is on tab bar / outside terminal). **Confidence: HIGH** |
| **Export/import connections** | Backup connections as JSON, migrate between machines. | Low | Simple JSON dump of SQLite connections table. **Confidence: HIGH** |

---

## Anti-Features

Features to explicitly NOT build. These are traps that dilute the product or introduce scope creep.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| **Built-in authentication system** | Complex (session management, password hashing, JWT/OAuth), distracts from core terminal experience. Premature for individual developer target. | v1: No auth. If deployed on localhost, no need. Add basic auth or reverse-proxy auth in v2. |
| **SFTP/File manager** | Entirely different UX paradigm (file tree, drag-and-drop, progress bars). Adds massive scope. Let users SCP/SFTP from their terminal. | v1: Terminal only. User can run `scp`, `rsync`, `sftp` commands in the terminal. Add SFTP panel in v2. |
| **SSH key management** | Security-sensitive (storing private keys server-side is risky). Requires encrypted storage, key generation UI. | v1: Password-only. Let users add key support in v2 with proper security model. |
| **Team/collaboration features** | Requires multi-user, permissions, audit logs — completely different product. | Individual-only for v1. Shared credentials and team vault are v3+ territory. |
| **Session recording/playback** | Complex (recording PTY output, storage, playback UI). Primarily enterprise/compliance feature. | Not needed for individual developer. Defer to v2+. |
| **Port forwarding UI** | Requires tunnel management, port conflict detection, active tunnel monitoring. Complex UX. | Users can set up tunnels via CLI. Port forwarding UI in v2. |
| **Mobile responsive design** | Terminal on mobile is fundamentally compromised (no physical keyboard, tiny viewport). Separate effort. | Desktop-first. Mobile-aware (don't break on mobile), but no mobile optimization. |
| **Plugin/extension system** | Massive scope. Premature abstraction. | Build features directly. Plugin system only when clear third-party integration demand exists. |
| **AI-powered autocomplete** | Termius has this. Cool but requires LLM integration, adds complexity and latency. | Skip for v1. Could be interesting differentiator later but not table-stakes. |
| **Mosh protocol support** | Niche. Mosh requires UDP which doesn't work through WebSocket proxy without significant engineering. | SSH only. Mosh support is not worth the WebSocket-over-UDP complexity. |

---

## Feature Dependencies

```
SSH Connection (core)
├── Terminal Emulator (@wterm/react)
│   ├── WebSocket Transport (browser ↔ Go backend)
│   │   └── Go SSH Library (backend ↔ target server)
│   └── Auto-resize (ResizeObserver + pty resize message)
├── Password Authentication
│   └── Connection Form UI (host, port, username, password)
└── Connection Management
    ├── Save Connections (SQLite CRUD)
    ├── Load Connections (sidebar list)
    └── Delete Connections

Multi-Tab Terminal
├── Tab Bar UI Component
├── Tab State Management (active sessions map)
└── Per-tab WebSocket connection lifecycle
    └── SSH Connection (per tab)

Connection Status
├── WebSocket lifecycle events
└── UI Status Indicator (tab + status bar)

Quick Connect
├── URL-bar input component
└── SSH Connection (ephemeral, no save)

Dark/Light Theme
├── shadcn Theme Provider
└── @wterm theme switching
    └── CSS custom properties sync

Keyboard Shortcuts
├── Global key listener (outside terminal focus)
└── Tab management actions
```

**Key dependency insight:** SSH Connection is the foundation. Everything depends on a working WebSocket ↔ SSH proxy. Multi-tab is N independent SSH sessions managed by tab state. Get one session working perfectly first, then multiply.

---

## MVP Definition (v1 Scope)

Aligned with PROJECT.md requirements and constraints.

### Must-Have (Ship Blockers)
1. **WebSocket SSH proxy** — Go backend accepts WebSocket, dials SSH target, pipes I/O
2. **Terminal rendering** — @wterm/react integrated, handles vim/htop/tmux correctly
3. **Connection form** — Enter host, port, username, password → connect
4. **Save connections** — CRUD to SQLite, list in sidebar
5. **Multi-tab** — Open multiple SSH sessions in tabs within single browser window
6. **Connection status** — Visual indicator of connected/connecting/disconnected per tab
7. **Copy/paste** — Native browser clipboard via DOM rendering
8. **Auto-resize** — Terminal resizes with window

### Should-Have (Valuable but Not Blockers)
9. **Quick-connect bar** — `user@host:port` one-shot connections
10. **Dark/light theme** — System preference detection + manual toggle
11. **Keyboard shortcuts** — Ctrl+T (new tab), Ctrl+W (close tab), Ctrl+Tab (switch)
12. **Connection grouping** — Tags/labels on saved connections
13. **Export/import connections** — JSON backup

### Won't-Have (Explicit v1 Exclusions)
- Authentication system
- SFTP/file transfer
- SSH key management
- Port forwarding UI
- Mobile responsive
- Session recording
- Team features

### MVP Success Criteria
A developer can:
1. Open the app in a browser
2. Save SSH connection details for 3+ servers
3. Click a saved connection → terminal opens with SSH session
4. Open multiple tabs → SSH into different servers simultaneously
5. Use vim, htop, tmux inside the terminal without issues
6. Copy text from terminal and paste into terminal
7. Resize browser window → terminal resizes correctly
8. Close tab → SSH session disconnects cleanly

---

## Detailed Competitor Analysis

### Termius (termius.com)
**Type:** SaaS + Native app (Mac/Win/Linux/iOS/Android)
**Stars:** N/A (proprietary), massive user base
**Strengths:**
- Polished cross-platform native apps
- Cloud sync across all devices
- Team collaboration features (shared vaults)
- AI-powered autocomplete
- Full protocol support: SSH, SFTP, Telnet, Mosh, Serial
- SSH key management with FIDO2 support
- Port forwarding, agent forwarding, jump hosts

**Weaknesses:**
- Subscription pricing ($10/mo Pro, $20/user/mo Team, $30/user/mo Business)
- Not self-hosted — data lives on their servers
- Free tier limited (local vault only, no sync)
- Heavy electron-based desktop app

**WebTerm positioning:** Self-hosted alternative for individual developers who don't want cloud dependency or subscriptions. Trade sync and mobile for privacy and simplicity.

### WebSSH2 (github.com/billchurch/webssh2)
**Type:** Self-hosted web app
**Stars:** 2.7k
**Strengths:**
- Simple web SSH client using xterm.js
- Multiple auth methods (password, key-based, SSO)
- Telnet support
- Host key verification
- Environment forwarding

**Weaknesses:**
- No connection saving (config file only)
- No multi-tab (single session per browser tab)
- Node.js backend (heavier than Go binary)
- Dated UI
- No mobile support

**WebTerm positioning:** Modern UI + connection management + multi-tab + Go binary. WebTerm is WebSSH2 with better UX.

### ttyd (github.com/tsl0922/ttyd)
**Type:** CLI tool for sharing local terminal
**Stars:** 11.5k
**Strengths:**
- Extremely simple — one command to share terminal
- WebGL2 rendering (fast)
- ZMODEM file transfer
- Basic authentication
- SSL support
- C binary — very lightweight

**Weaknesses:**
- Shares LOCAL terminal, not SSH client to remote servers
- No connection management
- No multi-tab
- Single-user use case (terminal sharing)

**WebTerm positioning:** Different product category. ttyd is for sharing your terminal; WebTerm is for managing SSH connections to remote servers.

### Next Terminal (github.com/dushixiang/next-terminal)
**Type:** Self-hosted web app (enterprise-focused)
**Stars:** 5.5k
**Strengths:**
- Multi-protocol: SSH, RDP, VNC, Telnet
- Session recording and playback
- RBAC user management
- Audit tracking
- Built on Apache Guacamole

**Weaknesses:**
- Enterprise focus — overkill for individual developer
- Heavy (requires Guacamole backend)
- Complex setup
- Chinese-first documentation

**WebTerm positioning:** Lightweight individual alternative. No Guacamole dependency, no RBAC overhead, simple SQLite.

### Apache Guacamole (guacamole.apache.org)
**Type:** Enterprise remote desktop gateway
**Stars:** N/A (Apache project)
**Strengths:**
- Mature, battle-tested
- Multi-protocol: VNC, RDP, SSH
- Well-documented API
- Enterprise features (LDAP, SSO, session recording)
- Clientless (pure HTML5)

**Weaknesses:**
- Complex setup (guacamole-server C component + Java web app)
- Enterprise-oriented
- Heavy infrastructure requirements
- UI is functional, not polished

**WebTerm positioning:** Single binary, no Java, no C compilation, modern React UI. For the developer who wants `docker run` and go.

### Wetty (github.com/butlerx/wetty)
**Type:** Self-hosted web terminal
**Stars:** 5.3k
**Strengths:**
- Simple xterm.js-based terminal
- Docker support
- SSL support
- Node.js based

**Weaknesses:**
- Shares LOCAL terminal (like ttyd), not SSH client
- No connection management
- Dated UI
- No multi-tab

**WebTerm positioning:** Same category distinction as ttyd. Wetty shares local terminal; WebTerm connects to remote servers.

---

## Key Insights for Roadmap

### Critical Path
1. **WebSocket ↔ SSH proxy is the hardest part** — Get this right first. Everything else is UI.
2. **@wterm is very new (v0.2.0, 52 commits)** — May encounter bugs with complex terminal apps. Need dedicated exploration phase.
3. **Multi-tab = N × (single session)** — Build one perfect session, then replicate.

### Feature Ordering Recommendation
1. Phase 1: SSH proxy + basic terminal (proof of life)
2. Phase 2: Connection management (save/load/delete)
3. Phase 3: Multi-tab UI
4. Phase 4: Polish (themes, shortcuts, quick-connect, status)

### Risk Factors
- **@wterm maturity** — Library published yesterday (v0.2.0, 2026-04-26). SSH example exists in their repo using Next.js. No Vite example yet. May need to fall back to xterm.js if critical bugs found.
- **WebSocket proxy performance** — Large terminal output (logs, cat large files) must be handled efficiently. Backpressure on WebSocket is a real concern.
- **Terminal fidelity** — Programs like vim, tmux, htop exercise terminal emulation heavily. Must test early and often.

---

## Sources

- Termius pricing/feature comparison: https://termius.com/pricing (fetched 2026-04-27) — **HIGH confidence**
- WebSSH2 GitHub: https://github.com/billchurch/webssh2 — **HIGH confidence**
- ttyd GitHub: https://github.com/tsl0922/ttyd — **HIGH confidence**
- Next Terminal GitHub: https://github.com/dushixiang/next-terminal — **HIGH confidence**
- Apache Guacamole: https://guacamole.apache.org/ — **HIGH confidence**
- Wetty GitHub: https://github.com/butlerx/wetty — **HIGH confidence**
- wterm.dev official: https://wterm.dev — **HIGH confidence**
- wterm GitHub (vercel-labs): https://github.com/vercel-labs/wterm — **HIGH confidence** (2.5k stars, Vercel Labs)
- wterm npm packages: @wterm/core, @wterm/dom, @wterm/react (v0.2.0, published 2026-04-26) — **HIGH confidence**
- WebSSH (iOS/macOS): https://webssh.net — **MEDIUM confidence** (different platform, feature overlap only)
- xterm.js official: https://xtermjs.org — **HIGH confidence**
