# Changelog

This is the single project changelog for the frontend work. Older frontend changelog files were merged here to avoid duplicate release notes.

## 0.1.21

- New **Message Center** (`backend/message_center`, HTTP `8085` / socket `9005`): drains `fx_events.outbound_notices`, writes `notifications`, partner HTTP push + delivery log, optional Mongo history; Session inbox APIs; dual public `/health` + private socket heartbeat.
- Doc [`doc/message_center.md`](doc/message_center.md); Core Engine can point `MESSAGE_CENTER_URL` at `8085` (sim `18101` remains).

## 0.1.20

- Split audiences: `funnyx-heartbeat` = **private** in-system socket PING/PONG + Config probe; `funnyx-health` = **public** `GET /health` for Cloudflare / Game Partners.
- Config Server registry liveness is socket-only (no longer probes peer `/health`).
- Services apply both; CC/STS/Admin heartbeat sockets `9001`/`9002`/`9003`; Webhook `9004`.

## 0.1.19

- New common crate `funnyx-heartbeat`: socket PING/PONG helpers, Config Server registry register+heartbeat client, staleness helpers.
- Wired registry heartbeat into Client Center, Admin API, Webhook Server, Session Token Server; Webhook socket uses shared PONG helper; Config Server device-status uses shared `is_stale`.

## 0.1.18

- Admin API: `POST /v1/admin/shop/packages` (seed `PLT_*`), `PATCH /v1/admin/shop/packages/{id}/status`.
- Admin Panel: PLT packages page (`/packages`) and Corp products monitor/suspend (`/corp-products`); finishes development.md §11.

## 0.1.17

- Client Center shop settle writes `fx_money.deposit_withdrawal_status_logs` with each `shop_topup` / `corp_shop_purchase` money txn (`event_type=completed`); idempotent backfill on paid settle retry (finishes development.md §9).

## 0.1.16

- Admin Panel **system setup wizard**: after login, incomplete installs must set `base_fiat_currency` (HKD/USD) and PLT package prices, then confirm (`system_initialized`).
- Admin API: `GET /v1/admin/system/setup`, `POST /v1/admin/system/setup/complete`, `GET`/`PUT /v1/admin/system/base-currency`, `GET`/`PUT /v1/admin/shop/packages*`.

## 0.1.15

- Company Basic Token APIs: Corp submit/list/update (`/v1/corp/basic-tokens*`); Admin approve/reject (`/v1/admin/basic-tokens*`); Session buyable list/orders (`/v1/corp-tokens*`).
- Webhook `POST /v1/webhook/corp-token/payment` settles via Client Center `/v1/internal/corp-token/settle` (99.9% credit / 0.1% fee), End User `outbound_notices`, Corp partner notice, and HTTP balance callback to Corp User server.
- Corp `company_coin_package` activation gated on Admin-approved Company Basic Token for that coin.

## 0.1.14

- Client Web e-shop: GridView catalog, classic cart, checkout with game-account binding, payment/orders DataTables; pending shop orders expire after **24 hours**.
- `funnyx_common`: `ShopDataProvider` / models for Client Center `/v1/shop/*` and game-account bindings.

## 0.1.13

- Client Center Corp MasterSigned APIs: create/update/activate e-shop products (`/v1/corp/shop/products*`); submit market pair, lock Game Partner Game Coin, and create a pending trade pool (`POST /v1/corp/markets`, `POST /v1/corp/markets/{id}/pool`).

## 0.1.12

- Admin API Corp e-shop product monitor/suspend: `GET /v1/admin/shop/corp-products`, `GET .../{id}`, `PATCH .../{id}/status` (`active` / `inactive` / `archived`).

## 0.1.11

- Webhook Server (`backend/webhook_server`, HTTP `8084` / socket `9004`): partner shop payment ingest (HTTP + S2S), Corp MasterSigned endpoint CRUD, retry settle into Client Center; `fx_events.webhook_events` retry columns + `webhook_endpoints`.
- Client Center `POST /v1/internal/shop/settle` (`X-Internal-Key`): paid/failed/cancelled shop orders, wallet credit, `shop_topup` / `corp_shop_purchase`; default `SHOP_PAYMENT_CALLBACK_URL` is `http://127.0.0.1:8084/v1/webhook/shop/payment`.

## 0.1.10

- Client Center e-shop Session APIs: base currency, PLT packages, Corp products, create/list/cancel orders (fiat snapshot + payment-gate checkout).

## 0.1.9

- Client Center identity: Argon2id end-user passwords; `fx_user.oauth_identities`; `user_sessions` OAuth grant fields; direct Postgres read/write for `fx_game.games` and `game_accounts` when `POSTGRES_URL` is set.

## 0.1.8

- Config Server (`backend/config_server`, port `8090`): JSON whitelist load/reload, in-memory service registry, HTTP heartbeat, device-status.

## 0.1.7

- Client Center game-account bind: catalog + mapping (`partner_fetch` via company-a `/api/players/lookup`, `direct` for Game App/OAuth); auto-bind on Partner OAuth complete.

## 0.1.6

- Client Web OAuth: Path B Partner start/callback (`OauthFlowService`, `/oauth/callback`); Path C platform authorize URL for Partners; login/register compare panel.

## 0.1.5

- Admin API (`backend/admin_api`, port `18300`): login/logout/me; Session Token Server issues admin sessions (`grant_type=admin_login`, `actor_type=admin`). Admin Panel wired via `AdminApiAuthProvider`.

## 0.1.4

- Client Web register/login call Client Center (`ClientCenterAuthProvider` → `/v1/client/login|register|logout`); `API_BASE_URL` defaults to `8083`.

## 0.1.3

- Client Center (`backend/client_center`, port `8083`): password login/register/logout/profile; Partner OAuth start/complete and Platform OAuth entry via Session Token Server.

## 0.1.2

- Session Token Server: OAuth 2.0 platform IdP (`/v1/oauth/*`) and Partner broker (`/v1/client/oauth/partner/*`), plus password login/register for Client Web sessions.

## 0.1.1

- Replace Riverpod / go_router with GetX controllers, bindings, and `GetMaterialApp` routes.
- Switch state handling to GetX (`SessionController`, `AuthMiddleware`); remove Riverpod.
- Shared token store, Dio JWT client, mock auth provider, theme, and responsive layout updates.
- Update app-level routing and shell structure for the admin panel and client web experience.

## 0.1.0

- Flutter Web scaffold with `funnyx_common`, envied, login, and admin shell.
- Flutter Web scaffold with `funnyx_common`, envied, login/register, and client shell.
- Shared frontend foundation and project structure for both apps.
