//! Theme management and synchronization with gpui-component.

use gpui::App;
use gpui_component::{Theme, ThemeMode};
use webterm_settings::Theme as SettingsTheme;

/// Apply the given settings theme to the GPUI application context.
pub fn apply_theme(theme: SettingsTheme, cx: &mut App) {
    match theme {
        SettingsTheme::Light => {
            Theme::change(ThemeMode::Light, None, cx);
        }
        SettingsTheme::Dark => {
            Theme::change(ThemeMode::Dark, None, cx);
        }
        SettingsTheme::System => {
            // System resolution defaults to Dark for development environments
            Theme::change(ThemeMode::Dark, None, cx);
        }
    }
}

/// Flip between Dark and Light themes.
pub fn toggle_theme(current: SettingsTheme) -> SettingsTheme {
    match current {
        SettingsTheme::Light => SettingsTheme::Dark,
        SettingsTheme::Dark | SettingsTheme::System => SettingsTheme::Light,
    }
}
