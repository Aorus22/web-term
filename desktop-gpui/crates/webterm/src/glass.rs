//! Liquid Glass material for the window chrome.
//!
//! The window is opened with `WindowBackgroundAppearance::Transparent` and the
//! root view deliberately paints no background, so every chrome leaf that stops
//! painting an opaque fill lets the desktop show through. This module turns the
//! theme's opaque colours into a translucent "glass" slab and adds the two
//! layers that make it read as a material rather than as a faded panel:
//!
//! * a hairline border whose ink comes from the theme's *polarity* (white on
//!   dark, black on light) instead of the opaque `border` token, and
//! * an inset rim — 1px of light along the top edge, 1px of shade along the
//!   bottom — which is the liquid-glass silhouette cue.
//!
//! # What this deliberately is not
//!
//! GPUI 0.3.3 has no `backdrop-filter`: there is no way to blur what is behind
//! an element, and `WindowBackgroundAppearance::Blurred` only does anything on
//! Wayland compositors that expose `org_kde_kwin_blur` (KWin, Hyprland) — GNOME
//! ignores it. So on GNOME the glass is a genuine translucent tint over the
//! desktop, not a frosted blur. Two things follow, and both are load-bearing:
//!
//! 1. Overlay surfaces (dialogs, sheets, menus, toasts) sit over the app's own
//!    content rather than over the desktop. A translucent panel over terminal
//!    text is unreadable, so their fill stays close to opaque and the scrim
//!    already in the view supplies the separation.
//! 2. The alpha lives in the *colour*, never in `.opacity()` on a subtree:
//!    GPUI has no group compositing, so element opacity multiplies into each
//!    primitive separately and would show every child through every other.
//!
//! Turning the feature off must restore the previous pixels exactly, so the
//! opaque path here returns the theme's own values and the plain elevation
//! stack — no rim, no polarity ink.

use gpui::{hsla, px, BoxShadow, Hsla, Rgba, Window};
use webterm_settings::DesktopSettings;

/// Alpha of the overlay fill, i.e. the value the Settings control writes.
pub const DEFAULT_OPACITY: f32 = 0.85;
/// Below this a translucent panel over content stops being readable.
pub const MIN_OPACITY: f32 = 0.55;
/// 1.0 is the feature's off-ramp within the translucent range: a fill that
/// hides everything behind it while still keeping the rim and polarity border.
pub const MAX_OPACITY: f32 = 1.0;

/// Chrome sits this fraction of the overlay alpha, so the desktop still reads
/// through the sidebar and tab strip even at the most opaque setting.
const CHROME_FACTOR: f32 = 0.78;
/// …with a floor, so the chrome never dissolves into the desktop at the lowest
/// setting. No ceiling is needed: the factor already keeps chrome below the
/// overlay alpha at every setting.
const CHROME_FLOOR: f32 = 0.45;

/// Settings presets, strongest glass last. Labels are what the UI shows.
pub const INTENSITY_STEPS: [(&str, f32); 3] = [("Subtle", 0.94), ("Medium", 0.85), ("Bold", 0.68)];

/// Which side of the window a glass surface lives on. They do not share an
/// alpha because what shows through them differs: chrome is over the desktop,
/// overlays are over the app's own content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlassTier {
    /// Window chrome that has the desktop behind it: sidebar, tab strip, sheets.
    Chrome,
    /// Floating surfaces over app content: dialogs, menus, toasts.
    Overlay,
}

/// The elevation a leaf had before glass, so the glass path can keep it.
///
/// Mirrors gpui's `shadow_*` scale rather than only the steps in use today:
/// a leaf that later grows a `shadow_sm` (the switch knobs already have one)
/// should find its step here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Elevation {
    None,
    Sm,
    Lg,
    Xl,
    /// gpui's `shadow_2xl`, used by the font dialog.
    Xxl,
}

/// Everything a surface needs to paint itself as glass.
#[derive(Debug, Clone, PartialEq)]
pub struct GlassStyle {
    pub fill: Rgba,
    pub border: Rgba,
    pub shadows: Vec<BoxShadow>,
    /// False for the pass-through path (feature off), which is what the tests
    /// assert on to prove the "off" rendering is byte-for-byte the old one.
    pub glassy: bool,
}

impl GlassStyle {
    pub fn is_glassy(&self) -> bool {
        self.glassy
    }
}

/// Build a glass style, or the feature's disabled rendering.
///
/// `base` is the theme colour the leaf paints today (its `bg` or `card`
/// token), `border` the theme's `border` token. When the setting is off this
/// returns exactly those colours with the plain elevation stack.
pub fn style_for(
    settings: &DesktopSettings,
    base: Rgba,
    border: Rgba,
    is_dark: bool,
    tier: GlassTier,
    elevation: Elevation,
) -> GlassStyle {
    if !settings.glass_enabled {
        return GlassStyle {
            fill: base,
            border,
            shadows: elevation_stack(elevation),
            glassy: false,
        };
    }
    style(base, is_dark, settings.glass_opacity, tier, elevation)
}

/// Glass style regardless of the setting — what [`style_for`] delegates to
/// once it knows the material is on, and what the tests exercise directly.
pub fn style(
    base: Rgba,
    is_dark: bool,
    opacity: f32,
    tier: GlassTier,
    elevation: Elevation,
) -> GlassStyle {
    let mut shadows = elevation_stack(elevation);
    // Order matters only in that these are painted after the fill; GPUI splits
    // the stack by `inset` on its own (see `paint_inset_shadows`).
    shadows.push(
        BoxShadow::new(px(0.), px(1.), rim_ink(is_dark))
            .blur_radius(px(1.))
            .inset(),
    );
    shadows.push(
        BoxShadow::new(px(0.), px(-1.), shade_ink(is_dark))
            .blur_radius(px(1.))
            .inset(),
    );
    GlassStyle {
        fill: base.alpha(tier_alpha(tier, opacity)),
        border: border_ink(is_dark),
        shadows,
        glassy: true,
    }
}

/// Clamp a persisted alpha into the readable window, tolerating hand-edited
/// files and NaN.
pub fn clamp_opacity(opacity: f32) -> f32 {
    if !opacity.is_finite() {
        return DEFAULT_OPACITY;
    }
    opacity.clamp(MIN_OPACITY, MAX_OPACITY)
}

/// Fill alpha for a tier.
pub fn tier_alpha(tier: GlassTier, opacity: f32) -> f32 {
    let opacity = clamp_opacity(opacity);
    match tier {
        GlassTier::Overlay => opacity,
        GlassTier::Chrome => (opacity * CHROME_FACTOR).max(CHROME_FLOOR),
    }
}

/// Hairline ink: white reads as a lifted edge on dark fills, black as a
/// definition line on light ones.
pub fn border_ink(is_dark: bool) -> Rgba {
    if is_dark {
        hsla(0., 0., 1., 0.10).into()
    } else {
        hsla(0., 0., 0., 0.08).into()
    }
}

/// 1px of light along the top inner edge.
pub fn rim_ink(is_dark: bool) -> Hsla {
    if is_dark {
        hsla(0., 0., 1., 0.07)
    } else {
        hsla(0., 0., 1., 0.85)
    }
}

/// 1px of shade along the bottom inner edge, so the slab reads as having
/// thickness rather than as a float.
pub fn shade_ink(is_dark: bool) -> Hsla {
    if is_dark {
        hsla(0., 0., 0., 0.18)
    } else {
        hsla(0., 0., 0., 0.04)
    }
}

/// Whether the compositor can blur what is behind the window.
///
/// Only the KDE blur protocol (`org_kde_kwin_blur`, implemented by KWin and
/// Hyprland) can do this, and gpui's Wayland backend is the only one that binds
/// it — X11 and GNOME sessions get nothing. Asking the platform would be
/// better, but gpui exposes no query, so this reads the desktop session the way
/// the compositor itself identifies it.
pub fn backdrop_blur_available() -> bool {
    if std::env::var_os("WAYLAND_DISPLAY").is_none() {
        return false;
    }
    let desktop = std::env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_default()
        .to_ascii_lowercase();
    desktop.contains("kde") || desktop.contains("hyprland")
}

/// Pick the window backdrop for the current glass setting.
///
/// `Transparent` is what the CSD frame already relies on (rounded corners show
/// real desktop); `Blurred` is the same thing plus a backdrop blur where the
/// compositor implements one. On GNOME the two are identical, so this never
/// makes the window worse — it only upgrades sessions that can frost it.
///
/// Only Linux is touched: X11 has no blur protocol to bind (so it lands on the
/// `Transparent` the window already had), and macOS/Windows keep whatever
/// backdrop the window was created with rather than have this repo's CI — which
/// covers neither — guess at theirs.
pub fn apply_backdrop_material(window: &mut Window, glass_enabled: bool) {
    #[cfg(target_os = "linux")]
    {
        let appearance = if glass_enabled && backdrop_blur_available() {
            gpui::WindowBackgroundAppearance::Blurred
        } else {
            gpui::WindowBackgroundAppearance::Transparent
        };
        window.set_background_appearance(appearance);
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (window, glass_enabled);
    }
}

/// The elevation stack a leaf had before glass.
///
/// These are gpui's own `shadow_sm`/`shadow_lg`/`shadow_xl` values copied
/// verbatim (`gpui-pre-macros/src/styles.rs`): a glass leaf needs the outer
/// layers and the rim in *one* vec, because `.shadow()` replaces the stack
/// rather than appending to it, and the styled helpers expose no getter.
pub fn elevation_stack(elevation: Elevation) -> Vec<BoxShadow> {
    match elevation {
        Elevation::None => Vec::new(),
        Elevation::Sm => vec![
            BoxShadow::new(px(0.), px(1.), hsla(0., 0., 0., 0.1)).blur_radius(px(3.)),
            BoxShadow::new(px(0.), px(1.), hsla(0., 0., 0., 0.1))
                .blur_radius(px(2.))
                .spread_radius(px(-1.)),
        ],
        Elevation::Lg => vec![
            BoxShadow::new(px(0.), px(10.), hsla(0., 0., 0., 0.1))
                .blur_radius(px(15.))
                .spread_radius(px(-3.)),
            BoxShadow::new(px(0.), px(4.), hsla(0., 0., 0., 0.1))
                .blur_radius(px(6.))
                .spread_radius(px(-4.)),
        ],
        Elevation::Xl => vec![
            BoxShadow::new(px(0.), px(20.), hsla(0., 0., 0., 0.1))
                .blur_radius(px(25.))
                .spread_radius(px(-5.)),
            BoxShadow::new(px(0.), px(8.), hsla(0., 0., 0., 0.1))
                .blur_radius(px(10.))
                .spread_radius(px(-6.)),
        ],
        Elevation::Xxl => vec![
            BoxShadow::new(px(0.), px(25.), hsla(0., 0., 0., 0.25))
                .blur_radius(px(50.))
                .spread_radius(px(-12.)),
        ],
    }
}
