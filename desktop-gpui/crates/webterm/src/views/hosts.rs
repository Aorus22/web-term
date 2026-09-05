//! Hosts Catalog view: connection cards grid, real-time search, tag filtering, and quick-connect.

use gpui::*;
use gpui_component::{Icon, IconName};
use webterm_backend_client::Connection;
use webterm_settings::Theme as SettingsTheme;

use crate::app_state::AppState;

/// Renders the host connection catalog view.
pub fn render_hosts_view(app: &mut AppState, cx: &mut Context<AppState>) -> AnyElement {
    let is_dark = app.theme != SettingsTheme::Light;
    let bg_color = if is_dark { rgb(0x121214) } else { rgb(0xf8fafc) };
    let toolbar_bg = if is_dark { rgb(0x18181b) } else { rgb(0xffffff) };
    let card_bg = if is_dark { rgb(0x1e1e24) } else { rgb(0xffffff) };
    let card_hover_bg = if is_dark { rgb(0x23232a) } else { rgb(0xfcfcfd) };
    let border_color = if is_dark { rgb(0x27272a) } else { rgb(0xe2e8f0) };
    let text_color = if is_dark { rgb(0xf4f4f5) } else { rgb(0x0f172a) };
    let muted_text = if is_dark { rgb(0xa1a1aa) } else { rgb(0x64748b) };
    let tag_bg = if is_dark { rgb(0x27272a) } else { rgb(0xf1f5f9) };
    let primary_color = if is_dark { rgb(0x0284c7) } else { rgb(0x0284c7) };
    let primary_hover = if is_dark { rgb(0x0369a1) } else { rgb(0x0369a1) };
    let icon_box_bg = if is_dark { rgb(0x27272a) } else { rgb(0xf1f5f9) };

    // Collect all distinct tags across all connections
    let all_tags = extract_unique_tags(&app.connections);
    let filtered_connections = filter_connections(
        &app.connections,
        &app.search_query,
        app.selected_tag.as_deref(),
    );

    let count = filtered_connections.len();
    let is_loading = app.is_loading_connections;

    div()
        .flex()
        .flex_col()
        .size_full()
        .bg(bg_color)
        // Top Toolbar: Matches fe/ HostsPage header
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
                // Left: Title, Divider, Search bar & Tag filters
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_3()
                        // Title
                        .child(
                            div()
                                .text_base()
                                .font_weight(FontWeight::BOLD)
                                .text_color(text_color)
                                .child("Hosts"),
                        )
                        // Vertical divider
                        .child(
                            div()
                                .h(px(18.0))
                                .w(px(1.0))
                                .bg(border_color),
                        )
                        // Search bar
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .px_3()
                                .py_1p5()
                                .rounded_lg()
                                .bg(if is_dark { rgb(0x202024) } else { rgb(0xf1f5f9) })
                                .border_1()
                                .border_color(border_color)
                                .gap_2()
                                .child(
                                    Icon::new(IconName::Search)
                                        .size(px(13.0))
                                        .text_color(muted_text),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(if app.search_query.is_empty() {
                                            muted_text
                                        } else {
                                            text_color
                                        })
                                        .child(SharedString::from(if app.search_query.is_empty() {
                                            "Filter hosts...".to_string()
                                        } else {
                                            app.search_query.clone()
                                        })),
                                )
                                .children(if !app.search_query.is_empty() {
                                    Some(
                                        div()
                                            .cursor_pointer()
                                            .child(
                                                Icon::new(IconName::Close)
                                                    .size(px(11.0))
                                                    .text_color(muted_text),
                                            )
                                            .hover(|s| s.text_color(text_color))
                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                                this.set_search_query(String::new(), cx);
                                            })),
                                    )
                                } else {
                                    None
                                }),
                        )
                        // Tag filter pills
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_1p5()
                                // "All" tag
                                .child({
                                    let is_selected = app.selected_tag.is_none();
                                    div()
                                        .px_2p5()
                                        .py_1()
                                        .rounded_full()
                                        .text_xs()
                                        .font_weight(FontWeight::MEDIUM)
                                        .cursor_pointer()
                                        .bg(if is_selected {
                                            primary_color
                                        } else {
                                            tag_bg
                                        })
                                        .text_color(if is_selected {
                                            rgb(0xffffff)
                                        } else {
                                            muted_text
                                        })
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
                                        .py_1()
                                        .rounded_full()
                                        .text_xs()
                                        .font_weight(FontWeight::MEDIUM)
                                        .cursor_pointer()
                                        .bg(if is_selected {
                                            primary_color
                                        } else {
                                            tag_bg
                                        })
                                        .text_color(if is_selected {
                                            rgb(0xffffff)
                                        } else {
                                            muted_text
                                        })
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
                // Right: Action buttons (Refresh, Export, Import, + New Host)
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        // Refresh button
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_1p5()
                                .px_3()
                                .py_1p5()
                                .rounded_lg()
                                .bg(if is_dark { rgb(0x202024) } else { rgb(0xffffff) })
                                .border_1()
                                .border_color(border_color)
                                .text_xs()
                                .text_color(muted_text)
                                .cursor_pointer()
                                .hover(|s| s.bg(tag_bg).text_color(text_color))
                                .child(Icon::new(IconName::RotateCw).size(px(12.0)))
                                .child(if is_loading { "Loading..." } else { "Refresh" })
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                    this.fetch_connections(cx);
                                })),
                        )
                        // Export button
                        .child(
                            div()
                                .px_3()
                                .py_1p5()
                                .rounded_lg()
                                .bg(if is_dark { rgb(0x202024) } else { rgb(0xffffff) })
                                .border_1()
                                .border_color(border_color)
                                .text_xs()
                                .text_color(muted_text)
                                .cursor_pointer()
                                .hover(|s| s.bg(tag_bg).text_color(text_color))
                                .child("Export")
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                    this.export_connections_to_disk(cx);
                                })),
                        )
                        // Import button
                        .child(
                            div()
                                .px_3()
                                .py_1p5()
                                .rounded_lg()
                                .bg(if is_dark { rgb(0x202024) } else { rgb(0xffffff) })
                                .border_1()
                                .border_color(border_color)
                                .text_xs()
                                .text_color(muted_text)
                                .cursor_pointer()
                                .hover(|s| s.bg(tag_bg).text_color(text_color))
                                .child("Import")
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                    this.open_import_modal(cx);
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
                                .hover(move |s| s.bg(primary_hover))
                                .text_xs()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(rgb(0xffffff))
                                .cursor_pointer()
                                .child(Icon::new(IconName::Plus).size(px(13.0)))
                                .child("New Host")
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                    this.open_create_connection_modal(cx);
                                })),
                        ),
                ),
        )
        // Main Catalog Content: Scrollable Cards Grid
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
                                .bg(icon_box_bg)
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(Icon::new(IconName::HardDrive).size(px(24.0)).text_color(muted_text)),
                        )
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(text_color)
                                .child(if app.connections.is_empty() {
                                    "No saved SSH hosts yet."
                                } else {
                                    "No hosts match your search or filter."
                                }),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(muted_text)
                                .child(if app.connections.is_empty() {
                                    "Click '+ New Host' to save your first connection."
                                } else {
                                    "Try clearing your search query or tag filter."
                                }),
                        )
                        .into_any_element()
                } else {
                    div()
                        .flex()
                        .flex_row()
                        .flex_wrap()
                        .gap_4()
                        .pb_12()
                        .children(filtered_connections.into_iter().map(|conn| {
                            let conn_id = conn.id.clone();
                            let conn_id_del = conn.id.clone();
                            let conn_id_edit = conn.id.clone();
                            let host_display = format!("{}@{}:{}", conn.username, conn.host, conn.port);
                            let is_key_auth = conn.auth_method == "key";

                            div()
                                .flex()
                                .flex_col()
                                .w(px(360.0))
                                .rounded_xl()
                                .bg(card_bg)
                                .border_1()
                                .border_color(border_color)
                                .p_4()
                                .gap_3()
                                .shadow_sm()
                                .hover(move |s| {
                                    s.bg(card_hover_bg)
                                        .border_color(if is_dark { rgb(0x38bdf8) } else { rgb(0x0284c7) })
                                })
                                // Card Top: Server Icon + Name & Host Monospace + Auth badge
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_start()
                                        .gap_3()
                                        // Left Icon Box
                                        .child(
                                            div()
                                                .size(px(36.0))
                                                .rounded_lg()
                                                .bg(icon_box_bg)
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .child(
                                                    Icon::new(IconName::HardDrive)
                                                        .size(px(18.0))
                                                        .text_color(if is_dark { rgb(0x38bdf8) } else { rgb(0x0284c7) }),
                                                ),
                                        )
                                        // Center details
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .flex_1()
                                                .gap_0p5()
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(text_color)
                                                        .child(conn.label.clone()),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .font_family("JetBrains Mono")
                                                        .text_color(muted_text)
                                                        .child(host_display),
                                                ),
                                        )
                                        // Right Auth badge
                                        .child(
                                            div()
                                                .px_2()
                                                .py_0p5()
                                                .rounded_md()
                                                .text_xs()
                                                .font_weight(FontWeight::MEDIUM)
                                                .bg(if is_key_auth {
                                                    if is_dark { rgb(0x0c4a6e) } else { rgb(0xe0f2fe) }
                                                } else {
                                                    tag_bg
                                                })
                                                .text_color(if is_key_auth {
                                                    if is_dark { rgb(0x38bdf8) } else { rgb(0x0284c7) }
                                                } else {
                                                    muted_text
                                                })
                                                .child(if is_key_auth { "Key" } else { "Password" }),
                                        ),
                                )
                                // Tags (if any)
                                .children(if !conn.tags.is_empty() {
                                    Some(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .flex_wrap()
                                            .gap_1()
                                            .children(conn.tags.into_iter().map(|tag| {
                                                div()
                                                    .px_2()
                                                    .py_0p5()
                                                    .rounded_md()
                                                    .text_xs()
                                                    .bg(tag_bg)
                                                    .text_color(muted_text)
                                                    .child(tag)
                                            })),
                                    )
                                } else {
                                    None
                                })
                                // Card Footer: Edit / Delete on left, Connect button on right
                                .child(
                                    div()
                                        .mt_1()
                                        .pt_2p5()
                                        .border_t_1()
                                        .border_color(if is_dark { rgb(0x27272a) } else { rgb(0xf1f5f9) })
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .justify_between()
                                        .child(
                                            div()
                                                .flex()
                                                .flex_row()
                                                .gap_3()
                                                .child(
                                                    div()
                                                        .cursor_pointer()
                                                        .text_xs()
                                                        .text_color(muted_text)
                                                        .hover(|s| s.text_color(text_color))
                                                        .child("Edit")
                                                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                                            this.open_edit_connection_modal(&conn_id_edit, cx);
                                                        })),
                                                )
                                                .child(
                                                    div()
                                                        .cursor_pointer()
                                                        .text_xs()
                                                        .text_color(rgb(0xef4444))
                                                        .hover(|s| s.text_color(rgb(0xdc2626)))
                                                        .child("Delete")
                                                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                                            this.delete_connection(&conn_id_del, cx);
                                                        })),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .gap_1p5()
                                                .px_3()
                                                .py_1()
                                                .rounded_lg()
                                                .bg(primary_color)
                                                .hover(move |s| s.bg(primary_hover))
                                                .cursor_pointer()
                                                .text_xs()
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .text_color(rgb(0xffffff))
                                                .child("Connect")
                                                .child(Icon::new(IconName::ExternalLink).size(px(11.0)))
                                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                                    this.connect_to_host(&conn_id, cx);
                                                })),
                                        ),
                                )
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
                let matches_tags = c.tags.iter().any(|t| t.to_lowercase().contains(&query_lower));
                matches_label || matches_host || matches_user || matches_tags
            } else {
                true
            }
        })
        .cloned()
        .collect()
}
