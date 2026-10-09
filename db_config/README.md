# Database Configuration

This folder contains database configuration and query definitions for the FunnyX platform.

## Purpose

- centralize SQL and database-related configuration
- keep database access definitions separated from application logic
- support system services that rely on PostgreSQL, Redis, MongoDB, and local storage
- reflect the project core of cross-game Game Partner Game Coin onboarding, corporate game-company management, market creation, deposit/withdrawal flows, and Platform Token e-shop top-up

## Domain alignment

The database model should support the platform lifecycle described in the project overview:

- Corporate User registration and onboarding
- Partner **OAuth 2.0** + Transfer endpoints (required); Item / game-assets list API for Marketplace game items; balance/deposit/withdraw endpoints when partner uses their own APIs
- Marketplace deals (`fx_marketplace`): coins + game items via opaque `offer_item_ref_id`; settle via Transfer; notices via `outbound_notices`
- Message Center outbox: `fx_events.outbound_notices`; money status audit: `deposit_withdrawal_status_logs`
- Game information registration and approval
- Game Account ID creation and game balance account management
- Game Partner Game Coin supply initialization
- market pair: Client Submitted → Under Pending (lock required amount on client balance) → Admin review/approve → Transfer Client Balance to Pool
- **C6 / C9 Corp fees:** Year-1 **80K** / renewal **10K** / pair **10K** (Admin fiat or USDT); `api_enabled` + `license_deadline_at` + `corp_partner_notices`
- market pool lock: required amount on client Game Partner Game Coin balance
- first market must pair with Platform Token
- deposit and withdrawal requests for users and partner endpoints (amount + game_coin code; coin may be token, crypto token, or stablecoin)
- e-shop fixed Platform Token packages (credit amounts only): `PLT_1000`/`1500`/`3000`/`10000`
- e-shop listed items paid in Admin base fiat HKD/USD; webhook then credits Platform Token / Game Coin / company token as applicable
- wallet credit for Platform Token only after successful shop settlement (`shop_topup` required)
- Company Basic Token: Corp submit → Admin approve → user buyable; 0.1% Company Basic Token buy fee (`coin_fee_ledger`)
- audit trail for market status, transaction status, shop payment events, fee payment, and approval actions
- product master: repository `readme.md`

## Suggested structure

- **[`../database/`](../database/)** — runnable PostgreSQL DDL (**12** `fx_*` schemas, **43** tables; one folder per schema)
- redis/ / mongodb/ / sqlite/ — reserved for other stores
- queries/ — Redis/Mongo design notes (PostgreSQL DDL lives in [`../database/`](../database/))

## Notes

- **ERD source of truth:** [database_relationships.md](database_relationships.md) — includes `corporate_users`, `games`, `market_pools`, `platform_fees`, balance locks, and e-shop tables.
- **Runnable DDL:** [`../database/`](../database/) (`run_all.sql`) — authoritative PostgreSQL schema.
- **Counts:** 12 PostgreSQL schemas (`fx_corp`, `fx_game`, `fx_user`, `fx_market`, `fx_market_data`, `fx_marketplace`, `fx_config`, `fx_shop`, `fx_corp_token`, `fx_money`, `fx_admin`, `fx_events`) and **43** tables.
- Use UTC timestamps stored as BIGINT for all time-based records.
- Keep database access patterns consistent across services.
- Prefer explicit query modules over inline SQL scattered across the codebase.
- Redis, PostgreSQL, and MongoDB are not treated as a single shared system-wide database. Each server group owns its own dedicated storage instance when required by the service workload.
- The data store should be managed at the service-group level, so cache and persistent storage scale independently and do not create cross-service contention.
- Platform data should be modelled around Corporate User, Game, Game Partner Game Coin, Market Pair, Market Balance Lock, Market Pool, Platform Fee, Transaction, Shop Package, and Shop Order entities.
- E-shop shop_orders are not exchange orders; they never enter the Core Engine matching path.
- **C2 + C4:** Platform fixed `PLT_*` (`shop_packages`) **and** Corp products (`corp_shop_products`); all e-shop list prices in Admin `base_fiat_currency` (**HKD**|**USD**); orders snapshot fiat + `seller_type` platform|corp.
