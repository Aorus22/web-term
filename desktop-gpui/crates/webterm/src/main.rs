//! WebTerm desktop client (GPUI).

use gpui::*;
use parking_lot::Mutex;
use std::borrow::Cow;
use std::sync::Arc;
use webterm::actions;
use webterm::app_state::AppState;
use webterm::bundle::resolve_backend_path;
use webterm::csd;
use webterm::glass;
use webterm::gtk_theme;
use webterm::theme;
use webterm::window_state;
use webterm_settings::DesktopSettings;
use webterm_supervisor::SpawnOptions;

fn main() {
    // 0. Enter Tokio runtime context so all Tokio primitives (timers, channels, reqwest)
    // work across the main GPUI thread and foreground async tasks.
    let _tokio_guard = webterm::app_state::TOKIO_RT.enter();

    // 0a. GTK has to be initialized on this (main) thread *before* any theme is
    // resolved and before any other GTK consumer. The probe reuses this to read
    // the desktop palette; when it fails the app keeps its built-in presets.
    if gtk_theme::init() {
        // Warm the cache so the first frame already carries desktop colours
        // when `Desktop (GTK)` is the persisted mode.
        let _ = gtk_theme::palette();
    }

    // 1. Headless bootstrap: load settings, ensure encryption key, resolve paths
    let mut settings = DesktopSettings::load().unwrap_or_default();
    let encryption_key = settings.ensure_encryption_key();

    let backend_path = resolve_backend_path(&settings);
    let db_path = webterm_settings::paths::db_path();

    let spawn_opts = SpawnOptions::new(backend_path, db_path, encryption_key).ok();

    // 2. Initial window geometry restored via window_state module
    let initial_bounds = window_state::restore(&settings).unwrap_or_else(|| {
        WindowBounds::Windowed(Bounds {
            origin: Point {
                x: px(window_state::DEFAULT_ORIGIN_X),
                y: px(window_state::DEFAULT_ORIGIN_Y),
            },
            size: size(
                px(window_state::DEFAULT_WIDTH as f32),
                px(window_state::DEFAULT_HEIGHT as f32),
            ),
        })
    });

    let settings_arc = Arc::new(Mutex::new(settings.clone()));

    // 3. Launch GPUI application
    Application::with_platform(gpui_platform::current_platform(false)).run(move |cx: &mut App| {
        // Register bundled monospace font asset
        let font_bytes: &'static [u8] = include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf");
        let _ = cx.text_system().add_fonts(vec![Cow::Borrowed(font_bytes)]);

        // Initialize gpui-component subsystem and keybindings
        gpui_component::init(cx);
        theme::apply_theme(settings.theme, &settings.theme_preset, cx);
        actions::bind_tab_keys(cx);

        let window_options = WindowOptions {
            window_bounds: Some(initial_bounds),
            window_min_size: Some(size(px(960.0), px(540.0))),
            // Must match StartupWMClass/Icon in dist/webterm-gpui.desktop so
            // docks (GNOME/KDE) group the window and show the app icon.
            app_id: Some("webterm-gpui".to_string()),
            // Transparent + client decorations: the root view draws its own
            // rounded floating frame (see AppState::render), giving rounded
            // corners and compositor shadow on Linux instead of a fused
            // rectangle.
            window_background: WindowBackgroundAppearance::Transparent,
            window_decorations: Some(WindowDecorations::Client),
            titlebar: Some(TitlebarOptions {
                title: Some("WebTerm Desktop".into()),
                appears_transparent: true,
                ..Default::default()
            }),
            ..Default::default()
        };

        let settings_for_observe = settings_arc.clone();
        let handle = cx.open_window(window_options, move |window, cx| {
            // X11 takes the window name from here; the CSD extents locator
            // matches on it (Wayland uses the same value via set_window_title).
            window.set_window_title(csd::MAIN_WINDOW_TITLE);
            #[cfg(target_os = "windows")]
            {
                use raw_window_handle::{HasWindowHandle, RawWindowHandle};
                if let Ok(handle) = HasWindowHandle::window_handle(window) {
                    if let RawWindowHandle::Win32(h) = handle.as_raw() {
                        let hwnd = h.hwnd.get();
                        extern "system" {
                            fn DwmSetWindowAttribute(
                                hwnd: isize,
                                dwAttribute: u32,
                                pvAttribute: *const std::ffi::c_void,
                                cbAttribute: u32,
                            ) -> i32;
                        }
                        let dark: i32 = 1;
                        unsafe {
                            DwmSetWindowAttribute(hwnd, 20, &dark as *const _ as _, 4);
                            DwmSetWindowAttribute(hwnd, 19, &dark as *const _ as _, 4);
                        }
                    }
                }
            }

            // Liquid Glass: keep the transparent backdrop the CSD frame relies
            // on, upgraded to a real frost where the compositor implements one
            // (KWin/Hyprland on Wayland). A no-op everywhere else.
            glass::apply_backdrop_material(window, settings.glass_enabled);

            window_state::observe(window, settings_for_observe, cx);

            let app_state = cx.new(|_cx| AppState::new(settings, spawn_opts));
            app_state.update(cx, |this, cx| {
                this.start_supervisor(cx);
                // ~1 s desktop-theme watcher (no-op when GTK is unavailable).
                this.watch_gtk_theme(cx);
            });
            // Input states need a Window; create them before first render.
            app_state.update(cx, |this, cx| {
                this.init_form_inputs(window, cx);
            });
            app_state
        });

        // 0b. Advertise the CSD shadow margin so mutter stops drawing its own
        // square shadow behind the rounded corners (X11 only, best-effort,
        // retried in the background until the X window exists).
        if let Ok(handle) = handle {
            let scale_factor = handle
                .update(cx, |_, window, _| window.scale_factor())
                .unwrap_or(1.0);
            csd::advertise_frame_extents(csd::MAIN_WINDOW_TITLE, scale_factor);
        }
    });
}
