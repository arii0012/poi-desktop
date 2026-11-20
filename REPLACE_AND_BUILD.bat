@echo off
chcp 65001 >nul
set "DEST=C:\Games\безопасные_программы\poi-desktop"
echo DEST=%DEST%
mkdir "%DEST%\src" 2>nul
copy /Y "%~dp0Cargo.toml" "%DEST%\Cargo.toml"
copy /Y "%~dp0src\main.rs" "%DEST%\src\main.rs"
del /F /Q "%DEST%\src\shapes.rs" 2>nul
del /F /Q "%DEST%\src\wallpaper.rs" 2>nul
del /F /Q "%DEST%\src\comfy.rs" 2>nul
del /F /Q "%DEST%\src\config.rs" 2>nul
echo src:
dir "%DEST%\src"
cd /d "%DEST%"
cargo clean
cargo build --release
if exist target\release\poi-desktop.exe (
  target\release\poi-desktop.exe
) else (
  echo BUILD FAILED
)
pause
