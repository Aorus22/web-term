#!/usr/bin/env bash
# In-container steps for the WebTerm GPUI AppImages.
#
# Split of responsibilities:
#   docker/Dockerfile.build                  -> toolchain + system headers
#   this script                              -> the steps, run against the MOUNTED repo (/src)
#   desktop-gpui/scripts/build-appimage.sh   -> the actual packaging (reused as-is)
#
# Usage (from the repo root):
#   docker run --rm -v "$PWD:/src" -w /src webterm-build bash docker/build-appimage.sh
#
# Output (one file per variant, plus checksums):
#   desktop-gpui/dist/WebTerm-GPUI_<version>_x86_64.AppImage          lean
#   desktop-gpui/dist/WebTerm-GPUI_<version>_x86_64-gtk.AppImage      GTK3 bundled
#   desktop-gpui/dist/SHA256SUMS.txt
set -euo pipefail

ROOT_DIR="${ROOT_DIR:-/src}"
GPUI_DIR="${ROOT_DIR}/desktop-gpui"
DIST_DIR="${GPUI_DIR}/dist"

# The repo script builds the Go backend (`be/cmd/server`) and the release GPUI
# client once, then stages/deploys/packs each variant: the lean AppImage asks
# linuxdeploy to skip libgtk-3.so.0, the -gtk one lets it walk the GTK3 tree.
# Both helper tools come from the image via the LINUXDEPLOY_URL /
# APPIMAGETOOL_URL env vars baked into Dockerfile.build.
#
# Note: the frontend (`fe/`) is intentionally not built here. The Go backend
# serves it from disk at run time (be/internal/api/routes.go) and nothing
# `go:embed`s it, so the desktop client does not need a vite bundle.
echo "[1/2] Building Go backend + GPUI release client + both AppImage variants..."
"${GPUI_DIR}/scripts/build-appimage.sh"

echo "[2/2] Checksums..."
cd "${DIST_DIR}"
sha256sum WebTerm-GPUI_*.AppImage | tee SHA256SUMS.txt

variants=()
for image in WebTerm-GPUI_*.AppImage; do
    variants+=("$image")
done
if [[ "${#variants[@]}" -ne 2 ]]; then
    echo "expected 2 AppImage variants, found ${#variants[@]}: ${variants[*]}" >&2
    exit 1
fi

echo
echo "Done. Artifacts:"
ls -la "${DIST_DIR}"/WebTerm-GPUI_*.AppImage

