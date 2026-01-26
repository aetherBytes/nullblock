use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum KolEntityType {
    Wallet,
    TwitterHandle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KolEntity {
    pub id: Uuid,
    pub entity_type: KolEntityType,
    pub identifier: String,
    pub display_name: Option<String>,
    pub linked_wallet: Option<String>,
    pub trust_score: Decimal,
    pub total_trades_tracked: i32,
    pub profitable_trades: i32,
    pub avg_profit_percent: Option<Decimal>,
    pub max_drawdown: Option<Decimal>,
    pub copy_trading_enabled: bool,
    pub copy_config: CopyTradeConfig,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl KolEntity {
    pub fn new(entity_type: KolEntityType, identifier: String, display_name: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            entity_type,
            identifier,
            display_name,
            linked_wallet: None,
            trust_score: Decimal::new(500, 1),
            total_trades_tracked: 0,
            profitable_trades: 0,
            avg_profit_percent: None,
            max_drawdown: None,
            copy_trading_enabled: false,
            copy_config: CopyTradeConfig::default(),
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn win_rate(&self) -> f64 {
        if self.total_trades_tracked == 0 {
            return 0.0;
        }
        self.profitable_trades as f64 / self.total_trades_tracked as f64
    }

    pub fn update_trust_score(&mut self) {
        let base_score = 50.0;
        let win_rate_factor = self.win_rate() * 30.0;
        let trade_count_factor = (self.total_trades_tracked.min(100) as f64 / 100.0) * 10.0;
        let avg_profit_factor = self.avg_profit_percent
            .map(|p| {
                let p_f64: f64 = p.try_into().unwrap_or(0.0);
                (p_f64.min(50.0) / 50.0) * 10.0
            })
            .unwrap_or(0.0);
        let drawdown_penalty = self.max_drawdown
            .map(|d| {
                let d_f64: f64 = d.try_into().unwrap_or(0.0);
                (d_f64 / 100.0) * 20.0
            })
            .unwrap_or(0.0);

        let score = base_score + win_rate_factor + trade_count_factor + avg_profit_factor - drawdown_penalty;
        self.trust_score = Decimal::from_f64_retain(score.max(0.0).min(100.0))
            .unwrap_or(Decimal::new(500, 1));
        self.updated_at = Utc::now();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyTradeConfig {
    pub max_position_sol: Decimal,
    pub delay_ms: u64,
    pub min_trust_score: Decimal,
    pub copy_percentage: Decimal,
    pub token_whitelist: Option<Vec<String>>,
    pub token_blacklist: Option<Vec<String>>,
}

impl Default for CopyTradeConfig {
    fn default() -> Self {
        Self {
            max_position_sol: Decimal::new(5, 1),      // 0.5 SOL
            delay_ms: 500,
            min_trust_score: Decimal::new(600, 1),    // 60.0
            copy_percentage: Decimal::new(5, 1),       // 0.5 = 50% (matches CopyExecutorConfig)
            token_whitelist: None,
            token_blacklist: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum KolTradeType {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KolTrade {
    pub id: Uuid,
    pub entity_id: Uuid,
    pub tx_signature: String,
    pub trade_type: KolTradeType,
    pub token_mint: String,
    pub token_symbol: Option<String>,
    pub amount_sol: Decimal,
    pub token_amount: Option<Decimal>,
    pub price_at_trade: Option<Decimal>,
    pub detected_at: DateTime<Utc>,
}

impl KolTrade {
    pub fn new(
        entity_id: Uuid,
        tx_signature: String,
        trade_type: KolTradeType,
        token_mint: String,
        amount_sol: Decimal,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            entity_id,
            tx_signature,
            trade_type,
            token_mint,
            token_symbol: None,
            amount_sol,
            token_amount: None,
            price_at_trade: None,
            detected_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CopyTradeStatus {
    Pending,
    Executing,
    Executed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyTrade {
    pub id: Uuid,
    pub entity_id: Uuid,
    pub kol_trade_id: Uuid,
    pub our_tx_signature: Option<String>,
    pub copy_amount_sol: Decimal,
    pub delay_ms: u64,
    pub profit_loss_lamports: Option<i64>,
    pub status: CopyTradeStatus,
    pub skip_reason: Option<String>,
    pub executed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl CopyTrade {
    pub fn new(entity_id: Uuid, kol_trade_id: Uuid, copy_amount_sol: Decimal, delay_ms: u64) -> Self {
        Self {
            id: Uuid::new_v4(),
            entity_id,
            kol_trade_id,
            our_tx_signature: None,
            copy_amount_sol,
            delay_ms,
            profit_loss_lamports: None,
            status: CopyTradeStatus::Pending,
            skip_reason: None,
            executed_at: None,
            created_at: Utc::now(),
        }
    }

    pub fn mark_executed(&mut self, tx_signature: String) {
        self.our_tx_signature = Some(tx_signature);
        self.status = CopyTradeStatus::Executed;
        self.executed_at = Some(Utc::now());
    }

    pub fn mark_failed(&mut self, reason: String) {
        self.status = CopyTradeStatus::Failed;
        self.skip_reason = Some(reason);
    }

    pub fn mark_skipped(&mut self, reason: String) {
        self.status = CopyTradeStatus::Skipped;
        self.skip_reason = Some(reason);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KolStats {
    pub entity_id: Uuid,
    pub display_name: Option<String>,
    pub identifier: String,
    pub trust_score: Decimal,
    pub total_trades: i32,
    pub profitable_trades: i32,
    pub win_rate: f64,
    pub avg_profit_percent: Option<Decimal>,
    pub max_drawdown_percent: Option<Decimal>,
    pub total_volume_sol: Decimal,
    pub our_copy_count: i32,
    pub our_copy_profit_sol: Decimal,
    pub copy_trading_enabled: bool,
    pub last_trade_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustScoreBreakdown {
    pub base_score: f64,
    pub win_rate_factor: f64,
    pub trade_count_factor: f64,
    pub avg_profit_factor: f64,
    pub drawdown_penalty: f64,
    pub final_score: f64,
}

impl KolEntity {
    pub fn trust_score_breakdown(&self) -> TrustScoreBreakdown {
        let base_score = 50.0;
        let win_rate_factor = self.win_rate() * 30.0;
        let trade_count_factor = (self.total_trades_tracked.min(100) as f64 / 100.0) * 10.0;
        let avg_profit_factor = self.avg_profit_percent
            .map(|p| {
                let p_f64: f64 = p.try_into().unwrap_or(0.0);
                (p_f64.min(50.0) / 50.0) * 10.0
            })
            .unwrap_or(0.0);
        let drawdown_penalty = self.max_drawdown
            .map(|d| {
                let d_f64: f64 = d.try_into().unwrap_or(0.0);
                (d_f64 / 100.0) * 20.0
            })
            .unwrap_or(0.0);

        TrustScoreBreakdown {
            base_score,
            win_rate_factor,
            trade_count_factor,
            avg_profit_factor,
            drawdown_penalty,
            final_score: (base_score + win_rate_factor + trade_count_factor + avg_profit_factor - drawdown_penalty)
                .max(0.0)
                .min(100.0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddKolRequest {
    pub wallet_address: Option<String>,
    pub twitter_handle: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateKolRequest {
    pub display_name: Option<String>,
    pub linked_wallet: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnableCopyRequest {
    pub max_position_sol: Option<Decimal>,
    pub delay_ms: Option<u64>,
    pub min_trust_score: Option<Decimal>,
    pub copy_percentage: Option<Decimal>,
    pub token_whitelist: Option<Vec<String>>,
    pub token_blacklist: Option<Vec<String>>,
}
