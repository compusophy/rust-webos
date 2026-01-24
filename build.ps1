$ErrorActionPreference = "Stop"

Write-Host "Building WASM Apps..." -ForegroundColor Cyan

# List of apps to build
$apps = @("terminal", "math", "desktop", "hello")

foreach ($app in $apps) {
    Write-Host "  Building $app..." -NoNewline
    Push-Location "apps/$app"
    cargo build --target wasm32-unknown-unknown --release
    Pop-Location
    Write-Host " Done." -ForegroundColor Green
}

Write-Host "Rebuilding Kernel..." -ForegroundColor Cyan
# Clear dist to avoid integrity issues
if (Test-Path "dist") {
    Remove-Item -Recurse -Force dist
}

# FORCE REBUILD of include_bytes! dependencies
# Rust's incremental compilation often misses external file changes relative to include_bytes!
Write-Host "Touching src/sys/fs.rs to force binary embed update..." -ForegroundColor Yellow
(Get-Content src/sys/fs.rs) | Set-Content src/sys/fs.rs

# Run Trunk
trunk build

Write-Host "Build Complete!" -ForegroundColor Green
