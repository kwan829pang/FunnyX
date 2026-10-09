# Marketplace

This doc is a detail file for the master architecture in [project.md](project.md). See the product summary in [readme.md](../readme.md) and the API index in [api-master.md](../api-master.md).

**Detail scope:** C7 place-deal gates, deal actions, asset acceptance via partner Transfer + Item List APIs, and Marketplace auth pointers — not e-shop fiat or Corp C6 fees.

Master gate summary: [project.md](project.md) §4 step 9. **C7** rules below are authoritative for who may **place** a deal.

- Users may sell value into **Platform Coin**, or into **Company Coin** — but only the **parent company’s** Company Coin, never another company’s coin.
- Supported deal assets: **Game Coin**, **Company Coin**, and **Game Items**.
- Deal settlement moves game assets via the Company Partner **Transfer** API (token/asset movement; not e-shop fiat).
- Game items: partner **Item / game-assets list API** (readme #5) returns opaque `item_ref_id` values; platform stores `offer_item_ref_id` / transfer `item_ref_id` (`VARCHAR`). No platform item catalog or escrow table (`database/fx_marketplace`).

## 1. Right to place a deal (C7)

A normal user may **place / post a deal** only when **all** of the following are true:

| # | Requirement | Meaning |
| --- | --- | --- |
| 1 | **Verified** | User has bought **Platform Token** on the platform (e-shop Platform Token package purchase settled / credited). |
| 2 | **Playing a listed game** | User is bound to / actively playing **at least one** game listed on the platform. |
| 3 | **Partner Transfer API open** | The deal’s game assets are only accepted when that game’s Company Partner has enabled the **Transfer** API for those assets (Game Coin / Company Coin / Game Items). Partners must expose this API to join the platform for marketplace deals. |
| 4 | **Item list when offering game_item** | For `offer_asset_type = game_item`, partner Item List API must return a pickable `item_ref_id` for the poster’s game account. |

If any gate fails, the user must **not** create a new listing. They may still browse; **request a deal** may remain available per product UI rules, but posting is blocked.

## 2. Deal actions

| Action | Who | Meaning |
| --- | --- | --- |
| Place / post a deal | Normal user meeting §1 | Create a listing to sell Game Coin, Company Coin, or Game Items (`offer_item_ref_id` required for items) |
| Request a deal | Normal user (Session) | Ask to buy or match against an existing deal |
| Browse | Normal user (Session) | View open deals |

Persistence: `fx_marketplace.marketplace_deals`, `marketplace_deal_requests`, `marketplace_transfer_events`. Status notices: `fx_events.outbound_notices`.

## 3. Asset acceptance (partner Transfer + Item List)

- Marketplace deals for a game are allowed only if that partner’s **transfer** endpoint is registered, approved, and enabled for the asset type (`PARTNER_ENDPOINT.type = transfer`).
- For game items, Client Web lists assets via partner Item List API, user picks an item, platform stores opaque `offer_item_ref_id`.
- On deal settle / fulfill, platform calls the partner Transfer API (same `item_ref_id` for items) to move Game Items and/or coins.
- Without Transfer API: asset type is not deal-eligible (listing create rejected). Without Item List: game_item listings are rejected.
- Want side: Platform Token or **parent** Company Coin only (app check against deal `corporate_user_id`).

See required partner APIs in [partner.md](partner.md) §0, §1.4, §1.5.

## 4. Auth

Client Web: Token Server token on all Marketplace APIs (after login/OAuth). Company Partner Transfer / Item List / Client Center: Master Account Code + Master ID + API Key + Secret. Partner join also requires OAuth 2.0. See [client_connect.md](client_connect.md), [partner.md](partner.md) §0.
