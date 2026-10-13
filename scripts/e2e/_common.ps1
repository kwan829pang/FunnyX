# Shared helpers for Phase 6 local E2E modules.
# Dot-sourced by scripts/local_e2e_checklist.ps1

$script:Gateway = if ($env:GATEWAY_URL) { $env:GATEWAY_URL.TrimEnd('/') } else { "http://127.0.0.1:8080" }
$script:Admin = if ($env:ADMIN_URL) { $env:ADMIN_URL.TrimEnd('/') } else { "http://127.0.0.1:18300" }
$script:Engine = if ($env:ENGINE_URL) { $env:ENGINE_URL.TrimEnd('/') } else { "http://127.0.0.1:18200" }
$script:Webhook = if ($env:WEBHOOK_URL) { $env:WEBHOOK_URL.TrimEnd('/') } else { "http://127.0.0.1:8084" }
$script:PaymentGate = if ($env:PAYMENT_GATE_URL) { $env:PAYMENT_GATE_URL.TrimEnd('/') } else { "http://127.0.0.1:18100" }
$script:CompanyA = if ($env:COMPANY_A_URL) { $env:COMPANY_A_URL.TrimEnd('/') } else { "http://127.0.0.1:18102" }

$script:BotUser = if ($env:BOT_USER) { $env:BOT_USER } else { "demo_user" }
$script:BotPass = if ($env:BOT_PASS) { $env:BOT_PASS } else { "demo" }
$script:AliceUser = if ($env:ALICE_USER) { $env:ALICE_USER } else { "alice_plat" }
$script:AlicePass = if ($env:ALICE_PASS) { $env:ALICE_PASS } else { "demo" }
$script:AdminUser = if ($env:ADMIN_USER) { $env:ADMIN_USER } else { "seed_admin" }
$script:AdminPass = if ($env:ADMIN_PASS) { $env:ADMIN_PASS } else { "demo" }
$script:GameAccountId = if ($env:GAME_ACCOUNT_ID) { [int]$env:GAME_ACCOUNT_ID } else { 1 }
$script:CallbackSignature = if ($env:CALLBACK_SIGNATURE) { $env:CALLBACK_SIGNATURE } else { "test-signature" }
$script:PostAuthKey = if ($env:POST_AUTH_KEY) { $env:POST_AUTH_KEY } else { "demo_key" }
$script:EngineAdminKey = if ($env:ENGINE_ADMIN_KEY) { $env:ENGINE_ADMIN_KEY } else { "demo-admin-key" }

$script:failed = 0
$script:passed = 0
$script:skipped = 0

function Step([string]$name, [scriptblock]$block) {
    Write-Host ""
    Write-Host "==> $name" -ForegroundColor Cyan
    try {
        & $block
        if ($null -ne $LASTEXITCODE -and $LASTEXITCODE -ne 0) {
            throw "exit $LASTEXITCODE"
        }
        Write-Host "PASS: $name" -ForegroundColor Green
        $script:passed++
    } catch {
        Write-Host "FAIL: $name — $_" -ForegroundColor Red
        $script:failed++
    }
}

function Skip-Step([string]$name, [string]$reason) {
    Write-Host ""
    Write-Host "==> $name" -ForegroundColor Cyan
    Write-Host "SKIP: $reason" -ForegroundColor Yellow
    $script:skipped++
}

function Get-Json($url, $headers = $null) {
    if ($headers) {
        return Invoke-RestMethod -Uri $url -Headers $headers -Method Get
    }
    return Invoke-RestMethod -Uri $url -Method Get
}

function Post-Json($url, $body, $headers = $null) {
    $json = if ($null -eq $body) { "{}" } elseif ($body -is [string]) { $body } else { $body | ConvertTo-Json -Compress -Depth 10 }
    if ($headers) {
        return Invoke-RestMethod -Uri $url -Headers $headers -Method Post -Body $json -ContentType "application/json"
    }
    return Invoke-RestMethod -Uri $url -Method Post -Body $json -ContentType "application/json"
}

function Test-HttpOk([string]$url) {
    try {
        $null = Invoke-RestMethod -Uri $url -Method Get -TimeoutSec 3
        return $true
    } catch {
        return $false
    }
}

function Get-PostAuthHash([string]$suffix = $script:PostAuthKey) {
    $sha = [System.Security.Cryptography.SHA256]::Create()
    try {
        $bytes = [Text.Encoding]::UTF8.GetBytes("Post_Auth_$suffix")
        $hash = $sha.ComputeHash($bytes)
        return ([BitConverter]::ToString($hash) -replace '-', '').ToLowerInvariant()
    } finally {
        $sha.Dispose()
    }
}

function Get-AuthHeaders([string]$token) {
    return @{ Authorization = "Bearer $token" }
}

function Get-CoinAvailable($walletResp, [string]$code) {
    if (-not $walletResp.wallets) { return 0.0 }
    foreach ($w in @($walletResp.wallets)) {
        $c = if ($w.game_coin) { [string]$w.game_coin } else { "" }
        if ($c.Equals($code, [StringComparison]::OrdinalIgnoreCase)) {
            return [double]$w.available
        }
    }
    return 0.0
}

function Wait-TxnStatus(
    [hashtable]$headers,
    [int]$txnId,
    [string]$wantStatus = "completed",
    [int]$timeoutSec = 15
) {
    $deadline = (Get-Date).AddSeconds($timeoutSec)
    while ((Get-Date) -lt $deadline) {
        $txns = Get-Json "$script:Gateway/v1/client/transactions?limit=50" $headers
        foreach ($t in @($txns.transactions)) {
            if ([int]$t.id -eq $txnId) {
                if ([string]$t.status -eq $wantStatus) { return $t }
                if ([string]$t.status -in @("failed", "cancelled", "rejected")) {
                    throw "txn $txnId terminal status=$($t.status)"
                }
            }
        }
        Start-Sleep -Milliseconds 400
    }
    throw "txn $txnId did not reach $wantStatus within ${timeoutSec}s"
}
