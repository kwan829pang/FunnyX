# Slice A — e-shop buy + webhook settle + duplicate event_id no double-credit

function Invoke-E2eShop {
    if (-not (Test-HttpOk "$script:PaymentGate/health")) {
        Skip-Step "E-shop buy + settle" "payment-gate not reachable at $script:PaymentGate"
        return
    }
    if (-not (Test-HttpOk "$script:Webhook/health")) {
        Skip-Step "E-shop buy + settle" "webhook_server not reachable at $script:Webhook"
        return
    }

    Step "E-shop platform package buy + settle + duplicate safe" {
        if (-not $script:clientToken) { throw "clientToken missing (login first)" }
        $headers = Get-AuthHeaders $script:clientToken

        $before = Get-Json "$script:Gateway/v1/client/wallet" $headers
        $pltBefore = Get-CoinAvailable $before "PLT"

        $pkgs = Get-Json "$script:Gateway/v1/shop/packages" $headers
        $pkg = $null
        foreach ($p in @($pkgs.packages)) {
            if ([string]$p.code -eq "PLT_1000") { $pkg = $p; break }
        }
        if (-not $pkg) { throw "PLT_1000 package not found (seed shop_packages?)" }
        $credit = [double]$pkg.coin_amount
        $fiat = [double]$pkg.fiat_price
        $fiatCur = if ($pkgs.fiat_currency) { [string]$pkgs.fiat_currency } else { "HKD" }

        $order = Post-Json "$script:Gateway/v1/shop/orders" @{
            seller_type     = "platform"
            package_id      = [int]$pkg.id
            game_account_id = $script:GameAccountId
        } $headers
        if (-not $order.id) { throw "create order missing id" }
        $partnerOrderNo = [string]$order.partner_order_no
        if ([string]::IsNullOrWhiteSpace($partnerOrderNo)) { throw "order missing partner_order_no" }
        if ($partnerOrderNo.StartsWith("LOCAL-")) {
            throw "payment-gate did not create checkout (partner_order_no=$partnerOrderNo); check PAYMENT_GATE_URL"
        }
        Write-Host "order id=$($order.id) partner_order_no=$partnerOrderNo credit=$credit"

        # Mark paid at payment-gate without auto-callback so we control event_id.
        $null = Post-Json "$script:PaymentGate/v1/shop/payment/$partnerOrderNo/complete" @{
            status         = "paid"
            fire_callback  = $false
        }

        $eventId = "e2e_shop_$(Get-Date -Format 'yyyyMMddHHmmss')_$([guid]::NewGuid().ToString('N').Substring(0,8))"
        $postAuth = Get-PostAuthHash
        $cbHeaders = @{
            "X-Post-Auth"        = $postAuth
            "X-Callback-Source"  = "ppk_payment_gate"
        }
        $payload = @{
            event_id         = $eventId
            partner_order_no = $partnerOrderNo
            shop_order_id    = [int]$order.id
            seller_type      = "platform"
            status           = "paid"
            fiat_currency    = $fiatCur
            fiat_paid        = $fiat
            credit_game_coin = "PLT"
            credit_amount    = $credit
            paid_at          = (Get-Date).ToUniversalTime().ToString("o")
            signature        = $script:CallbackSignature
            source           = "test"
        }

        $first = Post-Json "$script:Webhook/v1/webhook/shop/payment" $payload $cbHeaders
        if (-not $first.accepted) { throw "webhook not accepted: $($first | ConvertTo-Json -Compress)" }
        if ($first.duplicate -eq $true) { throw "first settle unexpectedly duplicate" }

        Start-Sleep -Milliseconds 500
        $paid = Get-Json "$script:Gateway/v1/shop/orders/$($order.id)" $headers
        if ([string]$paid.status -ne "paid") { throw "order status=$($paid.status) want paid" }

        $after = Get-Json "$script:Gateway/v1/client/wallet" $headers
        $pltAfter = Get-CoinAvailable $after "PLT"
        $delta = $pltAfter - $pltBefore
        if ([math]::Abs($delta - $credit) -gt 0.0001) {
            throw "PLT delta=$delta want $credit (before=$pltBefore after=$pltAfter)"
        }

        $txns = Get-Json "$script:Gateway/v1/client/transactions?limit=20" $headers
        $topup = $null
        foreach ($t in @($txns.transactions)) {
            if ([string]$t.transaction_type -eq "shop_topup" -and [string]$t.status -eq "completed") {
                $topup = $t
                break
            }
        }
        if (-not $topup) { throw "shop_topup completed txn not found" }

        $replay = Post-Json "$script:Webhook/v1/webhook/shop/payment" $payload $cbHeaders
        if ($replay.duplicate -ne $true -and $replay.accepted -ne $true) {
            throw "replay not accepted: $($replay | ConvertTo-Json -Compress)"
        }
        # Same event_id must be duplicate; if accepted again, still no double-credit.
        if ($replay.duplicate -ne $true) {
            Write-Host "WARN: replay duplicate flag false; checking wallet anyway" -ForegroundColor Yellow
        }

        Start-Sleep -Milliseconds 300
        $after2 = Get-Json "$script:Gateway/v1/client/wallet" $headers
        $pltReplay = Get-CoinAvailable $after2 "PLT"
        if ([math]::Abs($pltReplay - $pltAfter) -gt 0.0001) {
            throw "double-credit detected: after settle=$pltAfter after replay=$pltReplay"
        }
        Write-Host "shop settle ok; PLT $pltBefore → $pltAfter; duplicate event_id safe"
    }
}
