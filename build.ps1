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
    # 保留 dist 下的本地配置（config.yaml），重建目录后再还原
    $configBackup = Join-Path $env:TEMP "atomcode-switch-config.yaml.bak"
    $configPreserved = $false
    if (Test-Path (Join-Path $DistDir "config.yaml")) {
        Copy-Item (Join-Path $DistDir "config.yaml") -Destination $configBackup -Force
        $configPreserved = $true
    }
    Remove-Item $DistDir -Recurse -Force
}
New-Item -ItemType Directory -Path $DistDir | Out-Null
if ($configPreserved) {
    Move-Item $configBackup -Destination (Join-Path $DistDir "config.yaml") -Force
}

Copy-Item "target\release\$ProjectName.exe" -Destination "$DistDir\"

Write-Host "Output: $DistDir\$ProjectName.exe" -ForegroundColor Green
