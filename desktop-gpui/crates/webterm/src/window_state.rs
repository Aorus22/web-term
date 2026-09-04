//! Window state geometry restoration and debounced persistence.

use std::sync::Arc;
use std::time::Duration;
use gpui::*;
use parking_lot::Mutex;
use webterm_settings::{DesktopSettings, WindowState};

/// Debounce interval for window geometry saves.
pub const DEBOUNCE_INTERVAL: Duration = Duration::from_millis(1000);

/// Restore window geometry from persisted settings.
pub fn restore(settings: &DesktopSettings) -> Option<WindowBounds> {
    if let Some(ref state) = settings.window_state {
        let width = state.width.unwrap_or(1200);
        let height = state.height.unwrap_or(800);

        // Degenerate geometry guard: reject 0x0 or invalid sizes
        if width == 0 || height == 0 {
            return None;
        }

        let origin = match (state.x, state.y) {
            (Some(x), Some(y)) => Point {
                x: px(x as f32),
                y: px(y as f32),
            },
            _ => Point::default(),
        };

        let bounds = Bounds {
            origin,
            size: size(px(width as f32), px(height as f32)),
        };

        if state.maximized {
            Some(WindowBounds::Maximized(bounds))
        } else {
            Some(WindowBounds::Windowed(bounds))
        }
    } else {
        None
    }
}

/// Helper to extract WindowState from current Window state.
pub fn extract_window_state(window: &Window) -> Option<WindowState> {
    let bounds = window.bounds();
    let width = bounds.size.width.0 as u32;
    let height = bounds.size.height.0 as u32;

    // Degenerate geometry guard
    if width == 0 || height == 0 {
        return None;
    }

    let x = bounds.origin.x.0 as i32;
    let y = bounds.origin.y.0 as i32;
    let maximized = window.is_maximized();

    Some(WindowState {
        x: Some(x),
        y: Some(y),
        width: Some(width),
        height: Some(height),
        maximized,
    })
}

/// Observe window geometry changes and debounced-save to settings.
pub fn observe(
    window: &mut Window,
    settings: Arc<Mutex<DesktopSettings>>,
    cx: &mut App,
) {
    let last_saved = Arc::new(Mutex::new(settings.lock().window_state));
    let last_saved_close = last_saved.clone();
    let settings_close = settings.clone();

    // Flush-save on window close
    window.on_window_should_close(cx, move |window, _cx| {
        if let Some(new_state) = extract_window_state(window) {
            let mut last = last_saved_close.lock();
            if *last != Some(new_state) {
                *last = Some(new_state);
                let mut set = settings_close.lock();
                set.window_state = Some(new_state);
                let _ = set.save();
            }
        }
        true
    });
}
