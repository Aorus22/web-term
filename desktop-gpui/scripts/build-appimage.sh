#!/usr/bin/env bash
# Build portable AppImages for the WebTerm GPUI frontend.
#
# Two variants are produced from the same release binary, because linking the
# `gtk` crate for the desktop-theme probe (crates/webterm/src/gtk_theme.rs)
# makes libgtk-3.so.0 a hard DT_NEEDED of the client:
#
#   WebTerm-GPUI_<version>_<arch>.AppImage
#       LEAN. The GTK3 stack stays on the host: linuxdeploy is told to skip
#       libgtk-3.so.0 / libglib-2.0.so.0 / libgobject-2.0.so.0, and the deployed
#       tree is then pruned back to the libraries the client links directly
#       (libxkbcommon, libxkbcommon-x11). Smallest artifact, but the target host
#       must provide GTK3 — the client links it unconditionally.
#
#   WebTerm-GPUI_<version>_<arch>-gtk.AppImage
#       GTK3 BUNDLED. linuxdeploy walks the GTK3 dependency tree, so the app
#       starts on a host with no GTK3 at all. ~9 MB larger.
#
#   ./build-appimage.sh                 # build both variants
#   ./build-appimage.sh --lean          # only the lean one
#   ./build-appimage.sh --gtk-bundled   # only the GTK-bundled one
#   ./build-appimage.sh --install       # build, then register for this user
#   ./build-appimage.sh --clean         # drop the staged AppDirs
#
# Layout of each produced AppDir:
#   usr/bin/webterm (+ webterm-backend side-by-side)
#   usr/share/webterm-gpui/assets/{fonts/}
#   usr/share/applications/webterm-gpui.desktop
#   usr/share/icons/hicolor/{128x128,256x256,scalable}/apps/webterm-gpui.*
#
# Both helper tools are downloaded (and cached) as AppImages and run with
# `--appimage-extract-and-run`, so no FUSE/libfuse2 is required.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GPUI_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
ROOT="$(cd "${GPUI_DIR}/.." && pwd)"
APP_ID="webterm-gpui"
BIN_NAME="webterm"
BACKEND_NAME="webterm-backend"
APPIMAGE_ROOT="$GPUI_DIR/target/appimage"
DIST_DIR="$GPUI_DIR/dist"
CACHE_DIR="${XDG_CACHE_HOME:-$HOME/.cache}/webterm-appimage"

LINUXDEPLOY_URL="${LINUXDEPLOY_URL:-https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage}"
APPIMAGETOOL_URL="${APPIMAGETOOL_URL:-https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage}"

# Variant selection. No flag means "both": that is what CI packages, and it is
# the only way to compare the two bundles from one run.
WANT_LEAN=0
WANT_GTK=0
INSTALL=0
for arg in "$@"; do
    case "$arg" in
        --lean) WANT_LEAN=1 ;;
        --gtk-bundled) WANT_GTK=1 ;;
        --install) INSTALL=1 ;;
        --clean)
            echo "Removing $APPIMAGE_ROOT"
            rm -rf "$APPIMAGE_ROOT"
            exit 0
            ;;
        -h|--help)
            sed -n '2,/^$/p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *)
            echo "Unknown option: $arg" >&2
            exit 2
            ;;
    esac
done
if [[ "$WANT_LEAN" == 0 && "$WANT_GTK" == 0 ]]; then
    WANT_LEAN=1
    WANT_GTK=1
fi

log() { printf '\033[0;36m[appimage]\033[0m %s\n' "$1"; }
fail() { printf '\033[0;31m[appimage] %s\033[0m\n' "$1" >&2; exit 1; }

command -v cargo >/dev/null || fail "cargo not found (install Rust: https://rustup.rs)"
command -v go >/dev/null || fail "go not found"
command -v mksquashfs >/dev/null || fail "mksquashfs not found (install squashfs-tools)"

VERSION="$(grep -m1 '^version' "$GPUI_DIR/Cargo.toml" | sed 's/.*"\(.*\)".*/\1/')"
ARCH="$(uname -m)"
[[ "$ARCH" == "x86_64" ]] || fail "only x86_64 is supported by the bundled tools (got $ARCH)"

# --- Variant table ----------------------------------------------------------

variant_app_dir() { echo "$APPIMAGE_ROOT/$APP_ID-$1.AppDir"; }

variant_out_name() {
    case "$1" in
        lean) echo "WebTerm-GPUI_${VERSION}_${ARCH}.AppImage" ;;
        gtk) echo "WebTerm-GPUI_${VERSION}_${ARCH}-gtk.AppImage" ;;
    esac
}

variant_label() {
    case "$1" in
        lean) echo "lean (GTK3 from the host)" ;;
        gtk) echo "GTK3 bundled (runs without host GTK3)" ;;
    esac
}

# Libraries the lean variant deliberately leaves to the host.
#
# These are exactly the GTK-family entries in the client's `DT_NEEDED` list: the
# gtk-rs stack links gtk-3 plus glib and gobject directly, and the whole rest of
# the GTK3 world (gdk, pango, atk, cairo, gio, plus the X11/Wayland/dbus
# libraries hanging off them) is only reachable *through* them.
#
# `objdump -p target/release/webterm | grep NEEDED` is the source of truth; the
# deploy step below fails the build if the result ever drifts from this contract.
variant_excludes() {
    case "$1" in
        lean)
            printf '%s\n' \
                libgtk-3.so.0 \
                libglib-2.0.so.0 \
                libgobject-2.0.so.0
            ;;
    esac
}

# Reduce usr/lib to the libraries the client links directly, minus the ones the
# host provides (variant_excludes).
#
# `--exclude-library` is not enough on its own: linuxdeploy walks the entire
# dependency closure and only filters the excluded *names* out of the copy, so
# skipping libgtk-3.so.0 still leaves ~53 GTK libraries behind (gdk, pango, atk,
# cairo, gio, ...). For a genuinely lean bundle the tree is pruned afterwards.
prune_to_direct_deps() {
    local variant="$1" app_dir="$2"

    # The keep list: the client's direct dependencies minus the host-provided ones.
    local keep_list
    keep_list="$(objdump -p "$BIN" | awk '/NEEDED/ { print $2 }' \
        | grep -vxF -f <(variant_excludes "$variant"))"

    local kept=() dropped=0
    local lib base
    for lib in "$app_dir"/usr/lib/*; do
        [[ -e "$lib" ]] || continue
        base="$(basename "$lib")"
        if printf '%s\n' "$keep_list" | grep -Fxq "$base"; then
            kept+=("$base")
        else
            rm -f "$lib"
            dropped=$((dropped + 1))
        fi
    done

    log "$variant: dropped $dropped host-side libs, kept: ${kept[*]:-none}"
}

fetch_tool() {
    local url="$1" dest="$2"
    if [[ -x "$dest" ]]; then
        return
    fi
    log "Downloading $(basename "$dest")"
    mkdir -p "$(dirname "$dest")"
    curl -fL --retry 3 -o "$dest" "$url" || fail "download failed: $url"
    chmod +x "$dest"
}

run_appimage_tool() {
    "$1" --appimage-extract-and-run "${@:2}"
}

# --- Build (once) -----------------------------------------------------------

log "Building Go backend (release)"
(cd "$ROOT/be" && go build -ldflags "-s -w" -o "$GPUI_DIR/target/release/$BACKEND_NAME" ./cmd/server)

log "Building release binary (cargo build --release -p webterm)"
cargo build --release --manifest-path "$GPUI_DIR/Cargo.toml" --package webterm
BIN="$GPUI_DIR/target/release/$BIN_NAME"
[[ -x "$BIN" ]] || fail "release binary missing: $BIN"
BACKEND_BIN="$GPUI_DIR/target/release/$BACKEND_NAME"

fetch_tool "$LINUXDEPLOY_URL" "$CACHE_DIR/linuxdeploy-$ARCH.AppImage"
fetch_tool "$APPIMAGETOOL_URL" "$CACHE_DIR/appimagetool-$ARCH.AppImage"

ICON_SRC="$ROOT/desktop/assets/icon.png"
LOGO_SRC="$ROOT/desktop/assets/logo.svg"

# --- Stage / deploy / pack (per variant) ------------------------------------

stage_appdir() {
    local app_dir="$1"
    log "Staging $app_dir"
    rm -rf "$app_dir"
    install -Dm755 "$BIN" "$app_dir/usr/bin/$BIN_NAME"
    install -Dm755 "$BACKEND_BIN" "$app_dir/usr/bin/$BACKEND_NAME"
    # bundle.rs resolves `backend` adjacent to exe — keep that name too
    cp -a "$app_dir/usr/bin/$BACKEND_NAME" "$app_dir/usr/bin/backend"

    # Fonts are read at runtime; keep them next to the binary AND in share
    if [[ -d "$GPUI_DIR/crates/webterm/assets" ]]; then
        mkdir -p "$app_dir/usr/share/$APP_ID"
        cp -a "$GPUI_DIR/crates/webterm/assets" "$app_dir/usr/share/$APP_ID/"
        mkdir -p "$app_dir/usr/bin/assets"
        cp -a "$GPUI_DIR/crates/webterm/assets/"* "$app_dir/usr/bin/assets/" 2>/dev/null || true
        chmod -R a+rX "$app_dir/usr/share/$APP_ID" "$app_dir/usr/bin/assets"
    fi

    mkdir -p "$app_dir/usr/share/applications"
    sed 's|__BINDIR__/||g' "$GPUI_DIR/dist/$APP_ID.desktop" \
        > "$app_dir/usr/share/applications/$APP_ID.desktop"
    chmod 644 "$app_dir/usr/share/applications/$APP_ID.desktop"

    local icon_base="$app_dir/usr/share/icons/hicolor"
    install -Dm644 "$ICON_SRC" "$icon_base/256x256/apps/$APP_ID.png"
    if [[ -f "$LOGO_SRC" ]]; then
        install -Dm644 "$LOGO_SRC" "$icon_base/scalable/apps/$APP_ID.svg"
    fi
    cp "$icon_base/256x256/apps/$APP_ID.png" "$app_dir/.DirIcon"

    cat > "$app_dir/AppRun" << 'EOF'
#!/bin/sh
HERE="$(dirname "$(readlink -f "$0")")"
export PATH="$HERE/usr/bin:$PATH"
exec "$HERE/usr/bin/webterm" "$@"
EOF
    chmod +x "$app_dir/AppRun"
}

deploy_libs() {
    local variant="$1" app_dir="$2"
    local excludes=()
    local pattern
    while IFS= read -r pattern; do
        if [[ -n "$pattern" ]]; then
            excludes+=(--exclude-library "$pattern")
        fi
    done < <(variant_excludes "$variant")

    log "Deploying shared libraries for the $variant variant (linuxdeploy)"
    run_appimage_tool "$CACHE_DIR/linuxdeploy-$ARCH.AppImage" \
        --appdir "$app_dir" \
        --executable "$app_dir/usr/bin/$BIN_NAME" \
        --desktop-file "$app_dir/usr/share/applications/$APP_ID.desktop" \
        --icon-file "$app_dir/usr/share/icons/hicolor/256x256/apps/$APP_ID.png" \
        ${excludes[@]+"${excludes[@]}"}

    if [[ "$variant" == "lean" ]]; then
        prune_to_direct_deps "$variant" "$app_dir"
    fi

    # State the variant contract out loud, and enforce it: this is the CI line
    # that proves the lean and GTK-bundled artifacts really differ.
    local lib_count=0
    if [[ -d "$app_dir/usr/lib" ]]; then
        lib_count="$(find "$app_dir/usr/lib" -maxdepth 1 \( -type f -o -type l \) | wc -l)"
    fi
    if [[ "$variant" == "lean" ]]; then
        if [[ -e "$app_dir/usr/lib/libgtk-3.so.0" || -e "$app_dir/usr/lib/libglib-2.0.so.0" ]]; then
            fail "lean variant must leave the GTK3 stack on the host, \
but it bundled: $(ls "$app_dir/usr/lib" | grep -E '^(libgtk-3|libglib-2.0|libgobject-2.0)' | tr '\n' ' ')"
        fi
        # Canary for closure growth: the lean bundle is supposed to hold just the
        # client's direct libraries, so anything beyond a handful means the
        # GTK3 tree leaked back in and the prune needs looking at.
        if [[ "$lib_count" -gt 8 ]]; then
            fail "lean variant should ship only the client's direct libraries, \
but shipped $lib_count: $(ls "$app_dir/usr/lib" | tr '\n' ' ')"
        fi
        log "$variant: $lib_count bundled libs, GTK3 stack resolved from the host"
    else
        if [[ ! -e "$app_dir/usr/lib/libgtk-3.so.0" ]]; then
            fail "gtk variant must bundle libgtk-3.so.0, but it is missing"
        fi
        log "$variant: $lib_count bundled libs, GTK3 stack bundled"
    fi
}

pack_variant() {
    local variant="$1" app_dir="$2"
    local out_name out
    out_name="$(variant_out_name "$variant")"
    mkdir -p "$DIST_DIR"
    out="$DIST_DIR/$out_name"

    log "Packing $out_name (appimagetool)"
    ARCH="$ARCH" VERSION="$VERSION" \
        run_appimage_tool "$CACHE_DIR/appimagetool-$ARCH.AppImage" "$app_dir" "$out"
    chmod +x "$out"

    log "Done: $out"
    du -h "$out" | awk '{print "     size: " $1}'
}

build_variant() {
    local variant="$1"
    local app_dir out
    app_dir="$(variant_app_dir "$variant")"
    log "=== $variant: $(variant_label "$variant") ==="
    stage_appdir "$app_dir"
    deploy_libs "$variant" "$app_dir"
    pack_variant "$variant" "$app_dir"
}

if [[ "$WANT_LEAN" == 1 ]]; then
    build_variant lean
fi
if [[ "$WANT_GTK" == 1 ]]; then
    build_variant gtk
fi

# --- Optional user-level install -------------------------------------------

if [[ "$INSTALL" == 1 ]]; then
    # One desktop entry can only point at one AppImage, so --install registers
    # exactly one variant: whatever was asked for, or the portable one when both
    # were built.
    if [[ "$WANT_GTK" == 1 ]]; then
        INSTALL_VARIANT=gtk
    else
        INSTALL_VARIANT=lean
    fi
    OUT_NAME="$(variant_out_name "$INSTALL_VARIANT")"
    OUT="$DIST_DIR/$OUT_NAME"
    [[ -x "$OUT" ]] || fail "nothing to install: $OUT missing"

    BIN_DIR="$HOME/Applications"
    APP_DIR_USER="$HOME/.local/share/applications"
    ICON_HOME="$HOME/.local/share/icons/hicolor"

    log "Installing the $INSTALL_VARIANT variant for this user"
    mkdir -p "$BIN_DIR" "$APP_DIR_USER"
    install -Dm755 "$OUT" "$BIN_DIR/$OUT_NAME"
    INSTALLED_APPIMAGE="$BIN_DIR/$OUT_NAME"

    sed "s|__BINDIR__/$BIN_NAME|$INSTALLED_APPIMAGE|g" \
        "$GPUI_DIR/dist/$APP_ID.desktop" \
        > "$APP_DIR_USER/$APP_ID.desktop"
    install -Dm644 "$ICON_SRC" "$ICON_HOME/256x256/apps/$APP_ID.png"
    if [[ -f "$LOGO_SRC" ]]; then
        install -Dm644 "$LOGO_SRC" "$ICON_HOME/scalable/apps/$APP_ID.svg"
    fi

    update-desktop-database "$APP_DIR_USER" 2>/dev/null || true
    gtk-update-icon-cache -f -t "$ICON_HOME" 2>/dev/null || true

    log "Installed: $INSTALLED_APPIMAGE"
fi
