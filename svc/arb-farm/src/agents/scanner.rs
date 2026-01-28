use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{broadcast, RwLock, Mutex};
use tokio::time::{interval, Duration};
use uuid::Uuid;

use crate::error::AppResult;
use crate::events::{ArbEvent, AgentType, EventSource, scanner as scanner_topics, swarm as swarm_topics};
use crate::models::{Signal, SignalType, VenueType};
use crate::venues::MevVenue;
use super::StrategyEngine;
use super::strategies::{BehavioralStrategy, StrategyRegistry, VenueSnapshot, TokenData};

pub struct VenueRateLimiter {
    last_request: Mutex<HashMap<Uuid, Instant>>,
    min_interval_ms: u64,
}

impl VenueRateLimiter {
    pub fn new(min_interval_ms: u64) -> Self {
        Self {
            last_request: Mutex::new(HashMap::new()),
            min_interval_ms,
        }
    }

    pub async fn wait_for_venue(&self, venue_id: Uuid) {
        let mut last_requests = self.last_request.lock().await;
        let now = Instant::now();

        if let Some(last) = last_requests.get(&venue_id) {
            let elapsed = now.duration_since(*last);
            let min_interval = Duration::from_millis(self.min_interval_ms);

            if elapsed < min_interval {
                let wait_time = min_interval - elapsed;
                drop(last_requests); // Release lock during sleep
                tokio::time::sleep(wait_time).await;
                last_requests = self.last_request.lock().await;
            }
        }

        last_requests.insert(venue_id, Instant::now());
    }
}

const MAX_CACHED_SIGNALS: usize = 100;
const SIGNAL_CACHE_TTL_SECS: i64 = 600; // 10 minutes

pub struct ScannerAgent {
    id: Uuid,
    venues: Arc<RwLock<HashMap<Uuid, Box<dyn MevVenue>>>>,
    event_tx: broadcast::Sender<ArbEvent>,
    scan_interval_ms: u64,
    is_running: Arc<RwLock<bool>>,
    stats: Arc<RwLock<ScannerStats>>,
    strategy_engine: Arc<RwLock<Option<Arc<StrategyEngine>>>>,
    behavioral_strategies: Arc<StrategyRegistry>,
    rate_limiter: Arc<VenueRateLimiter>,
    recent_signals: Arc<RwLock<Vec<Signal>>>,
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

const DEFAULT_RATE_LIMIT_INTERVAL_MS: u64 = 150; // 150ms minimum between venue API calls (~6.7 req/sec)

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
            behavioral_strategies: Arc::new(StrategyRegistry::new()),
            rate_limiter: Arc::new(VenueRateLimiter::new(DEFAULT_RATE_LIMIT_INTERVAL_MS)),
            recent_signals: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn with_rate_limit_ms(mut self, min_interval_ms: u64) -> Self {
        self.rate_limiter = Arc::new(VenueRateLimiter::new(min_interval_ms));
        self
    }

    pub fn get_strategy_registry(&self) -> Arc<StrategyRegistry> {
        Arc::clone(&self.behavioral_strategies)
    }

    pub async fn register_behavioral_strategy(&self, strategy: Arc<dyn BehavioralStrategy>) {
        self.behavioral_strategies.register(strategy).await;
        tracing::info!(
            "📡 Scanner: Registered behavioral strategy (total: {})",
            self.behavioral_strategies.count().await
        );
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

        if let Err(e) = self.event_tx.send(ArbEvent::new(
            "venue_added",
            EventSource::Agent(AgentType::Scanner),
            scanner_topics::VENUE_ADDED,
            serde_json::json!({
                "venue_id": venue_id,
            }),
        )) {
            tracing::warn!("Event broadcast failed (channel full/closed): {}", e);
        }
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
        let behavioral_strategies = Arc::clone(&self.behavioral_strategies);
        let rate_limiter = Arc::clone(&self.rate_limiter);
        let recent_signals = Arc::clone(&self.recent_signals);

        if let Err(e) = event_tx.send(ArbEvent::new(
            "scanner_started",
            EventSource::Agent(AgentType::Scanner),
            swarm_topics::AGENT_STARTED,
            serde_json::json!({
                "agent_type": "scanner",
                "scan_interval_ms": scan_interval,
            }),
        )) {
            tracing::warn!("Event broadcast failed (channel full/closed): {}", e);
        }

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_millis(scan_interval));

            loop {
                ticker.tick().await;

                if !*is_running.read().await {
                    break;
                }

                let venues_guard = venues.read().await;
                let mut all_signals: Vec<Signal> = Vec::new();
                let mut all_token_data: Vec<TokenData> = Vec::new();
                let mut healthy_count = 0u32;

                for venue in venues_guard.values() {
                    if venue.is_healthy().await {
                        healthy_count += 1;

                        rate_limiter.wait_for_venue(venue.venue_id()).await;

                        match venue.scan_for_token_data().await {
                            Ok(token_data) => {
                                for td in token_data {
                                    all_token_data.push(TokenData {
                                        mint: td.mint,
                                        name: td.name,
                                        symbol: td.symbol,
                                        graduation_progress: td.graduation_progress,
                                        bonding_curve_address: td.bonding_curve_address,
                                        market_cap_sol: td.market_cap_usd,
                                        volume_24h_sol: td.volume_24h_usd,
                                        holder_count: td.holder_count,
                                        created_at: chrono::Utc::now(),
                                        last_trade_at: None,
                                        metadata: td.metadata,
                                    });
                                }
                            }
                            Err(e) => {
                                tracing::warn!(
                                    venue_id = %venue.venue_id(),
                                    venue_name = %venue.name(),
                                    error = %e,
                                    "⚠️ Scanner error for venue"
                                );
                                if let Err(send_err) = event_tx.send(ArbEvent::new(
                                    "scan_error",
                                    EventSource::Agent(AgentType::Scanner),
                                    "arb.scanner.error",
                                    serde_json::json!({
                                        "venue_id": venue.venue_id(),
                                        "venue_name": venue.name(),
                                        "error": e.to_string(),
                                    }),
                                )) {
                                    tracing::warn!("Event broadcast failed (channel full/closed): {}", send_err);
                                }
                            }
                        }
                    }
                }

                drop(venues_guard);

                let active_strategies = behavioral_strategies.get_active().await;
                if !active_strategies.is_empty() {
                    let snapshot = VenueSnapshot {
                        venue_id: Uuid::nil(),
                        venue_type: VenueType::BondingCurve,
                        venue_name: "pump_fun".to_string(),
                        tokens: all_token_data.clone(),
                        raw_signals: Vec::new(),
                        timestamp: chrono::Utc::now(),
                        is_healthy: true,
                    };

                    for strategy in active_strategies {
                        match strategy.scan(&snapshot).await {
                            Ok(mut strategy_signals) => {
                                if !strategy_signals.is_empty() {
                                    for signal in &mut strategy_signals {
                                        if let Some(obj) = signal.metadata.as_object_mut() {
                                            obj.entry("signal_source").or_insert(
                                                serde_json::Value::String(strategy.strategy_type().to_string())
                                            );
                                        }
                                    }

                                    tracing::info!(
                                        "📊 {} generated {} signals",
                                        strategy.name(),
                                        strategy_signals.len()
                                    );
                                    for signal in strategy_signals {
                                        if let Err(e) = event_tx.send(ArbEvent::new(
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
                                                "strategy_source": strategy.name(),
                                            }),
                                        )) {
                                            tracing::warn!("Event broadcast failed (channel full/closed): {}", e);
                                        }
                                        all_signals.push(signal);
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::warn!(
                                    strategy = %strategy.name(),
                                    error = %e,
                                    "⚠️ Behavioral strategy scan error"
                                );
                            }
                        }
                    }
                }

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

                if !all_signals.is_empty() {
                    let mut cache = recent_signals.write().await;
                    let now = chrono::Utc::now();
                    let cutoff = now - chrono::Duration::seconds(SIGNAL_CACHE_TTL_SECS);

                    cache.retain(|s| s.detected_at > cutoff);

                    for signal in all_signals.iter() {
                        let signal_source = signal.metadata.get("signal_source")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let already_cached = cache.iter().any(|s| {
                            s.token_mint == signal.token_mint &&
                            s.signal_type == signal.signal_type &&
                            s.metadata.get("signal_source")
                                .and_then(|v| v.as_str())
                                .unwrap_or("") == signal_source
                        });
                        if !already_cached {
                            cache.push(signal.clone());
                        }
                    }

                    cache.sort_by(|a, b| b.detected_at.cmp(&a.detected_at));
                    cache.truncate(MAX_CACHED_SIGNALS);
                }

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

        if let Err(e) = self.event_tx.send(ArbEvent::new(
            "scanner_stopped",
            EventSource::Agent(AgentType::Scanner),
            swarm_topics::AGENT_STOPPED,
            serde_json::json!({
                "agent_type": "scanner",
            }),
        )) {
            tracing::warn!("Event broadcast failed (channel full/closed): {}", e);
        }
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

    pub async fn get_recent_signals(&self, limit: Option<usize>) -> Vec<Signal> {
        let cache = self.recent_signals.read().await;
        let now = chrono::Utc::now();
        let cutoff = now - chrono::Duration::seconds(SIGNAL_CACHE_TTL_SECS);

        let mut signals: Vec<Signal> = cache
            .iter()
            .filter(|s| s.detected_at > cutoff && s.expires_at > now)
            .cloned()
            .collect();

        signals.sort_by(|a, b| b.detected_at.cmp(&a.detected_at));

        if let Some(limit) = limit {
            signals.truncate(limit);
        }

        signals
    }

    pub async fn get_cached_signals_by_venue(&self, venue_type: VenueType, limit: Option<usize>) -> Vec<Signal> {
        let signals = self.get_recent_signals(None).await;
        let mut filtered: Vec<Signal> = signals
            .into_iter()
            .filter(|s| s.venue_type == venue_type)
            .collect();

        if let Some(limit) = limit {
            filtered.truncate(limit);
        }

        filtered
    }

    pub async fn get_cached_high_confidence(&self, min_confidence: f64, limit: Option<usize>) -> Vec<Signal> {
        let signals = self.get_recent_signals(None).await;
        let mut filtered: Vec<Signal> = signals
            .into_iter()
            .filter(|s| s.confidence >= min_confidence)
            .collect();

        if let Some(limit) = limit {
            filtered.truncate(limit);
        }

        filtered
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
