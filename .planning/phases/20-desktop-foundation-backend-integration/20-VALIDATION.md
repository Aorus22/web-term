---
phase: "20"
slug: "desktop-foundation-backend-integration"
status: draft
nyquist_compliant: false
wave_0_complete: false
created: "2026-09-04"
---

# Phase 20 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust workspace, edition 2021+) |
| **Config file** | none — Wave 0 installs (workspace `Cargo.toml` + committed `Cargo.lock`) |
| **Quick run command** | `cargo test --workspace --manifest-path desktop-gpui/Cargo.toml` |
| **Full suite command** | `cargo test --workspace --manifest-path desktop-gpui/Cargo.toml` + `cargo build --workspace` |
| **Estimated runtime** | ~10-60s (supervisor integration tests spawn real child processes) |

**External prerequisite:** the real Go backend binary for integration tests — built via `go build -o desktop-gpui/test-support/backend(.exe) ./be/cmd/server` (task 20-01-02); tests fail loud (not skip) when it is absent.

---

## Sampling Rate

- **After every task commit:** `cargo test -p <touched-crate>` (≤15s)
- **After every plan wave:** `cargo test --workspace`
- **Before `$gsd-verify-work`:** full suite green + `cargo build --workspace`
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 20-01-01 | 01 | 1 | SHELL-01 | — | N/A | build | `cargo build --workspace` (desktop-gpui) | ❌ W0 | ⬜ pending |
| 20-01-02 | 01 | 1 | SHELL-01 | — | N/A | config | workflow YAML + fixture scripts | ❌ W0 | ⬜ pending |
| 20-01-03 | 01 | 1 | SHELL-01 | T-20-02 | secrets via env only, never argv; stderr tail capped | integration | `cargo test -p webterm-supervisor` | ❌ W0 | ⬜ pending |
| 20-02-01 | 02 | 2 | SHELL-01 | T-20-01 | key custody: user-only perms, never logged | unit | `cargo test -p webterm-settings` | ❌ W0 | ⬜ pending |
| 20-02-02 | 02 | 2 | SHELL-01 | — | N/A | unit | `cargo test -p webterm-backend-client` | ❌ W0 | ⬜ pending |
| 20-03-01 | 03 | 3 | SHELL-01 | — | N/A | build | `cargo build -p webterm` | ❌ W0 | ⬜ pending |
| 20-03-02 | 03 | 3 | SHELL-02 | T-20-02 | stderr tail redacted before display | build+clippy | `cargo clippy -p webterm -- -D warnings` | ❌ W0 | ⬜ pending |
| 20-03-03 | 03 | 3 | SHELL-03, SHELL-05 | — | N/A | build | `cargo build -p webterm` | ❌ W0 | ⬜ pending |
| 20-03-04 | 03 | 3 | SHELL-01/02/05 | — | N/A | manual (checkpoint) | visual — lifecycle, failure state, theme persist | ❌ | ⬜ pending |
| 20-04-01 | 04 | 4 | SHELL-06 | — | N/A | build | `cargo build -p webterm` | ❌ W0 | ⬜ pending |
| 20-04-02 | 04 | 4 | SHELL-06 | — | N/A | build+clippy | `cargo clippy -p webterm -- -D warnings` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `desktop-gpui/Cargo.toml` workspace + committed `Cargo.lock` (task 20-01-01)
- [ ] `desktop-gpui/test-support/` gitignored; backend binary built there (task 20-01-01/02)
- [ ] Go toolchain available (already a project prerequisite — `compile.sh`/`compile.bat` exist)

*No test framework install needed — cargo test is the framework.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Window opens, shows Starting→Ready | SHELL-01 | GPUI rendering is visual | `cargo run -p webterm`; observe status page |
| Failure state shows clear error + stderr tail | SHELL-02 | Requires seeing the UI | Temporarily set backend_path to a nonexistent file; observe Failed state |
| Theme persists across restart | SHELL-05 | Visual + restart cycle | Toggle theme, quit, relaunch, confirm |
| Window bounds persist across restart | SHELL-06 | Visual + restart cycle | Move/resize window, quit, relaunch, confirm |
| Backend process terminates on app close | SHELL-01 | OS process inspection | Close app; check Task Manager / `ps` for orphaned backend |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter (set by validate-phase)

**Approval:** pending
