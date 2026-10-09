# API Master

Master index of HTTP APIs across FunnyX. Product rules: [readme.md](readme.md). E-shop: [doc/e-shop.md](doc/e-shop.md). Auth detail: [doc/client_connect.md](doc/client_connect.md).

## Auth model (C8 — Client Web Token vs Company Partner Master)

### Client Web → platform (Token Server)

**All Client Web API calls** to the platform use a **token from the Session Token Server** (after login or OAuth). Master secrets never go into Client Web.

1. **Register** on Client Web, **or** verify via Partner **OAuth 2.0** (readme Required Partner API #1).
2. **Login** / OAuth callback (Public entry) → Session Token Server returns token.
3. Every subsequent Client Web API (e-shop, wallet, marketplace, orders, notices, sockets, etc.) must send that **Token Server token**.

Platform also exposes **OAuth 2.0** so Partners can login users already registered on the platform. Detail: [doc/client_connect.md](doc/client_connect.md), [doc/partner.md](doc/partner.md) §0.

```text
Client Web  --Token (Session Token Server)-->  Gateway  -->  Client Center
```

### Company Partner → Client Center (Master credentials)

**Direct** Company Partner (Client) calls to Client Center / platform corp APIs require:

- **Master Account Code**
- **Master ID**
- **API Key**
- **Secret** (signed request)

```text
Company Partner  --Master Account Code + Master ID + API Key + Secret-->  Gateway / Client Center
```

Auth legend (platform-facing columns in tables below):

| Code | Meaning |
| --- | --- |
| Public | **Non-auth** entry only: **no** Token Server token yet (register / login to obtain token). |
| Session | Client Web: Token Server token **required** on the HTTP/socket API call |
| MasterSigned / Corp | Company Partner → Client Center: Master Account Code + Master ID + API Key + Secret + timestamp |
| Admin | Admin role session / RBAC |
| Internal | Service-to-service or IP allowlist |
| Partner | Partner payment/webhook / Transfer / Partner OAuth endpoints |
| OAuth | OAuth 2.0 authorize / token / userinfo (Public browser redirect + confidential client for Partner) |

**C3:** `Public` = no session token on that call (login/register/OAuth entry only). **C8:** all other Client Web APIs require Token Server token; Company Partner → Client Center requires Master Account Code + Master ID + API Key + Secret.

---

## 1. Base (all service instances)

| Method | Path | Auth | Description |
| --- | --- | --- | --- |
| GET | `/health` | Public | Check that the server instance is alive and ready |
| POST | `/reload` | Admin / Internal | Reload local config (for example whitelist). Only if the instance supports it |

---

## 2. Client Center API

### 2.1 Common User API

Platform APIs for End Users. **Client Web** uses **Session** after password login or **OAuth 2.0** callback. Register on Client Web, or verify via Partner OAuth.

| Method | Path | Auth | Description |
| --- | --- | --- | --- |
| POST | `/v1/client/register` | Public | Register end user on Client Web and bind to a game / game account |
| POST | `/v1/client/login` | Public | Password login; Session Token Server returns session token |
| POST | `/v1/client/logout` | Session | Invalidate current session |
| GET | `/v1/oauth/authorize` | Public / OAuth | Platform OAuth 2.0 authorize (Partner logs in platform-registered users) |
| POST | `/v1/oauth/token` | OAuth | Platform OAuth 2.0 token exchange |
| GET | `/v1/oauth/userinfo` | OAuth | Platform OAuth 2.0 userinfo |
| GET | `/v1/client/oauth/partner/{partner_id}/start` | Public | Start Partner OAuth 2.0 via Session Token Server broker |
| GET | `/v1/client/oauth/partner/complete` | Public | After STS Partner callback; Client Center upserts user and returns/redirects session |
| GET | `/v1/client/oauth/partner/callback` | Public | Alias → STS Partner callback or CC complete |
| GET | `/v1/client/profile` | Session | Get current user profile |
| GET | `/v1/client/games` | Public | List active games available to bind (`game_id` must exist) |
| GET | `/v1/client/game-accounts` | Session | List linked game accounts for current end user |
| POST | `/v1/client/game-accounts/bind` | Session | Bind game account: `partner_fetch` (Path A) or `direct` (Path B / Game App) |
| GET | `/v1/client/wallet` | Session | Get wallet / balance for Platform Token and game coins |
| POST | `/v1/client/deposit` | Session | Create deposit request to partner endpoint (when partner exposes deposit API) |
| POST | `/v1/client/withdraw` | Session | Create withdrawal request to partner endpoint (when partner exposes withdraw API) |
| GET | `/v1/client/transactions` | Session | List deposit, withdrawal, and shop_topup transactions |
| POST | `/v1/session/token` | Public / login / OAuth path | Issue or refresh session token (Session Token Server) |
| POST | `/v1/session/validate` | Internal | Validate session token for Gateway HTTP/socket |

### 2.2 Corp User API

Corporate (partner) integrations authenticate with **Master Account Code**, **Master ID**, **API Key**, and **Secret** (signed requests) — same hop-#2 MasterSigned model as Client Server → Gateway. Used by game-company backends for corp operations and payment sync; not by end-user browsers calling Gateway directly.

Auth for this section: `Corp` = Master ID + API Key + signed Secret + timestamp. See [doc/client_connect.md](doc/client_connect.md) and [doc/partner.md](doc/partner.md).

| Method | Path | Auth | Description |
| --- | --- | --- | --- |
| POST | `/v1/corp/register` | Public | Register corporate company profile (pending approval) |
| POST | `/v1/corp/login` | Corp | Corporate login / obtain short-lived access context |
| GET | `/v1/corp/profile` | Corp | Get company profile, master code, and status |
| POST | `/v1/corp/api-keys` | Corp | Create API key and secret (secret shown once) |
| GET | `/v1/corp/api-keys` | Corp | List API keys (no secret values) |
| PATCH | `/v1/corp/api-keys/{id}/status` | Corp | Enable / disable an API key |
| PUT | `/v1/corp/endpoints` | Corp | Register or update **OAuth 2.0**, deposit/withdrawal (if used), payment, callback, and **Transfer** endpoints |
| GET | `/v1/corp/endpoints` | Corp | Get registered partner endpoints and status |
| POST | `/v1/corp/games` | Corp | Submit game information for onboarding |
| GET | `/v1/corp/games` | Corp | List company games and ready-state status |
| GET | `/v1/corp/games/{id}` | Corp | Get one game and Game Account ID mapping |
| POST | `/v1/corp/basic-tokens` | Corp | Submit Company Basic Token for admin approval (not buyable until approved) |
| GET | `/v1/corp/basic-tokens` | Corp | List company basic token submissions and approval status |
| GET | `/v1/corp/basic-tokens/{id}` | Corp | Get one Company Basic Token detail |
| PUT | `/v1/corp/basic-tokens/{id}` | Corp | Update a rejected/pending Company Basic Token before approval |
| GET | `/v1/corp/basic-tokens/{id}/buy-orders` | Corp | List user buy orders for this Company Basic Token |
| GET | `/v1/corp/basic-tokens/{id}/fees` | Corp | List 0.1% Company Basic Token buy fees collected on this token |
| POST | `/v1/corp/shop/products` | Corp | Create Corp e-shop product for sale (fiat_price in system base HKD/USD; Game Coin / Company Coin package or game item) |
| GET | `/v1/corp/shop/products` | Corp | List this company’s e-shop products |
| GET | `/v1/corp/shop/products/{id}` | Corp | Get one Corp e-shop product |
| PUT | `/v1/corp/shop/products/{id}` | Corp | Update draft/inactive Corp e-shop product |
| PATCH | `/v1/corp/shop/products/{id}/status` | Corp | Set `draft` / `active` / `inactive` / `archived` |
| GET | `/v1/corp/shop/orders` | Corp | List buy orders for this company’s e-shop products |
| GET | `/v1/corp/shop/orders/{order_id}` | Corp | Corp e-shop order detail |
| POST | `/v1/corp/payments/notify` | Corp | Push game/partner end-user payment accounting (who paid, item, amount). Not the e-shop wallet-credit path |
| PUT | `/v1/corp/payments/{payment_id}` | Corp | Update an existing corp payment record (amount, item, status). Does not replace shop webhook settlement |
| GET | `/v1/corp/payments` | Corp | Payment history (filter by user, game account, item, status, time range) |
| GET | `/v1/corp/payments/{payment_id}` | Corp | Payment detail for one end-user payment |
| GET | `/v1/corp/users/{user_id}/payments` | Corp | Payment history for one end user under this company |
| GET | `/v1/corp/fees` | Corp | List Corp platform fee invoices (year-1 80K, renewal 10K, pair 10K) |
| GET | `/v1/corp/fees/{id}` | Corp | Fee invoice detail and claim status |
| POST | `/v1/corp/fees/year1` | Corp | Pay year-1 package **80,000** in Admin base fiat (HKD/USD) or USDT (first-time contract money movement) |
| POST | `/v1/corp/fees/renewal` | Corp | Pay next-year license **10,000** (renew contract fee); or apply pair-fee claim |
| POST | `/v1/corp/fees/pair` | Corp | Pay new trade pair **10,000** in Admin base fiat or USDT; may flag claim toward next-year license |
| GET | `/v1/corp/license` | Corp | License status, `api_enabled`, `license_deadline_at` |
| GET | `/v1/corp/notices` | Corp | Partner notices (contract fee deadline / API disable warnings) |
| POST | `/v1/corp/markets` | Corp | Client Submitted market pair; locks required pool on Game Partner Game Coin balance; first market must pair with Platform Token; C6 fee gated |
| GET | `/v1/corp/markets` | Corp | List company market pairs and approval status |
| POST | `/v1/corp/game-coins/supply` | Corp | Submit Game Partner Game Coin supply increase request |
| GET | `/v1/corp/transactions` | Corp | Company-scoped deposit / withdrawal / shop-related txn history |

Payment notify / update payload (core fields):

| Field | Description |
| --- | --- |
| `user_id` | Platform end-user id |
| `game_account_id` | Game-side account id |
| `item_code` / `package_code` | What was paid for |
| `amount` | Token amount paid |
| `game_coin` | Platform / Company / Game Coin code. May be token, crypto token, or stablecoin (e.g. `USDT`). Not ISO fiat `HKD`/`USD` (those are e-shop pay currency only) |
| `status` | `pending` / `paid` / `failed` / `refunded` |
| `partner_order_no` | Corporate or partner payment reference |
| `paid_at` | Payment completion time (UTC BIGINT or ISO) |

#### 2.2.1 Corp Test API

Sandbox-only APIs for verifying auth, connectivity, and business logic between the platform and the game partner. Must not affect production balances or live markets. Mark all generated data as test / simulated.

| Method | Path | Auth | Description |
| --- | --- | --- | --- |
| GET | `/v1/corp/test/ping` | Corp | Auth + reachability check; returns master code, timestamp, and echo nonce |
| POST | `/v1/corp/test/echo` | Corp | Echo signed request body to validate Master ID / API Key / Secret signature |
| POST | `/v1/corp/test/endpoints/probe` | Corp | Platform calls partner deposit / withdrawal / payment / callback / transfer URLs and returns latency + HTTP status |
| GET | `/v1/corp/test/endpoints/status` | Corp | Last probe results for registered partner endpoints |
| POST | `/v1/corp/test/payments/simulate` | Corp | Simulate end-user payment notify (item, amount, status) without live settlement |
| POST | `/v1/corp/test/payments/{payment_id}/replay` | Corp | Replay a test payment update to verify idempotent handling |
| GET | `/v1/corp/test/payments` | Corp | List test payment history only (`source = test`) |
| POST | `/v1/corp/test/deposit/simulate` | Corp | Simulate deposit request/response path between platform and partner |
| POST | `/v1/corp/test/withdraw/simulate` | Corp | Simulate withdrawal request/response path between platform and partner |
| POST | `/v1/corp/test/webhook/callback` | Corp | Trigger a test webhook callback to the partner callback endpoint |
| GET | `/v1/corp/test/connection-report` | Corp | Summary report: auth OK, endpoint probes, last simulate results, failures |

Test rules:

- Available only when the corporate account or environment flag allows sandbox mode.
- Test paths must use distinct ids / `partner_order_no` prefixes (for example `TEST-`).
- Simulate APIs must not credit live wallets or change live market/pool state.
- Connection report is the preferred first check after partner endpoint registration.

---

## 3. Admin API (includes E-shop setup)

Admin Panel → Admin API. **C2:** Platform lane is fixed `PLT_*` packages; Corp lane products are created by Corp Users. **C4:** Admin sets system base fiat (`HKD`/`USD`); all e-shop listed items use that fiat for price and partner pay.

| Method | Path | Auth | Description |
| --- | --- | --- | --- |
| POST | `/v1/admin/login` | Public | Admin login |
| POST | `/v1/admin/logout` | Admin | Admin logout |
| GET | `/v1/admin/me` | Admin | Current admin profile and role |
| GET | `/v1/admin/game-coins` | Admin | List Game Partner Game Coin definitions |
| POST | `/v1/admin/game-coins` | Admin | Create Game Partner Game Coin (`type` platform/company/game; `asset_kind` token/crypto_token/stablecoin; Corp choice not blocked) |
| PUT | `/v1/admin/game-coins/{id}` | Admin | Update Game Partner Game Coin |
| PATCH | `/v1/admin/game-coins/{id}/status` | Admin | Enable / disable Game Partner Game Coin |
| GET | `/v1/admin/basic-tokens` | Admin | List Company Basic Token submissions |
| GET | `/v1/admin/basic-tokens/{id}` | Admin | Company Basic Token detail |
| POST | `/v1/admin/basic-tokens/{id}/approve` | Admin | Approve token → becomes buyable for end users |
| POST | `/v1/admin/basic-tokens/{id}/reject` | Admin | Reject token; not buyable |
| GET | `/v1/admin/corporate-users` | Admin | List corporate users |
| POST | `/v1/admin/corporate-users/{id}/approve` | Admin | Approve corporate onboarding |
| PATCH | `/v1/admin/corporate-users/{id}/api-access` | Admin | Enable / **disable partner APIs** (`api_enabled`) for unpaid/overdue C6 contract fees |
| PUT | `/v1/admin/corporate-users/{id}/license-deadline` | Admin | Set renew / first-payment **deadline datetime** (UTC) |
| POST | `/v1/admin/corporate-users/{id}/notices` | Admin | Send partner notice (fee type, amount, **deadline datetime**, API-disable warning) |
| GET | `/v1/admin/corporate-users/{id}/notices` | Admin | List partner contract-fee notices |
| GET | `/v1/admin/fees` | Admin | List Corp C6 fee invoices (80K / 10K / 10K; pay_currency HKD\|USD\|USDT) |
| GET | `/v1/admin/fees/{id}` | Admin | Fee invoice detail and claim linkage |
| GET | `/v1/admin/markets` | Admin | List market pairs |
| POST | `/v1/admin/markets` | Admin | Process / register approved market definition to Core Engine (corp submits; admin does not invent the pair) |
| POST | `/v1/admin/markets/{id}/approve` | Admin | Confirm approval; transfer locked client balance to pool; activate engine |
| POST | `/v1/admin/markets/{id}/reject` | Admin | Reject pending market pair and unlock reserved client balance |
| GET | `/v1/admin/system/setup` | Admin | Setup wizard status: required variables (`base_fiat_currency`, PLT packages, `system_initialized`) |
| POST | `/v1/admin/system/setup/complete` | Admin | Mark system initialized after required variables are set |
| GET | `/v1/admin/system/base-currency` | Admin | Get system base fiat (`HKD` or `USD`) — e-shop prices **and** C6 Corp fee fiat denomination |
| PUT | `/v1/admin/system/base-currency` | Admin | Set system base fiat (`HKD` or `USD`); e.g. switch to HKD instead of USD for e-shop + Corp fees |
| GET | `/v1/admin/shop/packages` | Admin | List platform fixed packages (any status) |
| POST | `/v1/admin/shop/packages` | Admin | Seed a fixed `PLT_*` platform package row (credit amount + fiat_price) |
| PUT | `/v1/admin/shop/packages/{id}` | Admin | Update platform package display name and **fiat_price** |
| PATCH | `/v1/admin/shop/packages/{id}/status` | Admin | Set platform package `active` / `inactive` / `archived` |
| GET | `/v1/admin/shop/corp-products` | Admin | List Corp e-shop products (monitor) |
| GET | `/v1/admin/shop/corp-products/{id}` | Admin | Corp e-shop product detail |
| PATCH | `/v1/admin/shop/corp-products/{id}/status` | Admin | Suspend / force `inactive` / `archived` on Corp product |
| GET | `/v1/admin/shop/orders` | Admin | List shop orders (platform + corp; filter by status / user / seller_type) |
| GET | `/v1/admin/shop/orders/{id}` | Admin | Shop order detail and payment events |
| GET | `/v1/admin/action-logs` | Admin | Admin action audit log |

---

## 4. Client API (includes E-shop buy)

Platform APIs for e-shop / market / notices. **Client Web** calls Gateway with **Session** after Token Server login.

**C2 + C4 e-shop:** Lane A platform fixed `PLT_*` + Lane B Corp products; all catalog prices and partner pay in system base **fiat** (`HKD`/`USD`). Credit/fulfill after webhook.

| Method | Path | Auth | Description |
| --- | --- | --- | --- |
| GET | `/v1/shop/base-currency` | Session | Current system base fiat currency (`HKD` or `USD`) for catalog display |
| GET | `/v1/shop/packages` | Session | List active platform fixed packages (credit amount + fiat_price) |
| GET | `/v1/shop/packages/{id}` | Session | Get one active platform package |
| GET | `/v1/shop/corp-products` | Session | List active Corp e-shop products (fiat_price in base currency) |
| GET | `/v1/shop/corp-products/{id}` | Session | Get one active Corp e-shop product |
| POST | `/v1/shop/orders` | Session | Create pending shop order (`seller_type` platform or corp) and start partner **fiat** payment |
| GET | `/v1/shop/orders` | Session | List current user shop orders (both lanes) |
| GET | `/v1/shop/orders/{id}` | Session | Get shop order status (`pending` / `paid` / `failed` / `expired` / `cancelled`) |
| POST | `/v1/shop/orders/{id}/cancel` | Session | Cancel a pending shop order before payment completes |
| POST | `/v1/internal/shop/settle` | Internal | Webhook Server settle (`X-Internal-Key`); paid/failed/cancelled + wallet credit |
| GET | `/v1/corp-tokens` | Session | List Admin-approved buyable Company Basic Tokens (separate from Corp shop products) |
| GET | `/v1/corp-tokens/{id}` | Session | Company Basic Token detail and buy offer |
| POST | `/v1/corp-tokens/{id}/orders` | Session | Create buy order for Company Basic Token (partner token pay) |
| GET | `/v1/corp-tokens/orders` | Session | List current user Company Basic Token buy orders |
| GET | `/v1/corp-tokens/orders/{order_id}` | Session | Company Basic Token buy order status |
| POST | `/v1/corp-tokens/orders/{order_id}/cancel` | Session | Cancel pending Company Basic Token buy order |
| GET | `/v1/markets` | Session | List active trading markets |
| GET | `/v1/markets/{id}/ticker` | Session | Market ticker snapshot |
| POST | `/v1/orders` | Session | Place exchange order (Market / Price) |
| GET | `/v1/orders` | Session | List user exchange orders |
| GET | `/v1/orders/{id}` | Session | Exchange order detail |
| POST | `/v1/orders/{id}/cancel` | Session | Cancel open exchange order |
| GET | `/v1/marketplace/eligibility` | Session | C7 gates: Platform Token verified, listed-game play, Transfer-eligible assets |
| GET | `/v1/marketplace/items` | Session | List partner game assets for place-deal pick (proxies partner Item List API; returns `item_ref_id`) |
| GET | `/v1/marketplace/deals` | Session | List open Marketplace deals |
| GET | `/v1/marketplace/deals/{id}` | Session | Marketplace deal detail |
| POST | `/v1/marketplace/deals` | Session | Place/post a deal (C7 + Transfer; `offer_item_ref_id` required when offering game_item) |
| POST | `/v1/marketplace/deals/{id}/request` | Session | Request a deal against an existing listing |
| POST | `/v1/marketplace/deals/{id}/cancel` | Session | Cancel own open deal |
| GET | `/v1/notices` | Session | *(prefer Message Center inbox)* List user notices / messages |
| GET | `/v1/notifications` | Session | Message Center user inbox (`backend/message_center`) |
| PATCH | `/v1/notifications/{id}` | Session | Mark notification read |

### Message Center (`backend/message_center`, HTTP **8085**; socket **9005**)

| Method | Path | Auth | Description |
| --- | --- | --- | --- |
| GET | `/health` | Public | Liveness (Cloudflare / partners) |
| POST | `/v1/notices` | Internal | Enqueue outbound notice (Core Engine) |
| GET | `/v1/notices` | Internal | List/filter outbox |
| GET | `/v1/notices/stats` | Internal | Delivery status counts |
| GET | `/v1/notices/{id}` | Internal | Inspect notice |
| GET | `/v1/notifications` | Session | End-user inbox after drain |
| PATCH | `/v1/notifications/{id}` | Session | Mark read |

Detail: [doc/message_center.md](doc/message_center.md).

---

## 5. Webhook / Partner callback APIs

Inbound callbacks to **Webhook Server** (`backend/webhook_server`, HTTP **8084**; S2S socket **9004**). Not for Client Web direct use. Client Web may poll `GET /v1/shop/orders/{id}` or Message Center `GET /v1/notifications` until Gateway socket delivery exists.

| Method | Path | Auth | Description |
| --- | --- | --- | --- |
| GET | `/health` | Public | Webhook Server liveness |
| POST | `/v1/webhook/shop/payment` | Partner | E-shop **fiat** payment callback (platform package or Corp product); idempotent persist + Client Center settle on `event_id` / `partner_order_no` |
| GET | `/v1/webhook/endpoints` | MasterSigned / Corp | List this Corp user's callback endpoint rows (`kind`: shop_payment / deposit / withdrawal / corp_token) |
| POST | `/v1/webhook/endpoints` | MasterSigned / Corp | Create/upsert this Corp user's endpoint row (stores `secret_hint` only) |
| PUT | `/v1/webhook/endpoints/{id}` | MasterSigned / Corp | Update this Corp user's endpoint row |
| GET | `/v1/webhook/events/{event_id}` | Internal | Debug inbound event (`X-Internal-Key`) |
| POST | `/v1/webhook/corp-token/payment` | Partner | Company Basic Token buy payment callback; credit 99.9%, take 0.1% fee; notice End User + Corp; HTTP balance callback to Corp server |
| POST | `/v1/webhook/deposit` | Partner | *(reserved)* External deposit notice |
| POST | `/v1/webhook/withdrawal` | Partner | *(reserved)* External withdrawal result notice |
| POST | `/v1/webhook/partner/callback` | Partner | *(reserved)* Generic partner callback |

---

## 6. Config Server API

| Method | Path | Auth | Description |
| --- | --- | --- | --- |
| GET | `/health` | Public | Config Server health |
| POST | `/reload` | Admin / Internal | Reload whitelist / JSON config |
| GET | `/v1/config/whitelist` | Admin / Internal | Get current allowlist snapshot |
| GET | `/v1/config/services` | Admin / Internal | List registered Gateway / Core Engine status |
| POST | `/v1/config/services` | Internal | Register a service instance |
| POST | `/v1/config/services/heartbeat` | Internal | HTTP heartbeat for a registered instance |
| GET | `/v1/config/device-status` | Admin / Internal | Device / instance status map |

---

## Notes

- Product master rules: [readme.md](readme.md).
- **Auth (C8 + OAuth):** Client Web: register/login/OAuth Public → then **all** Client Web APIs use Token Server token. Partner required APIs: **OAuth 2.0** + **Transfer** (balance/deposit/withdraw only if partner does not use platform wallet APIs); **Item / game-assets list** for Marketplace game items (`item_ref_id`). Company Partner → Client Center: **Master Account Code + Master ID + API Key + Secret**.
- Shop orders are not exchange orders; they never enter the Core Engine matching path.
- **C2 + C4 E-shop:** Platform fixed `PLT_*` **and** Corp-created products. All listed items priced/paid in Admin-configured system base fiat (**HKD** or **USD**). Partner fiat pay → webhook → token credit or item fulfill via `POST /v1/webhook/shop/payment` (not `/v1/corp/payments/notify`).
- Company Basic Token remains separate: Corp submit → Admin approve → buyable with tokens; **0.1%** fee; settle via `POST /v1/webhook/corp-token/payment`.
- Paid shop / corp-token settlement must be idempotent; duplicate partner callbacks must not double-credit or double-fulfill.
- **C5:** Company Coin / Game Coin may be a platform-style token, crypto token, or stablecoin; platform does not block the choice. Fiat `HKD`/`USD` is e-shop pay currency only, not a Game Partner Game Coin code.
- **C6 / C9 Corp fees:** Year-1 **80,000** / renewal **10,000** / new pair **10,000** (Admin fiat or USDT). Partner **money movement** = first-time or renew contract fee. Admin may set deadline, notice partner, and **disable partner APIs** if unpaid. See [doc/partner.md](doc/partner.md) §7.
- First market for a game must pair with Platform Token. Pool lock uses client Game Partner Game Coin balance (separate from the 80K/10K fee schedule).
- Wallet/deposit/withdraw/CBT amounts use Game Coin or company token codes (any allowed `asset_kind`). E-shop listed-item pay uses fiat `HKD`/`USD` (C4).
- Market flow: Client Submitted → Under Pending (lock required Game Partner Game Coin amount) → Admin review/approve → Transfer Client Balance to Pool.
- **C7 Marketplace place deal:** user verified (bought Platform Token on platform) + playing ≥1 listed game + deal asset’s partner has Transfer API open; game_item offers need partner Item List `item_ref_id`. See [doc/marketplace.md](doc/marketplace.md).
- Corp User APIs require Master Account Code, Master ID, API Key, and Secret; never expose Secret to end users.
- Corp Test APIs (`/v1/corp/test/...`) are sandbox-only for platform ↔ partner connection and logic checks; they must not touch live balances or markets.
- Prefer versioned paths under `/v1/...`; Gateway remains the public ingress for Client Server and Admin traffic.
