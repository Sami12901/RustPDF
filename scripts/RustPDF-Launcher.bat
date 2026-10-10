@echo off
setlocal enabledelayedexpansion
title RustPDF Launcher

echo =======================================================
echo          RustPDF - Desktop PDF Workbench
echo =======================================================
echo.

set SCRIPT_DIR=%~dp0
set APP_EXE=%SCRIPT_DIR%..\target\release\rustpdf.exe

if not exist "%APP_EXE%" (
    set APP_EXE=%SCRIPT_DIR%rustpdf.exe
)

if not exist "%APP_EXE%" (
    echo [INFO] RustPDF executable not found locally.
    echo [INFO] Checking for Rust toolchain to build or run...
    where cargo >nul 2>nul
    if %ERRORLEVEL% EQU 0 (
        echo [INFO] Launching RustPDF with Cargo...
        cd /d "%SCRIPT_DIR%.."
        cargo run --release -p rustpdf
        goto :EOF
    ) else (
        echo [INFO] Cargo not found. Downloading prebuilt RustPDF for Windows...
        set DOWNLOAD_URL=https://github.com/Sami12901/RustPDF/releases/latest/download/rustpdf-windows-x86_64.zip
        powershell -NoProfile -ExecutionPolicy Bypass -Command "Invoke-WebRequest -Uri '%DOWNLOAD_URL%' -OutFile '%SCRIPT_DIR%rustpdf-windows.zip'; Expand-Archive -Path '%SCRIPT_DIR%rustpdf-windows.zip' -DestinationPath '%SCRIPT_DIR%' -Force; Remove-Item '%SCRIPT_DIR%rustpdf-windows.zip'"
        if exist "%SCRIPT_DIR%rustpdf.exe" (
            set APP_EXE=%SCRIPT_DIR%rustpdf.exe
        )
    )
)

if exist "%APP_EXE%" (
    echo [OK] Launching RustPDF...
    start "" "%APP_EXE%" %*
) else (
    echo [ERROR] Could not start RustPDF.
    echo Please visit https://github.com/Sami12901/RustPDF/releases to download manually.
    pause
)
