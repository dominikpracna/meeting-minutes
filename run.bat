@echo off
REM Meetily CLI Run Script

echo.
echo ========================================
echo   Run Meetily CLI
echo ========================================
echo.

REM Check for Rust
where cargo >nul 2>&1
if errorlevel 1 (
    echo [ERROR] Rust is not installed.
    echo Please run backend\install_dependancies_for_windows.ps1
    pause
    exit /b 1
)

echo [INFO] Building CLI...
cd cli
cargo build --release
if errorlevel 1 (
    echo [ERROR] Build failed.
    pause
    exit /b 1
)

echo [INFO] Starting Meetily CLI...
echo.
REM Default to list devices, but user can pass args
target\release\meetily-cli.exe record
pause
