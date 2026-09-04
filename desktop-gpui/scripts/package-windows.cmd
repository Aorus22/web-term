@echo off
setlocal enabledelayedexpansion

echo =======================================================
echo Building WebTerm Desktop Distribution Bundle (Windows)
echo =======================================================

cd /d "%~dp0\..\.."
set "DIST_DIR=%CD%\dist\webterm-windows-x64"

echo Target distribution directory: %DIST_DIR%
if not exist "%DIST_DIR%" mkdir "%DIST_DIR%"
if not exist "%DIST_DIR%\assets" mkdir "%DIST_DIR%\assets"

echo [1/3] Compiling Go backend with loopback binding...
cd be
go build -ldflags "-s -w" -o "%DIST_DIR%\backend.exe" ./cmd/server
if errorlevel 1 (
    echo [ERROR] Failed to compile Go backend.
    exit /b 1
)
cd ..

echo [2/3] Compiling GPUI desktop client in release mode...
cargo build --release --manifest-path desktop-gpui/Cargo.toml --package webterm
if errorlevel 1 (
    echo [ERROR] Failed to compile GPUI desktop client.
    exit /b 1
)

echo [3/3] Assembling distribution bundle...
copy /Y "desktop-gpui\target\release\webterm.exe" "%DIST_DIR%\webterm.exe" >nul
if exist "desktop-gpui\crates\webterm\assets" (
    xcopy /E /I /Y "desktop-gpui\crates\webterm\assets" "%DIST_DIR%\assets" >nul
)

(
echo WebTerm Desktop Client
echo ======================
echo Launch with: webterm.exe
echo.
echo Architecture:
echo - Single launchable bundle with backend.exe and webterm.exe side-by-side.
echo - The backend process is automatically supervised and bound strictly to loopback (127.0.0.1).
echo - Native GPU terminal rendering via Alacritty.
) > "%DIST_DIR%\README.txt"

echo.
echo =======================================================
echo WebTerm Windows Bundle assembled successfully!
echo Location: %DIST_DIR%
echo Files:
dir "%DIST_DIR%" /b
echo =======================================================
