// KOL Copy Trading Strategy (WIP Stub)
//
// This strategy will unify KOL copy trading into the standard
// Scanner → StrategyEngine → Executor pipeline. Currently, KOL copy trading
// bypasses this pipeline entirely:
//
//   Current flow (bypass):
//     Helius Webhook → event_tx(kol::TRADE_DETECTED)
//       → AutonomousExecutor.handle_kol_trade()
//       → CopyTradeExecutor.execute_copy() → direct buy/sell
//
//   Intended flow (unified pipeline):
//     Helius Webhook → event_tx(kol::TRADE_DETECTED)
//       → KolCopyStrategy.push_trade() (buffers the event)
//       → Scanner calls strategy.scan() on next cycle
//       → scan() drains buffer → Vec<Signal(KolTrade)>
//       → StrategyEngine.process_signals() → Edge
//       → AutonomousExecutor.handle_edge_detected()
//
// Why a buffer pattern?
//   KOL trades arrive asynchronously via webhooks (push-based), but the Scanner
//   operates on a polling loop (pull-based). The buffer bridges this gap: webhook
//   events are pushed into an internal Vec, and scan() drains it each cycle.
//   This avoids creating a new VenueType for KOL data — it's not a venue (market
//   data source), it's wallet activity from tracked traders.
//
// What needs to change when wiring this up:
//   1. webhooks.rs: After recording the KOL trade, call
//      kol_copy_strategy.push_trade(event) instead of (or in addition to)
//      emitting kol_topics::TRADE_DETECTED for the bypass path.
//   2. server.rs: Register KolCopyStrategy with the Scanner's StrategyRegistry
//      and create a matching DB strategy record (strategy_type = "copy_trade").
//   3. strategy_engine.rs: Add a "copy_trade" match arm in
//      signal_matches_strategy() to filter by min trust score, token
//      whitelist/blacklist, etc.
//   4. autonomous_executor.rs: Remove (or gate behind a flag) the direct
//      handle_kol_trade() bypass once the unified pipeline is validated.

use async_trait::async_trait;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::AppResult;
use crate::models::{Signal, SignalType, SignalSignificance, VenueType};
use super::{BehavioralStrategy, VenueSnapshot};

// TODO: Import or define this struct to match the webhook payload shape.
// For now this mirrors the fields that webhooks.rs extracts from Helius
// enhanced transaction data.
#[derive(Debug, Clone)]
pub struct KolTradeEvent {
    pub kol_wallet: String,
    pub token_mint: String,
    pub trade_direction: KolTradeDirection,
    pub amount_sol: f64,
    pub trust_score: f64,
    pub kol_name: Option<String>,
    pub tx_signature: String,
    pub detected_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum KolTradeDirection {
    Buy,
    Sell,
}

pub struct KolCopyStrategy {
    name: String,
    is_active: AtomicBool,
    // Buffer for webhook-pushed KOL trade events. The Scanner's polling loop
    // calls scan() which drains this buffer and converts events to Signals.
    trade_buffer: RwLock<Vec<KolTradeEvent>>,
    min_trust_score: f64,
}

impl KolCopyStrategy {
    pub fn new() -> Self {
        Self {
            name: "KOL Copy Trading".to_string(),
            is_active: AtomicBool::new(false), // disabled by default (observation mode)
            trade_buffer: RwLock::new(Vec::new()),
            min_trust_score: 60.0,
        }
    }

    // Called by the webhook handler (webhooks.rs) to push a KOL trade event
    // into the buffer. This is the bridge between push-based webhooks and
    // pull-based scanner polling.
    //
    // TODO: Wire this up in webhooks.rs after the KOL trade is recorded in DB.
    // Replace or supplement the current kol_topics::TRADE_DETECTED event emission.
    pub async fn push_trade(&self, event: KolTradeEvent) {
        let mut buffer = self.trade_buffer.write().await;
        buffer.push(event);
    }
}

impl Default for KolCopyStrategy {
    fn default() -> Self {
        Self::new()
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
        // KOL trades can happen on both bonding curves and DEXes
        vec![VenueType::BondingCurve, VenueType::DexAmm]
    }

    // scan() drains the internal trade buffer and converts each KOL trade event
    // into a Signal. The VenueSnapshot parameter is unused — KOL signals come
    // from webhook pushes, not from venue polling data.
    //
    // TODO: When implementing for real:
    //   - Filter by min_trust_score
    //   - Apply per-KOL token whitelist/blacklist from CopyTradeConfig
    //   - Set venue_type based on whether the token has graduated
    //   - Calculate confidence from KOL trust score + recent win rate
    //   - Set appropriate expiry (KOL trades are time-sensitive, ~30s TTL)
    async fn scan(&self, _snapshot: &VenueSnapshot) -> AppResult<Vec<Signal>> {
        let mut buffer = self.trade_buffer.write().await;
        let events: Vec<KolTradeEvent> = buffer.drain(..).collect();
        drop(buffer);

        let mut signals = Vec::new();

        for event in events {
            if event.trust_score < self.min_trust_score {
                continue;
            }

            // TODO: Look up CopyTradeConfig for this KOL to check
            // token whitelist/blacklist and per-KOL copy percentage.

            // TODO: Determine venue_type based on token graduation status.
            // For now default to BondingCurve since most KOL trades on
            // pump.fun are pre-graduation.
            let venue_type = VenueType::BondingCurve;

            let confidence = (event.trust_score / 100.0).min(1.0);

            let significance = if event.trust_score >= 90.0 {
                SignalSignificance::Critical
            } else if event.trust_score >= 75.0 {
                SignalSignificance::High
            } else {
                SignalSignificance::Medium
            };

            let signal = Signal {
                id: Uuid::new_v4(),
                signal_type: SignalType::KolTrade,
                venue_id: Uuid::nil(), // KOL trades don't originate from a polled venue
                venue_type,
                token_mint: Some(event.token_mint.clone()),
                pool_address: None,
                estimated_profit_bps: 500, // TODO: estimate from KOL historical performance
                confidence,
                significance,
                metadata: serde_json::json!({
                    "signal_source": "kol_copy",
                    "kol_wallet": event.kol_wallet,
                    "kol_name": event.kol_name,
                    "trade_direction": format!("{:?}", event.trade_direction),
                    "amount_sol": event.amount_sol,
                    "trust_score": event.trust_score,
                    "tx_signature": event.tx_signature,
                }),
                detected_at: event.detected_at,
                // KOL trades are time-sensitive — 30 second TTL
                expires_at: event.detected_at + chrono::Duration::seconds(30),
            };

            signals.push(signal);
        }

        if !signals.is_empty() {
            tracing::info!(
                "KolCopyStrategy drained {} KOL trade signals from buffer",
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
