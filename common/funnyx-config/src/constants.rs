//! Platform-wide constant values (ports, buffer sizes, paths, timeouts).
//!
//! Used as compile-time defaults when `.env` keys are missing.
//! See `doc/socket_message.md`, `doc/project.md` §8, `technology.md`.

/// Normal Command fixed body size (bytes).
pub const NORMAL_COMMAND_SIZE: usize = 512;

/// Content data capacity inside a Normal Command (512 - 5 header bytes).
pub const NORMAL_COMMAND_CONTENT_SIZE: usize = 507;

/// HTTP health check path.
pub const HEALTH_PATH: &str = "/health";

/// Default HTTP listen bind for local/dev services.
pub const DEFAULT_HTTP_HOST: &str = "0.0.0.0";

/// Default Gateway HTTP port (dev).
pub const DEFAULT_GATEWAY_HTTP_PORT: u16 = 8080;

/// Default Client Center HTTP port (dev).
pub const DEFAULT_CLIENT_CENTER_HTTP_PORT: u16 = 8081;

/// Default Session Token Server HTTP port (dev).
pub const DEFAULT_SESSION_HTTP_PORT: u16 = 8082;

/// Default Admin API HTTP port (dev).
pub const DEFAULT_ADMIN_API_HTTP_PORT: u16 = 8083;

/// Default Config Server HTTP port (dev).
pub const DEFAULT_CONFIG_HTTP_PORT: u16 = 8090;

/// Default Webhook Server HTTP port (dev).
pub const DEFAULT_WEBHOOK_HTTP_PORT: u16 = 8084;

/// Default Webhook Server socket ingest port (dev).
pub const DEFAULT_WEBHOOK_SOCKET_PORT: u16 = 9004;

/// Default socket listen port for Gateway (dev).
pub const DEFAULT_GATEWAY_SOCKET_PORT: u16 = 9000;

/// Default Client Center heartbeat / S2S socket port (dev).
pub const DEFAULT_CLIENT_CENTER_SOCKET_PORT: u16 = 9001;

/// Default Session Token Server heartbeat socket port (dev).
pub const DEFAULT_SESSION_SOCKET_PORT: u16 = 9002;

/// Default Admin API heartbeat socket port (dev).
pub const DEFAULT_ADMIN_API_SOCKET_PORT: u16 = 9003;

/// Default Message Center HTTP port (dev).
pub const DEFAULT_MESSAGE_CENTER_HTTP_PORT: u16 = 8085;

/// Default Message Center heartbeat socket port (dev).
pub const DEFAULT_MESSAGE_CENTER_SOCKET_PORT: u16 = 9005;

/// Default log level string.
pub const DEFAULT_LOG_LEVEL: &str = "info";

/// Default runtime mode.
pub const DEFAULT_RUNTIME_MODE: &str = "development";

/// Heartbeat interval in milliseconds.
pub const DEFAULT_HEARTBEAT_INTERVAL_MS: u64 = 5_000;

/// Socket read/write timeout in milliseconds.
pub const DEFAULT_SOCKET_TIMEOUT_MS: u64 = 30_000;

/// HTTP client / request timeout in milliseconds.
pub const DEFAULT_HTTP_TIMEOUT_MS: u64 = 15_000;

/// Max string length for short config fields (system_type Date Type string max).
pub const MAX_SHORT_STRING_LEN: usize = 50;

/// Session Authorization header name (Client Web Token Server token).
pub const HEADER_AUTHORIZATION: &str = "Authorization";

/// Master Account Code header (Corp MasterSigned).
pub const HEADER_MASTER_ACCOUNT_CODE: &str = "X-Master-Account-Code";

/// Master ID header.
pub const HEADER_MASTER_ID: &str = "X-Master-Id";

/// API Key header.
pub const HEADER_API_KEY: &str = "X-Api-Key";

/// Request signature header.
pub const HEADER_SIGNATURE: &str = "X-Signature";

/// Request timestamp header (BIGINT ms).
pub const HEADER_TIMESTAMP: &str = "X-Timestamp";

/// Default relative path for Config Server whitelist JSON.
pub const DEFAULT_WHITELIST_PATH: &str = "config/whitelist.sample.json";

/// Default empty URL placeholder (filled when env missing; not a secret).
pub const DEFAULT_REDIS_URL: &str = "redis://127.0.0.1:6379";

/// Default local Postgres URL (dev).
pub const DEFAULT_POSTGRES_URL: &str = "postgres://funnyx:funnyx@127.0.0.1:5432/funnyx";

/// Default local Mongo URL (dev).
pub const DEFAULT_MONGO_URL: &str = "mongodb://127.0.0.1:27017";

/// Default service name when unset.
pub const DEFAULT_SERVICE_NAME: &str = "funnyx-service";
