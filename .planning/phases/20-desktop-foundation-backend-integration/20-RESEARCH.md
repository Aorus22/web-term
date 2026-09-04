# Phase 20 Research: Desktop Foundation & Backend Integration

**Phase:** 20 — Desktop Foundation & Backend Integration
**Researched:** 2026-09-04 (inline researcher — generic inline workaround; no subagent runtime in this session)
**Confidence:** HIGH for repo facts (verified by reading source); MEDIUM-HIGH for crate APIs (versions verified via crates.io API; API shapes from official docs)

## 1. Verified Crate Versions (crates.io API, 2026-09-04)

| Crate | Max stable | Notes |
|---|---|---|
| gpui | **0.2.2** | Pin exact; pre-1.0 churn is real (docs.rs warning). Windows (DirectX) + Linux (X11/Wayland). |
| gpui-component | **0.6.0** | Must be compatible with gpui 0.2.x — verify its Cargo dependency pin at add time. |
| tokio | 1.53.1 | Full features feature-flag. |
| portable-pty | 0.9.0 | NOT needed in Phase 20 (supervisor spawns the backend, not a PTY). |
| alacritty_terminal | 0.26.0 | NOT needed in Phase 20 (Phase 21). Listed to prevent accidental early adoption. |

Windows-specific gpui notes: DirectX renderer; no special system deps beyond standard toolchain. Linux: X11/Wayland backends — Wayland may need `wayland` feature flags depending on gpui's default features; CI matrix should build with defaults first.

## 2. Backend Integration Contract (verified from source — be/)

This is the load-bearing section. The supervisor's design is dictated by these facts:

### 2.1 Port handshake (already built — be/cmd/server/main.go)
- Backend prints `BACKEND_PORT:%d\n` to **stdout** after binding (`main.go:96`).
- `WEBTERM_PORT` env sets the listen spec (default `:8080`); the value `:0` yields an ephemeral port reported via the handshake.
- **Supervisor design:** spawn with `WEBTERM_PORT=:0`, capture stdout lines, parse `BACKEND_PORT:` → base URL `http://127.0.0.1:<port>`. Do NOT poll-guess ports.
- ⚠ The listener currently binds `0.0.0.0` (main.go:70) — not configurable via env. Loopback-only enforcement (QA-03) will need a small Go change in a later phase; do not attempt to work around it in Phase 20, but the supervisor must not advertise or expose the URL beyond the process.

### 2.2 Encryption key lifecycle (critical — be/internal/config/config.go)
- Without `WEBTERM_ENCRYPTION_KEY`, the backend generates a **random per-session** key and prints it (config.go:41-49). Stored passwords/keys in SQLite become **undecryptable on next launch**.
- **Supervisor must:** generate a 32-byte hex key once (on first run), persist it in the desktop settings file with **user-only permissions**, and pass it via env on every spawn. Never log it, never pass via argv (process-listing exposure).
- DB path: `WEBTERM_DB_PATH` (default `data/webterm.db`, relative to cwd). Desktop sets an absolute path under the app data dir so cwd doesn't matter.

### 2.3 Readiness probing (no /health endpoint exists)
- `routes.go` registers no health route; the listener starts **after** `db.Init` succeeds (main.go:26-29), so **any HTTP response on the port (including 404) proves readiness**.
- **Supervisor design:** after capturing BACKEND_PORT, poll `GET /api/settings` (cheap, side-effect-free 200) with timeout (e.g. 10s) → ready. A process-exit before readiness = startup failure (surface stderr tail in UI).

### 2.4 REST/WS surface (Phase 20 needs only a skeleton)
- Full CRUD: `/api/connections` (+ `/export`, `/import`), `/api/keys`, `/api/forwards` (+`/start`,`/stop`), `/api/settings`, `/api/sftp/*`, `/api/sessions`, `GET /ws` (SSH/local terminal I/O).
- Phase 20 backend-client needs: `get_settings` (readiness), one representative typed call (`list_connections`) to establish DTO mirroring + error handling, and the WS URL builder for later phases. Phase 22 expands the WS channel.

### 2.5 Lifecycle
- Graceful shutdown on SIGINT/SIGTERM with 5s drain (main.go:78-87). On Windows, taskkill/terminate is the practical equivalent; supervisor should attempt graceful kill then hard-kill on timeout.
- `AutoStartForwards` runs async at startup — harmless for Phase 20 but means the backend may open outbound SSH tunnels unprompted if forwards were left auto-started. Desktop UX note for Phase 26.

## 3. Supervisor Design (synthesis)

```
DesktopApp start
  → settings store ready? (config dir, JSON: {backend_path, encryption_key, theme, window_state})
  → spawn child: {backend_path} with env {WEBTERM_PORT:":0", WEBTERM_DB_PATH:<appdata>/webterm.db,
      WEBTERM_ENCRYPTION_KEY:<stored>, WEBTERM_ALLOWED_ORIGINS:"null"}
  → read stdout until BACKEND_PORT:N (timeout 10s) → base URL
  → poll GET /api/settings until 200 (timeout 10s) → READY
  → on exit of app: graceful kill (5s) → hard kill
  → on child death while running: transition UI to "backend crashed" state (SHELL-02)
```
- tokio::process::Command with `kill_on_drop(true)` as backstop; explicit stop path still required (kill_on_drop only fires on Handle drop).
- Single-instance guard: if the settings store records a live PID/port from a previous crashed run, probe it first — a healthy responding backend on the recorded port is adopted (re-attach contract from v0.3.0 survives backend reuse); a dead port is cleared. Keep it simple: adopt-or-clear, no PID file locking in this phase.
- Stale-port races are avoided by `:0` + handshake (no fixed port ever).

## 4. Desktop Settings Store

- Location: `dirs::config_dir()/webterm-desktop/settings.json` (Windows: `%APPDATA%\webterm-desktop\...`; Linux: `~/.config/webterm-desktop/...`).
- Contents (Phase 20 scope): `backend_path` (resolved default: bundled binary next to the app executable, override field for dev), `encryption_key` (hex, written 0600/user-only), `theme` (`dark|light|system`), `window_state` (x, y, width, height, maximized).
- Serialization: serde_json with graceful fallback (corrupt file → regenerate defaults, back up old file as settings.json.bak).
- SSH data stays in the backend's SQLite (established decision — the desktop store holds **UI prefs only**). The encryption key is the deliberate exception: it is a *decryption* secret for the backend's own DB, not a duplicate of SSH data.

## 5. GPUI 0.2 Integration Notes

- Entry: `gpui::Application::new().run(|cx| { ... cx.open_window(...) })` — matches the pattern documented in gpui-terminal's quick-start example (verified against gpui 0.2 docs).
- State: `cx.new(|cx| AppState)` entities; views implement `Render`; re-render via `cx.notify()`.
- gpui-component 0.6: `gpui_component::init(cx)` at startup provides theme registry + components (ActiveTheme, TitleBar, sidebar/nav primitives, dialogs). Theme switching via its theme system — verify exact 0.6 API at execution time (docs.rs/gpui-component); the plan treats exact component names as executor-verifiable, not load-bearing.
- Fonts: bundle a mono font asset for the app shell (Phase 21 terminal needs it too); gpui requires explicit font registration (`cx.text_system()`). Keep font choice consistent with web app aesthetics (JetBrains Mono as default candidate).
- Assets: gpui `Assets` impl embedding `assets/` via rust-embed is the standard pattern in Zed-derived apps.

## 6. Risks / Landmines

1. **gpui 0.2.2 breaking changes** — mitigated by exact pin + committed Cargo.lock (phase 20-01 does this first).
2. **gpui-component ↔ gpui version skew** — verify compatibility when pinning; if 0.6.0 targets a different gpui minor, align both (adjust pin, record in research addendum).
3. **Windows console subsystem** — a windowed GPUI binary on Windows must be built with `#![windows_subsystem = "windows"]` for release; but stdout capture of BACKEND_PORT requires the **child** (Go backend) console, not ours — no conflict. Dev builds keep the console for logs.
4. **0.0.0.0 bind** — backend currently binds all interfaces; documented above; Go-side fix deferred (Phase 27 QA-03 owns loopback enforcement).
5. **Zombie backends after crash** — adopt-or-clear single-instance probe (§3); restart-loop test included in plan 20-01 verification.

## Validation Architecture

**Test infrastructure:** Rust workspace uses `cargo test` per crate. The supervisor crate is the testable core (spawn/health/kill logic with no GPUI dependency — design keeps it framework-free so tests run headless in CI).

| Property | Value |
|---|---|
| Framework | cargo test (workspace), Rust 2021/2024 edition |
| Config file | none — Wave 0 installs (workspace + Cargo.lock) |
| Quick run | `cargo test -p webterm-supervisor` |
| Full suite | `cargo test --workspace` |
| Estimated runtime | ~10-60s (supervisor tests spawn real child processes) |

**Validation strategy:** every plan wave runs `cargo test --workspace` + `cargo build --workspace`; the tracer plan (20-01) additionally proves the spawn→handshake→ready loop against the real Go backend binary built from `be/` (Go toolchain available in dev environment; CI matrix compiles Go binary as a build step).

**RESEARCH COMPLETE** — proceed to planning.
