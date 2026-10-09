# FunnyX Flutter frontends

| Folder | Package | Role |
| --- | --- | --- |
| [common](common/) | `funnyx_common` | Shared HTTP, GetX session, theme, layout, auth provider |
| [admin_panel](admin_panel/) | `admin_panel` | Admin Flutter Web |
| [client_web](client_web/) | `client_web` | End-user Flutter Web |

Both apps depend on `common` via a path dependency and use **GetX** for state + routing.

Shared setup instructions are kept in [setup-reference.md](setup-reference.md) so the duplicate command block is not repeated across the app docs.

Config is typed with `envied` (`.env` + `.env.sample`). Session APIs attach `Authorization: Bearer` from `SessionController` except login/register/oauth/health.
