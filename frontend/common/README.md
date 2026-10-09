# funnyx_common

Shared Flutter library for `admin_panel` and `client_web`.

- `AppConfig` — API base URL / app name
- `SessionController` (GetX) — reactive session JWT
- `AuthMiddleware` — GetX route guard
- `ApiClient` — Dio + Bearer interceptor (skips public paths)
- `AuthDataProvider` / `MockAuthDataProvider` — swappable auth
- `AppTheme` — light / dark Material 3
- `ResponsiveScope` — mobile / tablet / desktop

```yaml
dependencies:
  funnyx_common:
    path: ../common
```
