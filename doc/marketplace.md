# Marketplace

This doc is a detail file for the master architecture in [project.md](project.md). See the product summary in [readme.md](../readme.md) and the API index in [api-master.md](../api-master.md).

**Detail scope:** C7 place-deal gates, deal actions, 1 PLT listing fee, take/settle via partner Transfer + Item List APIs, Message Center notices + partner webhook, and Marketplace auth pointers — not e-shop fiat or Corp C6 fees.

Master gate summary: [project.md](project.md) §4 step 9. **C7** rules below are authoritative for who may **place** a deal.

- Users may sell value into **Platform Token**, or into **Company Coin** (company-owned token / Company Basic Token asset on that corp) — but only the **parent company’s** Company Coin, never another company’s coin.
- Supported deal assets: **Game Coin**, **Company Coin**, and **Game Items**.
- Deal settlement moves game assets via the Company Partner **Transfer** API (token/asset movement; not e-shop fiat).
- Game items: partner **Item / game-assets list API** (readme #5) returns opaque `item_ref_id` values; platform stores `offer_item_ref_id` / transfer `item_ref_id` (`VARCHAR`). No platform item catalog or escrow table (`database/fx_marketplace`).

## 1. Right to place a deal (C7)

A normal user may **place / post a deal** only when **all** of the following are true:

| # | Requirement | Meaning |
| --- | --- | --- |
| 1 | **Verified** | User has bought **Platform Token** on the platform (e-shop Platform Token package purchase settled / credited), or holds PLT wallet balance. |
| 2 | **Playing a listed game** | User is bound to / actively playing **at least one** game listed on the platform. |
| 3 | **Partner Transfer API open** | The deal’s game assets are only accepted when that game’s Company Partner has enabled the **Transfer** API for those assets (Game Coin / Company Coin / Game Items). Partners must expose this API to join the platform for marketplace deals. |
| 4 | **Item list when offering game_item** | For `offer_asset_type = game_item`, partner Item List API must return a pickable `item_ref_id` for the poster’s game account. |

If any gate fails, the user must **not** create a new listing. They may still browse; **take a deal** remains available when eligible for that game.

**Listing fee:** placing a deal debits **1 PLT** from the poster’s platform wallet (game-account wallet row). Cancel does **not** refund the fee in v1.

## 2. Deal actions

| Action | Who | Meaning |
| --- | --- | --- |
| Place / post a deal | Normal user meeting §1 | Create an **open** listing (game, offer, want price); pay 1 PLT fee |
| Take a deal | Normal user (Session) | `POST .../request` with own game account → immediate settle |
| Browse | Normal user (Session) | View open deals |
| Cancel | Poster only | Cancel while status is `open` |

### Take / settle flow

1. User B takes an open deal (same `game_id`, not self-take).
2. Platform debits B’s wallet for `want_amount` of `want_game_coin`, credits poster A.
3. Platform calls partner **Transfer** to move offer assets A → B.
4. On success: deal `completed`, request `completed`, transfer event `succeeded`.
5. On Transfer failure: reverse wallet legs, deal/request `failed`.

Want side is always **platform wallets**. Partner Transfer moves **offer** assets only.

Persistence: `fx_marketplace.marketplace_deals`, `marketplace_deal_requests`, `marketplace_transfer_events`.

## 3. Asset acceptance (partner Transfer + Item List)

- Marketplace deals for a game are allowed only if that partner’s **transfer** endpoint is registered, approved, and enabled (`PARTNER_ENDPOINT.type = transfer`).
- For game items, Client Web lists assets via partner Item List API (proxied by `GET /v1/marketplace/items`), user picks an item, platform stores opaque `offer_item_ref_id`.
- On deal settle / fulfill, platform calls the partner Transfer API (same `item_ref_id` for items) to move Game Items and/or coins.
- Without Transfer API: asset type is not deal-eligible (listing create rejected). Without Item List: game_item listings are rejected.
- Want side: Platform Token or **parent** Company Coin only (app check against deal `corporate_user_id`).

See required partner APIs in [partner.md](partner.md) §0, §1.4, §1.5.

## 4. Notices + partner webhook

After place / take / cancel / fail, Client Center enqueues Message Center notices (`POST /v1/notices`, fallback insert into `fx_events.outbound_notices`):

| Deal action | Recipients | `source_type` | `event_type` |
| --- | --- | --- | --- |
| Place | Poster | `marketplace_deal` | `created` |
| Take success | Poster + Taker | `marketplace_deal` | `completed` |
| Take Transfer fail | Poster + Taker | `marketplace_deal` | `failed` |
| Cancel | Poster | `marketplace_deal` | `cancelled` |

Payload includes `corporate_user_id`, `deal_id`, `game_id`. Message Center drain writes user `notifications`, POSTs partner webhook (`webhook_endpoints.kind = marketplace`), and logs `webhook_events`. See [message_center.md](message_center.md).

## 5. Auth

Client Web: Token Server token on all Marketplace APIs (after login/OAuth). Company Partner Transfer / Item List / Client Center: Master Account Code + Master ID + API Key + Secret. Partner join also requires OAuth 2.0. See [client_connect.md](client_connect.md), [partner.md](partner.md) §0.

## 6. APIs (Session via Gateway → Client Center)

| Method | Path | Purpose |
| --- | --- | --- |
| GET | `/v1/marketplace/eligibility` | C7 gates + listing fee |
| GET | `/v1/marketplace/items?game_account_id=` | Proxy partner Item List |
| GET | `/v1/marketplace/deals` | List open deals |
| GET | `/v1/marketplace/deals/{id}` | Deal detail |
| POST | `/v1/marketplace/deals` | Place deal |
| POST | `/v1/marketplace/deals/{id}/request` | Take deal (settle) |
| POST | `/v1/marketplace/deals/{id}/cancel` | Cancel own open deal |

Client Web: `/marketplace`, `/marketplace/new`, `/marketplace/deals/:id`.
