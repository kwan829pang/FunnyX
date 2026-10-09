# Message Center

Drains `fx_events.outbound_notices`, writes user `notifications`, optional partner HTTP push, optional Mongo history.

Spec: [doc/message_center.md](../../doc/message_center.md). Dev stand-in (no Postgres): [dev_simulator/message-queue](../../dev_simulator/message-queue).

## Run

```bash
cd backend/message_center
cp .env.sample .env
# set POSTGRES_URL for real drain
cargo run
```

- HTTP: `8085`
- Heartbeat socket: `9005`
- Public: `GET /health`
- Internal: `POST /v1/notices` (`X-Internal-Key`) — Core Engine compatible
- Session: `GET /v1/notifications`, `PATCH /v1/notifications/{id}`
