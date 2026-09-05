//! WebTerm desktop client (GPUI).

use std::borrow::Cow;
use std::sync::Arc;
use gpui::*;
use parking_lot::Mutex;
use webterm::actions;
use webterm::app_state::AppState;
use webterm::bundle::resolve_backend_path;
use webterm::theme;
use webterm::window_state;
use webterm_settings::DesktopSettings;
use webterm_supervisor::SpawnOptions;

fn main() {
    // 0. Enter Tokio runtime context so all Tokio primitives (timers, channels, reqwest)
    // work across the main GPUI thread and foreground async tasks.
    let _tokio_guard = webterm::app_state::TOKIO_RT.enter();

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
