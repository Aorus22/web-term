//! New Tab launcher modal for Local Shell and Quick SSH connection.

use gpui::*;
use webterm_settings::Theme as SettingsTheme;
use crate::app_state::AppState;

/// Renders the modal overlay allowing user to spawn a Local Shell or Quick SSH session.
pub fn render_new_tab_modal(app: &mut AppState, cx: &mut Context<AppState>) -> AnyElement {
    let is_dark = app.theme != SettingsTheme::Light;
    let card_bg = if is_dark { rgb(0x27272a) } else { rgb(0xffffff) };
    let border_color = if is_dark { rgb(0x3f3f46) } else { rgb(0xe2e8f0) };
    let text_color = if is_dark { rgb(0xf4f4f5) } else { rgb(0x0f172a) };
    let muted_text = if is_dark { rgb(0xa1a1aa) } else { rgb(0x64748b) };
    let input_bg = if is_dark { rgb(0x18181b) } else { rgb(0xf8fafc) };

    let host_display: SharedString = if app.new_tab_host.is_empty() {
        "e.g. 192.168.1.100 or server.com".into()
    } else {
        app.new_tab_host.clone().into()
    };
    let port_display: SharedString = app.new_tab_port.clone().into();
    let user_display: SharedString = if app.new_tab_user.is_empty() {
        "root".into()
    } else {
        app.new_tab_user.clone().into()
    };

    div()
        .absolute()
        .inset_0()
        .flex()
        .items_center()
        .justify_center()
        .bg(rgba(0x00000088))
        // Dismiss when clicking outside modal card
        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
            this.close_new_tab_modal(cx);
        }))
        // Modal card (stops propagation)
        .child(
            div()
                .flex()
                .flex_col()
                .w(px(480.0))
                .rounded_xl()
                .bg(card_bg)
                .border_1()
                .border_color(border_color)
                .shadow_lg()
                .p_6()
                .gap_4()
                .on_mouse_down(MouseButton::Left, |_, _, _| {}) // stop propagation
                // Header
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .text_lg()
                                .font_weight(FontWeight::BOLD)
                                .text_color(text_color)
                                .child("New Terminal Tab"),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_center()
                                .size(px(24.0))
                                .rounded_md()
                                .hover(|s| s.bg(if is_dark { rgb(0x3f3f46) } else { rgb(0xe2e8f0) }))
                                .cursor_pointer()
                                .text_color(muted_text)
                                .child(svg().data(crate::icons::X_SVG).size(px(14.0)).text_color(muted_text))
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                    this.close_new_tab_modal(cx);
                                })),
                        ),
                )
                // Option 1: First-class Local Shell
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .p_4()
                        .rounded_lg()
                        .bg(if is_dark { rgb(0x18181b) } else { rgb(0xf1f5f9) })
                        .border_1()
                        .border_color(if is_dark { rgb(0x38bdf8) } else { rgb(0x0284c7) })
                        .hover(|s| s.bg(if is_dark { rgb(0x27272a) } else { rgb(0xe2e8f0) }))
                        .cursor_pointer()
                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                            this.close_new_tab_modal(cx);
                            this.open_local_tab(cx);
                        }))
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .child(
                                            svg()
                                                .data(crate::icons::TERMINAL_SVG)
                                                .size(px(18.0))
                                                .text_color(if is_dark { rgb(0x38bdf8) } else { rgb(0x0284c7) }),
                                        )
                                        .child(
                                            div()
                                                .text_base()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(text_color)
                                                .child("Local Shell"),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .px_1p5()
                                                .py_0p5()
                                                .rounded_md()
                                                .bg(if is_dark { rgb(0x0369a1) } else { rgb(0xe0f2fe) })
                                                .text_color(if is_dark { rgb(0xe0f2fe) } else { rgb(0x0369a1) })
                                                .child(if cfg!(windows) { "ConPTY" } else { "POSIX PTY" }),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap_1()
                                        .text_xs()
                                        .text_color(if is_dark { rgb(0x38bdf8) } else { rgb(0x0284c7) })
                                        .child("Launch")
                                        .child(
                                            svg()
                                                .data(crate::icons::ARROW_RIGHT_SVG)
                                                .size(px(12.0))
                                                .text_color(if is_dark { rgb(0x38bdf8) } else { rgb(0x0284c7) }),
                                        ),
                                ),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(muted_text)
                                .child("Launch an interactive shell session directly on your host machine"),
                        ),
                )
                // Divider
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .py_1()
                        .child(div().flex_1().h(px(1.0)).bg(border_color))
                        .child(div().text_xs().text_color(muted_text).child("OR QUICK SSH"))
                        .child(div().flex_1().h(px(1.0)).bg(border_color)),
                )
                // Option 2: Quick SSH Connection Form
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        // Host & Port row
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .gap_2()
                                .child(
                                    div()
                                        .flex_1()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().text_xs().text_color(muted_text).child("Host / IP"))
                                        .child(
                                            div()
                                                .px_3()
                                                .py_1()
                                                .rounded_md()
                                                .bg(input_bg)
                                                .border_1()
                                                .border_color(border_color)
                                                .text_sm()
                                                .text_color(text_color)
                                                .child(host_display),
                                        ),
                                )
                                .child(
                                    div()
                                        .w(px(70.0))
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().text_xs().text_color(muted_text).child("Port"))
                                        .child(
                                            div()
                                                .px_3()
                                                .py_1()
                                                .rounded_md()
                                                .bg(input_bg)
                                                .border_1()
                                                .border_color(border_color)
                                                .text_sm()
                                                .text_color(text_color)
                                                .child(port_display),
                                        ),
                                ),
                        )
                        // User row
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(div().text_xs().text_color(muted_text).child("Username"))
                                .child(
                                    div()
                                        .px_3()
                                        .py_1()
                                        .rounded_md()
                                        .bg(input_bg)
                                        .border_1()
                                        .border_color(border_color)
                                        .text_sm()
                                        .text_color(text_color)
                                        .child(user_display),
                                ),
                        )
                        // Connect button
                        .child(
                            div()
                                .mt_2()
                                .flex()
                                .flex_row()
                                .justify_end()
                                .gap_2()
                                .child(
                                    div()
                                        .px_4()
                                        .py_2()
                                        .rounded_md()
                                        .bg(if is_dark { rgb(0x3f3f46) } else { rgb(0xe2e8f0) })
                                        .hover(|s| s.bg(if is_dark { rgb(0x52525b) } else { rgb(0xcbd5e1) }))
                                        .cursor_pointer()
                                        .text_sm()
                                        .text_color(text_color)
                                        .child("Cancel")
                                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                            this.close_new_tab_modal(cx);
                                        })),
                                )
                                .child(
                                    div()
                                        .px_4()
                                        .py_2()
                                        .rounded_md()
                                        .bg(if is_dark { rgb(0x0284c7) } else { rgb(0x0ea5e9) })
                                        .hover(|s| s.bg(if is_dark { rgb(0x0369a1) } else { rgb(0x0284c7) }))
                                        .cursor_pointer()
                                        .text_sm()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(rgb(0xffffff))
                                        .child("Connect SSH")
                                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                            this.open_quick_ssh_tab(cx);
                                        })),
                                ),
                        ),
                ),
        )
        .into_any_element()
}
