use reqwest::Client;
use serde_json::Value;
use std::time::{Duration, Instant};
use tracing::info;

use crate::resources::discovery::aggregator::DiscoveryProvider;
use crate::resources::discovery::models::{
    DiscoveredAgent, DiscoveredProtocol, DiscoveredTool, HealthStatus, ProviderHealth,
};

pub struct AgentsProvider {
    client: Client,
    agents_url: String,
}

impl AgentsProvider {
    pub fn new() -> Self {
        let agents_url = std::env::var("AGENTS_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:9003".to_string());

        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            agents_url,
        }
    }

    async fn check_hecate(&self) -> Option<DiscoveredAgent> {
        let url = format!("{}/hecate/health", self.agents_url);

        match self.client.get(&url).send().await {
            Ok(response) if response.status().is_success() => {
                let status: Value = response.json().await.ok()?;

                let model = status
                    .get("model")
                    .and_then(|m| m.as_str())
                    .map(|s| s.to_string());

                Some(DiscoveredAgent {
                    name: "HECATE".to_string(),
                    agent_type: "conversational".to_string(),
                    status: HealthStatus::Healthy,
                    capabilities: vec![
                        "chat".to_string(),
                        "code_generation".to_string(),
                        "task_management".to_string(),
                        "personality_modes".to_string(),
                    ],
                    endpoint: self.agents_url.clone(),
                    provider: "agents".to_string(),
                    description: Some("Von Neumann-class vessel AI for conversational assistance".to_string()),
                    model,
                })
            }
            Ok(_) => {
                Some(DiscoveredAgent {
                    name: "HECATE".to_string(),
                    agent_type: "conversational".to_string(),
                    status: HealthStatus::Unhealthy,
                    capabilities: vec![],
                    endpoint: self.agents_url.clone(),
                    provider: "agents".to_string(),
                    description: Some("Von Neumann-class vessel AI (currently unavailable)".to_string()),
                    model: None,
                })
            }
            Err(_) => None,
        }
    }

    async fn check_siren(&self) -> Option<DiscoveredAgent> {
        let url = format!("{}/siren/health", self.agents_url);

        match self.client.get(&url).send().await {
            Ok(response) if response.status().is_success() => {
                let status: Value = response.json().await.ok()?;

                let model = status
                    .get("model")
                    .and_then(|m| m.as_str())
                    .map(|s| s.to_string());

                Some(DiscoveredAgent {
                    name: "Siren".to_string(),
                    agent_type: "specialized".to_string(),
                    status: HealthStatus::Healthy,
                    capabilities: vec![
                        "code_review".to_string(),
                        "technical_analysis".to_string(),
                        "documentation".to_string(),
                    ],
                    endpoint: self.agents_url.clone(),
                    provider: "agents".to_string(),
                    description: Some("Specialized agent for technical tasks".to_string()),
                    model,
                })
            }
            Ok(_) => {
                Some(DiscoveredAgent {
                    name: "Siren".to_string(),
                    agent_type: "specialized".to_string(),
                    status: HealthStatus::Unhealthy,
                    capabilities: vec![],
                    endpoint: self.agents_url.clone(),
                    provider: "agents".to_string(),
                    description: Some("Specialized agent (currently unavailable)".to_string()),
                    model: None,
                })
            }
            Err(_) => None,
        }
    }

    async fn discover_tools_impl(&self) -> Result<Vec<DiscoveredTool>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(vec![])
    }

    async fn discover_agents_impl(&self) -> Result<Vec<DiscoveredAgent>, Box<dyn std::error::Error + Send + Sync>> {
        info!("🤖 Discovering agents from {}", self.agents_url);

        let mut agents = Vec::new();

        if let Some(hecate) = self.check_hecate().await {
            agents.push(hecate);
        }

        if let Some(siren) = self.check_siren().await {
            agents.push(siren);
        }

        info!("✅ Discovered {} agents", agents.len());
        Ok(agents)
    }

    async fn discover_protocols_impl(&self) -> Result<Vec<DiscoveredProtocol>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(vec![])
    }

    async fn health_impl(&self) -> ProviderHealth {
        let start = Instant::now();
        let url = format!("{}/health", self.agents_url);

        match self.client.get(&url).send().await {
            Ok(response) if response.status().is_success() => ProviderHealth {
                provider: "agents".to_string(),
                status: HealthStatus::Healthy,
                latency_ms: Some(start.elapsed().as_millis() as u64),
                last_checked: chrono::Utc::now(),
                error: None,
            },
            Ok(response) => ProviderHealth {
                provider: "agents".to_string(),
                status: HealthStatus::Degraded,
                latency_ms: Some(start.elapsed().as_millis() as u64),
                last_checked: chrono::Utc::now(),
                error: Some(format!("HTTP {}", response.status())),
            },
            Err(e) => ProviderHealth {
                provider: "agents".to_string(),
                status: HealthStatus::Unhealthy,
                latency_ms: Some(start.elapsed().as_millis() as u64),
                last_checked: chrono::Utc::now(),
                error: Some(e.to_string()),
            },
        }
    }
}

impl DiscoveryProvider for AgentsProvider {
    fn name(&self) -> &str {
        "agents"
    }

    fn discover_tools(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<DiscoveredTool>, Box<dyn std::error::Error + Send + Sync>>> + Send + '_>> {
        Box::pin(self.discover_tools_impl())
    }

    fn discover_agents(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<DiscoveredAgent>, Box<dyn std::error::Error + Send + Sync>>> + Send + '_>> {
        Box::pin(self.discover_agents_impl())
    }

    fn discover_protocols(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<DiscoveredProtocol>, Box<dyn std::error::Error + Send + Sync>>> + Send + '_>> {
        Box::pin(self.discover_protocols_impl())
    }

    fn health(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = ProviderHealth> + Send + '_>> {
        Box::pin(self.health_impl())
    }
}
