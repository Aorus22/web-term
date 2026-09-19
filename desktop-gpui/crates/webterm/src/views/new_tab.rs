//! Onboarding / New Tab Page ("Welcome to WebTerm") matching fe/src/components/NewTabView.tsx 1:1.

use crate::app_state::AppState;
use crate::icons::{KEY_SVG, PLUS_SVG, SEARCH_SVG, TAG_SVG, TERMINAL_SVG};
use gpui::*;
use gpui_component::input::Input;
use gpui_component::Sizable;

/// Renders the Onboarding / New Tab Page.
pub fn render_new_tab_page(app: &mut AppState, cx: &mut Context<AppState>) -> AnyElement {
    let text_color = app.text_color();
    let muted_text = app.muted_text();
    let border_color = app.border_color();
    let card_bg = app.card_bg();
    let muted_bg = app.muted_bg();
    let primary_color = app.primary_color();

    let query = app.quick_connect_query.trim().to_lowercase();
    let connections = app.connections.clone();

    // Filter connections for QuickConnect dropdown
    let filtered_connections: Vec<_> = connections
        .iter()
        .filter(|c| {
            if query.is_empty() {
                true
            } else {
                c.label.to_lowercase().contains(&query)
                    || c.host.to_lowercase().contains(&query)
                    || c.username.to_lowercase().contains(&query)
            }
        })
        .take(5)
        .cloned()
        .collect();

    div()
        .flex()
        .flex_col()
        .items_center()
        .justify_start()
        .size_full()
        .id("new-tab-scroll-view")
        .overflow_y_scroll()
        .pt(px(48.0))
        .pb(px(64.0))
        .px(px(32.0))
        .bg(app.bg_color())
        .child(
            div()
                .flex()
                .flex_col()
                .w_full()
                .max_w(px(896.0))
                .gap(px(36.0))
                // 1. Header: Welcome to WebTerm
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .text_center()
                        .gap_1p5()
                        .child(
                            div()
                                .text_3xl()
                                .font_weight(FontWeight::BOLD)
                                .text_color(text_color)
                                .child("Welcome to WebTerm"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(muted_text)
                                .child("Connect to a server to get started"),
                        ),
                )
                // 2. QuickConnect Box
                .child(
                    div()
                        .w_full()
                        .max_w(px(540.0))
                        .mx_auto()
                        .flex()
                        .flex_col()
                        .rounded_xl()
                        .border_1()
                        .border_color(border_color)
                        .bg(card_bg)
                        .shadow_md()
                        .overflow_hidden()
                        // Search Input Bar
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .px_3p5()
                                .py_2p5()
                                .border_b_1()
                                .border_color(border_color)
                                .gap_2p5()
                                .child(svg().data(SEARCH_SVG).size(px(16.0)).text_color(muted_text))
                                .child(
                                    div().flex_1().child(
                                        Input::new(&app.inputs().quick_connect)
                                            .with_size(gpui_component::Size::Small)
                                            .w_full()
                                            .text_size(px(13.0)),
                                    ),
                                ),
                        )
                        // Group: New Connection
                        .child(
                            div()
                                .px_3()
                                .pt_2p5()
                                .pb_1()
                                .text_xs()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(muted_text)
                                .child("New Connection"),
                        )
                        .child(
                            div()
                                .mx_1p5()
                                .mb_1p5()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_2p5()
                                .px_3()
                                .py_2()
                                .rounded_lg()
                                .cursor_pointer()
                                .id("new_tab-01").hover(|s| s.bg(muted_bg))
                                .child(svg().data(PLUS_SVG).size(px(14.0)).text_color(muted_text))
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(text_color)
                                        .child("Create New..."),
                                )
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(|this, _, window, cx| {
                                        this.open_create_connection_modal(window, cx);
                                    }),
                                ),
                        )
                        // Group: Saved Connections (if any)
                        .children(if !filtered_connections.is_empty() {
                            Some(
                                div()
                                    .flex()
                                    .flex_col()
                                    .child(
                                        div()
                                            .px_3()
                                            .pt_2()
                                            .pb_1()
                                            .text_xs()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(muted_text)
                                            .child("Saved Connections"),
                                    )
                                    .children(filtered_connections.into_iter().map(|conn| {
                                        let conn_id = conn.id.clone();
                                        let label = if conn.label.trim().is_empty() {
                                            conn.host.clone()
                                        } else {
                                            conn.label.clone()
                                        };
                                        let host = conn.host.clone();
                                        let is_key = conn.auth_method == "key";

                                        div()
                                            .mx_1p5()
                                            .mb_1()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap_2p5()
                                            .px_3()
                                            .py_2()
                                            .rounded_lg()
                                            .cursor_pointer()
                                            .id(ElementId::Name(format!("nt-conn-{}", conn.id).into())).hover(|s| s.bg(muted_bg))
                                            .child(
                                                svg()
                                                    .data(TERMINAL_SVG)
                                                    .size(px(14.0))
                                                    .text_color(muted_text),
                                            )
                                            .children(if is_key {
                                                Some(
                                                    svg()
                                                        .data(KEY_SVG)
                                                        .size(px(13.0))
                                                        .text_color(muted_text),
                                                )
                                            } else {
                                                None
                                            })
                                            .child(
                                                div()
                                                    .flex_1()
                                                    .text_sm()
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .text_color(text_color)
                                                    .truncate()
                                                    .child(format!("{} ({})", label, host)),
                                            )
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(move |this, _, _window, cx| {
                                                    this.connect_to_host(&conn_id, cx);
                                                }),
                                            )
                                    })),
                            )
                        } else {
                            None
                        }),
                )
                // 3. Direct Access Section
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .child(
                            div()
                                .text_xl()
                                .font_weight(FontWeight::BOLD)
                                .text_color(text_color)
                                .child("Direct Access"),
                        )
                        .child(
                            div().flex().flex_row().w_full().child(
                                div()
                                    .w(px(280.0))
                                    .p_4()
                                    .rounded_xl()
                                    .border_1()
                                    .border_color(primary_color)
                                    .bg(muted_bg)
                                    .cursor_pointer()
                                    .id("new_tab-03").hover(|s| s.border_color(primary_color).bg(border_color))
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap_2()
                                            .text_base()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(primary_color)
                                            .child(
                                                svg()
                                                    .data(TERMINAL_SVG)
                                                    .size(px(16.0))
                                                    .text_color(primary_color),
                                            )
                                            .child("Local Terminal"),
                                    )
                                    .child(
                                        div()
                                            .mt_1p5()
                                            .text_xs()
                                            .text_color(muted_text)
                                            .child("Spawn a shell on the backend host"),
                                    )
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _window, cx| {
                                            this.open_local_tab(cx);
                                        }),
                                    ),
                            ),
                        ),
                )
                // 4. Saved Connections Section
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_4()
                        // Section Header
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .justify_between()
                                .pt_4()
                                .border_t_1()
                                .border_color(border_color)
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_0p5()
                                        .child(
                                            div()
                                                .text_xl()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(text_color)
                                                .child("Saved Connections"),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(muted_text)
                                                .child("Your frequently used SSH configurations"),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap_1p5()
                                        .px_3()
                                        .py_1p5()
                                        .rounded_lg()
                                        .border_1()
                                        .border_color(border_color)
                                        .bg(card_bg)
                                        .id("new_tab-04").hover(|s| s.bg(muted_bg))
                                        .cursor_pointer()
                                        .text_xs()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(text_color)
                                        .child(
                                            svg()
                                                .data(PLUS_SVG)
                                                .size(px(13.0))
                                                .text_color(muted_text),
                                        )
                                        .child("New Connection")
                                        .on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(|this, _, window, cx| {
                                                this.open_create_connection_modal(window, cx);
                                            }),
                                        ),
                                ),
                        )
                        // Saved Connections Cards Grid (3-column)
                        .children(if connections.is_empty() {
                            vec![div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .justify_center()
                                .py_12()
                                .rounded_xl()
                                .border_1()
                                .border_color(border_color)
                                .bg(card_bg)
                                .gap_3()
                                .child(
                                    svg()
                                        .data(TERMINAL_SVG)
                                        .size(px(32.0))
                                        .text_color(muted_text),
                                )
                                .child(
                                    div().text_sm().text_color(muted_text).child(
                                        "No saved connections yet. Create one to get started.",
                                    ),
                                )
                                .into_any_element()]
                        } else {
                            // Render 3-column rows
                            connections
                                .chunks(3)
                                .map(|row| {
                                    let mut row_div = div().flex().flex_row().w_full().gap_4();

                                    for conn in row {
                                        let conn_id = conn.id.clone();
                                        let label = if conn.label.trim().is_empty() {
                                            conn.host.clone()
                                        } else {
                                            conn.label.clone()
                                        };
                                        let subtitle = format!("{}@{}", conn.username, conn.host);
                                        let is_session_active =
                                            app.session_manager.tabs().iter().any(|t| {
                                                t.connection_id.as_deref() == Some(&conn.id)
                                            });

                                        row_div = row_div.child(
                                            div()
                                                .flex_1()
                                                .flex()
                                                .flex_col()
                                                .p_4()
                                                .rounded_xl()
                                                .border_1()
                                                .border_color(if is_session_active {
                                                    primary_color
                                                } else {
                                                    border_color
                                                })
                                                .bg(card_bg)
                                                .cursor_pointer()
                                                .id(ElementId::Name(format!("nt-conn-{}", conn.id).into())).hover(|s| {
                                                    s.border_color(primary_color).bg(muted_bg)
                                                })
                                                // Header: Title & Terminal icon
                                                .child(
                                                    div()
                                                        .flex()
                                                        .flex_row()
                                                        .items_center()
                                                        .justify_between()
                                                        .child(
                                                            div()
                                                                .text_base()
                                                                .font_weight(FontWeight::BOLD)
                                                                .text_color(text_color)
                                                                .truncate()
                                                                .child(label),
                                                        )
                                                        .child(
                                                            div()
                                                                .flex()
                                                                .flex_row()
                                                                .items_center()
                                                                .gap_2()
                                                                .children(if is_session_active {
                                                                    Some(
                                                                        div()
                                                                            .size(px(8.0))
                                                                            .rounded_full()
                                                                            .bg(rgb(0x22c55e)),
                                                                    )
                                                                } else {
                                                                    None
                                                                })
                                                                .child(
                                                                    svg()
                                                                        .data(TERMINAL_SVG)
                                                                        .size(px(14.0))
                                                                        .text_color(muted_text),
                                                                ),
                                                        ),
                                                )
                                                // Subtitle: user@host
                                                .child(
                                                    div()
                                                        .mt_1()
                                                        .text_xs()
                                                        .text_color(muted_text)
                                                        .truncate()
                                                        .child(subtitle),
                                                )
                                                // Tags
                                                .child(
                                                    div()
                                                        .mt_3()
                                                        .flex()
                                                        .flex_row()
                                                        .flex_wrap()
                                                        .gap_1p5()
                                                        .children(if !conn.tags.is_empty() {
                                                            conn.tags
                                                                .iter()
                                                                .map(|tag| {
                                                                    div()
                                                                        .px_2()
                                                                        .py_0p5()
                                                                        .rounded_md()
                                                                        .border_1()
                                                                        .border_color(border_color)
                                                                        .text_xs()
                                                                        .text_color(muted_text)
                                                                        .child(tag.clone())
                                                                })
                                                                .collect::<Vec<_>>()
                                                        } else {
                                                            vec![div()
                                                                .flex()
                                                                .flex_row()
                                                                .items_center()
                                                                .gap_1()
                                                                .text_xs()
                                                                .text_color(muted_text)
                                                                .child(
                                                                    svg()
                                                                        .data(TAG_SVG)
                                                                        .size(px(11.0))
                                                                        .text_color(muted_text),
                                                                )
                                                                .child("No tags")]
                                                        }),
                                                )
                                                .on_mouse_down(
                                                    MouseButton::Left,
                                                    cx.listener(move |this, _, _window, cx| {
                                                        this.connect_to_host(&conn_id, cx);
                                                    }),
                                                ),
                                        );
                                    }

                                    // Spacer for incomplete row
                                    for _ in 0..(3 - row.len()) {
                                        row_div = row_div.child(div().flex_1());
                                    }

                                    row_div.into_any_element()
                                })
                                .collect()
                        }),
                ),
        )
        .into_any_element()
}
