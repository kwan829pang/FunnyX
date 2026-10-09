//! Optional MongoDB notice history mirror (`MONGO_URL`).

use tracing::{info, warn};

use crate::config::Config;
use crate::models::OutboundNotice;
use crate::partner::notification_payload;

#[derive(Clone, Default)]
pub struct NoticeHistory {
    inner: Option<MongoInner>,
}

#[derive(Clone)]
struct MongoInner {
    db: String,
    client: mongodb::Client,
}

impl NoticeHistory {
    pub async fn connect(config: &Config) -> Self {
        let Some(url) = &config.mongo_url else {
            info!("MONGO_URL unset; notice history mirror disabled");
            return Self { inner: None };
        };
        match mongodb::Client::with_uri_str(url).await {
            Ok(client) => {
                info!(db = %config.mongo_db, "mongo notice history ready");
                Self {
                    inner: Some(MongoInner {
                        db: config.mongo_db.clone(),
                        client,
                    }),
                }
            }
            Err(e) => {
                warn!(error = %e, "mongo connect failed; history disabled");
                Self { inner: None }
            }
        }
    }

    pub async fn mirror_sent(&self, notice: &OutboundNotice, notification_id: i64) {
        let Some(inner) = &self.inner else {
            return;
        };
        let coll = inner
            .client
            .database(&inner.db)
            .collection::<mongodb::bson::Document>("notice_history");
        let payload = notification_payload(notice);
        let doc = mongodb::bson::doc! {
            "outbound_notice_id": notice.id,
            "notification_id": notification_id,
            "end_user_id": notice.end_user_id,
            "source_type": &notice.source_type,
            "source_id": notice.source_id,
            "event_type": &notice.event_type,
            "title": &notice.title,
            "body": &notice.body,
            "payload": mongodb::bson::to_bson(&payload).unwrap_or(mongodb::bson::Bson::Null),
            "sent_at_ms": chrono::Utc::now().timestamp_millis(),
        };
        if let Err(e) = coll.insert_one(doc, None).await {
            warn!(notice_id = notice.id, error = %e, "mongo notice_history insert failed");
        }
    }
}
