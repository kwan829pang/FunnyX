# FunnyX Phase 6 §24 — local cargo deep E2E checklist (not Docker).
# Usage (repo root):
#   pwsh -File scripts/local_e2e_checklist.ps1
#   powershell -File scripts/local_e2e_checklist.ps1
#
# Required services: Config, STS, Client Center, Gateway, Admin API, Webhook,
# Message Center, Core Engine, Postgres (+ 00_seed + 01_demo).
# Optional simulators (SKIP when down): payment-gate :18100, company-a :18102.
# Core Engine for exchange slice: QUOTE_ASSET=PLT (matches demo DEMO_COIN/PLT).

$ErrorActionPreference = "Continue"

. "$PSScriptRoot\e2e\_common.ps1"
. "$PSScriptRoot\e2e\shop.ps1"
. "$PSScriptRoot\e2e\money.ps1"
. "$PSScriptRoot\e2e\exchange.ps1"
. "$PSScriptRoot\e2e\marketplace.ps1"

Step "Gateway /health" {
    $h = Get-Json "$script:Gateway/health"
    if (-not $h.service) { throw "missing service" }
    if (-not $h.upstreams) { Write-Host "WARN: no upstreams field (older gateway?)" -ForegroundColor Yellow }
    Write-Host ($h | ConvertTo-Json -Compress)
}

Step "Core Engine /health" {
    $h = Get-Json "$script:Engine/health"
    if (-not $h.status -and -not $h.service) { throw "unexpected body" }
}

Step "Admin API /health" {
    Get-Json "$script:Admin/health" | Out-Null
}

Step "Client login via Gateway" {
    $session = Post-Json "$script:Gateway/v1/client/login" @{
        username = $script:BotUser
        password = $script:BotPass
    }
    if (-not $session.access_token) { throw "no access_token" }
    $script:clientToken = $session.access_token
    Write-Host "end_user session ok ($script:BotUser)"
}

Step "Session wallet" {
    $headers = Get-AuthHeaders $script:clientToken
    $w = Get-Json "$script:Gateway/v1/client/wallet" $headers
    $plt = Get-CoinAvailable $w "PLT"
    Write-Host "PLT available=$plt"
}

Step "Admin login" {
    $a = Post-Json "$script:Admin/v1/admin/login" @{
        username = $script:AdminUser
        password = $script:AdminPass
    }
    if (-not $a.access_token) { throw "no admin access_token" }
    $script:adminToken = $a.access_token
}

Step "Admin markets list" {
    $headers = Get-AuthHeaders $script:adminToken
    $m = Get-Json "$script:Admin/v1/admin/markets" $headers
    $n = 0
    if ($m.markets) { $n = @($m.markets).Count }
    Write-Host "admin markets count=$n"
}

# Phase 6 deep paths
Invoke-E2eShop
Invoke-E2eMoney
Invoke-E2eExchange
Invoke-E2eMarketplace

Write-Host ""
Write-Host "Result: $script:passed passed, $script:failed failed, $script:skipped skipped" -ForegroundColor $(if ($script:failed -eq 0) { "Green" } else { "Red" })
if ($script:failed -gt 0) { exit 1 }
exit 0
