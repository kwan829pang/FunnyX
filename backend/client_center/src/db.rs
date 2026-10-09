use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn connect(database_url: &str) -> anyhow::Result<PgPool> {
    Ok(PgPoolOptions::new()
        .max_connections(8)
        .connect(database_url)
        .await?)
}

pub fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}
