-- fx_market_data tables (1)
-- Candle / bar aggregates derived from fx_market.trades (and live engine updates)
-- Source of truth for historical OHLC charts; Redis ticker remains ephemeral live snapshot
-- No trade_id FK: bars are aggregated from prints / engine candle updates, not row-linked.

CREATE TABLE IF NOT EXISTS fx_market_data.market_ohlcvs (
    id BIGSERIAL PRIMARY KEY,
    market_pair_id BIGINT NOT NULL REFERENCES fx_market.market_pairs(id),
    timeframe VARCHAR(8) NOT NULL
        CHECK (timeframe IN ('15m', '1h')),
    open_time BIGINT NOT NULL,
    close_time BIGINT NOT NULL,
    open NUMERIC(28,8) NOT NULL,
    high NUMERIC(28,8) NOT NULL,
    low NUMERIC(28,8) NOT NULL,
    close NUMERIC(28,8) NOT NULL,
    volume NUMERIC(28,8) NOT NULL DEFAULT 0,
    quote_volume NUMERIC(28,8) NOT NULL DEFAULT 0,
    trade_count INT NOT NULL DEFAULT 0,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL DEFAULT 0,
    UNIQUE (market_pair_id, timeframe, open_time)
);
