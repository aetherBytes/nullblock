use crate::client::{AuthConfig, McpClient, McpServerConfig};
use crate::error::{McpError, McpResult};
use crate::filter::{filter_read_only, ToolFilter};
use crate::types::{CallToolResult, ClientInfo, McpTool, ServerCapabilities, ServerInfo};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

#[derive(Debug, Clone)]
pub struct ServiceEndpoint {
    pub name: String,
    pub url: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub capabilities: Option<ServerCapabilities>,
    pub server_info: Option<ServerInfo>,
    pub tools: Vec<McpTool>,
    pub last_refreshed: Option<Instant>,
    pub enabled: bool,
    pub is_remote: bool,
    pub has_auth: bool,
}

impl ServiceEndpoint {
    pub fn new(name: impl Into<String>, url: impl Into<String>) -> Self {
        let url_str: String = url.into();
        let is_remote = url_str.starts_with("https://") || !url_str.contains("localhost");
        Self {
            name: name.into(),
            url: url_str,
            description: None,
            tags: Vec::new(),
            capabilities: None,
            server_info: None,
            tools: Vec::new(),
            last_refreshed: None,
            enabled: true,
            is_remote,
            has_auth: false,
        }
    }

    pub fn from_server_config(config: &McpServerConfig) -> Self {
        let is_remote = config.url.starts_with("https://") || !config.url.contains("localhost");
        Self {
            name: config.name.clone(),
            url: config.url.clone(),
            description: config.description.clone(),
            tags: config.tags.clone(),
            capabilities: None,
            server_info: None,
            tools: Vec::new(),
            last_refreshed: None,
            enabled: config.enabled,
            is_remote,
            has_auth: config.auth.has_auth(),
        }
    }

    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }

    pub fn read_only_tool_count(&self) -> usize {
        filter_read_only(self.tools.clone()).len()
    }
}

pub struct ServiceRegistry {
    services: RwLock<HashMap<String, ServiceEndpoint>>,
    clients: RwLock<HashMap<String, Arc<McpClient>>>,
    tool_service_map: RwLock<HashMap<String, String>>,
    client_info: ClientInfo,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: RwLock::new(HashMap::new()),
            clients: RwLock::new(HashMap::new()),
            tool_service_map: RwLock::new(HashMap::new()),
            client_info: ClientInfo::default(),
        }
    }

    pub fn with_client_info(client_info: ClientInfo) -> Self {
        Self {
            services: RwLock::new(HashMap::new()),
            clients: RwLock::new(HashMap::new()),
            tool_service_map: RwLock::new(HashMap::new()),
            client_info,
        }
    }

    pub fn with_endpoints(endpoints: Vec<(&str, &str)>) -> Self {
        let registry = Self::new();

        let services: HashMap<String, ServiceEndpoint> = endpoints
            .into_iter()
            .map(|(name, url)| (name.to_string(), ServiceEndpoint::new(name, url)))
            .collect();

        let clients: HashMap<String, Arc<McpClient>> = services
            .iter()
            .map(|(name, endpoint)| (name.clone(), Arc::new(McpClient::new(&endpoint.url))))
            .collect();

        *futures::executor::block_on(registry.services.write()) = services;
        *futures::executor::block_on(registry.clients.write()) = clients;

        registry
    }

    pub fn from_server_configs(configs: Vec<McpServerConfig>) -> Self {
        let registry = Self::new();

        let services: HashMap<String, ServiceEndpoint> = configs
            .iter()
            .map(|cfg| (cfg.name.clone(), ServiceEndpoint::from_server_config(cfg)))
            .collect();

        let clients: HashMap<String, Arc<McpClient>> = configs
            .iter()
            .map(|cfg| {
                let client = McpClient::from_server_config(cfg, registry.client_info.clone());
                (cfg.name.clone(), Arc::new(client))
            })
            .collect();

        *futures::executor::block_on(registry.services.write()) = services;
        *futures::executor::block_on(registry.clients.write()) = clients;

        registry
    }

    pub async fn register(&self, name: &str, url: &str) -> McpResult<()> {
        info!(service = name, url = url, "Registering MCP service");

        let endpoint = ServiceEndpoint::new(name, url);
        let client = Arc::new(McpClient::new(url));

        {
            let mut services = self.services.write().await;
            services.insert(name.to_string(), endpoint);
        }
        {
            let mut clients = self.clients.write().await;
            clients.insert(name.to_string(), client);
        }

        Ok(())
    }

    pub async fn register_with_config(&self, config: McpServerConfig) -> McpResult<()> {
        info!(
            service = %config.name,
            url = %config.url,
            remote = config.url.starts_with("https://") || !config.url.contains("localhost"),
            has_auth = config.auth.has_auth(),
            "Registering MCP service with config"
        );

        let endpoint = ServiceEndpoint::from_server_config(&config);
        let client = Arc::new(McpClient::from_server_config(&config, self.client_info.clone()));

        {
            let mut services = self.services.write().await;
            services.insert(config.name.clone(), endpoint);
        }
        {
            let mut clients = self.clients.write().await;
            clients.insert(config.name, client);
        }

        Ok(())
    }

    pub async fn register_with_auth(
        &self,
        name: &str,
        url: &str,
        auth: AuthConfig,
    ) -> McpResult<()> {
        let config = McpServerConfig {
            name: name.to_string(),
            url: url.to_string(),
            auth,
            enabled: true,
            description: None,
            tags: Vec::new(),
            timeout_secs: 30,
            cache_ttl_secs: 300,
            health_check_interval_secs: None,
        };
        self.register_with_config(config).await
    }

    pub async fn unregister(&self, name: &str) {
        info!(service = name, "Unregistering MCP service");

        let tools_to_remove: Vec<String> = {
            let tool_map = self.tool_service_map.read().await;
            tool_map
                .iter()
                .filter(|(_, svc)| *svc == name)
                .map(|(tool, _)| tool.clone())
                .collect()
        };

        {
            let mut tool_map = self.tool_service_map.write().await;
            for tool in tools_to_remove {
                tool_map.remove(&tool);
            }
        }

        {
            let mut services = self.services.write().await;
            services.remove(name);
        }
        {
            let mut clients = self.clients.write().await;
            clients.remove(name);
        }
    }

    pub async fn discover_all_tools(&self) -> McpResult<Vec<McpTool>> {
        info!("Discovering tools from all registered services");

        let service_names: Vec<String> = {
            let services = self.services.read().await;
            services
                .iter()
                .filter(|(_, s)| s.enabled)
                .map(|(name, _)| name.clone())
                .collect()
        };

        let mut all_tools = Vec::new();
        let mut tool_map = HashMap::new();

        for name in service_names {
            match self.refresh(&name).await {
                Ok(tools) => {
                    for tool in &tools {
                        tool_map.insert(tool.name.clone(), name.clone());
                    }
                    all_tools.extend(tools);
                }
                Err(e) => {
                    warn!(service = %name, error = %e, "Failed to discover tools from service");
                }
            }
        }

        {
            let mut map = self.tool_service_map.write().await;
            *map = tool_map;
        }

        info!(total_tools = all_tools.len(), "Tool discovery complete");
        Ok(all_tools)
    }

    pub async fn refresh(&self, service_name: &str) -> McpResult<Vec<McpTool>> {
        debug!(service = service_name, "Refreshing tools for service");

        let client = {
            let clients = self.clients.read().await;
            clients
                .get(service_name)
                .cloned()
                .ok_or_else(|| McpError::ServiceNotFound(service_name.to_string()))?
        };

        let tools = client.list_tools_fresh().await?;
        let capabilities = client.get_server_capabilities().await;
        let server_info = client.get_server_info().await;

        {
            let mut services = self.services.write().await;
            if let Some(endpoint) = services.get_mut(service_name) {
                endpoint.tools = tools.clone();
                endpoint.capabilities = capabilities;
                endpoint.server_info = server_info;
                endpoint.last_refreshed = Some(Instant::now());
            }
        }

        {
            let mut tool_map = self.tool_service_map.write().await;
            for tool in &tools {
                tool_map.insert(tool.name.clone(), service_name.to_string());
            }
        }

        info!(
            service = service_name,
            tool_count = tools.len(),
            "Service tools refreshed"
        );

        Ok(tools)
    }

    pub async fn get_all_tools(&self) -> Vec<McpTool> {
        let services = self.services.read().await;
        services
            .values()
            .filter(|s| s.enabled)
            .flat_map(|s| s.tools.clone())
            .collect()
    }

    pub async fn get_read_only_tools(&self) -> Vec<McpTool> {
        filter_read_only(self.get_all_tools().await)
    }

    pub async fn get_tools_filtered<F: ToolFilter>(&self, filter: &F) -> Vec<McpTool> {
        self.get_all_tools()
            .await
            .into_iter()
            .filter(|t| filter.filter(t))
            .collect()
    }

    pub async fn get_tools_for_service(&self, service_name: &str) -> Option<Vec<McpTool>> {
        let services = self.services.read().await;
        services.get(service_name).map(|s| s.tools.clone())
    }

    pub async fn call_tool(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> McpResult<CallToolResult> {
        let service_name = {
            let tool_map = self.tool_service_map.read().await;
            tool_map
                .get(name)
                .cloned()
                .ok_or_else(|| McpError::ToolNotFound(name.to_string()))?
        };

        let client = {
            let clients = self.clients.read().await;
            clients
                .get(&service_name)
                .cloned()
                .ok_or_else(|| McpError::ServiceNotFound(service_name.clone()))?
        };

        debug!(
            tool = name,
            service = %service_name,
            "Routing tool call to service"
        );

        client.call_tool_json(name, arguments).await
    }

    pub async fn get_service_for_tool(&self, tool_name: &str) -> Option<String> {
        let tool_map = self.tool_service_map.read().await;
        tool_map.get(tool_name).cloned()
    }

    pub async fn list_services(&self) -> Vec<ServiceEndpoint> {
        let services = self.services.read().await;
        services.values().cloned().collect()
    }

    pub async fn get_service(&self, name: &str) -> Option<ServiceEndpoint> {
        let services = self.services.read().await;
        services.get(name).cloned()
    }

    pub async fn enable_service(&self, name: &str) -> bool {
        let mut services = self.services.write().await;
        if let Some(endpoint) = services.get_mut(name) {
            endpoint.enabled = true;
            info!(service = name, "Service enabled");
            true
        } else {
            false
        }
    }

    pub async fn disable_service(&self, name: &str) -> bool {
        let mut services = self.services.write().await;
        if let Some(endpoint) = services.get_mut(name) {
            endpoint.enabled = false;
            info!(service = name, "Service disabled");
            true
        } else {
            false
        }
    }

    pub async fn get_client(&self, service_name: &str) -> Option<Arc<McpClient>> {
        let clients = self.clients.read().await;
        clients.get(service_name).cloned()
    }

    pub async fn stats(&self) -> RegistryStats {
        let services = self.services.read().await;
        let tool_map = self.tool_service_map.read().await;

        let service_count = services.len();
        let enabled_count = services.values().filter(|s| s.enabled).count();
        let remote_count = services.values().filter(|s| s.is_remote).count();
        let auth_count = services.values().filter(|s| s.has_auth).count();
        let total_tools = tool_map.len();
        let read_only_tools = services
            .values()
            .flat_map(|s| filter_read_only(s.tools.clone()))
            .count();

        RegistryStats {
            service_count,
            enabled_service_count: enabled_count,
            remote_service_count: remote_count,
            authenticated_service_count: auth_count,
            total_tools,
            read_only_tools,
        }
    }

    pub async fn list_remote_services(&self) -> Vec<ServiceEndpoint> {
        let services = self.services.read().await;
        services.values().filter(|s| s.is_remote).cloned().collect()
    }

    pub async fn list_local_services(&self) -> Vec<ServiceEndpoint> {
        let services = self.services.read().await;
        services.values().filter(|s| !s.is_remote).cloned().collect()
    }

    pub async fn get_tools_by_service(&self) -> HashMap<String, Vec<McpTool>> {
        let services = self.services.read().await;
        services
            .iter()
            .filter(|(_, s)| s.enabled)
            .map(|(name, s)| (name.clone(), s.tools.clone()))
            .collect()
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct RegistryStats {
    pub service_count: usize,
    pub enabled_service_count: usize,
    pub remote_service_count: usize,
    pub authenticated_service_count: usize,
    pub total_tools: usize,
    pub read_only_tools: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_registry_creation() {
        let registry = ServiceRegistry::new();
        let stats = registry.stats().await;
        assert_eq!(stats.service_count, 0);
    }

    #[tokio::test]
    async fn test_registry_with_endpoints() {
        let registry = ServiceRegistry::with_endpoints(vec![
            ("arbfarm", "http://localhost:9007/mcp/jsonrpc"),
            ("engrams", "http://localhost:9004/mcp/jsonrpc"),
        ]);

        let stats = registry.stats().await;
        assert_eq!(stats.service_count, 2);
        assert_eq!(stats.remote_service_count, 0);
    }

    #[tokio::test]
    async fn test_register_unregister() {
        let registry = ServiceRegistry::new();

        registry
            .register("test", "http://localhost:8000/mcp/jsonrpc")
            .await
            .unwrap();
        assert_eq!(registry.stats().await.service_count, 1);

        registry.unregister("test").await;
        assert_eq!(registry.stats().await.service_count, 0);
    }

    #[tokio::test]
    async fn test_from_server_configs() {
        let configs = vec![
            McpServerConfig::new("local", "http://localhost:9007/mcp/jsonrpc"),
            McpServerConfig::new("remote", "https://api.example.com/mcp/jsonrpc")
                .with_api_key("secret"),
        ];

        let registry = ServiceRegistry::from_server_configs(configs);
        let stats = registry.stats().await;

        assert_eq!(stats.service_count, 2);
        assert_eq!(stats.remote_service_count, 1);
        assert_eq!(stats.authenticated_service_count, 1);
    }

    #[tokio::test]
    async fn test_register_with_auth() {
        let registry = ServiceRegistry::new();

        registry
            .register_with_auth(
                "external",
                "https://api.example.com/mcp/jsonrpc",
                AuthConfig::api_key("test-key"),
            )
            .await
            .unwrap();

        let stats = registry.stats().await;
        assert_eq!(stats.service_count, 1);
        assert_eq!(stats.remote_service_count, 1);
        assert_eq!(stats.authenticated_service_count, 1);
    }

    #[tokio::test]
    async fn test_list_remote_local_services() {
        let configs = vec![
            McpServerConfig::new("local1", "http://localhost:9007/mcp/jsonrpc"),
            McpServerConfig::new("local2", "http://localhost:9008/mcp/jsonrpc"),
            McpServerConfig::new("remote1", "https://api.example.com/mcp/jsonrpc"),
        ];

        let registry = ServiceRegistry::from_server_configs(configs);

        let local = registry.list_local_services().await;
        let remote = registry.list_remote_services().await;

        assert_eq!(local.len(), 2);
        assert_eq!(remote.len(), 1);
        assert_eq!(remote[0].name, "remote1");
    }
}
