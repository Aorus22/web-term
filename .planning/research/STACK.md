# Stack Research

**Domain:** Native desktop SSH client (GPUI frontend over existing Go backend)
**Researched:** 2026-09-04
**Confidence:** MEDIUM-HIGH (verified via docs.rs, crates.io, official Zed/GPUI sources; versions approximate — verify at `cargo add` time)
**Note:** Produced inline by the orchestrator (generic inline workaround — no subagent runtime in this session). Content follows the standard gsd-project-researcher contract.

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| gpui | 0.2.x (crates.io) | GPU-accelerated UI framework | The framework Zed itself ships on; now published standalone on crates.io with Windows + Linux (X11/Wayland) support. User's explicit choice. Pre-1.0: pin exact version, expect breaking changes between minors. |
| gpui-component | latest (Longbridge) | shadcn-inspired component library (60+ widgets, theming, tabs, dock) | Same design language as WebTerm's web UI (shadcn), proven in production at Longbridge; gives us light/dark themes, side/nav bars, inputs, menus instead of hand-rolling. |
| alacritty_terminal | 0.2x (pin exact) | VT emulation core (grid, VTE parser, ANSI/256/truecolor) | Same engine Zed's integrated terminal uses — the user's explicit requirement ("yang dipakai Zed, sudah terbukti"). Battle-tested against vim/htop/tmux. 0.x semver: minor bumps are breaking — pin exactly. |
| tokio + tokio-tungstenite | 1.x | Async runtime + WebSocket client | Connects to the Go backend's existing WS protocol (terminal I/O, SFTP events) and drives the I/O pipeline that feeds alacritty_terminal. |
| serde / serde_json | 1.x | REST/WS payload types | The Go backend speaks JSON; mirror the existing DTOs. |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| portable-pty | 0.8+ | Cross-platform PTY (ConPTY on Windows, OpenPTY on Unix) | Only if local-terminal is handled Rust-side; the Go backend currently uses creack/pty which is POSIX-only — Windows local terminal needs a ConPTY path somewhere. |
| reqwest | 0.12 | REST client for connection/key CRUD | Hosts, keys, SFTP listing, port-forward management against existing Go REST API. |
| arboard | latest | System clipboard | Terminal copy/paste (gpui-terminal already uses it internally). |
| dirs / etcetera | latest | Platform config dirs | Desktop-only settings (backend binary path, window state) in a native config dir — SSH data stays in the backend's SQLite. |
| tracing + tracing-subscriber | 0.1/0.3 | Logging | File-based logs for desktop diagnostics (no console in a windowed app). |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| cargo workspace | Single workspace rooted at `desktop-gpui/` | Crates: `app` (bin), optional `backend-client` (WS/REST types) split if the surface grows. |
| Go toolchain (existing) | Build the backend binary to bundle | Desktop app spawns `webterm-backend` (or existing server binary name) as child process. |
| GitHub Actions matrix | CI builds for `windows-latest` + `ubuntu-latest` | GPUI needs Mesa/Vulkan-ish deps on Linux runners; DirectX on Windows is supported by gpui 0.2. |
| cargo-deny / dependabot | Dependency pinning | Pre-1.0 ecosystem — surface bumps deliberately, never passively. |

## Installation

```bash
# Inside desktop-gpui/ (new cargo workspace)
cargo new --lib backend-client   # WS/REST client for the Go backend
cargo new --bin webterm          # GPUI application
# then in Cargo.toml files:
# gpui = "=0.2.2"                # exact pin (illustrative; resolve at add time)
# gpui-component = "0.x"         # pin minor
# alacritty_terminal = "=0.25.x" # exact pin (illustrative; resolve at add time)
```

Go side: no changes required for SSH/SFTP/port-forward parity; local-terminal-on-Windows may require a ConPTY addition (see ARCHITECTURE.md).

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| gpui-terminal crate for the terminal view | Port Zed's `terminal` crate approach (alacritty_terminal + custom GPUI renderer, like Zed's `terminal_view`) | gpui-terminal has gaps (scrollback and mouse selection listed as planned/partial). Spike it in the first terminal phase; if scrollback/selection block parity, vendor/port Zed's renderer pattern instead. |
| gpui-component for UI | Hand-rolled GPUI widgets | Only if a needed component is missing; gpui-component covers nav bars, inputs, menus, tabs, theming. |
| Bundling the Go backend as child process | Native Rust SSH (russh) | Never this milestone — user locked GPUI + Go backend; russh would discard a proven, encrypted-credential, session-persistent backend. |
| Keep web UI as-is, desktop beside it | Retire web UI | Desktop-replaces-web is explicitly not this milestone's goal. |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| Floating gpui/gpui-component versions | gpui is pre-1.0 — "there will often be breaking changes between versions" (official docs.rs). A passive minor bump can brick the build. | Exact/minor pins + deliberate upgrades. |
| Floating alacritty_terminal | 0.x semver: minor = breaking. Zed tracks it closely; we shouldn't. | Pin exact version; upgrade with intent. |
| Electron wrapper (`desktop/`) or the Tauri scaffold (`desktop-tauri/`) for new work | Superseded by this milestone's decision; maintaining three desktop shells fragments effort. | GPUI app; mark the old dirs deprecated in docs. |
| A second SQLite/settings store for SSH data | Divergence risk with the backend's AES-256-GCM credential store. | All SSH data via backend API; Rust config file holds desktop-only prefs only. |

## Stack Patterns by Variant

**If local terminal on Windows must land this milestone:**
- Either add a ConPTY backend to the Go local-terminal endpoint, or bypass it with portable-pty in Rust and feed the same WS-shaped I/O into the terminal view. Decide during the local-terminal phase; POSIX (Linux) can always use the existing backend path.

**If gpui-terminal proves too immature during the terminal spike:**
- Timebox the spike; fall back to vendoring the Zed `terminal`/`terminal_view` pattern (alacritty_terminal + custom render element). Budget: it is a known-quantity port, not research.

## Version Compatibility

| Package A | Compatible With | Notes |
|-----------|-----------------|-------|
| gpui 0.2.x | gpui-component (check its gpui dep pin) | gpui-component tracks gpui closely; keep their pins in lockstep. |
| alacritty_terminal | gpui-terminal (or vendored Zed code) | gpui-terminal declares an alacritty_terminal version; if vendoring, we choose the pin. |
| Go backend | any desktop release | Protocol compatibility guarded by the backend's existing REST/WS versioning; desktop consumes the same API the web UI uses. |

## Sources

- docs.rs/gpui (gpui 0.2.2) — standalone crate, pre-1.0 churn warning, platform support (HIGH)
- gpui.rs — official standalone site; component guidance drawn from Zed's crates (HIGH)
- docs.rs/gpui-terminal — TerminalView over alacritty_terminal, feature matrix incl. gaps (HIGH)
- crates.io/crates/gpui-component + longbridge.github.io/gpui-component — 60+ components, shadcn-inspired, production use at Longbridge (HIGH)
- zed.dev/docs/terminal + zed/crates/terminal_view README — Zed terminal is Alacritty-backed; abstraction boundary in `terminal` crate (HIGH)
- reddit/HN gpui-component thread — gpui hit crates.io only weeks before Oct 2025; git-only before that (MEDIUM)
- v2.tauri.app sidecar docs — lifecycle patterns for embedded backend binaries (MEDIUM, pattern-level)

---
*Stack research for: WebTerm Desktop (GPUI client over Go backend)*
*Researched: 2026-09-04*
