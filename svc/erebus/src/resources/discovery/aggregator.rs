use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn};

use super::models::{
    CategorySummary, DiscoveredAgent, DiscoveredProtocol, DiscoveredTool, DiscoveryResponse,
    HealthStatus, ProviderHealth, ToolCategory,
};

pub trait DiscoveryProvider: Send + Sync {
    fn name(&self) -> &str;
    fn discover_tools(
        &self,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<Vec<DiscoveredTool>, Box<dyn std::error::Error + Send + Sync>>,
                > + Send
                + '_,
        >,
    >;
    fn discover_agents(
        &self,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<Vec<DiscoveredAgent>, Box<dyn std::error::Error + Send + Sync>>,
                > + Send
                + '_,
        >,
    >;
    fn discover_protocols(
        &self,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<
                        Vec<DiscoveredProtocol>,
                        Box<dyn std::error::Error + Send + Sync>,
                    >,
                > + Send
                + '_,
        >,
    >;
    fn health(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ProviderHealth> + Send + '_>>;
}

struct CacheEntry<T> {
    data: T,
    expires_at: Instant,
}

pub struct DiscoveryAggregator {
    providers: Vec<Arc<dyn DiscoveryProvider>>,
    tools_cache: Arc<RwLock<Option<CacheEntry<Vec<DiscoveredTool>>>>>,
    agents_cache: Arc<RwLock<Option<CacheEntry<Vec<DiscoveredAgent>>>>>,
    protocols_cache: Arc<RwLock<Option<CacheEntry<Vec<DiscoveredProtocol>>>>>,
    cache_ttl: Duration,
}

impl DiscoveryAggregator {
    pub fn new(providers: Vec<Arc<dyn DiscoveryProvider>>) -> Self {
        Self {
            providers,
            tools_cache: Arc::new(RwLock::new(None)),
            agents_cache: Arc::new(RwLock::new(None)),
            protocols_cache: Arc::new(RwLock::new(None)),
            cache_ttl: Duration::from_secs(60),
        }
    }

    pub async fn discover_all(&self) -> DiscoveryResponse {
        let start = Instant::now();

        let (tools, agents, protocols) = tokio::join!(
            self.discover_tools(),
            self.discover_agents(),
            self.discover_protocols()
        );

        let hot: Vec<DiscoveredTool> = tools.iter().filter(|t| t.is_hot).cloned().collect();

        let mut provider_health = Vec::new();
        for provider in &self.providers {
            provider_health.push(provider.health().await);
        }

        DiscoveryResponse {
            tools,
            agents,
            protocols,
            hot,
            provider_health,
            discovery_time_ms: start.elapsed().as_millis() as u64,
        }
    }

    pub async fn discover_tools(&self) -> Vec<DiscoveredTool> {
        {
            let cache = self.tools_cache.read().await;
            if let Some(entry) = cache.as_ref() {
                if entry.expires_at > Instant::now() {
                    info!("📦 Returning cached tools");
                    return entry.data.clone();
                }
            }
        }

        info!(
            "🔍 Discovering tools from {} providers",
            self.providers.len()
        );
        let mut all_tools = Vec::new();

        for provider in &self.providers {
            match provider.discover_tools().await {
                Ok(tools) => {
                    info!(
                        "✅ Provider {} returned {} tools",
                        provider.name(),
                        tools.len()
                    );
                    all_tools.extend(tools);
                }
                Err(e) => {
                    warn!(
                        "❌ Provider {} failed to discover tools: {}",
                        provider.name(),
                        e
                    );
                }
            }
        }

        {
            let mut cache = self.tools_cache.write().await;
            *cache = Some(CacheEntry {
                data: all_tools.clone(),
                expires_at: Instant::now() + self.cache_ttl,
            });
        }

        all_tools
    }

    pub async fn discover_agents(&self) -> Vec<DiscoveredAgent> {
        {
            let cache = self.agents_cache.read().await;
            if let Some(entry) = cache.as_ref() {
                if entry.expires_at > Instant::now() {
                    info!("📦 Returning cached agents");
                    return entry.data.clone();
                }
            }
        }

        info!(
            "🤖 Discovering agents from {} providers",
            self.providers.len()
        );
        let mut all_agents = Vec::new();

        for provider in &self.providers {
            match provider.discover_agents().await {
                Ok(agents) => {
                    info!(
                        "✅ Provider {} returned {} agents",
                        provider.name(),
                        agents.len()
                    );
                    all_agents.extend(agents);
                }
                Err(e) => {
                    warn!(
                        "❌ Provider {} failed to discover agents: {}",
                        provider.name(),
                        e
                    );
                }
            }
        }

        {
            let mut cache = self.agents_cache.write().await;
            *cache = Some(CacheEntry {
                data: all_agents.clone(),
                expires_at: Instant::now() + self.cache_ttl,
            });
        }

        all_agents
    }

    pub async fn discover_protocols(&self) -> Vec<DiscoveredProtocol> {
        {
            let cache = self.protocols_cache.read().await;
            if let Some(entry) = cache.as_ref() {
                if entry.expires_at > Instant::now() {
                    info!("📦 Returning cached protocols");
                    return entry.data.clone();
                }
            }
        }

        info!(
            "🔌 Discovering protocols from {} providers",
            self.providers.len()
        );
        let mut all_protocols = Vec::new();

        for provider in &self.providers {
            match provider.discover_protocols().await {
                Ok(protocols) => {
                    info!(
                        "✅ Provider {} returned {} protocols",
                        provider.name(),
                        protocols.len()
                    );
                    all_protocols.extend(protocols);
                }
                Err(e) => {
                    warn!(
                        "❌ Provider {} failed to discover protocols: {}",
                        provider.name(),
                        e
                    );
                }
            }
        }

        {
            let mut cache = self.protocols_cache.write().await;
            *cache = Some(CacheEntry {
                data: all_protocols.clone(),
                expires_at: Instant::now() + self.cache_ttl,
            });
        }

        all_protocols
    }

    pub async fn get_hot_items(&self) -> Vec<DiscoveredTool> {
        let tools = self.discover_tools().await;
        tools.into_iter().filter(|t| t.is_hot).collect()
    }

    pub async fn get_provider_health(&self) -> Vec<ProviderHealth> {
        let mut health = Vec::new();
        for provider in &self.providers {
            health.push(provider.health().await);
        }
        health
    }

    pub fn get_category_summary(tools: &[DiscoveredTool]) -> Vec<CategorySummary> {
        let mut counts: HashMap<ToolCategory, usize> = HashMap::new();

        for tool in tools {
            *counts.entry(tool.category.clone()).or_insert(0) += 1;
        }

        let mut summaries: Vec<CategorySummary> = counts
            .into_iter()
            .map(|(category, count)| CategorySummary {
                icon: category.icon().to_string(),
                category,
                count,
            })
            .collect();

        summaries.sort_by(|a, b| b.count.cmp(&a.count));
        summaries
    }

    pub fn get_overall_health(health: &[ProviderHealth]) -> HealthStatus {
        if health.is_empty() {
            return HealthStatus::Unknown;
        }

        let healthy_count = health
            .iter()
            .filter(|h| matches!(h.status, HealthStatus::Healthy))
            .count();
        let total = health.len();

        if healthy_count == total {
            HealthStatus::Healthy
        } else if healthy_count > 0 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Unhealthy
        }
    }

    pub async fn invalidate_cache(&self) {
        {
            let mut cache = self.tools_cache.write().await;
            *cache = None;
        }
        {
            let mut cache = self.agents_cache.write().await;
            *cache = None;
        }
        {
            let mut cache = self.protocols_cache.write().await;
            *cache = None;
        }
        info!("🗑️ Discovery cache invalidated");
    }
}
