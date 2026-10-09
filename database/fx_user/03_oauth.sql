-- fx_user.oauth_identities — durable Partner / platform OAuth subject link
-- Applied after fx_game so game_id FK is valid.
-- user_sessions.oauth_identity_id is added here (sessions created in 01_tables).

CREATE TABLE IF NOT EXISTS fx_user.oauth_identities (
    id BIGSERIAL PRIMARY KEY,
    end_user_id BIGINT NOT NULL REFERENCES fx_user.end_users(id) ON DELETE CASCADE,
    provider VARCHAR(32) NOT NULL
        CHECK (provider IN ('partner', 'platform')),
    partner_id VARCHAR(64) NOT NULL,
    partner_user_id VARCHAR(128) NOT NULL,
    game_id BIGINT REFERENCES fx_game.games(id) ON DELETE SET NULL,
    game_account_id VARCHAR(128),
    status VARCHAR(32) NOT NULL DEFAULT 'active',
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL DEFAULT 0,
    UNIQUE (provider, partner_id, partner_user_id)
);

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'fx_user'
          AND table_name = 'user_sessions'
          AND column_name = 'oauth_identity_id'
    ) THEN
        ALTER TABLE fx_user.user_sessions
            ADD COLUMN oauth_identity_id BIGINT;
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'fk_user_sessions_oauth_identity'
    ) THEN
        ALTER TABLE fx_user.user_sessions
            ADD CONSTRAINT fk_user_sessions_oauth_identity
            FOREIGN KEY (oauth_identity_id)
            REFERENCES fx_user.oauth_identities(id)
            ON DELETE SET NULL;
    END IF;
END $$;
