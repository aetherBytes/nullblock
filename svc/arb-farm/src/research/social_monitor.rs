use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SourceType {
    Twitter,
    Telegram,
    Discord,
    Rss,
}

impl SourceType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "twitter" | "x" => SourceType::Twitter,
            "telegram" | "tg" => SourceType::Telegram,
            "discord" => SourceType::Discord,
            "rss" => SourceType::Rss,
            _ => SourceType::Twitter,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrackType {
    Alpha,
    Threat,
    Both,
}

impl TrackType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "alpha" => TrackType::Alpha,
            "threat" => TrackType::Threat,
            "both" => TrackType::Both,
            _ => TrackType::Both,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoredSource {
    pub id: Uuid,
    pub source_type: SourceType,
    pub handle: String,
    pub display_name: Option<String>,
    pub track_type: TrackType,
    pub is_active: bool,
    pub keywords: Vec<String>,
    pub last_checked_at: Option<DateTime<Utc>>,
    pub last_post_id: Option<String>,
    pub total_posts_tracked: u64,
    pub alerts_generated: u64,
    pub created_at: DateTime<Utc>,
}

impl MonitoredSource {
    pub fn new(source_type: SourceType, handle: String, track_type: TrackType) -> Self {
        Self {
            id: Uuid::new_v4(),
            source_type,
            handle,
            display_name: None,
            track_type,
            is_active: true,
            keywords: Vec::new(),
            last_checked_at: None,
            last_post_id: None,
            total_posts_tracked: 0,
            alerts_generated: 0,
            created_at: Utc::now(),
        }
    }

    pub fn with_keywords(mut self, keywords: Vec<String>) -> Self {
        self.keywords = keywords;
        self
    }

    pub fn with_display_name(mut self, name: String) -> Self {
        self.display_name = Some(name);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialAlert {
    pub id: Uuid,
    pub source_id: Uuid,
    pub source_handle: String,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub title: String,
    pub content: String,
    pub url: Option<String>,
    pub tokens_mentioned: Vec<String>,
    pub addresses_found: Vec<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    TradingAlpha,
    NewToken,
    PriceAlert,
    WhaleActivity,
    RugWarning,
    ScamAlert,
    KeywordMatch,
}

pub struct SocialMonitor {
    sources: Arc<RwLock<HashMap<Uuid, MonitoredSource>>>,
    alerts: Arc<RwLock<Vec<SocialAlert>>>,
    keywords_alpha: Vec<String>,
    keywords_threat: Vec<String>,
}

impl SocialMonitor {
    pub fn new() -> Self {
        Self {
            sources: Arc::new(RwLock::new(HashMap::new())),
            alerts: Arc::new(RwLock::new(Vec::new())),
            keywords_alpha: vec![
                "alpha".to_string(),
                "entry".to_string(),
                "buying".to_string(),
                "bullish".to_string(),
                "pump".to_string(),
                "moon".to_string(),
                "gem".to_string(),
                "100x".to_string(),
                "dex".to_string(),
                "arb".to_string(),
            ],
            keywords_threat: vec![
                "rug".to_string(),
                "scam".to_string(),
                "honeypot".to_string(),
                "warning".to_string(),
                "avoid".to_string(),
                "fake".to_string(),
                "hack".to_string(),
                "exploit".to_string(),
                "drained".to_string(),
                "stolen".to_string(),
            ],
        }
    }

    pub async fn add_source(&self, source: MonitoredSource) -> Uuid {
        let id = source.id;
        self.sources.write().await.insert(id, source);
        id
    }

    pub async fn add_twitter_account(&self, handle: &str, track_type: TrackType) -> Uuid {
        let handle = handle.trim_start_matches('@').to_string();
        let source = MonitoredSource::new(SourceType::Twitter, handle, track_type);
        self.add_source(source).await
    }

    pub async fn remove_source(&self, source_id: Uuid) -> bool {
        self.sources.write().await.remove(&source_id).is_some()
    }

    pub async fn get_source(&self, source_id: Uuid) -> Option<MonitoredSource> {
        self.sources.read().await.get(&source_id).cloned()
    }

    pub async fn list_sources(&self) -> Vec<MonitoredSource> {
        self.sources.read().await.values().cloned().collect()
    }

    pub async fn list_sources_by_type(&self, source_type: SourceType) -> Vec<MonitoredSource> {
        self.sources.read().await
            .values()
            .filter(|s| s.source_type == source_type)
            .cloned()
            .collect()
    }

    pub async fn toggle_source(&self, source_id: Uuid, active: bool) -> AppResult<()> {
        if let Some(source) = self.sources.write().await.get_mut(&source_id) {
            source.is_active = active;
        }
        Ok(())
    }

    pub fn analyze_content(&self, content: &str, source: &MonitoredSource) -> Option<SocialAlert> {
        let content_lower = content.to_lowercase();

        let tokens = self.extract_tokens(content);
        let addresses = self.extract_addresses(content);

        let mut alert_type = None;
        let mut severity = AlertSeverity::Low;

        match source.track_type {
            TrackType::Threat | TrackType::Both => {
                for keyword in &self.keywords_threat {
                    if content_lower.contains(keyword) {
                        alert_type = Some(AlertType::ScamAlert);
                        severity = AlertSeverity::High;
                        break;
                    }
                }
            }
            _ => {}
        }

        if alert_type.is_none() {
            match source.track_type {
                TrackType::Alpha | TrackType::Both => {
                    for keyword in &self.keywords_alpha {
                        if content_lower.contains(keyword) {
                            alert_type = Some(AlertType::TradingAlpha);
                            severity = AlertSeverity::Medium;
                            break;
                        }
                    }
                }
                _ => {}
            }
        }

        for keyword in &source.keywords {
            if content_lower.contains(&keyword.to_lowercase()) {
                alert_type = Some(AlertType::KeywordMatch);
                severity = AlertSeverity::Medium;
                break;
            }
        }

        if !tokens.is_empty() && alert_type.is_none() {
            alert_type = Some(AlertType::NewToken);
            severity = AlertSeverity::Low;
        }

        let alert_type = alert_type?;

        Some(SocialAlert {
            id: Uuid::new_v4(),
            source_id: source.id,
            source_handle: source.handle.clone(),
            alert_type,
            severity,
            title: format!("Alert from @{}", source.handle),
            content: content.chars().take(500).collect(),
            url: None,
            tokens_mentioned: tokens,
            addresses_found: addresses,
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
        })
    }

    fn extract_tokens(&self, content: &str) -> Vec<String> {
        let mut tokens = Vec::new();

        for word in content.split_whitespace() {
            if word.starts_with('$') && word.len() > 1 && word.len() < 15 {
                let token = word
                    .trim_matches(|c: char| !c.is_alphanumeric())
                    .to_uppercase();
                if !token.is_empty() && !tokens.contains(&token) {
                    tokens.push(token);
                }
            }
        }

        tokens
    }

    fn extract_addresses(&self, content: &str) -> Vec<String> {
        let mut addresses = Vec::new();

        for word in content.split_whitespace() {
            let cleaned = word.trim_matches(|c: char| !c.is_alphanumeric());
            if cleaned.len() >= 32 && cleaned.len() <= 44 {
                if cleaned.chars().all(|c| c.is_alphanumeric()) {
                    if !addresses.contains(&cleaned.to_string()) {
                        addresses.push(cleaned.to_string());
                    }
                }
            }
        }

        addresses
    }

    pub async fn add_alert(&self, alert: SocialAlert) {
        let source_id = alert.source_id;
        self.alerts.write().await.push(alert);

        if let Some(source) = self.sources.write().await.get_mut(&source_id) {
            source.alerts_generated += 1;
        }
    }

    pub async fn get_recent_alerts(&self, limit: usize) -> Vec<SocialAlert> {
        let alerts = self.alerts.read().await;
        alerts.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    pub async fn get_alerts_by_type(&self, alert_type: &AlertType, limit: usize) -> Vec<SocialAlert> {
        let alerts = self.alerts.read().await;
        alerts.iter()
            .filter(|a| std::mem::discriminant(&a.alert_type) == std::mem::discriminant(alert_type))
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    pub async fn get_alerts_for_source(&self, source_id: Uuid, limit: usize) -> Vec<SocialAlert> {
        let alerts = self.alerts.read().await;
        alerts.iter()
            .filter(|a| a.source_id == source_id)
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    pub async fn mark_source_checked(&self, source_id: Uuid, last_post_id: Option<String>) {
        if let Some(source) = self.sources.write().await.get_mut(&source_id) {
            source.last_checked_at = Some(Utc::now());
            if let Some(post_id) = last_post_id {
                source.last_post_id = Some(post_id);
            }
            source.total_posts_tracked += 1;
        }
    }

    pub fn get_default_alpha_accounts() -> Vec<(&'static str, &'static str)> {
        vec![
            ("@solaboratory", "Solana Alpha Lab"),
            ("@DefiLlama", "DeFi TVL Tracker"),
            ("@Jupiter_Aggregator", "Jupiter DEX"),
            ("@RaydiumProtocol", "Raydium AMM"),
        ]
    }

    pub fn get_default_threat_accounts() -> Vec<(&'static str, &'static str)> {
        vec![
            ("@ZachXBT", "Crypto Investigator"),
            ("@RugDocIO", "Rug Pull Detection"),
            ("@PeckShieldAlert", "Security Alerts"),
            ("@CertiKAlert", "Smart Contract Audits"),
        ]
    }

    pub async fn add_default_sources(&self) {
        for (handle, name) in Self::get_default_alpha_accounts() {
            let source = MonitoredSource::new(
                SourceType::Twitter,
                handle.trim_start_matches('@').to_string(),
                TrackType::Alpha,
            ).with_display_name(name.to_string());
            self.add_source(source).await;
        }

        for (handle, name) in Self::get_default_threat_accounts() {
            let source = MonitoredSource::new(
                SourceType::Twitter,
                handle.trim_start_matches('@').to_string(),
                TrackType::Threat,
            ).with_display_name(name.to_string());
            self.add_source(source).await;
        }
    }

    pub async fn get_stats(&self) -> MonitorStats {
        let sources = self.sources.read().await;
        let alerts = self.alerts.read().await;

        let total_sources = sources.len() as u64;
        let active_sources = sources.values().filter(|s| s.is_active).count() as u64;
        let total_alerts = alerts.len() as u64;

        let by_type = sources.values().fold(HashMap::new(), |mut acc, s| {
            *acc.entry(format!("{:?}", s.source_type)).or_insert(0u64) += 1;
            acc
        });

        let alerts_last_24h = alerts.iter()
            .filter(|a| a.created_at > Utc::now() - chrono::Duration::hours(24))
            .count() as u64;

        MonitorStats {
            total_sources,
            active_sources,
            total_alerts,
            alerts_last_24h,
            sources_by_type: by_type,
        }
    }
}

impl Default for SocialMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorStats {
    pub total_sources: u64,
    pub active_sources: u64,
    pub total_alerts: u64,
    pub alerts_last_24h: u64,
    pub sources_by_type: HashMap<String, u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_twitter_account() {
        let monitor = SocialMonitor::new();
        let id = monitor.add_twitter_account("@testuser", TrackType::Alpha).await;

        let source = monitor.get_source(id).await.unwrap();
        assert_eq!(source.handle, "testuser");
        assert!(matches!(source.source_type, SourceType::Twitter));
    }

    #[test]
    fn test_analyze_alpha_content() {
        let monitor = SocialMonitor::new();
        let source = MonitoredSource::new(
            SourceType::Twitter,
            "trader".to_string(),
            TrackType::Alpha,
        );

        let content = "Bullish on $SOL, this is alpha! Entry now.";
        let alert = monitor.analyze_content(content, &source);

        assert!(alert.is_some());
        let alert = alert.unwrap();
        assert!(matches!(alert.alert_type, AlertType::TradingAlpha));
    }

    #[test]
    fn test_analyze_threat_content() {
        let monitor = SocialMonitor::new();
        let source = MonitoredSource::new(
            SourceType::Twitter,
            "zachxbt".to_string(),
            TrackType::Threat,
        );

        let content = "Warning: This project is a scam, avoid $FAKE";
        let alert = monitor.analyze_content(content, &source);

        assert!(alert.is_some());
        let alert = alert.unwrap();
        assert!(matches!(alert.alert_type, AlertType::ScamAlert));
    }

    #[test]
    fn test_extract_tokens() {
        let monitor = SocialMonitor::new();
        let content = "Check out $BONK and $SOL, both looking good";
        let tokens = monitor.extract_tokens(content);

        assert!(tokens.contains(&"BONK".to_string()));
        assert!(tokens.contains(&"SOL".to_string()));
    }
}
