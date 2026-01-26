use async_trait::async_trait;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use uuid::Uuid;

use crate::database::repositories::KolRepository;
use crate::error::AppResult;
use crate::models::{Signal, SignalType, SignalSignificance, VenueType};
use super::{BehavioralStrategy, VenueSnapshot};

pub struct KolCopyStrategy {
    name: String,
    is_active: AtomicBool,
    kol_repo: Arc<KolRepository>,
    min_trust_score: f64,
    copy_delay_ms: u64,
    max_position_sol: f64,
}

impl KolCopyStrategy {
    pub fn new(kol_repo: Arc<KolRepository>) -> Self {
        Self {
            name: "KOL Copy Trading".to_string(),
            is_active: AtomicBool::new(true),
            kol_repo,
            min_trust_score: 60.0,
            copy_delay_ms: 500,
            max_position_sol: 0.5,
        }
    }

    pub fn with_min_trust_score(mut self, score: f64) -> Self {
        self.min_trust_score = score;
        self
    }

    pub fn with_copy_delay(mut self, delay_ms: u64) -> Self {
        self.copy_delay_ms = delay_ms;
        self
    }

    pub fn with_max_position(mut self, max_sol: f64) -> Self {
        self.max_position_sol = max_sol;
        self
    }

    fn create_kol_signal(
        &self,
        kol_id: Uuid,
        kol_identifier: &str,
        kol_trust_score: f64,
        trade: &crate::database::repositories::kol::KolTradeRecord,
    ) -> Signal {
        let confidence = (kol_trust_score / 100.0).min(1.0);

        let significance = if kol_trust_score >= 80.0 {
            SignalSignificance::High
        } else if kol_trust_score >= 60.0 {
            SignalSignificance::Medium
        } else {
            SignalSignificance::Low
        };

        Signal {
            id: Uuid::new_v4(),
            signal_type: SignalType::KolTrade,
            venue_id: Uuid::nil(),
            venue_type: VenueType::DexAmm,
            token_mint: Some(trade.token_mint.clone()),
            pool_address: None,
            estimated_profit_bps: 100,
            confidence,
            significance,
            metadata: serde_json::json!({
                "kol_id": kol_id.to_string(),
                "kol_identifier": kol_identifier,
                "kol_trust_score": kol_trust_score,
                "kol_trade_id": trade.id.to_string(),
                "trade_type": trade.trade_type,
                "tx_signature": trade.tx_signature,
                "amount_sol": trade.amount_sol,
                "token_symbol": trade.token_symbol,
                "detected_at": trade.detected_at.to_rfc3339(),
                "strategy": "kol_copy",
                "copy_delay_ms": self.copy_delay_ms,
                "max_position_sol": self.max_position_sol,
            }),
            detected_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + chrono::Duration::minutes(2),
        }
    }
}

#[async_trait]
impl BehavioralStrategy for KolCopyStrategy {
    fn strategy_type(&self) -> &str {
        "copy_trade"
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn supported_venues(&self) -> Vec<VenueType> {
        vec![VenueType::DexAmm, VenueType::BondingCurve]
    }

    async fn scan(&self, _snapshot: &VenueSnapshot) -> AppResult<Vec<Signal>> {
        let mut signals = Vec::new();

        let kols = match self.kol_repo.get_entities_for_copy(self.min_trust_score).await {
            Ok(kols) => kols,
            Err(e) => {
                tracing::warn!("Failed to get copy-enabled KOLs: {}", e);
                return Ok(signals);
            }
        };

        for kol in kols {
            let trust_score: f64 = kol.trust_score.try_into().unwrap_or(50.0);

            let pending_trades = match self.kol_repo.get_pending_copy_trades(kol.id).await {
                Ok(trades) => trades,
                Err(e) => {
                    tracing::warn!("Failed to get pending trades for KOL {}: {}", kol.id, e);
                    continue;
                }
            };

            for trade in pending_trades {
                let signal = self.create_kol_signal(
                    kol.id,
                    &kol.identifier,
                    trust_score,
                    &trade,
                );
                signals.push(signal);
            }
        }

        if !signals.is_empty() {
            tracing::info!(
                "KolCopyStrategy found {} pending copy opportunities",
                signals.len()
            );
        }

        Ok(signals)
    }

    fn is_active(&self) -> bool {
        self.is_active.load(Ordering::SeqCst)
    }

    async fn set_active(&self, active: bool) {
        self.is_active.store(active, Ordering::SeqCst);
    }
}
