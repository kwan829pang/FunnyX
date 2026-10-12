# Phase 1 §3 — schema + seed smoke (requires psql on PATH).
# Usage:
#   $env:PGPASSWORD='funnyx'; pwsh -File scripts/db_smoke.ps1
$ErrorActionPreference = "Stop"

$HostName = if ($env:PGHOST) { $env:PGHOST } else { "127.0.0.1" }
$Port = if ($env:PGPORT) { $env:PGPORT } else { "5432" }
$User = if ($env:PGUSER) { $env:PGUSER } else { "funnyx" }
$Db = if ($env:PGDATABASE) { $env:PGDATABASE } else { "funnyx" }
if (-not $env:PGPASSWORD) { $env:PGPASSWORD = "funnyx" }

function Sql([string]$q) {
    psql -h $HostName -p $Port -U $User -d $Db -v ON_ERROR_STOP=1 -t -A -c $q
}

Write-Host "==> ping" -ForegroundColor Cyan
Sql "SELECT 1;" | Out-Null

Write-Host "==> schemas" -ForegroundColor Cyan
$schemas = Sql "SELECT count(*) FROM information_schema.schemata WHERE schema_name LIKE 'fx_%';"
Write-Host "fx_* schemas: $schemas"
if ([int]$schemas -lt 10) { throw "expected >=10 fx_* schemas" }

Write-Host "==> seed admin" -ForegroundColor Cyan
$admins = Sql "SELECT count(*) FROM fx_admin.admin_users;"
Write-Host "admin_users: $admins"

Write-Host "==> PLT packages" -ForegroundColor Cyan
$pkgs = Sql "SELECT count(*) FROM fx_shop.shop_packages WHERE code LIKE 'PLT_%';"
Write-Host "PLT packages: $pkgs"

Write-Host "==> outbound_notices table" -ForegroundColor Cyan
Sql "SELECT count(*) FROM fx_events.outbound_notices;" | Out-Null

Write-Host "PASS: db smoke" -ForegroundColor Green
exit 0
