#!/usr/bin/env bash
# Build a portable AppImage for the WebTerm GPUI frontend.
#
#   ./build-appimage.sh              # build dist/*.AppImage
#   ./build-appimage.sh --install    # build, then register for this user
#   ./build-appimage.sh --clean      # drop the staged AppDir
#
# Layout of the produced AppDir:
#   usr/bin/webterm (+ webterm-backend side-by-side)
#   usr/share/webterm-gpui/assets/{fonts/}
#   usr/share/applications/webterm-gpui.desktop
#   usr/share/icons/hicolor/{128x128,256x256,scalable}/apps/webterm-gpui.*
#
# Bundling policy: linuxdeploy collects the binary's shared-library
# dependencies. GTK3, libvulkan and GPU drivers stay on the host on purpose.
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
APP_DIR="$GPUI_DIR/target/appimage/$APP_ID.AppDir"
DIST_DIR="$GPUI_DIR/dist"
CACHE_DIR="${XDG_CACHE_HOME:-$HOME/.cache}/webterm-appimage"

LINUXDEPLOY_URL="${LINUXDEPLOY_URL:-https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage}"
APPIMAGETOOL_URL="${APPIMAGETOOL_URL:-https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage}"

INSTALL=0
for arg in "$@"; do
    case "$arg" in
        --install) INSTALL=1 ;;
        --clean)
            echo "Removing $APP_DIR"
            rm -rf "$APP_DIR"
            exit 0
            ;;
        -h|--help)
            sed -n '2,20p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *)
            echo "Unknown option: $arg" >&2
            exit 2
            ;;
    esac
done

log() { printf '\033[0;36m[appimage]\033[0m %s\n' "$1"; }
fail() { printf '\033[0;31m[appimage] %s\033[0m\n' "$1" >&2; exit 1; }

command -v cargo >/dev/null || fail "cargo not found (install Rust: https://rustup.rs)"
command -v go >/dev/null || fail "go not found"
command -v mksquashfs >/dev/null || fail "mksquashfs not found (install squashfs-tools)"

VERSION="$(grep -m1 '^version' "$GPUI_DIR/Cargo.toml" | sed 's/.*"\(.*\)".*/\1/')"
ARCH="$(uname -m)"
[[ "$ARCH" == "x86_64" ]] || fail "only x86_64 is supported by the bundled tools (got $ARCH)"

OUT_NAME="WebTerm-GPUI_${VERSION}_${ARCH}.AppImage"

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

# --- Build ------------------------------------------------------------------

log "Building Go backend (release)"
(cd "$ROOT/be" && go build -ldflags "-s -w" -o "$GPUI_DIR/target/release/$BACKEND_NAME" ./cmd/server)

# The glass backdrop needs a locally patched gpui (see scripts/patch-gpui.sh).
log "Applying the gpui backdrop-blur patch"
"$SCRIPT_DIR/patch-gpui.sh"

log "Building release binary (cargo build --release -p webterm)"
# Cargo only sees the patched crate when it runs from the repository root,
# where patch-gpui.sh writes the generated .cargo/config.toml.
(cd "$ROOT" && cargo build --release --manifest-path "$GPUI_DIR/Cargo.toml" --package webterm)
BIN="$GPUI_DIR/target/release/$BIN_NAME"
[[ -x "$BIN" ]] || fail "release binary missing: $BIN"
BACKEND_BIN="$GPUI_DIR/target/release/$BACKEND_NAME"

log "Staging AppDir"
rm -rf "$APP_DIR"
install -Dm755 "$BIN" "$APP_DIR/usr/bin/$BIN_NAME"
install -Dm755 "$BACKEND_BIN" "$APP_DIR/usr/bin/$BACKEND_NAME"
# bundle.rs resolves `backend` adjacent to exe — keep that name too
cp -a "$APP_DIR/usr/bin/$BACKEND_NAME" "$APP_DIR/usr/bin/backend"

# Fonts are read at runtime; keep them next to the binary AND in share
if [[ -d "$GPUI_DIR/crates/webterm/assets" ]]; then
    mkdir -p "$APP_DIR/usr/share/$APP_ID"
    cp -a "$GPUI_DIR/crates/webterm/assets" "$APP_DIR/usr/share/$APP_ID/"
    mkdir -p "$APP_DIR/usr/bin/assets"
    cp -a "$GPUI_DIR/crates/webterm/assets/"* "$APP_DIR/usr/bin/assets/" 2>/dev/null || true
    chmod -R a+rX "$APP_DIR/usr/share/$APP_ID" "$APP_DIR/usr/bin/assets"
fi

mkdir -p "$APP_DIR/usr/share/applications"
sed 's|__BINDIR__/||g' "$GPUI_DIR/dist/$APP_ID.desktop" \
    > "$APP_DIR/usr/share/applications/$APP_ID.desktop"
chmod 644 "$APP_DIR/usr/share/applications/$APP_ID.desktop"

ICON_BASE="$APP_DIR/usr/share/icons/hicolor"
ICON_SRC="$ROOT/desktop/assets/icon.png"
LOGO_SRC="$ROOT/desktop/assets/logo.svg"
install -Dm644 "$ICON_SRC" "$ICON_BASE/256x256/apps/$APP_ID.png"
if [[ -f "$LOGO_SRC" ]]; then
    install -Dm644 "$LOGO_SRC" "$ICON_BASE/scalable/apps/$APP_ID.svg"
fi
cp "$ICON_BASE/256x256/apps/$APP_ID.png" "$APP_DIR/.DirIcon"

cat > "$APP_DIR/AppRun" << 'EOF'
#!/bin/sh
HERE="$(dirname "$(readlink -f "$0")")"
export PATH="$HERE/usr/bin:$PATH"
exec "$HERE/usr/bin/webterm" "$@"
EOF
chmod +x "$APP_DIR/AppRun"

fetch_tool "$LINUXDEPLOY_URL" "$CACHE_DIR/linuxdeploy-$ARCH.AppImage"
fetch_tool "$APPIMAGETOOL_URL" "$CACHE_DIR/appimagetool-$ARCH.AppImage"

log "Deploying shared-library dependencies (linuxdeploy)"
run_appimage_tool "$CACHE_DIR/linuxdeploy-$ARCH.AppImage" \
    --appdir "$APP_DIR" \
    --executable "$APP_DIR/usr/bin/$BIN_NAME" \
    --desktop-file "$APP_DIR/usr/share/applications/$APP_ID.desktop" \
    --icon-file "$ICON_BASE/256x256/apps/$APP_ID.png"

mkdir -p "$DIST_DIR"
OUT="$DIST_DIR/$OUT_NAME"

log "Packing $OUT_NAME (appimagetool)"
ARCH="$ARCH" VERSION="$VERSION" \
    run_appimage_tool "$CACHE_DIR/appimagetool-$ARCH.AppImage" "$APP_DIR" "$OUT"
chmod +x "$OUT"

log "Done: $OUT"
du -h "$OUT" | awk '{print "     size: " $1}'

# --- Optional user-level install -------------------------------------------

if [[ "$INSTALL" == "1" ]]; then
    BIN_DIR="$HOME/Applications"
    APP_DIR_USER="$HOME/.local/share/applications"
    ICON_HOME="$HOME/.local/share/icons/hicolor"

    log "Installing AppImage for this user"
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
