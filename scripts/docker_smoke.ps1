# Phase 5 §21/§24 — compose mesh /health smoke.
# Usage (after deploy/docker-compose.yml is up):
#   pwsh -File scripts/docker_smoke.ps1
$ErrorActionPreference = "Continue"
$failed = 0

$urls = @(
    "http://127.0.0.1:8080/health",
    "http://127.0.0.1:8090/health",
    "http://127.0.0.1:8082/health",
    "http://127.0.0.1:8083/health",
    "http://127.0.0.1:18300/health",
    "http://127.0.0.1:8084/health",
    "http://127.0.0.1:8085/health",
    "http://127.0.0.1:18200/health"
)

foreach ($u in $urls) {
    Write-Host "==> $u" -ForegroundColor Cyan
    try {
        $r = Invoke-RestMethod -Uri $u -Method Get -TimeoutSec 5
        Write-Host ("PASS " + ($r | ConvertTo-Json -Compress)) -ForegroundColor Green
    } catch {
        Write-Host "FAIL $u — $_" -ForegroundColor Red
        $failed++
    }
}

if ($failed -gt 0) {
    Write-Host "Result: $failed failed" -ForegroundColor Red
    exit 1
}
Write-Host "PASS: docker smoke" -ForegroundColor Green
exit 0
