-- Corp onboarding: contact/login, BR + game documents, web sessions
-- Safe to re-run (IF NOT EXISTS / ADD COLUMN IF NOT EXISTS)

ALTER TABLE fx_corp.corporate_users
    ADD COLUMN IF NOT EXISTS contact_email VARCHAR(255);

ALTER TABLE fx_corp.corporate_users
    ADD COLUMN IF NOT EXISTS contact_name VARCHAR(255);

ALTER TABLE fx_corp.corporate_users
    ADD COLUMN IF NOT EXISTS password_hash VARCHAR(255);

ALTER TABLE fx_corp.corporate_users
    ADD COLUMN IF NOT EXISTS approved_at BIGINT NOT NULL DEFAULT 0;

ALTER TABLE fx_corp.corporate_users
    ADD COLUMN IF NOT EXISTS approved_by_admin_id BIGINT;

ALTER TABLE fx_corp.corporate_users
    ADD COLUMN IF NOT EXISTS reject_reason TEXT;

CREATE UNIQUE INDEX IF NOT EXISTS uq_corporate_users_contact_email
    ON fx_corp.corporate_users (lower(contact_email))
    WHERE contact_email IS NOT NULL;

CREATE TABLE IF NOT EXISTS fx_corp.corp_documents (
    id BIGSERIAL PRIMARY KEY,
    corporate_user_id BIGINT NOT NULL REFERENCES fx_corp.corporate_users(id) ON DELETE CASCADE,
    game_id BIGINT REFERENCES fx_game.games(id) ON DELETE CASCADE,
    doc_type VARCHAR(64) NOT NULL
        CHECK (doc_type IN (
            'business_registration',
            'game_government_approval',
            'game_approved_material',
            'other'
        )),
    file_name VARCHAR(255) NOT NULL,
    stored_path VARCHAR(512) NOT NULL,
    content_type VARCHAR(128) NOT NULL DEFAULT 'application/octet-stream',
    byte_size BIGINT NOT NULL DEFAULT 0,
    status VARCHAR(32) NOT NULL DEFAULT 'submitted'
        CHECK (status IN ('submitted', 'accepted', 'rejected')),
    review_note TEXT,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_corp_documents_corp
    ON fx_corp.corp_documents (corporate_user_id, doc_type, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_corp_documents_game
    ON fx_corp.corp_documents (game_id)
    WHERE game_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS fx_corp.corp_web_sessions (
    id BIGSERIAL PRIMARY KEY,
    corporate_user_id BIGINT NOT NULL REFERENCES fx_corp.corporate_users(id) ON DELETE CASCADE,
    token VARCHAR(255) NOT NULL UNIQUE,
    expires_at BIGINT NOT NULL,
    created_at BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_corp_web_sessions_corp
    ON fx_corp.corp_web_sessions (corporate_user_id);

ALTER TABLE fx_game.games
    ADD COLUMN IF NOT EXISTS description TEXT;

ALTER TABLE fx_game.games
    ADD COLUMN IF NOT EXISTS reject_reason TEXT;

ALTER TABLE fx_game.games
    ADD COLUMN IF NOT EXISTS approved_at BIGINT NOT NULL DEFAULT 0;
