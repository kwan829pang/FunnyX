-- fx_user.user_wallets (3rd table) — run after fx_game

CREATE TABLE IF NOT EXISTS fx_user.user_wallets (
    id BIGSERIAL PRIMARY KEY,
    game_account_id BIGINT NOT NULL REFERENCES fx_game.game_accounts(id),
    game_coin_id BIGINT NOT NULL REFERENCES fx_game.game_coins(id),
    available NUMERIC(28,8) NOT NULL DEFAULT 0,
    locked NUMERIC(28,8) NOT NULL DEFAULT 0,
    updated_at BIGINT NOT NULL DEFAULT 0,
    UNIQUE (game_account_id, game_coin_id)
);
