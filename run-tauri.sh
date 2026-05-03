#!/bin/bash

# Build backend
echo "Building backend as sidecar..."
mkdir -p desktop-tauri/src-tauri/binaries
TRIPLE=$(. "$HOME/.cargo/env" && rustc -Vv | grep host: | awk '{print $2}')
cd be && go build -o ../desktop-tauri/src-tauri/binaries/webterm-backend-$TRIPLE ./cmd/server/main.go
cd ..

# Build frontend
echo "Building frontend..."
cd fe && npm install && npm run build
cd ..

# Run Tauri
echo "Starting Tauri..."
. "$HOME/.cargo/env"

# Fix for Wayland/WebKit crashes and GBM buffer errors on Linux
# 1. Disable DMABUF renderer (Crucial for fixing GBM errors on Wayland)
export WEBKIT_DISABLE_DMABUF_RENDERER=1
# 2. Use native Wayland if possible, fallback to X11 ONLY if it crashes
# export GDK_BACKEND=x11 
# 3. Disable hardware acceleration to ensure stability
export WEBKIT_DISABLE_HW_ACCELERATION=1

# We run 'tauri dev' but it will use the static 'frontendDist' since 'devUrl' is removed
cd desktop-tauri && npx @tauri-apps/cli dev
