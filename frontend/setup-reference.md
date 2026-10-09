# Frontend setup reference

This file stores the shared setup instructions for the Flutter apps in this workspace. The project keeps the setup flow in one place so the admin panel and client web apps do not duplicate the same command block.

## Shared setup

```bash
cp .env.sample .env
flutter pub get
dart run build_runner build --delete-conflicting-outputs
flutter run -d chrome
```

## Notes

- Shared code lives in [`common`](../common)
- Frontend apps use **GetX** for routing and state management
- `admin_panel` and `client_web` each keep their own route setup and feature pages
- Do not store company master credentials in the frontend apps

## Related docs

- [admin_panel/README.md](admin_panel/README.md)
- [client_web/README.md](client_web/README.md)
- [common/README.md](common/README.md)
