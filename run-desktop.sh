#!/bin/bash

# Build backend
echo "Building backend..."
cd be && go build -o webterm-backend ./cmd/server/main.go
cd ..

# Build frontend
echo "Building frontend..."
cd fe && npm run build
cd ..

# Run Electron
echo "Starting Electron..."
cd desktop && npm start
