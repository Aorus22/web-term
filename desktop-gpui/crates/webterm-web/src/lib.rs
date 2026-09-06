#![cfg(target_arch = "wasm32")]

use std::borrow::Cow;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use gpui_kit::prelude::*;
use gpui_kit::*;

thread_local! {
    static APPLICATION: RefCell<Option<ApplicationHandle>> = const { RefCell::new(None) };
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Info);

    gpui_kit::platform::web_init();
    let app = gpui_kit::platform::single_threaded_web();

    let launch = move |cx: &mut App| {
        // Bundled fonts
        let ibm_plex = Cow::Borrowed(include_bytes!("../fonts/IBMPlexSans-Regular.ttf").as_slice());
        let inter = Cow::Borrowed(include_bytes!("../fonts/Inter-Regular.ttf").as_slice());
        let mono = Cow::Borrowed(include_bytes!("../fonts/JetBrainsMono-Regular.ttf").as_slice());

        cx.text_system()
            .add_fonts(vec![ibm_plex, inter, mono])
            .expect("Failed to load fonts");

        gpui_component::init(cx);
        webterm_core::actions::bind_tab_keys(cx);

        let window_options = WindowOptions {
            titlebar: Some(TitlebarOptions {
                title: Some("WebTerm (GPUI Web)".into()),
                appears_transparent: true,
                ..Default::default()
            }),
            ..Default::default()
        };

        cx.open_window(window_options, |_window, cx| {
            let app_state = cx.new(|_cx| webterm_core::app_state::AppState::new());
            app_state.update(cx, |this, cx| {
                webterm_ui::theme::apply_theme(this.theme, cx);
                this.fetch_connections(cx);
                this.fetch_ssh_keys(cx);
                this.fetch_port_forwards(cx);
            });
            cx.new(|_cx| WebTermRoot { app: app_state })
        })
        .expect("Failed to open window");

        cx.activate(true);
    };

    APPLICATION.with(|application| {
        *application.borrow_mut() = Some(app.run_embedded(launch));
    });

    Ok(())
}

pub struct WebTermRoot {
    pub app: Entity<webterm_core::app_state::AppState>,
}

impl Render for WebTermRoot {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.app.update(cx, |this, cx| {
            webterm_ui::views::nav::render_nav_shell(this, cx)
        })
    }
}
