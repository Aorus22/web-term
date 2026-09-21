# Liquid Glass

The window chrome is painted as a translucent slab so the desktop reads through
it, and — where the compositor can blur — the window itself is frosted behind
that slab. This document is the operator's view: what changes, which knobs exist,
how the blur is obtained, how to check it, and what to do when it breaks.

Code: `crates/webterm/src/glass.rs` (all the material maths), the
`glass_enabled` / `glass_opacity` / `glass_backdrop` fields in
`crates/settings`, and the `WINDOW` section of the Settings page.
Blur plumbing: `patches/` + `scripts/patch-gpui.sh`.

## What is glass and what is not

| Surface | Tier | Behind it |
| --- | --- | --- |
| Sidebar, tab strip, right sheet | `Chrome` | the desktop (translucent, 0.78 × the setting) |
| Dialogs, modals, toasts, context menus, dropdowns, palette shell | `Overlay` | the app's own content (the setting itself, near-opaque) |
| Terminal pane, content panes, host/key cards, SFTP rows, transfer drawer | none | deliberately opaque — text over a moving desktop is unreadable |

Two alphas rather than one, because what shows through differs. Chrome can afford
transparency because a wallpaper is high-contrast noise the eye ignores; an
overlay sits over terminal text and stays close to opaque, with the scrim that is
already in the view supplying the separation.

Each glass leaf gets three things: a translucent fill, a hairline border whose
ink comes from the theme's *polarity* (white on dark, black on light) instead of
the opaque `border` token, and an inset rim — 1px of light on the top edge, 1px
of shade on the bottom — which is what makes the slab read as a material rather
than as a faded panel.

## Settings

Settings → **WINDOW**:

- **Liquid Glass** switch — the whole feature. Off restores the previous pixels
  exactly (same opaque fills, same plain elevation stack, no rim, no polarity
  border), which is what the tests assert.
- **Intensity** — `Subtle` 0.94, `Medium` 0.85, `Bold` 0.68. The number is the
  *overlay* alpha; chrome is derived from it.
- **Backdrop** — what the window asks the compositor to paint behind the glass:
  `Frost` or `Translucent`.

All three persist in `~/.config/webterm-desktop/settings.json`. The fields carry
serde defaults, so a settings file written before the material existed keeps
loading and simply gains the glass defaults.

The constant block at the top of `crates/webterm/src/glass.rs` holds every number
(`DEFAULT_OPACITY`, `MIN_OPACITY`, `MAX_OPACITY`, `CHROME_FACTOR`,
`CHROME_FLOOR`, `INTENSITY_STEPS`, `BACKDROP_STEPS`). Tune there, not at call
sites.

### What the knob means, and why it is a knob

GPUI 0.3.3 has no `backdrop-filter`. A blur *inside* the window is therefore
impossible: an element cannot see what is painted behind it. A blur *of the
window* is possible, but only because a compositor owns the framebuffer — a
Wayland client is not allowed to read the pixels behind its own surface (that
would let any app peek at other windows), so a client can only send a
translucent surface plus, optionally, a request for the compositor to blur.

That request is a protocol, and which protocol exists depends on the session:

| Session | Backdrop blur |
| --- | --- |
| KWin 6.7+ (KDE Plasma) | `ext-background-effect-v1` — works today |
| Mutter 51+ (GNOME 51) | `ext-background-effect-v1` — works today |
| Hyprland, and wlroots compositors that implement the standard protocol | `ext-background-effect-v1` — works today |
| GNOME ≤ 50 (Mutter ships no blur protocol) | none — translucent only, `Frost` looks exactly like `Translucent` |
| X11 | none: no client-side request exists, and `wayland_session()` gates the whole path off |
| Windows 11 | gpui maps `Blurred` to the DWM acrylic accent — but this app does not route the setting there |
| macOS | gpui implements `Blurred` as a `BlurredView` (`NSVisualEffectView` subclass) — but this app does not route the setting there |

On Windows and macOS the window keeps the `Transparent` backdrop it is created
with, because `apply_backdrop_material` is cfg-gated to Linux: neither platform is
covered by this repo's CI, so the code does not guess at their behaviour. Wiring
them up is a two-line change if that ever matters.

Where no blur is available, `Frost` and `Translucent` produce identical pixels —
which is why choosing explicitly here is safe: there is no session to guess at,
and the setting defaults to `Frost` on the grounds that it can only ever add.

## The gpui patch

Stock gpui binds KDE's older `org_kde_kwin_blur` and never touches the
cross-compositor `ext-background-effect-v1`, so `Blurred` stays a plain
transparent surface on GNOME 51 and Hyprland as well. The fix cannot live in this
crate: gpui never hands out the window's `wl_surface`, and the request must go out
over the window's own Wayland connection. So the binding is added inside the
backend crate.

`patches/gpui-pre-linux-background-effect.patch` (10 hunks, +58/−2, two files):

- `src/linux/wayland/client.rs` — bind `ext_background_effect_manager_v1`, register
  the two new objects.
- `src/linux/wayland/window.rs` — hold one `ext_background_effect_surface_v1` per
  window and, in `update_window()`, hand it a blur region built surface-locally
  from the surface size inset by `state.inset()`. Surface-local matters: the
  rectangle gpui already computes for the *opaque* area still carries the
  window's requested position until the first configure resets it, so reusing it
  would blur the wrong place on the first frame. KDE's `org_kde_kwin_blur`
  remains the fallback.

One honest caveat about that region: gpui's `inset()` is 0 for this app (it draws
its own rounded frame inside the surface, so gpui never sets up a CSD inset), and
the patch cannot know the app's own shadow padding. So the region is the whole
surface, corners included, which on a sharp wallpaper can show a faint frosted
halo just outside the rounded frame. Fixing that properly means telling gpui the
frame's padding — an upstream concern, not an app one.

### Applying it

```sh
cd desktop-gpui
./scripts/patch-gpui.sh          # copy the crate to vendor/ + write .cargo/config.toml
./scripts/patch-gpui.sh --check  # dry run: does the patch still apply?
./scripts/patch-gpui.sh 0.3.4    # patch a specific crate version
```

How it works, in order:

1. The pristine source comes from the cargo registry cache, or from
   `static.crates.io` if the crate was never unpacked.
2. It is copied to `vendor/gpui-pre-linux` and the patch is applied — `git apply`
   first (forced into its no-repository mode, because the vendor tree sits outside
   a git work tree), then GNU `patch --fuzz=3` as the fuzzy fallback.
3. The result is *verified*: five code anchors are grepped, and if any is missing
   the vendor tree is deleted and the script fails. A hunk that drifted because of
   a gpui bump can never silently produce a build without blur.
4. `.cargo/config.toml` is written **at the repository root** with
   `[patch.crates-io] gpui-pre-linux = { path = "desktop-gpui/vendor/gpui-pre-linux" }`.
   Paths in a config `[patch]` resolve relative to the directory holding `.cargo`,
   so a build must run from the root — that is why `build-appimage.sh` `cd`s there.
   The script refuses to overwrite a `.cargo/config.toml` it did not write.

Both `vendor/` and `/.cargo/config.toml` are gitignored, and a checkout that never
ran the script still builds — it just gets stock gpui and a translucent-only
backdrop. CI applies the patch on Linux before the workspace tests; the AppImage
build applies it before the release build.

Reverting: delete `.cargo/config.toml` (and `vendor/` for the disk space). One
side effect to expect: while the patch is active, cargo drops the `source` and
`checksum` lines of the `gpui-pre-linux` entry in `desktop-gpui/Cargo.lock`. Leave
that local change out of commits — `git checkout -- desktop-gpui/Cargo.lock`.

## Verifying it for real

GNOME 50 cannot blur, so a real check needs a compositor that can. Any KWin 6.7+
on the machine can be run nested inside the current GNOME session, which renders
into an ordinary window:

```sh
# 1. build the patched binary (must run from the repository root)
cd desktop-gpui && ./scripts/patch-gpui.sh
cd .. && cargo build --manifest-path desktop-gpui/Cargo.toml -p webterm

# 2. a nested KWin on its own socket
kwin_wayland --socket wayland-nested --no-lockscreen >/tmp/kwin-nested.log 2>&1 &

# 3. run against it with protocol logging, fresh settings (= backdrop defaults to Frost)
cd desktop-gpui
WAYLAND_DISPLAY=wayland-nested WAYLAND_DEBUG=1 XDG_CONFIG_HOME=/tmp/glass-verify-config \
  ./target/debug/webterm >/tmp/glass-verify.log 2>&1 &

# 4. what success looks like
grep -E "get_background_effect|set_blur_region" /tmp/glass-verify.log
```

A patched build prints a bound manager, one blur object per window, and a blur
region that is re-issued on every configure (resize):

```
 -> wl_registry#2.bind(58, "ext_background_effect_manager_v1", 1, new id [unknown]#17)
 -> ext_background_effect_manager_v1#17.get_background_effect(new id ext_background_effect_surface_v1#80, wl_surface#26)
 -> wl_region#81.add(0, 0, 1160, 660)
 -> ext_background_effect_surface_v1#80.set_blur_region(wl_region#81)
```

No `get_background_effect` line means the running binary is the stock-gpui one, or
the setting is `Translucent`, or the compositor does not advertise the global.

## Troubleshooting

| Symptom | Cause | Fix |
| --- | --- | --- |
| Glass looks opaque, no translucency | feature off, or alpha at 1.0 | Settings → WINDOW, switch + intensity |
| Translucent but never frosted | compositor has no blur protocol (e.g. GNOME 50), or setting is `Translucent` | set `Frost`; upgrade to GNOME 51 / use KWin or Hyprland |
| `Frost` selected and the log has no `get_background_effect` | build did not use the patched crate | run the build from the repository root, and `./scripts/patch-gpui.sh --check` |
| `patch did not apply` after a gpui bump | an anchor moved | rebuild the patch per `patches/README.md` |
| `Cargo.lock` shows a `gpui-pre-linux` diff | expected while patched | `git checkout -- desktop-gpui/Cargo.lock` |
| A dark smudge near panel edges | the frame's `shadow_xl` showing through the transparent window | replace that shadow with a 1px ring |
| A frosted halo just outside the rounded corners on KWin/Hyprland | the blur region covers the whole surface, because gpui's `inset()` is 0 here | cosmetic; see the caveat under *The gpui patch* |

## When gpui catches up

Once upstream binds `ext-background-effect-v1` itself, delete the patch, the
script, the generated config, and the `vendor` entry from `.gitignore`; the
`Backdrop` setting and `glass.rs` keep working unchanged, because the app-side
code only ever asked for `WindowBackgroundAppearance::Blurred`.
