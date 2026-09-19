//! Shared right-side sheet (push-aside drawer) primitive.
//!
//! Ported from the wa-bot GPUI app's drawer: the slot's width animates while
//! the sheet keeps its full size and is clipped, so the content beside it
//! narrows smoothly instead of being covered by an overlay.

use gpui::*;
use gpui_component::input::{Input, InputState, Textarea, TextareaState};
use gpui_component::Sizable;

use crate::app_state::AppState;

pub const SHEET_WIDTH: f32 = 540.0;
pub const SHEET_ANIM_MS: u64 = 240;

/// Quintic ease-out animation shared by all sheets.
pub fn sheet_animation() -> Animation {
    Animation::new(std::time::Duration::from_millis(SHEET_ANIM_MS))
        .with_easing(|d| 1.0 - (1.0 - d).powi(5))
}

/// Wrap a right-hand sheet so it plays the wa-bot sheet motion in both
/// directions: the slot's width animates while the sheet keeps its full size
/// and is clipped, so the view beside it narrows smoothly instead of snapping.
///
/// The open and close ids differ so mounting the closing element restarts the
/// animation at 0, which the exit reads as "still fully open" before shrinking
/// it away.
pub fn animated_sheet(
    open_id: &'static str,
    close_id: &'static str,
    closing: bool,
    sheet: AnyElement,
) -> AnyElement {
    let animation = sheet_animation();
    let slot = div()
        .h_full()
        .flex_shrink_0()
        .overflow_hidden()
        .child(sheet);
    if closing {
        slot.with_animation(close_id, animation, |el, delta| {
            el.w(px(SHEET_WIDTH * (1.0 - delta).clamp(0.0, 1.0)))
        })
        .into_any_element()
    } else {
        slot.with_animation(open_id, animation, |el, delta| {
            el.w(px(SHEET_WIDTH * delta.clamp(0.0, 1.0)))
        })
        .into_any_element()
    }
}

/// Fixed-width sheet panel body: full height, card background, left border.
pub fn sheet_panel(app: &AppState) -> Div {
    div()
        .flex()
        .flex_col()
        .w(px(SHEET_WIDTH))
        .min_w(px(SHEET_WIDTH))
        .max_w(px(SHEET_WIDTH))
        .h_full()
        .flex_shrink_0()
        .overflow_hidden()
        .bg(app.card_bg())
        .border_l_1()
        .border_color(app.border_color())
}

/// Sheet header: title + description on the left, X close button on the right.
/// `close` is an AppState close method invoked by the X button.
pub fn sheet_header(
    app: &AppState,
    cx: &mut Context<AppState>,
    title: &'static str,
    description: &'static str,
    close: fn(&mut AppState, &mut Context<AppState>),
) -> Div {
    let text_color = app.text_color();
    let muted_text = app.muted_text();
    let tag_bg = app.muted_bg();

    div()
        .flex()
        .flex_row()
        .items_start()
        .justify_between()
        .px_5()
        .py_4()
        .border_b_1()
        .border_color(app.border_color())
        .child(
            div()
                .flex()
                .flex_col()
                .gap_0p5()
                .child(
                    div()
                        .text_base()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(text_color)
                        .child(title),
                )
                .child(div().text_xs().text_color(muted_text).child(description)),
        )
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .size(px(24.0))
                .rounded_md()
                .id("sheet-01").hover(move |s| s.bg(tag_bg))
                .cursor_pointer()
                .text_color(muted_text)
                .child(
                    svg()
                        .data(crate::icons::X_SVG)
                        .size(px(14.0))
                        .text_color(muted_text),
                )
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, _window, cx| {
                        close(this, cx);
                    }),
                ),
        )
}

/// Scrollable sheet body column.
pub fn sheet_body() -> Stateful<Div> {
    div()
        .id("sheet-body")
        .flex()
        .flex_col()
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .px_5()
        .py_4()
        .gap_3()
}

/// Sheet footer pinned to the bottom: Cancel + primary action, matching the
/// web SheetFooter (border-t, right-aligned buttons). Handlers receive the
/// window so they can read/replace text input values.
pub fn sheet_footer(
    app: &AppState,
    cx: &mut Context<AppState>,
    cancel_label: &'static str,
    submit_label: &'static str,
    on_cancel: impl Fn(&mut AppState, &mut Window, &mut Context<AppState>) + 'static,
    on_submit: impl Fn(&mut AppState, &mut Window, &mut Context<AppState>) + 'static,
) -> Div {
    let border_color = app.border_color();
    let text_color = app.text_color();
    let tag_bg = app.muted_bg();
    let primary_color = app.primary_color();
    let primary_fg = app.primary_fg();

    div()
        .mt_auto()
        .flex()
        .flex_row()
        .justify_end()
        .gap_2()
        .px_5()
        .py_4()
        .border_t_1()
        .border_color(border_color)
        .child(
            div()
                .px_4()
                .py_2()
                .rounded_md()
                .bg(tag_bg)
                .id("sheet-02").hover(move |s| s.bg(border_color))
                .cursor_pointer()
                .text_sm()
                .text_color(text_color)
                .child(cancel_label)
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, window, cx| {
                        on_cancel(this, window, cx);
                    }),
                ),
        )
        .child(
            div()
                .px_4()
                .py_2()
                .rounded_md()
                .bg(primary_color)
                .id("sheet-03").hover(|s| s.opacity(0.9))
                .cursor_pointer()
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(primary_fg)
                .child(submit_label)
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, window, cx| {
                        on_submit(this, window, cx);
                    }),
                ),
        )
}

/// Error banner shared by sheet forms (validation / request failures).
pub fn sheet_error_banner(app: &AppState, message: SharedString) -> Div {
    let is_dark = app.is_dark();
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_1p5()
        .px_3()
        .py_2()
        .rounded_md()
        .bg(if is_dark {
            rgb(0x451a1a)
        } else {
            rgb(0xfee2e2)
        })
        .border_1()
        .border_color(rgb(0xef4444))
        .text_xs()
        .text_color(if is_dark {
            rgb(0xfca5a5)
        } else {
            rgb(0xb91c1c)
        })
        .child(
            svg()
                .data(crate::icons::ALERT_TRIANGLE_SVG)
                .size(px(13.0))
                .text_color(rgb(0xef4444)),
        )
        .child(message)
}

/// Labeled single-line text input row for sheet forms.
pub fn sheet_input_row(
    app: &AppState,
    label: impl Into<SharedString>,
    input: &Entity<InputState>,
) -> Div {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .text_xs()
                .text_color(app.muted_text())
                .child(label.into()),
        )
        .child(
            Input::new(input)
                .with_size(gpui_component::Size::Small)
                .w_full()
                .text_size(px(13.0)),
        )
}

/// Labeled multi-line textarea row for sheet forms (PEM content, etc.).
pub fn sheet_textarea_row(
    app: &AppState,
    label: impl Into<SharedString>,
    input: &Entity<TextareaState>,
    height: f32,
) -> Div {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .text_xs()
                .text_color(app.muted_text())
                .child(label.into()),
        )
        .child(
            Textarea::new(input)
                .w_full()
                .h(px(height))
                .text_size(px(12.0)),
        )
}
