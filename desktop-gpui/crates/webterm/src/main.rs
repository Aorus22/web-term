//! WebTerm desktop client (GPUI).

use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;
use gpui::*;
use parking_lot::Mutex;
use webterm::actions;
use webterm::app_state::AppState;
use webterm::theme;
use webterm::window_state;
use webterm_settings::DesktopSettings;
use webterm_supervisor::SpawnOptions;

fn resolve_backend_path(settings: &DesktopSettings) -> PathBuf {
    if let Some(ref path) = settings.backend_path {
        return path.clone();
    }

    // Default resolution candidates
    let exe_suffix = if cfg!(windows) { ".exe" } else { "" };
    let bin_name = format!("backend{}", exe_suffix);

    let candidates = [
        PathBuf::from("test-support").join(&bin_name),
        PathBuf::from("desktop-gpui/test-support").join(&bin_name),
        PathBuf::from("../../desktop-gpui/test-support").join(&bin_name),
        PathBuf::from("../test-support").join(&bin_name),
    ];

    for c in candidates {
        if c.exists() {
            return c.canonicalize().unwrap_or(c);
        }
    }

    // Fallback path
    PathBuf::from(format!("test-support/{}", bin_name))
}

fn main() {
    // 1. Headless bootstrap: load settings, ensure encryption key, resolve paths
    let mut settings = DesktopSettings::load().unwrap_or_default();
    let encryption_key = settings.ensure_encryption_key();

    let backend_path = resolve_backend_path(&settings);
    let db_path = webterm_settings::paths::db_path();

    let spawn_opts = SpawnOptions::new(backend_path, db_path, encryption_key).ok();

    // 2. Initial window geometry restored via window_state module
    let initial_bounds = window_state::restore(&settings).unwrap_or_else(|| {
        WindowBounds::Windowed(Bounds {
            origin: Point::default(),
            size: size(px(1200.0), px(800.0)),
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
        theme::apply_theme(settings.theme, cx);
        actions::bind_tab_keys(cx);

        let window_options = WindowOptions {
            window_bounds: Some(initial_bounds),
            titlebar: Some(TitlebarOptions {
                title: Some("WebTerm Desktop".into()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let settings_for_observe = settings_arc.clone();
        let _ = cx.open_window(window_options, move |window, cx| {
            window_state::observe(window, settings_for_observe, cx);

            let app_state = cx.new(|_cx| AppState::new(settings, spawn_opts));
            app_state.update(cx, |this, cx| {
                this.start_supervisor(cx);
            });
            app_state
        });
    });
}
