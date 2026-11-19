@echo off
chcp 65001 >nul
cd /d "%~dp0"
echo Building...
cargo build --release
if errorlevel 1 (
  echo BUILD FAILED
  pause
  exit /b 1
)
copy /Y "target\release\poi-desktop.exe" "poi-desktop.exe" >nul
if not exist "setup" mkdir setup
if exist "setup\ComfyUI-master.zip" (
  echo setup\ComfyUI-master.zip OK
) else (
  echo WARNING: put ComfyUI-master.zip into setup\
)
echo.
echo Run: poi-desktop.exe
echo Keep folder "setup" next to exe for AI auto-install.
pause
