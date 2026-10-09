-- Seed runner: DEVELOPMENT (00 baseline + 01 demo)
-- Do NOT run against production.
-- From database/:
--   psql -d funnyx -f seed/run_seed_dev.sql

\echo '=== seed/00_seed_data.sql (live + demo) ==='
\i seed/00_seed_data.sql

\echo '=== seed/01_demo_data.sql (development only) ==='
\i seed/01_demo_data.sql

\echo '=== seed development done ==='
