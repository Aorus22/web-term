#!/bin/bash

# Build backend
echo "Building backend..."
cd be && go build -o webterm-backend ./cmd/server/main.go
cd ..

# Build clipboard helpers for SSH targets
echo "Building clipboard helpers..."
cd be
GOOS=linux GOARCH=amd64 go build -o web-term-clip ./cmd/clip-helper
GOOS=windows GOARCH=amd64 go build -o web-term-clip.exe ./cmd/clip-helper
echo "Created: web-term-clip (Linux), web-term-clip.exe (Windows)"

# Build local clipboard helper (for backend local clipboard monitoring)
GOOS=linux GOARCH=amd64 go build -o clip-local ./cmd/clip-local
GOOS=darwin GOARCH=amd64 go build -o clip-local ./cmd/clip-local
echo "Created: clip-local"
cd ..

# Build frontend
echo "Building frontend..."
cd fe && npm run build
cd ..

# Run Electron
echo "Starting Electron..."
cd desktop && npm start
