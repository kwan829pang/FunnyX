# Redis Query and Cache Definitions

## Cache responsibilities

Redis is used for high-speed ephemeral runtime state. It is not treated as the source of truth for user balances, exchange orders, trades, or e-shop settlements.

## Recommended key patterns

- `session:{user_id}` -> active user session metadata
- `token:{token_id}` -> validation metadata for issued tokens
- `market:{market_id}:book` -> current order book snapshot
- `market:{market_id}:ticker` -> last price, bid, ask, volume (live only; historical OHLC is `fx_market_data.market_ohlcvs`)
- `market:{market_id}:status` -> market availability and engine status
- `config:system` -> runtime system configuration snapshot
- `gateway:route:{service_name}` -> gateway service mapping metadata
- `health:{service_name}` -> last health heartbeat timestamp
- `shop:packages:active` -> cached active Platform Token package catalog
- `shop:order:{shop_order_id}` -> pending shop order snapshot (TTL aligned to expires_at)
- `shop:partner:{partner_order_no}` -> reverse lookup from partner payment id to shop order
- `shop:settle:lock:{partner_order_no}` -> short-lived lock for idempotent webhook settlement

## Example Redis commands

```bash
# Store user session metadata
SET session:1001 "{\"token\":\"abc123\",\"expires_at\":1700000000000}"

# Store a token validation payload
SET token:tok_001 "{\"client_id\":1001,\"expires_at\":1700000000000,\"scope\":\"socket\"}"

# Store active order book snapshot
HSET market:101:book buy 5 sell 3 last_price 100.5

# Store ticker summary
SET market:101:ticker "{\"bid\":100.50,\"ask\":100.52,\"last\":100.51,\"volume\":12345.67}"

# Store config snapshot
SET config:system "{\"allow_list_version\":3,\"maintenance_mode\":false}"

# Cache active e-shop catalog (platform PLT_* packages + Corp products)
SET shop:packages:active "[{\"code\":\"PLT_1000\",\"coin_amount\":1000,\"game_coin\":\"PLT\"},{\"code\":\"PLT_1500\",\"coin_amount\":1500,\"game_coin\":\"PLT\"},{\"code\":\"PLT_3000\",\"coin_amount\":3000,\"game_coin\":\"PLT\"},{\"code\":\"PLT_10000\",\"coin_amount\":10000,\"game_coin\":\"PLT\"}]"
EXPIRE shop:packages:active 300

# Cache pending shop order with TTL until expiry
SET shop:order:5001 "{\"id\":5001,\"status\":\"pending\",\"partner_order_no\":\"PAY-7788\",\"coin_amount\":100}"
EXPIRE shop:order:5001 1800

# Map partner payment id to shop order
SET shop:partner:PAY-7788 "5001"
EXPIRE shop:partner:PAY-7788 1800

# Settlement lock to avoid double credit on duplicate webhooks
SET shop:settle:lock:PAY-7788 "1" NX EX 30
```

## Access patterns for first draft

```bash
# Get a current session
GET session:1001

# Check whether token is still valid
GET token:tok_001

# Read the live market snapshot
HGETALL market:101:book

# Read the latest ticker
GET market:101:ticker

# Read e-shop package catalog
GET shop:packages:active

# Resolve shop order from partner payment id
GET shop:partner:PAY-7788
GET shop:order:5001
```

## Notes

- Keep cache TTLs short for trading-state and session data.
- Align pending shop-order TTLs with `shop_orders.expires_at`; expire or delete cache keys when the order is paid, failed, or cancelled.
- Use Redis for fast reads and ephemeral state only.
- Maintain the ownership rule: Redis is provisioned per service group and not shared globally.
- The primary transaction data remains in PostgreSQL or other persistent storage.
- Wallet credits and shop settlement must always be committed in PostgreSQL; Redis locks only reduce duplicate processing races.
