//! MongoDB `chat_messages` persistence (Client Center — required when MONGO_URL set).

use futures_util::TryStreamExt;
use mongodb::bson::{doc, Document};
use mongodb::options::{FindOptions, IndexOptions};
use mongodb::{Client, Collection, IndexModel};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;

use crate::config::Config;
use crate::db::now_ms;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    #[serde(rename = "_id")]
    pub id: String,
    pub client_id: i64,
    pub channel: String,
    pub sender: String,
    pub receiver: String,
    pub content: String,
    pub created_at: i64,
    pub status: String,
}

#[derive(Clone, Default)]
pub struct ChatStore {
    inner: Option<ChatInner>,
}

#[derive(Clone)]
struct ChatInner {
    coll: Collection<Document>,
}

impl ChatStore {
    pub async fn connect(config: &Config) -> Self {
        let Some(url) = &config.mongo_url else {
            info!("MONGO_URL unset; chat_messages disabled (Session chat APIs return 503)");
            return Self { inner: None };
        };
        match Client::with_uri_str(url).await {
            Ok(client) => {
                let coll = client
                    .database(&config.mongo_db)
                    .collection::<Document>("chat_messages");
                if let Err(e) = ensure_indexes(&coll).await {
                    warn!(error = %e, "chat_messages index ensure failed");
                }
                info!(db = %config.mongo_db, "mongo chat_messages ready");
                Self {
                    inner: Some(ChatInner { coll }),
                }
            }
            Err(e) => {
                warn!(error = %e, "mongo connect failed; chat disabled");
                Self { inner: None }
            }
        }
    }

    pub fn is_ready(&self) -> bool {
        self.inner.is_some()
    }

    pub async fn insert(
        &self,
        client_id: i64,
        channel: &str,
        sender: &str,
        receiver: &str,
        content: &str,
    ) -> Result<ChatMessage, String> {
        let inner = self.inner.as_ref().ok_or("mongo chat unavailable")?;
        let msg = ChatMessage {
            id: format!("msg_{}", Uuid::new_v4().simple()),
            client_id,
            channel: channel.to_string(),
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            content: content.to_string(),
            created_at: now_ms(),
            status: "sent".into(),
        };
        let document = doc! {
            "_id": &msg.id,
            "client_id": msg.client_id,
            "channel": &msg.channel,
            "sender": &msg.sender,
            "receiver": &msg.receiver,
            "content": &msg.content,
            "created_at": msg.created_at,
            "status": &msg.status,
        };
        inner
            .coll
            .insert_one(document, None)
            .await
            .map_err(|e| e.to_string())?;
        Ok(msg)
    }

    pub async fn list_for_client(
        &self,
        client_id: i64,
        channel: Option<&str>,
        limit: i64,
    ) -> Result<Vec<ChatMessage>, String> {
        let inner = self.inner.as_ref().ok_or("mongo chat unavailable")?;
        let lim = limit.clamp(1, 200);
        let filter = if let Some(ch) = channel.filter(|s| !s.is_empty()) {
            doc! { "client_id": client_id, "channel": ch }
        } else {
            doc! { "client_id": client_id }
        };
        let opts = FindOptions::builder()
            .sort(doc! { "created_at": -1 })
            .limit(lim)
            .build();
        let mut cursor = inner
            .coll
            .find(filter, opts)
            .await
            .map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        while let Some(d) = cursor.try_next().await.map_err(|e| e.to_string())? {
            out.push(doc_to_message(&d)?);
        }
        Ok(out)
    }
}

async fn ensure_indexes(coll: &Collection<Document>) -> Result<(), String> {
    let models = vec![
        IndexModel::builder()
            .keys(doc! { "client_id": 1 })
            .options(IndexOptions::builder().name("idx_client_id".to_string()).build())
            .build(),
        IndexModel::builder()
            .keys(doc! { "channel": 1 })
            .options(IndexOptions::builder().name("idx_channel".to_string()).build())
            .build(),
        IndexModel::builder()
            .keys(doc! { "created_at": -1 })
            .options(
                IndexOptions::builder()
                    .name("idx_created_at".to_string())
                    .build(),
            )
            .build(),
    ];
    coll.create_indexes(models, None)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn doc_to_message(d: &Document) -> Result<ChatMessage, String> {
    Ok(ChatMessage {
        id: d
            .get_str("_id")
            .map_err(|e| e.to_string())?
            .to_string(),
        client_id: d.get_i64("client_id").map_err(|e| e.to_string())?,
        channel: d.get_str("channel").map_err(|e| e.to_string())?.into(),
        sender: d.get_str("sender").map_err(|e| e.to_string())?.into(),
        receiver: d.get_str("receiver").map_err(|e| e.to_string())?.into(),
        content: d.get_str("content").map_err(|e| e.to_string())?.into(),
        created_at: d.get_i64("created_at").map_err(|e| e.to_string())?,
        status: d.get_str("status").unwrap_or("sent").into(),
    })
}
