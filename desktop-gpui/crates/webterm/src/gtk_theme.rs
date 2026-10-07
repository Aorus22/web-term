//! GNOME/GTK theme probe — reads the *active* desktop palette so the UI can
//! follow it in the `Desktop (GTK)` theme mode.
//!
//! Two sources are merged, in this order of authority:
//!
//! 1. **The user's own GTK config** ([`UserCss`]) — `colors.css` / `gtk.css`
//!    under `~/.config/gtk-4.0`, then `~/.config/gtk-3.0`. On GNOME the
//!    `gtk-theme` setting is only half the story: libadwaita 1.6+ apps take
//!    their palette from `:root` custom properties in `~/.config/gtk-4.0/gtk.css`
//!    (what palette tools such as Rewaita write), with `@define-color` entries
//!    in the `colors.css` next to it. Those files are what the desktop
//!    *actually* looks like — a user whose `gtk-theme` is still `WhiteSur-Dark`
//!    but whose GTK4 config is Tokyo Night sees Tokyo Night everywhere else.
//!    Without user-CSS-first, this machine probes as WhiteSur and the Tokyo
//!    Night palette is lost.
//! 2. **The GTK3 style engine** — GTK is initialized on the main thread by
//!    [`init`] (called from `main` before the theme is used), so an unrealized
//!    `GtkWindow`'s style context resolves the theme's named colors without
//!    hand-rolling a theme parse. This is the fallback for users with no user
//!    CSS at all.
//!
//! # Threading contract
//!
//! GTK is **not** thread-safe: everything GTK-touching is confined to
//! [`probe`], [`current_key`] and [`init`], and every entry point bails out
//! unless it is running on the thread that called `gtk::init()`. Render paths
//! must never call those — they read [`cached_palette`], which is plain data.
//! The main-loop watcher (see `AppState::watch_gtk_theme`) polls
//! [`current_key`] about once a second and re-probes only when it changes.
//!
//! The CSS parser and [`palette_from_colors`] are pure and unit-tested so the
//! mapping stays testable without a display.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use gpui::Rgba;
use parking_lot::Mutex;

use crate::theme::{self, GtkPalette};

/// Raw named colors read from the active GTK theme (`None` = not defined).
/// Plain `Send` data — no GTK types escape this module.
#[derive(Clone, Debug, PartialEq)]
pub struct GtkColors {
    pub theme_name: String,
    /// `gtk-application-prefer-dark-theme`.
    pub prefer_dark: bool,
    pub bg: Rgba,
    pub fg: Rgba,
    pub base: Rgba,
    /// Raised surface (libadwaita `--card-bg-color`), when the user set one.
    pub card: Option<Rgba>,
    pub selected_bg: Option<Rgba>,
    pub selected_fg: Option<Rgba>,
    pub borders: Option<Rgba>,
    pub insensitive_fg: Option<Rgba>,
    pub error: Option<Rgba>,
    pub warning: Option<Rgba>,
    pub success: Option<Rgba>,
    pub accent_color: Option<Rgba>,
    pub accent_bg_color: Option<Rgba>,
    /// Whether the user's own `colors.css`/`gtk.css` supplied any of the above
    /// (Settings caption: "your GTK config" vs the named theme).
    pub user_css: bool,
}

/// Cache key: re-probe only when the GTK theme, its dark preference, or one of
/// the user's CSS files changes (editing `gtk.css` live-updates the UI).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThemeKey {
    pub name: String,
    pub prefer_dark: bool,
    /// Newest mtime among the user CSS files, in seconds (0 = none present).
    ///
    /// Load-bearing: editing `gtk.css` changes every colour without changing
    /// the theme name or `color-scheme`, so mtime is the only signal.
    pub css_mtime: u64,
}

/// The user's own GTK color tables (`colors.css` / `gtk.css`).
///
/// Names are normalized (`--window-bg-color`, `window_bg_color` and
/// `@theme_bg_color` all collapse to `theme_bg_color`), values keep their raw
/// text so `var()`/`@name` indirection can be followed at lookup time.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UserCss {
    vars: HashMap<String, String>,
    defines: HashMap<String, String>,
}

/// Files that make up the user's palette, **lowest priority first** (later
/// inserts overwrite earlier ones): GTK3 below GTK4, `colors.css` below the
/// `gtk.css` that carries the live libadwaita palette.
const USER_CSS_FILES: [(&str, &str); 4] = [
    ("gtk-3.0", "colors.css"),
    ("gtk-3.0", "gtk.css"),
    ("gtk-4.0", "colors.css"),
    ("gtk-4.0", "gtk.css"),
];

impl UserCss {
    pub fn is_empty(&self) -> bool {
        self.vars.is_empty() && self.defines.is_empty()
    }

    fn raw(&self, name: &str) -> Option<&str> {
        let key = normalize_css_name(name);
        self.vars
            .get(&key)
            .or_else(|| self.defines.get(&key))
            .map(|value| value.as_str())
    }

    /// Resolve one color by name, following `var(--x)` / `@x` indirection.
    pub fn get(&self, name: &str) -> Option<Rgba> {
        let mut value = self.raw(name)?.to_string();
        for _ in 0..4 {
            let reference = value
                .strip_prefix("var(")
                .and_then(|inner| inner.strip_suffix(')'))
                .map(|inner| inner.trim().to_string())
                .or_else(|| value.strip_prefix('@').map(|inner| inner.trim().to_string()));
            match reference {
                Some(reference) => value = self.raw(&reference)?.to_string(),
                None => return parse_css_color(&value),
            }
        }
        None
    }

    /// First of `names` that resolves — the priority lists in [`probe`] use it.
    pub fn get_any(&self, names: &[&str]) -> Option<Rgba> {
        names.iter().find_map(|name| self.get(name))
    }
}

fn normalize_css_name(name: &str) -> String {
    name.trim()
        .trim_start_matches("--")
        .trim_start_matches('@')
        .trim()
        .to_ascii_lowercase()
        .replace('-', "_")
}

/// `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa`, `rgb(...)`/`rgba(...)` and
/// `transparent`. Everything else (`currentColor`, `alpha(...)`, `color-mix`…)
/// returns `None` and leaves the token to the next source in the list.
fn parse_css_color(value: &str) -> Option<Rgba> {
    let value = value.trim();
    if let Some(hex) = value.strip_prefix('#') {
        let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
        return match hex.len() {
            3 | 4 => {
                let nibble =
                    |i: usize| u8::from_str_radix(&hex[i..i + 1], 16).ok().map(|v| v * 17);
                let alpha = match hex.len() {
                    4 => nibble(3)?,
                    _ => 0xff,
                };
                Some(gpui::rgba(
                    ((nibble(0)? as u32) << 24)
                        | ((nibble(1)? as u32) << 16)
                        | ((nibble(2)? as u32) << 8)
                        | alpha as u32,
                ))
            }
            6 => Some(gpui::rgb(
                ((byte(0)? as u32) << 16) | ((byte(2)? as u32) << 8) | byte(4)? as u32,
            )),
            8 => Some(gpui::rgba(
                ((byte(0)? as u32) << 24)
                    | ((byte(2)? as u32) << 16)
                    | ((byte(4)? as u32) << 8)
                    | byte(6)? as u32,
            )),
            _ => None,
        };
    }
    if value.eq_ignore_ascii_case("transparent") {
        return Some(gpui::rgba(0x00000000));
    }
    let open = value.find('(')?;
    if !value[..open].trim().eq_ignore_ascii_case("rgb")
        && !value[..open].trim().eq_ignore_ascii_case("rgba")
    {
        return None;
    }
    let inner = value[open + 1..].trim_end_matches(')').trim();
    let parts: Vec<&str> = inner.split(',').map(|part| part.trim()).collect();
    if parts.len() < 3 {
        return None;
    }
    let channel = |part: &str| part.split('%').next()?.trim().parse::<f32>().ok();
    let (r, g, b) = (channel(parts[0])?, channel(parts[1])?, channel(parts[2])?);
    let a = match parts.get(3) {
        Some(part) => channel(part)?,
        None => 1.0,
    };
    let to_byte = |v: f32| (v.clamp(0.0, 255.0).round() as u32) & 0xff;
    Some(gpui::rgba(
        (to_byte(r) << 24)
            | (to_byte(g) << 16)
            | (to_byte(b) << 8)
            | (a.clamp(0.0, 1.0) * 255.0).round() as u32,
    ))
}

/// Drop `/* … */` spans before parsing, **keeping their newlines** so two
/// declarations that shared a line around a comment do not get glued together
/// (which would silently drop the second one).
fn strip_css_comments(css: &str) -> String {
    let mut out = String::with_capacity(css.len());
    let mut rest = css;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        match rest[start + 2..].find("*/") {
            Some(end) => {
                for ch in rest[start + 2..start + 2 + end].chars() {
                    if ch == '\n' {
                        out.push('\n');
                    }
                }
                rest = &rest[start + 2 + end + 2..];
            }
            None => {
                for ch in rest[start + 2..].chars() {
                    if ch == '\n' {
                        out.push('\n');
                    }
                }
                return out;
            }
        }
    }
    out.push_str(rest);
    out
}

fn clean_css_value(value: &str) -> String {
    value
        .split(';')
        .next()
        .unwrap_or(value)
        .trim()
        .to_string()
}

/// Parse `--name: value;` custom properties and `@define-color name value;`
/// declarations out of one stylesheet.
///
/// A byte scanner rather than a line splitter: real user CSS puts several
/// declarations on one line (minified files, or lines joined when a comment
/// between them was stripped), and `:root { --a: …; --b: …; }` must yield both.
/// Values keep their raw text so `var()`/`@name` indirection can be followed at
/// lookup time.
pub fn parse_css_into(css: &str, out: &mut UserCss) {
    let css = strip_css_comments(css);
    let mut i = 0;
    while i < css.len() {
        if !css.is_char_boundary(i) {
            i += 1;
            continue;
        }
        let tail = &css[i..];
        if let Some(rest) = tail.strip_prefix("@define-color") {
            let end = rest.find(';').map(|end| i + "@define-color".len() + end);
            let decl = &css[i + "@define-color".len()..end.unwrap_or(css.len())];
            let mut parts = decl.trim().splitn(2, char::is_whitespace);
            if let (Some(name), Some(value)) = (parts.next(), parts.next()) {
                out.defines
                    .insert(normalize_css_name(name), clean_css_value(value));
            }
            i = end.map(|end| end + 1).unwrap_or(css.len());
        } else if let Some(rest) = tail.strip_prefix("--") {
            let end = rest.find(';').map(|end| i + 2 + end);
            let decl = &css[i + 2..end.unwrap_or(css.len())];
            if let Some((name, value)) = decl.split_once(':') {
                out.vars
                    .insert(normalize_css_name(name), clean_css_value(value));
            }
            i = end.map(|end| end + 1).unwrap_or(css.len());
        } else {
            i += tail.chars().next().map(char::len_utf8).unwrap_or(1);
        }
    }
}

fn user_css_paths() -> Vec<PathBuf> {
    let Some(config) = dirs::config_dir() else {
        return Vec::new();
    };
    USER_CSS_FILES
        .iter()
        .map(|(dir, file)| config.join(dir).join(file))
        .collect()
}

/// Read every user CSS file, in increasing priority order.
fn load_user_css() -> UserCss {
    let mut out = UserCss::default();
    for path in user_css_paths() {
        if let Ok(text) = std::fs::read_to_string(&path) {
            parse_css_into(&text, &mut out);
        }
    }
    out
}

/// Newest mtime among the user CSS files (cache key for live theme edits).
pub fn user_css_mtime() -> u64 {
    user_css_paths()
        .iter()
        .filter_map(|path| std::fs::metadata(path).ok())
        .filter_map(|meta| meta.modified().ok())
        .filter_map(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|since| since.as_secs())
        .max()
        .unwrap_or(0)
}

static GTK_READY: AtomicBool = AtomicBool::new(false);
static MAIN_THREAD: Mutex<Option<std::thread::ThreadId>> = Mutex::new(None);
static CACHE: Mutex<Option<(ThemeKey, GtkPalette)>> = Mutex::new(None);
/// Whether the last probe found user CSS — drives the Settings caption (read
/// without touching the filesystem again).
static USER_CSS_SEEN: AtomicBool = AtomicBool::new(false);
/// Theme name the cached palette came from (Settings caption / diagnostics).
static CACHED_NAME: Mutex<Option<String>> = Mutex::new(None);

/// GNOME's fallback accent (Adwaita blue) when a theme defines no accent at all.
const GNOME_BLUE: u32 = 0x3584e4;
/// Fallback status hues when the theme defines none; matches the values the
/// built-in presets and the views' status badges already use.
const FALLBACK_ERROR: u32 = 0xef4444;
const FALLBACK_WARNING: u32 = 0xf59e0b;
const FALLBACK_SUCCESS: u32 = 0x22c55e;

/// Initialize GTK on this (main) thread.
///
/// Must run before any theme is resolved and before any other GTK consumer
/// (tray/indicator) is created. Returns whether GTK is usable; the app keeps
/// running with its built-in palettes when it is not.
///
/// The whole probe is Linux-only (`gtk` is a Linux target dependency — see
/// `crates/webterm/Cargo.toml`), so elsewhere this is a documented no-op and
/// every caller falls back to `cx.window_appearance()`.
pub fn init() -> bool {
    #[cfg(not(target_os = "linux"))]
    {
        false
    }
    #[cfg(target_os = "linux")]
    {
        match gtk::init() {
            Ok(()) => {
                GTK_READY.store(true, Ordering::SeqCst);
                *MAIN_THREAD.lock() = Some(std::thread::current().id());
                true
            }
            Err(error) => {
                eprintln!("[webterm] GTK unavailable, desktop theme probing disabled: {error:?}");
                false
            }
        }
    }
}

pub fn is_available() -> bool {
    GTK_READY.load(Ordering::SeqCst)
}

/// GTK widgets and `GtkSettings` may only be touched from the thread that ran
/// `gtk::init()`. Every entry point bails out elsewhere instead of tripping
/// gtk-rs's `assert_initialized_main_thread!` panic.
fn on_main_thread() -> bool {
    match *MAIN_THREAD.lock() {
        Some(id) => id == std::thread::current().id(),
        None => false,
    }
}

/// Cheap change detector: theme name + dark preference + user CSS mtime.
pub fn current_key() -> Option<ThemeKey> {
    if !is_available() || !on_main_thread() {
        return None;
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
    #[cfg(target_os = "linux")]
    {
        let settings = gtk::Settings::default()?;
        use gtk::prelude::*;
        Some(ThemeKey {
            name: settings
                .gtk_theme_name()
                .map(|s| s.to_string())
                .unwrap_or_default(),
            prefer_dark: settings.is_gtk_application_prefer_dark_theme(),
            css_mtime: user_css_mtime(),
        })
    }
}

/// Name of the theme the cached palette came from. **No GTK access** — safe to
/// call from the render path for the Settings caption.
pub fn cached_name() -> Option<String> {
    CACHED_NAME.lock().clone().filter(|name| !name.is_empty())
}

/// Whether the cached palette came from the user's own `colors.css`/`gtk.css`
/// rather than from the theme named by `gtk-theme`.
pub fn user_css_active() -> bool {
    USER_CSS_SEEN.load(Ordering::SeqCst)
}

/// The palette last resolved by [`palette`], without probing GTK.
///
/// This is what `current_theme()`/render paths read: plain `Copy` data, no
/// filesystem or GTK access.
pub fn cached_palette() -> Option<GtkPalette> {
    CACHE.lock().as_ref().map(|(_, palette)| *palette)
}

/// Polarity of the cached palette, if one was ever resolved.
pub fn cached_is_dark() -> Option<bool> {
    cached_palette().map(|palette| palette.is_dark)
}

/// Resolved GTK palette, re-probed only when [`current_key`] changes.
///
/// Returns `None` when GTK is unavailable, the probe fails, or this is not the
/// GTK thread — the caller then falls back to its built-in palette.
pub fn palette() -> Option<GtkPalette> {
    let key = current_key()?;
    {
        let cache = CACHE.lock();
        if let Some((cached_key, palette)) = cache.as_ref() {
            if *cached_key == key {
                return Some(*palette);
            }
        }
    }
    let colors = probe()?;
    let palette = palette_from_colors(&colors);
    USER_CSS_SEEN.store(colors.user_css, Ordering::SeqCst);
    *CACHED_NAME.lock() = Some(colors.theme_name.clone());
    *CACHE.lock() = Some((key, palette));
    Some(palette)
}

/// Drop the cache (forced re-probe after a failed one, or tests).
pub fn invalidate_cache() {
    *CACHE.lock() = None;
    *CACHED_NAME.lock() = None;
}

/// Read the active theme's named colors: the user's own CSS first, then the
/// GTK3 style context of a throwaway popup window.
///
/// The window is never shown: creating it is what attaches the theme's style
/// provider to the style context, which is also how `GtkStyleContext::lookup_color`
/// is used from C.
#[cfg(target_os = "linux")]
fn probe() -> Option<GtkColors> {
    if !is_available() || !on_main_thread() {
        return None;
    }
    use gtk::prelude::*;

    let settings = gtk::Settings::default();
    let prefer_dark = settings
        .as_ref()
        .map(|s| s.is_gtk_application_prefer_dark_theme())
        .unwrap_or(false);
    let theme_name = settings
        .as_ref()
        .and_then(|s| s.gtk_theme_name())
        .map(|s| s.to_string())
        .unwrap_or_default();

    let user = load_user_css();
    let user_css = !user.is_empty();
    let window = gtk::Window::new(gtk::WindowType::Popup);
    let context = window.style_context();
    let lookup = |name: &str| {
        context.lookup_color(name).map(|c| Rgba {
            r: c.red() as f32,
            g: c.green() as f32,
            b: c.blue() as f32,
            a: c.alpha() as f32,
        })
    };
    // User CSS wins; the GTK3 theme is the fallback. Each list runs from the
    // most specific name to the most generic one.
    let from_user = |names: &[&str]| user.get_any(names);
    let pick = |names: &[&str], theme_names: &[&str]| {
        from_user(names).or_else(|| theme_names.iter().find_map(|name| lookup(name)))
    };

    // Every GTK theme must define a background and foreground; without them the
    // theme is unusable for us and the caller falls back to its own palette.
    let bg = pick(
        &["window_bg_color", "theme_bg_color_breeze", "theme_bg_color"],
        &["theme_bg_color", "bg_color"],
    )?;
    let fg = pick(
        &[
            "window_fg_color",
            "theme_fg_color_breeze",
            "theme_fg_color",
            "theme_text_color_breeze",
            "theme_text_color",
        ],
        &["theme_fg_color", "fg_color"],
    )?;
    let base = pick(
        &[
            "theme_base_color_breeze",
            "theme_base_color",
            "view_bg_color",
            "content_view_bg_breeze",
        ],
        &["theme_base_color", "base_color"],
    )
    .unwrap_or(bg);
    // Raised surfaces: libadwaita cards/popovers, else the base color.
    let card = from_user(&["card_bg_color", "popover_bg_color", "theme_card_bg_color"]);

    Some(GtkColors {
        theme_name,
        prefer_dark,
        bg,
        fg,
        base,
        card,
        user_css,
        selected_bg: pick(
            &[
                "accent_bg_color",
                "accent_color",
                "theme_selected_bg_color_breeze",
                "theme_selected_bg_color",
                "theme_hovering_selected_bg_color_breeze",
                // Palette-only configs (Rewaita-style) name the hues directly.
                "blue_1",
            ],
            &["theme_selected_bg_color", "selected_bg_color"],
        ),
        selected_fg: pick(
            &[
                "accent_fg_color",
                "theme_selected_fg_color_breeze",
                "theme_selected_fg_color",
            ],
            &["theme_selected_fg_color", "selected_fg_color"],
        ),
        borders: pick(
            &[
                "borders_breeze",
                "unfocused_borders_breeze",
                "borders",
                "border_color",
                "sidebar_border_color",
            ],
            &["borders", "unfocused_borders"],
        ),
        insensitive_fg: pick(
            &[
                "insensitive_fg_color_breeze",
                "insensitive_fg_color",
                "theme_unfocused_fg_color_breeze",
                "theme_unfocused_fg_color",
            ],
            &["insensitive_fg_color"],
        ),
        error: pick(
            &["error_color_breeze", "error_color", "red_1", "red_2"],
            &["error_color"],
        ),
        warning: pick(
            &[
                "warning_color_breeze",
                "warning_color",
                "yellow_1",
                "orange_1",
            ],
            &["warning_color"],
        ),
        success: pick(
            &["success_color_breeze", "success_color", "green_1"],
            &["success_color"],
        ),
        accent_color: pick(&["accent_color", "blue_1"], &["accent_color"]),
        accent_bg_color: pick(&["accent_bg_color"], &["accent_bg_color"]),
    })
}

/// Off Linux there is no `gtk` crate linked at all, so there is nothing to
/// probe; the app stays on its built-in presets.
#[cfg(not(target_os = "linux"))]
fn probe() -> Option<GtkColors> {
    None
}

/// Map the theme's named colors onto the app's token table (pure — unit-tested).
///
/// The result is shaped exactly like a [`crate::theme::ThemePreset`], so every
/// existing `AppState` accessor follows the desktop with no extra plumbing.
pub fn palette_from_colors(c: &GtkColors) -> GtkPalette {
    let bg = theme::hex_from_rgba(c.bg);
    let fg = theme::hex_from_rgba(c.fg);
    let base = theme::hex_from_rgba(c.base);
    let card = c.card.map(theme::hex_from_rgba).unwrap_or(base);

    // GNOME 47+ accents live in `accent_bg_color` (libadwaita); GTK3 themes
    // embed their own in `theme_selected_bg_color`.
    let accent = c
        .selected_bg
        .or(c.accent_bg_color)
        .or(c.accent_color)
        .map(theme::hex_from_rgba)
        .unwrap_or(GNOME_BLUE);

    let is_dark = c.prefer_dark || theme::hex_luminance(bg) < 0.5;

    // `accent_fg_color` can be dark (#222222 on this machine) — never assume
    // white text on the accent; use what the desktop declares, else compute it.
    let on_accent = c
        .selected_fg
        .map(theme::hex_from_rgba)
        .unwrap_or_else(|| theme::contrast_hex(accent));

    let border = c
        .borders
        .map(theme::hex_from_rgba)
        .unwrap_or_else(|| theme::mix_hex(base, fg, 0.18));

    // `insensitive_fg_color` is usually translucent; composite it so tokens stay
    // opaque and no unexpected layering shows up. The composite can land far
    // below readable (translucent white on near-black lands ~1.6:1), so hold it
    // to the same secondary-text floor the gallery presets obey.
    let muted_foreground = match c.insensitive_fg {
        Some(insensitive) => theme::mix_hex(bg, theme::hex_from_rgba(insensitive), insensitive.a),
        None => theme::mix_hex(bg, fg, 0.55),
    };
    let muted_foreground = theme::readable_muted(muted_foreground, &[card, bg], fg);

    let destructive = c
        .error
        .map(theme::hex_from_rgba)
        .unwrap_or(FALLBACK_ERROR);
    let warning = c
        .warning
        .map(theme::hex_from_rgba)
        .unwrap_or(FALLBACK_WARNING);
    let success = c
        .success
        .map(theme::hex_from_rgba)
        .unwrap_or(FALLBACK_SUCCESS);

    GtkPalette {
        is_dark,
        background: bg,
        foreground: fg,
        card,
        card_foreground: fg,
        // The desktop accent is the brand colour; both `primary` and `accent`
        // point at it so nav/tab highlights and control borders agree.
        primary: accent,
        primary_foreground: on_accent,
        secondary: theme::mix_hex(card, fg, 0.08),
        secondary_foreground: fg,
        muted: theme::mix_hex(bg, fg, 0.06),
        muted_foreground,
        accent,
        accent_foreground: on_accent,
        destructive,
        warning,
        success,
        border,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn color(hex: u32) -> Rgba {
        gpui::rgb(hex)
    }

    /// WhiteSur-Dark-esque fixture (values probed from the real theme).
    fn fixture() -> GtkColors {
        GtkColors {
            theme_name: "WhiteSur-Dark".to_string(),
            prefer_dark: false,
            bg: color(0x333333),
            fg: color(0xdedede),
            base: color(0x242424),
            card: None,
            user_css: false,
            selected_bg: Some(color(0x0860f2)),
            selected_fg: Some(color(0xffffff)),
            borders: Some(color(0x2f3140)),
            insensitive_fg: Some(Rgba {
                a: 0.35,
                ..color(0xdedede)
            }),
            error: Some(color(0xed5f5d)),
            warning: Some(color(0xe9873a)),
            success: Some(color(0x79b757)),
            accent_color: None,
            accent_bg_color: None,
        }
    }

    #[test]
    fn maps_theme_colors_onto_tokens() {
        let palette = palette_from_colors(&fixture());
        // #333333 background: dark despite `prefer_dark == false`.
        assert!(palette.is_dark);
        assert_eq!(palette.background, 0x333333);
        assert_eq!(palette.card, 0x242424);
        assert_eq!(palette.foreground, 0xdedede);
        assert_eq!(palette.accent, 0x0860f2);
        assert_eq!(palette.accent_foreground, 0xffffff);
        assert_eq!(palette.border, 0x2f3140);
        assert_eq!(palette.destructive, 0xed5f5d);
        assert_eq!(palette.warning, 0xe9873a);
        assert_eq!(palette.success, 0x79b757);
    }

    #[test]
    fn accent_falls_back_to_gnome_blue() {
        let mut colors = fixture();
        colors.selected_bg = None;
        colors.accent_bg_color = None;
        colors.accent_color = None;
        assert_eq!(palette_from_colors(&colors).accent, GNOME_BLUE);
        // …and a computed foreground when the theme declares none.
        assert_eq!(
            palette_from_colors(&colors).accent_foreground,
            theme::contrast_hex(GNOME_BLUE)
        );
    }

    #[test]
    fn light_theme_stays_light() {
        let colors = GtkColors {
            theme_name: "Adwaita".to_string(),
            prefer_dark: false,
            bg: color(0xf6f5f4),
            fg: color(0x2e3436),
            base: color(0xffffff),
            card: None,
            user_css: false,
            selected_bg: Some(color(0x3584e4)),
            selected_fg: None,
            borders: None,
            insensitive_fg: None,
            error: None,
            warning: None,
            success: None,
            accent_color: None,
            accent_bg_color: None,
        };
        let palette = palette_from_colors(&colors);
        assert!(!palette.is_dark);
        assert_eq!(palette.border, theme::mix_hex(0xffffff, 0x2e3436, 0.18));
        assert_eq!(palette.accent_foreground, theme::contrast_hex(0x3584e4));
        assert_eq!(palette.destructive, FALLBACK_ERROR);
    }

    /// Tokyo Night as it sits on this project's dev machine: `gtk-theme` is
    /// still `WhiteSur-Dark`, but the GTK4 user config (Rewaita output) is what
    /// every libadwaita app on the desktop actually renders.
    const TOKYO_NIGHT_GTK_CSS: &str = r#"
:root {
  --window-bg-color: #1a1b26;
  --window-fg-color: #a9b1d6;
  --card-bg-color: #282a38;
  --headerbar-bg-color: #16161e;
  --sidebar-bg-color: var(--window-bg-color);
  --blue-1: #7aa2f7;
  --green-1: #9ece6a;
  --yellow-1: #e0af68;
  --red-1: #f7768e;
  color: var(--window-fg-color);
}
"#;

    const TOKYO_NIGHT_COLORS_CSS: &str = r#"
/* exported from the desktop color scheme */
@define-color theme_bg_color_breeze #1a1b26;
@define-color theme_fg_color_breeze #a9b1d6;
@define-color theme_base_color_breeze #1a1b26;
@define-color theme_selected_bg_color_breeze #7aa2f7;
@define-color theme_selected_fg_color_breeze #1a1b26;
@define-color borders_breeze #373949;
@define-color insensitive_fg_color_breeze #484c5f;
@define-color error_color_breeze #f7768e;
@define-color warning_color_breeze #e0af68;
@define-color success_color_breeze #9ece6a;
@define-color borders alpha(currentColor, 0.12);
"#;

    fn user_css() -> UserCss {
        let mut user = UserCss::default();
        parse_css_into(TOKYO_NIGHT_GTK_CSS, &mut user);
        parse_css_into(TOKYO_NIGHT_COLORS_CSS, &mut user);
        user
    }

    #[test]
    fn css_parser_reads_vars_and_defines() {
        let user = user_css();
        assert!(!user.is_empty());
        // `--var` and `@define-color` land in the same namespace.
        assert_eq!(user.get("--window-bg-color"), Some(color(0x1a1b26)));
        assert_eq!(user.get("window_bg_color"), Some(color(0x1a1b26)));
        assert_eq!(user.get("theme_bg_color_breeze"), Some(color(0x1a1b26)));
        assert_eq!(user.get("borders_breeze"), Some(color(0x373949)));
        // var()/colour references are followed.
        assert_eq!(user.get("sidebar_bg_color"), Some(color(0x1a1b26)));
        // Unsupported values are skipped, not guessed.
        assert_eq!(user.get("borders"), None);
        assert_eq!(user.get("does_not_exist"), None);
    }

    #[test]
    fn css_parser_handles_shorthand_hex_and_comments() {
        let mut user = UserCss::default();
        parse_css_into(
            "@define-color a #abc;\n/* c1 */ --b: rgba(10, 20, 30, 0.5); /* c2\nspanning */ --c: #11223344;",
            &mut user,
        );
        assert_eq!(user.get("a"), Some(color(0xaabbcc)));
        // Alpha is quantized to a byte, so compare with a tolerance.
        let alpha = user.get("b").unwrap().a;
        assert!((alpha - 0.5).abs() < 0.01, "alpha was {alpha}");
        assert_eq!(user.get("c").unwrap().a, 0x44 as f32 / 255.0);
    }

    /// The user CSS accent declarations win over the theme's own selection
    /// color — on the dev machine the desktop accent is Tokyo Night pink
    /// (`accent_bg_color`) while `theme_selected_bg_color` is a different blue.
    #[test]
    fn user_css_drives_the_tokyo_night_palette() {
        let user = user_css();
        let colors = GtkColors {
            theme_name: "WhiteSur-Dark".to_string(),
            prefer_dark: true,
            bg: user.get_any(&["window_bg_color", "theme_bg_color"]).unwrap(),
            fg: user
                .get_any(&["window_fg_color", "theme_fg_color_breeze"])
                .unwrap(),
            base: user
                .get_any(&["theme_base_color_breeze", "view_bg_color"])
                .unwrap(),
            card: user.get_any(&["card_bg_color", "popover_bg_color"]),
            user_css: true,
            selected_bg: user.get_any(&["accent_bg_color", "theme_selected_bg_color_breeze"]),
            selected_fg: user.get_any(&["accent_fg_color", "theme_selected_fg_color_breeze"]),
            borders: user.get_any(&["borders_breeze", "borders"]),
            insensitive_fg: user.get_any(&["insensitive_fg_color_breeze", "insensitive_fg_color"]),
            error: user.get_any(&["error_color_breeze", "red_1"]),
            warning: user.get_any(&["warning_color_breeze", "yellow_1"]),
            success: user.get_any(&["success_color_breeze", "green_1"]),
            accent_color: None,
            accent_bg_color: None,
        };
        let palette = palette_from_colors(&colors);

        assert!(palette.is_dark);
        assert_eq!(palette.background, 0x1a1b26);
        assert_eq!(palette.card, 0x282a38);
        assert_eq!(palette.foreground, 0xa9b1d6);
        // The composite insensitive fg lands at 1.67:1 on the card; the
        // readability floor lifts it to 0x8c93b2 (4.69:1 card / 5.64:1 bg).
        assert_eq!(palette.muted_foreground, 0x8c93b2);
        assert_eq!(palette.accent, 0x7aa2f7);
        assert_eq!(palette.accent_foreground, 0x1a1b26);
        assert_eq!(palette.border, 0x373949);
        assert_eq!(palette.destructive, 0xf7768e);
        assert_eq!(palette.warning, 0xe0af68);
        assert_eq!(palette.success, 0x9ece6a);
    }

    #[test]
    fn no_gtk_means_no_palette_instead_of_a_panic() {
        // Tests never initialize GTK: the guards must return None, not assert.
        if !is_available() {
            assert!(current_key().is_none());
            assert!(palette().is_none());
        }
    }
}
