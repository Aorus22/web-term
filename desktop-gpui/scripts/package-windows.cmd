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

rem Ensure shader compiler is available for GPUI release build
set "FXC_TOOL=%CD%\desktop-gpui\tools\fxc\fxc.exe"
if not defined GPUI_FXC_PATH (
    if not exist "!FXC_TOOL!" (
        echo Compiling standalone FXC shader compiler helper...
        rustc -O desktop-gpui\tools\fxc\main.rs -o "!FXC_TOOL!"
    )
    set "GPUI_FXC_PATH=!FXC_TOOL!"
)

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

echo WebTerm Desktop Client> "%DIST_DIR%\README.txt"
echo ======================>> "%DIST_DIR%\README.txt"
echo Launch with: webterm.exe>> "%DIST_DIR%\README.txt"
echo.>> "%DIST_DIR%\README.txt"
echo Architecture:>> "%DIST_DIR%\README.txt"
echo - Single launchable bundle with backend.exe and webterm.exe side-by-side.>> "%DIST_DIR%\README.txt"
echo - The backend process is automatically supervised and bound strictly to loopback (127.0.0.1).>> "%DIST_DIR%\README.txt"
echo - Native GPU terminal rendering via Alacritty.>> "%DIST_DIR%\README.txt"

echo(
echo =======================================================
echo WebTerm Windows Bundle assembled successfully!
echo Location: %DIST_DIR%
echo Files:
dir "%DIST_DIR%" /b
echo =======================================================
