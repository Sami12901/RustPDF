# RustPDF Windows One-Line Installer
# Usage: irm https://raw.githubusercontent.com/Sami12901/RustPDF/main/scripts/install.ps1 | iex

$ErrorActionPreference = "Stop"

$repo = "Sami12901/RustPDF"
$installDir = "$env:LOCALAPPDATA\Programs\RustPDF"
$zipUrl = "https://github.com/$repo/releases/latest/download/rustpdf-windows-x86_64.zip"
$zipFile = "$env:TEMP\rustpdf-windows.zip"

Write-Host "=======================================================" -ForegroundColor Cyan
Write-Host "          Installing RustPDF for Windows               " -ForegroundColor White
Write-Host "=======================================================" -ForegroundColor Cyan

if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null
}

Write-Host "-> Downloading latest release from GitHub..." -ForegroundColor Yellow
try {
    Invoke-WebRequest -Uri $zipUrl -OutFile $zipFile -UseBasicParsing
    Write-Host "-> Extracting binaries..." -ForegroundColor Yellow
    Expand-Archive -Path $zipFile -DestinationPath $installDir -Force
    Remove-Item $zipFile -Force
} catch {
    Write-Warning "Could not download GitHub release binary (may not be released yet)."
    Write-Host "-> Checking if local workspace binary exists..." -ForegroundColor Yellow
    $localBin = Join-Path $PSScriptRoot "..\target\release\rustpdf.exe"
    if (Test-Path $localBin) {
        Copy-Item $localBin (Join-Path $installDir "rustpdf.exe") -Force
        $localCli = Join-Path $PSScriptRoot "..\target\release\rustpdf-cli.exe"
        if (Test-Path $localCli) {
            Copy-Item $localCli (Join-Path $installDir "rustpdf-cli.exe") -Force
        }
        Write-Host "-> Installed from local build!" -ForegroundColor Green
    } else {
        throw "Download failed and local release binary not found."
    }
}

# Create desktop shortcut
try {
    $wshell = New-Object -ComObject WScript.Shell
    $shortcutPath = [System.IO.Path]::Combine([Environment]::GetFolderPath("Desktop"), "RustPDF.lnk")
    $shortcut = $wshell.CreateShortcut($shortcutPath)
    $shortcut.TargetPath = "$installDir\rustpdf.exe"
    $shortcut.WorkingDirectory = $installDir
    $shortcut.Description = "RustPDF - Desktop PDF Workbench"
    $shortcut.Save()
    Write-Host "-> Created Desktop shortcut: $shortcutPath" -ForegroundColor Green
} catch {
    Write-Host "-> Note: Desktop shortcut creation skipped."
}

# Add to user PATH if not present
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$installDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$userPath;$installDir", "User")
    Write-Host "-> Added RustPDF to User PATH ($installDir)" -ForegroundColor Green
}

Write-Host "`n[SUCCESS] RustPDF has been installed successfully!" -ForegroundColor Green
Write-Host "Run 'rustpdf' from terminal or open the Desktop shortcut." -ForegroundColor Cyan
