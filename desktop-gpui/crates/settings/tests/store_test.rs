use std::fs;
use std::path::PathBuf;
use webterm_settings::paths;
use webterm_settings::{is_valid_64_hex, DesktopSettings, Theme, WindowState};

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
