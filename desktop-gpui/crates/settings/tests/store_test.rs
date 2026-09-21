use std::fs;
use std::path::PathBuf;
use webterm_settings::paths;
use webterm_settings::{is_valid_64_hex, DesktopSettings, GlassBackdrop, Theme, WindowState};

#[test]
fn test_roundtrip_persist() {
    let temp = tempfile::tempdir().unwrap();
    let base = temp.path();

    let mut settings = DesktopSettings::load_from(base).unwrap();
    settings.backend_path = Some(PathBuf::from("/usr/local/bin/webterm-backend"));
    settings.theme = Theme::Light;
    settings.window_state = Some(WindowState {
        x: Some(100),
        y: Some(120),
        width: Some(1400),
        height: Some(900),
        maximized: false,
    });
    settings.last_backend_url = Some("http://127.0.0.1:54321".to_string());
    settings.save().unwrap();

    let loaded = DesktopSettings::load_from(base).unwrap();
    assert_eq!(loaded.backend_path, settings.backend_path);
    assert_eq!(loaded.theme, Theme::Light);
    assert_eq!(loaded.window_state, settings.window_state);
    assert_eq!(loaded.last_backend_url, settings.last_backend_url);
}

#[test]
fn test_corrupt_file_recovery() {
    let temp = tempfile::tempdir().unwrap();
    let base = temp.path();

    paths::ensure_dirs_with_base(base).unwrap();
    let settings_file = paths::settings_path_with_base(base);
    fs::write(&settings_file, "{ corrupted json syntax ").unwrap();

    // Loading should not crash; it should rename to .bak and return defaults
    let loaded = DesktopSettings::load_from(base).unwrap();
    assert_eq!(loaded.theme, Theme::Dark);
    assert!(loaded.encryption_key.is_none());

    let bak_file = settings_file.with_extension("json.bak");
    assert!(bak_file.exists(), "corrupt file should be preserved as settings.json.bak");
}

#[test]
fn test_glass_fields_roundtrip() {
    let temp = tempfile::tempdir().unwrap();
    let base = temp.path();

    let mut settings = DesktopSettings::load_from(base).unwrap();
    // Defaults must be the glass look the material shipped with.
    assert!(settings.glass_enabled);
    assert_eq!(settings.glass_backdrop, GlassBackdrop::Blurred);

    settings.glass_enabled = false;
    settings.glass_opacity = 0.68;
    settings.glass_backdrop = GlassBackdrop::Translucent;
    settings.save().unwrap();

    let loaded = DesktopSettings::load_from(base).unwrap();
    assert!(!loaded.glass_enabled);
    assert_eq!(loaded.glass_opacity, 0.68);
    assert_eq!(loaded.glass_backdrop, GlassBackdrop::Translucent);
}

#[test]
fn test_settings_without_glass_fields_keep_loading() {
    let temp = tempfile::tempdir().unwrap();
    let base = temp.path();

    paths::ensure_dirs_with_base(base).unwrap();
    let settings_file = paths::settings_path_with_base(base);
    // A file written before the glass material existed: no glass keys at all.
    fs::write(
        &settings_file,
        r#"{"theme":"light","font_size":13.0,"scrollback":4000}"#,
    )
    .unwrap();

    let loaded = DesktopSettings::load_from(base).unwrap();
    assert_eq!(loaded.theme, Theme::Light);
    assert_eq!(loaded.scrollback, 4000);
    assert!(loaded.glass_enabled, "missing key must fall back to the glass default");
    assert_eq!(loaded.glass_opacity, 0.85);
    assert_eq!(loaded.glass_backdrop, GlassBackdrop::Blurred);
    assert!(
        !settings_file.with_extension("json.bak").exists(),
        "a pre-glass file is not corrupt and must not be quarantined"
    );
}

#[test]
fn test_backdrop_serialises_by_name() {
    // The value in settings.json is user-visible and hand-editable, so it must
    // stay a name rather than a number across serde changes.
    let json = serde_json::to_string(&GlassBackdrop::Translucent).unwrap();
    assert_eq!(json, "\"translucent\"");
    let parsed: GlassBackdrop = serde_json::from_str("\"blurred\"").unwrap();
    assert_eq!(parsed, GlassBackdrop::Blurred);
}

#[test]
fn test_key_stability_and_format() {
    let temp = tempfile::tempdir().unwrap();
    let base = temp.path();

    let mut settings = DesktopSettings::load_from(base).unwrap();
    let key1 = settings.ensure_encryption_key();

    // Assert key format is 64 lowercase hex characters
    assert!(is_valid_64_hex(&key1), "key must be 64 lowercase hex characters: {}", key1);

    // Consecutive call on same instance
    let key2 = settings.ensure_encryption_key();
    assert_eq!(key1, key2, "consecutive call on same instance must return equal key");

    // Load from disk anew
    let mut reloaded = DesktopSettings::load_from(base).unwrap();
    let key3 = reloaded.ensure_encryption_key();
    assert_eq!(key1, key3, "reloaded store must preserve stable encryption key");
}
