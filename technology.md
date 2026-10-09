# Technology Stack

This document summarizes the technologies, libraries, and runtime platforms used across the FunnyX system. The stack is organized by server instance and service role so that each component has a clear technology profile.

---

## 1. Common Platform Stack

This platform is built around a reusable Game Partner Game Coin model across multiple games and corp-user game companies. The technical stack therefore supports not only exchange matching and messaging, but also game onboarding, market creation, pool lock-up, approved cross-game value flow, and the Client Web e-shop (**C4:** fiat HKD/USD partner payment from Admin system base currency; webhook then credits tokens / fulfills items).

| Category | Technology | Usage |
| --- | --- | --- |
| Primary language | Rust | Core backend services and matching engine |
| Client UI | Flutter | Admin panel and client web application |
| Runtime | Linux / Ubuntu Core | Server deployment environment |
| Config management | `.env`, `env.sample` | Runtime config and environment templates |
| Serialization | `serde`, `serde_json` | JSON and structured data handling |
| Async runtime | `tokio` | High-performance async I/O and concurrency |
| Logging | `tracing`, `log`, `env_logger` | Structured and operational logging |
| Socket / network | `tokio-tungstenite`, raw socket handling | Real-time client and service communication |
| Compression | `lz4_flex`, `flate2` / gzip | Binary and HTTP payload optimization |
| Health checks | HTTP GET `/health`, socket `PING` / `PONG` | Service readiness and heartbeat validation |
| Time handling | UTC+0, BIGINT timestamps | Consistent distributed timestamps |

---

## 2. Gateway

| Component | Technology / Library | Purpose |
| --- | --- | --- |
| Runtime | Rust | Gateway service implementation |
| HTTP layer | `axum` or `actix-web` | Public API entry and routing |
| Socket layer | `tokio` + socket handlers | Client socket connection management |
| Load balancing | custom health-aware routing logic | Route traffic across healthy upstream services |
| In-memory config map | local runtime registry + route tables | Store service topology, health, and connection metadata in memory |
| Config discovery | Config Server heartbeat integration | Refresh local memory map and topology status |
| Health check | HTTP `/health`, socket heartbeat | Service readiness / failover protection |
| Connection strategy | connection pool + queue throttle | Prevent overload and maintain fairness |
| Security | IP whitelist, TLS, request validation | Public traffic protection |
| Serialization | `serde`, `serde_json` | Request parsing and API payload handling |
| Compression | `lz4_flex`, gzip | Optimize internal and external payload transfer |

---

## 3. Client Center

| Component | Technology / Library | Purpose |
| --- | --- | --- |
| Runtime | Rust | Internal orchestration and client service logic |
| API layer | `axum` / `actix-web` | Authenticated API endpoints |
| Socket layer | socket server with message dispatch | Client session and push notifications |
| Auth | Session (Client Web) / MasterSigned (Partner servers) / OAuth 2.0 | End User login or Partner/Platform OAuth → Session Token Server; Gateway validates Session; Partner must expose OAuth + Transfer |
| Event bus | internal event/message dispatcher | Route user and account events |
| Persistence | PostgreSQL + Redis + MongoDB | Account/order/session in PostgreSQL+Redis; **required** MongoDB for client chat message records |
| Chat storage | dedicated Client Center MongoDB | Persist client chat message history |
| Notification push | socket event publishing | Client updates, execution notices, shop settlement, and account events |
| Validation | risk checks, order validation, shop-order validation | Ensure request correctness before forwarding |
| E-shop settlement | partner **fiat** pay + webhook credit/fulfill | Pay HKD/USD (Admin base currency); then Platform `PLT_*` credit or Corp fulfill |
| Logging | `tracing` | Operational and request tracing |

---

## 4. Core Engine

| Component | Technology / Library | Purpose |
| --- | --- | --- |
| Runtime | Rust | Matching engine implementation |
| Core model | in-memory order book | Fast price matching and execution logic |
| Matching engine | custom matching loop | Match buy and sell orders with price-time priority |
| Ring buffer | custom queue / lock-free or reduced-lock structure | Handle order ingestion efficiently |
| Concurrency | `tokio` + worker threads | Parallel processing and order handling |
| Memory model | bounded arrays, atomic counters | Stable high-throughput execution |
| Socket integration | internal binary socket protocol | Receive validated order commands from Client Center |
| Event publishing | execution notifications to Message Center | Propagate trade results and updates |
| Monitoring | health check + metrics | Track engine liveness and performance |
| Serialization | compact binary messages | Low-latency internal payload transfer |

---

## 5. Message Center

| Component | Technology / Library | Purpose |
| --- | --- | --- |
| Runtime | Rust | Messaging and event distribution service |
| API layer | `axum` / `actix-web` | Client-facing message APIs |
| Socket layer | socket service | Real-time message delivery and notifications |
| Message storage | PostgreSQL / MongoDB | Persistent message records |
| Cache layer | Redis | Fast access to session and message metadata |
| Event routing | internal dispatcher | Forward messages to connected clients and services |
| Outbound push API | HTTP `POST` to partner servers | Send webhook-style event payloads to upstream or partner services |
| Partner delivery config | endpoint registry + auth tokens | Manage target URLs, signing keys, and retry rules for each partner |
| Delivery tracking | status log + ack checks | Record sending attempts and delivery outcomes |
| Ring buffer | custom buffering logic | Handle burst message ingestion |
| Compression | `lz4_flex` | Efficient socket payload transfer |
| Health | HTTP `/health`, socket `PING` / `PONG` | Service monitoring |

---

## 6. Webhook Server

| Component | Technology / Library | Purpose |
| --- | --- | --- |
| Runtime | Rust | External event ingestion service |
| HTTP listener | `axum` / `actix-web` | Receive external callback, deposit notices, and e-shop payment callbacks |
| Event parsing | `serde`, JSON / custom parser | Normalize webhook payloads |
| Ring buffer | event ingestion queue | Process high-rate external notifications |
| Event forwarding | internal socket / HTTP callers | Send processed data to Message Center and Client Center |
| Shop callbacks | idempotent settlement handoff | Forward paid/failed shop payment events without double-credit risk |
| Storage | SQLite / LevelDB | Local persistence for webhook processing state |
| Monitoring | HTTP `/health` | Service status verification |

---

## 7. Config Server

| Component | Technology / Library | Purpose |
| --- | --- | --- |
| Runtime | Rust | Central config and service registry |
| API layer | `axum` / `actix-web` | Service config and control endpoints |
| Socket layer | socket service | Service registration and heartbeat tracking |
| Config storage | JSON whitelist file | Load IP allowlist and config values from file-based configuration |
| Registry data | in-memory maps + config store | Track Gateway and Core Engine metadata |
| Access control | IP whitelist | Restrict non-public configuration endpoints |
| Health model | HTTP `/health`, socket heartbeat | Detect alive and healthy instances |
| Reload support | config reload APIs | Dynamic update without full restart |
| Logging | `tracing` | Operational and admin-level visibility |

---

## 8. Admin API

| Component | Technology / Library | Purpose |
| --- | --- | --- |
| Runtime | Rust | Backend for admin platform operations |
| API framework | `axum` / `actix-web` | REST API for admin actions |
| Role control | RBAC model | Root Admin, Manager, Author, Maker, Checker |
| Persistence | SQLite / LevelDB | Local admin config and metadata |
| Config integration | Config Server API client | Fetch active environment settings |
| Event / socket | socket push subsystem | Admin updates and notifications |
| Health check | HTTP `/health` | Operational verification |

---

## 9. Admin Panel

| Component | Technology / Library | Purpose |
| --- | --- | --- |
| Framework | Flutter Web | Admin web application |
| State management | `provider`, `bloc`, or `riverpod` | UI state and business logic |
| HTTP client | `dio` | API communication with Admin API |
| Runtime config | `flutter_dotenv` + `envied` pattern | Environment-based config handling |
| Security | whitelisting + secure session flow | Access restriction and admin safety |
| E-shop admin | base fiat + packages + Corp monitor | Set system base currency HKD/USD; set `PLT_*` fiat prices; suspend Corp products |

---

## 10. Client Web

| Component | Technology / Library | Purpose |
| --- | --- | --- |
| Framework | Flutter Web | End-user UI for trading, e-shop, and account views |
| State management | `provider`, `bloc`, or `riverpod` | Frontend state and session management |
| HTTP client | `dio` | REST calls to **Client Server** (API #1 only; not Gateway Master APIs) |
| Socket client | websocket library | Realtime after Session Token Server login returns session token to Client Web |
| Runtime config | `flutter_dotenv` + `envied` | Secure environment configuration |
| UI features | market dashboard, trade panel, notices, e-shop catalog (platform + Corp), checkout, order status | Trading and e-shop (platform packages + Corp products) |
| Auth note | C8 + OAuth 2.0 | Client Web: Token Server token after login/OAuth; Partner: OAuth + Transfer required; Corp APIs use Master Account Code + Master ID + API Key + Secret; see `doc/client_connect.md` |

---

## 11. Session Token Server

| Component | Technology / Library | Purpose |
| --- | --- | --- |
| Runtime | Rust | Token-based session validation |
| API layer | `axum` / `actix-web` | Token issuance and validation endpoints |
| Cache | Redis | Token and session retrieval |
| Security | signed tokens, expiration policies | Issue after password login or OAuth 2.0 callback; protect authentication flows |
| Health check | HTTP `/health` | Service monitoring |

---

## 12. Database and Storage Systems

| Storage Layer | Technology | Purpose |
| --- | --- | --- |
| Transactional DB | PostgreSQL | Account, trade, e-shop, configuration, and operational data |
| Cache DB | Redis | Session caching, message cache, shop package catalog, and hot data |
| Message DB | MongoDB | Chat, message, and shop payment notice storage (Message Center + **required Client Center chat**) |
| Local config DB | SQLite / LevelDB | Small local state, config, and admin data |
| Time format | UTC+0 + BIGINT | Standardized timestamps |

Note: these stores are not treated as a single shared platform-wide database. Each server group owns its own dedicated Redis or database instance when needed.

### 12.1 Redis and Database Usage by Server Type

| Server Type | Redis | PostgreSQL | MongoDB | SQLite / LevelDB | Notes |
| --- | --- | --- | --- | --- | --- |
| Config Server | No | No | No | No | Metadata and service registry are held in memory; no persistent DB required by default |
| Gateway | Optional local cache only | No | No | No | Primarily in-memory routing map and active connection state |
| Client Center | Yes, dedicated instance for the group | Yes, dedicated instance for the group | **Required**, dedicated instance for the group | No | PostgreSQL/Redis for account, orders, e-shop, session; **MongoDB required for client chat message records** |
| Core Engine | Optional local cache if needed | Yes, dedicated instance if required for history / settlement | No | No | Uses in-memory matching state and may persist trade history when needed |
| Message Center | Yes, dedicated instance for the group | Yes, dedicated instance for the group | Yes, dedicated instance for the group | No | Redis for fast cache, PostgreSQL for transactional data, MongoDB for message records |
| Webhook Server | Optional local cache | No | No | Yes, dedicated local storage | Local state and notification processing data |
| Admin API | Yes, dedicated Admin Redis if needed | No | No | Yes, dedicated local DB | Local admin config and metadata |
| Session Token Server | Yes, dedicated Token Redis instance | No | No | No | Token/session cache and validation |
| Admin Panel | No | No | No | No | Frontend UI, no direct database usage |
| Client Web | No | No | No | No | Frontend UI, no direct database usage |

---

## 13. Common Libraries by Function

| Function | Libraries / Tools | Description |
| --- | --- | --- |
| Rust async runtime | `tokio` | Core async task and networking model |
| Serialization | `serde`, `serde_json` | Structured object encoding and decoding |
| Compression | `lz4_flex`, `flate2` | Byte and transport compression |
| Logging | `tracing`, `log`, `env_logger` | Operational visibility and debugging |
| Web / API | `axum`, `actix-web` | HTTP server implementation |
| Socket | custom socket protocol, `tokio-tungstenite` | Real-time communication |
| DB integration | `sqlx`, `tokio-postgres`, `mongodb`, `redis` | Database access and data operations |
| Config | `dotenvy`, `env_logger`, config files | Environment management and local setup |
| Flutter config | `flutter_dotenv`, `envied` | Secure and typed environment values |
| UI state | `provider`, `bloc`, `riverpod` | Flutter front-end state handling |

---

## 14. Recommended Technology Summary

| Layer | Core Technologies |
| --- | --- |
| Backend | Rust, Tokio, Axum / Actix-web |
| Frontend | Flutter, Flutter Web |
| Real-time transport | Socket protocol, WebSocket, binary buffers |
| Data | PostgreSQL, Redis, MongoDB, SQLite / LevelDB |
| Messaging | Ring buffer, event bus, socket push |
| Security | IP whitelist, token auth, signature validation |
| Health | HTTP GET `/health`, socket `PING` / `PONG` |
| Config | `.env`, `env.sample`, `envied` |

This stack is designed to support a low-latency trading system with strong throughput, stable service discovery, and reliable real-time communication across internal and external components.

---

## 15. Development Network Setting for Docker

The local development environment can use Docker-defined private networks to isolate service groups and maintain predictable internal communication. The following IP ranges are recommended for each logical server group.

| Server Group | Docker Network | IP Range | Notes |
| --- | --- | --- | --- |
| Config Server | `funnyx-config` | `172.20.0.0/24` | Service discovery and allowlist config |
| Gateway | `funnyx-gateway` | `172.20.1.0/24` | Public ingress and upstream routing |
| Client Center | `funnyx-client-center` | `172.20.2.0/24` | Client API and socket orchestration |
| Core Engine | `funnyx-core-engine` | `172.20.3.0/24` | Private internal matching engine network |
| Message Center | `funnyx-message-center` | `172.20.4.0/24` | Event distribution and message flow |
| Webhook Server | `funnyx-webhook` | `172.20.5.0/24` | External callback processing |
| Admin API | `funnyx-admin` | `172.20.6.0/24` | Admin control plane |
| Session Token Server | `funnyx-session` | `172.20.7.0/24` | Token issuance and validation |
| PostgreSQL Group | `funnyx-db-postgres` | `172.20.10.0/24` | Dedicated PostgreSQL persistence |
| Redis Group | `funnyx-db-redis` | `172.20.11.0/24` | Dedicated Redis cache for each service group |
| MongoDB Group | `funnyx-db-mongo` | `172.20.12.0/24` | Dedicated MongoDB document storage |
| SQLite / LevelDB Group | `funnyx-local-storage` | `172.20.13.0/24` | Local data for admin and webhook node storage |

These ranges are intended for local Docker development and can be adjusted to match the actual deployment environment. The key principle is that each service group and its owned storage should live in a dedicated subnet to avoid accidental cross-service coupling.
