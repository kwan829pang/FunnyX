-- fx_config tables (1)
-- Required setting: base_fiat_currency = HKD | USD

CREATE TABLE IF NOT EXISTS fx_config.system_settings (
    id BIGSERIAL PRIMARY KEY,
    setting_key VARCHAR(64) NOT NULL UNIQUE,
    setting_value VARCHAR(128) NOT NULL,
    updated_by_admin_id BIGINT REFERENCES fx_admin.admin_users(id),
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL DEFAULT 0
);
