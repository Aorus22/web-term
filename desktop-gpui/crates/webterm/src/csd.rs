//! Client-side-decoration frame geometry.
//!
//! The window is transparent + `WindowDecorations::Client`, and the visible
//! "card" is inset from the window rectangle by [`WINDOW_SHADOW_MARGIN`] on
//! every side. That margin exists for two reasons:
//!
//! 1. It is where we paint our **own** shadow ([`window_shadow`]) with
//!    `BoxShadow`, which gpui draws following the corner radius.
//! 2. It is what we advertise to X as `_GTK_FRAME_EXTENTS`
//!    ([`advertise_frame_extents`]), so mutter treats the *inset* frame as the
//!    window instead of drawing its own shadow around the square rectangle —
//!    that square shadow is exactly the dark "wedge" that used to show through
//!    the transparent rounded corners.
//!
//! # Measured mutter behaviour (X11)
//!
//! * `_GTK_FRAME_EXTENTS` is in **device pixels**, not logical: a 16 logical
//!   margin is `32,32,32,32` at scale factor 2.
//! * Mutter **grows the client by the extents**: `client = requested + 2e`
//!   (measured `1024×1104 → 1056×1136` for `e = 16`). The window size the app
//!   *requests* must therefore be the visible card alone; asking for the card
//!   plus the margin would double it. `window_state` compensates when
//!   persisting geometry ([`extents_active`]).
//! * Writing the same value repeatedly is a no-op; a different value shifts
//!   the window incrementally. The retry loop is needed because the X window
//!   only exists after the view is created, and the property is allowed to
//!   arrive after the map (mutter recomputes on `PropertyNotify`).
//!
//! Everything X11-specific is best-effort: on native Wayland, or when GTK/X is
//! unavailable, the functions here fail quietly and the app keeps its margin
//! and its own shadow.

use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(target_os = "linux")]
use std::time::Duration;

use gpui::{px, rgba, BoxShadow};
#[cfg(target_os = "linux")]
use x11rb::connection::Connection;
#[cfg(target_os = "linux")]
use x11rb::protocol::xproto::{AtomEnum, ConnectionExt, PropMode};
#[cfg(target_os = "linux")]
use x11rb::rust_connection::RustConnection;
#[cfg(target_os = "linux")]
use x11rb::wrapper::ConnectionExt as _;

/// Transparent margin reserved around the visible card, in **logical** px.
///
/// Big enough for the painted shadow's blur to fade out before the window edge
/// (a clipped shadow edge would give the margin away), small enough not to read
/// as padding.
pub const WINDOW_SHADOW_MARGIN: f32 = 16.0;

/// Which X11 window the extents belong to — the main window's title.
pub const MAIN_WINDOW_TITLE: &str = "WebTerm Desktop";

/// The shadow painted into [`WINDOW_SHADOW_MARGIN`].
///
/// Roughly GTK's `0 4px 12px rgba(0, 0, 0, .45)`. `BoxShadow` follows the
/// element's corner radius in gpui, so the shadow is rounded like the card even
/// though the window rectangle is square.
pub fn window_shadow() -> Vec<BoxShadow> {
    vec![BoxShadow::new(px(0.), px(4.), rgba(0x00000073).into()).blur_radius(px(12.))]
}

static EXTENTS_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Whether `_GTK_FRAME_EXTENTS` was successfully written to this process's
/// window. When true, mutter grew the client by `2 * margin`, so persisted
/// window geometry is stored in card coordinates (see `window_state`).
pub fn extents_active() -> bool {
    EXTENTS_ACTIVE.load(Ordering::SeqCst)
}

/// Advertise `_GTK_FRAME_EXTENTS = [margin; 4]` for the window titled `title`,
/// retrying in the background until the X window exists.
///
/// Non-fatal: on Wayland (or without X) there is simply no window to set it on.
#[cfg(target_os = "linux")]
pub fn advertise_frame_extents(title: &'static str, scale_factor: f32) {
    // Device pixels: GTK scales this property too.
    let margin = ((WINDOW_SHADOW_MARGIN * scale_factor).round() as i64).max(1) as u32;
    std::thread::spawn(move || {
        for _ in 0..40 {
            if set_frame_extents(title, margin).is_ok() {
                EXTENTS_ACTIVE.store(true, Ordering::SeqCst);
                return;
            }
            std::thread::sleep(Duration::from_millis(150));
        }
        eprintln!(
            "[webterm] could not advertise _GTK_FRAME_EXTENTS for {title:?}; \
             mutter will draw its own square shadow behind the rounded corners"
        );
    });
}

/// No-op off Linux: `_GTK_FRAME_EXTENTS` is an X11 convention, and the Windows
/// compositor never grows the client, so the margin stays purely visual.
#[cfg(not(target_os = "linux"))]
pub fn advertise_frame_extents(_title: &'static str, _scale_factor: f32) {}

/// Sets `_GTK_FRAME_EXTENTS = [margin; 4]` on the window whose title matches
/// `title` (this process only).
#[cfg(target_os = "linux")]
pub fn set_frame_extents(title: &str, margin: u32) -> Result<(), String> {
    let (conn, root) = connect_root()?;
    let window = locate_window_by_title(&conn, root, title)?;
    let gtk_frame_extents = conn
        .intern_atom(false, b"_GTK_FRAME_EXTENTS")
        .map_err(|error| format!("intern_atom error: {error}"))?
        .reply()
        .map_err(|error| format!("intern_atom reply: {error}"))?
        .atom;
    conn.change_property32(
        PropMode::REPLACE,
        window,
        gtk_frame_extents,
        AtomEnum::CARDINAL,
        &[margin; 4],
    )
    .map_err(|error| format!("ChangeProperty error: {error}"))?
    .check()
    .map_err(|error| format!("ChangeProperty rejected: {error}"))?;
    conn.flush()
        .map_err(|error| format!("flush error: {error}"))?;
    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn set_frame_extents(_title: &str, _margin: u32) -> Result<(), String> {
    Err("_GTK_FRAME_EXTENTS is X11-only".to_string())
}

#[cfg(target_os = "linux")]
fn connect_root() -> Result<(RustConnection, u32), String> {
    let (conn, screen_num) =
        x11rb::connect(None).map_err(|error| format!("X connect error: {error}"))?;
    let root = conn
        .setup()
        .roots
        .get(screen_num)
        .map(|screen| screen.root)
        .ok_or_else(|| format!("no root window for screen {screen_num}"))?;
    Ok((conn, root))
}

/// Find one of this process's windows by its `_NET_WM_NAME` / `WM_NAME`.
///
/// Title matching survives the size change that comes with the shadow margin
/// (the popup locator's WM_CLASS + size heuristic would not).
#[cfg(target_os = "linux")]
fn locate_window_by_title(
    conn: &RustConnection,
    root: u32,
    title: &str,
) -> Result<u32, String> {
    let tree = conn
        .query_tree(root)
        .map_err(|error| format!("query_tree error: {error}"))?
        .reply()
        .map_err(|error| format!("query_tree reply: {error}"))?;
    let net_wm_pid = conn
        .intern_atom(false, b"_NET_WM_PID")
        .map_err(|error| format!("intern_atom error: {error}"))?
        .reply()
        .map_err(|error| format!("intern_atom reply: {error}"))?
        .atom;
    for window in tree.children {
        if window_pid(conn, net_wm_pid, window) != Some(std::process::id()) {
            continue;
        }
        if window_name(conn, window).as_deref() == Some(title) {
            return Ok(window);
        }
    }
    Err(format!("no window titled {title:?} among this pid's windows"))
}

/// `_NET_WM_NAME` (UTF8_STRING), falling back to the legacy `WM_NAME`.
#[cfg(target_os = "linux")]
fn window_name(conn: &RustConnection, window: u32) -> Option<String> {
    let utf8 = || -> Option<String> {
        let atom = conn.intern_atom(false, b"UTF8_STRING").ok()?.reply().ok()?.atom;
        let net_wm_name = conn
            .intern_atom(false, b"_NET_WM_NAME")
            .ok()?
            .reply()
            .ok()?
            .atom;
        let reply = conn
            .get_property(false, window, net_wm_name, atom, 0, 512)
            .ok()?
            .reply()
            .ok()?;
        if reply.value.is_empty() {
            return None;
        }
        String::from_utf8(reply.value).ok()
    };
    utf8().or_else(|| {
        let reply = conn
            .get_property(false, window, AtomEnum::WM_NAME, AtomEnum::STRING, 0, 512)
            .ok()?
            .reply()
            .ok()?;
        String::from_utf8(reply.value).ok()
    })
}

#[cfg(target_os = "linux")]
fn window_pid(conn: &RustConnection, net_wm_pid: u32, window: u32) -> Option<u32> {
    let reply = conn
        .get_property(false, window, net_wm_pid, AtomEnum::CARDINAL, 0, 1)
        .ok()?
        .reply()
        .ok()?;
    reply.value32().and_then(|mut values| values.next())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shadow_fits_inside_the_reserved_margin() {
        // A clipped shadow edge would give the transparent margin away: the
        // blur plus the downward offset must stay within the margin.
        for shadow in window_shadow() {
            let blur: f32 = shadow.blur_radius.into();
            let offset_y: f32 = shadow.offset.y.into();
            let spread: f32 = shadow.spread_radius.into();
            assert!(
                offset_y + blur + spread <= WINDOW_SHADOW_MARGIN,
                "shadow (offset {offset_y} + blur {blur} + spread {spread}) \
                 must fit in {WINDOW_SHADOW_MARGIN} logical px"
            );
        }
    }

    #[test]
    fn margin_scales_to_device_pixels_for_extents() {
        // The property is in device pixels: at the machine's scale factor 2 the
        // 16 logical margin must be advertised as 32.
        assert_eq!((WINDOW_SHADOW_MARGIN * 2.0).round() as u32, 32);
        assert_eq!((WINDOW_SHADOW_MARGIN * 1.0).round() as u32, 16);
    }
}
