@echo off
REM Meetily Run Script
REM This script simplifies running the application in development mode

echo.
echo ========================================
echo   Run Meetily
echo ========================================
echo.

REM Check if we are in the root directory and need to move to frontend
if exist "frontend\package.json" (
    echo    Found frontend directory, switching...
    cd frontend
) else if not exist "package.json" (
    echo    ❌ Error: Could not find package.json. Please run from the project root or frontend directory.
    pause
    exit /b 1
)

REM Check if pnpm is installed
where pnpm >nul 2>&1
if %errorlevel% equ 0 (
    echo    Using pnpm...
    call pnpm install
    call pnpm run tauri:dev
) else (
    REM Fallback to npm
    where npm >nul 2>&1
    if %errorlevel% equ 0 (
        echo    Using npm...
        call npm install
        call npm run tauri:dev
    ) else (
        echo    ❌ Error: Node.js (npm/pnpm) is not installed or not in PATH.
        pause
        exit /b 1
    )
)

if errorlevel 1 (
    echo.
    echo ❌ Application exited with error.
    pause
    exit /b 1
)

echo.
echo ✅ Application stopped.
exit /b 0
