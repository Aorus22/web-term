//! Field diagnostics log for the terminal crate.
//!
//! Append-only, size-capped log file so a user can reproduce an issue
//! (e.g. scrolling does nothing) and share what actually happened inside
//! the app. Disabled entirely with `WEBTERM_NO_DEBUG_LOG=1`.
//!
//! Location: `$HOME/.local/share/webterm-gpui/debug.log`
//! (falls back to the system temp dir when HOME is unavailable).

use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::SystemTime;

/// Max log size before the file is truncated and started fresh.
const MAX_BYTES: u64 = 512 * 1024;

static WRITE_LOCK: Mutex<()> = Mutex::new(());

fn log_path() -> PathBuf {
    let base = std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir())
        .join(".local")
        .join("share")
        .join("webterm-gpui");
    let _ = std::fs::create_dir_all(&base);
    base.join("debug.log")
}

/// Append one line to the debug log. Cheap; failures are silently ignored so
/// diagnostics can never break the app.
pub fn debug_log(tag: &str, msg: impl std::fmt::Display) {
    if std::env::var("WEBTERM_NO_DEBUG_LOG").is_ok() {
        return;
    }
    let Ok(_guard) = WRITE_LOCK.lock() else {
        return;
    };
    let path = log_path();
    if let Ok(meta) = std::fs::metadata(&path) {
        if meta.len() > MAX_BYTES {
            let _ = std::fs::remove_file(&path);
        }
    }
    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let _ = writeln!(file, "[{secs}][{tag}] {msg}");
    }
}
