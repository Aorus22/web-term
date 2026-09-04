---
phase: 20-desktop-foundation-backend-integration
plan: 03
subsystem: desktop-ui-shell
tags: [rust, gpui, gpui-component, desktop-shell, theme, navigation, lifecycle-ui]

requires:
  - phase: 20-01
    provides: Supervisor crate, Cargo workspace
  - phase: 20-02
    provides: Settings store, BackendClient
provides:
  - GPUI desktop window with supervisor-driven application lifecycle
  - Status view (Starting with spinner, Failed with redacted stderr tail, Retry/Quit buttons)
  - Navigation shell with Hosts, SSH Keys, SFTP, and Settings entries
  - Persisted Dark/Light theme toggle
  - Bundled JetBrains Mono font asset
affects: [20-04, 21-terminal-engine-gpui-view, 22, 23, 24, 25, 26]

actuals:
  tokens: 2200
  tasks: 4
  commits: 3

tech-stack:
  added: [gpui =0.3.3 (via gpui-pre), gpui-platform =0.3.3, gpui-component =0.6.0]
  patterns: [lifecycle view routing (Starting/Failed/Ready), secret-redacted stderr tail display, reactive theme toggle with persistent settings]

key-files:
  created:
    - desktop-gpui/crates/webterm/Cargo.toml
    - desktop-gpui/crates/webterm/src/main.rs
    - desktop-gpui/crates/webterm/src/app_state.rs
    - desktop-gpui/crates/webterm/src/theme.rs
    - desktop-gpui/crates/webterm/src/views/mod.rs
    - desktop-gpui/crates/webterm/src/views/status.rs
    - desktop-gpui/crates/webterm/src/views/nav.rs
    - desktop-gpui/crates/webterm/assets/fonts/JetBrainsMono-Regular.ttf

key-decisions:
  - "Aligned gpui pin to gpui-pre 0.3.3 to guarantee binary compatibility with gpui-component 0.6.0 (documented version deviation)"
  - "Secret redaction on Failed status page masks both encryption key prefix and environment variable assignments"
  - "Navigation shell routes between 4 core views with placeholder banners mapping each area to its implementing milestone phase"

patterns-established:
  - "GPUI AppState entity pattern subscribing to tokio supervisor status channel"
  - "Safe secret redaction for UI error diagnostics"

requirements-completed: [SHELL-01, SHELL-02, SHELL-03, SHELL-05]

coverage:
  - id: D1
    description: "GPUI window entry with supervisor-driven startup lifecycle"
    requirement: SHELL-01
    verification:
      - kind: automated
        ref: "cargo build -p webterm"
        status: pass
  - id: D2
    description: "Honest error reporting with stderr tail redaction and Retry/Quit controls"
    requirement: SHELL-02
    verification:
      - kind: unit
        ref: "desktop-gpui/crates/webterm/src/views/status.rs"
        status: pass
  - id: D3
    description: "Theme toggle persisted to settings store"
    requirement: SHELL-03
    verification:
      - kind: unit
        ref: "desktop-gpui/crates/webterm/src/theme.rs"
        status: pass
  - id: D4
    description: "Four-entry navigation shell with placeholder views and font registration"
    requirement: SHELL-05
    verification:
      - kind: automated
        ref: "desktop-gpui/crates/webterm/src/views/nav.rs"
        status: pass
---

# Plan 20-03 Summary: GPUI Desktop Tracer Shell & Lifecycle UI

Implemented the desktop UI tracer slice:
1. `webterm` application binary with GPUI window initialization, JetBrains Mono font loading, and supervisor integration.
2. Status view supporting `Starting`, honest `Failed` panel (with key-material redaction, failure reason, and Retry/Quit controls), and transition to the navigation shell on `Ready`.
3. Navigation shell with sidebar entries for Hosts, SSH Keys, SFTP, and Settings, plus Dark/Light theme switching persisted to the settings store.
