pub mod external;

use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;

use crate::error::AppResult;
use crate::models::{
    AlertSeverity, BlockedEntity, ThreatAlert, ThreatAlertType, ThreatCategory,
    ThreatEntityType, ThreatFactors, ThreatScore, ThreatStats, WalletAnalysis,
    WatchedWallet, WhitelistedEntity, ScamAssociation,
};

pub use external::{
    BirdeyeClient, GoPlusClient, RugCheckClient,
    HolderAnalysis, GoPlusAnalysis, RugCheckAnalysis, WashTradingAnalysis,
};

lazy_static::lazy_static! {
    static ref BLOCKED_STORE: RwLock<HashMap<String, BlockedEntity>> = RwLock::new(HashMap::new());
    static ref WHITELIST_STORE: RwLock<HashMap<String, WhitelistedEntity>> = RwLock::new(HashMap::new());
    static ref WATCHED_STORE: RwLock<HashMap<Uuid, WatchedWallet>> = RwLock::new(HashMap::new());
    static ref ALERTS_STORE: RwLock<Vec<ThreatAlert>> = RwLock::new(Vec::new());
    static ref SCORE_CACHE: RwLock<HashMap<String, ThreatScore>> = RwLock::new(HashMap::new());
    static ref WALLET_ANALYSIS_CACHE: RwLock<HashMap<String, WalletAnalysis>> = RwLock::new(HashMap::new());
}

pub struct ThreatDetector {
    rugcheck: RugCheckClient,
    goplus: GoPlusClient,
    birdeye: BirdeyeClient,
}

impl ThreatDetector {
    pub fn new(
        rugcheck_url: String,
        goplus_url: String,
        birdeye_url: String,
        birdeye_key: Option<String>,
    ) -> Self {
        Self {
            rugcheck: RugCheckClient::new(rugcheck_url),
            goplus: GoPlusClient::new(goplus_url),
            birdeye: BirdeyeClient::new(birdeye_url, birdeye_key),
        }
    }

    pub async fn check_token(&self, mint: &str) -> AppResult<ThreatScore> {
        if let Some(cached) = SCORE_CACHE.read().unwrap().get(mint) {
            let age = chrono::Utc::now() - cached.created_at;
            if age.num_minutes() < 5 {
                return Ok(cached.clone());
            }
        }

        if WHITELIST_STORE.read().unwrap().contains_key(mint) {
            let factors = ThreatFactors::default();
            let score = ThreatScore::calculate(mint.to_string(), factors);
            return Ok(score);
        }

        if let Some(blocked) = BLOCKED_STORE.read().unwrap().get(mint) {
            let mut factors = ThreatFactors::default();
            factors.rugcheck_score = Some(0.0);
            factors.goplus_honeypot = Some(blocked.threat_category == ThreatCategory::Honeypot);
            let mut score = ThreatScore::calculate(mint.to_string(), factors);
            score.overall_score = blocked.threat_score.unwrap_or(1.0);
            score.risk_level = AlertSeverity::Critical;
            score.recommendation = format!("BLOCKED: {}", blocked.reason);
            return Ok(score);
        }

        let mut factors = ThreatFactors::default();
        let mut external_data = serde_json::json!({});

        if let Ok(rugcheck_response) = self.rugcheck.check_token(mint).await {
            let analysis = self.rugcheck.analyze_risks(&rugcheck_response);
            factors.has_mint_authority = analysis.has_mint_authority;
            factors.has_freeze_authority = analysis.has_freeze_authority;
            factors.rugcheck_score = Some(analysis.score / 100.0);
            factors.top_10_concentration = analysis.top_10_concentration / 100.0;
            external_data["rugcheck"] = serde_json::to_value(&analysis).unwrap_or_default();
        }

        if let Ok(goplus_info) = self.goplus.check_token(mint).await {
            let analysis = self.goplus.analyze(&goplus_info);
            factors.goplus_honeypot = Some(analysis.is_honeypot);
            factors.goplus_is_blacklisted = Some(analysis.is_blacklisted);
            factors.creator_holdings_percent = analysis.creator_percent / 100.0;
            if analysis.cannot_sell_all {
                factors.goplus_honeypot = Some(true);
            }
            external_data["goplus"] = serde_json::to_value(&analysis).unwrap_or_default();
        }

        if let Ok(holders) = self.birdeye.get_holders(mint, 50).await {
            if let Ok(token_info) = self.birdeye.get_token_info(mint).await {
                let supply = token_info.supply.unwrap_or(1.0);
                let holder_analysis = self.birdeye.analyze_holders(&holders, supply);
                if factors.top_10_concentration == 0.0 {
                    factors.top_10_concentration = holder_analysis.top_10_concentration / 100.0;
                }
                external_data["holder_analysis"] = serde_json::to_value(&holder_analysis).unwrap_or_default();
            }

            if let Ok(trades) = self.birdeye.get_recent_trades(mint, 100).await {
                let wash_analysis = self.birdeye.detect_wash_trading(&trades);
                factors.wash_trade_likelihood = wash_analysis.wash_trading_likelihood;
                external_data["wash_trading"] = serde_json::to_value(&wash_analysis).unwrap_or_default();
            }
        }

        let mut score = ThreatScore::calculate(mint.to_string(), factors);
        score.external_data = external_data;

        SCORE_CACHE.write().unwrap().insert(mint.to_string(), score.clone());

        if score.overall_score >= 0.7 {
            self.create_alert(
                ThreatAlertType::SuspiciousActivity,
                AlertSeverity::High,
                ThreatEntityType::Token,
                mint.to_string(),
                serde_json::json!({
                    "score": score.overall_score,
                    "recommendation": &score.recommendation
                }),
                "flagged".to_string(),
            );
        }

        Ok(score)
    }

    pub async fn check_wallet(&self, address: &str) -> AppResult<WalletAnalysis> {
        if let Some(cached) = WALLET_ANALYSIS_CACHE.read().unwrap().get(address) {
            return Ok(cached.clone());
        }

        let is_blocked = BLOCKED_STORE.read().unwrap().get(address).is_some();

        let mut analysis = WalletAnalysis {
            wallet_address: address.to_string(),
            is_known_scammer: is_blocked,
            scam_associations: Vec::new(),
            risk_score: if is_blocked { 1.0 } else { 0.0 },
            total_rugs_associated: 0,
            known_aliases: Vec::new(),
            first_seen: None,
            total_tokens_created: 0,
            rug_rate: 0.0,
        };

        if is_blocked {
            if let Some(blocked) = BLOCKED_STORE.read().unwrap().get(address) {
                analysis.scam_associations.push(ScamAssociation {
                    token_mint: "N/A".to_string(),
                    token_name: None,
                    scam_type: blocked.threat_category.clone(),
                    date: blocked.created_at,
                    details: blocked.reason.clone(),
                });
            }
        }

        WALLET_ANALYSIS_CACHE.write().unwrap().insert(address.to_string(), analysis.clone());

        Ok(analysis)
    }

    pub fn block_entity(
        &self,
        entity_type: ThreatEntityType,
        address: String,
        category: ThreatCategory,
        reason: String,
        reported_by: String,
    ) -> BlockedEntity {
        let mut entity = BlockedEntity::new(
            entity_type.clone(),
            address.clone(),
            category,
            reason.clone(),
            reported_by,
        );

        if let Some(cached) = SCORE_CACHE.read().unwrap().get(&address) {
            entity.threat_score = Some(cached.overall_score);
        }

        BLOCKED_STORE.write().unwrap().insert(address.clone(), entity.clone());

        self.create_alert(
            ThreatAlertType::SuspiciousActivity,
            AlertSeverity::High,
            entity_type,
            address,
            serde_json::json!({"reason": reason}),
            "blocked".to_string(),
        );

        entity
    }

    pub fn whitelist_entity(
        &self,
        entity_type: ThreatEntityType,
        address: String,
        reason: String,
        whitelisted_by: String,
    ) -> WhitelistedEntity {
        let entity = WhitelistedEntity::new(entity_type, address.clone(), reason, whitelisted_by);
        WHITELIST_STORE.write().unwrap().insert(address.clone(), entity.clone());
        BLOCKED_STORE.write().unwrap().remove(&address);
        entity
    }

    pub fn add_watched_wallet(&self, wallet: WatchedWallet) -> WatchedWallet {
        let id = wallet.id;
        WATCHED_STORE.write().unwrap().insert(id, wallet.clone());
        wallet
    }

    pub fn get_blocked(&self, category: Option<ThreatCategory>, limit: usize) -> Vec<BlockedEntity> {
        let store = BLOCKED_STORE.read().unwrap();
        let mut entities: Vec<_> = store.values()
            .filter(|e| {
                if let Some(ref cat) = category {
                    return &e.threat_category == cat;
                }
                true
            })
            .cloned()
            .collect();

        entities.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        entities.truncate(limit);
        entities
    }

    pub fn get_whitelisted(&self, limit: usize) -> Vec<WhitelistedEntity> {
        let store = WHITELIST_STORE.read().unwrap();
        let mut entities: Vec<_> = store.values().cloned().collect();
        entities.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        entities.truncate(limit);
        entities
    }

    pub fn get_watched(&self, limit: usize) -> Vec<WatchedWallet> {
        let store = WATCHED_STORE.read().unwrap();
        let mut wallets: Vec<_> = store.values()
            .filter(|w| w.is_active)
            .cloned()
            .collect();
        wallets.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        wallets.truncate(limit);
        wallets
    }

    pub fn get_alerts(&self, severity: Option<AlertSeverity>, limit: usize) -> Vec<ThreatAlert> {
        let store = ALERTS_STORE.read().unwrap();
        let mut alerts: Vec<_> = store.iter()
            .filter(|a| {
                if let Some(ref sev) = severity {
                    return &a.severity == sev;
                }
                true
            })
            .cloned()
            .collect();

        alerts.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        alerts.truncate(limit);
        alerts
    }

    pub fn get_score_history(&self, mint: &str) -> Option<ThreatScore> {
        SCORE_CACHE.read().unwrap().get(mint).cloned()
    }

    pub fn create_alert(
        &self,
        alert_type: ThreatAlertType,
        severity: AlertSeverity,
        entity_type: ThreatEntityType,
        address: String,
        details: serde_json::Value,
        action_taken: String,
    ) -> ThreatAlert {
        let alert = ThreatAlert::new(
            alert_type,
            severity,
            entity_type,
            address,
            details,
            action_taken,
        );

        ALERTS_STORE.write().unwrap().push(alert.clone());
        alert
    }

    pub fn get_stats(&self) -> ThreatStats {
        let blocked = BLOCKED_STORE.read().unwrap();
        let whitelist = WHITELIST_STORE.read().unwrap();
        let watched = WATCHED_STORE.read().unwrap();
        let alerts = ALERTS_STORE.read().unwrap();
        let cache = SCORE_CACHE.read().unwrap();

        let now = chrono::Utc::now();
        let day_ago = now - chrono::Duration::hours(24);

        let alerts_last_24h = alerts.iter()
            .filter(|a| a.created_at > day_ago)
            .count() as u64;

        let blocked_tokens = blocked.values()
            .filter(|e| e.entity_type == ThreatEntityType::Token)
            .count() as u64;

        let blocked_wallets = blocked.values()
            .filter(|e| e.entity_type == ThreatEntityType::Wallet)
            .count() as u64;

        let threats_detected = cache.values()
            .filter(|s| s.overall_score >= 0.5)
            .count() as u64;

        ThreatStats {
            total_tokens_checked: cache.len() as u64,
            threats_detected,
            rugs_prevented: blocked_tokens,
            blocked_tokens,
            blocked_wallets,
            whitelisted_count: whitelist.len() as u64,
            watched_wallets: watched.values().filter(|w| w.is_active).count() as u64,
            alerts_last_24h,
        }
    }

    pub fn is_blocked(&self, address: &str) -> bool {
        BLOCKED_STORE.read().unwrap().contains_key(address)
    }

    pub fn is_whitelisted(&self, address: &str) -> bool {
        WHITELIST_STORE.read().unwrap().contains_key(address)
    }

    pub fn remove_from_blocklist(&self, address: &str) -> bool {
        BLOCKED_STORE.write().unwrap().remove(address).is_some()
    }

    pub fn remove_from_whitelist(&self, address: &str) -> bool {
        WHITELIST_STORE.write().unwrap().remove(address).is_some()
    }
}

impl Default for ThreatDetector {
    fn default() -> Self {
        Self::new(
            "https://api.rugcheck.xyz/v1".to_string(),
            "https://api.gopluslabs.io/api/v1".to_string(),
            "https://public-api.birdeye.so".to_string(),
            None,
        )
    }
}
