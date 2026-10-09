# company-a-server

Local Rust API that simulates **Company Partner A** endpoints for FunnyX integration testing: OAuth 2.0, Transfer, Item List, optional balance / deposit / withdrawal.

Aligned with [doc/partner.md](../../doc/partner.md) §0–§3 and UML [doc/partner-uml.md](../../doc/partner-uml.md).

## Run

```bash
cd dev_simulator/company-a-server
cp .env.sample .env
cargo run
```

Default listen: `http://127.0.0.1:18102`

## Partner setup snapshot

```bash
curl -s http://127.0.0.1:18102/v1/partner/setup
```

Returns the endpoint registration payload (oauth / transfer / item_list / balance / deposit / withdrawal / callback) pointing at this simulator.

## Auth

| Surface | Auth |
| --- | --- |
| OAuth authorize / login / token / userinfo | OAuth client_id + client_secret; Bearer access token for userinfo |
| `/api/*` partner APIs | Header `X-Api-Key: demo-api-key` |
| `GET /api/players/lookup` | Confirm player is on `game_id` (used by Client Center Path A bind) |

## `X-Sim-Status`

Transfer / deposit / withdrawal submits always start **pending**. Schedule final status:

```http
X-Sim-Status: SUCCESS 3000
X-Sim-Status: REJECTED 5000
```

## Demo data

| User | Password | game_account_id | Items |
| --- | --- | --- | --- |
| `alice_01` | `demo` | `player_9001` | `sword_01`, `shield_02` |
| `bob_02` | `demo` | `player_9002` | `potion_03` |

## Quick examples

```bash
# Path A bind helper — is alice playing game_001?
curl -s "http://127.0.0.1:18102/api/players/lookup?game_id=game_001&username=alice_01" \
  -H "x-api-key: demo-api-key"
```

See root [../README.md](../README.md) and [doc/partner-uml.md](../../doc/partner-uml.md).
