use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use chrono::Utc;
use tokio::sync::RwLock;

use crate::models::{
    BalanceResponse, GameItem, MoneyRequest, MoneyResponse, TransferRequest, TransferResponse,
    TransferView, UserInfoResponse,
};
use crate::sim_status::SimStatus;

#[derive(Debug, Clone)]
pub struct DemoUser {
    pub partner_user_id: String,
    pub username: String,
    pub game_id: String,
    pub game_account_id: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct TransferRecord {
    pub transfer_id: String,
    pub request_id: String,
    pub status: String,
    pub asset_type: String,
    pub from_game_account_id: String,
    pub to_game_account_id: String,
    pub item_ref_id: Option<String>,
    pub amount: f64,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

impl TransferRecord {
    pub fn to_view(&self) -> TransferView {
        TransferView {
            transfer_id: self.transfer_id.clone(),
            request_id: self.request_id.clone(),
            status: self.status.clone(),
            asset_type: self.asset_type.clone(),
            from_game_account_id: self.from_game_account_id.clone(),
            to_game_account_id: self.to_game_account_id.clone(),
            item_ref_id: self.item_ref_id.clone(),
            amount: self.amount,
            created_at: self.created_at,
            updated_at: self.updated_at,
            source: "test".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MoneyRecord {
    pub partner_txn_id: String,
    pub request_id: String,
    pub transaction_type: String,
    pub status: String,
    pub amount: f64,
    pub game_coin: String,
    pub game_account_id: String,
}

#[derive(Clone)]
pub struct AppState {
    pub partner_id: String,
    pub company_name: String,
    pub public_base_url: String,
    pub api_key: String,
    pub client_id: String,
    pub client_secret: String,
    seq: Arc<AtomicU64>,
    users: Arc<RwLock<HashMap<String, DemoUser>>>,
    /// game_account_id -> items
    items: Arc<RwLock<HashMap<String, Vec<GameItem>>>>,
    /// (game_account_id, game_coin) -> balance
    balances: Arc<RwLock<HashMap<(String, String), f64>>>,
    /// auth_code -> partner_user_id
    oauth_codes: Arc<RwLock<HashMap<String, String>>>,
    /// access_token -> partner_user_id
    oauth_tokens: Arc<RwLock<HashMap<String, String>>>,
    transfers: Arc<RwLock<HashMap<String, TransferRecord>>>,
    money_txns: Arc<RwLock<HashMap<String, MoneyRecord>>>,
}

impl AppState {
    pub fn new(
        partner_id: String,
        company_name: String,
        public_base_url: String,
        api_key: String,
        client_id: String,
        client_secret: String,
    ) -> Self {
        let mut users = HashMap::new();
        users.insert(
            "pu_9001".into(),
            DemoUser {
                partner_user_id: "pu_9001".into(),
                username: "alice_01".into(),
                game_id: "game_001".into(),
                game_account_id: "player_9001".into(),
                password: "demo".into(),
            },
        );
        users.insert(
            "pu_9002".into(),
            DemoUser {
                partner_user_id: "pu_9002".into(),
                username: "bob_02".into(),
                game_id: "game_001".into(),
                game_account_id: "player_9002".into(),
                password: "demo".into(),
            },
        );

        let mut items = HashMap::new();
        items.insert(
            "player_9001".into(),
            vec![
                GameItem {
                    item_ref_id: "sword_01".into(),
                    name: "Iron Sword".into(),
                    qty: 1,
                    game_account_id: "player_9001".into(),
                },
                GameItem {
                    item_ref_id: "shield_02".into(),
                    name: "Wooden Shield".into(),
                    qty: 1,
                    game_account_id: "player_9001".into(),
                },
            ],
        );
        items.insert(
            "player_9002".into(),
            vec![GameItem {
                item_ref_id: "potion_03".into(),
                name: "Health Potion".into(),
                qty: 5,
                game_account_id: "player_9002".into(),
            }],
        );

        let mut balances = HashMap::new();
        balances.insert(("player_9001".into(), "GCA".into()), 10_000.0);
        balances.insert(("player_9001".into(), "PLT".into()), 500.0);
        balances.insert(("player_9002".into(), "GCA".into()), 8_000.0);

        Self {
            partner_id,
            company_name,
            public_base_url,
            api_key,
            client_id,
            client_secret,
            seq: Arc::new(AtomicU64::new(1000)),
            users: Arc::new(RwLock::new(users)),
            items: Arc::new(RwLock::new(items)),
            balances: Arc::new(RwLock::new(balances)),
            oauth_codes: Arc::new(RwLock::new(HashMap::new())),
            oauth_tokens: Arc::new(RwLock::new(HashMap::new())),
            transfers: Arc::new(RwLock::new(HashMap::new())),
            money_txns: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn next_id(&self, prefix: &str) -> String {
        let n = self.seq.fetch_add(1, Ordering::Relaxed);
        format!("{prefix}-{n}")
    }

    pub async fn find_user_by_username(&self, username: &str) -> Option<DemoUser> {
        self.users
            .read()
            .await
            .values()
            .find(|u| u.username == username)
            .cloned()
    }

    pub async fn find_user(&self, partner_user_id: &str) -> Option<DemoUser> {
        self.users.read().await.get(partner_user_id).cloned()
    }

    /// Confirm player exists and is on `game_id` (partner-side id, e.g. game_001).
    pub async fn lookup_player(
        &self,
        game_id: &str,
        username: Option<&str>,
        partner_user_id: Option<&str>,
        _platform_end_user_id: Option<i64>,
    ) -> Option<DemoUser> {
        let user = if let Some(puid) = partner_user_id.filter(|s| !s.is_empty()) {
            self.find_user(puid).await?
        } else if let Some(name) = username.filter(|s| !s.is_empty()) {
            self.find_user_by_username(name).await?
        } else {
            return None;
        };
        if user.game_id != game_id {
            return None;
        }
        Some(user)
    }

    pub async fn issue_auth_code(&self, partner_user_id: &str) -> String {
        let code = format!("code_{}", uuid::Uuid::new_v4().simple());
        self.oauth_codes
            .write()
            .await
            .insert(code.clone(), partner_user_id.to_string());
        code
    }

    pub async fn exchange_code(&self, code: &str) -> Option<(String, String)> {
        let partner_user_id = self.oauth_codes.write().await.remove(code)?;
        let token = format!("atk_{}", uuid::Uuid::new_v4().simple());
        self.oauth_tokens
            .write()
            .await
            .insert(token.clone(), partner_user_id.clone());
        Some((token, partner_user_id))
    }

    pub async fn userinfo_from_token(&self, token: &str) -> Option<UserInfoResponse> {
        let partner_user_id = self.oauth_tokens.read().await.get(token)?.clone();
        let user = self.find_user(&partner_user_id).await?;
        Some(UserInfoResponse {
            partner_user_id: user.partner_user_id,
            game_id: user.game_id,
            game_account_id: user.game_account_id,
            username: user.username,
            status: "active".into(),
            source: "test".into(),
        })
    }

    pub async fn list_items(&self, game_account_id: &str) -> Vec<GameItem> {
        self.items
            .read()
            .await
            .get(game_account_id)
            .cloned()
            .unwrap_or_default()
    }

    pub async fn balance(&self, game_account_id: &str, game_coin: &str) -> BalanceResponse {
        let bal = self
            .balances
            .read()
            .await
            .get(&(game_account_id.to_string(), game_coin.to_string()))
            .copied()
            .unwrap_or(0.0);
        BalanceResponse {
            game_account_id: game_account_id.into(),
            game_coin: game_coin.into(),
            balance: bal,
            source: "test".into(),
        }
    }

    pub async fn create_transfer(&self, req: TransferRequest) -> TransferRecord {
        let now = Utc::now();
        let transfer_id = self.next_id("XFER");
        let record = TransferRecord {
            transfer_id: transfer_id.clone(),
            request_id: req.request_id,
            status: "pending".into(),
            asset_type: req.asset_type,
            from_game_account_id: req.from_game_account_id,
            to_game_account_id: req.to_game_account_id,
            item_ref_id: req.item_ref_id,
            amount: req.amount,
            created_at: req.created_at.unwrap_or(now),
            updated_at: now,
        };
        self.transfers
            .write()
            .await
            .insert(transfer_id, record.clone());
        record
    }

    pub async fn apply_transfer_status(
        &self,
        transfer_id: &str,
        sim: SimStatus,
    ) -> anyhow::Result<TransferRecord> {
        let snapshot = {
            let guard = self.transfers.read().await;
            guard
                .get(transfer_id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("transfer not found"))?
        };
        if snapshot.status != "pending" {
            anyhow::bail!("transfer already finalized as '{}'", snapshot.status);
        }
        let status = sim.txn_status();
        if status == "success" {
            self.move_assets(&snapshot).await?;
        }
        let mut guard = self.transfers.write().await;
        let record = guard
            .get_mut(transfer_id)
            .ok_or_else(|| anyhow::anyhow!("transfer not found"))?;
        record.status = status.into();
        record.updated_at = Utc::now();
        Ok(record.clone())
    }

    async fn move_assets(&self, record: &TransferRecord) -> anyhow::Result<()> {
        match record.asset_type.as_str() {
            "game_item" => {
                let item_ref = record
                    .item_ref_id
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("item_ref_id required for game_item"))?;
                let mut items = self.items.write().await;
                let from = items
                    .get_mut(&record.from_game_account_id)
                    .ok_or_else(|| anyhow::anyhow!("from account has no inventory"))?;
                let idx = from
                    .iter()
                    .position(|i| &i.item_ref_id == item_ref)
                    .ok_or_else(|| anyhow::anyhow!("item_ref_id not found on from account"))?;
                let mut item = from.remove(idx);
                item.game_account_id = record.to_game_account_id.clone();
                items
                    .entry(record.to_game_account_id.clone())
                    .or_default()
                    .push(item);
            }
            "game_coin" | "company_coin" => {
                let coin = "GCA";
                let mut bals = self.balances.write().await;
                let from_key = (record.from_game_account_id.clone(), coin.into());
                let to_key = (record.to_game_account_id.clone(), coin.into());
                let from_bal = bals.get(&from_key).copied().unwrap_or(0.0);
                if from_bal < record.amount {
                    anyhow::bail!("insufficient balance");
                }
                bals.insert(from_key, from_bal - record.amount);
                let to_bal = bals.get(&to_key).copied().unwrap_or(0.0);
                bals.insert(to_key, to_bal + record.amount);
            }
            other => anyhow::bail!("unsupported asset_type '{other}'"),
        }
        Ok(())
    }

    pub async fn get_transfer(&self, transfer_id: &str) -> Option<TransferRecord> {
        self.transfers.read().await.get(transfer_id).cloned()
    }

    pub async fn create_money(&self, req: MoneyRequest) -> MoneyRecord {
        let partner_txn_id = self.next_id("PTXN");
        let record = MoneyRecord {
            partner_txn_id: partner_txn_id.clone(),
            request_id: req.request_id,
            transaction_type: req.transaction_type,
            status: "pending".into(),
            amount: req.amount,
            game_coin: req.game_coin,
            game_account_id: req.game_account_id,
        };
        self.money_txns
            .write()
            .await
            .insert(partner_txn_id, record.clone());
        record
    }

    pub async fn apply_money_status(
        &self,
        partner_txn_id: &str,
        sim: SimStatus,
    ) -> anyhow::Result<MoneyRecord> {
        let snapshot = {
            let guard = self.money_txns.read().await;
            guard
                .get(partner_txn_id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("money txn not found"))?
        };
        if snapshot.status != "pending" {
            anyhow::bail!("money txn already finalized as '{}'", snapshot.status);
        }
        let status = sim.txn_status();
        if status == "success" {
            let mut bals = self.balances.write().await;
            let key = (snapshot.game_account_id.clone(), snapshot.game_coin.clone());
            let cur = bals.get(&key).copied().unwrap_or(0.0);
            let next = if snapshot.transaction_type == "withdrawal" {
                if cur < snapshot.amount {
                    anyhow::bail!("insufficient balance for withdrawal");
                }
                cur - snapshot.amount
            } else {
                cur + snapshot.amount
            };
            bals.insert(key, next);
        }
        let mut guard = self.money_txns.write().await;
        let record = guard
            .get_mut(partner_txn_id)
            .ok_or_else(|| anyhow::anyhow!("money txn not found"))?;
        record.status = status.into();
        Ok(record.clone())
    }

    pub fn transfer_response(
        record: &TransferRecord,
        sim_status: &str,
        sim_delay_ms: u64,
    ) -> TransferResponse {
        TransferResponse {
            request_id: record.request_id.clone(),
            transfer_id: record.transfer_id.clone(),
            status: "pending".into(),
            sim_status: sim_status.into(),
            sim_delay_ms,
            asset_type: record.asset_type.clone(),
            item_ref_id: record.item_ref_id.clone(),
            amount: record.amount,
            source: "test".into(),
        }
    }

    pub fn money_response(
        record: &MoneyRecord,
        sim_status: &str,
        sim_delay_ms: u64,
    ) -> MoneyResponse {
        MoneyResponse {
            request_id: record.request_id.clone(),
            partner_txn_id: record.partner_txn_id.clone(),
            transaction_type: record.transaction_type.clone(),
            status: "pending".into(),
            sim_status: sim_status.into(),
            sim_delay_ms,
            amount: record.amount,
            game_coin: record.game_coin.clone(),
            source: "test".into(),
        }
    }
}
