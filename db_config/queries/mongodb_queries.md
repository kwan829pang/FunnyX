# MongoDB Query and Collection Definitions

## Suggested collections

- `chat_messages`
- `notifications` (history mirror after PG `outbound_notices` sent; PG is wait-to-send source of truth)
- `user_activity`
- `webhook_events`
- `shop_payment_notices`
- `audit_logs`

## Example document structures

### `chat_messages` (Client Center MongoDB — required)
```json
{
  "_id": "msg_001",
  "client_id": 1001,
  "channel": "public",
  "sender": "user_1001",
  "receiver": "user_2002",
  "content": "hello",
  "created_at": 1700000000000,
  "status": "sent"
}
```

Client Center owns chat persistence. Do not treat this collection as optional.

### `notifications`
```json
{
  "_id": "note_001",
  "client_id": 1001,
  "type": "order_filled",
  "payload": {
    "order_id": 9001,
    "side": "buy",
    "qty": 2.5,
    "price": 100.5
  },
  "created_at": 1700000000000,
  "read_status": false
}
```

### `webhook_events`

**C5:** `game_coin` is a Company Coin or Game Coin **code**. The platform does **not** restrict the underlying asset: platform-style token, crypto token, or stablecoin (example: `USDT`). That is not e-shop fiat `HKD`/`USD`.

```json
{
  "_id": "evt_001",
  "source": "webhook_server",
  "event_type": "deposit_notice",
  "payload": {
    "tx_id": "tx_123",
    "game_coin": "USDT",
    "asset_kind": "stablecoin",
    "amount": 1000
  },
  "received_at": 1700000000000
}
```

### `shop_payment_notices`
```json
{
  "_id": "shop_evt_001",
  "source": "webhook_server",
  "event_type": "shop_payment_callback",
  "event_id": "pay_evt_8899",
  "partner_order_no": "PAY-7788",
  "shop_order_id": 5001,
  "client_id": 1001,
  "status": "paid",
  "payload": {
    "package_code": "PLT_1000",
    "fiat_currency": "HKD",
    "fiat_paid": 88.00,
    "credit_amount": 1000,
    "credit_game_coin": "PLT"
  },
  "received_at": 1700000000000
}
```

## Example MongoDB query statements

```javascript
// Get recent messages for a user
db.chat_messages.find({
  client_id: 1001
}).sort({ created_at: -1 }).limit(20);

// Fetch unread notifications
db.notifications.find({
  client_id: 1001,
  read_status: false
}).sort({ created_at: -1 });

// Query webhook deposit notices
db.webhook_events.find({
  event_type: "deposit_notice",
  received_at: { $gte: 1700000000000 }
}).sort({ received_at: -1 });

// Query e-shop payment notices by partner order
db.shop_payment_notices.find({
  partner_order_no: "PAY-7788"
}).sort({ received_at: -1 });

// Query e-shop payment notices for a client
db.shop_payment_notices.find({
  client_id: 1001,
  event_type: "shop_payment_callback"
}).sort({ received_at: -1 }).limit(20);

// Search audit activities
db.audit_logs.find({
  actor: "admin_01",
  action: "market_updated"
}).sort({ timestamp: -1 }).limit(50);
```

## Notes

- MongoDB is intended for chat, notification, event, and audit-style records.
- **Client Center requires its own dedicated MongoDB** to store client chat message records (`chat_messages`). This is not optional.
- Message Center also uses dedicated MongoDB for message/notice delivery history.
- Prefer indexed fields such as `client_id`, `event_type`, `channel`, `partner_order_no`, `shop_order_id`, and `created_at` / `received_at`.
- Retention policy should be defined separately for chat history, message history, webhook logs, and shop payment notices.
- MongoDB should be provisioned per service group where document storage is required; it is not assumed to be a single shared platform database.
- **C5:** webhook `game_coin` examples such as `USDT` are valid Company/Game Coin codes (stablecoin). Do not treat them as e-shop fiat `HKD`/`USD`.
- Authoritative shop settlement state remains in PostgreSQL (`shop_orders`, `shop_payment_events`); MongoDB stores chat history (Client Center) and notice/history copies (Message Center).
