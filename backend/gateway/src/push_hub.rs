//! Live Session client writers for Message Center → Client Web push.

use std::collections::HashMap;
use std::sync::Arc;

use serde::Serialize;
use tokio::sync::{mpsc, RwLock};

pub type PushTx = mpsc::UnboundedSender<String>;

#[derive(Clone, Default)]
pub struct PushHub {
    inner: Arc<RwLock<Inner>>,
}

#[derive(Default)]
struct Inner {
    /// connection_id → (end_user_id, tx)
    by_conn: HashMap<String, (i64, PushTx)>,
    /// end_user_id → connection_ids
    by_user: HashMap<i64, Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PushEnvelope {
    pub r#type: &'static str,
    pub notification_id: i64,
    pub end_user_id: i64,
    pub source_type: String,
    pub event_type: String,
    pub title: Option<String>,
    pub body: Option<String>,
    pub payload: serde_json::Value,
}

impl PushHub {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn register(&self, connection_id: String, end_user_id: i64, tx: PushTx) {
        let mut g = self.inner.write().await;
        g.by_conn
            .insert(connection_id.clone(), (end_user_id, tx));
        g.by_user
            .entry(end_user_id)
            .or_default()
            .push(connection_id);
    }

    pub async fn unregister(&self, connection_id: &str) {
        let mut g = self.inner.write().await;
        if let Some((uid, _)) = g.by_conn.remove(connection_id) {
            if let Some(list) = g.by_user.get_mut(&uid) {
                list.retain(|id| id != connection_id);
                if list.is_empty() {
                    g.by_user.remove(&uid);
                }
            }
        }
    }

    /// Fan-out JSON text to all sockets for `end_user_id`. Returns delivered count.
    pub async fn push_json(&self, end_user_id: i64, json: &str) -> usize {
        let g = self.inner.read().await;
        let Some(conns) = g.by_user.get(&end_user_id) else {
            return 0;
        };
        let mut n = 0;
        for cid in conns {
            if let Some((_, tx)) = g.by_conn.get(cid) {
                if tx.send(json.to_string()).is_ok() {
                    n += 1;
                }
            }
        }
        n
    }

    pub async fn push_envelope(&self, env: &PushEnvelope) -> Result<usize, String> {
        let json = serde_json::to_string(env).map_err(|e| e.to_string())?;
        Ok(self.push_json(env.end_user_id, &json).await)
    }
}
