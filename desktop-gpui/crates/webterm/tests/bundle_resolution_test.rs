use webterm::bundle::resolve_backend_path_with_current_exe;
use webterm_settings::DesktopSettings;

#[test]
fn test_bundle_resolution_adjacent_to_exe() {
    let temp_dir = tempfile::tempdir().unwrap();
    let bundle_dir = temp_dir.path().join("webterm-bundle");
    std::fs::create_dir_all(&bundle_dir).unwrap();

    let exe_suffix = if cfg!(windows) { ".exe" } else { "" };
    let app_exe = bundle_dir.join(format!("webterm{}", exe_suffix));
    let backend_exe = bundle_dir.join(format!("backend{}", exe_suffix));

    // Create dummy files mimicking bundled package
    std::fs::write(&app_exe, b"mock-app-binary").unwrap();
    std::fs::write(&backend_exe, b"mock-backend-binary").unwrap();

    let settings = DesktopSettings::default();

    // Resolution should locate backend adjacent to app_exe
    let resolved = resolve_backend_path_with_current_exe(&settings, Some(app_exe));
    assert_eq!(
        resolved.canonicalize().unwrap(),
        backend_exe.canonicalize().unwrap(),
        "Must resolve backend adjacent to running executable"
    );
}

#[test]
fn test_bundle_resolution_settings_override_takes_precedence() {
    let temp_dir = tempfile::tempdir().unwrap();
    let bundle_dir = temp_dir.path().join("webterm-bundle");
    std::fs::create_dir_all(&bundle_dir).unwrap();

    let exe_suffix = if cfg!(windows) { ".exe" } else { "" };
    let app_exe = bundle_dir.join(format!("webterm{}", exe_suffix));
    let bundled_backend = bundle_dir.join(format!("backend{}", exe_suffix));
    let custom_backend = temp_dir.path().join(format!("custom-backend{}", exe_suffix));

    std::fs::write(&app_exe, b"mock-app").unwrap();
    std::fs::write(&bundled_backend, b"mock-bundled").unwrap();
    std::fs::write(&custom_backend, b"mock-custom").unwrap();

    let mut settings = DesktopSettings::default();
    settings.backend_path = Some(custom_backend.clone());

    // Explicit override should take precedence over bundled binary
    let resolved = resolve_backend_path_with_current_exe(&settings, Some(app_exe));
    assert_eq!(resolved, custom_backend);
}
