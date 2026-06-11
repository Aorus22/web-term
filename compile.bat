@echo off
setlocal enabledelayedexpansion

set COMPILED_DIR=compiled
set BUILD_ELECTRON=false

:: Parse args
if /I "%1"=="--electron" set BUILD_ELECTRON=true
if /I "%1"=="-e" set BUILD_ELECTRON=true
if /I "%1"=="--quick" set BUILD_ELECTRON=false
if /I "%1"=="-q" set BUILD_ELECTRON=false

if "%BUILD_ELECTRON%"=="true" (
  :: Full clean + full rebuild
  if exist "%COMPILED_DIR%" rmdir /s /q "%COMPILED_DIR%"
  mkdir "%COMPILED_DIR%"

  echo Building backend...
  cd be
  go build -o "..\%COMPILED_DIR%\resources\be\webterm-backend.exe" ./cmd/server/main.go
  if errorlevel 1 exit /b %errorlevel%
  cd ..

  echo Building frontend...
  cd fe
  call npm run build
  if errorlevel 1 exit /b %errorlevel%
  cd ..
  if not exist "%COMPILED_DIR%\resources\fe" mkdir "%COMPILED_DIR%\resources\fe"
  if exist "%COMPILED_DIR%\resources\fe\dist" rmdir /s /q "%COMPILED_DIR%\resources\fe\dist"
  xcopy /e /i fe\dist "%COMPILED_DIR%\resources\fe\dist" > nul

  echo Building Electron app for: --win...
  :: Build backend binary for electron-builder (expected in be/ directory)
  cd be
  go build -o webterm-backend.exe ./cmd/server/main.go
  if errorlevel 1 exit /b %errorlevel%
  cd ..

  cd desktop
  call npx electron-builder --win
  if errorlevel 1 exit /b %errorlevel%
  cd ..

  :: Copy electron-builder output to compiled/ root (mirroring structure)
  if exist "desktop\dist\win-unpacked" (
    echo Copying win-unpacked to %COMPILED_DIR%/...
    :: Copy everything EXCEPT resources/ (we'll replace with ours)
    for /D %%d in (desktop\dist\win-unpacked\*) do (
      set "name=%%~nxd"
      if not "!name!"=="resources" (
        if exist "%COMPILED_DIR%\!name!" rmdir /s /q "%COMPILED_DIR%\!name!" 2>nul
        xcopy /e /i "%%d" "%COMPILED_DIR%\!name!\" > nul
      )
    )
    for %%f in (desktop\dist\win-unpacked\*) do (
      if not "%%~nxf"=="resources" (
        copy "%%f" "%COMPILED_DIR%\" > nul 2>nul
      )
    )
    :: Replace resources/ with ours (which has latest be + fe)
    :: But we need app.asar from electron-builder
    if exist "desktop\dist\win-unpacked\resources\app.asar" (
      copy "desktop\dist\win-unpacked\resources\app.asar" "%COMPILED_DIR%\resources\app.asar" > nul
    )
  )

  :: Clean up temp files for electron-builder
  if exist "be\webterm-backend" del "be\webterm-backend" 2>nul
  if exist "be\webterm-backend.exe" del "be\webterm-backend.exe" 2>nul

  :: Copy distributable packages
  if exist "desktop\dist\*.exe" (
    copy "desktop\dist\*.exe" "%COMPILED_DIR%\" > nul 2>nul
  )
  if exist "desktop\dist\*.dmg" (
    copy "desktop\dist\*.dmg" "%COMPILED_DIR%\" > nul 2>nul
  )

) else (
  :: Quick rebuild: just update backend + frontend in existing compiled/
  if not exist "%COMPILED_DIR%\resources\app.asar" (
    echo Error: %COMPILED_DIR% not found or incomplete. Run 'compile.bat --electron' first.
    exit /b 1
  )

  if not exist "%COMPILED_DIR%\resources\be" mkdir "%COMPILED_DIR%\resources\be"
  if not exist "%COMPILED_DIR%\resources\fe" mkdir "%COMPILED_DIR%\resources\fe"

  echo Building backend...
  cd be
  go build -o "..\%COMPILED_DIR%\resources\be\webterm-backend.exe" ./cmd/server/main.go
  if errorlevel 1 exit /b %errorlevel%
  cd ..

  echo Building frontend...
  cd fe
  call npm run build
  if errorlevel 1 exit /b %errorlevel%
  cd ..

  echo Updating frontend...
  if exist "%COMPILED_DIR%\resources\fe\dist" rmdir /s /q "%COMPILED_DIR%\resources\fe\dist"
  xcopy /e /i fe\dist "%COMPILED_DIR%\resources\fe\dist" > nul
)

echo.
echo Build complete! Output in %COMPILED_DIR%/:
dir "%COMPILED_DIR%" /w
echo.
echo Structure mirrors AppImage:
echo   %COMPILED_DIR%/WebTerm Setup *.exe   ^<-- Electron installer
echo   %COMPILED_DIR%/resources/be/             ^<-- Go backend
echo   %COMPILED_DIR%/resources/fe/dist/        ^<-- Frontend
echo.
echo Usage:
echo   compile.bat                 Quick rebuild (update be + fe only^)
echo   compile.bat --electron      Full rebuild (new Electron installer^)
