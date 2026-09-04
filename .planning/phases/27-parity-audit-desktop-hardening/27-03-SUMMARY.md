# Summary 27-03: Packaging, Launchable Bundle & Loopback Enforcement

## Frontmatter
- **Phase:** 27-parity-audit-desktop-hardening
- **Plan:** 27-03
- **Requirement:** QA-03 (App ships as a single launchable bundle with the backend bound to loopback only)
- **Status:** Complete

## Overview
Implemented strict loopback binding for the backend supervisor, portable bundle binary discovery, and automated distribution packaging scripts:
1. **Loopback Enforcement (`127.0.0.1`):**
   - Updated Go backend `be/internal/config/config.go` with `Host` field loaded from `WEBTERM_HOST` (default `0.0.0.0` for web server backward compatibility).
   - In `be/cmd/server/main.go`, bound TCP listener using `net.JoinHostPort(cfg.Host, portStr)`.
   - Updated `webterm_supervisor::Supervisor` in `desktop-gpui/crates/supervisor/src/lib.rs` to inject `WEBTERM_HOST=127.0.0.1` in `SpawnOptions::env_vars()`.
   - Created integration test `desktop-gpui/crates/supervisor/tests/loopback_test.rs` validating that supervisor enforces `WEBTERM_HOST=127.0.0.1` and the spawned backend listener only accepts loopback traffic.
2. **Packaged Bundle Executable Resolution:**
   - Created `desktop-gpui/crates/webterm/src/bundle.rs` providing `resolve_backend_path` and `resolve_backend_path_with_current_exe`.
   - Checks `DesktopSettings::backend_path` override first, then looks adjacent to `current_exe()` (e.g. `dist/webterm-windows-x64/backend.exe` side-by-side with `webterm.exe`), and falls back to development paths.
   - Added unit test `desktop-gpui/crates/webterm/tests/bundle_resolution_test.rs` (2/2 passing).
3. **Distribution Packaging Scripts:**
   - `desktop-gpui/scripts/package-windows.cmd` and `package-windows.ps1`: Builds stripped Go backend (`backend.exe`), compiles release GPUI client (`webterm.exe`), copies assets, and outputs portable bundle to `dist/webterm-windows-x64/`.
   - `desktop-gpui/scripts/package-linux.sh`: Builds stripped Go backend (`backend`), compiles release GPUI client (`webterm`), copies assets, and outputs portable bundle to `dist/webterm-linux-x64/`.

## Verification
- `go test ./...` in `be/` passed (all packages).
- `cargo test --manifest-path desktop-gpui/Cargo.toml --package webterm-supervisor --test loopback_test` passed (1/1 test).
- `cargo test --manifest-path desktop-gpui/Cargo.toml --package webterm --test bundle_resolution_test` passed (2/2 tests).
- Full desktop workspace passed (126/126 tests).
- Zero warnings under `cargo clippy --workspace --manifest-path desktop-gpui/Cargo.toml -- -D warnings`.
