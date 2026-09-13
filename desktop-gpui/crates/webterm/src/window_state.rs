//! Window state geometry restoration, toggle maximize/restore, and debounced persistence.

use gpui::*;
use parking_lot::Mutex;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::sync::Arc;
use webterm_settings::{DesktopSettings, WindowState};

/// Default window dimensions for balanced layout (comfortable for dual-pane SFTP and terminal, not oversized).
pub const DEFAULT_WIDTH: u32 = 1160;
pub const DEFAULT_HEIGHT: u32 = 660;

/// Default centered window origin for standard displays.
pub const DEFAULT_ORIGIN_X: f32 = 180.0;
pub const DEFAULT_ORIGIN_Y: f32 = 60.0;

/// Check if the window is currently maximized.
pub fn is_window_maximized(window: &Window) -> bool {
    #[cfg(target_os = "windows")]
    {
        if let Ok(handle) = HasWindowHandle::window_handle(window) {
            if let RawWindowHandle::Win32(h) = handle.as_raw() {
                let hwnd = h.hwnd.get();
                extern "system" {
                    fn IsZoomed(hwnd: isize) -> i32;
                }
                return unsafe { IsZoomed(hwnd) != 0 };
            }
        }
    }
    window.is_maximized()
}

/// Toggle maximize / restore.
///
/// On Windows, GPUI's native `zoom()` only calls `SW_MAXIMIZE` and fails to restore.
/// We call `ShowWindowAsync` (asynchronous to avoid re-entrant WM_SIZE / WM_PAINT paint drops)
/// and ensure DWM dark mode is active to prevent any white flash during window transitions.
/// Returns the new maximized state.
pub fn toggle_maximize(window: &mut Window) -> bool {
    #[cfg(target_os = "windows")]
    {
        if let Ok(handle) = HasWindowHandle::window_handle(window) {
            if let RawWindowHandle::Win32(h) = handle.as_raw() {
                let hwnd = h.hwnd.get();
                extern "system" {
                    fn IsZoomed(hwnd: isize) -> i32;
                    fn ShowWindowAsync(hwnd: isize, nCmdShow: i32) -> i32;
                    fn DwmSetWindowAttribute(
                        hwnd: isize,
                        dwAttribute: u32,
                        pvAttribute: *const std::ffi::c_void,
                        cbAttribute: u32,
                    ) -> i32;
                    fn RedrawWindow(
                        hwnd: isize,
                        lprcUpdate: *const std::ffi::c_void,
                        hrgnUpdate: isize,
                        flags: u32,
                    ) -> i32;
                }
                const SW_RESTORE: i32 = 9;
                const SW_MAXIMIZE: i32 = 3;
                const RDW_INVALIDATE: u32 = 0x0001;
                const RDW_UPDATENOW: u32 = 0x0100;
                const RDW_ALLCHILDREN: u32 = 0x0080;

                let is_max = unsafe { IsZoomed(hwnd) != 0 };
                let dark: i32 = 1;

                unsafe {
                    // Force dark mode frame so DWM never flashes white
                    DwmSetWindowAttribute(hwnd, 20, &dark as *const _ as _, 4);
                    DwmSetWindowAttribute(hwnd, 19, &dark as *const _ as _, 4);

                    if is_max {
                        ShowWindowAsync(hwnd, SW_RESTORE);
                    } else {
                        ShowWindowAsync(hwnd, SW_MAXIMIZE);
                    }
                    RedrawWindow(
                        hwnd,
                        std::ptr::null(),
                        0,
                        RDW_INVALIDATE | RDW_UPDATENOW | RDW_ALLCHILDREN,
                    );
                }

                window.on_next_frame(|window, _cx| {
                    window.refresh();
                });

                return !is_max;
            }
        }
    }
    let was_max = window.is_maximized();
    window.zoom_window();
    window.on_next_frame(|window, _cx| {
        window.refresh();
    });
    !was_max
}

/// Restore window geometry from persisted settings.
pub fn restore(settings: &DesktopSettings) -> Option<WindowBounds> {
    if let Some(ref state) = settings.window_state {
        // If the saved state has legacy oversized 1200x800 or tiny 940x560, reset to default 1160x660
        let is_legacy = (state.width == Some(1200)
            && (state.height == Some(800) || state.height == Some(662)))
            || (state.width == Some(940) && state.height == Some(560))
            || state.x.map(|x| x <= -1000).unwrap_or(false)
            || state.y.map(|y| y <= -1000).unwrap_or(false);

        let width = if is_legacy {
            DEFAULT_WIDTH
        } else {
            state.width.unwrap_or(DEFAULT_WIDTH).clamp(1000, 1920)
        };
        let height = if is_legacy {
            DEFAULT_HEIGHT
        } else {
            state.height.unwrap_or(DEFAULT_HEIGHT).clamp(560, 1200)
        };

        // Degenerate geometry guard: reject 0x0 or invalid sizes
        if width == 0 || height == 0 {
            return None;
        }

        let origin = match (state.x, state.y) {
            (Some(x), Some(y)) if x > 10 && y > 10 && x < 10000 && y < 10000 && !is_legacy => {
                Point {
                    x: px(x as f32),
                    y: px(y as f32),
                }
            }
            _ => Point {
                x: px(DEFAULT_ORIGIN_X),
                y: px(DEFAULT_ORIGIN_Y),
            },
        };

        let bounds = Bounds {
            origin,
            size: size(px(width as f32), px(height as f32)),
        };

        if state.maximized && !is_legacy {
            Some(WindowBounds::Maximized(bounds))
        } else {
            Some(WindowBounds::Windowed(bounds))
        }
    } else {
        None
    }
}

/// Helper to extract WindowState from current Window state.
pub fn extract_window_state(
    window: &Window,
    prev_state: Option<&WindowState>,
) -> Option<WindowState> {
    let bounds = window.bounds();
    let current_width = (bounds.size.width / px(1.0)) as u32;
    let current_height = (bounds.size.height / px(1.0)) as u32;

    // Degenerate geometry guard
    if current_width == 0 || current_height == 0 {
        return None;
    }

    let x = (bounds.origin.x / px(1.0)) as i32;
    let y = (bounds.origin.y / px(1.0)) as i32;

    // Guard against Windows minimized coordinates (-32000 or -25600)
    if x <= -10000 || y <= -10000 {
        return None;
    }

    let maximized = is_window_maximized(window);

    // If currently maximized, preserve previous windowed bounds so restoring goes back to compact size!
    let (saved_x, saved_y, saved_width, saved_height) = if maximized {
        if let Some(prev) = prev_state {
            (
                prev.x.or(Some(DEFAULT_ORIGIN_X as i32)),
                prev.y.or(Some(DEFAULT_ORIGIN_Y as i32)),
                prev.width.unwrap_or(DEFAULT_WIDTH).clamp(1000, 1920),
                prev.height.unwrap_or(DEFAULT_HEIGHT).clamp(560, 1200),
            )
        } else {
            (
                Some(DEFAULT_ORIGIN_X as i32),
                Some(DEFAULT_ORIGIN_Y as i32),
                DEFAULT_WIDTH,
                DEFAULT_HEIGHT,
            )
        }
    } else {
        (
            Some(x.max(0)),
            Some(y.max(0)),
            current_width.clamp(1000, 1920),
            current_height.clamp(560, 1200),
        )
    };

    Some(WindowState {
        x: saved_x,
        y: saved_y,
        width: Some(saved_width),
        height: Some(saved_height),
        maximized,
    })
}

/// Observe window geometry changes and debounced-save to settings.
pub fn observe(window: &mut Window, settings: Arc<Mutex<DesktopSettings>>, cx: &mut App) {
    let last_saved = Arc::new(Mutex::new(settings.lock().window_state));
    let last_saved_close = last_saved.clone();
    let settings_close = settings.clone();

    // Flush-save on window close
    window.on_window_should_close(cx, move |window, _cx| {
        let prev = last_saved_close.lock().clone();
        if let Some(new_state) = extract_window_state(window, prev.as_ref()) {
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
