use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::{AppError, AppResult};
use crate::helius::HeliusClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenHolder {
    pub address: String,
    pub balance: u64,
    pub balance_percent: f64,
    pub is_creator: bool,
    pub is_suspicious: bool,
    pub first_seen_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HolderDistribution {
    pub mint: String,
    pub total_holders: u32,
    pub total_supply: u64,
    pub circulating_supply: u64,
    pub top_10_holders: Vec<TokenHolder>,
    pub top_10_concentration: f64,
    pub top_20_concentration: f64,
    pub top_50_concentration: f64,
    pub creator_address: Option<String>,
    pub creator_holdings_percent: f64,
    pub gini_coefficient: f64,
    pub unique_wallets_24h: u32,
    pub new_holders_24h: i32,
    pub wash_trade_likelihood: f64,
    pub analyzed_at: DateTime<Utc>,
}

impl HolderDistribution {
    pub fn is_healthy(&self) -> bool {
        self.top_10_concentration < 50.0
            && self.creator_holdings_percent < 10.0
            && self.wash_trade_likelihood < 0.5
            && self.total_holders >= 50
    }

    pub fn health_score(&self) -> f64 {
        let mut score: f64 = 0.0;

        if self.total_holders >= 100 {
            score += 20.0;
        } else if self.total_holders >= 50 {
            score += 10.0;
        }

        if self.top_10_concentration < 30.0 {
            score += 25.0;
        } else if self.top_10_concentration < 50.0 {
            score += 15.0;
        } else if self.top_10_concentration < 70.0 {
            score += 5.0;
        }

        if self.creator_holdings_percent < 5.0 {
            score += 20.0;
        } else if self.creator_holdings_percent < 10.0 {
            score += 10.0;
        }

        if self.wash_trade_likelihood < 0.2 {
            score += 20.0;
        } else if self.wash_trade_likelihood < 0.5 {
            score += 10.0;
        }

        if self.gini_coefficient < 0.5 {
            score += 15.0;
        } else if self.gini_coefficient < 0.7 {
            score += 10.0;
        }

        score.min(100.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WashTradeAnalysis {
    pub circular_transfer_count: u32,
    pub same_source_accounts: u32,
    pub rapid_trade_pairs: u32,
    pub volume_inflation_estimate: f64,
    pub likelihood_score: f64,
}

pub struct HolderAnalyzer {
    helius_client: Arc<HeliusClient>,
    distribution_cache: Arc<RwLock<HashMap<String, HolderDistribution>>>,
    known_suspicious_wallets: Arc<RwLock<Vec<String>>>,
}

impl HolderAnalyzer {
    pub fn new(helius_client: Arc<HeliusClient>) -> Self {
        Self {
            helius_client,
            distribution_cache: Arc::new(RwLock::new(HashMap::new())),
            known_suspicious_wallets: Arc::new(RwLock::new(Vec::new())),
        }
    }

    #[cfg(test)]
    pub fn new_mock() -> Self {
        Self {
            helius_client: Arc::new(HeliusClient::new_mock()),
            distribution_cache: Arc::new(RwLock::new(HashMap::new())),
            known_suspicious_wallets: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn add_suspicious_wallet(&self, wallet: &str) {
        let mut wallets = self.known_suspicious_wallets.write().await;
        if !wallets.contains(&wallet.to_string()) {
            wallets.push(wallet.to_string());
        }
    }

    pub async fn analyze_holders(
        &self,
        mint: &str,
        creator_address: Option<&str>,
    ) -> AppResult<HolderDistribution> {
        self.analyze_holders_with_count(mint, creator_address, None)
            .await
    }

    pub async fn analyze_holders_with_count(
        &self,
        mint: &str,
        creator_address: Option<&str>,
        venue_holder_count: Option<u32>,
    ) -> AppResult<HolderDistribution> {
        let holders_response = self.helius_client.get_token_largest_accounts(mint).await?;

        let suspicious_wallets = self.known_suspicious_wallets.read().await;

        let total_supply: u64 = holders_response
            .value
            .iter()
            .map(|h| h.amount.parse::<u64>().unwrap_or(0))
            .sum();

        let mut holders: Vec<TokenHolder> = holders_response
            .value
            .iter()
            .map(|h| {
                let balance = h.amount.parse::<u64>().unwrap_or(0);
                let balance_percent = if total_supply > 0 {
                    (balance as f64 / total_supply as f64) * 100.0
                } else {
                    0.0
                };
                let is_creator = creator_address.map(|c| c == h.address).unwrap_or(false);
                let is_suspicious = suspicious_wallets.contains(&h.address);

                TokenHolder {
                    address: h.address.clone(),
                    balance,
                    balance_percent,
                    is_creator,
                    is_suspicious,
                    first_seen_at: None,
                }
            })
            .collect();

        holders.sort_by(|a, b| b.balance.cmp(&a.balance));

        let top_10_concentration: f64 = holders.iter().take(10).map(|h| h.balance_percent).sum();
        let top_20_concentration: f64 = holders.iter().take(20).map(|h| h.balance_percent).sum();
        let top_50_concentration: f64 = holders.iter().take(50).map(|h| h.balance_percent).sum();

        let creator_holdings_percent = if let Some(creator) = creator_address {
            holders
                .iter()
                .find(|h| h.address == creator)
                .map(|h| h.balance_percent)
                .unwrap_or(0.0)
        } else {
            0.0
        };

        let gini = self.calculate_gini_coefficient(&holders);

        let wash_trade_likelihood = self.estimate_wash_trading(&holders).await;

        let total_holders = venue_holder_count.unwrap_or(holders.len() as u32);

        let distribution = HolderDistribution {
            mint: mint.to_string(),
            total_holders,
            total_supply,
            circulating_supply: total_supply,
            top_10_holders: holders.iter().take(10).cloned().collect(),
            top_10_concentration,
            top_20_concentration,
            top_50_concentration,
            creator_address: creator_address.map(String::from),
            creator_holdings_percent,
            gini_coefficient: gini,
            unique_wallets_24h: 0,
            new_holders_24h: 0,
            wash_trade_likelihood,
            analyzed_at: Utc::now(),
        };

        {
            let mut cache = self.distribution_cache.write().await;
            cache.insert(mint.to_string(), distribution.clone());
        }

        Ok(distribution)
    }

    fn calculate_gini_coefficient(&self, holders: &[TokenHolder]) -> f64 {
        if holders.is_empty() {
            return 0.0;
        }

        let n = holders.len() as f64;
        let total: f64 = holders.iter().map(|h| h.balance as f64).sum();

        if total == 0.0 {
            return 0.0;
        }

        let mut sorted_balances: Vec<f64> = holders.iter().map(|h| h.balance as f64).collect();
        sorted_balances.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let mut cumsum = 0.0;
        let mut sum_of_ranks = 0.0;

        for (i, balance) in sorted_balances.iter().enumerate() {
            cumsum += balance;
            sum_of_ranks += (i as f64 + 1.0) * balance;
        }

        let gini = (2.0 * sum_of_ranks) / (n * cumsum) - (n + 1.0) / n;
        gini.clamp(0.0, 1.0)
    }

    async fn estimate_wash_trading(&self, holders: &[TokenHolder]) -> f64 {
        let mut score: f64 = 0.0;

        let suspicious_count = holders.iter().filter(|h| h.is_suspicious).count();
        if suspicious_count > 0 {
            score += (suspicious_count as f64 / holders.len() as f64) * 0.3;
        }

        let similar_balance_groups = self.find_similar_balance_groups(holders);
        if similar_balance_groups > 5 {
            score += 0.2;
        } else if similar_balance_groups > 2 {
            score += 0.1;
        }

        let very_small_holders = holders.iter().filter(|h| h.balance_percent < 0.01).count();

        if very_small_holders > holders.len() / 2 {
            score += 0.15;
        }

        if holders.len() < 20 {
            let top_5_percent: f64 = holders.iter().take(5).map(|h| h.balance_percent).sum();
            if top_5_percent > 90.0 {
                score += 0.25;
            }
        }

        score.clamp(0.0, 1.0)
    }

    fn find_similar_balance_groups(&self, holders: &[TokenHolder]) -> usize {
        if holders.len() < 3 {
            return 0;
        }

        let mut groups = 0;
        let tolerance = 0.001;

        for i in 0..holders.len() {
            for j in (i + 1)..holders.len() {
                if j >= holders.len() {
                    break;
                }

                let diff = (holders[i].balance_percent - holders[j].balance_percent).abs();
                if diff < tolerance && holders[i].balance_percent > 0.1 {
                    groups += 1;
                }
            }
        }

        groups
    }

    pub async fn get_cached_distribution(&self, mint: &str) -> Option<HolderDistribution> {
        let cache = self.distribution_cache.read().await;
        cache.get(mint).cloned()
    }

    pub async fn get_or_analyze(
        &self,
        mint: &str,
        creator: Option<&str>,
        max_age_seconds: i64,
    ) -> AppResult<HolderDistribution> {
        self.get_or_analyze_with_count(mint, creator, max_age_seconds, None)
            .await
    }

    pub async fn get_or_analyze_with_count(
        &self,
        mint: &str,
        creator: Option<&str>,
        max_age_seconds: i64,
        venue_holder_count: Option<u32>,
    ) -> AppResult<HolderDistribution> {
        if let Some(cached) = self.get_cached_distribution(mint).await {
            let age = (Utc::now() - cached.analyzed_at).num_seconds();
            if age < max_age_seconds {
                return Ok(cached);
            }
        }

        self.analyze_holders_with_count(mint, creator, venue_holder_count)
            .await
    }

    pub async fn analyze_wash_trading_detailed(&self, mint: &str) -> AppResult<WashTradeAnalysis> {
        let distribution = self.get_or_analyze(mint, None, 3600).await?;

        let holders = &distribution.top_10_holders;

        let similar_groups = self.find_similar_balance_groups(holders);

        let suspicious_count = holders.iter().filter(|h| h.is_suspicious).count();

        let volume_inflation = if distribution.wash_trade_likelihood > 0.5 {
            distribution.wash_trade_likelihood * 50.0
        } else {
            distribution.wash_trade_likelihood * 20.0
        };

        Ok(WashTradeAnalysis {
            circular_transfer_count: 0,
            same_source_accounts: suspicious_count as u32,
            rapid_trade_pairs: similar_groups as u32,
            volume_inflation_estimate: volume_inflation,
            likelihood_score: distribution.wash_trade_likelihood,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gini_coefficient() {
        let analyzer = HolderAnalyzer {
            helius_client: Arc::new(HeliusClient::new_mock()),
            distribution_cache: Arc::new(RwLock::new(HashMap::new())),
            known_suspicious_wallets: Arc::new(RwLock::new(Vec::new())),
        };

        let equal_holders: Vec<TokenHolder> = (0..10)
            .map(|i| TokenHolder {
                address: format!("wallet_{}", i),
                balance: 100,
                balance_percent: 10.0,
                is_creator: false,
                is_suspicious: false,
                first_seen_at: None,
            })
            .collect();

        let gini = analyzer.calculate_gini_coefficient(&equal_holders);
        assert!(gini < 0.1, "Equal distribution should have low Gini");

        let unequal_holders: Vec<TokenHolder> = vec![
            TokenHolder {
                address: "whale".to_string(),
                balance: 900,
                balance_percent: 90.0,
                is_creator: false,
                is_suspicious: false,
                first_seen_at: None,
            },
            TokenHolder {
                address: "small".to_string(),
                balance: 100,
                balance_percent: 10.0,
                is_creator: false,
                is_suspicious: false,
                first_seen_at: None,
            },
        ];

        let gini = analyzer.calculate_gini_coefficient(&unequal_holders);
        assert!(gini > 0.3, "Unequal distribution should have higher Gini");
    }

    #[test]
    fn test_distribution_health() {
        let healthy = HolderDistribution {
            mint: "healthy".to_string(),
            total_holders: 150,
            total_supply: 1_000_000_000,
            circulating_supply: 1_000_000_000,
            top_10_holders: vec![],
            top_10_concentration: 35.0,
            top_20_concentration: 50.0,
            top_50_concentration: 70.0,
            creator_address: None,
            creator_holdings_percent: 5.0,
            gini_coefficient: 0.5,
            unique_wallets_24h: 50,
            new_holders_24h: 10,
            wash_trade_likelihood: 0.1,
            analyzed_at: Utc::now(),
        };

        assert!(healthy.is_healthy());
        assert!(healthy.health_score() >= 60.0);

        let unhealthy = HolderDistribution {
            mint: "unhealthy".to_string(),
            total_holders: 20,
            total_supply: 1_000_000_000,
            circulating_supply: 1_000_000_000,
            top_10_holders: vec![],
            top_10_concentration: 85.0,
            top_20_concentration: 95.0,
            top_50_concentration: 99.0,
            creator_address: None,
            creator_holdings_percent: 30.0,
            gini_coefficient: 0.9,
            unique_wallets_24h: 5,
            new_holders_24h: -5,
            wash_trade_likelihood: 0.8,
            analyzed_at: Utc::now(),
        };

        assert!(!unhealthy.is_healthy());
        assert!(unhealthy.health_score() < 30.0);
    }
}
