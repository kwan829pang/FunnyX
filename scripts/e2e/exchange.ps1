# Slice C — market approve (if pending) + Session place/cancel

function Invoke-E2eExchange {
    if (-not (Test-HttpOk "$script:Engine/health")) {
        Skip-Step "Market/exchange lifecycle" "core engine not reachable at $script:Engine"
        return
    }

    Step "Admin market approve if pending_locked" {
        if (-not $script:adminToken) { throw "adminToken missing" }
        $headers = Get-AuthHeaders $script:adminToken
        $m = Get-Json "$script:Admin/v1/admin/markets" $headers
        $list = @()
        if ($m.markets) { $list = @($m.markets) }
        $pending = @($list | Where-Object { [string]$_.status -eq "pending_locked" })
        if ($pending.Count -eq 0) {
            Write-Host "no pending_locked markets (ok if already active)"
            return
        }
        $id = [int]$pending[0].id
        $res = Post-Json "$script:Admin/v1/admin/markets/$id/approve" @{} $headers
        Write-Host ("approved market id=$id engine_activated=" + $res.engine_activated)
        if ($res.engine_activated -eq $false) {
            Write-Host "WARN: DB approved but engine activate failed — will try direct engine pair" -ForegroundColor Yellow
        }
    }

    Step "Session markets + place + cancel" {
        if (-not $script:clientToken) { throw "clientToken missing" }
        $headers = Get-AuthHeaders $script:clientToken
        $m = Get-Json "$script:Gateway/v1/markets" $headers
        $list = @()
        if ($m.markets) { $list = @($m.markets) }

        if ($list.Count -eq 0) {
            throw "no active Session markets — apply seed 01_demo_data (demo DEMO_COIN/PLT) and approve, or set QUOTE_ASSET=PLT on Core Engine"
        }

        $first = $list[0]
        $symbol = if ($first.symbol) { [string]$first.symbol } else { [string]$first.market_name }
        $marketId = [int]$first.id
        Write-Host "using market id=$marketId symbol=$symbol"

        # Ensure engine lists the pair (idempotent best-effort).
        try {
            $null = Invoke-RestMethod -Uri "$script:Engine/v1/admin/pairs" -Method Post -Headers @{
                "X-Admin-Key" = $script:EngineAdminKey
            } -ContentType "application/json" -Body (@{
                symbol            = $symbol
                market_id         = "mkt_$marketId"
                pool_depth        = 1000
                initial_price     = 1
                base_amount       = 100
                quote_amount      = 100
                funding_source    = "gamecoin_lockup"
                admin_id          = "1"
                corporate_user_id = 1
                seed_book         = $false
            } | ConvertTo-Json -Compress)
            Write-Host "engine pair ensured for $symbol"
        } catch {
            Write-Host "engine create pair note (may already exist): $_" -ForegroundColor Yellow
        }

        $order = Post-Json "$script:Gateway/v1/orders" @{
            symbol          = $symbol
            side            = "buy"
            order_type      = "price"
            price           = 1
            quantity        = 1
            game_account_id = $script:GameAccountId
            market_pair_id  = $marketId
        } $headers
        $oid = [int]$order.order.id
        if ($oid -le 0) { throw "place order missing id" }
        Write-Host "placed order id=$oid status=$($order.order.status) remaining=$($order.order.remaining)"

        $cancelled = Post-Json "$script:Gateway/v1/orders/$oid/cancel" @{} $headers
        $st = [string]$cancelled.order.status
        if ($st -notmatch "(?i)cancel") {
            # Engine may use numeric/string variants; accept remaining=0 or cancelled-like.
            if ([int64]$cancelled.order.remaining -ne 0 -and $st -notmatch "(?i)fill") {
                throw "cancel unexpected status=$st remaining=$($cancelled.order.remaining)"
            }
        }
        Write-Host "cancel/lifecycle ok status=$st remaining=$($cancelled.order.remaining)"
    }
}
