# Database seeds

SQL under this folder loads **after** [`../run_all.sql`](../run_all.sql) (schema must exist).

| File | Audience | Purpose |
| --- | --- | --- |
| [`00_seed_data.sql`](00_seed_data.sql) | **Live + demo** | Required baseline: seed admin, `base_fiat_currency=HKD`, Platform Token `PLT`, fixed `PLT_*` shop packages |
| [`01_demo_data.sql`](01_demo_data.sql) | **Development only** | Demo corp, game, company coin, corp shop product, end user, wallets |

## Apply

From `database/`:

```bash
# Schema
psql -d funnyx -f run_all.sql

# Live / staging / any environment that needs baseline catalog
psql -d funnyx -f seed/00_seed_data.sql

# Local development only (never production)
psql -d funnyx -f seed/01_demo_data.sql
```

Or use runners:

```bash
psql -d funnyx -f seed/run_seed.sql          # 00 only
psql -d funnyx -f seed/run_seed_dev.sql      # 00 + 01
```

All seed scripts are idempotent (`ON CONFLICT`).
