//! Port Forwarding rules management view: rule cards, toggle, create/edit sheet, and deletion modal.

use crate::app_state::{AppState, ForwardModalMode};
use crate::views::sheet::{
    sheet_body, sheet_error_banner, sheet_footer, sheet_header, sheet_input_row, sheet_panel,
};
use gpui::*;

/// Render the Port Forwarding rules management view.
pub fn render_forwards_view(app: &mut AppState, cx: &mut Context<AppState>) -> AnyElement {
    let bg_color = app.bg_color();
    let toolbar_bg = app.bg_color();
    let card_bg = app.card_bg();
    let border_color = app.border_color();
    let text_color = app.text_color();
    let muted_text = app.muted_text();
    let muted_bg = app.muted_bg();
    let destructive_color = app.destructive_color();
    let primary_color = app.primary_color();
    let primary_fg = app.primary_fg();

    let forwards = app.forwards.clone();
    let count = forwards.len();
    let is_loading = app.is_loading_forwards;

    // The forward sheet squeezes the toolbar; hiding the action button keeps
    // it from clipping behind the sheet edge.
    let sheet_open = app.forward_modal.is_some() || app.forward_sheet_closing;

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
        // Top Toolbar matching fe/ PortForwardsPage header
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
                // Left: Title + "SSH TUNNELING" subtitle
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_4()
                        .child(
                            div()
                                .text_lg()
                                .font_weight(FontWeight::BOLD)
                                .text_color(text_color)
                                .child("Port Forwards"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(muted_text)
                                .child("SSH TUNNELING"),
                        ),
                )
                // Right: Action (+ Create Forward, hidden while a sheet is open)
                .children(if sheet_open {
                    None
                } else {
                    Some(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_1p5()
                            .px_3p5()
                            .py_1p5()
                            .rounded_lg()
                            .bg(primary_color)
                            .id("forwards-01")
                            .hover(|s| s.opacity(0.9))
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(primary_fg)
                            .cursor_pointer()
                            .child(svg().data(crate::icons::PLUS_SVG).size(px(13.0)).text_color(primary_fg))
                            .child("Create Forward")
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, window, cx| {
                                this.open_create_forward_modal(window, cx);
                            }))
                            .into_any_element(),
                    )
                }),
        )
        // Main Content Area
        .child(
            div()
                .id("forwards-scroll-area")
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
                        .rounded_xl()
                        .border_1()
                        .border_dashed()
                        .border_color(border_color)
                        .child(
                            div()
                                .size(px(48.0))
                                .rounded_full()
                                .bg(muted_bg)
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
                                .id("forwards-02").hover(|s| s.opacity(0.9))
                                .cursor_pointer()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(primary_fg)
                                .child(svg().data(crate::icons::PLUS_SVG).size(px(13.0)).text_color(primary_fg))
                                .child("Create First Forward")
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, window, cx| {
                                    this.open_create_forward_modal(window, cx);
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
                                .py_3()
                                .px_4()
                                .gap_4()
                                .rounded_xl()
                                .bg(card_bg)
                                .border_1()
                                .border_color(if is_active {
                                    primary_color
                                } else {
                                    border_color
                                })
                                .id(ElementId::Name(format!("fwd-card-{}", forward.id).into())).hover(move |s| s.border_color(primary_color))
                                // Left & Middle: Status Icon + Rule Info
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap_3()
                                        .flex_1()
                                        .min_w_0()
                                        // Status Icon Pill
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .size(px(36.0))
                                                .rounded_md()
                                                .bg(muted_bg)
                                                .flex_shrink_0()
                                                .child(
                                                    svg()
                                                        .data(crate::icons::ARROW_LEFT_RIGHT_SVG)
                                                        .size(px(16.0))
                                                        .text_color(if is_active {
                                                            primary_color
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
                                                .flex_1()
                                                .min_w_0()
                                                .gap_0p5()
                                                // Header row: Name + Type Badge + Active Badge
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
                                                                .text_color(text_color)
                                                                .truncate()
                                                                .child(forward.name.clone()),
                                                        )
                                                        .children(if is_reverse {
                                                            Some(
                                                                div()
                                                                    .px_1p5()
                                                                    .py_0()
                                                                    .rounded_sm()
                                                                    .text_xs()
                                                                    .font_weight(FontWeight::MEDIUM)
                                                                    .bg(muted_bg)
                                                                    .text_color(rgb(0xd97706))
                                                                    .child("Reverse"),
                                                            )
                                                        } else {
                                                            None
                                                        })
                                                        .children(if is_active {
                                                            Some(
                                                                div()
                                                                    .px_1p5()
                                                                    .py_0()
                                                                    .rounded_sm()
                                                                    .text_xs()
                                                                    .font_weight(FontWeight::MEDIUM)
                                                                    .bg(muted_bg)
                                                                    .text_color(primary_color)
                                                                    .child("Active"),
                                                            )
                                                        } else {
                                                            None
                                                        }),
                                                )
                                                // Connection label
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(muted_text)
                                                        .truncate()
                                                        .child(conn_label),
                                                )
                                                // Mapping string
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(muted_text)
                                                        .truncate()
                                                        .child(mapping_str),
                                                )
                                                // Error message (if any)
                                                .children(if has_error {
                                                    Some(
                                                        div()
                                                            .text_xs()
                                                            .text_color(destructive_color)
                                                            .truncate()
                                                            .child(format!("Error: {error_msg}")),
                                                    )
                                                } else {
                                                    None
                                                }),
                                        ),
                                )
                                // Right: Modern Switch Toggle + Edit + Delete
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap_3()
                                        .flex_shrink_0()
                                        // Modern Switch Toggle
                                        .child(
                                            div()
                                                .w(px(40.0))
                                                .h(px(22.0))
                                                .rounded_full()
                                                .cursor_pointer()
                                                .bg(if is_active { primary_color } else { muted_bg })
                                                .border_1()
                                                .border_color(if is_active { primary_color } else { border_color })
                                                .p(px(2.0))
                                                .flex()
                                                .items_center()
                                                .child(
                                                    if is_active {
                                                        div()
                                                            .size(px(16.0))
                                                            .rounded_full()
                                                            .bg(primary_fg)
                                                            .shadow_sm()
                                                            .ml_auto()
                                                    } else {
                                                        div()
                                                            .size(px(16.0))
                                                            .rounded_full()
                                                            .bg(muted_text)
                                                            .shadow_sm()
                                                    }
                                                )
                                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                                    this.toggle_forward_active(&id_toggle, cx);
                                                })),
                                        )
                                        // Edit Button
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .size(px(26.0))
                                                .rounded_md()
                                                .cursor_pointer()
                                                .id("forwards-04").hover(move |s| s.bg(muted_bg))
                                                .child(svg().data(crate::icons::EDIT_SVG).size(px(14.0)).text_color(muted_text))
                                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, window, cx| {
                                                    this.open_edit_forward_modal(&id_edit, window, cx);
                                                })),
                                        )
                                        // Delete Button
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .size(px(26.0))
                                                .rounded_md()
                                                .cursor_pointer()
                                                .id("forwards-05").hover(move |s| s.bg(muted_bg))
                                                .child(svg().data(crate::icons::TRASH_SVG).size(px(14.0)).text_color(muted_text))
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

/// Render Create / Edit Port Forward sheet (right-side push-aside panel).
pub fn render_forward_sheet(app: &mut AppState, cx: &mut Context<AppState>) -> AnyElement {
    let is_dark = app.is_dark();
    let border_color = app.border_color();
    let text_color = app.text_color();
    let muted_text = app.muted_text();
    let input_bg = app.bg_color();
    let tag_bg = app.muted_bg();

    let form = match &app.forward_modal {
        Some(f) => f.clone(),
        None => return div().into_any_element(),
    };

    let is_edit = matches!(form.mode, ForwardModalMode::Edit(_));
    let title = if is_edit {
        "Edit Port Forward"
    } else {
        "Create Port Forward"
    };
    let is_reverse = form.forward_type == "reverse";

    let connections = app.connections.clone();
    let port_presets = ["3000", "5432", "6379", "8000", "8080", "9000", "27017"];

    sheet_panel(app)
        .child(sheet_header(
            app,
            cx,
            title,
            if is_edit {
                "Modify the port forwarding rule."
            } else {
                "Set up an SSH port forwarding tunnel."
            },
            AppState::close_forward_modal,
        ))
        .children(form
            .error_message
            .map(|err| sheet_error_banner(app, err.into()).into_any_element()))
        // Scrollable form body
        .child(sheet_body()
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
                        .child(sheet_input_row(app, "Rule Name *", &app.inputs().fwd_name))
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
                                        .id(ElementId::Name(format!("fwd-preset-{}", preset).into())).hover(|s| s.bg(if is_dark { rgb(0x52525b) } else { rgb(0xcbd5e1) }))
                                        .cursor_pointer()
                                        .text_xs()
                                        .text_color(text_color)
                                        .child(p_str.clone())
                                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, window, cx| {
                                            AppState::set_input_value(&this.inputs().fwd_name, &p_str, window, cx);
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
                                .child(sheet_input_row(app, "Local Port *", &app.inputs().fwd_local_port))
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
                                                .id(ElementId::Name(format!("fwd-lport-{}", p).into())).hover(|s| s.bg(if is_dark { rgb(0x52525b) } else { rgb(0xcbd5e1) }))
                                                .cursor_pointer()
                                                .text_xs()
                                                .text_color(text_color)
                                                .child(port_str.clone())
                                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, window, cx| {
                                                    AppState::set_input_value(&this.inputs().fwd_local_port, &port_str, window, cx);
                                                    let remote = AppState::input_value(&this.inputs().fwd_remote_port, cx);
                                                    if remote.trim().is_empty() {
                                                        AppState::set_input_value(&this.inputs().fwd_remote_port, &port_str, window, cx);
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
                                .child(sheet_input_row(app, "Remote Port *", &app.inputs().fwd_remote_port))
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
                                                .id(ElementId::Name(format!("fwd-rport-{}", p).into())).hover(|s| s.bg(if is_dark { rgb(0x52525b) } else { rgb(0xcbd5e1) }))
                                                .cursor_pointer()
                                                .text_xs()
                                                .text_color(text_color)
                                                .child(port_str.clone())
                                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, window, cx| {
                                                    AppState::set_input_value(&this.inputs().fwd_remote_port, &port_str, window, cx);
                                                }))
                                        })),
                                ),
                        ),
                )
        )
        // Footer
        .child(sheet_footer(
            app,
            cx,
            "Cancel",
            if is_edit { "Save Changes" } else { "Create Forward" },
            |this, _window, cx| this.close_forward_modal(cx),
            |this, window, cx| this.save_forward_form(window, cx),
        ))
        .into_any_element()
}

/// Render Delete Confirmation modal dialog overlay.
pub fn render_delete_forward_modal(app: &mut AppState, cx: &mut Context<AppState>) -> AnyElement {
    let is_dark = app.is_dark();
    let card_bg = app.card_bg();
    let border_color = app.border_color();
    let text_color = app.text_color();
    let muted_text = app.muted_text();
    let tag_bg = app.muted_bg();

    let target = match &app.delete_forward_target {
        Some(t) => t.clone(),
        None => return div().into_any_element(),
    };

    div()
        .absolute()
        .inset_0()
        // BlockMouse: keeps clicks/hovers/scroll from reaching the page behind.
        .occlude()
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
                                .id("forwards-09").hover(|s| s.bg(if is_dark { rgb(0x52525b) } else { rgb(0xcbd5e1) }))
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
                                .id("forwards-10").hover(|s| s.bg(rgb(0xdc2626)))
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
