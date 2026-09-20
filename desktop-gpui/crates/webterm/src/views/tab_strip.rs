//! Terminal Tab Strip UI component: frameless titlebar with sidebar toggle, active session tabs, new tab button, draggable region, and custom window controls.

use crate::app_state::{AppState, View};
use crate::glass::{Elevation, GlassTier};
use crate::icons::{
    COPY_SVG, MAXIMIZE_SVG, MINIMIZE_SVG, PANEL_LEFT_SVG, PLUS_SVG, RESTORE_SVG, X_SVG,
};
use crate::session::SessionStatus;
use gpui::prelude::FluentBuilder as _;
use gpui::*;

/// Renders the horizontal top bar containing sidebar toggle, open terminal tabs, draggable titlebar area, and custom window controls.
///
/// `framed` rounds the top corners (Zed-style titlebar): this bar owns them
/// since it floats over the full width, including above the sidebar.
pub fn render_tab_strip(
    app: &mut AppState,
    framed: bool,
    cx: &mut Context<AppState>,
) -> impl IntoElement {
    let active_index = app.session_manager.active_index();
    let border_color = app.border_color();
    // The bar floats over the desktop (the window background is transparent),
    // so it paints glass; the active tab gets the denser overlay tier so the
    // selected pill still separates from the bar it sits on.
    let bar_glass = app.glass_style(app.bg_color(), GlassTier::Chrome, Elevation::None);
    let tab_glass = app.glass_style(app.card_bg(), GlassTier::Overlay, Elevation::None);
    let muted_text = app.muted_text();

    let tabs_len = app.session_manager.tab_count();
    let hover_bg = app.accent_color();

    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .h(px(40.0))
        .bg(bar_glass.fill)
        .border_b_1()
        .border_color(bar_glass.border)
        .shadow(bar_glass.shadows)
        .pl_2()
        .pr_0()
        .gap_2()
        .when(framed, |d| {
            d.rounded_tl(crate::app_state::FRAME_ROUNDING)
                .rounded_tr(crate::app_state::FRAME_ROUNDING)
        })
        // Left: Sidebar Toggle Button (PanelLeft)
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .size(px(28.0))
                .rounded_md()
                .cursor_pointer()
                .id("tab_strip-01").hover(|s| s.bg(hover_bg))
                .child(
                    svg()
                        .data(PANEL_LEFT_SVG)
                        .size(px(16.0))
                        .text_color(muted_text),
                )
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _, _window, cx| {
                        this.toggle_sidebar(cx);
                    }),
                ),
        )
        // Tabs container: shrinkable scroll strip so many tabs never push
        // the window controls. Each tab is a fixed 160px (title truncated)
        // so all tabs render identically regardless of title length.
        .child(
            div()
                .id("tab-strip-tabs")
                .flex()
                .flex_row()
                .items_center()
                .flex_shrink_1()
                .min_w_0()
                .overflow_x_scroll()
                .gap_1p5()
                .children((0..tabs_len).map(|idx| {
            let is_active =
                !app.show_hosts_catalog && app.active_view == View::Hosts && idx == active_index;
            let tab = &app.session_manager.tabs()[idx];
            let title = tab.title.clone();
            let status = tab.status.clone();

            let dot_color = match &status {
                SessionStatus::Connected => rgb(0x22c55e), // Green
                SessionStatus::Connecting | SessionStatus::Reconnecting { .. } => rgb(0xeab308), // Yellow
                SessionStatus::Disconnected(_) => rgb(0xef4444), // Red
            };

            let tab_bg = if is_active {
                tab_glass.fill
            } else {
                rgba(0x00000000)
            };

            let tab_border = if is_active {
                tab_glass.border
            } else {
                rgba(0x00000000)
            };

            let text_color = if is_active {
                app.text_color()
            } else {
                muted_text
            };

            div()
                .flex()
                .flex_row()
                .items_center()
                .h(px(28.0))
                .w(px(160.0))
                .flex_shrink_0()
                .px_2p5()
                .gap_2()
                .rounded_md()
                .bg(tab_bg)
                .border_1()
                .border_color(tab_border)
                .cursor_pointer()
                .id(ElementId::NamedInteger("tab".into(), idx as u64)).hover(|s| s.bg(if is_active { tab_bg } else { hover_bg }))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, _window, cx| {
                        this.show_hosts_catalog = false;
                        this.active_view = View::Hosts;
                        this.switch_tab(idx, cx);
                    }),
                )
                // Status dot
                .child(div().size(px(6.0)).flex_shrink_0().rounded_full().bg(dot_color))
                // Tab title (truncated: fixed-width tabs stay identical)
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .truncate()
                        .text_xs()
                        .font_weight(if is_active {
                            FontWeight::SEMIBOLD
                        } else {
                            FontWeight::NORMAL
                        })
                        .text_color(text_color)
                        .child(title),
                )
                // Close button ('×')
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .size(px(16.0))
                        .flex_shrink_0()
                        .rounded_sm()
                        .id("tab_strip-03").hover(|s| s.bg(border_color))
                        .child(svg().data(X_SVG).size(px(10.0)).text_color(muted_text))
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, _window, cx| {
                                cx.stop_propagation();
                                this.close_tab(idx, cx);
                            }),
                        ),
                )
        })))
        // '+' New Tab button with Popover
        .child(
            div()
                .relative()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .size(px(28.0))
                        .rounded_md()
                        .cursor_pointer()
                        .id("tab_strip-04").hover(|s| s.bg(hover_bg))
                        .child(svg().data(PLUS_SVG).size(px(14.0)).text_color(muted_text))
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, _, window, cx| {
                                let has_active_session = this.session_manager.tab_count() > 0
                                    && this.active_view == View::Hosts
                                    && !this.show_hosts_catalog;
                                if has_active_session {
                                    this.toggle_new_tab_popover(cx);
                                } else {
                                    this.open_new_tab_page(window, cx);
                                }
                            }),
                        ),
                )
                // Popover dropdown menu
                .children(if app.show_new_tab_popover {
                    let popover_glass =
                        app.glass_style(app.card_bg(), GlassTier::Overlay, Elevation::Lg);
                    Some(
                        div()
                            .absolute()
                            .top(px(32.0))
                            .left(px(0.0))
                            .w(px(220.0))
                            .p_1p5()
                            .rounded_xl()
                            .bg(popover_glass.fill)
                            .border_1()
                            .border_color(popover_glass.border)
                            .shadow(popover_glass.shadows)
                            .gap_1()
                            .flex()
                            .flex_col()
                            .on_mouse_down(MouseButton::Left, |_, _, _| {}) // stop propagation
                            // Option 1: Duplicate
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_3()
                                    .px_3()
                                    .py_2()
                                    .rounded_lg()
                                    .cursor_pointer()
                                    .id("tab_strip-05").hover(|s| s.bg(hover_bg))
                                    .child(
                                        svg().data(COPY_SVG).size(px(18.0)).text_color(muted_text),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(FontWeight::SEMIBOLD)
                                                    .text_color(app.text_color())
                                                    .child("Duplicate"),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(muted_text)
                                                    .child("Same connection & directory"),
                                            ),
                                    )
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _window, cx| {
                                            this.duplicate_active_tab(cx);
                                        }),
                                    ),
                            )
                            // Option 2: New Connection
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_3()
                                    .px_3()
                                    .py_2()
                                    .rounded_lg()
                                    .cursor_pointer()
                                    .id("tab_strip-06").hover(|s| s.bg(hover_bg))
                                    .child(
                                        svg().data(PLUS_SVG).size(px(18.0)).text_color(muted_text),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(FontWeight::SEMIBOLD)
                                                    .text_color(app.text_color())
                                                    .child("New Connection"),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(muted_text)
                                                    .child("Connect to a server"),
                                            ),
                                    )
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, window, cx| {
                                            this.open_new_tab_page(window, cx);
                                        }),
                                    ),
                            ),
                    )
                } else {
                    None
                }),
        )
        // Center: Draggable Title Bar Area
        // NOTE (linux): gpui-pre 0.3.3 ignores WindowControlArea hit-test on
        // X11/Wayland (on_hit_test_window_control is a no-op), so the
        // declarative Drag area alone never moves the window. The explicit
        // start_window_move() below is what actually drags on Linux; it is a
        // harmless duplicate on Windows/macOS.
        .child(
            div()
                .flex_1()
                .h_full()
                .window_control_area(WindowControlArea::Drag)
                .on_mouse_down(MouseButton::Left, |_, window, _| {
                    window.start_window_move();
                }),
        )
        // Right: Custom Window Controls (Minimize, Maximize/Restore, Close)
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .h_full()
                // Minimize Button
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .w(px(44.0))
                        .h_full()
                        .cursor_pointer()
                        .id("tab_strip-07").hover(|s| s.bg(hover_bg))
                        .child(
                            svg()
                                .data(MINIMIZE_SVG)
                                .size(px(12.0))
                                .text_color(muted_text),
                        )
                        .on_mouse_down(MouseButton::Left, |_, window, _| {
                            window.minimize_window();
                        }),
                )
                // Maximize / Restore Button
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .w(px(44.0))
                        .h_full()
                        .cursor_pointer()
                        .id("tab_strip-08").hover(|s| s.bg(hover_bg))
                        .child(
                            svg()
                                .data(if app.is_maximized {
                                    RESTORE_SVG
                                } else {
                                    MAXIMIZE_SVG
                                })
                                .size(px(12.0))
                                .text_color(muted_text),
                        )
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, _, window, cx| {
                                let new_max = crate::window_state::toggle_maximize(window);
                                this.is_maximized = new_max;
                                cx.notify();
                            }),
                        ),
                )
                // Close Button (Red on hover)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .w(px(46.0))
                        .h_full()
                        .cursor_pointer()
                        .id("tab_strip-09").hover(|s| s.bg(rgb(0xe81123)).text_color(rgb(0xffffff)))
                        .child(svg().data(X_SVG).size(px(13.0)).text_color(muted_text))
                        .on_mouse_down(MouseButton::Left, |_, window, _| {
                            window.remove_window();
                        }),
                ),
        )
}
