@echo off
REM Meetily Run Script
REM This script simplifies running the application in development mode

echo.
echo ========================================
echo   Run Meetily
echo ========================================
echo.

REM Trace execution
echo [INFO] Checking for Cargo...

REM Check if Rust/Cargo is installed
where cargo >nul 2>&1
if not errorlevel 1 goto :cargo_found

REM Cargo not found handling
echo.
echo [ERROR] Rust/Cargo is NOT found in your PATH.
echo.
echo    Meetily requires Rust, Node.js, and other dependencies to run.
echo    We can attempt to install them automatically using a PowerShell script.
echo.
set /p "INSTALL_DEPS=Would you like to install dependencies now? (Y/N): "

if /i "%INSTALL_DEPS%"=="Y" goto :install_deps
goto :install_declined

:install_deps
echo.
echo    [INFO] Launching dependency installer...
echo    (This requires Administrator privileges and may prompt for confirmation)
echo.

REM Check script location
if exist "backend\install_dependancies_for_windows.ps1" (
    set "INSTALL_SCRIPT=backend\install_dependancies_for_windows.ps1"
) else (
    if exist "..\backend\install_dependancies_for_windows.ps1" (
        set "INSTALL_SCRIPT=..\backend\install_dependancies_for_windows.ps1"
    ) else (
        echo    [ERROR] Could not find installer script.
        pause
        exit /b 1
    )
)

powershell -ExecutionPolicy Bypass -File "%INSTALL_SCRIPT%"
if errorlevel 1 goto :install_failed

echo.
echo    [SUCCESS] Installation attempt finished.
echo    [IMPORTANT] You MUST restart your terminal/CMD window now to refresh environment variables.
echo    Please close this window and run 'run.bat' again.
pause
exit /b 0

:install_failed
echo.
echo    [ERROR] Installation script encountered errors.
echo    Please review the messages above.
pause
exit /b 1

:install_declined
echo.
echo    [ERROR] Cannot proceed without Rust. Exiting.
pause
exit /b 1

:cargo_found
echo [INFO] Cargo found.

REM Check directories
if exist "frontend\package.json" (
    echo [INFO] Found frontend directory, switching...
    cd frontend
) else (
    if not exist "package.json" (
        echo [ERROR] Could not find package.json.
        pause
        exit /b 1
    )
)

REM Check for pnpm/npm
where pnpm >nul 2>&1
if not errorlevel 1 goto :found_pnpm

where npm >nul 2>&1
if not errorlevel 1 goto :found_npm

echo [ERROR] Node.js (npm or pnpm) is not installed or not in PATH.
pause
exit /b 1

:found_pnpm
echo [INFO] Using pnpm...
call pnpm install
call pnpm run tauri:dev
if errorlevel 1 goto :app_error
goto :success

:found_npm
echo [INFO] Using npm...
call npm install
call npm run tauri:dev
if errorlevel 1 goto :app_error
goto :success

:app_error
echo.
echo [ERROR] Application exited with error.
pause
exit /b 1

:success
echo.
echo [INFO] Application stopped.
exit /b 0
