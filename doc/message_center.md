# Message Center

Detail for [project.md](project.md) §5 / [development.md](../development.md) §15. Crate: [`backend/message_center`](../backend/message_center/).

**Detail scope:** Drain Postgres outbox, user inbox APIs, partner HTTP push, dual health/heartbeat. Not Gateway socket fan-out to Client Web (§13). Not Webhook inbound settle.

## Ports

| Surface | Port | Audience |
| --- | --- | --- |
| HTTP | **8085** | Public `GET /health`; Internal notices; Session inbox |
| Socket | **9005** | Private PING/PONG (`funnyx-heartbeat`) for Config registry |

Dev stand-in without Postgres: [`dev_simulator/message-queue`](../dev_simulator/message-queue) on **18101**. Core Engine `MESSAGE_CENTER_URL` may point at either; production-shaped path uses `http://127.0.0.1:8085`.

## Flow

```
Producers (Webhook / CC / Core Engine POST)
    → fx_events.outbound_notices (delivery_status=pending)
Message Center drain (SKIP LOCKED)
    → fx_events.notifications (user inbox)
    → mark outbound sent
    → optional partner HTTP POST + webhook_events log
    → optional Mongo notice_history (MONGO_URL)
Client Web (Session) → GET /v1/notifications
```

`corp_partner_notices` is **not** drained here (corp/Admin channel).

## APIs

| Method | Path | Auth | Purpose |
| --- | --- | --- | --- |
| GET | `/health` | Public | Third-party / Cloudflare liveness |
| POST | `/v1/notices` | Internal | Enqueue (Core Engine body) |
| GET | `/v1/notices` | Internal | List / filter outbox |
| GET | `/v1/notices/stats` | Internal | Counts by `delivery_status` |
| GET | `/v1/notices/{id}` | Internal | Inspect one |
| GET | `/v1/notifications` | Session | End-user inbox |
| PATCH | `/v1/notifications/{id}` | Session | Mark read |

## Partner callback resolve order

1. `payload.callback_url`
2. `webhook_endpoints` for `payload.corporate_user_id` + kind derived from `source_type`
3. Env `PARTNER_NOTICE_URL`

## Event coverage

`source_type` / `event_type` enums match [`database/fx_events/01_tables.sql`](../database/fx_events/01_tables.sql) (`created`, `pending_payment`, `completed`, `cancelled`, `rejected`, `filled`, …).
