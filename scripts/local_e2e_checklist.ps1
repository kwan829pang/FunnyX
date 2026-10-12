# FunnyX Phase 5 §24 — local cargo E2E checklist (not Docker).
# Usage (repo root):
#   pwsh -File scripts/local_e2e_checklist.ps1
$ErrorActionPreference = "Continue"

$Gateway = if ($env:GATEWAY_URL) { $env:GATEWAY_URL.TrimEnd('/') } else { "http://127.0.0.1:8080" }
$Admin = if ($env:ADMIN_URL) { $env:ADMIN_URL.TrimEnd('/') } else { "http://127.0.0.1:18300" }
$Engine = if ($env:ENGINE_URL) { $env:ENGINE_URL.TrimEnd('/') } else { "http://127.0.0.1:18200" }
$BotUser = if ($env:BOT_USER) { $env:BOT_USER } else { "demo_user" }
$BotPass = if ($env:BOT_PASS) { $env:BOT_PASS } else { "demo" }
$AdminUser = if ($env:ADMIN_USER) { $env:ADMIN_USER } else { "seed_admin" }
$AdminPass = if ($env:ADMIN_PASS) { $env:ADMIN_PASS } else { "demo" }

$failed = 0
$passed = 0

function Step([string]$name, [scriptblock]$block) {
    Write-Host ""
    Write-Host "==> $name" -ForegroundColor Cyan
    try {
        & $block
        if ($LASTEXITCODE -and $LASTEXITCODE -ne 0) {
            throw "exit $LASTEXITCODE"
        }
        Write-Host "PASS: $name" -ForegroundColor Green
        $script:passed++
    } catch {
        Write-Host "FAIL: $name — $_" -ForegroundColor Red
        $script:failed++
    }
}

function Get-Json($url, $headers = $null) {
    if ($headers) {
        return Invoke-RestMethod -Uri $url -Headers $headers -Method Get
    }
    return Invoke-RestMethod -Uri $url -Method Get
}

function Post-Json($url, $body, $headers = $null) {
    $json = $body | ConvertTo-Json -Compress -Depth 8
    if ($headers) {
        return Invoke-RestMethod -Uri $url -Headers $headers -Method Post -Body $json -ContentType "application/json"
    }
    return Invoke-RestMethod -Uri $url -Method Post -Body $json -ContentType "application/json"
}

Step "Gateway /health" {
    $h = Get-Json "$Gateway/health"
    if (-not $h.service) { throw "missing service" }
    if (-not $h.upstreams) { Write-Host "WARN: no upstreams field (older gateway?)" -ForegroundColor Yellow }
    Write-Host ($h | ConvertTo-Json -Compress)
}

Step "Core Engine /health" {
    $h = Get-Json "$Engine/health"
    if (-not $h.status -and -not $h.service) { throw "unexpected body" }
}

Step "Admin API /health" {
    Get-Json "$Admin/health" | Out-Null
}

Step "Client login via Gateway" {
    $session = Post-Json "$Gateway/v1/client/login" @{ username = $BotUser; password = $BotPass }
    if (-not $session.access_token) { throw "no access_token" }
    $script:clientToken = $session.access_token
    Write-Host "end_user session ok"
}

Step "Session wallet" {
    $headers = @{ Authorization = "Bearer $($script:clientToken)" }
    $w = Get-Json "$Gateway/v1/client/wallet" $headers
    Write-Host ("wallet keys: " + (($w | Get-Member -MemberType NoteProperty).Name -join ","))
}

Step "Admin login" {
    $a = Post-Json "$Admin/v1/admin/login" @{ username = $AdminUser; password = $AdminPass }
    if (-not $a.access_token) { throw "no admin access_token" }
    $script:adminToken = $a.access_token
}

Step "Admin markets list" {
    $headers = @{ Authorization = "Bearer $($script:adminToken)" }
    $m = Get-Json "$Admin/v1/admin/markets" $headers
    Write-Host ("markets count hint: " + ($m | ConvertTo-Json -Compress).Substring(0, [Math]::Min(120, ($m | ConvertTo-Json -Compress).Length)))
}

Step "Session markets + optional place" {
    $headers = @{ Authorization = "Bearer $($script:clientToken)" }
    $m = Get-Json "$Gateway/v1/markets" $headers
    $list = @()
    if ($m.markets) { $list = @($m.markets) }
    Write-Host "session markets: $($list.Count)"
    if ($list.Count -eq 0) {
        Write-Host "SKIP: no markets to place against" -ForegroundColor Yellow
        return
    }
    $first = $list[0]
    $symbol = if ($first.symbol) { $first.symbol } else { $first.market_name }
    $ga = if ($env:GAME_ACCOUNT_ID) { [int]$env:GAME_ACCOUNT_ID } else { 1 }
    try {
        $order = Post-Json "$Gateway/v1/orders" @{
            symbol           = $symbol
            side             = "buy"
            order_type       = "price"
            price            = 1
            quantity         = 1
            game_account_id  = $ga
        } $headers
        Write-Host ("placed order id=" + $order.order.id)
    } catch {
        Write-Host "WARN: place order failed (engine/pair/wallet): $_" -ForegroundColor Yellow
    }
}

Step "Optional duplicate webhook settle" {
    if (-not $env:SHOP_ORDER_ID -or -not $env:EVENT_ID) {
        Write-Host "SKIP: set SHOP_ORDER_ID and EVENT_ID to exercise webhook" -ForegroundColor Yellow
        return
    }
    Write-Host "SKIP: wire explicit webhook curl for your env (payment-gate / webhook_server)" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Result: $passed passed, $failed failed" -ForegroundColor $(if ($failed -eq 0) { "Green" } else { "Red" })
if ($failed -gt 0) { exit 1 }
exit 0
