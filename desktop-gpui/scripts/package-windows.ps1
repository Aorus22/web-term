# WebTerm Desktop Packaging Script for Windows
$ErrorActionPreference = "Stop"

Write-Host "=======================================================" -ForegroundColor Cyan
Write-Host "Building WebTerm Desktop Distribution Bundle (Windows)" -ForegroundColor Cyan
Write-Host "=======================================================" -ForegroundColor Cyan

$Root = Resolve-Path "$PSScriptRoot\..\.."
$DistDir = "$Root\dist\webterm-windows-x64"

Write-Host "Target distribution directory: $DistDir"
New-Item -ItemType Directory -Force -Path $DistDir | Out-Null
New-Item -ItemType Directory -Force -Path "$DistDir\assets" | Out-Null

Write-Host "[1/3] Compiling Go backend with loopback binding..." -ForegroundColor Yellow
Push-Location "$Root\be"
try {
    go build -ldflags "-s -w" -o "$DistDir\backend.exe" ./cmd/server
} finally {
    Pop-Location
}

Write-Host "[2/3] Compiling GPUI desktop client in release mode..." -ForegroundColor Yellow
cargo build --release --manifest-path "$Root\desktop-gpui\Cargo.toml" --package webterm

Write-Host "[3/3] Assembling distribution bundle..." -ForegroundColor Yellow
Copy-Item "$Root\desktop-gpui\target\release\webterm.exe" "$DistDir\webterm.exe" -Force

if (Test-Path "$Root\desktop-gpui\crates\webterm\assets") {
    Copy-Item "$Root\desktop-gpui\crates\webterm\assets\*" "$DistDir\assets" -Recurse -Force
}

$Readme = @"
WebTerm Desktop Client
======================
Launch with: webterm.exe

Architecture:
- Single launchable bundle with backend.exe and webterm.exe side-by-side.
- The backend process is automatically supervised and bound strictly to loopback (127.0.0.1).
- Native GPU terminal rendering via Alacritty.
"@
Set-Content -Path "$DistDir\README.txt" -Value $Readme

Write-Host ""
Write-Host "=======================================================" -ForegroundColor Green
Write-Host "WebTerm Windows Bundle assembled successfully!" -ForegroundColor Green
Write-Host "Location: $DistDir" -ForegroundColor Green
Get-ChildItem $DistDir | Select-Object Name, Length, LastWriteTime | Format-Table -AutoSize
Write-Host "=======================================================" -ForegroundColor Green
