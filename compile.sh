#!/bin/bash
set -e

COMPILED_DIR="compiled"
BUILD_ELECTRON=false

# Parse args
for arg in "$@"; do
  case "$arg" in
    --electron|-e) BUILD_ELECTRON=true ;;
    --quick|-q)    BUILD_ELECTRON=false ;;
    *)             echo "Unknown arg: $arg"; exit 1 ;;
  esac
done

if $BUILD_ELECTRON; then
  # Full clean + full rebuild
  rm -rf "$COMPILED_DIR"
  mkdir -p "$COMPILED_DIR"

  echo "Building backend..."
  (cd be && go build -o ../"$COMPILED_DIR"/resources/be/webterm-backend ./cmd/server/main.go)

  echo "Building frontend..."
  (cd fe && npm run build)
  mkdir -p "$COMPILED_DIR/resources/fe"
  cp -r fe/dist "$COMPILED_DIR/resources/fe/dist"

  case "$(uname -s)" in
    Linux*)   ELECTRON_TARGET="--linux" ;;
    Darwin*)  ELECTRON_TARGET="--mac" ;;
    CYGWIN*|MINGW*|MSYS*) ELECTRON_TARGET="--win" ;;
    *)        ELECTRON_TARGET="--linux --win --mac" ;;
  esac

  echo "Building Electron app for: $ELECTRON_TARGET..."
  # Build backend binary for electron-builder (expected in be/ directory)
  (cd be && go build -o webterm-backend ./cmd/server/main.go)
  touch be/webterm-backend.exe

  (cd desktop && npx electron-builder $ELECTRON_TARGET)

  # Clean up temp files
  rm -f be/webterm-backend be/webterm-backend.exe

  # Copy electron-builder output to compiled/ root (mirroring AppImage structure)
  if [ -d "desktop/dist/linux-unpacked" ]; then
    echo "Copying linux-unpacked to $COMPILED_DIR/..."
    # Copy everything EXCEPT resources/ (we'll replace with ours)
    for item in desktop/dist/linux-unpacked/*; do
      name=$(basename "$item")
      if [ "$name" != "resources" ]; then
        cp -r "$item" "$COMPILED_DIR/$name"
      fi
    done
    # Replace resources/ with ours (which has latest be + fe)
    # But we need app.asar from electron-builder
    cp -r desktop/dist/linux-unpacked/resources/app.asar "$COMPILED_DIR/resources/app.asar"
  elif [ -d "desktop/dist/win-unpacked" ]; then
    for item in desktop/dist/win-unpacked/*; do
      name=$(basename "$item")
      if [ "$name" != "resources" ]; then
        cp -r "$item" "$COMPILED_DIR/$name"
      fi
    done
    cp -r desktop/dist/win-unpacked/resources/app.asar "$COMPILED_DIR/resources/app.asar"
  elif [ -d "desktop/dist/mac" ]; then
    cp -r desktop/dist/mac/* "$COMPILED_DIR/" 2>/dev/null || true
    # On mac the app is inside the .app bundle
    if [ -d "desktop/dist/mac/WebTerm.app" ]; then
      cp -r desktop/dist/mac/WebTerm.app "$COMPILED_DIR/"
    fi
  fi

  # Copy distributable packages
  cp desktop/dist/*.AppImage "$COMPILED_DIR/" 2>/dev/null || true
  cp desktop/dist/*.exe "$COMPILED_DIR/" 2>/dev/null || true
  cp desktop/dist/*.dmg "$COMPILED_DIR/" 2>/dev/null || true

  # Extract AppImage to squashfs-root for easy running
  echo "Extracting AppImage to squashfs-root..."
  rm -rf squashfs-root
  appimage=$(ls "$COMPILED_DIR"/WebTerm-*.AppImage 2>/dev/null | head -1)
  if [ -n "$appimage" ]; then
    "$appimage" --appimage-extract > /dev/null 2>&1
    mv squashfs-root "$COMPILED_DIR/"
    echo "AppImage extracted to $COMPILED_DIR/squashfs-root"
  fi

else
  # Quick rebuild: just update backend + frontend in existing compiled/
  if [ ! -f "$COMPILED_DIR/resources/app.asar" ]; then
    echo "Error: $COMPILED_DIR not found or incomplete. Run 'compile.sh --electron' first."
    exit 1
  fi

  mkdir -p "$COMPILED_DIR/resources/be" "$COMPILED_DIR/resources/fe"

  echo "Building backend..."
  (cd be && go build -o ../"$COMPILED_DIR"/resources/be/webterm-backend ./cmd/server/main.go)

  echo "Building frontend..."
  (cd fe && npm run build)

  echo "Updating frontend..."
  rm -rf "$COMPILED_DIR/resources/fe/dist"
  cp -r fe/dist "$COMPILED_DIR/resources/fe/dist"

  # Also update inside squashfs-root if it exists
  if [ -d "$COMPILED_DIR/squashfs-root/resources/be" ]; then
    echo "Updating backend in squashfs-root..."
    cp "$COMPILED_DIR/resources/be/webterm-backend" "$COMPILED_DIR/squashfs-root/resources/be/webterm-backend"
  fi
  if [ -d "$COMPILED_DIR/squashfs-root/resources/fe" ]; then
    echo "Updating frontend in squashfs-root..."
    rm -rf "$COMPILED_DIR/squashfs-root/resources/fe/dist"
    cp -r fe/dist "$COMPILED_DIR/squashfs-root/resources/fe/dist"
  fi
fi

echo ""
echo "Build complete! Output in $COMPILED_DIR/:"
ls -la "$COMPILED_DIR/"
echo ""
echo "Structure mirrors AppImage:"
echo "  $COMPILED_DIR/webterm-desktop     ← Electron binary"
echo "  $COMPILED_DIR/resources/be/          ← Go backend"
echo "  $COMPILED_DIR/resources/fe/dist/     ← Frontend"
echo "  $COMPILED_DIR/WebTerm-*.AppImage     ← Portable build"
echo ""
echo "Usage:"
echo "  ./compile.sh                # Quick rebuild (update be + fe only)"
echo "  ./compile.sh --electron|-e   # Full rebuild (new Electron + AppImage)"
