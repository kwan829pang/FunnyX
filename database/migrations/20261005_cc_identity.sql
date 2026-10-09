-- Idempotent upgrades for databases that already applied the first-draft DDL.
-- Safe to re-run. New installs get the same shape from run_all.sql.

\set ON_ERROR_STOP on

ALTER TABLE fx_user.end_users
    ADD COLUMN IF NOT EXISTS password_hash VARCHAR(255);

ALTER TABLE fx_user.user_sessions
    ADD COLUMN IF NOT EXISTS refresh_token VARCHAR(255);

ALTER TABLE fx_user.user_sessions
    ADD COLUMN IF NOT EXISTS token_type VARCHAR(16);

ALTER TABLE fx_user.user_sessions
    ADD COLUMN IF NOT EXISTS grant_type VARCHAR(32);

ALTER TABLE fx_user.user_sessions
    ADD COLUMN IF NOT EXISTS scope VARCHAR(128);

ALTER TABLE fx_user.user_sessions
    ADD COLUMN IF NOT EXISTS actor_type VARCHAR(32);

ALTER TABLE fx_user.user_sessions
    ADD COLUMN IF NOT EXISTS partner_id VARCHAR(64);

ALTER TABLE fx_user.user_sessions
    ADD COLUMN IF NOT EXISTS partner_user_id VARCHAR(128);

ALTER TABLE fx_user.user_sessions
    ADD COLUMN IF NOT EXISTS game_account_id VARCHAR(128);

ALTER TABLE fx_user.user_sessions
    ADD COLUMN IF NOT EXISTS refresh_expires_at BIGINT;

ALTER TABLE fx_user.user_sessions
    ADD COLUMN IF NOT EXISTS revoked_at BIGINT;

UPDATE fx_user.user_sessions SET token_type = 'Bearer' WHERE token_type IS NULL;
UPDATE fx_user.user_sessions SET grant_type = 'login' WHERE grant_type IS NULL;
UPDATE fx_user.user_sessions SET scope = 'http,socket' WHERE scope IS NULL;
UPDATE fx_user.user_sessions SET actor_type = 'end_user' WHERE actor_type IS NULL;
UPDATE fx_user.user_sessions SET refresh_expires_at = 0 WHERE refresh_expires_at IS NULL;
UPDATE fx_user.user_sessions SET revoked_at = 0 WHERE revoked_at IS NULL;

ALTER TABLE fx_game.games
    ADD COLUMN IF NOT EXISTS partner_code VARCHAR(64);

ALTER TABLE fx_game.games
    ADD COLUMN IF NOT EXISTS partner_game_id VARCHAR(128);

ALTER TABLE fx_game.game_accounts
    ADD COLUMN IF NOT EXISTS partner_user_id VARCHAR(128);

ALTER TABLE fx_game.game_accounts
    ADD COLUMN IF NOT EXISTS bind_source VARCHAR(32);

UPDATE fx_game.game_accounts SET bind_source = 'direct' WHERE bind_source IS NULL;
