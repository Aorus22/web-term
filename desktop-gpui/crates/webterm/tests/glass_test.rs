//! Unit tests for the Liquid Glass material: tier alphas, the pass-through
//! rendering when the feature is off, and the persistence of the setting.

use webterm::app_state::AppState;
use webterm::glass::{self, Elevation, GlassTier};
use webterm::theme;
use webterm_settings::{DesktopSettings, Theme as SettingsTheme};

/// A path under the OS temp dir, unique per test run.
fn temp_base(tag: &str) -> std::path::PathBuf {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("webterm_glass_test_{tag}_{now}"));
    let _ = std::fs::create_dir_all(&dir);
    dir
}

#[test]
fn test_opacity_clamping_is_readable_and_finite() {
    assert_eq!(glass::clamp_opacity(0.85), 0.85);
    // Below the floor a translucent panel over content stops being readable.
    assert_eq!(glass::clamp_opacity(0.1), glass::MIN_OPACITY);
    // Above it there is nothing left to hide: the range tops out at opaque.
    assert_eq!(glass::clamp_opacity(5.0), glass::MAX_OPACITY);
    // A hand-edited or corrupted file must not be able to produce NaN alpha.
    assert_eq!(glass::clamp_opacity(f32::NAN), glass::DEFAULT_OPACITY);
    assert_eq!(glass::clamp_opacity(-0.0), glass::MIN_OPACITY);
}

#[test]
fn test_tier_alphas_keep_chrome_more_transparent_than_overlays() {
    // Overlay surfaces sit over the app's own content, so the setting *is*
    // their alpha.
    assert_eq!(glass::tier_alpha(GlassTier::Overlay, 0.85), 0.85);
    assert_eq!(glass::tier_alpha(GlassTier::Overlay, 1.0), 1.0);

    // Chrome has the desktop behind it and always stays lighter than an
    // overlay at the same setting.
    let chrome = glass::tier_alpha(GlassTier::Chrome, 0.85);
    assert!((chrome - 0.85 * 0.78).abs() < 1e-6, "chrome alpha was {chrome}");
    assert!(chrome < glass::tier_alpha(GlassTier::Overlay, 0.85));

    // …but never dissolves, even at the most transparent setting.
    assert_eq!(
        glass::tier_alpha(GlassTier::Chrome, glass::MIN_OPACITY),
        0.45
    );

    // Out-of-range values are clamped before they are scaled.
    assert_eq!(
        glass::tier_alpha(GlassTier::Chrome, 9.0),
        glass::tier_alpha(GlassTier::Chrome, 1.0)
    );
}

#[test]
fn test_disabled_glass_returns_the_theme_pixels() {
    let mut settings = DesktopSettings::default();
    settings.glass_enabled = false;

    let preset: &theme::ThemePreset = &theme::THEME_PRESETS[0];
    let base = preset.card_bg();
    let border = preset.border();

    for tier in [GlassTier::Chrome, GlassTier::Overlay] {
        let style = glass::style_for(
            &settings,
            base,
            border,
            preset.is_dark,
            tier,
            Elevation::Lg,
        );
        assert!(!style.is_glassy());
        // Exactly the colours the leaf painted before the material existed.
        assert_eq!(style.fill, base);
        assert_eq!(style.border, border);
        // …and the plain elevation stack, with no rim bolted on.
        assert_eq!(style.shadows, glass::elevation_stack(Elevation::Lg));
        assert!(style.shadows.iter().all(|s| !s.inset));
    }
}

#[test]
fn test_enabled_glass_translucency_rim_and_polarity_ink() {
    let mut settings = DesktopSettings::default();
    settings.glass_enabled = true;
    settings.glass_opacity = 0.85;

    let chrome = glass::style_for(
        &settings,
        theme::THEME_PRESETS[0].card_bg(),
        theme::THEME_PRESETS[0].border(),
        true,
        GlassTier::Chrome,
        Elevation::None,
    );
    assert!(chrome.is_glassy());
    assert!((chrome.fill.a - glass::tier_alpha(GlassTier::Chrome, 0.85)).abs() < 1e-6);
    // Two inset layers on top of the (empty) elevation: top rim + bottom shade.
    assert_eq!(chrome.shadows.len(), 2);
    assert!(chrome.shadows.iter().all(|s| s.inset));
    assert_eq!(chrome.shadows[0].offset.y, gpui::px(1.0));
    assert_eq!(chrome.shadows[1].offset.y, gpui::px(-1.0));
    // The orange/red/green channels are untouched: the fill is the theme's own
    // colour, only faded.
    let base = theme::THEME_PRESETS[0].card_bg();
    assert_eq!((chrome.fill.r, chrome.fill.g, chrome.fill.b), (base.r, base.g, base.b));

    // A leaf that had elevation keeps it, plus the rim.
    let elevated = glass::style_for(
        &settings,
        base,
        theme::THEME_PRESETS[0].border(),
        true,
        GlassTier::Overlay,
        Elevation::Lg,
    );
    assert_eq!(elevated.shadows.len(), glass::elevation_stack(Elevation::Lg).len() + 2);
    assert!(elevated.shadows.iter().filter(|s| !s.inset).count() == 2);
}

#[test]
fn test_border_and_rim_ink_follow_theme_polarity() {
    // A white hairline reads as a lifted edge on dark fills, a black one as
    // definition on light fills.
    let dark_border = glass::border_ink(true);
    let light_border = glass::border_ink(false);
    assert_eq!(dark_border.a, 0.10);
    assert_eq!(light_border.a, 0.08);
    assert!(dark_border.r > light_border.r);

    // The top rim is a sheen: strong on light themes (where the fill is nearly
    // white anyway) and barely-there on dark ones.
    assert!(glass::rim_ink(false).a > glass::rim_ink(true).a);
    // The bottom shade is the opposite.
    assert!(glass::shade_ink(true).a > glass::shade_ink(false).a);
}

#[test]
fn test_app_state_glass_style_tracks_the_setting_and_the_theme() {
    // Default settings ship glass on, so the chrome starts translucent.
    let mut settings = DesktopSettings::default();
    settings.theme = SettingsTheme::Dark;
    settings.theme_preset = "default-dark".to_string();
    let app = AppState::new(settings.clone(), None);
    assert!(app.glass_enabled());

    let style = app.glass_style(app.card_bg(), GlassTier::Chrome, Elevation::None);
    assert!(style.is_glassy());
    assert!(style.fill.a < 1.0);
    assert_eq!(style.fill.r, app.card_bg().r);

    // Switching the material off returns the theme's own opaque colours…
    let mut off = settings;
    off.glass_enabled = false;
    let app_off = AppState::new(off, None);
    let plain = app_off.glass_style(app_off.bg_color(), GlassTier::Chrome, Elevation::None);
    assert!(!plain.is_glassy());
    assert_eq!(plain.fill, app_off.bg_color());
    assert_eq!(plain.fill.a, 1.0);
    assert_eq!(plain.border, app_off.border_color());

    // …while a light preset keeps a light fill rather than a fixed tint.
    let mut light = DesktopSettings::default();
    light.theme = SettingsTheme::Light;
    light.theme_preset = "default-light".to_string();
    let app_light = AppState::new(light, None);
    assert!(!app_light.is_dark());
    let light_style =
        app_light.glass_style(app_light.card_bg(), GlassTier::Chrome, Elevation::None);
    // Still a near-white fill — only its alpha changed, so the preset's hue
    // survives the material instead of being replaced by a fixed tint.
    assert!(light_style.fill.r > 0.9);
    assert!(light_style.fill.a < 1.0);
}

#[test]
fn test_glass_backdrop_is_opt_in_per_compositor() {
    // `backdrop_blur_available` only reports true for the Wayland compositors
    // that implement org_kde_kwin_blur; the rest of the world gets the plain
    // transparent backdrop the CSD frame already uses.
    let wayland = std::env::var_os("WAYLAND_DISPLAY");
    let desktop = std::env::var("XDG_CURRENT_DESKTOP").ok();

    std::env::remove_var("WAYLAND_DISPLAY");
    std::env::set_var("XDG_CURRENT_DESKTOP", "KDE");
    assert!(!glass::backdrop_blur_available(), "X11/KDE must not claim blur");

    if wayland.is_some() {
        std::env::set_var("WAYLAND_DISPLAY", "wayland-0");
        std::env::set_var("XDG_CURRENT_DESKTOP", "GNOME");
        assert!(!glass::backdrop_blur_available(), "GNOME has no blur protocol");
        std::env::set_var("XDG_CURRENT_DESKTOP", "Hyprland");
        assert!(glass::backdrop_blur_available());
        std::env::set_var("XDG_CURRENT_DESKTOP", "KDE");
        assert!(glass::backdrop_blur_available());
    }

    // Restore whatever the test runner had.
    match wayland {
        Some(v) => std::env::set_var("WAYLAND_DISPLAY", v),
        None => std::env::remove_var("WAYLAND_DISPLAY"),
    }
    match desktop {
        Some(v) => std::env::set_var("XDG_CURRENT_DESKTOP", v),
        None => std::env::remove_var("XDG_CURRENT_DESKTOP"),
    }
}

#[test]
fn test_glass_settings_roundtrip_and_legacy_defaults() {
    let base = temp_base("roundtrip");

    let mut settings = DesktopSettings::load_from(&base).expect("should load defaults");
    assert!(settings.glass_enabled, "glass is on by default");
    assert_eq!(settings.glass_opacity, glass::DEFAULT_OPACITY);

    settings.glass_enabled = false;
    settings.glass_opacity = 0.68;
    settings.save_to(&base).expect("should save");

    let reloaded = DesktopSettings::load_from(&base).expect("should reload");
    assert!(!reloaded.glass_enabled);
    assert_eq!(reloaded.glass_opacity, 0.68);

    // A file written before the material existed has no glass keys at all and
    // must still load — with the defaults, not with a deserialization error
    // (which would send the whole settings file to settings.json.bak).
    let path = webterm_settings::paths::settings_path_with_base(&base);
    let raw = std::fs::read_to_string(&path).expect("settings file exists");
    let mut json: serde_json::Value = serde_json::from_str(&raw).expect("valid json");
    let obj = json.as_object_mut().expect("object");
    obj.remove("glass_enabled");
    obj.remove("glass_opacity");
    std::fs::write(&path, serde_json::to_string_pretty(&json).unwrap()).expect("write legacy");

    let legacy = DesktopSettings::load_from(&base).expect("legacy file must load");
    assert!(legacy.glass_enabled);
    assert_eq!(legacy.glass_opacity, glass::DEFAULT_OPACITY);
    // The unrelated fields survived the round trip, i.e. nothing was reset.
    assert_eq!(legacy.theme, SettingsTheme::Dark);
    assert!(legacy.theme_preset == "default-dark");

    let _ = std::fs::remove_dir_all(&base);
}
