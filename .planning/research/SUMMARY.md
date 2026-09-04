# Project Research Summary

**Project:** WebTerm
**Domain:** Native desktop SSH client (GPUI frontend over existing Go backend)
**Researched:** 2026-09-04
**Confidence:** MEDIUM-HIGH

## Executive Summary

WebTerm v0.5.0 adds a native desktop client built with GPUI — the GPU-accelerated framework Zed ships on, now published standalone on crates.io (0.2.x) with Windows + Linux support — while deliberately reusing the existing Go backend (WebSocket SSH proxy, SQLite with AES-256-GCM, SFTP, port forwarding) as a locally-spawned child process. The terminal renders via `alacritty_terminal`, the same engine Zed's integrated terminal uses, satisfying the user's proven-engine requirement. This architecture converts the riskiest unknowns (SSH, credential storage, session persistence) into already-validated assets, and concentrates all genuine new risk in three areas: the GPUI terminal rendering bridge, the backend process supervisor, and Windows local-terminal PTY support.

The recommended approach is a four-crate cargo workspace (`webterm` app, `terminal` bridge, `backend-client`, `backend-supervisor`) that isolates gpui churn and makes the terminal implementation swappable. The decisive early decision is a timeboxed spike in the first terminal phase: the convenient `gpui-terminal` crate documents scrollback as "planned" and mouse selection as "partial", so the spike either validates it against real vim/htop + copy/paste workflows or falls back to porting Zed's proven `terminal`/`terminal_view` pattern — a known-quantity port, not new research.

Key risks and mitigations: (1) **pre-1.0 dependency churn** — pin exact versions, commit Cargo.lock, upgrade deliberately; (2) **process lifecycle bugs** — one supervisor path with ephemeral-port handoff, health-gated startup, and restart-loop tests; (3) **Windows ConPTY gap** — the backend's local terminal uses POSIX-only creack/pty, so the local-terminal phase must explicitly choose Go-side ConPTY or Rust-side portable-pty; (4) **parity drift** — every phase maps to specific validated REQ-IDs and a dedicated parity-audit phase closes the milestone.

## Key Findings

### Recommended Stack

GPUI anchors the stack; the frontend component layer and terminal engine slot in around it with strong existing art. Full detail in STACK.md.

**Core technologies:**
- **gpui 0.2.x (crates.io):** GPU-accelerated UI framework, standalone since late 2025; Windows (DirectX) + Linux (X11/Wayland). Pre-1.0 — pin exactly.
- **gpui-component (Longbridge):** 60+ shadcn-inspired components with theming — matches WebTerm's web aesthetic; proven in production.
- **alacritty_terminal (pinned 0.2x):** the Zed-proven VT engine (grid, VTE parser, truecolor); GPUI renders its grid state.
- **tokio + tokio-tungstenite / reqwest:** drives the backend-client (WS terminal I/O, REST CRUD) mirroring the existing Go API.
- **portable-pty (conditional):** Windows local-terminal fallback if ConPTY isn't added Go-side.

### Expected Features

Parity with the web app is the baseline; the desktop differentiates on native performance and one-click launch. Full landscape in FEATURES.md.

**Must have (table stakes):**
- Tabbed sessions with status dots, SSH + key/password auth, copy/paste + selection, scrollback, resize reflow, connection manager (cards/tags/quick-connect/import-export), dark/light theme, keyboard shortcuts, reconnection UX, settings persistence

**Should have (competitive):**
- GPU-rendered alacritty terminal (headline), dual-pane SFTP with drag-and-drop, one-click app (backend as child process), port-forwarding UI

**Defer (v2+):**
- Command palette, system tray, global hotkey, auto-updater (user deferred extras), macOS target

### Architecture Approach

A sidecar-style desktop shell: the GPUI app spawns the bundled Go backend on an ephemeral loopback port, health-gates startup, and speaks the exact REST/WS protocol the web UI uses; terminal state (alacritty_terminal `Term`) is separated from GPUI rendering exactly as Zed does, with push-based I/O wakeups; session re-attach reuses the backend's existing session-manager contract. Full design in ARCHITECTURE.md.

**Major components:**
1. **backend-supervisor** — spawn/health/kill lifecycle; loopback-only bind; stale-instance handling
2. **backend-client** — typed REST + WS client; the only crate that knows the protocol; DTO contract tests
3. **terminal** — alacritty state + GPUI render bridge; swappable behind a boundary (gpui-terminal vs vendored Zed pattern)
4. **webterm app** — shell, views (Hosts/Keys/SFTP/Settings), theme, keybinds; the only gpui-dependent crate

### Critical Pitfalls

1. **gpui-terminal feature gaps (selection/scrollback "planned")** — timeboxed spike with go/no-go in the first terminal phase; fallback is the vendored Zed pattern
2. **Pre-1.0 dependency churn** — exact pins + committed lockfile + deliberate upgrade PRs from the foundation phase
3. **Backend process lifecycle bugs** — single supervisor path, ephemeral port handoff, restart-loop and kill-mid-session tests
4. **Windows ConPTY gap for local terminal** — explicit Go-side vs Rust-side decision with Windows CI smoke test
5. **Parity drift** — REQ-ID mapping per phase + final parity-audit phase against PROJECT.md's validated list

## Implications for Roadmap

Based on research, suggested phase structure (numbering continues from v0.4.0 → Phase 20; eight phases map cleanly, adjust count as needed):

### Phase 20: Desktop Foundation & Backend Integration
**Rationale:** Nothing renders without the shell + a reachable backend; supervisor and client are prerequisites for every feature phase.
**Delivers:** Cargo workspace, pinned deps + CI, GPUI window with basic shell, backend-supervisor (spawn/health/kill), backend-client skeleton, startup UX.
**Addresses:** settings persistence, one-click launch.
**Avoids:** dependency churn pitfall (pins from day one), lifecycle bugs (tests here).

### Phase 21: Terminal Rendering Spike (go/no-go)
**Rationale:** The root risk; validate gpui-terminal selection + scrollback before building sessions on it.
**Delivers:** One working terminal view rendering local/WS-fed data through alacritty_terminal; spike report with go/no-go and (if no-go) the vendored-Zed fallback plan.
**Addresses:** terminal table stakes (selection, scrollback, truecolor).
**Avoids:** discovering gpui-terminal gaps after sessions are built on it.

### Phase 22: SSH Terminal Sessions & Tabs
**Rationale:** With rendering proven, wire real sessions — the product's core.
**Delivers:** SSH connect (password/key), multi-tab with status dots, resize sync, shortcuts, WS drop reconnection.
**Addresses:** core SSH parity items.
**Avoids:** UX drift on shortcuts (desktop conventions decided here).

### Phase 23: Hosts & SSH Keys Management
**Rationale:** CRUD surfaces are low-risk and unblock daily usability; backend API exists unchanged.
**Delivers:** Hosts cards/kebab/tags/search/quick-connect, import/export, key pool + passphrase flow, per-connection auth method.
**Addresses:** hosts + keys parity.

### Phase 24: Local Terminal (incl. Windows ConPTY decision)
**Rationale:** Isolated PTY decision with its own go/no-go and CI smoke test.
**Delivers:** Local terminal tab on Linux (backend path) + Windows (chosen ConPTY path); local-first New Tab ordering parity.
**Addresses:** local terminal parity on both platforms.
**Avoids:** silent Windows breakage of a validated feature.

### Phase 25: SFTP Dual-Pane Manager
**Rationale:** Largest UI surface; backend-client and session state are stable by now; can partially parallelize with Phase 23 if desired.
**Delivers:** Dual-pane browsing, local/remote sources, list/upload/download/delete/rename with streaming, drag-and-drop, external OS DnD.
**Addresses:** SFTP parity.

### Phase 26: Port Forwarding, Settings & Theme Polish
**Rationale:** Remaining parity surfaces are small once the shell and views exist.
**Delivers:** Port-forward management UI, settings page (desktop-scope; no engine selector), theme system with terminal palette sync, window-state persistence.
**Addresses:** forwarding + settings + theme parity.

### Phase 27: Parity Audit & Desktop Hardening
**Rationale:** Research flags parity drift as the failure mode of parity milestones; a dedicated close-out phase makes "daily-driveable" an explicit, verified bar.
**Delivers:** REQ-by-REQ audit vs PROJECT.md validated list, reconnection chaos tests (WS drop vs backend kill), performance pass on heavy output, packaging/installers for Windows + Linux.
**Addresses:** the done-criteria.
**Avoids:** shipping 90% parity.

### Phase Ordering Rationale

- Foundation → terminal spike → sessions follows the dependency chain (shell/engine → render proof → real I/O).
- Hosts/keys before SFTP because SFTP shares connection state; SFTP's size justifies its own phase.
- Local terminal is isolated so the ConPTY decision cannot destabilize SSH parity work.
- The audit phase converts the "22 validated requirements" checklist into gate criteria.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 21 (terminal spike):** gpui-terminal maturity unknowns; scrollback/selection implementation specifics
- **Phase 24 (local terminal):** ConPTY integration choice needs a concrete spike (Go wrapper availability vs portable-pty bypass)

Phases with standard patterns (skip research-phase):
- **Phase 23 (hosts/keys):** CRUD over existing API, well-understood
- **Phase 26 (settings/theme):** configuration + theming patterns are documented in gpui-component

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | MEDIUM-HIGH | gpui/gpui-component/alacritty_terminal verified on official sources; exact current versions to confirm at add time |
| Features | MEDIUM-HIGH | Parity baseline is our own validated list; competitor landscape consistent |
| Architecture | MEDIUM-HIGH | Mirrors Zed's real terminal split + Tauri sidecar conventions; protocol specifics from this repo's backend |
| Pitfalls | MEDIUM | Crate-maturity gaps documented; lifecycle/platform quirks pattern-level |

**Overall confidence:** MEDIUM-HIGH

### Gaps to Address

- **gpui-terminal real-world maturity:** resolve in Phase 21 spike with explicit go/no-go criteria
- **ConPTY integration path for Go:** verify availability/quality of Go ConPTY wrappers in Phase 24 planning before choosing Go-side vs Rust-side
- **Exact current crate versions:** confirm at `cargo add` time; STACK.md versions are approximate
- **gpui-component ↔ gpui pin lockstep:** verify compatibility matrix when pinning

## Sources

### Primary (HIGH confidence)
- docs.rs/crate/gpui (0.2.2) — standalone crate, pre-1.0 churn warning, platform support
- docs.rs/gpui-terminal — TerminalView architecture, callback surface, feature matrix incl. gaps
- crates.io/crates/gpui-component + longbridge.github.io/gpui-component — component set, production use
- zed.dev/docs/terminal + zed/crates/terminal_view README — Alacritty-backed terminal, abstraction boundary
- tokio::process / std::process docs — child supervision semantics

### Secondary (MEDIUM confidence)
- v2.tauri.app/develop/sidecar + plugins-workspace#3062 — sidecar lifecycle conventions and pain points
- creack/pty and portable-pty docs — POSIX-only vs ConPTY scopes
- Reddit/HN threads — gpui ecosystem timing, component library perception

### Tertiary (LOW confidence)
- Feature-comparison blog posts (SSH client landscape) — table-stakes perception only

---
*Research completed: 2026-09-04*
*Ready for roadmap: yes*
