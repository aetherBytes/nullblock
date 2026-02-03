use reqwest::Client;
use serde_json::{json, Value};
use std::time::{Duration, Instant};
use tracing::info;

use crate::resources::discovery::aggregator::DiscoveryProvider;
use crate::resources::discovery::models::{
    DiscoveredAgent, DiscoveredProtocol, DiscoveredTool, HealthStatus, ProviderHealth, ToolCategory,
};

pub struct McpProvider {
    client: Client,
    name: String,
    base_url: String,
    related_cow: Option<String>,
}

impl McpProvider {
    pub fn new(name: &str, base_url: &str, related_cow: Option<String>) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            name: name.to_string(),
            base_url: base_url.to_string(),
            related_cow,
        }
    }

    pub fn from_env(
        name: &str,
        env_var: &str,
        default_port: u16,
        related_cow: Option<String>,
    ) -> Self {
        let base_url =
            std::env::var(env_var).unwrap_or_else(|_| format!("http://localhost:{}", default_port));

        Self::new(name, &base_url, related_cow)
    }

    async fn discover_tools_impl(
        &self,
    ) -> Result<Vec<DiscoveredTool>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/mcp/jsonrpc", self.base_url);

        info!("🔧 Discovering MCP tools from {} ({})", self.name, url);

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
                category: ToolCategory::from_tool_name(name),
                is_hot: false,
                provider: self.name.clone(),
                related_cow: self.related_cow.clone(),
                endpoint: url.clone(),
            });
        }

        info!(
            "✅ Discovered {} tools from {}",
            discovered_tools.len(),
            self.name
        );
        Ok(discovered_tools)
    }

    async fn discover_agents_impl(
        &self,
    ) -> Result<Vec<DiscoveredAgent>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(vec![])
    }

    async fn discover_protocols_impl(
        &self,
    ) -> Result<Vec<DiscoveredProtocol>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/mcp/jsonrpc", self.base_url);

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
            .await;

        match init_response {
            Ok(response) if response.status().is_success() => {
                let response_json: Value = response.json().await.unwrap_or_default();
                let version = response_json
                    .get("result")
                    .and_then(|r| r.get("protocolVersion"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();

                Ok(vec![DiscoveredProtocol {
                    name: format!("{} MCP", self.name),
                    protocol_type: "mcp".to_string(),
                    version,
                    endpoint: url,
                    provider: self.name.clone(),
                    description: self
                        .related_cow
                        .as_ref()
                        .map(|cow| format!("MCP server for {}", cow)),
                }])
            }
            _ => Ok(vec![]),
        }
    }

    async fn health_impl(&self) -> ProviderHealth {
        let start = Instant::now();
        let url = format!("{}/health", self.base_url);

        match self.client.get(&url).send().await {
            Ok(response) if response.status().is_success() => ProviderHealth {
                provider: self.name.clone(),
                status: HealthStatus::Healthy,
                latency_ms: Some(start.elapsed().as_millis() as u64),
                last_checked: chrono::Utc::now(),
                error: None,
            },
            Ok(response) => ProviderHealth {
                provider: self.name.clone(),
                status: HealthStatus::Degraded,
                latency_ms: Some(start.elapsed().as_millis() as u64),
                last_checked: chrono::Utc::now(),
                error: Some(format!("HTTP {}", response.status())),
            },
            Err(e) => ProviderHealth {
                provider: self.name.clone(),
                status: HealthStatus::Unhealthy,
                latency_ms: Some(start.elapsed().as_millis() as u64),
                last_checked: chrono::Utc::now(),
                error: Some(e.to_string()),
            },
        }
    }
}

impl DiscoveryProvider for McpProvider {
    fn name(&self) -> &str {
        &self.name
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
