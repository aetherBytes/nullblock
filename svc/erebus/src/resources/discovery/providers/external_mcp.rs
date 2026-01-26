use nullblock_mcp_client::{
    AuthConfig, McpServerConfig, RegistryStats, ServiceEndpoint, ServiceRegistry,
};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::resources::discovery::aggregator::DiscoveryProvider;
use crate::resources::discovery::models::{
    DiscoveredAgent, DiscoveredProtocol, DiscoveredTool, HealthStatus, ProviderHealth, ToolCategory,
};

const CONFIG_FILE_PATH: &str = "config/mcp-services.toml";
const ENV_VAR_EXTERNAL_MCP: &str = "EXTERNAL_MCP_SERVICES";

pub struct ExternalMcpProvider {
    registry: Arc<RwLock<ServiceRegistry>>,
    name: String,
}

impl ExternalMcpProvider {
    pub fn new() -> Self {
        Self {
            registry: Arc::new(RwLock::new(ServiceRegistry::new())),
            name: "external-mcp".to_string(),
        }
    }

    pub async fn initialize(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("🌐 Initializing external MCP provider");

        let configs = self.load_external_configs()?;
        if configs.is_empty() {
            info!("No external MCP services configured");
            return Ok(());
        }

        let registry = self.registry.write().await;
        for config in configs {
            info!(
                name = %config.name,
                url = %config.url,
                has_auth = config.auth.has_auth(),
                "Registering external MCP service"
            );
            if let Err(e) = registry.register_with_config(config.clone()).await {
                warn!(
                    name = %config.name,
                    error = %e,
                    "Failed to register external MCP service"
                );
            }
        }

        info!("✅ External MCP provider initialized");
        Ok(())
    }

    fn load_external_configs(&self) -> Result<Vec<McpServerConfig>, Box<dyn std::error::Error + Send + Sync>> {
        let mut configs = Vec::new();

        if Path::new(CONFIG_FILE_PATH).exists() {
            match self.load_from_file(CONFIG_FILE_PATH) {
                Ok(file_configs) => {
                    info!(count = file_configs.len(), "Loaded configs from file");
                    configs.extend(file_configs);
                }
                Err(e) => {
                    warn!(error = %e, "Failed to load config file");
                }
            }
        }

        if let Ok(env_configs) = self.load_from_env() {
            info!(count = env_configs.len(), "Loaded configs from environment");
            configs.extend(env_configs);
        }

        configs.retain(|c| !c.url.contains("localhost") || c.url.starts_with("https://"));

        Ok(configs)
    }

    fn load_from_file(&self, path: &str) -> Result<Vec<McpServerConfig>, Box<dyn std::error::Error + Send + Sync>> {
        let content = std::fs::read_to_string(path)?;
        let parsed: toml::Value = toml::from_str(&content)?;

        let mut configs = Vec::new();

        if let Some(services) = parsed.get("services").and_then(|s| s.as_table()) {
            for (name, service) in services {
                let url = service.get("url").and_then(|u| u.as_str()).unwrap_or("");
                let enabled = service.get("enabled").and_then(|e| e.as_bool()).unwrap_or(true);

                if url.is_empty() || !enabled {
                    continue;
                }

                let url = self.resolve_env_var(url);

                let mut auth = AuthConfig::default();

                if let Some(auth_section) = service.get("auth").and_then(|a| a.as_table()) {
                    if let Some(key) = auth_section.get("api_key").and_then(|k| k.as_str()) {
                        auth.api_key = Some(key.to_string());
                    }
                    if let Some(env_var) = auth_section.get("api_key_env").and_then(|k| k.as_str()) {
                        if let Ok(key) = std::env::var(env_var) {
                            auth.api_key = Some(key);
                        }
                    }
                    if let Some(header) = auth_section.get("api_key_header").and_then(|h| h.as_str()) {
                        auth.api_key_header = Some(header.to_string());
                    }
                    if let Some(token) = auth_section.get("bearer_token").and_then(|t| t.as_str()) {
                        auth.bearer_token = Some(token.to_string());
                    }
                    if let Some(env_var) = auth_section.get("bearer_token_env").and_then(|t| t.as_str()) {
                        if let Ok(token) = std::env::var(env_var) {
                            auth.bearer_token = Some(token);
                        }
                    }
                }

                let description = service.get("description").and_then(|d| d.as_str()).map(String::from);
                let tags: Vec<String> = service
                    .get("tags")
                    .and_then(|t| t.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_default();

                configs.push(McpServerConfig {
                    name: name.clone(),
                    url,
                    auth,
                    enabled: true,
                    description,
                    tags,
                    timeout_secs: 30,
                    cache_ttl_secs: 300,
                    health_check_interval_secs: None,
                });
            }
        }

        Ok(configs)
    }

    fn load_from_env(&self) -> Result<Vec<McpServerConfig>, Box<dyn std::error::Error + Send + Sync>> {
        let env_value = std::env::var(ENV_VAR_EXTERNAL_MCP)?;
        let configs: Vec<McpServerConfig> = serde_json::from_str(&env_value)?;
        Ok(configs)
    }

    fn resolve_env_var(&self, value: &str) -> String {
        if value.starts_with("${") && value.ends_with("}") {
            let env_var = &value[2..value.len() - 1];
            std::env::var(env_var).unwrap_or_else(|_| value.to_string())
        } else {
            value.to_string()
        }
    }

    async fn discover_tools_impl(&self) -> Result<Vec<DiscoveredTool>, Box<dyn std::error::Error + Send + Sync>> {
        let registry = self.registry.read().await;
        let services = registry.list_services().await;

        if services.is_empty() {
            debug!("No external MCP services registered");
            return Ok(vec![]);
        }

        info!("🔧 Discovering tools from {} external MCP services", services.len());
        drop(registry);

        let mut all_tools = Vec::new();

        let registry = self.registry.write().await;
        let mcp_tools = match registry.discover_all_tools().await {
            Ok(tools) => tools,
            Err(e) => {
                warn!(error = %e, "Failed to discover tools from registry");
                vec![]
            }
        };
        drop(registry);

        let registry = self.registry.read().await;
        for tool in mcp_tools {
            let service_name = registry.get_service_for_tool(&tool.name).await;
            let service = if let Some(ref name) = service_name {
                registry.get_service(name).await
            } else {
                None
            };

            let (endpoint, related_cow, is_external) = if let Some(svc) = service {
                (svc.url.clone(), svc.description.clone(), svc.is_remote)
            } else {
                ("unknown".to_string(), None, true)
            };

            if !is_external {
                continue;
            }

            let discovered_tool = DiscoveredTool {
                name: tool.name.clone(),
                description: tool.description.clone().unwrap_or_default(),
                input_schema: tool.input_schema.clone(),
                category: ToolCategory::from_tool_name(&tool.name),
                is_hot: false,
                provider: format!("external/{}", service_name.as_deref().unwrap_or("unknown")),
                related_cow,
                endpoint,
            };

            all_tools.push(discovered_tool);
        }

        info!("✅ Discovered {} tools from external MCP services", all_tools.len());
        Ok(all_tools)
    }

    async fn discover_agents_impl(&self) -> Result<Vec<DiscoveredAgent>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(vec![])
    }

    async fn discover_protocols_impl(&self) -> Result<Vec<DiscoveredProtocol>, Box<dyn std::error::Error + Send + Sync>> {
        let registry = self.registry.read().await;
        let services = registry.list_services().await;

        let mut protocols = Vec::new();

        for service in services {
            if !service.is_remote {
                continue;
            }

            let version = service
                .server_info
                .as_ref()
                .map(|info| info.version.clone())
                .unwrap_or_else(|| "unknown".to_string());

            protocols.push(DiscoveredProtocol {
                name: format!("{} MCP", service.name),
                protocol_type: "mcp".to_string(),
                version,
                endpoint: service.url.clone(),
                provider: "external-mcp".to_string(),
                description: service.description.clone(),
            });
        }

        Ok(protocols)
    }

    async fn health_impl(&self) -> ProviderHealth {
        let start = Instant::now();
        let registry = self.registry.read().await;
        let stats = registry.stats().await;

        if stats.service_count == 0 {
            return ProviderHealth {
                provider: self.name.clone(),
                status: HealthStatus::Unknown,
                latency_ms: Some(start.elapsed().as_millis() as u64),
                last_checked: chrono::Utc::now(),
                error: Some("No external MCP services configured".to_string()),
            };
        }

        let services = registry.list_remote_services().await;
        let mut healthy = 0;

        for service in &services {
            if let Some(client) = registry.get_client(&service.name).await {
                match client.health_check().await {
                    Ok(true) => healthy += 1,
                    _ => {}
                }
            }
        }

        let status = if healthy == services.len() {
            HealthStatus::Healthy
        } else if healthy > 0 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Unhealthy
        };

        ProviderHealth {
            provider: self.name.clone(),
            status,
            latency_ms: Some(start.elapsed().as_millis() as u64),
            last_checked: chrono::Utc::now(),
            error: None,
        }
    }

    pub async fn get_registry_stats(&self) -> RegistryStats {
        self.registry.read().await.stats().await
    }

    pub async fn list_services(&self) -> Vec<ServiceEndpoint> {
        self.registry.read().await.list_services().await
    }

    pub async fn register_service(&self, config: McpServerConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let registry = self.registry.write().await;
        registry.register_with_config(config).await?;
        Ok(())
    }

    pub async fn unregister_service(&self, name: &str) {
        let registry = self.registry.write().await;
        registry.unregister(name).await;
    }

    pub async fn refresh_tools(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let registry = self.registry.write().await;
        let tools = registry.discover_all_tools().await?;
        Ok(tools.len())
    }
}

impl Default for ExternalMcpProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl DiscoveryProvider for ExternalMcpProvider {
    fn name(&self) -> &str {
        &self.name
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
