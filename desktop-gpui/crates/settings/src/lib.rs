//! Desktop settings store.
//!
//! Holds **UI preferences only** (backend path, encryption-key custody, theme,
//! window state). SSH/host/key data stays in the backend's SQLite — established
//! architecture decision (20-RESEARCH §4).

pub mod paths;

use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Invalid encryption key format: {0}")]
    InvalidKey(String),
}

/// UI theme preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    Dark,
    Light,
    System,
}

/// Window dimensions and layout state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WindowState {
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub maximized: bool,
}

/// Persisted session tab state to survive app restart.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedSessionTab {
    pub session_id: String,
    pub title: String,
    pub session_type: String, // "local" | "ssh"
    pub connection_id: Option<String>,
}

fn default_theme_preset() -> String {
    "default-dark".to_string()
}

fn default_theme_mode_filter() -> String {
    "all".to_string()
}

fn default_font_family() -> String {
    "Geist Mono".to_string()
}

fn default_font_size() -> f32 {
    14.0
}

fn default_cursor_style() -> String {
    "block".to_string()
}

fn default_cursor_blink() -> bool {
    true
}

fn default_scrollback() -> u32 {
    1000
}

fn default_glass_enabled() -> bool {
    true
}

/// Alpha of the Liquid Glass fill on overlay surfaces (sheets, dialogs,
/// toasts, menus). Higher is more opaque and reads as less glassy; the chrome
/// tier derives a more transparent value from it. Clamped again at use site —
/// a hand-edited settings file must not be able to make text unreadable.
fn default_glass_opacity() -> f32 {
    0.85
}

/// Desktop settings store holding UI preferences and encryption key custody.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DesktopSettings {
    #[serde(default)]
    pub backend_path: Option<PathBuf>,
    #[serde(default)]
    pub encryption_key: Option<String>,
    #[serde(default)]
    pub theme: Theme,
    #[serde(default = "default_theme_preset")]
    pub theme_preset: String,
    #[serde(default = "default_theme_mode_filter")]
    pub theme_mode_filter: String,
    #[serde(default = "default_font_family")]
    pub font_family: String,
    #[serde(default = "default_font_size")]
    pub font_size: f32,
    #[serde(default = "default_cursor_style")]
    pub cursor_style: String,
    #[serde(default = "default_cursor_blink")]
    pub cursor_blink: bool,
    #[serde(default = "default_scrollback")]
    pub scrollback: u32,
    /// Liquid Glass window chrome. Fields carry serde defaults so settings
    /// files written before the material existed keep loading.
    #[serde(default = "default_glass_enabled")]
    pub glass_enabled: bool,
    #[serde(default = "default_glass_opacity")]
    pub glass_opacity: f32,
    #[serde(default)]
    pub window_state: Option<WindowState>,
    #[serde(default)]
    pub last_backend_url: Option<String>,
    #[serde(default)]
    pub open_sessions: Vec<SavedSessionTab>,

    #[serde(skip)]
    pub custom_base: Option<PathBuf>,
}

impl Default for DesktopSettings {
    fn default() -> Self {
        Self {
            backend_path: None,
            encryption_key: None,
            theme: Theme::Dark,
            theme_preset: default_theme_preset(),
            theme_mode_filter: default_theme_mode_filter(),
            font_family: default_font_family(),
            font_size: default_font_size(),
            cursor_style: default_cursor_style(),
            cursor_blink: default_cursor_blink(),
            scrollback: default_scrollback(),
            glass_enabled: default_glass_enabled(),
            glass_opacity: default_glass_opacity(),
            window_state: None,
            last_backend_url: None,
            open_sessions: Vec::new(),
            custom_base: None,
        }
    }
}

impl DesktopSettings {
    /// Load settings from the default config directory.
    pub fn load() -> Result<Self, SettingsError> {
        let base = paths::default_base_dir();
        Self::load_from(&base)
    }

    /// Load settings using an injectable base directory (for testing).
    pub fn load_from(base: &Path) -> Result<Self, SettingsError> {
        let file_path = paths::settings_path_with_base(base);
        if !file_path.exists() {
            let settings = Self {
                custom_base: Some(base.to_path_buf()),
                ..Default::default()
            };
            return Ok(settings);
        }

        let content = match fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(e) => return Err(SettingsError::Io(e)),
        };

        match serde_json::from_str::<DesktopSettings>(&content) {
            Ok(mut settings) => {
                settings.custom_base = Some(base.to_path_buf());
                Ok(settings)
            }
            Err(_) => {
                // Corrupt file recovery: rename to settings.json.bak and regenerate defaults
                let bak_path = file_path.with_extension("json.bak");
                let _ = fs::rename(&file_path, &bak_path);

                let settings = Self {
                    custom_base: Some(base.to_path_buf()),
                    ..Default::default()
                };
                let _ = settings.save_to(base);
                Ok(settings)
            }
        }
    }

    /// Save settings to current base directory (or default).
    pub fn save(&self) -> Result<(), SettingsError> {
        if let Some(ref base) = self.custom_base {
            self.save_to(base)
        } else {
            self.save_to(&paths::default_base_dir())
        }
    }

    /// Save settings using an injectable base directory.
    pub fn save_to(&self, base: &Path) -> Result<(), SettingsError> {
        paths::ensure_dirs_with_base(base)?;
        let file_path = paths::settings_path_with_base(base);
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&file_path, content)?;

        // Set user-only permissions (0600) on Unix.
        // On Windows, the user's %APPDATA% profile directory ACL provides user-only privacy.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = fs::metadata(&file_path) {
                let mut perms = metadata.permissions();
                perms.set_mode(0o600);
                let _ = fs::set_permissions(&file_path, perms);
            }
        }

        Ok(())
    }

    /// Retrieve the stable encryption key, or generate a 32-byte (64 lowercase hex) key,
    /// persist immediately, and return it.
    pub fn ensure_encryption_key(&mut self) -> String {
        if let Some(ref key) = self.encryption_key {
            if is_valid_64_hex(key) {
                return key.clone();
            }
        }

        // Generate 32 random bytes -> 64 lowercase hex characters
        let mut bytes = [0u8; 32];
        for b in &mut bytes {
            *b = rand::random::<u8>();
        }
        let hex_key = bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>();
        self.encryption_key = Some(hex_key.clone());
        let _ = self.save();
        hex_key
    }
}

/// Validate 64 lowercase hex characters.
pub fn is_valid_64_hex(key: &str) -> bool {
    key.len() == 64 && key.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
}
