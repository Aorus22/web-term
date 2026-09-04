---
phase: 20-desktop-foundation-backend-integration
plan: 01
subsystem: desktop-supervisor
tags: [rust, cargo, gpui, tokio, supervisor, process-lifecycle, integration-testing]

requires: []
provides:
  - Cargo workspace pinning gpui 0.2.2 and gpui-component 0.6.0
  - Cross-platform CI matrix workflow for Windows and Linux with Go backend fixture
  - WebTerm backend supervisor crate with port handshake, readiness probe, and lifecycle management
affects: [20-02, 20-03, 20-04, 21-terminal-engine-gpui-view]

actuals:
  tokens: 1800
  tasks: 3
  commits: 3

tech-stack:
  added: [gpui =0.2.2, gpui-component =0.6.0, tokio =1.53.1, reqwest =0.12.28, parking_lot =0.12.5, thiserror =2.0.20]
  patterns: [ephemeral port binding with stdout handshake capture, ring-buffer stderr tail capture, graceful-then-hard child termination]

key-files:
  created:
    - desktop-gpui/Cargo.toml
    - desktop-gpui/Cargo.lock
    - .github/workflows/desktop-ci.yml
    - desktop-gpui/scripts/build-test-backend.cmd
    - desktop-gpui/scripts/build-test-backend.sh
    - desktop-gpui/crates/supervisor/src/lib.rs
    - desktop-gpui/crates/supervisor/tests/integration.rs
  modified:
    - desktop-gpui/crates/supervisor/Cargo.toml

key-decisions:
  - "Pinned gpui exactly to 0.2.2 and gpui-component to 0.6.0 to prevent breaking upstream churn"
  - "Secrets (encryption key) passed exclusively via environment variables, never command-line arguments"
  - "Supervisor captures ephemeral port via BACKEND_PORT:<port> stdout line and probes /api/settings for readiness"

patterns-established:
  - "Child process supervision: spawn with env secrets, async stdout handshake, stderr ring buffer, readiness poll"

requirements-completed: [SHELL-01]

coverage:
  - id: D1
    description: "Desktop cargo workspace scaffold with exact pins"
    requirement: SHELL-01
    verification:
      - kind: automated
        ref: "cargo build --workspace"
        status: pass
  - id: D2
    description: "CI matrix covering windows-latest and ubuntu-latest with Go backend build"
    requirement: SHELL-01
    verification:
      - kind: automated
        ref: ".github/workflows/desktop-ci.yml"
        status: pass
  - id: D3
    description: "Supervisor spawn, handshake, readiness, and lifecycle integration tests"
    requirement: SHELL-01
    verification:
      - kind: integration
        ref: "desktop-gpui/crates/supervisor/tests/integration.rs"
        status: pass
---

# Plan 20-01 Summary: Desktop Cargo Workspace, CI Matrix & Backend Supervisor

Established the foundation for the desktop GPUI client:
1. Created the Cargo workspace with exact pins (GPUI 0.2.2, gpui-component 0.6.0, tokio 1.53, reqwest 0.12 with rustls-tls).
2. Set up GitHub Actions CI matrix for Windows and Ubuntu runners compiling the Go backend test fixture.
3. Implemented the headless `webterm-supervisor` crate with ephemeral port capture, readiness polling, lifecycle controls, and verified against the real Go backend fixture with 4 green integration tests.
