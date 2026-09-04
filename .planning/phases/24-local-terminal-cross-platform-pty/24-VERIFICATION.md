---
phase: 24-local-terminal-cross-platform-pty
verified: 2026-09-05T01:00:00Z
status: passed
score: 3/3 must-haves verified
build_verification:
  desktop_build: passed
  desktop_tests: passed
  desktop_clippy: passed
overrides_applied: 0
overrides: []
gaps: []
deferred: []
human_verification: []
---

# Phase 24: Local Terminal (Cross-Platform PTY) Verification Report

**Phase Goal:** Local terminal as a first-class New Tab option on both target platforms (Linux via POSIX PTY, Windows via ConPTY), resolving the Windows ConPTY decision explicitly, and providing automated CI smoke testing.
**Verified:** 2026-09-05
**Status:** passed
**Re-verification:** No — initial verification

## Build & Test Verification

| Check | Result |
|-------|--------|
| Cargo build (`cargo build --workspace --manifest-path desktop-gpui/Cargo.toml`) | ✓ PASSED |
| Cargo tests (`cargo test --workspace --manifest-path desktop-gpui/Cargo.toml`) | ✓ PASSED (97/97 tests passed across all 5 workspace crates) |
| Clippy checks (`cargo clippy --workspace --manifest-path desktop-gpui/Cargo.toml -- -D warnings`) | ✓ PASSED (0 warnings) |
| Go tests (`cd be && go test ./...`) | ✓ PASSED (100% Go backend test suites passed) |

## Requirement Traceability

| Requirement | Description | Status | Evidence |
|-------------|-------------|--------|----------|
| **TERM-05** | User can open a local terminal tab (shell on the desktop host), first-class in New Tab | ✓ VERIFIED | 1. **Architecture Decision:** Explicit decision record created in `CONPTY-DECISION.md` documenting Go backend ConPTY path (`github.com/UserExistsError/conpty` on Windows, `github.com/creack/pty` on Unix) vs `portable-pty`.<br>2. **New Tab Ordering:** `desktop-gpui/crates/webterm/src/views/new_tab_modal.rs` renders "💻 Local Shell" at the very top of the modal with platform PTY badge (`ConPTY` on Windows, `POSIX PTY` on Linux) and shortcut cues.<br>3. **CWD & Lifecycle:** `AppState::open_local_tab_with_cwd` supports working directory inheritance, `TerminalTab::is_local()` and `is_restartable()` support process exit and session restart.<br>4. **Automated Smoke & CI Tests:** `desktop-gpui/crates/webterm/tests/local_smoke_test.rs` tests real supervisor backend spawning, local PTY connect, echo I/O (`CONPTY_SMOKE_OK`), dynamic resize (120x40), and clean teardown on Windows and Linux.<br>5. **Go Backend Tests:** `be/internal/ssh/spawn_test.go` verifies `spawnLocalPTY` and `resizeLocalPTY` natively. |

## Success Criteria Verification

1. **Criterion 1: Local terminal opens from New Tab as a first-class option on Linux (backend PTY path):**
   - Verified: Linux uses `github.com/creack/pty` in `spawn_unix.go` invoked via `WsConnectRequest::for_local` / `for_local_with_cwd`.
   - New Tab modal prioritizes Local Shell as the primary launcher card.
2. **Criterion 2: Local terminal opens on Windows via the chosen ConPTY path, with a recorded decision:**
   - Verified: Windows uses `github.com/UserExistsError/conpty` in `spawn_windows.go`.
   - Architectural Decision Record recorded and accepted in `.planning/phases/24-local-terminal-cross-platform-pty/CONPTY-DECISION.md`.
3. **Criterion 3: A Windows CI smoke test exercises the local terminal path:**
   - Verified: `desktop-gpui/crates/webterm/tests/local_smoke_test.rs` runs against the backend fixture in CI via `.github/workflows/desktop-ci.yml`, exercising real Windows ConPTY spawning, echo execution, and PTY resize.
