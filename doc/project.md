<!-- Master Index Page for others md file which is under doc folder -->

# FunnyX Platform Architecture (Master Index)

This file is the index and roundup page for the architecture docs under `doc/`. The project’s single source of truth remains [readme.md](../readme.md). This file summarizes the architecture and links the detail docs; keep summaries here and put procedures, payloads, and deeper design in the documents below.

API index: [api-master.md](../api-master.md). Schedule: [development.md](../development.md). Stack: [technology.md](../technology.md). DB DDL: [database/](../database/). ERD drafts: [db_config/](../db_config/).

## Document map

| Master topic (this file) | Detail document |
| --- | --- |
| Auth / Client Web vs Company Partner / OAuth 2.0 | [client_connect.md](client_connect.md) |
| Corp onboarding, OAuth + Transfer APIs, fees (C6/C9), deposit/withdraw | [partner.md](partner.md) |
| Partner setup / OAuth / Transfer / Item List UML | [partner-uml.md](partner-uml.md) |
| E-shop catalog, fiat pay, settlement | [e-shop.md](e-shop.md) |
| Webhook Server ingress, socket targets, which API to call | [webhook.md](webhook.md) |
| Marketplace place-deal gates (C7) | [marketplace.md](marketplace.md) |
| End-to-end diagrams | [flow_chart.md](flow_chart.md) |
| Gateway LB + Core Engine matching | [gateway_core_engine_logic.md](gateway_core_engine_logic.md) |
| Socket byte protocols | [socket_message.md](socket_message.md) |
| System constants / enums | [system_type.md](system_type.md) |
| Dev simulators / local helpers | [dev.md](dev.md) |

---

## 1. Overview

FunnyX is a high-performance, Rust-first trading and messaging platform for a **cross-game platform token and partner-token ecosystem**.

- Corporate Users onboard games, company token flows, and market pairs.
- Pending markets **lock** required pool amount on the **client Game Partner Game Coin balance**; after Admin approval, balance is **transferred into the market pool**.
- End users trade, use e-shop, and place Marketplace deals through Client Web.
- Timestamps: **UTC+0**, stored as **BIGINT**.

---

## 2. Core technology

| Area | Master rule |
| --- | --- |
| Language | Rust for server infrastructure |
| S2S comms | Binary sockets, byte-buffer payloads |
| Config | Every Rust/Flutter project: `.env` + `env.sample`; Flutter uses `envied` |
| Detail | [technology.md](../technology.md), [socket_message.md](socket_message.md) |

---

## 3. System storage

No single shared Redis/DB for the whole platform. Each service group owns its stores:

| Store | Role |
| --- | --- |
| Redis | Session / hot cache (per group) |
| PostgreSQL | Transactional business data (per group) |
| MongoDB | Chat / notices / document history where required (Client Center chat **required**) |
| Local (SQLite/LevelDB) | Admin / Webhook self-hosted state |

Config Server loads operational JSON (whitelist, routes, heartbeat). DDL: [database/](../database/). ERD/query drafts: [db_config/](../db_config/).

---

## 4. Platform business flow (master)

1. Game company registers as Corporate User.
2. Admin approves; game ready + Game Account ID.
3. Initial Game Partner Game Coin supply → game balance account.
4. **Corp fees (C6/C9):** Year-1 **80,000** / renew **10,000** / extra pair **10,000** follow the Admin-configured base policy (`HKD`/`USD` or `USDT` as approved). Partner “money movement” = first-time or renew fee. Admin may set deadline, notice partner, disable APIs. → [partner.md](partner.md)
5. Market: Client Submitted → Under Pending (lock Game Partner Game Coin) → Admin review → Transfer to Pool. First market pairs with **Platform Token**. → [flow_chart.md](flow_chart.md), [partner.md](partner.md)
6. Partner APIs: **OAuth 2.0** (required) + **Transfer** (required); **Item / game-assets list** for Marketplace game items; balance/deposit/withdraw only if partner uses their own APIs. → [partner.md](partner.md), [client_connect.md](client_connect.md)
7. **E-shop (C2/C4):** Platform `PLT_*` + Corp products; platform-token purchases follow the Admin base currency, while company-owned token settlement follows the owner company’s configuration; credit/fulfill after webhook. → [e-shop.md](e-shop.md)
8. Company Basic Token: Corp submit → Admin approve → buy under the company’s configured settlement rules; **0.1%** platform fee remains under the Admin-defined policy. → [partner.md](partner.md), [e-shop.md](e-shop.md)
9. **Marketplace (C7):** place deal only if verified (bought Platform Token) + playing ≥1 listed game + partner Transfer open; game items need Item List API → opaque `item_ref_id`. → [marketplace.md](marketplace.md)

---

## 5. Server types (summary)

| Service | Role (master) | Detail |
| --- | --- | --- |
| Config Server | Whitelist JSON, registry, heartbeat, single instance | [system_type.md](system_type.md) |
| Admin Panel | Flutter Web admin UI, IP whitelist | — |
| Admin API | Control plane, RBAC, markets/Game Partner Game Coin/fees/API disable | [partner.md](partner.md), [api-master.md](../api-master.md) |
| Webhook Server | External callbacks (e-shop fiat, deposits); idempotent settle | [webhook.md](webhook.md), [e-shop.md](e-shop.md) |
| Message Center | Notices, partner push, outbox drain | [message_center.md](message_center.md), [socket_message.md](socket_message.md) |
| Client Center | Accounts, orders, e-shop, chat MongoDB; orchestrates Core Engine | [client_connect.md](client_connect.md) |
| Gateway | Public HTTP/socket ingress, LB, in-memory route map | [gateway_core_engine_logic.md](gateway_core_engine_logic.md) |
| Core Engine | Private matching; Market / Price orders only | [gateway_core_engine_logic.md](gateway_core_engine_logic.md) |
| Client Web | Flutter end-user UI (trade, e-shop, marketplace) | [client_connect.md](client_connect.md), [e-shop.md](e-shop.md), [marketplace.md](marketplace.md) |
| Session Token Server | Issues/validates Client Web tokens | [client_connect.md](client_connect.md) |

### 5.1 Auth master (C8 + OAuth 2.0)

| Caller | Credential |
| --- | --- |
| Client Web | Token from Session Token Server on **all** APIs after Public login/register/**OAuth** |
| Company Partner → Client Center | **Master Account Code + Master ID + API Key + Secret** |

**OAuth 2.0 (readme):** Partner OAuth verifies users on the platform; platform OAuth lets Partners login platform-registered users. Required partner APIs: OAuth + Transfer (always); Item List for Marketplace game items; balance + deposit/withdraw only if partner does not use platform wallet APIs.

Detail: [client_connect.md](client_connect.md), [partner.md](partner.md) §0. Gateway: [gateway_core_engine_logic.md](gateway_core_engine_logic.md).

### 5.2 Client Web pages (master list)

Register/Login/OAuth (Public) · Company News · Dashboard · Market List · Trade Panel · Marketplace Deals (C7) · Notices · E-shop catalog/checkout/status · Company Basic Token list/buy.

---

## 6. Server group structure

| Group | Service | Deploy | Storage notes |
| --- | --- | --- | --- |
| 1 | Config Server | Single | In-memory registry |
| 2 | Admin Panel | Single | — |
| 3.1 | Admin API | Single | Admin Redis + SQLite/LevelDB |
| 3.2 | Webhook Server | Single | Local SQLite/LevelDB |
| 4 | Message Center | 1..N | Redis + PostgreSQL + MongoDB |
| 5 | Client Center | 1..N | Redis + PostgreSQL + **MongoDB (chat required)** |
| 6.1 | Gateway | 1..N (min 2) | In-memory map |
| 6.2 | Core Engine | 1..N | Matching store as needed |
| 7 | Client Web | Single | — |
| 8 | Session Token Server | Single | Dedicated Token Redis |

Constants: [system_type.md](system_type.md).

---

## 7. Inter-service communication

Server groups use socket byte buffers (lz4 where specified). Protocols and layouts: [socket_message.md](socket_message.md). Message type / server type enums: [system_type.md](system_type.md).

---

## 8. Common library and environment

Shared Rust workspace under [`common/`](../common/):

| Crate | Role |
| --- | --- |
| `funnyx-error` | Shared `Error` / `Result` |
| `funnyx-types` | System enums ([system_type.md](system_type.md)) |
| `funnyx-time` | UTC+0 BIGINT timestamps |
| `funnyx-config` | Platform constants + `GlobalConfig`; missing `.env` keys use built-in defaults |
| `funnyx-health` | **Public** HTTP `GET /health` (Cloudflare, Game Partners, external LB) |
| `funnyx-heartbeat` | **Private** socket PING/PONG + registry register/probe (in-system only) |
| `funnyx-socket-msg` | Socket message layouts ([socket_message.md](socket_message.md)) |
| `funnyx-socket` | Socket transport (lz4 / gzip / PING/PONG) |
| `funnyx-net-api` | HTTP envelopes + `/v1` paths ([api-master.md](../api-master.md)) |
| `funnyx-auth` | Session / MasterSigned / OAuth credential helpers ([client_connect.md](client_connect.md)) |
| `funnyx-prelude` | Re-export facade |

Every project: `.env` + `env.sample` (hosts, DB/Redis, Config Server, heartbeat, logs, upload paths, runtime mode). Sample keys: [`common/env.sample`](../common/env.sample). Flutter: access via `envied` only.

---

## 9. Client integration

Web entry + Partner APIs. Auth and calling styles: [client_connect.md](client_connect.md). Partner contracts: [partner.md](partner.md).

---

## 10. Admin, markets, token model (master)

**Market enablement (after Corp submit):**

1. Token definitions exist (Platform Token `PLT` + Company Basic Token + Game Coin).
2. Pending lock on client Game Partner Game Coin balance; C6 fee valid; first pair includes Platform Token `PLT`.
3. Admin approve → transfer lock to pool → register Core Engine → activate.
4. Reject → unlock and revise.

**Token model (C5):** Company may use `token` / `crypto_token` / `stablecoin` as Company Basic Token or Game Coin; platform does not block. The Admin-configured base currency (`HKD` or `USD`) applies to platform e-shop pricing and platform-fee configuration, while company-owned token settlement follows the owning company’s configuration.

Detail: [partner.md](partner.md), [flow_chart.md](flow_chart.md), [gateway_core_engine_logic.md](gateway_core_engine_logic.md). Dev bot: [dev.md](dev.md).

---

## 11. Operations (master)

- Config Server = operational truth; Gateway keeps a **local in-memory** snapshot for routing (detail tables in [gateway_core_engine_logic.md](gateway_core_engine_logic.md)).
- All services apply **both**: private socket PING/PONG (`funnyx-heartbeat`) for in-system status; public HTTP `GET /health` (`funnyx-health`) for Cloudflare / Game Partners.
- Scale Message Center, Client Center, Core Engine horizontally; Gateway ≥ 2 instances.

---

## 12. Security (master)

- Gateway public ingress.
- Client Web: Token Server token after login or OAuth 2.0.
- Company Partner: Master Account Code + Master ID + API Key + Secret; Admin may disable APIs on unpaid C6 fees (C9).
- Partner join: OAuth 2.0 + Transfer required; Item List for Marketplace game items ([readme.md](../readme.md)).
- Admin: IP allowlist + RBAC.
- Internal: binary sockets + validated config.

Detail: [client_connect.md](client_connect.md), [partner.md](partner.md) §7.2.

---

## 13. Data integrity

UTC+0 BIGINT timestamps for trades, audits, sessions, messages, and config sync.

---

## 14. Engineering principles

1. High performance (binary sockets, compact encoding).
2. Low latency (buffers / ring buffers).
3. Service isolation (gateway, engine, client, admin, message).
4. Shared Rust common workspace (`common/` `funnyx-*` crates).
5. Health, heartbeat, failover-ready deploy.
6. Central config + RBAC admin.

---

## Related roots (outside `doc/`)

| File | Role |
| --- | --- |
| [readme.md](../readme.md) | Product motivation / disclaimer (READ-ONLY for AI) |
| [api-master.md](../api-master.md) | HTTP API tables |
| [development.md](../development.md) | Build schedule |
| [technology.md](../technology.md) | Stack and Docker nets |
| [database/](../database/) | PostgreSQL DDL (12 `fx_*` schemas, 43 tables) |
| [db_config/](../db_config/) | ERD and SQL drafts |
