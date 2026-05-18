#!/bin/bash

COMPILED_DIR="compiled"
BIN_DIR="$COMPILED_DIR/bin"
rm -rf "$COMPILED_DIR"
mkdir -p "$BIN_DIR"

echo "Building backend..."
cd be && go build -o ../"$BIN_DIR"/webterm-backend ./cmd/server/main.go
cd ..

echo "Building frontend..."
cd fe && npm run build
cd ..

echo "Copying frontend to $COMPILED_DIR/fe..."
cp -r fe/dist "$COMPILED_DIR/fe"

echo "Building Electron app..."
cd desktop && npx electron-builder --win --mac --linux
cd ..

echo "Copying desktop builds to $COMPILED_DIR/electron..."
cp -r desktop/dist/linux-unpacked "$COMPILED_DIR/electron"
cp desktop/dist/WebTerm-1.0.0.AppImage "$COMPILED_DIR/" 2>/dev/null

echo "All builds complete! Output in $COMPILED_DIR/:"
ls -la "$COMPILED_DIR/"