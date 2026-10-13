# Slice D — Marketplace eligibility gates + eligible place + intentional gate fail

function Invoke-E2eMarketplace {
    Step "Marketplace eligibility (demo_user)" {
        if (-not $script:clientToken) { throw "clientToken missing" }
        $headers = Get-AuthHeaders $script:clientToken
        $el = Get-Json "$script:Gateway/v1/marketplace/eligibility" $headers
        $e = $el.eligibility
        if (-not $e) { throw "missing eligibility object" }
        Write-Host ("can_place=$($e.can_place) plt=$($e.verified_platform_token) game=$($e.playing_listed_game) transfer=$($e.transfer_api_open) reasons=$($e.reasons -join ',')" )
        if ($e.can_place -ne $true) {
            throw "demo_user should be eligible (PLT wallet + listed game + transfer endpoint). reasons=$($e.reasons -join ',')"
        }
        $script:marketplaceEligible = $true
    }

    Step "Marketplace place deal when eligible" {
        if (-not $script:marketplaceEligible) { throw "eligibility step did not pass" }
        $headers = Get-AuthHeaders $script:clientToken
        $deal = Post-Json "$script:Gateway/v1/marketplace/deals" @{
            poster_game_account_id = $script:GameAccountId
            offer_asset_type       = "company_coin"
            offer_game_coin_id     = 2
            offer_amount           = 1
            want_asset_type        = "platform_token"
            want_game_coin_id      = 1
            want_amount            = 1
        } $headers
        if (-not $deal.id) { throw "place deal missing id" }
        Write-Host "placed deal id=$($deal.id) status=$($deal.status) fee=$($deal.listing_fee_plt)"
        $script:placedDealId = [int]$deal.id

        # Cancel to avoid leaving open inventory noise across runs.
        try {
            $c = Post-Json "$script:Gateway/v1/marketplace/deals/$($script:placedDealId)/cancel" @{} $headers
            Write-Host "cancelled deal status=$($c.status)"
        } catch {
            Write-Host "WARN: cancel deal: $_" -ForegroundColor Yellow
        }
    }

    Step "Marketplace gate FAIL (alice_plat no listed game bind)" {
        $alice = Post-Json "$script:Gateway/v1/client/login" @{
            username = $script:AliceUser
            password = $script:AlicePass
        }
        if (-not $alice.access_token) { throw "alice_plat login failed" }
        $ah = Get-AuthHeaders $alice.access_token
        $el = Get-Json "$script:Gateway/v1/marketplace/eligibility" $ah
        $e = $el.eligibility
        if ($e.can_place -eq $true) {
            throw "alice_plat unexpectedly eligible (expected gate fail)"
        }
        $reasons = @($e.reasons)
        if ($reasons.Count -eq 0) { throw "expected eligibility reasons for alice_plat" }
        Write-Host ("gate reasons: " + ($reasons -join ","))

        $threw = $false
        try {
            $null = Post-Json "$script:Gateway/v1/marketplace/deals" @{
                poster_game_account_id = 1
                offer_asset_type       = "company_coin"
                offer_game_coin_id     = 2
                offer_amount           = 1
                want_asset_type        = "platform_token"
                want_game_coin_id      = 1
                want_amount            = 1
            } $ah
        } catch {
            $threw = $true
            Write-Host "place rejected as expected: $_"
        }
        if (-not $threw) { throw "place deal should fail for alice_plat" }
    }
}
