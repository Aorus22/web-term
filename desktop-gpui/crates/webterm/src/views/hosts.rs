//! Hosts Catalog view: connection cards grid, tag filtering, and quick-connect matching web client.

use crate::app_state::AppState;
use crate::icons::{ARROW_DOWN_SVG, ARROW_UP_SVG, PLUS_SVG};
use gpui::*;
use gpui_component::input::Input;
use gpui_component::Sizable;
use webterm_backend_client::Connection;

/// Renders the host connection catalog view matching fe/src/features/hosts/components/HostsPage.tsx.
pub fn render_hosts_view(app: &mut AppState, cx: &mut Context<AppState>) -> AnyElement {
    let bg_color = app.bg_color();
    let toolbar_bg = app.bg_color();
    let border_color = app.border_color();
    let text_color = app.text_color();
    let muted_text = app.muted_text();
    let muted_bg = app.muted_bg();
    let card_bg = app.card_bg();
    let primary_color = app.primary_color();
    let primary_fg = app.primary_fg();

    // Extract tags & filter connections
    let all_tags = extract_unique_tags(&app.connections);
    let filtered_connections = filter_connections(
        &app.connections,
        &app.search_query,
        app.selected_tag.as_deref(),
    );

    let count = filtered_connections.len();

    div()
        .flex()
        .flex_col()
        .size_full()
        .bg(bg_color)
        // Top Toolbar matching fe/ HostsPage header
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .w_full()
                .px_6()
                .py_3()
                .bg(toolbar_bg)
                .border_b_1()
                .border_color(border_color)
                // Left: Title, Divider, & Tag filters
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_4()
                        // Title: "Hosts"
                        .child(
                            div()
                                .text_lg()
                                .font_weight(FontWeight::BOLD)
                                .text_color(text_color)
                                .child("Hosts"),
                        )
                        // Search box (matches web TagFilter row)
                        .child(
                            div().w(px(220.0)).child(
                                Input::new(&app.inputs().hosts_search)
                                    .with_size(gpui_component::Size::Small)
                                    .w_full()
                                    .text_size(px(12.0)),
                            ),
                        )
                        // Tags (if any exist)
                        .children(if !all_tags.is_empty() {
                            Some(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_3()
                                    .child(
                                        div()
                                            .h(px(20.0))
                                            .w(px(1.0))
                                            .bg(border_color),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap_1p5()
                                            // "All" tag pill
                                            .child({
                                                let is_selected = app.selected_tag.is_none();
                                                div()
                                                    .px_2p5()
                                                    .py_0p5()
                                                    .rounded_full()
                                                    .text_xs()
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .cursor_pointer()
                                                    .bg(if is_selected {
                                                        muted_bg
                                                    } else {
                                                        rgba(0x00000000)
                                                    })
                                                    .text_color(if is_selected {
                                                        text_color
                                                    } else {
                                                        muted_text
                                                    })
                                                    .hover(move |s| s.text_color(text_color))
                                                    .child("All")
                                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                                        this.select_tag_filter(None, cx);
                                                    }))
                                            })
                                            // Individual tags
                                            .children(all_tags.into_iter().map(|tag| {
                                                let is_selected = app.selected_tag.as_ref() == Some(&tag);
                                                let tag_clone = tag.clone();
                                                div()
                                                    .px_2p5()
                                                    .py_0p5()
                                                    .rounded_full()
                                                    .text_xs()
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .cursor_pointer()
                                                    .bg(if is_selected {
                                                        muted_bg
                                                    } else {
                                                        rgba(0x00000000)
                                                    })
                                                    .text_color(if is_selected {
                                                        text_color
                                                    } else {
                                                        muted_text
                                                    })
                                                    .hover(move |s| s.text_color(text_color))
                                                    .child(tag)
                                                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                                        let next = if this.selected_tag.as_ref() == Some(&tag_clone) {
                                                            None
                                                        } else {
                                                            Some(tag_clone.clone())
                                                        };
                                                        this.select_tag_filter(next, cx);
                                                    }))
                                            })),
                                    ),
                            )
                        } else {
                            None
                        }),
                )
                // Right: Action buttons (Export, Import, + New Host)
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        // Export button
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_1p5()
                                .px_3()
                                .py_1p5()
                                .rounded_lg()
                                .bg(card_bg)
                                .border_1()
                                .border_color(border_color)
                                .text_xs()
                                .text_color(muted_text)
                                .cursor_pointer()
                                .hover(move |s| s.bg(muted_bg).text_color(text_color))
                                .child(
                                    svg()
                                        .data(ARROW_DOWN_SVG)
                                        .size(px(12.0))
                                        .text_color(muted_text),
                                )
                                .child("Export")
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                    this.export_connections_to_disk(cx);
                                })),
                        )
                        // Import button
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_1p5()
                                .px_3()
                                .py_1p5()
                                .rounded_lg()
                                .bg(card_bg)
                                .border_1()
                                .border_color(border_color)
                                .text_xs()
                                .text_color(muted_text)
                                .cursor_pointer()
                                .hover(move |s| s.bg(muted_bg).text_color(text_color))
                                .child(
                                    svg()
                                        .data(ARROW_UP_SVG)
                                        .size(px(12.0))
                                        .text_color(muted_text),
                                )
                                .child("Import")
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                    this.import_connections_from_file(cx);
                                })),
                        )
                        // + New Host button
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_1p5()
                                .px_3p5()
                                .py_1p5()
                                .rounded_lg()
                                .bg(primary_color)
                                .hover(|s| s.opacity(0.9))
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(primary_fg)
                                .cursor_pointer()
                                .child(
                                    svg()
                                        .data(PLUS_SVG)
                                        .size(px(13.0))
                                        .text_color(primary_fg),
                                )
                                .child("New Host")
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, window, cx| {
                                    this.open_create_connection_modal(window, cx);
                                })),
                        ),
                ),
        )
        // Main Catalog Content: Scrollable 3-Column Responsive Cards Grid
        .child(
            div()
                .id("hosts-scroll-area")
                .flex_1()
                .w_full()
                .overflow_y_scroll()
                .p_6()
                .child(if count == 0 {
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .py_24()
                        .gap_3()
                        .text_color(muted_text)
                        .child(
                            div()
                                .size(px(48.0))
                                .rounded_full()
                                .bg(muted_bg)
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(svg().data(crate::icons::SERVER_SVG).size(px(24.0)).text_color(muted_text)),
                        )
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(text_color)
                                .child(if app.connections.is_empty() {
                                    "No saved SSH hosts yet."
                                } else {
                                    "No hosts match your filter."
                                }),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(muted_text)
                                .child(if app.connections.is_empty() {
                                    "Click '+ New Host' to save your first connection."
                                } else {
                                    "Try clearing your tag filter."
                                }),
                        )
                        .into_any_element()
                } else {
                    // Chunk connections into rows of 3 to produce 1:1 grid-cols-3 layout matching Electron
                    let chunks: Vec<Vec<Connection>> = filtered_connections
                        .chunks(3)
                        .map(|c| c.to_vec())
                        .collect();

                    div()
                        .flex()
                        .flex_col()
                        .gap_4()
                        .w_full()
                        .pb_12()
                        .children(chunks.into_iter().map(|chunk| {
                            let chunk_len = chunk.len();
                            div()
                                .flex()
                                .flex_row()
                                .gap_4()
                                .w_full()
                                .children(chunk.into_iter().map(|conn| {
                                    let conn_id = conn.id.clone();
                                    let conn_id_click = conn.id.clone();
                                    let conn_id_del = conn.id.clone();
                                    let conn_id_edit = conn.id.clone();
                                    let conn_id_dup = conn.id.clone();

                                    // Active session count for this connection
                                    let conn_sessions = app.session_manager.tabs()
                                        .iter()
                                        .filter(|t| t.connection_id.as_deref() == Some(&conn_id))
                                        .count();
                                    let is_active = conn_sessions > 0;

                                    // Clean host format: omit :22 when port is 22 or 0
                                    let host_display = if conn.port == 22 || conn.port == 0 {
                                        format!("{}@{}", conn.username, conn.host)
                                    } else {
                                        format!("{}@{}:{}", conn.username, conn.host, conn.port)
                                    };

                                    let card_border = if is_active {
                                        primary_color
                                    } else {
                                        border_color
                                    };

                                    let icon_color = if is_active {
                                        primary_color
                                    } else {
                                        muted_text
                                    };

                                    div()
                                        .flex_1()
                                        .min_w_0()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .justify_between()
                                        .py_4()
                                        .px_4()
                                        .gap_3()
                                        .rounded_xl()
                                        .bg(card_bg)
                                        .border_1()
                                        .border_color(card_border)
                                        .cursor_pointer()
                                        .hover(move |s| s.border_color(primary_color))
                                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                            this.connect_to_host(&conn_id_click, cx);
                                        }))
                                        // Left side: Icon + Info
                                        .child(
                                            div()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .gap_3()
                                                .flex_1()
                                                .min_w_0()
                                                // Server icon box (subtle muted_bg, NO pink box)
                                                .child(
                                                    div()
                                                        .size(px(34.0))
                                                        .rounded_md()
                                                        .bg(muted_bg)
                                                        .flex()
                                                        .items_center()
                                                        .justify_center()
                                                        .flex_shrink_0()
                                                        .child(
                                                            svg()
                                                                .data(crate::icons::SERVER_SVG)
                                                                .size(px(16.0))
                                                                .text_color(icon_color),
                                                        ),
                                                )
                                                // Middle: Info (Label + User@Host + Tags)
                                                .child(
                                                    div()
                                                        .flex()
                                                        .flex_col()
                                                        .flex_1()
                                                        .min_w_0()
                                                        .gap_0p5()
                                                        .child(
                                                            div()
                                                                .text_sm()
                                                                .font_weight(FontWeight::BOLD)
                                                                .text_color(text_color)
                                                                .truncate()
                                                                .child(conn.label.clone()),
                                                        )
                                                        .child(
                                                            div()
                                                                .text_xs()
                                                                .text_color(muted_text)
                                                                .truncate()
                                                                .child(host_display),
                                                        )
                                                        .children(if !conn.tags.is_empty() {
                                                            Some(
                                                                div()
                                                                    .flex()
                                                                    .flex_row()
                                                                    .flex_wrap()
                                                                    .gap_1()
                                                                    .mt_1()
                                                                    .children(conn.tags.into_iter().map(|tag| {
                                                                        div()
                                                                            .px_1p5()
                                                                            .py_0()
                                                                            .rounded_sm()
                                                                            .text_xs()
                                                                            .bg(muted_bg)
                                                                            .text_color(muted_text)
                                                                            .child(tag)
                                                                    })),
                                                            )
                                                        } else {
                                                            None
                                                        }),
                                                ),
                                        )
                                        // Right: Active session green badge & subtle actions
                                        .child(
                                            div()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .gap_2()
                                                .flex_shrink_0()
                                                // Active sessions circular green badge matching Electron (e.g. "1")
                                                .children(if is_active {
                                                    Some(
                                                        div()
                                                            .size(px(20.0))
                                                            .rounded_full()
                                                            .bg(primary_color)
                                                            .flex()
                                                            .items_center()
                                                            .justify_center()
                                                            .child(
                                                                div()
                                                                    .text_xs()
                                                                    .font_weight(FontWeight::BOLD)
                                                                    .text_color(primary_fg)
                                                                    .child(conn_sessions.to_string()),
                                                            ),
                                                    )
                                                } else {
                                                    None
                                                })
                                                // Duplicate icon button
                                                .child(
                                                    div()
                                                        .flex()
                                                        .items_center()
                                                        .justify_center()
                                                        .size(px(24.0))
                                                        .rounded_md()
                                                        .hover(move |s| s.bg(muted_bg))
                                                        .child(
                                                            svg()
                                                                .data(crate::icons::COPY_SVG)
                                                                .size(px(13.0))
                                                                .text_color(muted_text),
                                                        )
                                                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                                            this.duplicate_connection(&conn_id_dup, cx);
                                                        })),
                                                )
                                                // Edit icon button
                                                .child(
                                                    div()
                                                        .flex()
                                                        .items_center()
                                                        .justify_center()
                                                        .size(px(24.0))
                                                        .rounded_md()
                                                        .hover(move |s| s.bg(muted_bg))
                                                        .child(
                                                            svg()
                                                                .data(crate::icons::EDIT_SVG)
                                                                .size(px(13.0))
                                                                .text_color(muted_text),
                                                        )
                                                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, window, cx| {
                                                            this.open_edit_connection_modal(&conn_id_edit, window, cx);
                                                        })),
                                                )
                                                // Delete icon button
                                                .child(
                                                    div()
                                                        .flex()
                                                        .items_center()
                                                        .justify_center()
                                                        .size(px(24.0))
                                                        .rounded_md()
                                                        .hover(move |s| s.bg(muted_bg))
                                                        .child(
                                                            svg()
                                                                .data(crate::icons::TRASH_SVG)
                                                                .size(px(13.0))
                                                                .text_color(muted_text),
                                                        )
                                                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                                            this.open_delete_connection_modal(&conn_id_del, cx);
                                                        })),
                                                ),
                                        )
                                }))
                                // Spacer cards for last row so cards stay 1/3 width
                                .children((0..(3 - chunk_len)).map(|_| {
                                    div().flex_1().min_w_0()
                                }))
                        }))
                        .into_any_element()
                }),
        )
        .into_any_element()
}

/// Extract all unique sorted tags across a collection of connections.
pub fn extract_unique_tags(connections: &[Connection]) -> Vec<String> {
    let mut tags = Vec::new();
    for conn in connections {
        for tag in &conn.tags {
            if !tags.contains(tag) {
                tags.push(tag.clone());
            }
        }
    }
    tags.sort();
    tags
}

/// Filter connections by tag and free-text search query.
pub fn filter_connections(
    connections: &[Connection],
    search_query: &str,
    selected_tag: Option<&str>,
) -> Vec<Connection> {
    let query_lower = search_query.trim().to_lowercase();
    connections
        .iter()
        .filter(|c| {
            if let Some(tag) = selected_tag {
                if !c.tags.iter().any(|t| t.eq_ignore_ascii_case(tag)) {
                    return false;
                }
            }
            if !query_lower.is_empty() {
                let matches_label = c.label.to_lowercase().contains(&query_lower);
                let matches_host = c.host.to_lowercase().contains(&query_lower);
                let matches_user = c.username.to_lowercase().contains(&query_lower);
                let matches_tags = c
                    .tags
                    .iter()
                    .any(|t| t.to_lowercase().contains(&query_lower));
                matches_label || matches_host || matches_user || matches_tags
            } else {
                true
            }
        })
        .cloned()
        .collect()
}
