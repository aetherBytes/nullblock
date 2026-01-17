use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio::time::{interval, Duration};
use uuid::Uuid;

use crate::error::AppResult;
use crate::events::{ArbEvent, AgentType, EventSource, scanner as scanner_topics, swarm as swarm_topics};
use crate::models::{Signal, VenueType};
use crate::venues::MevVenue;
use super::StrategyEngine;

pub struct ScannerAgent {
    id: Uuid,
    venues: Arc<RwLock<HashMap<Uuid, Box<dyn MevVenue>>>>,
    event_tx: broadcast::Sender<ArbEvent>,
    scan_interval_ms: u64,
    is_running: Arc<RwLock<bool>>,
    stats: Arc<RwLock<ScannerStats>>,
    strategy_engine: Arc<RwLock<Option<Arc<StrategyEngine>>>>,
}

#[derive(Debug, Clone, Default)]
pub struct ScannerStats {
    pub total_scans: u64,
    pub total_signals_detected: u64,
    pub signals_by_type: HashMap<String, u64>,
    pub signals_by_venue: HashMap<String, u64>,
    pub last_scan_at: Option<chrono::DateTime<chrono::Utc>>,
    pub healthy_venues: u32,
    pub total_venues: u32,
}

#[derive(Debug, Clone)]
pub struct ScannerStatus {
    pub id: Uuid,
    pub is_running: bool,
    pub scan_interval_ms: u64,
    pub stats: ScannerStats,
    pub venue_statuses: Vec<VenueStatus>,
}

#[derive(Debug, Clone)]
pub struct VenueStatus {
    pub id: Uuid,
    pub name: String,
    pub venue_type: VenueType,
    pub is_healthy: bool,
}

impl ScannerAgent {
    pub fn new(event_tx: broadcast::Sender<ArbEvent>, scan_interval_ms: u64) -> Self {
        Self {
            id: Uuid::new_v4(),
            venues: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            scan_interval_ms,
            is_running: Arc::new(RwLock::new(false)),
            stats: Arc::new(RwLock::new(ScannerStats::default())),
            strategy_engine: Arc::new(RwLock::new(None)),
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub async fn set_strategy_engine(&self, engine: Arc<StrategyEngine>) {
        let mut se = self.strategy_engine.write().await;
        *se = Some(engine);
        tracing::info!("📡 Scanner: Strategy engine connected (auto-processing enabled)");
    }

    pub async fn add_venue(&self, venue: Box<dyn MevVenue>) {
        let mut venues = self.venues.write().await;
        let venue_id = venue.venue_id();
        venues.insert(venue_id, venue);

        let mut stats = self.stats.write().await;
        stats.total_venues += 1;

        let _ = self.event_tx.send(ArbEvent::new(
            "venue_added",
            EventSource::Agent(AgentType::Scanner),
            scanner_topics::VENUE_ADDED,
            serde_json::json!({
                "venue_id": venue_id,
            }),
        ));
    }

    pub async fn remove_venue(&self, venue_id: Uuid) -> bool {
        let mut venues = self.venues.write().await;
        if venues.remove(&venue_id).is_some() {
            let mut stats = self.stats.write().await;
            stats.total_venues = stats.total_venues.saturating_sub(1);
            true
        } else {
            false
        }
    }

    pub async fn get_status(&self) -> ScannerStatus {
        let is_running = *self.is_running.read().await;
        let stats = self.stats.read().await.clone();

        let venues = self.venues.read().await;
        let mut venue_statuses = Vec::new();

        for venue in venues.values() {
            let is_healthy = venue.is_healthy().await;
            venue_statuses.push(VenueStatus {
                id: venue.venue_id(),
                name: venue.name().to_string(),
                venue_type: venue.venue_type(),
                is_healthy,
            });
        }

        ScannerStatus {
            id: self.id,
            is_running,
            scan_interval_ms: self.scan_interval_ms,
            stats,
            venue_statuses,
        }
    }

    pub async fn start(&self) {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return;
        }
        *is_running = true;
        drop(is_running);

        let venues = Arc::clone(&self.venues);
        let event_tx = self.event_tx.clone();
        let stats = Arc::clone(&self.stats);
        let is_running = Arc::clone(&self.is_running);
        let scan_interval = self.scan_interval_ms;
        let strategy_engine = Arc::clone(&self.strategy_engine);

        let _ = event_tx.send(ArbEvent::new(
            "scanner_started",
            EventSource::Agent(AgentType::Scanner),
            swarm_topics::AGENT_STARTED,
            serde_json::json!({
                "agent_type": "scanner",
                "scan_interval_ms": scan_interval,
            }),
        ));

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_millis(scan_interval));

            loop {
                ticker.tick().await;

                if !*is_running.read().await {
                    break;
                }

                let venues_guard = venues.read().await;
                let mut all_signals: Vec<Signal> = Vec::new();
                let mut healthy_count = 0u32;

                for venue in venues_guard.values() {
                    if venue.is_healthy().await {
                        healthy_count += 1;

                        match venue.scan_for_signals().await {
                            Ok(signals) => {
                                for signal in signals {
                                    let _ = event_tx.send(ArbEvent::new(
                                        "signal_detected",
                                        EventSource::Agent(AgentType::Scanner),
                                        scanner_topics::SIGNAL_DETECTED,
                                        serde_json::json!({
                                            "signal_id": signal.id,
                                            "signal_type": format!("{:?}", signal.signal_type),
                                            "venue_id": signal.venue_id,
                                            "venue_type": format!("{:?}", signal.venue_type),
                                            "estimated_profit_bps": signal.estimated_profit_bps,
                                            "confidence": signal.confidence,
                                            "significance": format!("{:?}", signal.significance),
                                            "token_mint": signal.token_mint,
                                            "pool_address": signal.pool_address,
                                            "metadata": signal.metadata,
                                        }),
                                    ));
                                    all_signals.push(signal);
                                }
                            }
                            Err(e) => {
                                let _ = event_tx.send(ArbEvent::new(
                                    "scan_error",
                                    EventSource::Agent(AgentType::Scanner),
                                    "arb.scanner.error",
                                    serde_json::json!({
                                        "venue_id": venue.venue_id(),
                                        "venue_name": venue.name(),
                                        "error": e.to_string(),
                                    }),
                                ));
                            }
                        }
                    }
                }

                drop(venues_guard);

                let mut stats_guard = stats.write().await;
                stats_guard.total_scans += 1;
                stats_guard.last_scan_at = Some(chrono::Utc::now());
                stats_guard.healthy_venues = healthy_count;
                stats_guard.total_signals_detected += all_signals.len() as u64;

                for signal in &all_signals {
                    let signal_type = format!("{:?}", signal.signal_type);
                    *stats_guard.signals_by_type.entry(signal_type).or_insert(0) += 1;

                    let venue_id = signal.venue_id.to_string();
                    *stats_guard.signals_by_venue.entry(venue_id).or_insert(0) += 1;
                }
                drop(stats_guard);

                // Auto-process signals through strategy engine if configured
                if !all_signals.is_empty() {
                    let se_guard = strategy_engine.read().await;
                    if let Some(engine) = se_guard.as_ref() {
                        let signal_count = all_signals.len();
                        let results = engine.process_signals(all_signals.clone()).await;
                        let edge_count = results.iter().filter(|r| r.approved).count();
                        if edge_count > 0 {
                            tracing::info!(
                                "📡 Auto-processed {} signals → {} edges created",
                                signal_count,
                                edge_count
                            );
                        }
                    }
                }
            }
        });
    }

    pub async fn stop(&self) {
        let mut is_running = self.is_running.write().await;
        *is_running = false;

        let _ = self.event_tx.send(ArbEvent::new(
            "scanner_stopped",
            EventSource::Agent(AgentType::Scanner),
            swarm_topics::AGENT_STOPPED,
            serde_json::json!({
                "agent_type": "scanner",
            }),
        ));
    }

    pub async fn scan_once(&self) -> AppResult<Vec<Signal>> {
        let venues = self.venues.read().await;
        let mut all_signals = Vec::new();

        for venue in venues.values() {
            if venue.is_healthy().await {
                match venue.scan_for_signals().await {
                    Ok(signals) => {
                        all_signals.extend(signals);
                    }
                    Err(e) => {
                        tracing::warn!(
                            venue_id = %venue.venue_id(),
                            venue_name = %venue.name(),
                            error = %e,
                            "Venue scan failed"
                        );
                    }
                }
            }
        }

        let mut stats = self.stats.write().await;
        stats.total_scans += 1;
        stats.last_scan_at = Some(chrono::Utc::now());
        stats.total_signals_detected += all_signals.len() as u64;

        Ok(all_signals)
    }

    pub async fn get_signals_by_type(&self, signal_type: &str) -> AppResult<Vec<Signal>> {
        let all_signals = self.scan_once().await?;
        Ok(all_signals
            .into_iter()
            .filter(|s| format!("{:?}", s.signal_type).to_lowercase().contains(&signal_type.to_lowercase()))
            .collect())
    }

    pub async fn get_signals_by_venue(&self, venue_type: VenueType) -> AppResult<Vec<Signal>> {
        let all_signals = self.scan_once().await?;
        Ok(all_signals
            .into_iter()
            .filter(|s| s.venue_type == venue_type)
            .collect())
    }

    pub async fn get_high_confidence_signals(&self, min_confidence: f64) -> AppResult<Vec<Signal>> {
        let all_signals = self.scan_once().await?;
        Ok(all_signals
            .into_iter()
            .filter(|s| s.confidence >= min_confidence)
            .collect())
    }
}
