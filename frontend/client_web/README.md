# FunnyX Client Web

Flutter Web end-user UI for auth, wallet, trade, and e-shop flows. Shared code lives under [`../common`](../common), and the shared frontend setup commands are kept in [`../setup-reference.md`](../setup-reference.md).

## Auth (register / login / OAuth)

Client Web calls **Gateway** (default `http://127.0.0.1:8080`), which proxies to Client Center. Sessions (`sts_…`) come from **Session Token Server**.

| UI | API / flow |
| --- | --- |
| Login | `POST /v1/client/login` |
| Register | `POST /v1/client/register` |
| Logout | `POST /v1/client/logout` |
| **Path B** Partner OAuth | `GET /v1/client/oauth/partner/{id}/start` → Partner → STS → `/oauth/callback` |
| **Path C** Platform OAuth | Partner embeds `/v1/oauth/authorize` (STS IdP); Client Web shows copyable authorize URL |

### Compare the two OAuth directions

| | Path B — Partner → Platform | Path C — Platform → Partner |
| --- | --- | --- |
| Where user registered | Partner game / website | FunnyX Client Web |
| Who starts OAuth | **Client Web** (“Continue with Partner OAuth”) | **Partner** app/site |
| Broker / IdP | STS Partner broker via Client Center | STS Platform IdP (`/v1/oauth/*`) |
| Result for Client Web | `sts_…` session on callback | Partner gets its own session; user already has platform account |

```text
Path B: Client Web → CC /partner/.../start → STS → Partner authorize
      → STS callback → CC complete → Client Web /oauth/callback?access_token=sts_…

Path C: Partner → CC/STS /v1/oauth/authorize → platform login/consent
      → redirect code to Partner → Partner /oauth/token + userinfo
```

Demo Partner Path B needs `company-a-server` on `18102`.

```bash
cd backend/session_token_server && cargo run
cd backend/client_center && cargo run
cd backend/gateway && cargo run                  # HTTP_PORT=8080
cd dev_simulator/company-a-server && cargo run   # HTTP_PORT=18102
```

Password demo: `demo_user` / `demo`. Partner OAuth demo user: `alice_01` / `demo` on company-a.

## Wallet + home shell

Session routes (Bearer via Gateway → Client Center):

| Route | UI | API |
| --- | --- | --- |
| `/home` | Profile summary + PLT balance chip | `GET /v1/client/profile`, `GET /v1/client/wallet` |
| `/wallet` | Balances, txn history, deposit / withdraw | `GET /v1/client/wallet`, `GET /v1/client/transactions`, `POST /v1/client/deposit\|withdraw` |

Deposit/withdraw need an active game-account binding and `POSTGRES_URL` on Client Center; status stays pending until partner webhook settle.

## E-shop

Session routes under `/shop` (Bearer via Client Center):

| Route | UI |
| --- | --- |
| `/shop` | GridView catalog (platform + corp) |
| `/shop/cart` | Classic cart table |
| `/shop/checkout` | Pick game account + place orders (one API order per line) |
| `/shop/payment` | Payment DataTable (Pay / Cancel); pending **24h** |
| `/shop/orders`, `/shop/orders/:id` | History + detail poll |

Requires an active game-account binding for checkout (`GET /v1/client/game-accounts`).

## Corp onboarding (1A Client Web + 2A multipart)

Public/Corp Bearer routes (not end-user session):

| Route | UI |
| --- | --- |
| `/corp/register` | Company + BR file upload |
| `/corp/login` | Email or master code + password |
| `/corp/home` | Approval status + game list |
| `/corp/games/new` | Game + government/approved docs |

Detail: [doc/corp_onboarding.md](../../doc/corp_onboarding.md).

## Themes & layouts

Client Web ships **two** visual systems (see `lib/theme/client_theme.dart`):

| Audience | Theme | Layout shell | Routes |
| --- | --- | --- | --- |
| Normal user (shop) | Light e‑commerce look inspired by [NexaraCart `nexara_cart`](https://github.com/Mahmud0808/NexaraCart) — black accent, soft grey, Nunito | `ClientShell` — top bar + rail / bottom nav | `/login`, `/home`, `/wallet`, `/shop`, `/marketplace`, … |
| Corporate | Slate portal (Source Sans 3) | `CorpShell` — dark sidebar; auth uses `AuthLayout` | `/corp/login`, `/corp/register`, `/corp/home`, `/corp/games/new` |

`ClientThemeController` switches `ThemeData` when the route is under `/corp/*`.

## App notes

- Public routes: `/login`, `/register`, `/oauth/callback`, `/corp/register`, `/corp/login`
- Session routes: `/home`, `/wallet`, `/shop` (+ cart / checkout / payment / orders), `/marketplace*`; Corp: `/corp/home`, `/corp/games/new`
- State and routing use **GetX** (`ClientCenterAuthProvider`, `WalletDataProvider`, `ShopDataProvider`, `OauthFlowService`, `CorpController`)
- `AuthMiddleware` gates Session routes (Token required except Public login/register/OAuth)
- Never store master credentials in the app
- Locales: `en`, `zh_TW`, `zh_CN`

For the common setup command flow, see [`../setup-reference.md`](../setup-reference.md).
