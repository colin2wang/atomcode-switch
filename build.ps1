$ErrorActionPreference = "Stop"

$ProjectName = "atomcode-switch"
$DistDir = "dist"

Write-Host "Building $ProjectName (release)..." -ForegroundColor Cyan
cargo build --release

if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}

if (Test-Path $DistDir) {
    Remove-Item $DistDir -Recurse -Force
}
New-Item -ItemType Directory -Path $DistDir | Out-Null

Copy-Item "target\release\$ProjectName.exe" -Destination "$DistDir\"

Write-Host "Output: $DistDir\$ProjectName.exe" -ForegroundColor Green
