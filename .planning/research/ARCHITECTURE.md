# Architecture Research

**Domain:** Native desktop SSH client (GPUI frontend over existing Go backend)
**Researched:** 2026-09-04
**Confidence:** MEDIUM-HIGH (patterns verified against Zed's real terminal architecture and gpui-terminal's documented design; integration specifics inferred from this repo's backend)
**Note:** Produced inline by the orchestrator (generic inline workaround — no subagent runtime in this session).

## Standard Architecture

### System Overview

```
┌──────────────────────────────────────────────────────────────────┐
│                     desktop-gpui/ (Rust, GPUI)                   │
│                                                                  │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐   │
│  │ Hosts view │ │ Keys view  │ │ SFTP view  │ │ Settings   │   │
│  └─────┬──────┘ └─────┬──────┘ └─────┬──────┘ └─────┬──────┘   │
│        └──────┬───────┴──────┬───────┴──────┬───────┘          │
│         ┌─────┴──────────────┴──────────────┴─────┐            │
│         │   App shell: tabs, theme, keybinds      │            │
│         └─────┬───────────────────────────────────┘            │
│         ┌─────┴───────────────────────────────────┐            │
│         │  Terminal view (alacritty_terminal)     │            │
│         │  grid + VTE parse + GPUI render         │            │
│         └─────┬───────────────────────────────────┘            │
│         ┌─────┴───────────────────────────────────┐            │
│         │  backend-client crate                    │            │
│         │  WS (terminal I/O) + REST (CRUD)         │            │
│         └─────┬───────────────────────────────────┘            │
│         ┌─────┴───────────────────────────────────┐            │
│         │  Backend supervisor (child process)      │            │
│         │  spawn → health poll → graceful kill     │            │
│         └─────┬───────────────────────────────────┘            │
└───────────────┼──────────────────────────────────────────────────┘
                │ loopback HTTP + WebSocket
┌───────────────┴──────────────────────────────────────────────────┐
│         webterm Go backend (existing, unmodified)                 │
│  REST (hosts/keys/sftp/fwd CRUD) · WS SSH proxy · SQLite          │
│  (AES-256-GCM credentials) · local PTY (POSIX) · port forwards    │
└──────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Typical Implementation |
|-----------|----------------|------------------------|
| App shell | Window, tab strip, 2-page nav, theme state, global keybinds | GPUI entities + gpui-component themes; one `WebTermAppState` entity owning UI state |
| Terminal view | VT emulation + rendering of one session | alacritty_terminal `Term` state + GPUI render element; gpui-terminal crate or vendored Zed pattern |
| Session manager | Maps tabs ↔ backend WS sessions; re-attach on reload/refresh | Mirrors backend session-manager concept; one WS connection per terminal tab |
| backend-client | Typed REST + WS client for the Go API | reqwest + tokio-tungstenite; serde types mirrored from backend DTOs |
| Backend supervisor | Owns child process lifecycle | std/tokio `Child`, health polling on the backend's existing health/status endpoint, kill on app exit (tokio::process kill_on_drop) |
| Desktop settings | Window state, theme, backend binary path, auto-start-backend | JSON file in native config dir; never duplicates SSH data |

## Recommended Project Structure

```
desktop-gpui/
├── Cargo.toml                  # [workspace] members = ["crates/*"]
├── crates/
│   ├── webterm/               # the GPUI application (bin)
│   │   ├── src/
│   │   │   ├── main.rs        # Application::new(), backend spawn, window open
│   │   │   ├── app_state.rs   # root entity: tabs, active view, theme
│   │   │   ├── views/
│   │   │   │   ├── hosts.rs        # host cards, kebab menus, quick-connect
│   │   │   │   ├── keys.rs         # key pool, upload, passphrase
│   │   │   │   ├── sftp.rs         # dual-pane manager
│   │   │   │   ├── settings.rs     # desktop settings page
│   │   │   │   └── terminal_tab.rs # terminal tab container
│   │   │   └── theme.rs       # dark/light palettes incl. terminal colors
│   │   └── Cargo.toml
│   ├── terminal/              # alacritty_terminal + GPUI render bridge
│   │   └── src/
│   │       ├── term_state.rs  # Term, parser, scrollback, selection
│   │       ├── renderer.rs    # grid → GPUI paint primitives
│   │       └── input.rs       # keystroke → escape sequences
│   ├── backend-client/        # REST + WS client for the Go backend
│   │   └── src/
│   │       ├── rest.rs        # hosts, keys, sftp, forwards, health
│   │       ├── ws_terminal.rs # terminal I/O framing (matches backend protocol)
│   │       └── types.rs       # serde DTOs mirroring Go structs
│   └── backend-supervisor/    # child-process lifecycle (small, testable)
│       └── src/lib.rs         # spawn, wait healthy, kill; port acquisition
└── dist/                      # bundled backend binaries per target triple
```

### Structure Rationale

- **crates/webterm:** all UI; the only crate that depends on gpui/gpui-component directly — keeps framework churn contained when gpui 0.x breaks APIs.
- **crates/terminal:** the alacritty bridge lives alone so a gpui-terminal → vendored-Zed swap never touches app code.
- **crates/backend-client:** mirrors the backend's DTOs; the seam where protocol drift is caught by tests.
- **crates/backend-supervisor:** tiny and separately testable — spawn/health/kill logic with no GPUI dependency.

## Architectural Patterns

### Pattern 1: Backend-as-child-process (sidecar) with health gate

**What:** Desktop binary launches the bundled Go backend on a free loopback port, polls its health endpoint until ready, then opens the UI; on exit, terminates the child.
**When to use:** Whenever a desktop shell must own a service dependency — Tauri sidecars, Electron spawn patterns.
**Trade-offs:** One-click UX and no install step vs process-supervision code (zombie processes, port races, crash loops).

**Example:**
```rust
// tokio::process with kill_on_drop; port 0 or ephemeral probe
let mut backend = tokio::process::Command::new(backend_path)
    .arg(format!("--addr=127.0.0.1:{}", port))
    .kill_on_drop(true)
    .spawn()?;
wait_until_healthy(format!("http://127.0.0.1:{port}/health")).await?;
```

### Pattern 2: Terminal state/render separation (Zed's proven split)

**What:** alacritty_terminal owns grid + parser (`Term`); a GPUI render element reads grid state each frame and batches styled cells into paint primitives. I/O arrives on a background task, is fed through the parser, then wakes the render entity.
**When to use:** Any GPUI terminal embedding — this is exactly how Zed's `terminal` + `terminal_view` crates and gpui-terminal are built.
**Trade-offs:** Push-based wakeups are efficient; the cost is that scrollback/selection live in the state layer and must be implemented (or sourced from gpui-terminal) rather than inherited from a widget.

**Example:**
```rust
// per-frame render reads grid; input writes bytes; parser mutates state
impl Render for TerminalElement { /* read Term grid → batch cells → paint */ }
```

### Pattern 3: Session re-attach across app restarts

**What:** The backend already persists sessions and maps IDs to live SSH connections (validated v0.3.0). The desktop app reuses the same contract: after reconnect, list sessions and re-attach instead of reconnecting SSH.
**When to use:** App crash/restart without losing remote sessions.
**Trade-offs:** Free parity with the web app vs desktop must treat "backend died" differently from "WS dropped".

**Example:**
```rust
// on WS reconnect: GET /sessions → re-attach each tab by stored session id
```

## Data Flow

### Terminal I/O Flow

```
Keystroke (GPUI window)
    ↓
input.rs → escape sequence bytes
    ↓
WS send → Go backend → SSH channel → remote host
    ↓ (output path, reverse)
SSH channel → Go backend → WS message
    ↓
tokio task → alacritty_terminal VTE parser → Term grid
    ↓ notify
GPUI render → paint cells
```

### State Management

```
WebTermAppState (root entity)
    ↓ (subscribe/notify)
Views (hosts/keys/sftp/settings) ←→ backend-client (reqwest calls)
Terminal tabs → Session manager → WS tasks → Terminal state entities
```

### Key Data Flows

1. **Session lifecycle:** open tab → resolve host → WS dial → backend spawns SSH → stream.
2. **SFTP ops:** pane path changes → REST list → render entries; transfers stream over WS/HTTP with progress events.
3. **Theme change:** settings entity → gpui-component theme swap + terminal ColorPalette update (runtime `update_config`).

## Scaling Considerations

| Scale | Architecture Adjustments |
|-------|--------------------------|
| 1 user, <20 tabs | Nothing — single process, single backend |
| Heavy output (yes/cat large file) | Batched paint + parser on background thread; gpui-terminal's flume-channel pattern is the mitigation |
| Many SFTP transfers | Queue with per-transfer progress; avoid unbounded buffering |

### Scaling Priorities

1. **First bottleneck:** terminal output throughput — solved by architecture (background parse + push-based repaint).
2. **Second bottleneck:** font shaping cost per frame — cache glyph runs; follow Zed's text system usage.

## Anti-Patterns

### Anti-Pattern 1: Duplicating SSH state in Rust

**What people do:** Keep a parallel Rust-side model of connections/keys "for speed".
**Why it's wrong:** Two sources of truth for credentials/sessions → drift, double prompts, security surprises.
**Do this instead:** Backend is the only owner of SSH state; Rust holds UI state + session IDs only.

### Anti-Pattern 2: Hand-rolling the terminal grid

**What people do:** Parse VT sequences ad hoc for a head start.
**Why it's wrong:** VT fidelity is a decade of edge cases (the very reason alacritty_terminal/Zed exists).
**Do this instead:** alacritty_terminal for state; GPUI only renders.

### Anti-Pattern 3: Unpinned UI-framework dependencies

**What people do:** `gpui = "0.2"` style loose pins during fast iteration.
**Why it's wrong:** Pre-1.0 minors break builds silently weeks later.
**Do this instead:** Exact pins via Cargo.lock + deliberate upgrade PRs.

## Integration Points

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| Go backend (bundled) | Child process + loopback REST/WS | Health-gate UI startup; reuse every existing endpoint untouched |
| OS clipboard | arboard (OSC 52 callback + Ctrl+C/V paths) | Wayland via data-control |
| OS config dir | dirs crate → JSON settings | Desktop prefs only |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| webterm ↔ backend-client | Typed async calls | The only crate that knows HTTP/WS details |
| webterm ↔ terminal | Terminal state entity API | Hides alacritty types from views |
| webterm ↔ supervisor | Spawn/health/kill handles | Supervisor returns `BackendHandle` with base URL |

## Sources

- Zed crates: `terminal` (alacritty-backed state) + `terminal_view` README — the state/render split this design mirrors (HIGH)
- docs.rs/gpui-terminal — documented TerminalView architecture: background reader thread, flume channel, push-based repaint, callback surface (HIGH)
- v2.tauri.app/develop/sidecar — sidecar lifecycle conventions incl. port-free checking (MEDIUM)
- tokio::process docs — kill_on_drop supervision pattern (HIGH)
- WebTerm backend codebase — REST/WS surface, session re-attach contract, creack/pty local terminal (HIGH, internal)

---
*Architecture research for: WebTerm Desktop (GPUI client over Go backend)*
*Researched: 2026-09-04*
