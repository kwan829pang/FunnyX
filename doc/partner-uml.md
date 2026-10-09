# Partner integration UML

Master contracts: [partner.md](partner.md). Auth identity sources: [client_connect.md](client_connect.md). Marketplace gates: [marketplace.md](marketplace.md). Local simulator: [`dev_simulator/company-a-server`](../dev_simulator/company-a-server/). Client Center bind APIs: [`backend/client_center/README.md`](../backend/client_center/README.md).

**Detail scope:** UML / sequence views of Company Partner setup, OAuth, **game-account mapping** (Path A / Path B), Transfer, Item List, optional balance/deposit/withdraw — not fee tables or e-shop fiat JSON (those live in [partner.md](partner.md)).

---

## 1. Component overview

Partner backend endpoints the Corporate User registers with FunnyX (`partner.md` §1.2). Simulator implements these on `company-a-server` (default `:18102`).

```mermaid
flowchart LR
  subgraph FunnyX["FunnyX platform"]
    CW[Client Web]
    CC[Client Center]
    STS[Session Token Server]
    WH[Webhook Server]
    MC[Message Center]
  end

  subgraph Partner["Company Partner server"]
    OAuth[OAuth 2.0<br/>authorize / token / userinfo]
    Lookup[Player lookup<br/>Path A bind]
    Xfer[Transfer API]
    Items[Item List API]
    Bal[Balance API optional]
    Dep[Deposit / Withdrawal optional]
    Pay[Shop payment endpoint<br/>often payment-gate sim]
  end

  CW -->|login / register / OAuth entry| CC
  CC -->|issue / validate session| STS
  CW -->|Path B OAuth start| CC
  CC -->|broker Path B| STS
  STS -->|authorize / token / userinfo| OAuth
  CC -->|Path A bind partner_fetch| Lookup
  CC -->|MasterSigned / API key| Xfer
  CC -->|list game_item| Items
  CC -->|optional| Bal
  CC -->|optional| Dep
  CC -->|e-shop fiat create| Pay
  Pay -->|callback| WH
  WH --> CC
  CC --> MC
```

---

## 2. Corporate partner setup (endpoint registration)

```mermaid
sequenceDiagram
  actor Corp as Corporate User
  participant Admin as Admin API / Panel
  participant CC as Client Center
  participant Partner as Company Partner server

  Corp->>Admin: Register company + Master credentials
  Admin-->>Corp: Approved corp account
  Corp->>CC: Submit partner_endpoints JSON
  Note over Corp,CC: oauth_*, transfer, item_list, player_lookup,<br/>optional deposit/withdraw/payment
  CC->>Partner: Probe GET /health (optional)
  Partner-->>CC: 200 ok
  CC-->>Corp: Endpoints stored status=active
  Note over Corp,CC: C6 fee paid before full API enable (C9)
```

**Setup payload shape** (see also `GET /v1/partner/setup` on the simulator):

```json
{
  "partner_id": "partner_2001",
  "company_name": "Company A Game Partner",
  "server_name": "company-a-server",
  "endpoints": [
    { "type": "oauth_authorize", "endpoint": "http://127.0.0.1:18102/oauth/authorize" },
    { "type": "oauth_token", "endpoint": "http://127.0.0.1:18102/oauth/token" },
    { "type": "oauth_userinfo", "endpoint": "http://127.0.0.1:18102/oauth/userinfo" },
    { "type": "player_lookup", "endpoint": "http://127.0.0.1:18102/api/players/lookup" },
    { "type": "transfer", "endpoint": "http://127.0.0.1:18102/api/transfer" },
    { "type": "item_list", "endpoint": "http://127.0.0.1:18102/api/items" },
    { "type": "balance", "endpoint": "http://127.0.0.1:18102/api/balance" },
    { "type": "deposit", "endpoint": "http://127.0.0.1:18102/api/deposit" },
    { "type": "withdrawal", "endpoint": "http://127.0.0.1:18102/api/withdrawal" },
    { "type": "callback", "endpoint": "http://127.0.0.1:18102/api/callback" }
  ],
  "auth_type": "signature",
  "api_key": "demo-api-key",
  "status": "active"
}
```

---

## 3. Game-account mapping (both login paths)

After identity exists, the platform writes a mapping keyed by **`end_user_id` + `game_id`** → `game_account_id` (in-memory on Client Center today; target table `fx_game.game_accounts`). **Prerequisite:** the game row must already exist on the platform catalog (`GET /v1/client/games`).

Narrative for Path A/B/C registration sources: [client_connect.md](client_connect.md) §2 — this section is the UML for **mapping creation only**.

```mermaid
flowchart TD
  U[End User] --> S{Where did they register?}
  S -->|Platform Client Web| A[Path A — partner_fetch]
  S -->|Partner Game App / site| B[Path B — OAuth direct]
  A --> G{game_id exists on platform?}
  B --> G
  G -->|no| X[Reject — register game first]
  G -->|yes| M[(Mapping: end_user_id + game_id → game_account_id)]
  M --> T[Client Web uses sts_… on Session APIs]
```

| | Path A — Platform then Partner fetch | Path B — Game App / Partner OAuth |
| --- | --- | --- |
| Identity first | `POST /v1/client/register` or `login` → STS session | Partner OAuth via CC → STS broker |
| Game check | Platform `game_id` / `game_code` | Same (resolve via `partner_game_id` or `partner_id`) |
| Partner call | `GET /api/players/lookup` | Already covered by OAuth userinfo |
| Mapping write | `POST /v1/client/game-accounts/bind` `mode=partner_fetch` | Auto on `/partner/complete`, or `mode=direct` |

### 3.1 Path A — Platform register/login, then Partner lookup bind

```mermaid
sequenceDiagram
  actor User as End User
  participant CW as Client Web
  participant CC as Client Center
  participant STS as Session Token Server
  participant Cat as Game catalog
  participant Partner as Company Partner

  User->>CW: Register / password login
  CW->>CC: POST /v1/client/register|login
  CC->>STS: Issue session (end_user_id)
  STS-->>CC: sts_…
  CC-->>CW: access_token

  Note over User,Cat: Next step — bind mapping
  CW->>CC: GET /v1/client/games
  CC->>Cat: Resolve game_id / DEMO_GAME
  Cat-->>CC: active + partner_game_id

  CW->>CC: POST /v1/client/game-accounts/bind<br/>mode=partner_fetch<br/>game_id + partner_username<br/>Bearer → end_user_id key
  CC->>Cat: Assert game_id active
  CC->>Partner: GET /api/players/lookup<br/>game_id=game_001&username=…
  Partner-->>CC: partner_user_id, game_account_id, playing=true
  CC->>CC: Upsert mapping<br/>(end_user_id, game_id, game_account_id)
  CC-->>CW: GameAccountBinding
```

### 3.2 Path B — Partner OAuth (Game App), direct mapping

Client Web does **not** exchange the Partner token itself; Session Token Server brokers OAuth, then Client Center writes the mapping.

```mermaid
sequenceDiagram
  actor User as End User
  participant CW as Client Web
  participant CC as Client Center
  participant STS as Session Token Server
  participant Partner as Partner OAuth
  participant Cat as Game catalog

  User->>CW: Continue with Partner OAuth
  CW->>CC: GET /v1/client/oauth/partner/{id}/start
  CC->>STS: Start Partner broker
  STS->>Partner: GET /oauth/authorize
  Partner->>User: Login / consent
  Partner-->>STS: Redirect ?code=…
  STS->>Partner: POST /oauth/token
  Partner-->>STS: access_token
  STS->>Partner: GET /oauth/userinfo
  Partner-->>STS: partner_user_id, game_id, game_account_id
  STS->>STS: Link/create end_user + issue sts_…
  STS->>CC: /partner/complete (+ claims)
  CC->>Cat: Resolve partner game_id → platform game_id
  CC->>CC: Direct upsert mapping<br/>(end_user_id, game_id, game_account_id)
  CC-->>CW: /oauth/callback?access_token=sts_…
```

```mermaid
classDiagram
  class PartnerOAuth {
    +authorize(client_id, redirect_uri, state)
    +token(code, client_secret)
    +userinfo(access_token)
  }
  class PlayerLookup {
    +lookup(game_id, username|partner_user_id)
  }
  class UserInfoClaim {
    +partner_user_id
    +game_id
    +game_account_id
    +username
    +status
  }
  class GameAccountBinding {
    +end_user_id
    +game_id
    +game_account_id
    +partner_id
    +partner_user_id
    +bind_source partner_fetch|direct
  }
  PartnerOAuth --> UserInfoClaim : Path B userinfo
  PlayerLookup --> UserInfoClaim : Path A lookup
  UserInfoClaim --> GameAccountBinding : CC upsert
```

---

## 4. Marketplace: Item List + Transfer (C7)

Place-deal gates require Transfer open; `game_item` also requires Item List → opaque `item_ref_id`. Mapping from §3 must already exist for the user’s `game_account_id`.

```mermaid
sequenceDiagram
  actor User as End User
  participant CW as Client Web
  participant CC as Client Center
  participant Partner as Company Partner

  User->>CW: Place deal (game_item)
  CW->>CC: Request item list for game account
  CC->>Partner: GET /api/items?game_account_id=player_9001
  Partner-->>CC: items[{item_ref_id, name, qty}]
  CC-->>CW: Pickable items
  User->>CW: Select item_ref_id=sword_01
  CW->>CC: Create marketplace deal (offer_item_ref_id)
  Note over CC: Persist VARCHAR offer_item_ref_id<br/>no platform item escrow table

  User->>CW: Request / settle deal
  CW->>CC: Fulfill deal
  CC->>Partner: POST /api/transfer
  Note over Partner: from/to game accounts,<br/>asset_type=game_item, item_ref_id
  Partner-->>CC: transfer_id status=pending→success
  CC-->>CW: Deal completed + outbound_notice
```

```mermaid
stateDiagram-v2
  [*] --> Pending: POST /api/transfer
  Pending --> Success: X-Sim-Status SUCCESS {ms}
  Pending --> Rejected: X-Sim-Status REJECTED {ms}
  Pending --> Cancelled: X-Sim-Status CANCEL {ms}
  Success --> [*]
  Rejected --> [*]
  Cancelled --> [*]
```

---

## 5. Optional deposit / withdrawal (partner wallet path)

Used only when the partner provides their own money APIs instead of platform wallet APIs. Fee / fiat tables: [partner.md](partner.md).

```mermaid
sequenceDiagram
  actor User as End User
  participant CW as Client Web
  participant CC as Client Center
  participant Partner as Company Partner
  participant MC as Message Center

  User->>CW: Deposit request
  CW->>CC: Create deposit_withdrawal_txn
  CC->>Partner: POST /api/deposit
  Partner-->>CC: partner_txn_id status=pending
  Note over Partner: After X-Sim-Status delay → success/rejected
  Partner-->>CC: Final status (poll or callback)
  CC->>MC: outbound_notices
  MC-->>CW: User notification
```

---

## 6. Required vs optional API matrix

```mermaid
flowchart TB
  subgraph Required["Always required"]
    A1[OAuth 2.0]
    A4[Transfer]
  end
  subgraph Bind["Required for Path A bind"]
    A6[Player lookup]
  end
  subgraph Marketplace["Required for game_item deals"]
    A5[Item / game-assets list]
  end
  subgraph Optional["Only if partner owns wallet APIs"]
    A2[Balance]
    A3[Deposit / Withdrawal]
  end
  Join[Corporate join] --> Required
  Join --> Bind
  Join --> Marketplace
  Join --> Optional
```

| # | API | Simulator path | Required? |
| --- | --- | --- | --- |
| 1 | OAuth 2.0 | `/oauth/authorize`, `/oauth/token`, `/oauth/userinfo` | Yes |
| 2 | Balance | `GET /api/balance` | Optional |
| 3 | Deposit / Withdrawal | `POST /api/deposit`, `POST /api/withdrawal` | Optional |
| 4 | Transfer | `POST /api/transfer` | Yes |
| 5 | Item List | `GET /api/items` | Yes for Marketplace `game_item` |
| 6 | Player lookup | `GET /api/players/lookup` | Yes for Path A `partner_fetch` bind |

---

## 7. Local simulator topology

```mermaid
flowchart LR
  PG[payment-gate :18100]
  MQ[message-queue :18101]
  CA[company-a-server :18102]
  STS[session-token-server :8082]
  CC[client-center :8083]
  Plat[FunnyX later Gateway / etc.]

  CC --> STS
  CC -->|OAuth Transfer Items lookup| CA
  Plat -->|shop fiat| PG
  Plat -->|notice drain test| MQ
  PG -->|webhook| Plat
  CA -->|optional callback| Plat
```

Run guide: [`dev_simulator/README.md`](../dev_simulator/README.md), [`dev_simulator/company-a-server/README.md`](../dev_simulator/company-a-server/README.md).
