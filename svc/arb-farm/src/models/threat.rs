use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ThreatCategory {
    RugPull,
    Honeypot,
    ScamWallet,
    WashTrader,
    SandwichAttacker,
    MaliciousContract,
    PumpAndDump,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ThreatEntityType {
    Token,
    Wallet,
    Contract,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatFactors {
    pub has_mint_authority: bool,
    pub has_freeze_authority: bool,
    pub has_blacklist: bool,
    pub is_upgradeable: bool,
    pub top_10_concentration: f64,
    pub creator_holdings_percent: f64,
    pub suspicious_holder_count: u32,
    pub sell_pressure_score: f64,
    pub wash_trade_likelihood: f64,
    pub bundle_manipulation_detected: bool,
    pub rugcheck_score: Option<f64>,
    pub goplus_honeypot: Option<bool>,
    pub goplus_is_blacklisted: Option<bool>,
    pub community_warnings: u32,
}

impl Default for ThreatFactors {
    fn default() -> Self {
        Self {
            has_mint_authority: false,
            has_freeze_authority: false,
            has_blacklist: false,
            is_upgradeable: false,
            top_10_concentration: 0.0,
            creator_holdings_percent: 0.0,
            suspicious_holder_count: 0,
            sell_pressure_score: 0.0,
            wash_trade_likelihood: 0.0,
            bundle_manipulation_detected: false,
            rugcheck_score: None,
            goplus_honeypot: None,
            goplus_is_blacklisted: None,
            community_warnings: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatScore {
    pub id: Uuid,
    pub token_mint: String,
    pub overall_score: f64,
    pub factors: ThreatFactors,
    pub confidence: f64,
    pub risk_level: AlertSeverity,
    pub recommendation: String,
    pub external_data: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

impl ThreatScore {
    pub fn calculate(token_mint: String, factors: ThreatFactors) -> Self {
        let mut score = 0.0;

        if factors.has_mint_authority {
            score += 0.15;
        }
        if factors.has_freeze_authority {
            score += 0.10;
        }
        if factors.has_blacklist {
            score += 0.10;
        }
        if factors.is_upgradeable {
            score += 0.05;
        }

        if factors.top_10_concentration > 0.7 {
            score += 0.20;
        } else if factors.top_10_concentration > 0.5 {
            score += 0.10;
        }

        if factors.creator_holdings_percent > 0.2 {
            score += 0.15;
        } else if factors.creator_holdings_percent > 0.1 {
            score += 0.08;
        }

        score += factors.sell_pressure_score * 0.10;
        score += factors.wash_trade_likelihood * 0.10;

        if factors.bundle_manipulation_detected {
            score += 0.10;
        }

        if let Some(rugcheck) = factors.rugcheck_score {
            score += (1.0 - rugcheck) * 0.15;
        }

        if factors.goplus_honeypot == Some(true) {
            score += 0.30;
        }

        if factors.goplus_is_blacklisted == Some(true) {
            score += 0.20;
        }

        score += (factors.community_warnings.min(10) as f64 / 10.0) * 0.10;

        let overall_score = score.min(1.0);

        let risk_level = if overall_score >= 0.7 {
            AlertSeverity::Critical
        } else if overall_score >= 0.5 {
            AlertSeverity::High
        } else if overall_score >= 0.3 {
            AlertSeverity::Medium
        } else {
            AlertSeverity::Low
        };

        let recommendation = match risk_level {
            AlertSeverity::Critical => "AVOID - High probability of rug pull or scam".to_string(),
            AlertSeverity::High => "CAUTION - Significant risk factors detected".to_string(),
            AlertSeverity::Medium => "MONITOR - Some concerning factors present".to_string(),
            AlertSeverity::Low => "SAFE - Low risk profile".to_string(),
        };

        let mut confidence: f64 = 0.5;
        if factors.rugcheck_score.is_some() {
            confidence += 0.2;
        }
        if factors.goplus_honeypot.is_some() {
            confidence += 0.2;
        }
        if factors.top_10_concentration > 0.0 {
            confidence += 0.1;
        }

        Self {
            id: Uuid::new_v4(),
            token_mint,
            overall_score,
            factors,
            confidence: confidence.min(1.0),
            risk_level,
            recommendation,
            external_data: serde_json::json!({}),
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedEntity {
    pub id: Uuid,
    pub entity_type: ThreatEntityType,
    pub address: String,
    pub threat_category: ThreatCategory,
    pub threat_score: Option<f64>,
    pub reason: String,
    pub evidence_url: Option<String>,
    pub reported_by: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

impl BlockedEntity {
    pub fn new(
        entity_type: ThreatEntityType,
        address: String,
        threat_category: ThreatCategory,
        reason: String,
        reported_by: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            entity_type,
            address,
            threat_category,
            threat_score: None,
            reason,
            evidence_url: None,
            reported_by,
            is_active: true,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistedEntity {
    pub id: Uuid,
    pub entity_type: ThreatEntityType,
    pub address: String,
    pub reason: String,
    pub whitelisted_by: String,
    pub created_at: DateTime<Utc>,
}

impl WhitelistedEntity {
    pub fn new(
        entity_type: ThreatEntityType,
        address: String,
        reason: String,
        whitelisted_by: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            entity_type,
            address,
            reason,
            whitelisted_by,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchedWallet {
    pub id: Uuid,
    pub wallet_address: String,
    pub related_token_mint: Option<String>,
    pub watch_reason: String,
    pub alert_on_sell: bool,
    pub alert_on_transfer: bool,
    pub alert_threshold_sol: Option<Decimal>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

impl WatchedWallet {
    pub fn new(wallet_address: String, watch_reason: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            wallet_address,
            related_token_mint: None,
            watch_reason,
            alert_on_sell: true,
            alert_on_transfer: true,
            alert_threshold_sol: None,
            is_active: true,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ThreatAlertType {
    RugDetected,
    HoneypotDetected,
    LargeSell,
    CreatorDumping,
    ConcentrationSpike,
    SuspiciousActivity,
    WashTradingDetected,
    BlacklistedInteraction,
    CommunityWarning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatAlert {
    pub id: Uuid,
    pub alert_type: ThreatAlertType,
    pub severity: AlertSeverity,
    pub entity_type: ThreatEntityType,
    pub address: String,
    pub token_symbol: Option<String>,
    pub details: serde_json::Value,
    pub action_taken: String,
    pub created_at: DateTime<Utc>,
}

impl ThreatAlert {
    pub fn new(
        alert_type: ThreatAlertType,
        severity: AlertSeverity,
        entity_type: ThreatEntityType,
        address: String,
        details: serde_json::Value,
        action_taken: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            alert_type,
            severity,
            entity_type,
            address,
            token_symbol: None,
            details,
            action_taken,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckTokenRequest {
    pub token_mint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckWalletRequest {
    pub wallet_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportThreatRequest {
    pub entity_type: ThreatEntityType,
    pub address: String,
    pub threat_category: ThreatCategory,
    pub reason: String,
    pub evidence_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchWalletRequest {
    pub wallet_address: String,
    pub related_token_mint: Option<String>,
    pub watch_reason: String,
    pub alert_on_sell: Option<bool>,
    pub alert_on_transfer: Option<bool>,
    pub alert_threshold_sol: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistRequest {
    pub entity_type: ThreatEntityType,
    pub address: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletAnalysis {
    pub wallet_address: String,
    pub is_known_scammer: bool,
    pub scam_associations: Vec<ScamAssociation>,
    pub risk_score: f64,
    pub total_rugs_associated: u32,
    pub known_aliases: Vec<String>,
    pub first_seen: Option<DateTime<Utc>>,
    pub total_tokens_created: u32,
    pub rug_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScamAssociation {
    pub token_mint: String,
    pub token_name: Option<String>,
    pub scam_type: ThreatCategory,
    pub date: DateTime<Utc>,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatStats {
    pub total_tokens_checked: u64,
    pub threats_detected: u64,
    pub rugs_prevented: u64,
    pub blocked_tokens: u64,
    pub blocked_wallets: u64,
    pub whitelisted_count: u64,
    pub watched_wallets: u64,
    pub alerts_last_24h: u64,
}
