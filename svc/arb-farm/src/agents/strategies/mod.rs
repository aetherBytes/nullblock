pub mod venue_snapshot;
pub mod volume_hunter;
pub mod graduation_sniper_strategy;

pub use venue_snapshot::{TokenData, VenueSnapshot};
pub use volume_hunter::VolumeHunterStrategy;
pub use graduation_sniper_strategy::GraduationSniperStrategy;

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::AppResult;
use crate::models::{Edge, RiskParams, Signal, VenueType};

#[async_trait]
pub trait BehavioralStrategy: Send + Sync {
    fn strategy_type(&self) -> &str;

    fn name(&self) -> &str;

    fn supported_venues(&self) -> Vec<VenueType>;

    async fn scan(&self, snapshot: &VenueSnapshot) -> AppResult<Vec<Signal>>;

    fn create_edge(&self, _signal: &Signal, _risk: &RiskParams) -> Option<Edge> {
        None
    }

    fn is_active(&self) -> bool;

    async fn set_active(&self, active: bool);
}

pub struct StrategyRegistry {
    strategies: Arc<RwLock<Vec<Arc<dyn BehavioralStrategy>>>>,
}

impl StrategyRegistry {
    pub fn new() -> Self {
        Self {
            strategies: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn register(&self, strategy: Arc<dyn BehavioralStrategy>) {
        let mut strategies = self.strategies.write().await;
        strategies.push(strategy);
    }

    pub async fn list(&self) -> Vec<Arc<dyn BehavioralStrategy>> {
        self.strategies.read().await.clone()
    }

    pub async fn get_active(&self) -> Vec<Arc<dyn BehavioralStrategy>> {
        let strategies = self.strategies.read().await;
        strategies.iter()
            .filter(|s| s.is_active())
            .cloned()
            .collect()
    }

    pub async fn get_by_venue(&self, venue_type: VenueType) -> Vec<Arc<dyn BehavioralStrategy>> {
        let strategies = self.strategies.read().await;
        strategies.iter()
            .filter(|s| s.is_active() && s.supported_venues().contains(&venue_type))
            .cloned()
            .collect()
    }

    pub async fn toggle(&self, name: &str, active: bool) -> bool {
        let strategies = self.strategies.read().await;
        for strategy in strategies.iter() {
            if strategy.name() == name {
                strategy.set_active(active).await;
                return true;
            }
        }
        false
    }

    pub async fn count(&self) -> usize {
        self.strategies.read().await.len()
    }

    pub async fn active_count(&self) -> usize {
        let strategies = self.strategies.read().await;
        strategies.iter().filter(|s| s.is_active()).count()
    }
}

impl Default for StrategyRegistry {
    fn default() -> Self {
        Self::new()
    }
}
