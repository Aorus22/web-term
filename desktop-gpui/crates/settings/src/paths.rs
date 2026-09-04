use std::path::{Path, PathBuf};

/// Get the system default base config directory.
pub fn default_base_dir() -> PathBuf {
    dirs::config_dir().unwrap_or_else(|| PathBuf::from("."))
}

/// Get the WebTerm config directory within the given base directory.
pub fn config_dir_with_base(base: &Path) -> PathBuf {
    base.join("webterm-desktop")
}

/// Get the default WebTerm config directory.
pub fn config_dir() -> PathBuf {
    config_dir_with_base(&default_base_dir())
}

/// Get the settings.json path within the given base directory.
pub fn settings_path_with_base(base: &Path) -> PathBuf {
    config_dir_with_base(base).join("settings.json")
}

/// Get the default settings.json path.
pub fn settings_path() -> PathBuf {
    settings_path_with_base(&default_base_dir())
}

/// Get the backend SQLite DB path within the given base directory.
pub fn db_path_with_base(base: &Path) -> PathBuf {
    config_dir_with_base(base).join("webterm.db")
}

/// Get the default backend SQLite DB path.
pub fn db_path() -> PathBuf {
    db_path_with_base(&default_base_dir())
}

/// Ensure config directory exists for the given base directory.
pub fn ensure_dirs_with_base(base: &Path) -> std::io::Result<PathBuf> {
    let dir = config_dir_with_base(base);
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Ensure the default config directory exists.
pub fn ensure_dirs() -> std::io::Result<PathBuf> {
    ensure_dirs_with_base(&default_base_dir())
}
