$ErrorActionPreference = "Stop"

# Ensure we have a linked project in root
if (-not (Test-Path ".vercel")) {
    Write-Host "Root project not linked. Linking now..."
    vercel link --yes
}

Write-Host "Building Release..."
trunk build --release

if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed"
    exit 1
}

Write-Host "Preparing dist for deployment..."
# Copy config files to dist so Vercel knows what to do
Copy-Item "vercel.json" -Destination "dist/" -Force
# Copy the project link from root so we don't get asked again
Copy-Item ".vercel" -Destination "dist/" -Recurse -Force

Write-Host "Deploying to Vercel (Production)..."
Set-Location dist
# --yes skips the "Are you sure?" prompt
vercel deploy --prod --yes
Set-Location ..

Write-Host "Deployment Complete."
