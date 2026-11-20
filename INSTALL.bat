@echo off
chcp 65001 >nul
echo.
echo === PoiDesktop FORCE INSTALL ===
echo.
set "DEST=C:\Games\безопасные_программы\poi-desktop"
echo Destination: %DEST%
echo.

if not exist "%DEST%" mkdir "%DEST%"
if not exist "%DEST%\src" mkdir "%DEST%\src"

copy /Y "%~dp0Cargo.toml" "%DEST%\Cargo.toml"
copy /Y "%~dp0src\main.rs" "%DEST%\src\main.rs"
copy /Y "%~dp0src\shapes.rs" "%DEST%\src\shapes.rs"
copy /Y "%~dp0src\wallpaper.rs" "%DEST%\src\wallpaper.rs"
del /F /Q "%DEST%\src\comfy.rs" 2>nul
del /F /Q "%DEST%\src\config.rs" 2>nul

echo.
echo --- src folder now ---
dir "%DEST%\src"
echo.
findstr /C:"hover_preset" "%DEST%\src\main.rs" >nul
if %errorlevel%==0 (
  echo ERROR: old main.rs still has hover_preset
) else (
  echo OK: new main.rs
)
echo.
cd /d "%DEST%"
echo Running cargo clean + build...
cargo clean
cargo build --release
if exist "target\release\poi-desktop.exe" (
  echo.
  echo Starting...
  start "" "target\release\poi-desktop.exe"
) else (
  echo BUILD FAILED - see errors above
)
pause
