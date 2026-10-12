# Corp onboarding wizards

Client Web company registration (BR upload) and game document submission, with Admin approve/reject.

## Flow

1. **Register** (`POST /v1/corp/register`, multipart) — company profile + BR file → `corporate_users.status=pending`, document `business_registration`.
2. **Wait** — Corp portal shows pending until Admin approves.
3. **Admin approve** (`POST /v1/admin/corporate-users/{id}/approve`) — `status=active`, `api_enabled=true`.
4. **Submit game** (`POST /v1/corp/games`, multipart) — game metadata + `government_approval` / `approved_material` → `games.status=pending`.
5. **Admin approve game** (`POST /v1/admin/games/{id}/approve`) — `games.status=active` (listed for players).

Files are stored under Client Center `UPLOAD_DIR` (shared with Admin via same path for review downloads).

## Client Web routes

| Route | Purpose |
| --- | --- |
| `/corp/register` | Wizard: company + BR upload |
| `/corp/login` | Email or master code + password |
| `/corp/home` | Status, credentials tip, game list |
| `/corp/games/new` | Game + government / materials upload |

## Admin Panel

`/corp-onboard` — Companies and Games tabs with Approve / Reject.
