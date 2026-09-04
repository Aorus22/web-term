@echo off
setlocal
cd /d "%~dp0\..\.."
if not exist "desktop-gpui\test-support" mkdir "desktop-gpui\test-support"
cd be
go build -o "..\desktop-gpui\test-support\backend.exe" ./cmd/server
