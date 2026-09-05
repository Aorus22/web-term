//! Port Forwarding rules management view: rule cards, toggle, create/edit, and deletion modals.

use gpui::*;
use crate::app_state::{AppState, ForwardModalMode};

/// Render the Port Forwarding rules management view.
pub fn render_forwards_view(app: &mut AppState, cx: &mut Context<AppState>) -> AnyElement {
    let is_dark = app.is_dark();
    let bg_color = app.bg_color();
    let card_bg = app.card_bg();
    let border_color = app.border_color();
    let text_color = app.text_color();
    let muted_text = app.muted_text();
    let tag_bg = if is_dark { app.accent_color() } else { rgb(0xe2e8f0) };
    let hover_bg = if is_dark { app.accent_color() } else { rgb(0xf1f5f9) };
    let primary_color = app.primary_color();
    let primary_fg = rgb(app.current_theme().primary_foreground);

    let forwards = app.forwards.clone();
    let count = forwards.len();
    let is_loading = app.is_loading_forwards;

    // Create connection label lookup map
    let mut conn_map = std::collections::HashMap::new();
    for c in &app.connections {
        conn_map.insert(c.id.clone(), c.label.clone());
    }

    div()
        .flex()
        .flex_col()
        .size_full()
        .bg(bg_color)
        .text_color(text_color)
        // Top Toolbar
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .px_4()
                .py_3()
                .border_b_1()
                .border_color(border_color)
                .bg(card_bg)
                // Left: Title + Count badge
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .text_base()
                                .font_weight(FontWeight::BOLD)
                                .child("Port Forwards"),
                        )
                        .child(
                            div()
                                .px_2p5()
                                .py_0p5()
                                .rounded_full()
                                .bg(tag_bg)
                                .text_xs()
                                .text_color(muted_text)
                                .child(format!("{count} rule{}", if count == 1 { "" } else { "s" })),
                        ),
                )
                // Right: Actions (Refresh, + Create Forward)
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        // Refresh
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
                                .hover(move |s| s.bg(tag_bg).text_color(text_color))
                                .child(svg().data(crate::icons::REFRESH_CW_SVG).size(px(12.0)).text_color(muted_text))
                                .child(if is_loading { "Loading..." } else { "Refresh" })
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                    this.fetch_forwards(cx);
                                })),
                        )
                        // + Create Forward button
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
                                .child(svg().data(crate::icons::PLUS_SVG).size(px(13.0)).text_color(primary_fg))
                                .child("Create Forward")
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                    this.open_create_forward_modal(cx);
                                })),
                        ),
                ),
        )
        // Main Content Area
        .child(
            div()
                .id("forwards-scroll-area")
                .flex_1()
                .w_full()
                .overflow_y_scroll()
                .p_4()
                .child(if count == 0 {
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .p_12()
                        .gap_3()
                        .rounded_xl()
                        .border_1()
                        .border_dashed()
                        .border_color(border_color)
                        .child(
                            div()
                                .size(px(48.0))
                                .rounded_full()
                                .bg(tag_bg)
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(svg().data(crate::icons::ARROW_LEFT_RIGHT_SVG).size(px(24.0)).text_color(muted_text)),
                        )
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(if is_loading {
                                    "Loading port forwarding rules..."
                                } else {
                                    "No port forwards configured"
                                }),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(muted_text)
                                .child("Establish local (ssh -L) or reverse (ssh -R) tunnels across your saved SSH hosts."),
                        )
                        .child(
                            div()
                                .mt_2()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_1p5()
                                .px_4()
                                .py_2()
                                .rounded_lg()
                                .bg(primary_color)
                                .hover(|s| s.opacity(0.9))
                                .cursor_pointer()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(primary_fg)
                                .child(svg().data(crate::icons::PLUS_SVG).size(px(13.0)).text_color(primary_fg))
                                .child("Create First Forward")
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                    this.open_create_forward_modal(cx);
                                })),
                        )
                        .into_any_element()
                } else {
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .children(forwards.into_iter().map(|forward| {
                            let id_toggle = forward.id.clone();
                            let id_edit = forward.id.clone();
                            let forward_del = forward.clone();
                            let is_active = forward.active;
                            let is_reverse = forward.is_reverse();
                            let mapping_str = forward.mapping_display();
                            let conn_label = conn_map
                                .get(&forward.connection_id)
                                .cloned()
                                .unwrap_or_else(|| "Unknown Connection".to_string());
                            let has_error = !forward.error.is_empty();
                            let error_msg = forward.error.clone();

                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .justify_between()
                                .p_4()
                                .rounded_xl()
                                .bg(card_bg)
                                .border_1()
                                .border_color(if is_active {
                                    if is_dark { rgb(0x166534) } else { rgb(0x86efac) }
                                } else {
                                    border_color
                                })
                                .hover(|s| s.bg(hover_bg))
                                // Left & Middle: Status Icon + Rule Info
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap_3()
                                        // Status Icon Pill
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .size(px(36.0))
                                                .rounded_lg()
                                                .bg(if is_active {
                                                    if is_dark { rgb(0x14532d) } else { rgb(0xdcfce7) }
                                                } else {
                                                    tag_bg
                                                })
                                                .child(
                                                    svg()
                                                        .data(crate::icons::ARROW_LEFT_RIGHT_SVG)
                                                        .size(px(16.0))
                                                        .text_color(if is_active {
                                                            if is_dark { rgb(0x4ade80) } else { rgb(0x16a34a) }
                                                        } else {
                                                            muted_text
                                                        }),
                                                ),
                                        )
                                        // Information
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .gap_1()
                                                // Header row: Name + Type Badge + Active Badge
                                                .child(
                                                    div()
                                                        .flex()
                                                        .flex_row()
                                                        .items_center()
                                                        .gap_2()
                                                        .child(
                                                            div()
                                                                .text_sm()
                                                                .font_weight(FontWeight::BOLD)
                                                                .text_color(text_color)
                                                                .child(forward.name.clone()),
                                                        )
                                                        .child(
                                                            div()
                                                                .px_2()
                                                                .py_0p5()
                                                                .rounded_full()
                                                                .bg(if is_reverse {
                                                                    if is_dark { rgb(0x78350f) } else { rgb(0xfef3c7) }
                                                                } else {
                                                                    if is_dark { rgb(0x1e3a8a) } else { rgb(0xdbeafe) }
                                                                })
                                                                .text_xs()
                                                                .font_weight(FontWeight::MEDIUM)
                                                                .text_color(if is_reverse {
                                                                    if is_dark { rgb(0xfcd34d) } else { rgb(0xb45309) }
                                                                } else {
                                                                    if is_dark { rgb(0x93c5fd) } else { rgb(0x1d4ed8) }
                                                                })
                                                                .child(if is_reverse { "Reverse (ssh -R)" } else { "Local (ssh -L)" }),
                                                        )
                                                        .child(
                                                            div()
                                                                .px_2()
                                                                .py_0p5()
                                                                .rounded_full()
                                                                .bg(if is_active {
                                                                    if is_dark { rgb(0x14532d) } else { rgb(0xdcfce7) }
                                                                } else {
                                                                    tag_bg
                                                                })
                                                                .text_xs()
                                                                .font_weight(FontWeight::SEMIBOLD)
                                                                .text_color(if is_active {
                                                                    if is_dark { rgb(0x4ade80) } else { rgb(0x16a34a) }
                                                                } else {
                                                                    muted_text
                                                                })
                                                                .child(if is_active { "Active" } else { "Inactive" }),
                                                        ),
                                                )
                                                // Connection label & Mapping string
                                                .child(
                                                    div()
                                                        .flex()
                                                        .flex_row()
                                                        .items_center()
                                                        .gap_3()
                                                        .text_xs()
                                                        .child(
                                                            div()
                                                                .text_color(muted_text)
                                                                .child(format!("via: {conn_label}")),
                                                        )
                                                        .child(
                                                            div()
                                                                .font_family("JetBrains Mono")
                                                                .font_weight(FontWeight::MEDIUM)
                                                                .text_color(if is_active {
                                                                    if is_dark { rgb(0x38bdf8) } else { rgb(0x0284c7) }
                                                                } else {
                                                                    text_color
                                                                })
                                                                .child(mapping_str),
                                                        ),
                                                )
                                                // Error message (if any)
                                                .children(if has_error {
                                                    Some(
                                                        div()
                                                            .text_xs()
                                                            .text_color(rgb(0xef4444))
                                                            .child(format!("Error: {error_msg}")),
                                                    )
                                                } else {
                                                    None
                                                }),
                                        ),
                                )
                                // Right: Active Toggle Switch + Edit + Delete
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap_2()
                                        // Start/Stop Toggle Button
                                        .child(
                                            div()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .gap_1p5()
                                                .px_3()
                                                .py_1p5()
                                                .rounded_md()
                                                .cursor_pointer()
                                                .border_1()
                                                .border_color(if is_active {
                                                    if is_dark { rgb(0x22c55e) } else { rgb(0x16a34a) }
                                                } else {
                                                    border_color
                                                })
                                                .bg(if is_active {
                                                    if is_dark { rgb(0x14532d) } else { rgb(0xdcfce7) }
                                                } else {
                                                    tag_bg
                                                })
                                                .text_xs()
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .text_color(if is_active {
                                                    if is_dark { rgb(0x4ade80) } else { rgb(0x16a34a) }
                                                } else {
                                                    text_color
                                                })
                                                .child(
                                                    svg()
                                                        .data(if is_active { crate::icons::SQUARE_SVG } else { crate::icons::PLAY_SVG })
                                                        .size(px(12.0))
                                                        .text_color(if is_active {
                                                            if is_dark { rgb(0x4ade80) } else { rgb(0x16a34a) }
                                                        } else {
                                                            text_color
                                                        }),
                                                )
                                                .child(if is_active { "Stop Tunnel" } else { "Start Tunnel" })
                                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                                    this.toggle_forward_active(&id_toggle, cx);
                                                })),
                                        )
                                        // Edit Button
                                        .child(
                                            div()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .gap_1()
                                                .px_2p5()
                                                .py_1p5()
                                                .rounded_md()
                                                .cursor_pointer()
                                                .bg(tag_bg)
                                                .hover(|s| s.bg(if is_dark { rgb(0x52525b) } else { rgb(0xcbd5e1) }))
                                                .text_xs()
                                                .text_color(text_color)
                                                .child(svg().data(crate::icons::EDIT_SVG).size(px(12.0)).text_color(muted_text))
                                                .child("Edit")
                                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                                    this.open_edit_forward_modal(&id_edit, cx);
                                                })),
                                        )
                                        // Delete Button
                                        .child(
                                            div()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .gap_1()
                                                .px_2p5()
                                                .py_1p5()
                                                .rounded_md()
                                                .cursor_pointer()
                                                .bg(if is_dark { rgb(0x451a1a) } else { rgb(0xfee2e2) })
                                                .hover(|s| s.bg(if is_dark { rgb(0x7f1d1d) } else { rgb(0xfecaca) }))
                                                .text_xs()
                                                .text_color(rgb(0xef4444))
                                                .child(svg().data(crate::icons::TRASH_SVG).size(px(12.0)).text_color(rgb(0xef4444)))
                                                .child("Delete")
                                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                                    this.open_delete_forward_modal(forward_del.clone(), cx);
                                                })),
                                        ),
                                )
                        }))
                        .into_any_element()
                }),
        )
        .into_any_element()
}

/// Render Create / Edit Port Forward modal dialog overlay.
pub fn render_forward_modal(app: &mut AppState, cx: &mut Context<AppState>) -> AnyElement {
    let is_dark = app.is_dark();
    let card_bg = app.card_bg();
    let border_color = app.border_color();
    let text_color = app.text_color();
    let muted_text = app.muted_text();
    let input_bg = app.bg_color();
    let tag_bg = if is_dark { app.accent_color() } else { rgb(0xe2e8f0) };
    let primary_color = app.primary_color();
    let primary_fg = rgb(app.current_theme().primary_foreground);

    let form = match &app.forward_modal {
        Some(f) => f.clone(),
        None => return div().into_any_element(),
    };

    let is_edit = matches!(form.mode, ForwardModalMode::Edit(_));
    let title = if is_edit { "Edit Port Forward" } else { "Create Port Forward" };
    let is_reverse = form.forward_type == "reverse";

    let connections = app.connections.clone();
    let port_presets = ["3000", "5432", "6379", "8000", "8080", "9000", "27017"];

    div()
        .absolute()
        .inset_0()
        .flex()
        .items_center()
        .justify_center()
        .bg(rgba(0x00000088))
        // Dismiss on background click
        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
            this.close_forward_modal(cx);
        }))
        // Modal Card
        .child(
            div()
                .flex()
                .flex_col()
                .w(px(520.0))
                .rounded_xl()
                .bg(card_bg)
                .border_1()
                .border_color(border_color)
                .shadow_lg()
                .p_6()
                .gap_4()
                .on_mouse_down(MouseButton::Left, |_, _, _| {}) // stop propagation
                // Modal Header
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
                                .child(title),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_center()
                                .size(px(24.0))
                                .rounded_md()
                                .hover(|s| s.bg(tag_bg))
                                .cursor_pointer()
                                .text_color(muted_text)
                                .child(svg().data(crate::icons::X_SVG).size(px(14.0)).text_color(muted_text))
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                    this.close_forward_modal(cx);
                                })),
                        ),
                )
                // Error banner (if validation failed)
                .children(form.error_message.map(|err| {
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_1p5()
                        .px_3()
                        .py_2()
                        .rounded_md()
                        .bg(if is_dark { rgb(0x451a1a) } else { rgb(0xfee2e2) })
                        .border_1()
                        .border_color(rgb(0xef4444))
                        .text_xs()
                        .text_color(if is_dark { rgb(0xfca5a5) } else { rgb(0xb91c1c) })
                        .child(svg().data(crate::icons::ALERT_TRIANGLE_SVG).size(px(13.0)).text_color(rgb(0xef4444)))
                        .child(err)
                }))
                // Forward Type Selector (Local vs Reverse)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(div().text_xs().text_color(muted_text).child("Forwarding Type"))
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .gap_2()
                                // Local pill
                                .child(
                                    div()
                                        .px_3()
                                        .py_1p5()
                                        .rounded_md()
                                        .cursor_pointer()
                                        .text_xs()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .bg(if !is_reverse {
                                            if is_dark { rgb(0x0284c7) } else { rgb(0x0ea5e9) }
                                        } else {
                                            tag_bg
                                        })
                                        .text_color(if !is_reverse { rgb(0xffffff) } else { muted_text })
                                        .child("Local (ssh -L: localhost → remote)")
                                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                            if let Some(f) = &mut this.forward_modal {
                                                f.forward_type = "local".to_string();
                                                cx.notify();
                                            }
                                        })),
                                )
                                // Reverse pill
                                .child(
                                    div()
                                        .px_3()
                                        .py_1p5()
                                        .rounded_md()
                                        .cursor_pointer()
                                        .text_xs()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .bg(if is_reverse {
                                            if is_dark { rgb(0x0284c7) } else { rgb(0x0ea5e9) }
                                        } else {
                                            tag_bg
                                        })
                                        .text_color(if is_reverse { rgb(0xffffff) } else { muted_text })
                                        .child("Reverse (ssh -R: remote → localhost)")
                                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                            if let Some(f) = &mut this.forward_modal {
                                                f.forward_type = "reverse".to_string();
                                                cx.notify();
                                            }
                                        })),
                                ),
                        ),
                )
                // Rule Name Row
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(div().text_xs().text_color(muted_text).child("Rule Name *"))
                        .child(
                            div()
                                .px_3()
                                .py_1p5()
                                .rounded_md()
                                .bg(input_bg)
                                .border_1()
                                .border_color(border_color)
                                .text_sm()
                                .text_color(if form.name.is_empty() { muted_text } else { text_color })
                                .child(if form.name.is_empty() {
                                    "e.g. Local Postgres or Staging API".to_string()
                                } else {
                                    form.name.clone()
                                }),
                        )
                        // Preset names for quick selection
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_1()
                                .pt_1()
                                .child(div().text_xs().text_color(muted_text).child("Presets:"))
                                .children(["Postgres", "Redis", "MySQL", "Web API", "Mongo"].iter().map(|preset| {
                                    let p_str = preset.to_string();
                                    div()
                                        .px_2()
                                        .py_0p5()
                                        .rounded_md()
                                        .bg(tag_bg)
                                        .hover(|s| s.bg(if is_dark { rgb(0x52525b) } else { rgb(0xcbd5e1) }))
                                        .cursor_pointer()
                                        .text_xs()
                                        .text_color(text_color)
                                        .child(p_str.clone())
                                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                            if let Some(f) = &mut this.forward_modal {
                                                f.name = p_str.clone();
                                                cx.notify();
                                            }
                                        }))
                                })),
                        ),
                )
                // Connection Selector
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(div().text_xs().text_color(muted_text).child("SSH Connection *"))
                        .child(if connections.is_empty() {
                            div()
                                .p_3()
                                .rounded_md()
                                .bg(input_bg)
                                .border_1()
                                .border_color(border_color)
                                .text_xs()
                                .text_color(muted_text)
                                .child("No saved SSH connections found. Add a host in the 'Hosts' view.")
                                .into_any_element()
                        } else {
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .children(connections.into_iter().map(|conn| {
                                    let is_selected = form.connection_id == conn.id;
                                    let conn_id = conn.id.clone();
                                    div()
                                        .px_3()
                                        .py_1p5()
                                        .rounded_md()
                                        .cursor_pointer()
                                        .border_1()
                                        .border_color(if is_selected {
                                            if is_dark { rgb(0x38bdf8) } else { rgb(0x0284c7) }
                                        } else {
                                            border_color
                                        })
                                        .bg(if is_selected {
                                            if is_dark { rgb(0x0c4a6e) } else { rgb(0xe0f2fe) }
                                        } else {
                                            input_bg
                                        })
                                        .child(
                                            div()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .justify_between()
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .font_weight(if is_selected {
                                                            FontWeight::BOLD
                                                        } else {
                                                            FontWeight::NORMAL
                                                        })
                                                        .text_color(text_color)
                                                        .child(format!("{} ({}@{}:{})", conn.label, conn.username, conn.host, conn.port)),
                                                )
                                                .child(if is_selected {
                                                    div().child(svg().data(crate::icons::CHECK_SVG).size(px(14.0)).text_color(rgb(0x0284c7)))
                                                } else {
                                                    div()
                                                }),
                                        )
                                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                            if let Some(f) = &mut this.forward_modal {
                                                f.connection_id = conn_id.clone();
                                                cx.notify();
                                            }
                                        }))
                                }))
                                .into_any_element()
                        }),
                )
                // Ports Row: Local Port & Remote Port
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .gap_4()
                        // Local Port
                        .child(
                            div()
                                .flex_1()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(div().text_xs().text_color(muted_text).child("Local Port *"))
                                .child(
                                    div()
                                        .px_3()
                                        .py_1p5()
                                        .rounded_md()
                                        .bg(input_bg)
                                        .border_1()
                                        .border_color(border_color)
                                        .text_sm()
                                        .text_color(if form.local_port.is_empty() { muted_text } else { text_color })
                                        .child(if form.local_port.is_empty() { "e.g. 5432".to_string() } else { form.local_port.clone() }),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .flex_wrap()
                                        .gap_1()
                                        .pt_1()
                                        .children(port_presets.iter().map(|p| {
                                            let port_str = p.to_string();
                                            div()
                                                .px_1p5()
                                                .py_0p5()
                                                .rounded_sm()
                                                .bg(tag_bg)
                                                .hover(|s| s.bg(if is_dark { rgb(0x52525b) } else { rgb(0xcbd5e1) }))
                                                .cursor_pointer()
                                                .text_xs()
                                                .text_color(text_color)
                                                .child(port_str.clone())
                                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                                    if let Some(f) = &mut this.forward_modal {
                                                        f.local_port = port_str.clone();
                                                        if f.remote_port.is_empty() {
                                                            f.remote_port = port_str.clone();
                                                        }
                                                        cx.notify();
                                                    }
                                                }))
                                        })),
                                ),
                        )
                        // Remote Port
                        .child(
                            div()
                                .flex_1()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(div().text_xs().text_color(muted_text).child("Remote Port *"))
                                .child(
                                    div()
                                        .px_3()
                                        .py_1p5()
                                        .rounded_md()
                                        .bg(input_bg)
                                        .border_1()
                                        .border_color(border_color)
                                        .text_sm()
                                        .text_color(if form.remote_port.is_empty() { muted_text } else { text_color })
                                        .child(if form.remote_port.is_empty() { "e.g. 5432".to_string() } else { form.remote_port.clone() }),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .flex_wrap()
                                        .gap_1()
                                        .pt_1()
                                        .children(port_presets.iter().map(|p| {
                                            let port_str = p.to_string();
                                            div()
                                                .px_1p5()
                                                .py_0p5()
                                                .rounded_sm()
                                                .bg(tag_bg)
                                                .hover(|s| s.bg(if is_dark { rgb(0x52525b) } else { rgb(0xcbd5e1) }))
                                                .cursor_pointer()
                                                .text_xs()
                                                .text_color(text_color)
                                                .child(port_str.clone())
                                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                                    if let Some(f) = &mut this.forward_modal {
                                                        f.remote_port = port_str.clone();
                                                        cx.notify();
                                                    }
                                                }))
                                        })),
                                ),
                        ),
                )
                // Footer Action Buttons
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
                                .bg(tag_bg)
                                .hover(|s| s.bg(if is_dark { rgb(0x52525b) } else { rgb(0xcbd5e1) }))
                                .cursor_pointer()
                                .text_sm()
                                .text_color(text_color)
                                .child("Cancel")
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                    this.close_forward_modal(cx);
                                })),
                        )
                                .child(
                                    div()
                                        .px_4()
                                        .py_2()
                                        .rounded_md()
                                        .bg(primary_color)
                                        .hover(|s| s.opacity(0.9))
                                        .cursor_pointer()
                                        .text_sm()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(primary_fg)
                                        .child(if is_edit { "Save Changes" } else { "Create Forward" })
                                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                            this.save_forward_form(cx);
                                        })),
                                ),
                        ),
                )
                .into_any_element()
        }

        /// Render Delete Confirmation modal dialog overlay.
        pub fn render_delete_forward_modal(app: &mut AppState, cx: &mut Context<AppState>) -> AnyElement {
            let is_dark = app.is_dark();
            let card_bg = app.card_bg();
            let border_color = app.border_color();
            let text_color = app.text_color();
            let muted_text = app.muted_text();
            let tag_bg = if is_dark { app.accent_color() } else { rgb(0xe2e8f0) };

    let target = match &app.delete_forward_target {
        Some(t) => t.clone(),
        None => return div().into_any_element(),
    };

    div()
        .absolute()
        .inset_0()
        .flex()
        .items_center()
        .justify_center()
        .bg(rgba(0x00000088))
        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
            this.close_delete_forward_modal(cx);
        }))
        .child(
            div()
                .flex()
                .flex_col()
                .w(px(440.0))
                .rounded_xl()
                .bg(card_bg)
                .border_1()
                .border_color(border_color)
                .shadow_lg()
                .p_6()
                .gap_4()
                .on_mouse_down(MouseButton::Left, |_, _, _| {})
                .child(
                    div()
                        .text_lg()
                        .font_weight(FontWeight::BOLD)
                        .text_color(text_color)
                        .child("Delete Port Forward"),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(muted_text)
                        .child(format!(
                            "Are you sure you want to delete '{}'? Any active tunnel will be immediately terminated. This action cannot be undone.",
                            target.name
                        )),
                )
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
                                .bg(tag_bg)
                                .hover(|s| s.bg(if is_dark { rgb(0x52525b) } else { rgb(0xcbd5e1) }))
                                .cursor_pointer()
                                .text_sm()
                                .text_color(text_color)
                                .child("Cancel")
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                    this.close_delete_forward_modal(cx);
                                })),
                        )
                        .child(
                            div()
                                .px_4()
                                .py_2()
                                .rounded_md()
                                .bg(rgb(0xef4444))
                                .hover(|s| s.bg(rgb(0xdc2626)))
                                .cursor_pointer()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(rgb(0xffffff))
                                .child("Delete Rule")
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                    this.confirm_delete_forward(cx);
                                })),
                        ),
                ),
        )
        .into_any_element()
}
