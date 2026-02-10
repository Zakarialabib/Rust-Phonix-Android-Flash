<#
.SYNOPSIS
    Phoenix Tools Bootstrap Script for Windows
.DESCRIPTION
    Sets up the development environment for Phoenix CLI and GUI tools.
#>

param(
    [switch]$Dev,
    [switch]$Release,
    [switch]$Gui,
    [switch]$Help
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir

Write-Host ""
Write-Host "  Phoenix Tools Bootstrap" -ForegroundColor Cyan
Write-Host ""

if ($Help) {
    Write-Host "Usage: bootstrap.ps1 [-Dev] [-Release] [-Gui] [-Help]"
    exit 0
}

# 1. Check/Install Rust
Write-Host "[1/4] Checking for Rust..." -ForegroundColor Yellow
if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) {
    Write-Host "      Rust not found. Installing..." -ForegroundColor Yellow
    Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "$env:TEMP\rustup-init.exe"
    Start-Process -FilePath "$env:TEMP\rustup-init.exe" -ArgumentList "-y" -Wait -NoNewWindow
    $env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
}
Write-Host "      Found: $(rustc --version)" -ForegroundColor Green

# 2. Add Target
Write-Host "[2/4] Adding ARM64 target..." -ForegroundColor Yellow
& rustup target add aarch64-unknown-linux-gnu 2>$null

# 3. Build CLI
Write-Host "[3/4] Building phoenix-cli..." -ForegroundColor Yellow
Set-Location $ProjectRoot
$buildFlag = if ($Release) { "--release" } else { "" }
& cargo build $buildFlag -p phoenix-cli
if ($LASTEXITCODE -ne 0) { throw "CLI build failed" }

# 4. Build/Run GUI
if ($Gui) {
    Write-Host "[4/4] Setting up GUI..." -ForegroundColor Yellow
    Set-Location "$ProjectRoot\ui"
    & npm install
    
    Write-Host "      Note: GUI is now a standalone Tauri project for better stability." -ForegroundColor Cyan
    Write-Host "      To start: cd ui; npm run tauri dev" -ForegroundColor Cyan
}

# Install to bin
$binPath = "$env:USERPROFILE\.phoenix\bin"
if (-not (Test-Path $binPath)) { New-Item -ItemType Directory -Path $binPath -Force }
$target = if ($Release) { "release" } else { "debug" }
Copy-Item "$ProjectRoot\target\$target\phoenix.exe" "$binPath\phoenix.exe" -Force

Write-Host ""
Write-Host "✅ Setup Complete!" -ForegroundColor Green
