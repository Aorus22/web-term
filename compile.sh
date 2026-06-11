#!/bin/bash
set -e

COMPILED_DIR="compiled"
BIN_DIR="$COMPILED_DIR/bin"
BUILD_ELECTRON=false

# Parse args
for arg in "$@"; do
  case "$arg" in
    --electron|-e) BUILD_ELECTRON=true ;;
    --quick|-q)    BUILD_ELECTRON=false ;;
    *)             echo "Unknown arg: $arg"; exit 1 ;;
  esac
done

# Full clean only if building electron (first time)
if $BUILD_ELECTRON; then
  rm -rf "$COMPILED_DIR"
  mkdir -p "$BIN_DIR" "$COMPILED_DIR/fe"
else
  mkdir -p "$BIN_DIR" "$COMPILED_DIR/fe"
fi

echo "Building backend..."
(cd be && go build -o ../"$BIN_DIR"/webterm-backend ./cmd/server/main.go)

echo "Building frontend..."
(cd fe && npm run build)

echo "Copying frontend to $COMPILED_DIR/fe/dist..."
rm -rf "$COMPILED_DIR/fe/dist"
cp -r fe/dist "$COMPILED_DIR/fe/dist"

# Also update frontend + backend inside existing electron resources (fast, no repack)
if [ -d "$COMPILED_DIR/electron/resources/fe" ]; then
  echo "Updating frontend in electron resources..."
  rm -rf "$COMPILED_DIR/electron/resources/fe/dist"
  cp -r fe/dist "$COMPILED_DIR/electron/resources/fe/dist"
fi
if [ -d "$COMPILED_DIR/electron/resources/be" ]; then
  echo "Updating backend in electron resources..."
  cp "$BIN_DIR/webterm-backend" "$COMPILED_DIR/electron/resources/be/webterm-backend"
fi

if $BUILD_ELECTRON; then
  case "$(uname -s)" in
    Linux*)   ELECTRON_TARGET="--linux" ;;
    Darwin*)  ELECTRON_TARGET="--mac" ;;
    CYGWIN*|MINGW*|MSYS*) ELECTRON_TARGET="--win" ;;
    *)        ELECTRON_TARGET="--linux --win --mac" ;;
  esac

  echo "Building Electron app for: $ELECTRON_TARGET..."
  (cd desktop && npx electron-builder $ELECTRON_TARGET)

  echo "Copying desktop builds to $COMPILED_DIR/electron..."
  if [ -d "desktop/dist/linux-unpacked" ]; then
    cp -r desktop/dist/linux-unpacked "$COMPILED_DIR/electron"
  elif [ -d "desktop/dist/win-unpacked" ]; then
    cp -r desktop/dist/win-unpacked "$COMPILED_DIR/electron"
  elif [ -d "desktop/dist/mac" ]; then
    cp -r desktop/dist/mac "$COMPILED_DIR/electron"
  fi
  cp desktop/dist/*.AppImage "$COMPILED_DIR/" 2>/dev/null
  cp desktop/dist/*.exe "$COMPILED_DIR/" 2>/dev/null
  cp desktop/dist/*.dmg "$COMPILED_DIR/" 2>/dev/null
  rm -rf squashfs-root 2>/dev/null

  # Update frontend in the freshly copied electron resources too
  if [ -d "$COMPILED_DIR/electron/resources/fe" ]; then
    echo "Updating frontend in fresh electron resources..."
    rm -rf "$COMPILED_DIR/electron/resources/fe/dist"
    cp -r fe/dist "$COMPILED_DIR/electron/resources/fe/dist"
  fi

  # Extract AppImage to squashfs-root for easy running
  echo "Extracting AppImage to squashfs-root..."
  rm -rf squashfs-root
  "$COMPILED_DIR"/WebTerm-*.AppImage --appimage-extract > /dev/null 2>&1
  mv squashfs-root "$COMPILED_DIR/"
  echo "AppImage extracted to $COMPILED_DIR/squashfs-root"
fi

echo ""
echo "Build complete! Output in $COMPILED_DIR/:"
ls -la "$COMPILED_DIR/"
echo ""
echo "Tip: use './compile.sh' for fast frontend-only rebuild"
echo "     use './compile.sh -e' for full electron repack"
