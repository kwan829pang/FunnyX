-- fx_user tables (2 of 4) — end_users, user_sessions
-- oauth_identities is in 03_oauth.sql (after fx_game for optional game_id FK)
-- user_wallets is in 02_user_wallets.sql (after fx_game for FKs)

CREATE TABLE IF NOT EXISTS fx_user.end_users (
    id BIGSERIAL PRIMARY KEY,
    username VARCHAR(128) NOT NULL UNIQUE,
    email VARCHAR(255),
    -- Argon2id PHC string; NULL = OAuth-only account (no password login)
    password_hash VARCHAR(255),
    status VARCHAR(32) NOT NULL DEFAULT 'active',
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL DEFAULT 0
);

-- Platform session issued by Session Token Server; row is the OAuth/password
-- workflow record on Client Center (access + refresh + grant context).
CREATE TABLE IF NOT EXISTS fx_user.user_sessions (
    id BIGSERIAL PRIMARY KEY,
    end_user_id BIGINT NOT NULL REFERENCES fx_user.end_users(id) ON DELETE CASCADE,
    token VARCHAR(255) NOT NULL UNIQUE,
    refresh_token VARCHAR(255) UNIQUE,
    token_type VARCHAR(16) NOT NULL DEFAULT 'Bearer',
    grant_type VARCHAR(32) NOT NULL DEFAULT 'login'
        CHECK (grant_type IN ('login', 'oauth', 'partner_oauth', 'platform_oauth', 'refresh')),
    scope VARCHAR(128) NOT NULL DEFAULT 'http,socket',
    actor_type VARCHAR(32) NOT NULL DEFAULT 'end_user',
    partner_id VARCHAR(64),
    partner_user_id VARCHAR(128),
    game_account_id VARCHAR(128),
    expires_at BIGINT NOT NULL,
    refresh_expires_at BIGINT NOT NULL DEFAULT 0,
    revoked_at BIGINT NOT NULL DEFAULT 0,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL DEFAULT 0
);
