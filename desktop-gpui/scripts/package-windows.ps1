param(
    [switch]$Release,
    [switch]$SkipBackend,
    [switch]$BuildBackend
)

$Profile = if ($Release) { "release" } else { "debug" }
$CargoArgs = if ($Release) { @("build", "--release") } else { @("build") }

# WebTerm Desktop Packaging Script for Windows
$ErrorActionPreference = "Stop"

Write-Host "=======================================================" -ForegroundColor Cyan
Write-Host "Building WebTerm Desktop Distribution Bundle ($Profile)" -ForegroundColor Cyan
Write-Host "=======================================================" -ForegroundColor Cyan

$Root = Resolve-Path "$PSScriptRoot\..\.."
$DistDir = "$Root\dist\webterm-windows-x64"

Write-Host "Target distribution directory: $DistDir"
New-Item -ItemType Directory -Force -Path $DistDir | Out-Null
New-Item -ItemType Directory -Force -Path "$DistDir\assets" | Out-Null

$BackendExists = Test-Path "$DistDir\backend.exe"
if ($SkipBackend -or (-not $BuildBackend -and $BackendExists)) {
    Write-Host "[1/3] Backend binary already exists at $DistDir\backend.exe (skipping backend build)..." -ForegroundColor Cyan
} else {
    Write-Host "[1/3] Compiling Go backend with loopback binding..." -ForegroundColor Yellow
    Push-Location "$Root\be"
    try {
        go build -ldflags "-s -w" -o "$DistDir\backend.exe" ./cmd/server
    } finally {
        Pop-Location
    }
}

# Ensure shader compiler is available for GPUI build
$FxcTool = "$Root\desktop-gpui\tools\fxc\fxc.exe"
if ([string]::IsNullOrEmpty($env:GPUI_FXC_PATH) -or -not (Test-Path $env:GPUI_FXC_PATH -ErrorAction SilentlyContinue)) {
    if (-not (Test-Path $FxcTool)) {
        Write-Host "Compiling standalone FXC shader compiler helper..." -ForegroundColor Yellow
        rustc -O "$Root\desktop-gpui\tools\fxc\main.rs" -o $FxcTool
    }
    $env:GPUI_FXC_PATH = $FxcTool
}

Write-Host "[2/3] Compiling GPUI desktop client in $Profile mode..." -ForegroundColor Yellow
if ($Release) {
    cargo build --release --manifest-path "$Root\desktop-gpui\Cargo.toml" --package webterm
} else {
    cargo build --manifest-path "$Root\desktop-gpui\Cargo.toml" --package webterm
}

Write-Host "[3/3] Assembling distribution bundle..." -ForegroundColor Yellow
Stop-Process -Name webterm, backend -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 200
Copy-Item "$Root\desktop-gpui\target\$Profile\webterm.exe" "$DistDir\webterm.exe" -Force

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
