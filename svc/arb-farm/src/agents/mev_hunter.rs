use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use crate::error::AppResult;
use crate::events::{AgentType, ArbEvent, AtomicityLevel, EventSource, Significance};
use crate::models::{Signal, SignalType, VenueType};
use crate::venues::{MevVenue, ProfitEstimate, Quote, QuoteParams};

pub struct MevHunter {
    id: Uuid,
    event_tx: broadcast::Sender<ArbEvent>,
    dex_venues: Arc<RwLock<Vec<Arc<dyn MevVenue>>>>,
    lending_venues: Arc<RwLock<Vec<Arc<dyn MevVenue>>>>,
    pending_transactions: Arc<RwLock<Vec<PendingTransaction>>>,
    config: MevHunterConfig,
}

#[derive(Debug, Clone)]
pub struct MevHunterConfig {
    pub min_arb_profit_bps: i32,
    pub min_liquidation_profit_bps: i32,
    pub min_jit_profit_bps: i32,
    pub min_backrun_profit_bps: i32,
    pub max_gas_lamports: u64,
    pub arb_pairs: Vec<ArbPair>,
}

impl Default for MevHunterConfig {
    fn default() -> Self {
        Self {
            min_arb_profit_bps: 20,
            min_liquidation_profit_bps: 50,
            min_jit_profit_bps: 10,
            min_backrun_profit_bps: 15,
            max_gas_lamports: 100_000,
            arb_pairs: default_arb_pairs(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArbPair {
    pub token_a: String,
    pub token_b: String,
    pub name: String,
}

fn default_arb_pairs() -> Vec<ArbPair> {
    vec![
        ArbPair {
            token_a: "So11111111111111111111111111111111111111112".to_string(), // SOL
            token_b: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
            name: "SOL/USDC".to_string(),
        },
        ArbPair {
            token_a: "So11111111111111111111111111111111111111112".to_string(), // SOL
            token_b: "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string(), // USDT
            name: "SOL/USDT".to_string(),
        },
        ArbPair {
            token_a: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
            token_b: "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string(), // USDT
            name: "USDC/USDT".to_string(),
        },
    ]
}

#[derive(Debug, Clone)]
pub struct PendingTransaction {
    pub tx_signature: String,
    pub from_address: String,
    pub to_address: String,
    pub token_mint: Option<String>,
    pub amount_lamports: u64,
    pub detected_at: chrono::DateTime<chrono::Utc>,
    pub tx_type: PendingTxType,
}

#[derive(Debug, Clone)]
pub enum PendingTxType {
    Swap { input_mint: String, output_mint: String, expected_output: u64 },
    AddLiquidity { pool_address: String, amount_a: u64, amount_b: u64 },
    RemoveLiquidity { pool_address: String },
    Unknown,
}

#[derive(Debug, Clone)]
pub struct DexArbOpportunity {
    pub id: Uuid,
    pub pair_name: String,
    pub buy_venue: String,
    pub sell_venue: String,
    pub buy_price: f64,
    pub sell_price: f64,
    pub spread_bps: i32,
    pub estimated_profit_lamports: i64,
    pub route: ArbRoute,
    pub atomicity: AtomicityLevel,
    pub detected_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct ArbRoute {
    pub legs: Vec<ArbLeg>,
    pub total_input_lamports: u64,
    pub total_output_lamports: u64,
}

#[derive(Debug, Clone)]
pub struct ArbLeg {
    pub venue: String,
    pub action: String,
    pub input_mint: String,
    pub output_mint: String,
    pub input_amount: u64,
    pub expected_output: u64,
}

#[derive(Debug, Clone)]
pub struct JitOpportunity {
    pub id: Uuid,
    pub pool_address: String,
    pub token_a_mint: String,
    pub token_b_mint: String,
    pub pending_swap_amount: u64,
    pub optimal_liquidity_amount: u64,
    pub estimated_fee_profit_lamports: i64,
    pub window_blocks: u32,
    pub detected_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct BackrunOpportunity {
    pub id: Uuid,
    pub target_tx: String,
    pub token_mint: String,
    pub expected_price_impact_bps: i32,
    pub backrun_direction: String,
    pub estimated_profit_lamports: i64,
    pub detected_at: chrono::DateTime<chrono::Utc>,
}

impl MevHunter {
    pub fn new(event_tx: broadcast::Sender<ArbEvent>, config: MevHunterConfig) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_tx,
            dex_venues: Arc::new(RwLock::new(Vec::new())),
            lending_venues: Arc::new(RwLock::new(Vec::new())),
            pending_transactions: Arc::new(RwLock::new(Vec::new())),
            config,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub async fn add_dex_venue(&self, venue: Arc<dyn MevVenue>) {
        let mut venues = self.dex_venues.write().await;
        venues.push(venue);
    }

    pub async fn add_lending_venue(&self, venue: Arc<dyn MevVenue>) {
        let mut venues = self.lending_venues.write().await;
        venues.push(venue);
    }

    pub async fn add_pending_transaction(&self, tx: PendingTransaction) {
        let mut pending = self.pending_transactions.write().await;
        pending.push(tx);
        if pending.len() > 1000 {
            pending.drain(0..500);
        }
    }

    pub async fn scan_dex_arb(&self) -> AppResult<Vec<Signal>> {
        let venues = self.dex_venues.read().await;
        let mut signals = Vec::new();

        if venues.len() < 2 {
            return Ok(signals);
        }

        for pair in &self.config.arb_pairs {
            let mut quotes: Vec<(String, Quote)> = Vec::new();

            for venue in venues.iter() {
                let params = QuoteParams {
                    input_mint: pair.token_a.clone(),
                    output_mint: pair.token_b.clone(),
                    amount_lamports: 1_000_000_000,
                    slippage_bps: 50,
                };

                if let Ok(quote) = venue.get_quote(&params).await {
                    quotes.push((venue.name().to_string(), quote));
                }
            }

            if quotes.len() >= 2 {
                if let Some(signal) = self.find_arb_opportunity(&pair.name, &quotes).await {
                    if signal.estimated_profit_bps >= self.config.min_arb_profit_bps {
                        let _ = self.event_tx.send(ArbEvent::new(
                            "dex_arb_detected",
                            EventSource::Agent(AgentType::MevHunter),
                            "arb.mev.dex_arb.detected",
                            serde_json::json!({
                                "signal_id": signal.id,
                                "pair": pair.name,
                                "profit_bps": signal.estimated_profit_bps,
                                "confidence": signal.confidence,
                            }),
                        ));
                        signals.push(signal);
                    }
                }
            }
        }

        Ok(signals)
    }

    async fn find_arb_opportunity(&self, pair_name: &str, quotes: &[(String, Quote)]) -> Option<Signal> {
        let mut best_buy: Option<(&str, &Quote)> = None;
        let mut best_sell: Option<(&str, &Quote)> = None;

        for (venue, quote) in quotes {
            let rate = quote.output_amount as f64 / quote.input_amount as f64;

            if best_buy.is_none() || rate > best_buy.unwrap().1.output_amount as f64 / best_buy.unwrap().1.input_amount as f64 {
                best_buy = Some((venue.as_str(), quote));
            }
        }

        for (venue, quote) in quotes {
            if Some(venue.as_str()) == best_buy.map(|(v, _)| v) {
                continue;
            }
            let rate = quote.output_amount as f64 / quote.input_amount as f64;
            if best_sell.is_none() || rate < best_sell.unwrap().1.output_amount as f64 / best_sell.unwrap().1.input_amount as f64 {
                best_sell = Some((venue.as_str(), quote));
            }
        }

        let (buy_venue, buy_quote) = best_buy?;
        let (sell_venue, sell_quote) = best_sell?;

        let buy_rate = buy_quote.output_amount as f64 / buy_quote.input_amount as f64;
        let sell_rate = sell_quote.output_amount as f64 / sell_quote.input_amount as f64;

        if buy_rate <= sell_rate {
            return None;
        }

        let spread_bps = ((buy_rate - sell_rate) / sell_rate * 10000.0) as i32;

        if spread_bps < self.config.min_arb_profit_bps {
            return None;
        }

        let estimated_profit = ((buy_quote.output_amount as i64 - sell_quote.output_amount as i64)
            - self.config.max_gas_lamports as i64)
            .max(0);

        let confidence = if spread_bps > 100 { 0.9 } else if spread_bps > 50 { 0.7 } else { 0.5 };

        let significance = if spread_bps > 100 {
            Significance::Critical
        } else if spread_bps > 50 {
            Significance::High
        } else {
            Significance::Medium
        };

        Some(Signal {
            id: Uuid::new_v4(),
            signal_type: SignalType::DexArb,
            venue_id: self.id,
            venue_type: VenueType::DexAmm,
            token_mint: Some(buy_quote.input_mint.clone()),
            pool_address: None,
            estimated_profit_bps: spread_bps,
            confidence,
            significance,
            metadata: serde_json::json!({
                "pair": pair_name,
                "buy_venue": buy_venue,
                "sell_venue": sell_venue,
                "buy_rate": buy_rate,
                "sell_rate": sell_rate,
                "spread_bps": spread_bps,
                "estimated_profit_lamports": estimated_profit,
                "atomicity": "fully_atomic",
                "route": {
                    "legs": [
                        {
                            "venue": buy_venue,
                            "action": "buy",
                            "input_mint": buy_quote.input_mint,
                            "output_mint": buy_quote.output_mint,
                            "input_amount": buy_quote.input_amount,
                            "expected_output": buy_quote.output_amount,
                        },
                        {
                            "venue": sell_venue,
                            "action": "sell",
                            "input_mint": sell_quote.output_mint,
                            "output_mint": sell_quote.input_mint,
                            "input_amount": buy_quote.output_amount,
                            "expected_output": (buy_quote.output_amount as f64 / sell_rate) as u64,
                        }
                    ]
                }
            }),
            detected_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + chrono::Duration::seconds(10),
        })
    }

    pub async fn scan_jit_opportunities(&self) -> AppResult<Vec<Signal>> {
        let pending = self.pending_transactions.read().await;
        let mut signals = Vec::new();

        for tx in pending.iter() {
            if let PendingTxType::Swap { input_mint, output_mint, expected_output } = &tx.tx_type {
                if tx.amount_lamports >= 10_000_000_000 {
                    let optimal_liquidity = tx.amount_lamports / 10;
                    let estimated_fee_profit = (tx.amount_lamports as f64 * 0.003) as i64;

                    if estimated_fee_profit > (self.config.min_jit_profit_bps as i64 * tx.amount_lamports as i64 / 10000) {
                        let signal = Signal {
                            id: Uuid::new_v4(),
                            signal_type: SignalType::JitLiquidity,
                            venue_id: self.id,
                            venue_type: VenueType::DexAmm,
                            token_mint: Some(input_mint.clone()),
                            pool_address: None,
                            estimated_profit_bps: (estimated_fee_profit * 10000 / tx.amount_lamports as i64) as i32,
                            confidence: 0.6,
                            significance: Significance::High,
                            metadata: serde_json::json!({
                                "target_tx": tx.tx_signature,
                                "input_mint": input_mint,
                                "output_mint": output_mint,
                                "swap_amount": tx.amount_lamports,
                                "optimal_liquidity": optimal_liquidity,
                                "estimated_fee_profit": estimated_fee_profit,
                                "atomicity": "fully_atomic",
                            }),
                            detected_at: chrono::Utc::now(),
                            expires_at: chrono::Utc::now() + chrono::Duration::seconds(2),
                        };

                        let _ = self.event_tx.send(ArbEvent::new(
                            "jit_opportunity_detected",
                            EventSource::Agent(AgentType::MevHunter),
                            "arb.mev.jit.detected",
                            serde_json::json!({
                                "signal_id": signal.id,
                                "swap_amount": tx.amount_lamports,
                                "estimated_profit": estimated_fee_profit,
                            }),
                        ));

                        signals.push(signal);
                    }
                }
            }
        }

        Ok(signals)
    }

    pub async fn scan_backrun_opportunities(&self) -> AppResult<Vec<Signal>> {
        let pending = self.pending_transactions.read().await;
        let mut signals = Vec::new();

        for tx in pending.iter() {
            if let PendingTxType::Swap { input_mint, output_mint, expected_output } = &tx.tx_type {
                if tx.amount_lamports >= 50_000_000_000 {
                    let expected_impact_bps = (tx.amount_lamports as f64 / 1_000_000_000_000.0 * 100.0) as i32;

                    if expected_impact_bps >= 10 {
                        let estimated_profit = (tx.amount_lamports as f64 * expected_impact_bps as f64 / 10000.0 * 0.5) as i64;

                        let signal = Signal {
                            id: Uuid::new_v4(),
                            signal_type: SignalType::Backrun,
                            venue_id: self.id,
                            venue_type: VenueType::DexAmm,
                            token_mint: Some(output_mint.clone()),
                            pool_address: None,
                            estimated_profit_bps: (estimated_profit * 10000 / tx.amount_lamports as i64) as i32,
                            confidence: 0.5,
                            significance: Significance::Medium,
                            metadata: serde_json::json!({
                                "target_tx": tx.tx_signature,
                                "input_mint": input_mint,
                                "output_mint": output_mint,
                                "swap_amount": tx.amount_lamports,
                                "expected_impact_bps": expected_impact_bps,
                                "backrun_direction": "buy",
                                "estimated_profit_lamports": estimated_profit,
                                "atomicity": "fully_atomic",
                            }),
                            detected_at: chrono::Utc::now(),
                            expires_at: chrono::Utc::now() + chrono::Duration::seconds(1),
                        };

                        let _ = self.event_tx.send(ArbEvent::new(
                            "backrun_opportunity_detected",
                            EventSource::Agent(AgentType::MevHunter),
                            "arb.mev.backrun.detected",
                            serde_json::json!({
                                "signal_id": signal.id,
                                "target_tx": tx.tx_signature,
                                "expected_impact_bps": expected_impact_bps,
                                "estimated_profit": estimated_profit,
                            }),
                        ));

                        signals.push(signal);
                    }
                }
            }
        }

        Ok(signals)
    }

    pub async fn scan_liquidations(&self) -> AppResult<Vec<Signal>> {
        let venues = self.lending_venues.read().await;
        let mut signals = Vec::new();

        for venue in venues.iter() {
            match venue.scan_for_signals().await {
                Ok(venue_signals) => {
                    for signal in venue_signals {
                        if signal.signal_type == SignalType::Liquidation {
                            if signal.estimated_profit_bps >= self.config.min_liquidation_profit_bps {
                                let _ = self.event_tx.send(ArbEvent::new(
                                    "liquidation_detected",
                                    EventSource::Agent(AgentType::MevHunter),
                                    "arb.mev.liquidation.detected",
                                    serde_json::json!({
                                        "signal_id": signal.id,
                                        "venue": venue.name(),
                                        "profit_bps": signal.estimated_profit_bps,
                                        "confidence": signal.confidence,
                                    }),
                                ));
                                signals.push(signal);
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        venue = %venue.name(),
                        error = %e,
                        "Failed to scan lending venue for liquidations"
                    );
                }
            }
        }

        Ok(signals)
    }

    pub async fn scan_all(&self) -> AppResult<Vec<Signal>> {
        let mut all_signals = Vec::new();

        let arb_signals = self.scan_dex_arb().await?;
        all_signals.extend(arb_signals);

        let liquidation_signals = self.scan_liquidations().await?;
        all_signals.extend(liquidation_signals);

        let jit_signals = self.scan_jit_opportunities().await?;
        all_signals.extend(jit_signals);

        let backrun_signals = self.scan_backrun_opportunities().await?;
        all_signals.extend(backrun_signals);

        all_signals.sort_by(|a, b| {
            b.estimated_profit_bps.cmp(&a.estimated_profit_bps)
        });

        Ok(all_signals)
    }

    pub async fn get_stats(&self) -> MevHunterStats {
        let dex_count = self.dex_venues.read().await.len();
        let lending_count = self.lending_venues.read().await.len();
        let pending_count = self.pending_transactions.read().await.len();

        MevHunterStats {
            id: self.id,
            dex_venues_count: dex_count,
            lending_venues_count: lending_count,
            pending_transactions_count: pending_count,
            config: self.config.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MevHunterStats {
    pub id: Uuid,
    pub dex_venues_count: usize,
    pub lending_venues_count: usize,
    pub pending_transactions_count: usize,
    pub config: MevHunterConfig,
}
