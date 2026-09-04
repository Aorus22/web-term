#!/usr/bin/env bash
set -euo pipefail

echo "======================================================="
echo "Building WebTerm Desktop Distribution Bundle (Linux)"
echo "======================================================="

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"
DIST_DIR="${ROOT_DIR}/dist/webterm-linux-x64"

echo "Target distribution directory: ${DIST_DIR}"
mkdir -p "${DIST_DIR}/assets"

echo "[1/3] Compiling Go backend with loopback binding..."
(
    cd "${ROOT_DIR}/be"
    go build -ldflags "-s -w" -o "${DIST_DIR}/backend" ./cmd/server
)

echo "[2/3] Compiling GPUI desktop client in release mode..."
cargo build --release --manifest-path "${ROOT_DIR}/desktop-gpui/Cargo.toml" --package webterm

echo "[3/3] Assembling distribution bundle..."
cp "${ROOT_DIR}/desktop-gpui/target/release/webterm" "${DIST_DIR}/webterm"
chmod +x "${DIST_DIR}/webterm" "${DIST_DIR}/backend"

if [ -d "${ROOT_DIR}/desktop-gpui/crates/webterm/assets" ]; then
    cp -r "${ROOT_DIR}/desktop-gpui/crates/webterm/assets/"* "${DIST_DIR}/assets/"
fi

cat << 'EOF' > "${DIST_DIR}/README.txt"
WebTerm Desktop Client (Linux)
==============================
Launch with: ./webterm

Architecture:
- Single launchable bundle with backend and webterm side-by-side.
- The backend process is automatically supervised and bound strictly to loopback (127.0.0.1).
- Native GPU terminal rendering via Alacritty.
EOF

echo ""
echo "======================================================="
echo "WebTerm Linux Bundle assembled successfully!"
echo "Location: ${DIST_DIR}"
ls -la "${DIST_DIR}"
echo "======================================================="
