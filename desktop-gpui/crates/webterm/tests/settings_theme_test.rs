//! Unit tests for Desktop settings, theme toggling, and live palette synchronization.

use std::path::PathBuf;
use webterm_settings::{DesktopSettings, Theme as SettingsTheme};
use webterm_terminal::ColorPalette;

#[test]
fn test_theme_toggling_logic() {
    assert_eq!(
        webterm::theme::toggle_theme(SettingsTheme::Dark),
        SettingsTheme::Light
    );
    assert_eq!(
        webterm::theme::toggle_theme(SettingsTheme::Light),
        SettingsTheme::Dark
    );
    assert_eq!(
        webterm::theme::toggle_theme(SettingsTheme::System),
        SettingsTheme::Light
    );
}

#[test]
fn test_settings_theme_and_backend_path_override_roundtrip() {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp_dir = std::env::temp_dir().join(format!("webterm_settings_test_{}", now));
    let _ = std::fs::create_dir_all(&temp_dir);

    let mut settings = DesktopSettings::load_from(&temp_dir).expect("should load initial settings");
    assert_eq!(settings.theme, SettingsTheme::Dark);
    assert!(settings.backend_path.is_none());

    // Modify theme and set backend path override
    settings.theme = SettingsTheme::Light;
    settings.backend_path = Some(PathBuf::from("C:\\custom\\webterm-backend.exe"));
    settings.save_to(&temp_dir).expect("should save settings");

    // Reload and verify persistence
    let reloaded = DesktopSettings::load_from(&temp_dir).expect("should reload settings");
    assert_eq!(reloaded.theme, SettingsTheme::Light);
    assert_eq!(
        reloaded.backend_path,
        Some(PathBuf::from("C:\\custom\\webterm-backend.exe"))
    );

    // Clear backend override
    let mut cleared = reloaded;
    cleared.backend_path = None;
    cleared
        .save_to(&temp_dir)
        .expect("should save cleared override");

    let final_check = DesktopSettings::load_from(&temp_dir).expect("should load final settings");
    assert!(final_check.backend_path.is_none());

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_terminal_palette_dark_light_separation() {
    let dark_palette = ColorPalette::dark_default();
    let light_palette = ColorPalette::light_default();

    // Foreground and background must not match between themes
    assert_ne!(dark_palette.background, light_palette.background);
    assert_ne!(dark_palette.foreground, light_palette.foreground);
    assert_ne!(dark_palette.cursor, light_palette.cursor);
    assert_ne!(dark_palette.selection, light_palette.selection);

    // Verify ANSI color slot differentiation
    assert_ne!(dark_palette.ansi[0], light_palette.ansi[0]);
}
