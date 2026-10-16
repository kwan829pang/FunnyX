# Bring up FunnyX full stack: Postgres/Redis/Mongo + platform + simulators.
# Usage (repo root):
#   pwsh -File scripts/docker_up_all.ps1
#   pwsh -File scripts/docker_up_all.ps1 -Down
#   pwsh -File scripts/docker_up_all.ps1 -Rebuild
param(
    [switch]$Down,
    [switch]$NoBuild
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$compose = Join-Path $root "deploy\docker-compose.all.yml"

if (-not (Test-Path $compose)) {
    Write-Error "Missing $compose"
}

Set-Location $root

if ($Down) {
    docker compose -f $compose down
    exit $LASTEXITCODE
}

$args = @("compose", "-f", $compose, "up", "-d")
if (-not $NoBuild) {
    $args += "--build"
}

Write-Host "==> docker $($args -join ' ')" -ForegroundColor Cyan
& docker @args
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "==> waiting for gateway health..." -ForegroundColor Cyan
$deadline = (Get-Date).AddMinutes(8)
$ok = $false
while ((Get-Date) -lt $deadline) {
    try {
        $null = Invoke-RestMethod -Uri "http://127.0.0.1:8080/health" -TimeoutSec 3
        $ok = $true
        break
    } catch {
        Start-Sleep -Seconds 5
    }
}

docker compose -f $compose ps
if (-not $ok) {
    Write-Host "Gateway not healthy yet — first Rust image builds can take a long time." -ForegroundColor Yellow
    Write-Host "Check: docker compose -f deploy/docker-compose.all.yml logs -f" -ForegroundColor Yellow
    exit 1
}

Write-Host "PASS: full stack is up (gateway healthy)" -ForegroundColor Green
Write-Host "Smoke: pwsh -File scripts/docker_smoke.ps1" -ForegroundColor Cyan
exit 0
