@echo off
REM Meetily Run Script
REM This script simplifies running the application in development mode

echo.
echo ========================================
echo   Run Meetily
echo ========================================
echo.

REM Check if Rust/Cargo is installed
where cargo >nul 2>&1
if errorlevel 1 (
    echo.
    echo ❌ Rust/Cargo is NOT found in your PATH.
    echo.
    echo    Meetily requires Rust, Node.js, and other dependencies to run.
    echo    We can attempt to install them automatically using a PowerShell script.
    echo.
    set /p "INSTALL_DEPS=Would you like to install dependencies now? (Y/N): "

    if /i "%INSTALL_DEPS%"=="Y" (
        echo.
        echo    🚀 Launching dependency installer...
        echo    (This requires Administrator privileges and may prompt for confirmation)
        echo.

        REM Run installer and capture exit code
        set INSTALL_EXIT_CODE=0
        if exist "backend\install_dependancies_for_windows.ps1" (
            powershell -ExecutionPolicy Bypass -File backend\install_dependancies_for_windows.ps1
            set INSTALL_EXIT_CODE=%errorlevel%
        ) else if exist "..\backend\install_dependancies_for_windows.ps1" (
            powershell -ExecutionPolicy Bypass -File ..\backend\install_dependancies_for_windows.ps1
            set INSTALL_EXIT_CODE=%errorlevel%
        ) else (
            echo    ❌ Could not find installer script at backend\install_dependancies_for_windows.ps1
            pause
            exit /b 1
        )

        if errorlevel 1 (
            echo.
            echo    ⚠️  Installation script encountered errors.
            echo    Please review the error messages above.
            echo    You may need to run the installer manually or fix issues (like file locks) and reboot.
            echo.
            echo    Command: powershell -ExecutionPolicy Bypass -File backend\install_dependancies_for_windows.ps1
            pause
            exit /b 1
        )

        echo.
        echo    ✅ Installation attempt finished.
        echo    ⚠️  IMPORTANT: You MUST restart your terminal/CMD window now to refresh environment variables.
        echo    Please close this window and run 'run.bat' again.
        pause
        exit /b 0
    ) else (
        echo.
        echo    ❌ Cannot proceed without Rust. Exiting.
        pause
        exit /b 1
    )
)

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
if not errorlevel 1 goto :found_pnpm

REM Fallback to npm
where npm >nul 2>&1
if not errorlevel 1 goto :found_npm

goto :not_found

:found_pnpm
echo    Using pnpm...
call pnpm install
call pnpm run tauri:dev
goto :check_error

:found_npm
echo    Using npm...
call npm install
call npm run tauri:dev
goto :check_error

:not_found
echo    ❌ Error: Node.js (npm or pnpm) is not installed or not in PATH.
pause
exit /b 1

:check_error
if errorlevel 1 (
    echo.
    echo ❌ Application exited with error.
    pause
    exit /b 1
)

echo.
echo ✅ Application stopped.
exit /b 0
