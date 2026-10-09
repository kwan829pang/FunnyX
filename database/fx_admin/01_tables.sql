-- fx_admin tables (2)

CREATE TABLE IF NOT EXISTS fx_admin.admin_users (
    id BIGSERIAL PRIMARY KEY,
    username VARCHAR(128) NOT NULL UNIQUE,
    role VARCHAR(64) NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'active',
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS fx_admin.admin_action_logs (
    id BIGSERIAL PRIMARY KEY,
    admin_id BIGINT NOT NULL REFERENCES fx_admin.admin_users(id),
    action VARCHAR(128) NOT NULL,
    metadata JSONB,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL DEFAULT 0
);
