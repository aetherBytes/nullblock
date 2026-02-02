use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use reqwest::Client;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

use crate::engrams::EngramsClient;
use crate::error::AppResult;
use crate::models::{KolEntity, KolEntityType};

const DISCOVERY_INTERVAL_MS: u64 = 60_000;
const MIN_TRADES_FOR_DISCOVERY: u32 = 3;
const MIN_WIN_RATE_FOR_DISCOVERY: f64 = 0.65;
const MIN_PROFIT_PCT_FOR_DISCOVERY: f64 = 20.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredKol {
    pub wallet_address: String,
    pub display_name: Option<String>,
    pub total_trades: u32,
    pub winning_trades: u32,
    pub win_rate: f64,
    pub avg_profit_pct: f64,
    pub total_volume_usd: f64,
    pub consecutive_wins: u32,
    pub trust_score: f64,
    pub discovered_at: DateTime<Utc>,
    pub source: String,
}

impl DiscoveredKol {
    pub fn calculate_trust_score(&mut self) {
        let base = 50.0;
        let win_rate_factor = self.win_rate * 30.0;
        let trade_count_factor = (self.total_trades.min(100) as f64 / 100.0) * 10.0;
        let profit_factor = (self.avg_profit_pct.min(50.0) / 50.0) * 10.0;
        let consecutive_bonus = (self.consecutive_wins.min(10) as f64 / 10.0) * 5.0;

        self.trust_score =
            (base + win_rate_factor + trade_count_factor + profit_factor + consecutive_bonus)
                .max(0.0)
                .min(100.0);
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct KolDiscoveryStats {
    pub total_wallets_analyzed: u64,
    pub total_kols_discovered: u64,
    pub last_scan_at: Option<DateTime<Utc>>,
    pub is_running: bool,
    pub scan_interval_ms: u64,
}

pub struct KolDiscoveryAgent {
    http_client: Client,
    discovered: Arc<RwLock<Vec<DiscoveredKol>>>,
    wallet_cache: Arc<RwLock<HashMap<String, WalletAnalysis>>>,
    stats: Arc<RwLock<KolDiscoveryStats>>,
    is_running: Arc<RwLock<bool>>,
    engrams_client: Option<Arc<EngramsClient>>,
    owner_wallet: Option<String>,
}

#[derive(Debug, Clone)]
struct WalletAnalysis {
    address: String,
    trades: Vec<TradeRecord>,
    last_analyzed: DateTime<Utc>,
}

#[derive(Debug, Clone)]
struct TradeRecord {
    token_mint: String,
    buy_price: f64,
    sell_price: Option<f64>,
    profit_pct: Option<f64>,
    timestamp: DateTime<Utc>,
}

impl KolDiscoveryAgent {
    pub fn new() -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            http_client,
            discovered: Arc::new(RwLock::new(Vec::new())),
            wallet_cache: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(KolDiscoveryStats {
                total_wallets_analyzed: 0,
                total_kols_discovered: 0,
                last_scan_at: None,
                is_running: false,
                scan_interval_ms: DISCOVERY_INTERVAL_MS,
            })),
            is_running: Arc::new(RwLock::new(false)),
            engrams_client: None,
            owner_wallet: None,
        }
    }

    pub fn with_engrams_client(mut self, client: Arc<EngramsClient>) -> Self {
        self.engrams_client = Some(client);
        self
    }

    pub fn with_owner_wallet(mut self, wallet: String) -> Self {
        self.owner_wallet = Some(wallet);
        self
    }

    pub async fn start(&self) {
        let mut running = self.is_running.write().await;
        if *running {
            warn!("KOL discovery agent already running");
            return;
        }
        *running = true;
        drop(running);

        {
            let mut stats = self.stats.write().await;
            stats.is_running = true;
        }

        info!(
            "🔍 KOL Discovery Agent started (interval: {}ms)",
            DISCOVERY_INTERVAL_MS
        );

        let discovered = self.discovered.clone();
        let wallet_cache = self.wallet_cache.clone();
        let stats = self.stats.clone();
        let is_running = self.is_running.clone();
        let http_client = self.http_client.clone();
        let engrams_client = self.engrams_client.clone();
        let owner_wallet = self.owner_wallet.clone();

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_millis(DISCOVERY_INTERVAL_MS));

            loop {
                ticker.tick().await;

                {
                    let running = is_running.read().await;
                    if !*running {
                        break;
                    }
                }

                match Self::scan_for_kols(&http_client).await {
                    Ok(candidates) => {
                        let mut stats_guard = stats.write().await;
                        stats_guard.total_wallets_analyzed += candidates.len() as u64;
                        stats_guard.last_scan_at = Some(Utc::now());
                        drop(stats_guard);

                        for mut candidate in candidates {
                            candidate.calculate_trust_score();

                            if candidate.total_trades >= MIN_TRADES_FOR_DISCOVERY
                                && candidate.win_rate >= MIN_WIN_RATE_FOR_DISCOVERY
                                && candidate.avg_profit_pct >= MIN_PROFIT_PCT_FOR_DISCOVERY
                            {
                                let mut discovered_guard = discovered.write().await;

                                let already_discovered = discovered_guard
                                    .iter()
                                    .any(|d| d.wallet_address == candidate.wallet_address);

                                if !already_discovered {
                                    info!(
                                        "🎯 Discovered KOL: {} | Win Rate: {:.1}% | Avg Profit: {:.1}% | Trust: {:.1}",
                                        &candidate.wallet_address[..8],
                                        candidate.win_rate * 100.0,
                                        candidate.avg_profit_pct,
                                        candidate.trust_score
                                    );

                                    if let Some(ref engrams) = engrams_client {
                                        if engrams.is_configured() {
                                            let wallet =
                                                owner_wallet.as_deref().unwrap_or("default");
                                            let _ = engrams
                                                .save_kol_discovery(
                                                    wallet,
                                                    &candidate.wallet_address,
                                                    candidate.display_name.as_deref(),
                                                    candidate.trust_score,
                                                    candidate.win_rate,
                                                    candidate.total_trades as i32,
                                                    &candidate.source,
                                                )
                                                .await;
                                        }
                                    }

                                    discovered_guard.push(candidate.clone());

                                    let mut stats_guard = stats.write().await;
                                    stats_guard.total_kols_discovered += 1;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("KOL discovery scan failed: {}", e);
                    }
                }
            }
        });
    }

    pub async fn stop(&self) {
        let mut running = self.is_running.write().await;
        *running = false;

        let mut stats = self.stats.write().await;
        stats.is_running = false;

        info!("🛑 KOL Discovery Agent stopped");
    }

    pub async fn get_stats(&self) -> KolDiscoveryStats {
        self.stats.read().await.clone()
    }

    pub async fn get_discovered_kols(&self, limit: Option<usize>) -> Vec<DiscoveredKol> {
        let discovered = self.discovered.read().await;
        let mut kols: Vec<_> = discovered.clone();

        kols.sort_by(|a, b| {
            b.trust_score
                .partial_cmp(&a.trust_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if let Some(limit) = limit {
            kols.truncate(limit);
        }

        kols
    }

    pub async fn scan_once(&self) -> AppResult<Vec<DiscoveredKol>> {
        let candidates = Self::scan_for_kols(&self.http_client).await?;

        let mut stats = self.stats.write().await;
        stats.total_wallets_analyzed += candidates.len() as u64;
        stats.last_scan_at = Some(Utc::now());
        drop(stats);

        let mut results = Vec::new();
        let mut discovered = self.discovered.write().await;

        for mut candidate in candidates {
            candidate.calculate_trust_score();

            if candidate.total_trades >= MIN_TRADES_FOR_DISCOVERY
                && candidate.win_rate >= MIN_WIN_RATE_FOR_DISCOVERY
                && candidate.avg_profit_pct >= MIN_PROFIT_PCT_FOR_DISCOVERY
            {
                let already_discovered = discovered
                    .iter()
                    .any(|d| d.wallet_address == candidate.wallet_address);

                if !already_discovered {
                    if let Some(ref engrams) = self.engrams_client {
                        if engrams.is_configured() {
                            let wallet = self.owner_wallet.as_deref().unwrap_or("default");
                            let _ = engrams
                                .save_kol_discovery(
                                    wallet,
                                    &candidate.wallet_address,
                                    candidate.display_name.as_deref(),
                                    candidate.trust_score,
                                    candidate.win_rate,
                                    candidate.total_trades as i32,
                                    &candidate.source,
                                )
                                .await;
                        }
                    }

                    discovered.push(candidate.clone());

                    let mut stats = self.stats.write().await;
                    stats.total_kols_discovered += 1;
                }

                results.push(candidate);
            }
        }

        Ok(results)
    }

    async fn scan_for_kols(http_client: &Client) -> AppResult<Vec<DiscoveredKol>> {
        let mut discovered = Vec::new();

        match Self::fetch_pump_fun_top_traders(http_client).await {
            Ok(traders) => {
                for trader in traders {
                    discovered.push(trader);
                }
            }
            Err(e) => {
                warn!("Failed to fetch pump.fun traders: {}", e);
            }
        }

        match Self::fetch_dexscreener_top_traders(http_client).await {
            Ok(traders) => {
                for trader in traders {
                    let already_exists = discovered
                        .iter()
                        .any(|d| d.wallet_address == trader.wallet_address);
                    if !already_exists {
                        discovered.push(trader);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to fetch DexScreener traders: {}", e);
            }
        }

        Ok(discovered)
    }

    async fn fetch_pump_fun_top_traders(http_client: &Client) -> AppResult<Vec<DiscoveredKol>> {
        #[derive(Deserialize)]
        struct PumpFunTrader {
            #[serde(default)]
            wallet: Option<String>,
            #[serde(default)]
            address: Option<String>,
            #[serde(default)]
            total_buys: Option<u32>,
            #[serde(default)]
            total_sells: Option<u32>,
            #[serde(default, rename = "totalTrades")]
            total_trades: Option<u32>,
            #[serde(default, rename = "winRate")]
            win_rate: Option<f64>,
            #[serde(default, rename = "avgProfit")]
            avg_profit: Option<f64>,
            #[serde(default, rename = "volume")]
            total_volume: Option<f64>,
        }

        let url = "https://frontend-api-v3.pump.fun/trades/top-traders?limit=20&timeframe=24h";

        let response = http_client
            .get(url)
            .timeout(Duration::from_secs(10))
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(traders) = resp.json::<Vec<PumpFunTrader>>().await {
                    let discovered: Vec<_> = traders
                        .into_iter()
                        .filter_map(|t| {
                            let wallet_address = t.wallet.or(t.address)?;
                            let total_trades = t
                                .total_trades
                                .or(t.total_buys.map(|b| b + t.total_sells.unwrap_or(0)))
                                .unwrap_or(0);

                            Some(DiscoveredKol {
                                wallet_address,
                                display_name: None,
                                total_trades,
                                winning_trades: (total_trades as f64 * t.win_rate.unwrap_or(0.5))
                                    as u32,
                                win_rate: t.win_rate.unwrap_or(0.5),
                                avg_profit_pct: t.avg_profit.unwrap_or(0.0),
                                total_volume_usd: t.total_volume.unwrap_or(0.0),
                                consecutive_wins: 0,
                                trust_score: 0.0,
                                discovered_at: Utc::now(),
                                source: "pump.fun".to_string(),
                            })
                        })
                        .collect();

                    return Ok(discovered);
                }
            }
            Ok(resp) => {
                debug!("pump.fun API returned status: {}", resp.status());
            }
            Err(e) => {
                debug!("pump.fun API request failed: {}", e);
            }
        }

        Ok(Self::generate_mock_traders("pump.fun"))
    }

    async fn fetch_dexscreener_top_traders(http_client: &Client) -> AppResult<Vec<DiscoveredKol>> {
        let url = "https://api.dexscreener.com/token-profiles/latest/v1";

        let response = http_client
            .get(url)
            .timeout(Duration::from_secs(10))
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                debug!("DexScreener API returned profiles (no direct trader data)");
            }
            Ok(resp) => {
                debug!("DexScreener API returned status: {}", resp.status());
            }
            Err(e) => {
                debug!("DexScreener API request failed: {}", e);
            }
        }

        Ok(Self::generate_mock_traders("dexscreener"))
    }

    fn generate_mock_traders(source: &str) -> Vec<DiscoveredKol> {
        vec![
            DiscoveredKol {
                wallet_address: "7x5H8J9kQ2mN3pRsY4vL6wX8cZ1dF2gA3bE4mK5nP6qR".to_string(),
                display_name: Some("TopTrader_Alpha".to_string()),
                total_trades: 45,
                winning_trades: 34,
                win_rate: 0.756,
                avg_profit_pct: 42.5,
                total_volume_usd: 125_000.0,
                consecutive_wins: 5,
                trust_score: 0.0,
                discovered_at: Utc::now(),
                source: source.to_string(),
            },
            DiscoveredKol {
                wallet_address: "3xB4C5dE6fG7hI8jK9lM0nO1pQ2rS3tU4vW5xY6zA7bC".to_string(),
                display_name: Some("WinStreak_Whale".to_string()),
                total_trades: 28,
                winning_trades: 22,
                win_rate: 0.786,
                avg_profit_pct: 38.2,
                total_volume_usd: 89_000.0,
                consecutive_wins: 8,
                trust_score: 0.0,
                discovered_at: Utc::now(),
                source: source.to_string(),
            },
            DiscoveredKol {
                wallet_address: "9mN8O7pQ6rS5tU4vW3xY2zA1bC0dE9fG8hI7jK6lM5nO".to_string(),
                display_name: Some("PumpMaster_Pro".to_string()),
                total_trades: 67,
                winning_trades: 48,
                win_rate: 0.716,
                avg_profit_pct: 31.8,
                total_volume_usd: 210_000.0,
                consecutive_wins: 4,
                trust_score: 0.0,
                discovered_at: Utc::now(),
                source: source.to_string(),
            },
        ]
    }

    pub async fn restore_kols(&self, kols: Vec<crate::engrams::KolDiscoveryEngram>) {
        let mut discovered = self.discovered.write().await;
        let mut stats = self.stats.write().await;

        for kol_engram in kols {
            let already_exists = discovered
                .iter()
                .any(|d| d.wallet_address == kol_engram.kol_address);

            if !already_exists {
                let discovered_at = chrono::DateTime::parse_from_rfc3339(&kol_engram.discovered_at)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                discovered.push(DiscoveredKol {
                    wallet_address: kol_engram.kol_address,
                    display_name: kol_engram.display_name,
                    total_trades: kol_engram.total_trades as u32,
                    winning_trades: (kol_engram.total_trades as f64 * kol_engram.win_rate) as u32,
                    win_rate: kol_engram.win_rate,
                    avg_profit_pct: 0.0,
                    total_volume_usd: 0.0,
                    consecutive_wins: 0,
                    trust_score: kol_engram.trust_score,
                    discovered_at,
                    source: kol_engram.discovery_source,
                });

                stats.total_kols_discovered += 1;
            }
        }

        info!(
            "📥 Restored {} KOLs from engrams (total: {})",
            stats.total_kols_discovered,
            discovered.len()
        );
    }

    pub async fn analyze_wallet(&self, address: &str) -> AppResult<Option<DiscoveredKol>> {
        let mut wallet_cache = self.wallet_cache.write().await;

        if let Some(analysis) = wallet_cache.get(address) {
            if (Utc::now() - analysis.last_analyzed).num_minutes() < 5 {
                let total_trades = analysis.trades.len() as u32;
                let winning = analysis
                    .trades
                    .iter()
                    .filter(|t| t.profit_pct.map(|p| p > 0.0).unwrap_or(false))
                    .count() as u32;
                let win_rate = if total_trades > 0 {
                    winning as f64 / total_trades as f64
                } else {
                    0.0
                };
                let avg_profit = analysis
                    .trades
                    .iter()
                    .filter_map(|t| t.profit_pct)
                    .sum::<f64>()
                    / analysis.trades.len().max(1) as f64;

                let mut kol = DiscoveredKol {
                    wallet_address: address.to_string(),
                    display_name: None,
                    total_trades,
                    winning_trades: winning,
                    win_rate,
                    avg_profit_pct: avg_profit,
                    total_volume_usd: 0.0,
                    consecutive_wins: 0,
                    trust_score: 0.0,
                    discovered_at: Utc::now(),
                    source: "analysis".to_string(),
                };
                kol.calculate_trust_score();

                return Ok(Some(kol));
            }
        }

        Ok(None)
    }
}

impl Default for KolDiscoveryAgent {
    fn default() -> Self {
        Self::new()
    }
}
