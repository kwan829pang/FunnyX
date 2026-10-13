# Slice B — deposit / withdrawal partner path via company-a → webhook → wallet

function Invoke-E2eMoney {
    if (-not (Test-HttpOk "$script:CompanyA/health")) {
        Skip-Step "Deposit/withdraw partner path" "company-a-server not reachable at $script:CompanyA"
        return
    }
    if (-not (Test-HttpOk "$script:Webhook/health")) {
        Skip-Step "Deposit/withdraw partner path" "webhook_server not reachable at $script:Webhook"
        return
    }

    Step "Deposit partner settle" {
        if (-not $script:clientToken) { throw "clientToken missing" }
        $headers = Get-AuthHeaders $script:clientToken
        $before = Get-Json "$script:Gateway/v1/client/wallet" $headers
        $coinBefore = Get-CoinAvailable $before "DEMO_COIN"
        $amount = 10.0

        $resp = Post-Json "$script:Gateway/v1/client/deposit" @{
            game_account_id = $script:GameAccountId
            game_coin       = "DEMO_COIN"
            amount          = $amount
        } $headers
        $txnId = [int]$resp.txn.id
        if ($txnId -le 0) { throw "deposit missing txn.id" }
        Write-Host "deposit txn=$txnId partner_txn_id=$($resp.partner_txn_id) status=$($resp.partner_status)"

        $done = Wait-TxnStatus $headers $txnId "completed" 20
        $after = Get-Json "$script:Gateway/v1/client/wallet" $headers
        $coinAfter = Get-CoinAvailable $after "DEMO_COIN"
        $delta = $coinAfter - $coinBefore
        if ([math]::Abs($delta - $amount) -gt 0.0001) {
            throw "DEMO_COIN delta=$delta want $amount (txn status=$($done.status))"
        }
        Write-Host "deposit settled; DEMO_COIN $coinBefore → $coinAfter"
        $script:lastDepositTxnId = $txnId
    }

    Step "Withdraw partner settle" {
        if (-not $script:clientToken) { throw "clientToken missing" }
        $headers = Get-AuthHeaders $script:clientToken
        $before = Get-Json "$script:Gateway/v1/client/wallet" $headers
        $coinBefore = Get-CoinAvailable $before "DEMO_COIN"
        $amount = 5.0
        if ($coinBefore -lt $amount) { throw "insufficient DEMO_COIN for withdraw (have $coinBefore)" }

        $resp = Post-Json "$script:Gateway/v1/client/withdraw" @{
            game_account_id = $script:GameAccountId
            game_coin       = "DEMO_COIN"
            amount          = $amount
        } $headers
        $txnId = [int]$resp.txn.id
        if ($txnId -le 0) { throw "withdraw missing txn.id" }
        Write-Host "withdraw txn=$txnId partner_txn_id=$($resp.partner_txn_id)"

        $done = Wait-TxnStatus $headers $txnId "completed" 20
        $after = Get-Json "$script:Gateway/v1/client/wallet" $headers
        $coinAfter = Get-CoinAvailable $after "DEMO_COIN"
        $delta = $coinBefore - $coinAfter
        if ([math]::Abs($delta - $amount) -gt 0.0001) {
            throw "DEMO_COIN withdraw delta=$delta want $amount (txn status=$($done.status))"
        }

        $txns = Get-Json "$script:Gateway/v1/client/transactions?limit=30" $headers
        $foundWithdraw = $false
        foreach ($t in @($txns.transactions)) {
            if ([int]$t.id -eq $txnId -and [string]$t.status -eq "completed") {
                $foundWithdraw = $true
                break
            }
        }
        if (-not $foundWithdraw) { throw "completed withdrawal txn not in history" }
        Write-Host "withdraw settled; DEMO_COIN $coinBefore → $coinAfter"
    }
}
