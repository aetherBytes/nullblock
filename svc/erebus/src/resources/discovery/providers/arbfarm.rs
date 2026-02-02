use reqwest::Client;
use serde_json::{json, Value};
use std::time::{Duration, Instant};
use tracing::info;

use crate::resources::discovery::aggregator::DiscoveryProvider;
use crate::resources::discovery::models::{
    DiscoveredAgent, DiscoveredProtocol, DiscoveredTool, HealthStatus, ProviderHealth, ToolCategory,
};

pub struct ArbFarmProvider {
    client: Client,
    base_url: String,
}

impl ArbFarmProvider {
    pub fn new() -> Self {
        let base_url = std::env::var("ARB_FARM_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:9007".to_string());

        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            base_url,
        }
    }

    fn is_hot_tool(name: &str) -> bool {
        name.starts_with("consensus_")
    }

    fn categorize_tool(name: &str) -> ToolCategory {
        ToolCategory::from_tool_name(name)
    }

    async fn discover_tools_impl(
        &self,
    ) -> Result<Vec<DiscoveredTool>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/mcp/jsonrpc", self.base_url);

        info!("🔧 Discovering ArbFarm MCP tools from {}", url);

        let init_response = self
            .client
            .post(&url)
            .json(&json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "protocolVersion": "2025-11-25",
                    "capabilities": {},
                    "clientInfo": {
                        "name": "erebus-discovery",
                        "version": "1.0"
                    }
                }
            }))
            .send()
            .await?;

        if !init_response.status().is_success() {
            return Err(format!("MCP initialize failed: {}", init_response.status()).into());
        }

        let tools_response = self
            .client
            .post(&url)
            .json(&json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/list",
                "params": {}
            }))
            .send()
            .await?;

        if !tools_response.status().is_success() {
            return Err(format!("MCP tools/list failed: {}", tools_response.status()).into());
        }

        let response_json: Value = tools_response.json().await?;

        let tools_array = response_json
            .get("result")
            .and_then(|r| r.get("tools"))
            .and_then(|t| t.as_array())
            .ok_or("No tools array in response")?;

        let mut discovered_tools = Vec::new();

        for tool in tools_array {
            let name = tool.get("name").and_then(|n| n.as_str()).unwrap_or("");
            let description = tool
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("");
            let input_schema = tool.get("inputSchema").cloned().unwrap_or(json!({}));

            if name.is_empty() {
                continue;
            }

            discovered_tools.push(DiscoveredTool {
                name: name.to_string(),
                description: description.to_string(),
                input_schema,
                category: Self::categorize_tool(name),
                is_hot: Self::is_hot_tool(name),
                provider: "arbfarm".to_string(),
                related_cow: Some("ArbFarm".to_string()),
                endpoint: format!("{}/mcp/jsonrpc", self.base_url),
            });
        }

        info!("✅ Discovered {} ArbFarm tools", discovered_tools.len());
        Ok(discovered_tools)
    }

    async fn discover_agents_impl(
        &self,
    ) -> Result<Vec<DiscoveredAgent>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/api/swarm/status", self.base_url);

        info!("🤖 Discovering ArbFarm swarm agents from {}", url);

        match self.client.get(&url).send().await {
            Ok(response) if response.status().is_success() => {
                let status: Value = response.json().await?;

                let mut agents = Vec::new();

                if let Some(swarm) = status.get("swarm").and_then(|s| s.as_object()) {
                    for (agent_name, agent_info) in swarm {
                        let is_active = agent_info
                            .get("active")
                            .and_then(|a| a.as_bool())
                            .unwrap_or(false);

                        agents.push(DiscoveredAgent {
                            name: format!("arbfarm-{}", agent_name),
                            agent_type: "specialized".to_string(),
                            status: if is_active {
                                HealthStatus::Healthy
                            } else {
                                HealthStatus::Unhealthy
                            },
                            capabilities: vec![agent_name.clone()],
                            endpoint: self.base_url.clone(),
                            provider: "arbfarm".to_string(),
                            description: Some(format!("ArbFarm {} agent", agent_name)),
                            model: None,
                        });
                    }
                }

                if let Some(executor) = status.get("executor").and_then(|e| e.as_object()) {
                    let is_active = executor
                        .get("active")
                        .and_then(|a| a.as_bool())
                        .unwrap_or(false);

                    agents.push(DiscoveredAgent {
                        name: "arbfarm-executor".to_string(),
                        agent_type: "specialized".to_string(),
                        status: if is_active {
                            HealthStatus::Healthy
                        } else {
                            HealthStatus::Unhealthy
                        },
                        capabilities: vec!["execution".to_string(), "trading".to_string()],
                        endpoint: self.base_url.clone(),
                        provider: "arbfarm".to_string(),
                        description: Some("ArbFarm autonomous execution agent".to_string()),
                        model: None,
                    });
                }

                info!("✅ Discovered {} ArbFarm agents", agents.len());
                Ok(agents)
            }
            Ok(_) => Ok(vec![]),
            Err(_) => Ok(vec![]),
        }
    }

    async fn discover_protocols_impl(
        &self,
    ) -> Result<Vec<DiscoveredProtocol>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(vec![DiscoveredProtocol {
            name: "ArbFarm MCP".to_string(),
            protocol_type: "mcp".to_string(),
            version: "2025-11-25".to_string(),
            endpoint: format!("{}/mcp/jsonrpc", self.base_url),
            provider: "arbfarm".to_string(),
            description: Some("ArbFarm MEV Strategy MCP Server with 97+ tools".to_string()),
        }])
    }

    async fn health_impl(&self) -> ProviderHealth {
        let start = Instant::now();
        let url = format!("{}/health", self.base_url);

        match self.client.get(&url).send().await {
            Ok(response) if response.status().is_success() => ProviderHealth {
                provider: "arbfarm".to_string(),
                status: HealthStatus::Healthy,
                latency_ms: Some(start.elapsed().as_millis() as u64),
                last_checked: chrono::Utc::now(),
                error: None,
            },
            Ok(response) => ProviderHealth {
                provider: "arbfarm".to_string(),
                status: HealthStatus::Degraded,
                latency_ms: Some(start.elapsed().as_millis() as u64),
                last_checked: chrono::Utc::now(),
                error: Some(format!("HTTP {}", response.status())),
            },
            Err(e) => ProviderHealth {
                provider: "arbfarm".to_string(),
                status: HealthStatus::Unhealthy,
                latency_ms: Some(start.elapsed().as_millis() as u64),
                last_checked: chrono::Utc::now(),
                error: Some(e.to_string()),
            },
        }
    }
}

impl DiscoveryProvider for ArbFarmProvider {
    fn name(&self) -> &str {
        "arbfarm"
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
