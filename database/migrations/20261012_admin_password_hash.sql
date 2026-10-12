-- Admin password hashes for Postgres-backed login (fx_admin).
ALTER TABLE fx_admin.admin_users
    ADD COLUMN IF NOT EXISTS password_hash VARCHAR(255);

COMMENT ON COLUMN fx_admin.admin_users.password_hash IS
    'Argon2id hash; NULL means demo bootstrap password until first set';
