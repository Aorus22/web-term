# desktop-gpui

The GPUI desktop app (`webterm`): SSH terminal with a native client, a Go backend
supervised as a child process, and a hand-drawn window frame instead of server-side
decorations.

## Layout

| Path | What |
| --- | --- |
| `crates/webterm` | the app: views, app state, window setup, `glass.rs` (the Liquid Glass material) |
| `crates/terminal` | terminal emulation and rendering |
| `crates/backend-client` | HTTP/WebSocket client for the Go backend |
| `crates/supervisor` | spawns and watches that backend |
| `crates/settings` | the settings store (`~/.config/webterm-desktop/settings.json`) |
| `patches/`, `vendor/` | the local gpui patch that makes the window backdrop blur (see below) |
| `docs/` | operator-facing notes |

## Build and run

```sh
# from the repository root: cargo discovers `.cargo/config.toml` by walking up
# from the working directory, and that file is what routes gpui to the patched
# copy — the one that can actually blur the window backdrop
cargo build --manifest-path desktop-gpui/Cargo.toml -p webterm
cargo test  --manifest-path desktop-gpui/Cargo.toml --workspace
```

Without `.cargo/config.toml` (i.e. before running `scripts/patch-gpui.sh`) the
same commands build stock gpui and the glass backdrop stays a translucent tint.
Nothing else changes.

The Go backend is looked up as `backend` / `webterm-backend` next to the running
executable, then as `test-support/backend*` relative to the working directory — so
with the fixture built, a plain `cargo run` needs no further setup.

## Scripts

| Script | Purpose |
| --- | --- |
| `scripts/patch-gpui.sh` | vendor + patch `gpui-pre-linux` so `Blurred` reaches KWin 6.7+/GNOME 51+/Hyprland; `--check` is a dry run |
| `scripts/build-appimage.sh` | Go backend + release binary + AppImage; `--install` drops it in `~/Applications` |
| `scripts/package-linux.sh`, `scripts/package-windows.*` | platform packaging |
| `scripts/build-test-backend.*` | build the Go backend as a test fixture |

## Docs

- [`docs/liquid-glass.md`](docs/liquid-glass.md) — the Liquid Glass material: what
  is translucent, the three settings, why window blur needs a compositor protocol,
  the gpui patch, and how to verify the frost under a nested KWin.
- [`patches/README.md`](patches/README.md) — the patch itself and how to refresh it
  after a gpui bump.
