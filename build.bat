@echo off
chcp 65001 >nul
echo ========================================
echo   PoiDesktop - сборка
echo ========================================
echo.

where rustc >nul 2>&1
if errorlevel 1 (
    echo [!] Rust не найден.
    echo     Скачай и установи: https://rustup.rs
    echo     Файл: rustup-init.exe
    echo     При установке выбери 1 (default) и дождись конца.
    echo.
    echo     После установки ЗАКРОЙ и снова открой это окно.
    pause
    exit /b 1
)

echo [OK] Rust найден
rustc --version
echo.

echo [*] Очистка старой сборки...
cargo clean 2>nul

echo [*] Сборка (первый раз может занять 5-15 минут)...
echo.
cargo build --release
if errorlevel 1 (
    echo.
    echo [ОШИБКА] Сборка не удалась.
    echo.
    echo Попробуй по очереди:
    echo   1. rustup update
    echo   2. Установи "Build Tools for Visual Studio":
    echo      https://visualstudio.microsoft.com/visual-cpp-build-tools/
    echo      В установщике отметь "Desktop development with C++"
    echo   3. Перезагрузи ПК и снова запусти build.bat
    echo.
    echo Пришли сюда ПОЛНЫЙ текст ошибки — помогу точечно.
    pause
    exit /b 1
)

echo.
echo ========================================
echo   ГОТОВО!
echo ========================================
echo.
echo Файл: target\release\poi-desktop.exe
echo.
if exist target\release\poi-desktop.exe (
    copy /Y target\release\poi-desktop.exe poi-desktop.exe >nul
    echo Скопирован также в: poi-desktop.exe (рядом с build.bat)
    echo.
    echo Запуск: poi-desktop.exe
)
echo.
pause
