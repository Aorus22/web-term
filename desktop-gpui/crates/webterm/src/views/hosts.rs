//! Hosts Catalog view: connection cards grid, real-time search, tag filtering, and quick-connect.

use gpui::*;
use webterm_backend_client::Connection;
use webterm_settings::Theme as SettingsTheme;

use crate::app_state::AppState;

/// Renders the host connection catalog view.
pub fn render_hosts_view(app: &mut AppState, cx: &mut Context<AppState>) -> AnyElement {
    let is_dark = app.theme != SettingsTheme::Light;
    let bg_color = if is_dark { rgb(0x18181b) } else { rgb(0xf8fafc) };
    let card_bg = if is_dark { rgb(0x27272a) } else { rgb(0xffffff) };
    let border_color = if is_dark { rgb(0x3f3f46) } else { rgb(0xe2e8f0) };
    let text_color = if is_dark { rgb(0xf4f4f5) } else { rgb(0x0f172a) };
    let muted_text = if is_dark { rgb(0xa1a1aa) } else { rgb(0x64748b) };
    let tag_bg = if is_dark { rgb(0x3f3f46) } else { rgb(0xe2e8f0) };

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
        .p_4()
        .gap_4()
        // Top Toolbar: Search, Tags, Action Buttons
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .flex_wrap()
                .gap_3()
                // Left: Search input and Tag filters
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_3()
                        .flex_wrap()
                        // Search bar
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .px_3()
                                .py_1()
                                .rounded_md()
                                .bg(card_bg)
                                .border_1()
                                .border_color(border_color)
                                .gap_2()
                                .child(div().text_sm().text_color(muted_text).child("🔍"))
                                .child(
                                    div()
                                        .text_sm()
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
                                            .text_xs()
                                            .text_color(muted_text)
                                            .hover(|s| s.text_color(text_color))
                                            .child("×")
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
                                .gap_1()
                                // "All" tag
                                .child({
                                    let is_selected = app.selected_tag.is_none();
                                    div()
                                        .px_2()
                                        .py_1()
                                        .rounded_full()
                                        .text_xs()
                                        .cursor_pointer()
                                        .bg(if is_selected {
                                            rgb(0x0284c7)
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
                                        .px_2()
                                        .py_1()
                                        .rounded_full()
                                        .text_xs()
                                        .cursor_pointer()
                                        .bg(if is_selected {
                                            rgb(0x0284c7)
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
                // Right: Action buttons (+ Add Host, Export, Import, Refresh)
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        // Refresh button
                        .child(
                            div()
                                .px_3()
                                .py_1()
                                .rounded_md()
                                .bg(card_bg)
                                .border_1()
                                .border_color(border_color)
                                .text_xs()
                                .cursor_pointer()
                                .hover(|s| s.bg(tag_bg))
                                .child(if is_loading { "⟳ Loading..." } else { "⟳ Refresh" })
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                    this.fetch_connections(cx);
                                })),
                        )
                        // Export button
                        .child(
                            div()
                                .px_3()
                                .py_1()
                                .rounded_md()
                                .bg(card_bg)
                                .border_1()
                                .border_color(border_color)
                                .text_xs()
                                .cursor_pointer()
                                .hover(|s| s.bg(tag_bg))
                                .child("Export")
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                    this.export_connections_to_disk(cx);
                                })),
                        )
                        // Import button
                        .child(
                            div()
                                .px_3()
                                .py_1()
                                .rounded_md()
                                .bg(card_bg)
                                .border_1()
                                .border_color(border_color)
                                .text_xs()
                                .cursor_pointer()
                                .hover(|s| s.bg(tag_bg))
                                .child("Import")
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                    this.open_import_modal(cx);
                                })),
                        )
                        // + Add Host button
                        .child(
                            div()
                                .px_3()
                                .py_1()
                                .rounded_md()
                                .bg(if is_dark { rgb(0x0284c7) } else { rgb(0x0ea5e9) })
                                .hover(|s| s.bg(if is_dark { rgb(0x0369a1) } else { rgb(0x0284c7) }))
                                .text_xs()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(rgb(0xffffff))
                                .cursor_pointer()
                                .child("+ Add Host")
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                    this.open_create_connection_modal(cx);
                                })),
                        ),
                ),
        )
        // Main Catalog Content: Cards Grid or Empty State
        .child(if count == 0 {
            div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .py_16()
                .gap_2()
                .text_color(muted_text)
                .child(div().text_2xl().child("🖥️"))
                .child(
                    div()
                        .text_sm()
                        .child(if app.connections.is_empty() {
                            "No saved SSH hosts yet. Click '+ Add Host' to save your first connection."
                        } else {
                            "No hosts match your search or filter."
                        }),
                )
                .into_any_element()
        } else {
            div()
                .flex()
                .flex_row()
                .flex_wrap()
                .gap_4()
                .children(filtered_connections.into_iter().map(|conn| {
                    let conn_id = conn.id.clone();
                    let conn_id_del = conn.id.clone();
                    let conn_id_edit = conn.id.clone();
                    let host_display = format!("{}@{}:{}", conn.username, conn.host, conn.port);
                    let is_key_auth = conn.auth_method == "key";

                    div()
                        .flex()
                        .flex_col()
                        .w(px(320.0))
                        .rounded_lg()
                        .bg(card_bg)
                        .border_1()
                        .border_color(border_color)
                        .p_4()
                        .gap_3()
                        .shadow_sm()
                        .hover(|s| s.border_color(if is_dark { rgb(0x38bdf8) } else { rgb(0x0284c7) }))
                        // Card Header: Label & Auth badge
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(text_color)
                                        .child(conn.label.clone()),
                                )
                                .child(
                                    div()
                                        .px_2()
                                        .py_0p5()
                                        .rounded_full()
                                        .text_xs()
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
                                        .child(if is_key_auth { "🔑 Key" } else { "🔒 Password" }),
                                ),
                        )
                        // Host details
                        .child(
                            div()
                                .text_xs()
                                .font_family("JetBrains Mono")
                                .text_color(muted_text)
                                .child(host_display),
                        )
                        // Tags
                        .child(
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
                        // Card Actions: Connect, Edit, Delete
                        .child(
                            div()
                                .mt_1()
                                .flex()
                                .flex_row()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .gap_2()
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
                                        .px_3()
                                        .py_1()
                                        .rounded_md()
                                        .bg(if is_dark { rgb(0x166534) } else { rgb(0x22c55e) })
                                        .hover(|s| s.bg(if is_dark { rgb(0x15803d) } else { rgb(0x16a34a) }))
                                        .cursor_pointer()
                                        .text_xs()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(rgb(0xffffff))
                                        .child("Connect ➔")
                                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                            this.connect_to_host(&conn_id, cx);
                                        })),
                                ),
                        )
                }))
                .into_any_element()
        })
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
