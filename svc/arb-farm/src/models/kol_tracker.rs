use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::AppResult;
use crate::webhooks::parser::ParsedSwap;

pub struct KOLTracker {
    kols: Arc<RwLock<HashMap<String, KOLEntry>>>,
    trades: Arc<RwLock<Vec<ParsedSwap>>>,
    max_trades: usize,
}

#[derive(Debug, Clone)]
struct KOLEntry {
    pub address: String,
    pub name: Option<String>,
    pub trust_score: f64,
    pub trade_count: u32,
    pub total_volume_lamports: u64,
}

impl KOLTracker {
    pub fn new() -> Self {
        Self {
            kols: Arc::new(RwLock::new(HashMap::new())),
            trades: Arc::new(RwLock::new(Vec::new())),
            max_trades: 1000,
        }
    }

    pub async fn add_kol(&self, address: &str, name: Option<String>, trust_score: f64) {
        let mut kols = self.kols.write().await;
        kols.insert(
            address.to_string(),
            KOLEntry {
                address: address.to_string(),
                name,
                trust_score,
                trade_count: 0,
                total_volume_lamports: 0,
            },
        );
    }

    pub async fn remove_kol(&self, address: &str) {
        let mut kols = self.kols.write().await;
        kols.remove(address);
    }

    pub async fn get_kol_name(&self, address: &str) -> Option<String> {
        let kols = self.kols.read().await;
        kols.get(address).and_then(|k| k.name.clone())
    }

    pub async fn get_trust_score(&self, address: &str) -> Option<f64> {
        let kols = self.kols.read().await;
        kols.get(address).map(|k| k.trust_score)
    }

    pub async fn record_trade(&self, swap: ParsedSwap) -> AppResult<()> {
        let address = swap.wallet_address.clone();
        let volume = swap.input_amount;

        // Update KOL stats if tracked
        {
            let mut kols = self.kols.write().await;
            if let Some(kol) = kols.get_mut(&address) {
                kol.trade_count += 1;
                kol.total_volume_lamports += volume;
            }
        }

        // Store the trade
        {
            let mut trades = self.trades.write().await;
            trades.insert(0, swap);

            // Trim to max size
            if trades.len() > self.max_trades {
                trades.truncate(self.max_trades);
            }
        }

        Ok(())
    }

    pub async fn get_recent_trades(
        &self,
        wallet_address: Option<&str>,
        limit: usize,
    ) -> Vec<ParsedSwap> {
        let trades = self.trades.read().await;

        let filtered: Vec<_> = if let Some(addr) = wallet_address {
            trades
                .iter()
                .filter(|t| t.wallet_address == addr)
                .cloned()
                .collect()
        } else {
            trades.clone()
        };

        filtered.into_iter().take(limit).collect()
    }

    pub async fn is_tracked(&self, address: &str) -> bool {
        let kols = self.kols.read().await;
        kols.contains_key(address)
    }

    pub async fn get_all_kols(&self) -> Vec<(String, Option<String>, f64)> {
        let kols = self.kols.read().await;
        kols.values()
            .map(|k| (k.address.clone(), k.name.clone(), k.trust_score))
            .collect()
    }
}

impl Default for KOLTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_kol_tracker() {
        let tracker = KOLTracker::new();

        tracker.add_kol("wallet1", Some("TestKOL".to_string()), 0.8).await;

        assert_eq!(tracker.get_kol_name("wallet1").await, Some("TestKOL".to_string()));
        assert_eq!(tracker.get_trust_score("wallet1").await, Some(0.8));
        assert!(tracker.is_tracked("wallet1").await);
        assert!(!tracker.is_tracked("wallet2").await);
    }
}
