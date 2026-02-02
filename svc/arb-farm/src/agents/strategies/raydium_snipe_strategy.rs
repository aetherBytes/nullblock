use async_trait::async_trait;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::RwLock;
use uuid::Uuid;

use super::{BehavioralStrategy, GraduationEvent, VenueSnapshot};
use crate::error::AppResult;
use crate::models::{Signal, SignalSignificance, SignalType, VenueType};

pub struct RaydiumSnipeStrategy {
    name: String,
    is_active: AtomicBool,
    event_buffer: RwLock<Vec<GraduationEvent>>,
}

impl RaydiumSnipeStrategy {
    pub fn new() -> Self {
        Self {
            name: "Raydium Snipe".to_string(),
            is_active: AtomicBool::new(true),
            event_buffer: RwLock::new(Vec::new()),
        }
    }

    pub async fn push_graduation(&self, event: GraduationEvent) {
        let mut buffer = self.event_buffer.write().await;
        buffer.push(event);
    }
}

impl Default for RaydiumSnipeStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BehavioralStrategy for RaydiumSnipeStrategy {
    fn strategy_type(&self) -> &str {
        "raydium_snipe"
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn supported_venues(&self) -> Vec<VenueType> {
        vec![VenueType::DexAmm]
    }

    async fn scan(&self, _snapshot: &VenueSnapshot) -> AppResult<Vec<Signal>> {
        let mut buffer = self.event_buffer.write().await;
        let events: Vec<GraduationEvent> = buffer.drain(..).collect();
        drop(buffer);

        let mut signals = Vec::new();

        for event in events {
            let signal = Signal {
                id: Uuid::new_v4(),
                signal_type: SignalType::CurveGraduation,
                venue_id: Uuid::nil(),
                venue_type: VenueType::DexAmm,
                token_mint: Some(event.mint.clone()),
                pool_address: event.raydium_pool.clone(),
                estimated_profit_bps: 500,
                confidence: 0.85,
                significance: SignalSignificance::Critical,
                metadata: serde_json::json!({
                    "signal_source": "raydium_snipe",
                    "symbol": event.symbol,
                    "name": event.name,
                    "raydium_pool": event.raydium_pool,
                    "last_progress": event.last_progress,
                    "progress_percent": 100.0,
                }),
                detected_at: chrono::Utc::now(),
                expires_at: chrono::Utc::now() + chrono::Duration::seconds(60),
            };

            tracing::info!(
                mint = %event.mint,
                symbol = %event.symbol,
                pool = ?event.raydium_pool,
                "🎓 RaydiumSnipe detected graduation: {}",
                event.symbol
            );

            signals.push(signal);
        }

        if !signals.is_empty() {
            tracing::info!(
                "🎓 RaydiumSnipeStrategy generated {} graduation signals",
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
