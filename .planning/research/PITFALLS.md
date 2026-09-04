# Pitfalls Research

**Domain:** Native desktop SSH client (GPUI frontend over existing Go backend)
**Researched:** 2026-09-04
**Confidence:** MEDIUM (mix of documented crate gaps, platform quirks, and pattern-level inference)
**Note:** Produced inline by the orchestrator (generic inline workaround — no subagent runtime in this session).

## Critical Pitfalls

### Pitfall 1: gpui-terminal feature gaps (scrollback, mouse selection)

**What goes wrong:**
The convenient `gpui-terminal` crate lists scrollback as "planned" and mouse selection as "partial/planned" in its feature matrix. Building the milestone on it and discovering this at parity-testing time means the terminal is unusable for real work (copy/paste is table stakes).

**Why it happens:**
Young crate, published recently; docs advertise the architecture but the matrix exposes gaps. Teams assume "complete terminal emulator" includes selection/scrollback.

**How to avoid:**
Timebox a spike in the first terminal phase: verify selection + scrollback behavior against real vim/htop usage. If gaps block parity, switch to the vendored Zed pattern (port the `terminal` crate's state handling + rendering approach) — a known-quantity port, not new research.

**Warning signs:**
"Planned" in the feature matrix; examples avoid selection demos; issues mention copy/paste workarounds.

**Phase to address:**
First terminal-rendering phase (make the spike a plan-level deliverable with go/no-go criteria).

---

### Pitfall 2: Pre-1.0 dependency churn (gpui, gpui-component, alacritty_terminal)

**What goes wrong:**
`cargo update` or an added dependency bumps gpui across a breaking minor; the build breaks weeks later with no local change. alacritty_terminal is 0.x — minor = breaking. Wasted days debugging "nothing changed" failures.

**Why it happens:**
All three core deps are pre-1.0 and actively developed; Rust's default encouraged flow (loose requirements + Cargo.lock) hides breakage until an update.

**How to avoid:**
Pin exact/minor versions in Cargo.toml; commit Cargo.lock; do dependency upgrades as deliberate, isolated PRs with a build+smoke-test checklist. Check gpui-component ↔ gpui pin compatibility on every bump.

**Warning signs:**
Divergent builds between machines; CI green but local red after an unrelated change; "works on the lockfile from last month".

**Phase to address:**
Project foundation phase (pins + lockfile + CI from day one).

---

### Pitfall 3: Backend process lifecycle bugs (zombies, port races, orphaned children)

**What goes wrong:**
The spawned Go backend outlives a crashed desktop app (orphan), two backends race for the same port after rapid restart, or a "restart backend" flow kills the wrong PID — leaving users with ghost sessions and confusing connection errors.

**Why it happens:**
Child-process supervision has boring-but-critical edge cases: kill_on_drop vs explicit termination, SIGTERM equivalents on Windows (taskkill semantics), health-poll timeouts vs infinite retries.

**How to avoid:**
One supervisor code path (kill_on_drop + explicit graceful stop); bind the backend to an ephemeral port chosen by the parent and pass it via flag; health-poll with a hard timeout; on startup, detect and adopt-or-kill stale instances (single-instance design or PID handshake). Test crash/restart loops explicitly.

**Warning signs:**
"Address already in use" after crashes; multiple backend processes in Task Manager; sessions that "come back" after app exit.

**Phase to address:**
Backend integration phase (supervisor is small enough to get right early; write restart-loop tests then).

---

### Pitfall 4: Local terminal on Windows (ConPTY gap)

**What goes wrong:**
The backend's local-terminal feature uses creack/pty, which is POSIX-only. On Windows the "local terminal" tab either fails or is silently dropped — breaking full-parity on one of the two target platforms.

**Why it happens:**
Web app deployments were POSIX servers; desktop targets Windows for the first time.

**How to avoid:**
Decide explicitly in the local-terminal phase: (a) add a ConPTY backend to the Go local-terminal endpoint (golang.org/x/sys/windows or an existing ConPTY wrapper), or (b) bypass the backend for local-terminal on Windows and feed portable-pty (ConPTY under the hood) I/O through the same WS-shaped pipeline. Linux always uses the backend path.

**Warning signs:**
Local terminal tested only on Linux during dev; feature matrix assumption "local works everywhere" never verified on Windows CI.

**Phase to address:**
Local terminal phase (explicit go/no-go on ConPTY approach with a Windows CI smoke test).

---

### Pitfall 5: Parity drift — "desktop done" that silently drops web features

**What goes wrong:**
Milestone ships with 90% parity: keyboard shortcuts subtly different, theme sync missing in SFTP, quick-connect absent, import/export forgotten. Users (you) keep opening the web UI, and the desktop app fails its own "daily-driveable" bar.

**Why it happens:**
Parity is a checklist of 22 validated requirements spread across 4 milestones; nobody holds the full list in their head while building greenfield UI.

**How to avoid:**
Treat PROJECT.md's validated list as the acceptance matrix; map every REQ to a desktop phase (the roadmap does exactly this); run an explicit parity-audit pass as a late phase before calling the milestone done.

**Warning signs:**
Phases completing without a mapping to a specific REQ-ID; "we'll get to shortcuts later" patterns.

**Phase to address:**
Every phase (mapping enforced by roadmap) + dedicated parity-audit phase at the end.

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Skipping the backend-client crate, scattering reqwest calls in views | Faster first screens | Protocol drift, untestable UI code | Never — the crate is cheap to start with |
| Rendering terminal with naive per-cell layout (no glyph-run caching) | Quicker spike | Slow on heavy output, rewrite later | Spike phase only |
| Hardcoded backend port | Faster setup | Port collisions, breaks multi-instance dev | Never |
| Unversioned DTO mirroring (hand-rolled structs without tests) | Faster client work | Silent breakage on backend changes | Prototype only; add contract tests before SFTP phase |
| Single-crate monolith desktop app | Simpler start | gpui churn infects all code; hard to swap terminal impl | Never for this milestone's risk profile |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| Go backend spawn | Assuming a fixed port is free | Parent picks ephemeral port, passes via flag; backend must honor it |
| WS terminal protocol | Assuming web UI's framing is "obvious" — reimplementing it wrong | Read the backend's WS handler and mirror message types exactly; contract-test roundtrip |
| Re-attach | Treating app restart same as WS drop | Two distinct paths: WS reconnect vs backend-process restart (sessions survive the first, not always the second) |
| Clipboard (OSC 52) | Only wiring OSC 52, forgetting local Ctrl+C/V semantics | Support both: app-level clipboard ops and OSC 52 callback via arboard |
| Theme sync | Terminal palette not following app theme (bug fixed in web v0.4!) | Single theme source in app state feeding both gpui-component theme and terminal ColorPalette |
| Windows paths | Unix-style path handling in SFTP panes | Path abstraction per pane-source (local Windows vs remote POSIX) |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Repainting whole window per terminal byte | Fan spins, UI stutters under output | Push-based notify + coalesced repaint (gpui-terminal's flume pattern) | Immediately under `yes`/cat large file |
| Per-cell text shaping | Laggy scrolling, slow vim | Glyph-run caching, batch identical-style cells | Visible at full-screen redraws |
| Unbounded WS receive buffer | Memory climb on fast output | Bounded channel with backpressure/coalescing | Heavy build logs |
| Blocking UI thread on reqwest | Frozen panes during SFTP listing | All backend I/O async from the start | First slow network op |

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Logging backend traffic incl. terminal bytes | Secrets in plaintext log files | tracing filters exclude WS payloads; log metadata only |
| Desktop settings file holding connection copies | Divergence from encrypted backend store | Settings hold UI prefs only (rule already in STACK.md) |
| Binding backend to 0.0.0.0 in desktop mode | LAN-exposed SSH proxy on user machines | Loopback bind enforced by supervisor flags |
| Passing secrets via command-line args | Visible in process listings | Env vars or config file for anything sensitive |

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Web shortcuts that fight OS conventions | Ctrl+W closes window (browser muscle memory inverted) | Map desktop shortcuts to native conventions; keep web parity where it doesn't conflict |
| Missing window-state persistence | Resize/reposition every launch | Persist bounds + maximized state in desktop settings |
| Backend "starting…" with no progress | App feels hung on cold start | Health-gate with visible startup state + failure diagnostics |
| Font defaults that render nerd-font glyphs wrong | Broken prompt rendering | line_height_multiplier + document recommended mono fonts (gpui-terminal guidance) |

## "Looks Done But Isn't" Checklist

- [ ] **Terminal:** Often missing scrollback + selection under real use — verify vim + copy from `htop` output
- [ ] **Shortcuts:** Often missing tab cycling + close — verify Ctrl+Tab/Ctrl+W behavior matrix
- [ ] **Reconnection:** Often missing the *backend-restart* case (vs WS drop) — kill the child process mid-session and verify
- [ ] **SFTP:** Often missing drag-and-drop from OS file manager — verify external DnD on Windows Explorer and Linux file managers
- [ ] **Theme:** Often missing terminal palette sync with app theme — toggle dark/light mid-session
- [ ] **Import/export:** Often forgotten entirely — roundtrip a connection JSON from the web app

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| gpui-terminal gaps discovered late | MEDIUM | Switch to vendored Zed pattern; terminal crate boundary keeps blast radius contained |
| gpui breaking change mid-milestone | LOW-MEDIUM | Pin rollback; upgrade in isolated PR with checklist |
| Orphaned backend processes reported | LOW | Supervisor handshake + stale-instance detection patch |
| Parity gap found at audit | MEDIUM | Parity matrix makes gaps explicit; add remediation phase before ship |

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| gpui-terminal gaps | First terminal phase (spike w/ go/no-go) | Selection + scrollback demo against vim/htop |
| Dependency churn | Foundation phase | Pinned manifest + committed lockfile in CI |
| Process lifecycle | Backend integration phase | Restart-loop + kill-app-mid-session tests |
| Windows ConPTY | Local terminal phase | Windows CI smoke test of local tab |
| Parity drift | Roadmap mapping + final audit phase | Every REQ mapped; audit checklist executed |

## Sources

- docs.rs/gpui-terminal feature matrix (scrollback "Planned", mouse "Partial") (HIGH)
- docs.rs/crate/gpui — official pre-1.0 breaking-changes warning (HIGH)
- crates.io alacritty_terminal — 0.x semver semantics (HIGH)
- v2.tauri.app sidecar docs + plugins-workspace#3062 — recognized sidecar lifecycle pain incl. port-free checks (MEDIUM)
- creack/pty POSIX-only scope + portable-pty ConPTY support (HIGH for library scope)
- WebTerm STATE.md accumulated decisions (theme sync history; session re-attach design) (HIGH, internal)

---
*Pitfalls research for: WebTerm Desktop (GPUI client over Go backend)*
*Researched: 2026-09-04*
