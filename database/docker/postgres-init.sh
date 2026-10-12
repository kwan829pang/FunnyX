#!/bin/sh
# First-boot schema apply for FunnyX (DDL only — no seed).
set -eu

DDL_DIR="${FUNNYX_DDL_DIR:-/ddl}"

if [ ! -f "${DDL_DIR}/run_all.sql" ]; then
  echo "funnyx postgres-init: missing ${DDL_DIR}/run_all.sql" >&2
  exit 1
fi

echo "funnyx postgres-init: applying ${DDL_DIR}/run_all.sql"
cd "${DDL_DIR}"
psql -v ON_ERROR_STOP=1 \
  --username "${POSTGRES_USER}" \
  --dbname "${POSTGRES_DB}" \
  -f run_all.sql
echo "funnyx postgres-init: schema apply complete"
