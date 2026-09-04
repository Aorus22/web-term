---
phase: 20-desktop-foundation-backend-integration
plan: 02
subsystem: desktop-settings-client
tags: [rust, settings, encryption-key, rest-client, reqwest, serde]

requires:
  - phase: 20-01
    provides: Cargo workspace, exact dependency pins, supervisor crate
provides:
  - Desktop settings store with corruption recovery and persistent encryption key custody
  - Typed backend REST client with DTO mirroring and readiness probe
affects: [20-03, 20-04, 21, 22, 23]

actuals:
  tokens: 1500
  tasks: 2
  commits: 2

tech-stack:
  added: [dirs =6.0.0, reqwest =0.12.28, serde_json =1.0.151]
  patterns: [settings store corrupt-backup and regenerate, stable encryption key custody, in-process TCP stub testing]

key-files:
  created:
    - desktop-gpui/crates/settings/src/paths.rs
    - desktop-gpui/crates/settings/tests/store_test.rs
    - desktop-gpui/crates/backend-client/src/rest.rs
    - desktop-gpui/crates/backend-client/src/types.rs
  modified:
    - desktop-gpui/crates/settings/src/lib.rs
    - desktop-gpui/crates/backend-client/src/lib.rs

key-decisions:
  - "Desktop settings holds UI preferences only; SQLite credentials decryption key is the sole secret managed in settings.json"
  - "Corrupt settings.json files are backed up to settings.json.bak rather than silently discarded"
  - "Stable 64-character lowercase hex encryption key generated once and reused across sessions"

patterns-established:
  - "Injectable base path pattern for test isolation without polluting user app data"
  - "In-process tokio TcpListener stub for HTTP client integration testing"

requirements-completed: [SHELL-01]

coverage:
  - id: D1
    description: "DesktopSettings persistence, corruption recovery, and encryption key stability"
    requirement: SHELL-01
    verification:
      - kind: unit
        ref: "desktop-gpui/crates/settings/tests/store_test.rs"
        status: pass
  - id: D2
    description: "BackendClient REST methods and readiness probe"
    requirement: SHELL-01
    verification:
      - kind: unit
        ref: "desktop-gpui/crates/backend-client/src/lib.rs"
        status: pass
---

# Plan 20-02 Summary: Settings Store & Typed Backend Client

Implemented the two headless support crates:
1. `webterm-settings`: Implemented `DesktopSettings` and `paths.rs` supporting JSON persistence, corrupt file recovery (backup to `settings.json.bak`), strict file permissions, and stable custody of the 64-hex-character `WEBTERM_ENCRYPTION_KEY`.
2. `webterm-backend-client`: Implemented `types.rs` mirroring Go models (`Settings`, `Connection`) and `rest.rs` with `get_settings`, `list_connections`, and `is_ready` probes. Tested with in-process mock HTTP servers.
