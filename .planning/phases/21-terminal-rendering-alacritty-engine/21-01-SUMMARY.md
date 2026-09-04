---
phase: 21-terminal-rendering-alacritty-engine
plan: 01
subsystem: terminal-engine
tags: [rust, alacritty_terminal, gpui, terminal-colors, spike-verdict]

requires:
  - phase: 20-01
    provides: Cargo workspace and CI configuration
provides:
  - Formal spike verdict on gpui-terminal (NO-GO for external crate, GO for in-tree crate)
  - webterm-terminal workspace crate scaffold with exact dependencies (alacritty_terminal 0.25.1, flume 0.12.0)
  - ColorPalette system mapping 16 ANSI, 256 indexed, and 24-bit TrueColor to GPUI colors
affects: [21-02, 21-03, 22-ssh-terminal-sessions-tabs]

actuals:
  tokens: 1650
  tasks: 2
  commits: 2

tech-stack:
  added: [alacritty_terminal =0.25.1, flume =0.12.0, anyhow =1.0.104]
  patterns: [in-tree terminal crate boundary, 256-color cube and grayscale interpolation, TrueColor RGB to GPUI Hsla conversion]

key-files:
  created:
    - .planning/phases/21-terminal-rendering-alacritty-engine/SPIKE-gpui-terminal.md
    - desktop-gpui/crates/terminal/Cargo.toml
    - desktop-gpui/crates/terminal/src/lib.rs
    - desktop-gpui/crates/terminal/src/colors.rs
  modified:
    - desktop-gpui/Cargo.toml
    - desktop-gpui/Cargo.lock

key-decisions:
  - "Decided NO-GO on external gpui-terminal = 0.1.0 crate due to fatal dual-GPUI version conflict (gpui 0.2.2 vs gpui-pre 0.3.3)"
  - "Decided GO for in-tree webterm-terminal crate adopting gpui-terminal's clean architecture bound directly to gpui-pre 0.3.3"
  - "ColorPalette precomputes 256-color table and provides dark_default / light_default themes matching WebTerm UI"

patterns-established:
  - "Terminal color resolution from Alacritty Color enum (Named, Spec, Indexed) to GPUI Hsla"

requirements-completed: [TERM-01]

coverage:
  - id: T1
    description: "Spike report recorded with explicit go/no-go verdict"
    requirement: TERM-01
    verification:
      - kind: automated
        ref: "SPIKE-gpui-terminal.md"
        status: pass
  - id: T2
    description: "webterm-terminal crate scaffold compiles and passes unit tests"
    requirement: TERM-01
    verification:
      - kind: automated
        ref: "cargo test -p webterm-terminal"
        status: pass
---

# Plan 21-01: gpui-terminal Spike Report & Crate Scaffolding Summary

Plan 21-01 evaluated the `gpui-terminal` crate, resolved Criterion 5 with an explicit spike report, and scaffolded the in-tree `webterm-terminal` crate.

## Deliverables

1. **Spike Report (`SPIKE-gpui-terminal.md`)**:
   - Analyzed `gpui-terminal = "0.1.0"` from crates.io.
   - Identified fatal version incompatibility: `gpui-terminal` hard-pins `gpui = "0.2.2"`, conflicting with our workspace pin `gpui-pre = "=0.3.3"` (which is strictly required by `gpui-component = "=0.6.0"`).
   - Documented why Cargo cannot reconcile these two versions (distinct `Entity`, `Window`, `App` types and duplicate platform runtime initialization).
   - Formally recorded **NO-GO** for external crate dependency and **GO** for in-tree crate vendoring.

2. **Crate Scaffold (`desktop-gpui/crates/terminal`)**:
   - Added `crates/terminal` to workspace members and pinned `alacritty_terminal = "=0.25.1"`, `flume = "=0.12.0"`, and `anyhow = "=1.0.104"`.
   - Scaffolded `webterm-terminal` crate.

3. **Color Palette System (`colors.rs`)**:
   - Implemented `ColorPalette` with full support for 16 ANSI colors, 256 indexed colors (6x6x6 color cube + 24 grayscale steps), and 24-bit TrueColor RGB.
   - Implemented `dark_default()` and `light_default()` presets aligned with WebTerm UI themes.
   - All 3 unit tests passed cleanly (`cargo test -p webterm-terminal`).
