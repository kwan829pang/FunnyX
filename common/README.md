# FunnyX common Rust workspace

Shared libraries for backend services. See `doc/project.md` §8 and `development.md` Phase 4 §11.

| Crate | Role |
| --- | --- |
| `funnyx-error` | Shared `Error` / `Result` |
| `funnyx-types` | `system_type.md` enums |
| `funnyx-time` | UTC+0 BIGINT timestamps |
| `funnyx-config` | Constants + `GlobalConfig` (missing env → defaults) |
| `funnyx-health` | HTTP `/health` payloads |
| `funnyx-heartbeat` | Socket PING/PONG helpers, Config Server registry heartbeat client, staleness |
| `funnyx-socket-msg` | Socket message encode/decode |
| `funnyx-socket` | Framing, lz4, gzip, PING/PONG |
| `funnyx-net-api` | HTTP envelopes + `/v1` path constants |
| `funnyx-auth` | Session / MasterSigned / OAuth types |
| `funnyx-prelude` | Re-exports |

```bash
cd common
cargo test
```

Services depend on path crates, e.g. `funnyx-prelude = { path = "../common/funnyx-prelude" }`.
