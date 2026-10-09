# Partner and User Registration Integration

This doc is a detail file for the architecture in [project.md](project.md). See the product summary in [readme.md](../readme.md), the API index in [api-master.md](../api-master.md), and the UML in [partner-uml.md](partner-uml.md).

**Detail scope:** End-user and Corporate User registration payloads, **OAuth 2.0** partner login, partner endpoint contracts (deposit/withdraw, e-shop fiat, Transfer), C6 fee tables, market pair lock/approve flow, e-shop payment JSON, and Company Basic Token — not the master server map or auth model.

## 0. Required Company Partner APIs (from [readme.md](../readme.md))


| #   | API                                                                  | Always required?                                                                      |
| --- | -------------------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| 1   | **OAuth 2.0 Login** for user verify on the platform                  | **Yes**                                                                               |
| 2   | User current balance                                                 | Only if partner provides their own balance APIs instead of using platform wallet APIs |
| 3   | User deposit and withdrawal APIs                                     | Only if partner provides their own deposit/withdraw instead of platform APIs          |
| 4   | **Transfer** API for Game Items and coins (Company Coin / Game Coin) | **Yes**                                                                               |
| 5   | **Item / game-assets list** API (list out user account game assets)  | **Yes** for Marketplace **game_item** place/pick; used with Transfer                  |


**Rule:** If the partner prefers to provide their own APIs instead of using platform APIs, then **1, 2, 3, and 4** are all needed (plus **5** when Marketplace game items are enabled). Otherwise **1 and 4** are required; add **5** for Marketplace game-item listings.

**Bidirectional OAuth 2.0:**

- Platform provides OAuth 2.0 login for users already registered at the Partner.
- Partner OAuth (item 1) verifies users on the platform; it also allows a user who registered on the platform to login at the Partner.
- Suggest users open a normal user account on the platform (readme suggestion).

Detail flows: [client_connect.md](client_connect.md).

## 1. Registration Flow

Platform onboarding steps: [project.md](project.md) §4. Below: request shapes, partner APIs, and settlement rules.

### 1.1 Normal User Registration

End users may create an account on **Client Web**, use an account already at the **Company Partner**, or complete identity via **OAuth 2.0** (platform ↔ partner). After identity is established, **Session Token Server** returns a session token for Client Web → Gateway calls.

Requirements:

- **Path A — Client Web register:** User registers on Client Web / game entry page and binds to an existing game.
- **Path B — Partner OAuth / account:** User exists at the partner. Partner **must expose OAuth 2.0** (and may expose register/link) so the partner identity can verify, register, or bind into the platform account.
- **Path C — Platform OAuth for Partner:** User registered on platform; Partner uses platform OAuth 2.0 to login that user on the Partner side.
- The user selects an existing game from the supported game list.
- The system creates a user profile and maps it to the selected game account (and partner user id when Path B).
- The game identity should be stored with the user profile for later wallet, deposit, and withdrawal operations.
- Each user may link to one or more existing game accounts depending on business rules.

Recommended user registration payload:

```json
{
  "user_id": "user_1001",
  "username": "alice_01",
  "email": "alice@example.com",
  "game_id": "game_001",
  "game_account_id": "player_9001",
  "register_source": "web",
  "status": "active",
  "created_at": "2026-09-25T12:00:00Z"
}
```



### 1.2 Corporate User Registration

Corporate users must register their backend server endpoints so the platform can accept deposit and withdrawal requests from users.

Requirements:

- Corporate users register company or partner information.
- Corporate users submit server endpoint URLs for **OAuth 2.0**, optional deposit/withdrawal (if not using platform wallet APIs), e-shop **fiat** payment, and **Transfer** (game assets / coins) — Transfer required for Marketplace deals (C7).
- The platform validates endpoint availability and authentication mode.
- The system stores partner metadata, API key information, signature configuration, and active/inactive status.
- Only approved corporate partners with valid C6 contract fee status may use platform APIs; they also support OAuth-linked Client Web registration and marketplace asset transfers.

Recommended partner registration payload:

```json
{
  "partner_id": "partner_2001",
  "company_name": "Game Partner Co.",
  "server_name": "partner_gateway",
  "endpoints": [
    { "type": "oauth_authorize", "endpoint": "https://partner.example.com/oauth/authorize" },
    { "type": "oauth_token", "endpoint": "https://partner.example.com/oauth/token" },
    { "type": "oauth_userinfo", "endpoint": "https://partner.example.com/oauth/userinfo" },
    { "type": "deposit", "endpoint": "https://partner.example.com/api/deposit" },
    { "type": "withdrawal", "endpoint": "https://partner.example.com/api/withdrawal" },
    { "type": "transfer", "endpoint": "https://partner.example.com/api/transfer" },
    { "type": "payment", "endpoint": "https://partner.example.com/api/shop/payment" },
    { "type": "callback", "endpoint": "https://partner.example.com/api/callback" }
  ],
  "auth_type": "signature",
  "api_key": "***",
  "secret_key": "***",
  "status": "active",
  "created_at": "2026-09-25T12:00:00Z"
}
```



### 1.3 Required Partner OAuth 2.0 (user verify)

Master product text: [readme.md](../readme.md) Required Company Partner APIs.


| Concern     | Rule                                                                                                                    |
| ----------- | ----------------------------------------------------------------------------------------------------------------------- |
| Purpose     | OAuth 2.0 login so the platform can verify a partner user (and link/register into Client Web)                           |
| Direction A | Partner is OAuth provider → platform Client Web uses it for Path B verify                                               |
| Direction B | Platform is OAuth provider → Partner logins users already registered on platform                                        |
| Auth        | OAuth 2.0 authorize + token (+ userinfo); Client Web never holds Master secrets                                         |
| Result      | Partner user identity + game account binding usable by Client Center; then Session Token Server issues platform session |


Optional legacy register/link (non-OAuth) may still exist for migration; OAuth 2.0 is the required join path for user verify.

Recommended partner userinfo claim set (minimal):

```json
{
  "partner_user_id": "pu_9001",
  "game_id": "game_001",
  "game_account_id": "player_9001",
  "username": "alice_01",
  "status": "active"
}
```



### 1.4 Required Partner Transfer API (Marketplace / game assets)

**C7:** To join the platform for Marketplace deals, the Company Partner **must** provide a **Transfer** API the platform can call to move Game Items and coins (Company Coin / Game Coin) for deal settlement.


| Concern | Rule                                                                                              |
| ------- | ------------------------------------------------------------------------------------------------- |
| Purpose | Transfer game assets / coins between accounts for Marketplace deal fulfill                        |
| Caller  | Platform (after deal match / settle)                                                              |
| Auth    | Partner API key / signature                                                                       |
| Assets  | Game Items, Company Coin, Game Coin (as enabled per partner)                                      |
| Gate    | Marketplace **place deal** only accepts assets when this API is registered and open for that game |


Recommended transfer request (minimal):

```json
{
  "request_id": "xfer_7001",
  "partner_id": "partner_2001",
  "from_game_account_id": "player_9001",
  "to_game_account_id": "player_9002",
  "asset_type": "game_item",
  "item_ref_id": "sword_01",
  "game_coin": null,
  "amount": 1,
  "deal_id": "deal_4001",
  "created_at": "2026-09-28T04:00:00Z"
}
```

Without an approved Transfer endpoint, that partner’s game assets are **not** eligible for Marketplace listings.

### 1.5 Required Partner Item / game-assets list API (Marketplace game items)

**Readme Required Partner API #5:** list out user account game assets so Client Web can pick an item when placing a Marketplace deal.


| Concern | Rule                                                                                                                                                                                                          |
| ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Purpose | Return the user’s inventoriable game items (and related asset ids) for a game account                                                                                                                         |
| Caller  | Platform (on behalf of Session user browsing/placing Marketplace deals)                                                                                                                                       |
| Auth    | Partner API key / signature                                                                                                                                                                                   |
| Result  | Opaque `item_ref_id` (`VARCHAR`) per item — platform stores this on `marketplace_deals.offer_item_ref_id` / `marketplace_transfer_events.item_ref_id`; **not** a platform FK and **not** a local item catalog |
| Gate    | Place deal with `offer_asset_type = game_item` requires a successful list/pick of `item_ref_id` plus Transfer API open                                                                                        |


Recommended list response item (minimal):

```json
{
  "item_ref_id": "sword_01",
  "name": "Iron Sword",
  "qty": 1,
  "game_account_id": "player_9001"
}
```

On settle, Transfer uses the same `item_ref_id`. No platform escrow/inventory table in first draft — ownership moves via Transfer + deal status.

## 2. Payment Integration Requirements

The platform should support both deposit and withdrawal operations through a common business structure.

### 2.1 Simple End-to-End Flow



#### Normal User Flow

1. User enters the platform and chooses an existing game.
2. User registers on Client Web, or verifies via Partner/Platform **OAuth 2.0**, then Session Token Server issues the platform session.
3. The system binds the user to the selected game account.
4. User sees the wallet and payment entry page for that game.
5. User submits a deposit or withdrawal request.
6. The platform validates identity, balance, and game account mapping.
7. The request is sent to the registered corp partner server.
8. The partner server processes the request and returns the result.
9. The platform updates the wallet and records the transaction status.



#### Corporate Partner Flow

1. Corporate user registers company and server information.
2. Corporate user provides deposit endpoint and withdrawal endpoint.
3. The platform validates the endpoint and authentication method.
4. The system stores the partner as an approved payment provider.
5. When a user requests a deposit or withdrawal, the platform routes the request to the correct partner endpoint.
6. The partner server processes the transaction and returns success or failure.
7. The platform updates the wallet and pushes a status notification back to the user.



### 2.2 Deposit Flow

- User selects a game.
- User submits a deposit request.
- The platform validates user identity and game account.
- The request is forwarded to the connected corp server endpoint.
- The corp server processes the amount and returns success or failure.
- The platform updates the wallet, balance, or game state after confirmation.



### 2.3 Withdrawal Flow

- User requests a withdrawal from the game wallet.
- The platform validates available balance and account status.
- The system submits the withdrawal request to the corp server endpoint.
- The corp server confirms payment or transfer completion.
- The platform records the final wallet update and status message.



## 3. Common Data Structure for Deposit and Withdrawal

Both deposit and withdrawal requests should follow the same base structure so the service layer can handle them consistently.

```json
{
  "request_id": "req_778899",
  "partner_id": "partner_2001",
  "user_id": "user_1001",
  "game_id": "game_001",
  "game_account_id": "player_9001",
  "transaction_type": "deposit",
  "amount": 100.00,
  "game_coin": "PLT",
  "status": "pending",
  "channel": "partner_token",
  "source": "web",
  "created_at": "2026-09-25T12:00:00Z",
  "updated_at": "2026-09-25T12:00:05Z",
  "metadata": {
    "order_no": "ORD-1001",
    "remark": "top up game wallet"
  }
}
```

Field definitions:

- request_id: unique transaction request identifier
- partner_id: partner or corp user identity
- user_id: platform user identity
- game_id: selected game identifier
- game_account_id: mapped account in the game
- transaction_type: deposit or withdrawal
- amount: token amount for deposit or withdrawal
- game_coin: Platform / Company / Game Coin code. Corp may use token, crypto token, or stablecoin (e.g. USDT). Not ISO fiat HKD/USD.
- status: pending, processing, success, failed, rejected, cancelled
- channel: partner token channel or method
- source: web, mobile, admin, game client
- metadata: extra business fields such as order number or remarks



## 4. Shared Request Contract

To keep the system consistent, both deposit and withdrawal requests should use the same schema:

```json
{
  "request_id": "string",
  "partner_id": "string",
  "user_id": "string",
  "game_id": "string",
  "game_account_id": "string",
  "transaction_type": "deposit | withdrawal",
  "amount": "number",
  "game_coin": "string",
  "status": "pending | processing | success | failed | rejected | cancelled",
  "channel": "string",
  "source": "string",
  "created_at": "datetime",
  "updated_at": "datetime",
  "metadata": "object"
}
```



## 5. Platform Rules

- All deposit and withdrawal requests must be validated against user identity and game account mapping.
- The partner server must be registered and approved before it can receive payment traffic.
- Response codes and acknowledgement status must be tracked for every request.
- Failed requests should be retried according to a retry policy, but duplicates must be prevented by request_id.
- Sensitive data such as secret keys and private tokens should never be exposed in logs or response payloads.



## 6. Monitoring and Audit

The Message Center and payment layer should track:

- total deposit requests
- total withdrawal requests
- successful and failed counts
- retry counts
- response status from partner endpoints
- user and partner mapping validity
- audit trail of all transaction states



## 7. Corp Platform Fees (C6) — How Companies Pay to Use the Platform

Platform charges Corporate Users with a fixed fee schedule (amounts below). **Fiat denomination is Admin-configurable** — Admin sets system base fiat to **HKD** or **USD** (same Admin control as e-shop C4). Corp may pay that fiat **or** **USDT**. Not Platform Token for this fee schedule. Authoritative for corp licensing/pair fees; [readme.md](../readme.md) older 10k/80k PLT wording is superseded here.


| #   | Fee                   | Amount     | Currency                                         |
| --- | --------------------- | ---------- | ------------------------------------------------ |
| 1   | **Year-1 package**    | **80,000** | Admin base fiat (**HKD** or **USD**) or **USDT** |
| 2   | **Next-year license** | **10,000** | same                                             |
| 3   | **New trade pair**    | **10,000** | same                                             |


Year-1 package = onboard fee + **one-year license** + **one-time first pair** (bundled).

**Claim rule (pair → next-year license):** If the company already paid a **new trade pair** fee (**10,000**) in the current license year, that payment may be **claimed as the next-year license fee** (so the same 10K is not charged again as renewal for that following year). Tracking is per corporate account and license year.

Rules:

- Admin User configures `base_fiat_currency` = **HKD** or **USD**. Changing it (e.g. take **HKD** not **USD**) applies to new Corp fee invoices and e-shop list prices.
- Corp pays each invoice in the current Admin base fiat **or** in **USDT** (stablecoin rail; C5).
- Fee invoices snapshot `pay_currency` at pay time so a later Admin currency switch does not rewrite paid invoices.
- Year-1 **80K** is required before the company is fully onboarded for live market use (includes first pair entitlement).
- First market for a game must still pair with **Platform Token**.
- Pool lock for a market uses company/game **Game Partner Game Coin balance** (required pool amount). That lock is **not** the 80K fee.
- Separate from Company Basic Token 0.1% buy fee.



### 7.1 Partner “money movement” (C6) = contract fee payment

In partner onboarding language, **money movement** means the Corporate User’s **platform contract fee payment**, not end-user deposit/withdrawal:


| Payment                | Meaning                                                               |
| ---------------------- | --------------------------------------------------------------------- |
| **First-time payment** | Year-1 package (**80,000**) — onboard + license + first pair          |
| **Renew contract fee** | Next-year license (**10,000**), or pair fee claimed as renewal per §7 |


End-user deposit / withdrawal / Transfer are separate partner APIs and are **not** what “partner money movement” refers to here.

### 7.2 Admin: disable partner APIs + deadline notice

When first-time or renew fees are unpaid / overdue, Admin can cut partner API access and warn the partner before the deadline.


| Capability                                | Actor                          | Behavior                                                                                                     |
| ----------------------------------------- | ------------------------------ | ------------------------------------------------------------------------------------------------------------ |
| Set license / renew **deadline datetime** | Admin                          | Store `license_deadline_at` (UTC) on the corporate account                                                   |
| Send **notice message** to partner        | Admin / system                 | Notify partner of deadline datetime and fee type (first-time or renew); Message Center / corp notice channel |
| **Disable partner APIs**                  | Admin (or auto after deadline) | Reject Company Partner MasterSigned calls (`api_enabled = false`) until fee paid / Admin re-enables          |
| Re-enable APIs                            | Admin                          | After paid year-1 / renew (or manual override)                                                               |


Rules:

- While APIs are disabled: Corp Master Account Code + Master ID + API Key + Secret calls return disabled/forbidden (except fee-pay and read-notice endpoints as product allows).
- Notices must include at least: partner id, fee type, amount currency, **deadline datetime**, and whether APIs will be disabled at that time.
- Optional auto-job: when `now >= license_deadline_at` and fee not paid → set `api_enabled = false` and publish a final notice.



## 8. Market Pair Submission and Approval Flow

Corp users submit a Game Partner Game Coin pair for market creation after fee eligibility (§7).

**Client Submitted → Under Pending (lock required pool amount on client Game Partner Game Coin balance) → Wait Admin User Review → Confirm approval by Admin User → Transfer Client Balance to Pool**

### 8.1 Client Submitted

Requirements:

- Corporate users submit a Game Partner Game Coin pair. The **first** market for a game must be paired with the **Platform Token** (example: `GAME / PLT`).
- Year-1 package (**80K**) covers the **first** pair. Each later pair requires **10K** new-pair fee (or claim rule in §7).
- Submit pool depth, initial price, and required base/quote amounts with the request.
- Lock funding: required pool amount from company/game **Game Partner Game Coin balance** (not the 80K license fee).

Recommended market pair payload:

```json
{
  "pair_id": "pair_3001",
  "partner_id": "partner_2001",
  "base_gamecoin": "GAME",
  "quote_gamecoin": "PLT",
  "market_name": "GAME/PLT",
  "funding_source": "gamecoin_lockup",
  "pair_fee_amount": 10000,
  "pair_fee_currency": "HKD",
  "pair_fee_claim_next_year": false,
  "status": "submitted",
  "submitted_by": "corp_user_01",
  "created_at": "2026-09-25T12:00:00Z"
}
```



### 8.2 Under Pending (lock required amount)

- Platform moves the request to `pending` and **locks** the required pool amount on the **client Game Partner Game Coin balance**.
- Funds are reserved on the client side; they are **not** transferred into the market pool yet.
- Status example: `pending_locked`.

Recommended pending/lock payload:

```json
{
  "pair_id": "pair_3001",
  "pool_depth": 5000000.00,
  "initial_price": 0.025,
  "base_amount": 200000.00,
  "quote_amount": 5000.00,
  "funding_source": "gamecoin_lockup",
  "lock_status": "client_balance_locked",
  "status": "pending_locked",
  "start_time": "2026-09-25T12:30:00Z"
}
```



### 8.3 Wait Admin User Review / Confirm approval

- Admin panel reviews the submitted pair, Corp fee status (§7), funding source, and locked client amount.
- First market must include Platform Token as base or quote.
- Admin approves or rejects the pair.
- On reject: unlock the reserved client balance and return for correction.



### 8.4 Transfer Client Balance to Pool (after approval)

- On admin approval, transfer the locked client balance into the market pool wallet.
- Create/activate the market core engine.
- Market cannot start live trading until transfer completes and the engine is active.
- Pool depth and initial price must stay consistent with the locked amounts.



## 9. E-shop Partner Payment Integration

Corporate partners expose a **fiat** payment endpoint used by the Client Web e-shop for **platform fixed** `PLT_`* packages **and** **Corp-created e-shop products**. Currency is the Admin-configured system base (**HKD** or **USD**). Flow is separate from exchange trading and from game deposit/withdrawal, but reuses partner registration and callback security. Create/pay payloads use `fiat_currency` / `fiat_price` (or `fiat_paid`); credit/fulfill after webhook. See [e-shop.md](e-shop.md) (C2 + C4).

### 9.1 Create shop payment

Recommended create-payment payload:

```json
{
  "request_id": "req_shop_1001",
  "shop_order_id": 5001,
  "partner_id": "partner_2001",
  "user_id": "user_1001",
  "game_account_id": "player_9001",
  "package_code": "PLT_1000",
  "credit_game_coin": "PLT",
  "credit_amount": 1000,
  "fiat_currency": "HKD",
  "fiat_price": 88.00,
  "callback_url": "https://platform.example.com/webhook/shop/payment",
  "return_url": "https://client.example.com/shop/orders/5001",
  "created_at": "2026-09-28T01:00:00Z"
}
```

Recommended create-payment response:

```json
{
  "partner_order_no": "PAY-7788",
  "checkout_url": "https://partner.example.com/pay/PAY-7788",
  "status": "pending",
  "expires_at": "2026-09-28T01:30:00Z"
}
```



### 9.2 Shop payment callback

```json
{
  "event_id": "pay_evt_8899",
  "partner_order_no": "PAY-7788",
  "shop_order_id": 5001,
  "status": "paid",
  "fiat_currency": "HKD",
  "fiat_paid": 88.00,
  "credit_game_coin": "PLT",
  "credit_amount": 1000,
  "paid_at": "2026-09-28T01:05:00Z",
  "signature": "***"
}
```

Rules:

- Partner endpoints used for e-shop must be registered and approved.
- E-shop: fixed `PLT_*` credit amounts plus Admin `fiat_price`; Corp products also fiat-priced. Currency = Admin system base (**HKD** or **USD**).
- Callback status values: `paid`, `failed`, `cancelled`.
- Settlement is idempotent on `event_id` and `partner_order_no`.
- Platform Token is credited only after a valid paid callback.
- Full e-shop behavior is defined in `doc/e-shop.md`.



## 10. Company Basic Token Submission

Corporate Users may submit their Company Basic Token for Admin approval. Only after approval may end users buy that token.

Recommended submit payload:

```json
{
  "request_id": "req_cbt_1001",
  "partner_id": "partner_2001",
  "game_id": "game_001",
  "token_code": "GAMECOIN",
  "token_name": "Game Company Basic Token",
  "buy_enabled_after_approve": true,
  "status": "submitted",
  "created_at": "2026-09-28T01:00:00Z"
}
```

Buy settlement rules after approval:

- End users buy via partner **token** payment (`/v1/webhook/corp-token/payment`).
- Platform charges **0.1%** of the purchased Company Basic Token as fee; user receives **99.9%**.
- Company Basic Token is separate from fixed Platform Token e-shop packages (`PLT_*`).



## 11. Summary

The platform supports these main flows:

1. Normal user registration on Client Web **or** Partner/Platform **OAuth 2.0** verify; Session Token Server issues platform session; bind to an existing game.
2. Corporate join: required **OAuth 2.0** + **Transfer**; balance/deposit/withdraw only if partner uses their own APIs instead of platform wallet APIs.
3. **C6 Corp fees:** Year-1 **80K** / renewal **10K** / extra pair **10K** (Admin fiat or USDT). Partner “money movement” = first-time or renew fee only (§7.1). **C9:** Admin may set deadline, notice partner, and **disable APIs** if unpaid (§7.2).
4. Corporate market flow: submit → pending lock required pool on Game Partner Game Coin client balance → admin review/approve → transfer to pool → activate core engine. First market must pair with Platform Token.
5. E-shop partner **fiat** payment (HKD/USD): platform fixed `PLT_`* **and** Corp-created products; credit/fulfill after webhook settlement.
6. Company Basic Token: Corp submit → Admin approve → user buyable (token pay); 0.1% Company Basic Token buy fee to platform (separate from Corp e-shop products).
7. **C7 Marketplace:** place deal only if user bought Platform Token, plays ≥1 listed game, and partner Transfer API is open for that asset. Game items require partner Item List API (#5); store opaque `item_ref_id`.

This approval-based market creation flow keeps the system controlled, auditable, and safe before a market enters the live trading stage.