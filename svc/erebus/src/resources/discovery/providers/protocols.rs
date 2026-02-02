use reqwest::Client;
use serde_json::Value;
use std::time::{Duration, Instant};
use tracing::info;

use crate::resources::discovery::aggregator::DiscoveryProvider;
use crate::resources::discovery::models::{
    DiscoveredAgent, DiscoveredProtocol, DiscoveredTool, HealthStatus, ProviderHealth,
};

pub struct ProtocolsProvider {
    client: Client,
    protocols_url: String,
}

impl ProtocolsProvider {
    pub fn new() -> Self {
        let protocols_url = std::env::var("PROTOCOLS_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:8001".to_string());

        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            protocols_url,
        }
    }

    async fn discover_tools_impl(
        &self,
    ) -> Result<Vec<DiscoveredTool>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(vec![])
    }

    async fn discover_agents_impl(
        &self,
    ) -> Result<Vec<DiscoveredAgent>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(vec![])
    }

    async fn discover_protocols_impl(
        &self,
    ) -> Result<Vec<DiscoveredProtocol>, Box<dyn std::error::Error + Send + Sync>> {
        info!("🔌 Discovering protocols from {}", self.protocols_url);

        let mut protocols = Vec::new();

        let mcp_url = format!("{}/mcp/health", self.protocols_url);
        match self.client.get(&mcp_url).send().await {
            Ok(response) if response.status().is_success() => {
                let _status: Value = response.json().await.unwrap_or_default();

                protocols.push(DiscoveredProtocol {
                    name: "NullBlock MCP".to_string(),
                    protocol_type: "mcp".to_string(),
                    version: "2025-11-25".to_string(),
                    endpoint: format!("{}/mcp/jsonrpc", self.protocols_url),
                    provider: "protocols".to_string(),
                    description: Some("Core NullBlock MCP server".to_string()),
                });
            }
            _ => {}
        }

        let a2a_url = format!("{}/a2a/health", self.protocols_url);
        match self.client.get(&a2a_url).send().await {
            Ok(response) if response.status().is_success() => {
                protocols.push(DiscoveredProtocol {
                    name: "A2A Protocol".to_string(),
                    protocol_type: "a2a".to_string(),
                    version: "1.0".to_string(),
                    endpoint: format!("{}/a2a", self.protocols_url),
                    provider: "protocols".to_string(),
                    description: Some("Agent-to-Agent communication protocol".to_string()),
                });
            }
            _ => {}
        }

        info!("✅ Discovered {} protocols", protocols.len());
        Ok(protocols)
    }

    async fn health_impl(&self) -> ProviderHealth {
        let start = Instant::now();
        let url = format!("{}/health", self.protocols_url);

        match self.client.get(&url).send().await {
            Ok(response) if response.status().is_success() => ProviderHealth {
                provider: "protocols".to_string(),
                status: HealthStatus::Healthy,
                latency_ms: Some(start.elapsed().as_millis() as u64),
                last_checked: chrono::Utc::now(),
                error: None,
            },
            Ok(response) => ProviderHealth {
                provider: "protocols".to_string(),
                status: HealthStatus::Degraded,
                latency_ms: Some(start.elapsed().as_millis() as u64),
                last_checked: chrono::Utc::now(),
                error: Some(format!("HTTP {}", response.status())),
            },
            Err(e) => ProviderHealth {
                provider: "protocols".to_string(),
                status: HealthStatus::Unhealthy,
                latency_ms: Some(start.elapsed().as_millis() as u64),
                last_checked: chrono::Utc::now(),
                error: Some(e.to_string()),
            },
        }
    }
}

impl DiscoveryProvider for ProtocolsProvider {
    fn name(&self) -> &str {
        "protocols"
    }

    fn discover_tools(
        &self,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<Vec<DiscoveredTool>, Box<dyn std::error::Error + Send + Sync>>,
                > + Send
                + '_,
        >,
    > {
        Box::pin(self.discover_tools_impl())
    }

    fn discover_agents(
        &self,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<Vec<DiscoveredAgent>, Box<dyn std::error::Error + Send + Sync>>,
                > + Send
                + '_,
        >,
    > {
        Box::pin(self.discover_agents_impl())
    }

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
    > {
        Box::pin(self.discover_protocols_impl())
    }

    fn health(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ProviderHealth> + Send + '_>> {
        Box::pin(self.health_impl())
    }
}
