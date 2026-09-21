# Patches

Background reading: [`docs/liquid-glass.md`](../docs/liquid-glass.md) covers the
material, its settings and how to verify the blur end to end.

## `gpui-pre-linux-background-effect.patch`

Makes gpui's Wayland backend bind [`ext-background-effect-v1`], the
cross-compositor protocol for backdrop blur, so `WindowBackgroundAppearance::Blurred`
actually frosts the window on **KWin 6.7+, Mutter 51+ (GNOME 51) and Hyprland**.

Why it has to live inside gpui: a blur is painted by the compositor, which is the
only party allowed to read the pixels behind a surface, and the request has to go
out over the window's own Wayland connection. gpui never hands out that
`wl_surface`, so no application code — in `webterm` or anywhere else — can ask for
it from the outside.

Why it is small: `wayland-protocols` (already a dependency, and gpui already
enables its `staging` feature) generates the client bindings; the patch only binds
the manager and requests a blur region. Ten hunks, +58/−2, two files:

- `src/linux/wayland/client.rs` — bind `ext_background_effect_manager_v1`,
  register the two new objects.
- `src/linux/wayland/window.rs` — hold one `ext_background_effect_surface_v1` per
  window and, in `update_window()`, hand it a blur region built surface-locally
  from the surface size inset by `state.inset()`. Surface-local rather than the
  already-computed opaque rectangle, because that one still carries the window's
  requested position until the first configure. KDE's older `org_kde_kwin_blur`
  stays as the fallback for compositors that do not speak the standard protocol
  yet.

Known cosmetic gap: `state.inset()` is 0 for this app, which draws its own
rounded frame inside the surface, so the blur region covers the whole surface and
a frosted halo can appear just outside the corners. Closing that needs gpui to
know the frame's padding. See `docs/liquid-glass.md`.

## Applying it

```sh
cd desktop-gpui
./scripts/patch-gpui.sh          # write vendor/gpui-pre-linux + .cargo/config.toml
./scripts/patch-gpui.sh --check  # dry run: does the patch still apply?
```

Both `vendor/` and the generated `.cargo/config.toml` are gitignored. A checkout
that never runs the script still builds; it just gets stock gpui, where the glass
backdrop is a translucent tint instead of a blur.

One side effect worth knowing: the first build from the repository root rewrites
`desktop-gpui/Cargo.lock` so the patched path is recorded instead of the registry
crate. `git checkout -- desktop-gpui/Cargo.lock` before committing, or remove
`.cargo/config.toml` to go back to stock gpui (the lock returns to its committed
form on the next build).

## Following a gpui bump

1. Bump the `gpui-pre` pin in `desktop-gpui/Cargo.toml` and run `cargo update`.
2. `./scripts/patch-gpui.sh` — it reads the new version from `Cargo.lock`,
   re-fetches the crate and re-applies the patch.
3. If a hunk drifted, the script names the anchors it could not find and leaves
   `vendor/` alone. Fix the two files by hand in `vendor/gpui-pre-linux/`, then
   refresh the patch against a pristine copy of the same version:

   ```sh
   curl -fsSL https://static.crates.io/crates/gpui-pre-linux/gpui-pre-linux-<v>.crate \
     | tar xz -C /tmp
   diff -ruN /tmp/gpui-pre-linux-<v>/src vendor/gpui-pre-linux/src \
     | sed 's|^--- /tmp/gpui-pre-linux-<v>/|--- a/|; s|^+++ vendor/gpui-pre-linux/|+++ b/|' \
     > patches/gpui-pre-linux-background-effect.patch
   ```

4. Once upstream binds the protocol itself, delete the patch, the script, the
   generated config and the `vendor` entry from `.gitignore`.

[`ext-background-effect-v1`]: https://wayland.app/protocols/ext-background-effect-v1
