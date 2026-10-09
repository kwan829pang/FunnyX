# Development Schedule

This schedule is derived from the architecture and design documents under `doc` and `db_config`, with product rules mastered in [readme.md](readme.md). It prioritizes a usable vertical slice first: **database → client registration → e-shop API/frontend**, then the remaining platform services in dependency order. Educational / non-commercial scope follows the readme disclaimer.

## Current status (2026-10-02)

| Phase | Status | Notes |
| --- | --- | --- |
| **1. Database** | **In progress** | PostgreSQL DDL **done**; Redis/Mongo + env samples **done**; seed SQL **done**; Docker bootstrap / smoke **not started**; **dev_simulator** (payment-gate, message-queue, company-a-server, market-bot + Compose) **done** |
| **2. Client register / login** | **In progress** | **STS** OAuth **done**; **Client Center** login + Postgres identity **done** (`8083`); Client Web wired to CC; Gateway still open |
| 3. E-shop API + frontend | **In progress** | Client Center shop + Webhook settle + Client Web GridView/cart/checkout **done**; wallet UI open |
| **4+. Platform expansion** | **Partial** | **Core Engine** done; **Admin API** auth **done** (`18300`); **Config Server** **done** (`8090`); **Webhook Server** **done** (`8084`/`9004`); Gateway/Message still open |

**PostgreSQL (done):** [`database/`](database/) — **12** `fx_*` schemas, **45** tables, deferred FKs, indexes, `run_all.sql`. Aligned with [db_config/database_relationships.md](db_config/database_relationships.md) (corp incl. supply requests + payment records, game, user, market, market_data/OHLCV, marketplace, shop, corp_token, money + status logs, admin, events/`outbound_notices` + `webhook_endpoints`). Timestamps: BIGINT UTC. Seeds: [`database/seed/`](database/seed/) — `00_seed_data.sql` (live+demo), `01_demo_data.sql` (dev only).

**Still open in Phase 1:** local Docker/init for PostgreSQL/Redis/Mongo, smoke-test apply schema + seeds, wire simulators to seed IDs.

## Recommended Execution Order

1. Database schema and query contracts ← **schema + seed SQL + simulators done; finish Docker/smoke**
2. Client registration / login (API + Client Web) ← **STS + Client Center login/OAuth done; next: Client Web ↔ Client Center + Postgres**
3. E-shop API and frontend (packages, checkout, webhook credit)
4. Shared contracts, Config Server, and Gateway ← **Config Server done (`8090`); Gateway next**
5. Session Token Server and Client Center hardening ← **STS + CC auth baseline done; Postgres + Redis prod path**
6. Webhook Server (partner payment callbacks) ← **done (`8084` / socket `9004`)**
7. Admin API / Admin Panel (packages, markets, Game Partner Game Coin) ← **auth login/logout/me done via STS**
8. Message Center (notices and partner push)
9. Market onboarding and Game Partner Game Coin / pool lock-up
10. Core Engine and exchange order matching ← **engine skeleton + maintenance done; wire via Client Center**
11. Health, Docker local env, simulation bot
12. Hardening and end-to-end validation

---

## Phase 1 - Database First

### 1. PostgreSQL schema and ownership — **DONE**
- [x] Implement first-draft schema under [`database/`](database/) (`run_all.sql`).
- [x] Align entities with `db_config/database_relationships.md`:
  - accounts, sessions, wallets/balances
  - Game Partner Game Coin, markets, exchange orders/trades, OHLCV (`fx_market_data`)
  - shop packages, shop orders, shop payment events
  - corp token + fee ledger
  - marketplace deals / requests / transfer events (`fx_marketplace`)
  - deposit/withdrawal + status logs; `outbound_notices` / notifications
  - corp Game Partner Game Coin supply requests + end-user `corp_payment_records`
- [x] Indexes for corp, account, shop order, partner order number, market, marketplace, outbound notice, and status lookups (`99_indexes.sql`).
- [x] UTC BIGINT timestamp convention.
- [x] Supply-increase + corp payment history tables (`fx_corp/02_supply_and_payments.sql`).

### 2. Redis and MongoDB contracts — **DONE** (draft; shared IP URLs for early phase)
- [x] Draft Redis keys in [db_config/queries/redis_queries.md](db_config/queries/redis_queries.md) (session/token, shop catalog/TTL, market ticker/book).
- [x] Draft MongoDB collections in [db_config/queries/mongodb_queries.md](db_config/queries/mongodb_queries.md) (notifications mirror, webhook events, shop payment notices, **Client Center `chat_messages`** required).
- [x] Env connection samples use `xxx@xxx.xxx.xxx.xxx` IP form in [`common/env.sample`](common/env.sample). Per-service-group dedicated store ownership is deferred (not required in this development phase).
- Keep PostgreSQL as source of truth for balances, shop settlement, and `outbound_notices`; Client Center MongoDB required for chat history.

### 3. Seed and local DB bootstrap — **IN PROGRESS**
- [x] Seed SQL folder [`database/seed/`](database/seed/):
  - `00_seed_data.sql` — **live + demo** baseline (seed admin, `base_fiat_currency=HKD`, Platform Token `PLT`, fixed `PLT_*` packages with fiat prices)
  - `01_demo_data.sql` — **development only** (demo corp/game/user/corp product/wallets)
  - runners: `seed/run_seed.sql` (00), `seed/run_seed_dev.sql` (00+01)
- [ ] Provide local Docker/init scripts for PostgreSQL, Redis, and MongoDB (shared instances OK for early development).
- [ ] Smoke-test: `run_all.sql` + `seed/run_seed_dev.sql`; package list / order insert / payment-event / outbound_notice checks.

### 4. Dev simulator — **DONE** (seed wiring still open)
- [x] Create [`dev_simulator/`](dev_simulator/) workspace (see also [doc/dev.md](doc/dev.md)).
- [x] Complete **payment-gate** — Rust API (`dev_simulator/payment-gate`, port `18100`): shop fiat create/complete + deposit create/complete + webhook callbacks (`source=test`).
- [x] Complete **company-a-server** — Rust API (`dev_simulator/company-a-server`, port `18102`): OAuth 2.0, Transfer, Item List, optional balance/deposit/withdraw; UML [doc/partner-uml.md](doc/partner-uml.md).
- [x] Complete **message-queue** — Rust API (`dev_simulator/message-queue`, port `18101`): enqueue / drain / ack aligned with `outbound_notices`.
- [x] Complete **market-bot** — Rust bot (`dev_simulator/market-bot`, port `18103`): poll Core Engine books, rest liquidity, take for partial/full fills (Gateway/Client Center later).
- [ ] Wire simulators to seed/demo data (`database/seed/01_demo_data.sql`) and `common/env.sample` IP-style URLs.
- [x] Document how to run payment-gate + message-queue + company-a-server + market-bot (`dev_simulator/README.md` + per-folder READMEs).
- [x] Docker Compose for all dev tools: [`dev_simulator/docker-compose.yml`](dev_simulator/docker-compose.yml) (+ Core Engine).

---

**Flutter frontends (skeleton):** [`frontend/`](frontend/) — `common` (`funnyx_common`), `admin_panel`, `client_web` (envied + **GetX** state/routing; login/register pages; JWT/session token on Session APIs). Partner OAuth UI entry on Client Web still open.

**Session Token Server (done):** [`backend/session_token_server`](backend/session_token_server/) — sessions + **OAuth 2.0** (platform IdP + Partner broker); port `8082`.

**Client Center (login/OAuth done):** [`backend/client_center`](backend/client_center/) — register/login/logout/profile; Partner OAuth start/complete + Platform OAuth entry delegated to STS; port `8083`. Postgres identity (`POSTGRES_URL`) with Argon2id + in-memory fallback.

**Admin API (auth + corp products + system setup):** [`backend/admin_api`](backend/admin_api/) — `/v1/admin/login|logout|me`; Corp e-shop monitor/suspend; system setup (`/v1/admin/system/setup*`, base currency, PLT packages); STS `actor_type=admin`; port `18300`.

**Config Server (done):** [`backend/config_server`](backend/config_server/) — JSON whitelist load/reload, in-memory service registry, socket registry probe, HTTP `/health` for monitors; port `8090`.

**Webhook Server (done):** [`backend/webhook_server`](backend/webhook_server/) — partner shop HTTP callback + S2S socket ingest, Corp MasterSigned endpoint CRUD, retry worker; settles via Client Center; port `8084` / `9004`.

**Message Center (done v1):** [`backend/message_center`](backend/message_center/) — drain `outbound_notices` → `notifications`, partner push, Session inbox; port `8085` / socket `9005`.

## Phase 2 - Client Relative (Register First) — **IN PROGRESS**

### 5. Minimal client backend path
- [x] Session Token Server public surface for:
  - **Public** register / login → session token
  - Partner **OAuth 2.0** start/callback; platform OAuth authorize/token/userinfo
  - session validate / revoke (Internal / Bearer)
- [x] Client Center login API surface (`8083`): register / login / logout / profile; Partner OAuth start/complete; Platform OAuth entry → STS.
- [x] Persist client accounts, OAuth links, user sessions, and game-account binding in PostgreSQL when `POSTGRES_URL` is set (Argon2id `password_hash`, `fx_user.oauth_identities`, OAuth columns on `user_sessions`; CC reads `fx_game.games` / `game_accounts` directly). In-memory fallback if URL unset.
- [ ] Return session token that Client Web uses on Gateway for all **Session** APIs after login/OAuth (Gateway not started).

### 6. Client Web - auth pages
- [x] Scaffold Register/Login pages (**Public**) on Client Web (GetX + envied).
- [x] Wire Client Web login/register to live **Client Center** (`ClientCenterAuthProvider` → `/v1/client/login|register`).
- [x] Partner **OAuth 2.0** login entry on Client Web (Path B: start → `/oauth/callback` with `sts_…`).
- [x] Document / expose Platform OAuth authorize URL for Partners (Path C; STS IdP via Client Center).
- [x] Bind user to an existing game / game account after register/OAuth:
  - game must exist (`GET /v1/client/games`); mapping key = `end_user_id` + `game_id`
  - **(A) `partner_fetch`**: Client Center calls Partner `/api/players/lookup` then writes mapping
  - **(B) `direct`**: Game App / Partner OAuth creates mapping (also on `/partner/complete`)
- [ ] Add authenticated shell: dashboard entry and basic profile/wallet view (**Session** token required on Gateway APIs).
- [x] Keep Flutter `.env` + `envied` setup for Client Web.
- Do not put Master credentials in Client Web; Company Partner → Client Center uses Master Account Code + Master ID + API Key + Secret.

### 7. Session and identity basics
- [x] Session Token Server: [`backend/session_token_server`](backend/session_token_server/) — issue / validate / revoke; **OAuth 2.0** platform IdP + Partner broker; demo login/register.
- [x] Client Center login orchestrates STS token issue; OAuth flows enter via CC and run on STS.
- [x] Session Token Server issues tokens on login or OAuth callback; Client Web must send that token on **Session** APIs after login/OAuth.
- [x] Cache sessions in Redis with expiry (or in-memory via `SESSION_STORE=memory`; Redis via `SESSION_STORE=redis`).
- [ ] Gate Token-required Client Web routes before e-shop checkout (`Public` only for register/login/OAuth entry).
- [x] Wire Client Center PostgreSQL as durable source of truth for accounts when `POSTGRES_URL` is set (in-memory fallback otherwise).

---

## Phase 3 - E-shop API and Frontend — **IN PROGRESS** (Client Center catalog/orders)

Goal: end users can buy **platform fixed** `PLT_*` packages **and** **Corp-created e-shop products** with partner **fiat** payment (HKD/USD); credit/fulfill after webhook. See `doc/e-shop.md` (C2 + C4).

### 8. E-shop backend API — **DONE** (Client Center Session APIs; webhook in step 9)
- [x] Expose system base fiat currency; list active platform `PLT_*` and Corp products with `fiat_price` (`GET /v1/shop/*` on Client Center).
- [x] Create pending `shop_orders` with `seller_type` `platform` or `corp` and fiat snapshot (`POST /v1/shop/orders`).
- [x] Call partner **fiat** payment create API (payment-gate `18100`); store `partner_order_no`.
- [x] Expose order status query / cancel for Client Web (`GET /v1/shop/orders`, `GET /v1/shop/orders/{id}`, `POST .../cancel`).
- [x] Keep shop orders outside the Core Engine matching path.
- [x] Webhook credit/fulfill (step 9); Client Web catalog UI still placeholder.

### 9. Partner payment + webhook settlement — **DONE** (Message Center drain in §15)
- [x] Accept shop **fiat** payment callbacks on Webhook Server (HTTP `POST /v1/webhook/shop/payment` + socket ingest `9004`).
- [x] Settle idempotently on `event_id` / `partner_order_no`; match `fiat_currency` / `fiat_paid` in Client Center `POST /v1/internal/shop/settle`.
- [x] On paid platform: credit Platform Token once and insert `shop_topup` txn (Postgres).
- [x] On paid corp: credit coin package; insert `corp_shop_purchase` txn (Postgres; item-only fulfill later).
- [x] On failed/cancelled: update shop order status without credit/fulfill.
- [x] Emit `outbound_notices` (`source_type=shop_order`) on settle; Client Web polls order status until Message Center / Gateway exist.
- [x] Retry failed settle (`retry_count`, backoff 5s/30s/2m/10m, then `dead`).
- [x] Append `deposit_withdrawal_status_logs` (`event_type=completed`) with money txn insert on paid settle; idempotent backfill on retry.

### 10. Client Web e-shop UI — **DONE** (wallet page later)
- [x] Catalog GridView: platform packages + Corp products with fiat prices in base currency.
- [x] Classic cart + checkout (game account) + payment **DataTable**; pending orders **24h** TTL in Client Center.
- [x] Order history / detail with status poll (pending / paid / failed / expired / cancelled).
- [ ] Show updated wallet / fulfillment after paid settlement (home wallet UI).

### 11. Admin + Corp product management — **DONE**
- [x] Admin: list / detail / suspend Corp e-shop products (`GET`/`PATCH /v1/admin/shop/corp-products*` on Admin API `18300`); Admin Panel `/corp-products`.
- [x] Admin: set system base fiat (`HKD`/`USD`); seed/activate fixed `PLT_*` and set **fiat_price** (`POST`/`PUT`/`PATCH /v1/admin/shop/packages*`, setup wizard + Admin Panel `/packages`).
- [x] Corp: create/update/activate Corp e-shop products with **fiat_price** (`/v1/corp/shop/products` on Client Center).

### 11b. Company Basic Token — **DONE** (API slice)
- [x] Corp MasterSigned: submit/list/update Company Basic Token (`/v1/corp/basic-tokens*` on Client Center).
- [x] Admin approve/reject (`/v1/admin/basic-tokens*`); approve → `buyable=true` + Corp partner notice.
- [x] Session buyable catalog + orders (`/v1/corp-tokens*`); after approve, tokens appear for End Users.
- [x] Activating Corp `company_coin_package` e-shop products requires an approved CBT for that `credit_game_coin_id`.
- [x] Webhook `POST /v1/webhook/corp-token/payment` → CC `/v1/internal/corp-token/settle` (credit 99.9%, fee 0.1%); outbound notice to End User; Corp notice + HTTP balance callback to `webhook_endpoints` kind `corp_token` or `partner_endpoints` type `callback`.

---

## Phase 4 - Suggested Platform Expansion — **PARTIAL** (Core Engine ahead of schedule)

After the e-shop vertical slice works, expand the rest of the platform in this order.

### 12. Shared contracts and Config Server — **DONE**
- [x] Shared Rust common workspace under `common/`: `funnyx-error`, `funnyx-types`, `funnyx-time`, `funnyx-config`, `funnyx-health`, `funnyx-heartbeat`, `funnyx-socket-msg`, `funnyx-socket`, `funnyx-net-api`, `funnyx-auth`, `funnyx-prelude`.
- [x] Config Server: [`backend/config_server`](backend/config_server/) — JSON whitelist load/reload, in-memory service registry, **private** socket registry probe; public HTTP `/health`; port `8090`.
- [x] Project skeletons and `.env` / `env.sample` for remaining services (`common/env.sample` for shared keys).
- [x] Dual common libs: `funnyx-heartbeat` (**private** socket PING/PONG + register/probe); `funnyx-health` (**public** `GET /health` for Cloudflare / Game Partners). Services (e.g. CC) apply both. Wired CC `9001` / STS `9002` / Admin `9003` / Webhook `9004`.

### 13. Gateway
- Public HTTP/socket ingress.
- Health-aware routing to Client Center and later Core Engine.
- In-memory route map, service registry, connection pool, failover.

### 14. Client Center hardening
- Full authenticated API/socket session model.
- Account validation, wallet reads, deposit/withdrawal request path.
- Forward exchange orders later; keep e-shop settlement local to Client Center.
- Provision **required dedicated MongoDB** for client chat message persistence (`chat_messages`).

### 15. Message Center — **DONE** (v1; Gateway fan-out later)
- [x] Crate [`backend/message_center`](backend/message_center/) — HTTP `8085`, heartbeat socket `9005`; doc [`doc/message_center.md`](doc/message_center.md).
- [x] Drain PostgreSQL `outbound_notices` where `delivery_status=pending` (SKIP LOCKED); memory fallback without `POSTGRES_URL`.
- [x] On send success: write user `notifications`, mark outbound `sent`; optional Mongo `notice_history` when `MONGO_URL` set.
- [x] Event coverage: DDL enums (`created` / `pending_payment` / `completed` / `cancelled` / `rejected` / `filled` / …).
- [x] Session inbox `GET`/`PATCH /v1/notifications`; Internal `POST/GET /v1/notices*` (Core Engine compatible); public `/health` + private socket heartbeat.
- [x] Partner HTTP POST (callback resolve + `webhook_events` delivery log). Gateway socket push to Client Web deferred (§13).
- [ ] Redis claim assist (optional; Postgres SKIP LOCKED is sufficient for v1).

### 16. Admin API / Admin Panel beyond packages — **IN PROGRESS** (auth + setup + packages/corp UI done)
- [x] Admin API auth: [`backend/admin_api`](backend/admin_api/) login/logout/me; STS `admin_login` / `actor_type=admin`.
- [x] Admin Panel login wired to Admin API (`AdminApiAuthProvider`).
- [x] Admin Corp e-shop product monitor/suspend (`/v1/admin/shop/corp-products*`) + Panel `/corp-products`.
- [x] **System setup wizard** + PLT packages CRUD/status (`POST`/`PUT`/`PATCH /v1/admin/shop/packages*`); Panel `/packages`.
- [ ] Role-based admin access beyond demo directory; persist `fx_admin`.
- [ ] Game Partner Game Coin lifecycle, corporate onboarding, market approval UI.
- [ ] Connect Admin API to Config Server.

### 17. Market onboarding and liquidity lock-up — **IN PROGRESS** (Corp submit + pending pool)
- [x] Corp submit market pair + lock client Game Partner Game Coin + pending `market_pools` (`POST /v1/corp/markets` on Client Center).
- [ ] Admin review/approve; transfer lock into pool wallets; Core Engine activate.
- Corporate User game registration → ready state → Game Account ID.
- Initial Game Partner Game Coin supply and balance account.
- Market flow: Client Submitted → Under Pending (lock required pool on client Game Partner Game Coin balance) → Wait Admin User Review → Confirm approval by Admin User → Transfer Client Balance to Pool → Core Engine activate.
- **C6 / C9 Corp fees:** Year-1 **80K** / renewal **10K** / extra pair **10K** (Admin fiat or USDT). Admin: license deadline + partner notices + disable APIs if unpaid.
- First market must pair with Platform Token.
- Activate market only after admin approval and successful transfer of locked client balance into the pool.

### 18. Core Engine and exchange order flow — **DONE** (local matching; Client Center path open)
- [x] Skeleton under [`backend/core_engine`](backend/core_engine/): quote-asset shard (max **10** pairs/engine), Market (01) / Price (02), price-time book.
- [x] Publish fills/cancels to Message Center + partner server group (`MESSAGE_CENTER_URL`, `PARTNER_NOTICE_URL`).
- [x] Info APIs: `GET /v1/engine`, `GET /v1/engine/pairs`, books/trades/notices; order place/cancel.
- [x] Maintenance: local snapshot persist; admin save / stop / start (`/v1/admin/maintenance*`).
- [ ] Wire Client Center → Core Engine private path; ring-buffer worker hardening; balance settlement via Client Center.
- In-memory order book; Market Order (01) and Price Order (02).
- Price-time priority matching; cancel and partial fill.

### 18b. Marketplace deals (C7)
- Gate **place deal**: (1) user bought Platform Token on platform, (2) playing ≥1 listed game, (3) partner Transfer API open for the asset’s game, (4) for game_item — partner Item List API returns pickable `item_ref_id`.
- Persist `offer_item_ref_id` / transfer `item_ref_id` (`VARCHAR`); no platform item escrow table.
- Require partner endpoint `type=transfer` for Marketplace-eligible games.
- Client APIs: eligibility, item list proxy, list/create/request/cancel deals. See `doc/marketplace.md`.
- Emit `outbound_notices` on deal status changes.

### 19. Deposit / withdrawal / transfer partner flow
- Reuse partner endpoints for game deposit/withdrawal and Marketplace Transfer.
- Validate identity, balance, and game-account mapping.
- Record transaction status (`deposit_withdrawal_status_logs`) and notify users (`outbound_notices`).

---

## Phase 5 - Reliability, Local Env, and Hardening — **NOT STARTED**

### 20. Health and heartbeat — **PARTIAL** (common libs + registry probe)
- [x] **Private** socket PING/PONG (`funnyx-heartbeat`) — in-system; Config Server probes `socket_url`.
- [x] **Public** HTTP GET `/health` (`funnyx-health`) — third parties (Cloudflare, Game Partners); not registry heartbeat.
- [ ] Gateway degraded-mode / failover from private heartbeat state.

### 21. Docker local development environment
- Dedicated Docker networks per service group (see `technology.md`).
- Startup order: DB stores → Config → Session/Client → Gateway → Webhook → Message → Admin → Core Engine.

### 22. Auto order matching bot
- [x] Local bot for exchange pressure testing after Core Engine exists (`dev_simulator/market-bot`).
- [ ] Random timing through Gateway → Client Center → Core Engine (direct engine poll for now).

### 23. Security and performance
- Finalize IP allowlist, signed/session request checks.
- Tune ring-buffer capacity, queue depth metrics, health timeouts.

### 24. Final integration testing
- Client register/login.
- E-shop purchase, partner callback, single wallet credit; duplicate webhook does not double-credit.
- Deposit/withdrawal partner path.
- Market approval and exchange order lifecycle.
- Marketplace place-deal gates (Platform Token buy + listed game + Transfer API + Item List for game_item).
- Health, failover, and Docker local environment.

---

## Why this order

| Priority | Focus | Reason |
| --- | --- | --- |
| 1 | Database + local simulators | Stable schema/seeds plus `dev_simulator/` helpers so later APIs can be smoke-tested without live partners |
| 2 | Client register/login/OAuth | **STS + Client Center login/OAuth done**; finish Client Web ↔ CC and Postgres; Partner OAuth + Transfer (+ Item List for Marketplace); Master credentials for Corp APIs |
| 3 | E-shop API + frontend | Delivers the first complete user value path (buy Platform Token) |
| 4+ | Gateway, admin, markets, Core Engine | Core Engine local matching already usable; exchange via Gateway/Client Center once top-up path is proven |

This schedule keeps early delivery aligned with the e-shop product slice while leaving exchange matching and market onboarding as the next major build after wallet top-up works.
