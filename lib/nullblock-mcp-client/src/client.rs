use crate::error::{McpError, McpResult};
use crate::types::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

const DEFAULT_CACHE_TTL_SECS: u64 = 300;
const DEFAULT_TIMEOUT_SECS: u64 = 30;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key_header: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bearer_token: Option<String>,
    #[serde(default)]
    pub custom_headers: HashMap<String, String>,
}

impl AuthConfig {
    pub fn none() -> Self {
        Self::default()
    }

    pub fn api_key(key: impl Into<String>) -> Self {
        Self {
            api_key: Some(key.into()),
            api_key_header: Some("X-API-Key".to_string()),
            ..Default::default()
        }
    }

    pub fn api_key_with_header(key: impl Into<String>, header: impl Into<String>) -> Self {
        Self {
            api_key: Some(key.into()),
            api_key_header: Some(header.into()),
            ..Default::default()
        }
    }

    pub fn bearer_token(token: impl Into<String>) -> Self {
        Self {
            bearer_token: Some(token.into()),
            ..Default::default()
        }
    }

    pub fn with_custom_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.custom_headers.insert(key.into(), value.into());
        self
    }

    pub fn has_auth(&self) -> bool {
        self.api_key.is_some() || self.bearer_token.is_some()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl_secs: u64,
    #[serde(default)]
    pub health_check_interval_secs: Option<u64>,
}

fn default_true() -> bool {
    true
}

fn default_timeout() -> u64 {
    DEFAULT_TIMEOUT_SECS
}

fn default_cache_ttl() -> u64 {
    DEFAULT_CACHE_TTL_SECS
}

impl McpServerConfig {
    pub fn new(name: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            url: url.into(),
            auth: AuthConfig::default(),
            enabled: true,
            description: None,
            tags: Vec::new(),
            timeout_secs: DEFAULT_TIMEOUT_SECS,
            cache_ttl_secs: DEFAULT_CACHE_TTL_SECS,
            health_check_interval_secs: None,
        }
    }

    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.auth = AuthConfig::api_key(key);
        self
    }

    pub fn with_bearer_token(mut self, token: impl Into<String>) -> Self {
        self.auth = AuthConfig::bearer_token(token);
        self
    }

    pub fn with_auth(mut self, auth: AuthConfig) -> Self {
        self.auth = auth;
        self
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

pub struct McpClient {
    endpoint_url: String,
    name: String,
    http_client: reqwest::Client,
    request_id: AtomicU64,
    initialized: RwLock<bool>,
    server_capabilities: RwLock<Option<ServerCapabilities>>,
    server_info: RwLock<Option<ServerInfo>>,
    protocol_version: RwLock<Option<String>>,
    tools_cache: RwLock<ToolsCache>,
    cache_ttl: Duration,
    client_info: ClientInfo,
    auth: AuthConfig,
    is_remote: bool,
}

struct ToolsCache {
    tools: Vec<McpTool>,
    last_fetched: Instant,
}

impl McpClient {
    pub fn new(endpoint_url: impl Into<String>) -> Self {
        let url: String = endpoint_url.into();
        let is_remote = url.starts_with("https://") || !url.contains("localhost");
        Self::with_full_config(
            url,
            "unnamed".to_string(),
            ClientInfo::default(),
            AuthConfig::default(),
            DEFAULT_CACHE_TTL_SECS,
            DEFAULT_TIMEOUT_SECS,
            is_remote,
        )
    }

    pub fn with_config(
        endpoint_url: impl Into<String>,
        client_info: ClientInfo,
        cache_ttl_secs: u64,
    ) -> Self {
        let url: String = endpoint_url.into();
        let is_remote = url.starts_with("https://") || !url.contains("localhost");
        Self::with_full_config(
            url,
            "unnamed".to_string(),
            client_info,
            AuthConfig::default(),
            cache_ttl_secs,
            DEFAULT_TIMEOUT_SECS,
            is_remote,
        )
    }

    pub fn from_server_config(config: &McpServerConfig, client_info: ClientInfo) -> Self {
        let is_remote = config.url.starts_with("https://") || !config.url.contains("localhost");
        Self::with_full_config(
            config.url.clone(),
            config.name.clone(),
            client_info,
            config.auth.clone(),
            config.cache_ttl_secs,
            config.timeout_secs,
            is_remote,
        )
    }

    fn with_full_config(
        endpoint_url: String,
        name: String,
        client_info: ClientInfo,
        auth: AuthConfig,
        cache_ttl_secs: u64,
        timeout_secs: u64,
        is_remote: bool,
    ) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            endpoint_url,
            name,
            http_client,
            request_id: AtomicU64::new(1),
            initialized: RwLock::new(false),
            server_capabilities: RwLock::new(None),
            server_info: RwLock::new(None),
            protocol_version: RwLock::new(None),
            tools_cache: RwLock::new(ToolsCache {
                tools: Vec::new(),
                last_fetched: Instant::now() - Duration::from_secs(cache_ttl_secs + 1),
            }),
            cache_ttl: Duration::from_secs(cache_ttl_secs),
            client_info,
            auth,
            is_remote,
        }
    }

    pub fn with_auth(mut self, auth: AuthConfig) -> Self {
        self.auth = auth;
        self
    }

    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.auth = AuthConfig::api_key(key);
        self
    }

    pub fn with_bearer_token(mut self, token: impl Into<String>) -> Self {
        self.auth = AuthConfig::bearer_token(token);
        self
    }

    pub fn endpoint_url(&self) -> &str {
        &self.endpoint_url
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn is_remote(&self) -> bool {
        self.is_remote
    }

    pub fn has_auth(&self) -> bool {
        self.auth.has_auth()
    }

    fn next_request_id(&self) -> u64 {
        self.request_id.fetch_add(1, Ordering::SeqCst)
    }

    fn build_request(&self, request_body: &serde_json::Value) -> reqwest::RequestBuilder {
        let mut builder = self
            .http_client
            .post(&self.endpoint_url)
            .header("Content-Type", "application/json")
            .json(request_body);

        if let Some(ref token) = self.auth.bearer_token {
            builder = builder.header("Authorization", format!("Bearer {}", token));
        }

        if let Some(ref api_key) = self.auth.api_key {
            let header_name = self.auth.api_key_header.as_deref().unwrap_or("X-API-Key");
            builder = builder.header(header_name, api_key);
        }

        for (key, value) in &self.auth.custom_headers {
            builder = builder.header(key, value);
        }

        builder
    }

    async fn send_request(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> McpResult<serde_json::Value> {
        let id = self.next_request_id();

        let mut request_body = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method
        });

        if let Some(params) = params {
            request_body["params"] = params;
        }

        debug!(method = method, id = id, endpoint = %self.endpoint_url, "Sending MCP request");

        let response = self.build_request(&request_body).send().await?;

        if !response.status().is_success() {
            return Err(McpError::ProtocolError(format!(
                "MCP {} failed with status: {}",
                method,
                response.status()
            )));
        }

        let data: serde_json::Value = response.json().await?;

        if let Some(error) = data.get("error") {
            let code = error.get("code").and_then(|c| c.as_i64()).unwrap_or(-1);
            let message = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error")
                .to_string();
            return Err(McpError::JsonRpcError { code, message });
        }

        data.get("result")
            .cloned()
            .ok_or_else(|| McpError::InvalidResponse("MCP response missing result".to_string()))
    }

    async fn send_notification(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> McpResult<()> {
        let mut request_body = json!({
            "jsonrpc": "2.0",
            "method": method
        });

        if let Some(params) = params {
            request_body["params"] = params;
        }

        debug!(method = method, "Sending MCP notification");

        let response = self.build_request(&request_body).send().await?;

        if !response.status().is_success() {
            warn!(
                "MCP notification {} returned status: {}",
                method,
                response.status()
            );
        }

        Ok(())
    }

    pub async fn connect(&self) -> McpResult<InitializeResult> {
        info!(
            endpoint = %self.endpoint_url,
            name = %self.name,
            remote = self.is_remote,
            has_auth = self.auth.has_auth(),
            "Initializing MCP connection"
        );

        let init_params = json!({
            "protocolVersion": MCP_PROTOCOL_VERSION,
            "capabilities": {
                "experimental": {},
                "sampling": {}
            },
            "clientInfo": {
                "name": self.client_info.name,
                "version": self.client_info.version
            }
        });

        let result = self.send_request("initialize", Some(init_params)).await?;

        let init_result: InitializeResult = serde_json::from_value(result)?;

        if init_result.protocol_version != MCP_PROTOCOL_VERSION {
            warn!(
                server_version = %init_result.protocol_version,
                client_version = MCP_PROTOCOL_VERSION,
                "Server protocol version differs from client"
            );
        }

        {
            let mut caps = self.server_capabilities.write().await;
            *caps = Some(init_result.capabilities.clone());
        }
        {
            let mut info = self.server_info.write().await;
            *info = Some(init_result.server_info.clone());
        }
        {
            let mut version = self.protocol_version.write().await;
            *version = Some(init_result.protocol_version.clone());
        }

        self.send_notification("initialized", Some(json!({})))
            .await?;

        {
            let mut initialized = self.initialized.write().await;
            *initialized = true;
        }

        info!(
            server_name = %init_result.server_info.name,
            server_version = %init_result.server_info.version,
            endpoint = %self.endpoint_url,
            "MCP connection established"
        );

        Ok(init_result)
    }

    pub async fn ensure_connected(&self) -> McpResult<()> {
        let initialized = *self.initialized.read().await;
        if !initialized {
            self.connect().await?;
        }
        Ok(())
    }

    pub async fn is_connected(&self) -> bool {
        *self.initialized.read().await
    }

    pub async fn get_server_capabilities(&self) -> Option<ServerCapabilities> {
        self.server_capabilities.read().await.clone()
    }

    pub async fn get_server_info(&self) -> Option<ServerInfo> {
        self.server_info.read().await.clone()
    }

    pub async fn list_tools(&self) -> McpResult<Vec<McpTool>> {
        self.ensure_connected().await?;

        {
            let cache = self.tools_cache.read().await;
            if cache.last_fetched.elapsed() < self.cache_ttl {
                debug!(count = cache.tools.len(), "Returning cached tools");
                return Ok(cache.tools.clone());
            }
        }

        info!(endpoint = %self.endpoint_url, "Fetching MCP tools list");
        let result = self.send_request("tools/list", None).await?;

        let tools_list: ToolsListResult = serde_json::from_value(result)?;

        {
            let mut cache = self.tools_cache.write().await;
            cache.tools = tools_list.tools.clone();
            cache.last_fetched = Instant::now();
        }

        info!(count = tools_list.tools.len(), endpoint = %self.endpoint_url, "Cached MCP tools");
        Ok(tools_list.tools)
    }

    pub async fn list_tools_fresh(&self) -> McpResult<Vec<McpTool>> {
        self.invalidate_tools_cache().await;
        self.list_tools().await
    }

    pub async fn call_tool(
        &self,
        name: &str,
        arguments: HashMap<String, serde_json::Value>,
    ) -> McpResult<CallToolResult> {
        self.ensure_connected().await?;

        info!(tool = name, endpoint = %self.endpoint_url, "Calling MCP tool");

        let params = json!({
            "name": name,
            "arguments": arguments
        });

        let result = self.send_request("tools/call", Some(params)).await?;

        let tool_result: CallToolResult = serde_json::from_value(result)?;

        if tool_result.is_error() {
            let error_text = tool_result.text_content();
            error!(tool = name, error = %error_text, "Tool returned error");
        } else {
            debug!(tool = name, "Tool executed successfully");
        }

        Ok(tool_result)
    }

    pub async fn call_tool_json(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> McpResult<CallToolResult> {
        let args: HashMap<String, serde_json::Value> = if arguments.is_object() {
            serde_json::from_value(arguments)?
        } else {
            HashMap::new()
        };
        self.call_tool(name, args).await
    }

    pub async fn ping(&self) -> McpResult<()> {
        self.ensure_connected().await?;
        let _ = self.send_request("ping", None).await?;
        debug!("MCP ping successful");
        Ok(())
    }

    pub async fn health_check(&self) -> McpResult<bool> {
        match self.ping().await {
            Ok(()) => Ok(true),
            Err(e) => {
                warn!(endpoint = %self.endpoint_url, error = %e, "Health check failed");
                Ok(false)
            }
        }
    }

    pub async fn invalidate_tools_cache(&self) {
        let mut cache = self.tools_cache.write().await;
        cache.last_fetched = Instant::now() - self.cache_ttl - Duration::from_secs(1);
        debug!("MCP tools cache invalidated");
    }

    pub async fn disconnect(&self) {
        let mut initialized = self.initialized.write().await;
        *initialized = false;
        debug!(endpoint = %self.endpoint_url, "MCP client disconnected");
    }

    pub async fn get_cached_tools(&self) -> Vec<McpTool> {
        self.tools_cache.read().await.tools.clone()
    }

    pub fn get_tools_for_prompt_sync(&self) -> String {
        match futures::executor::block_on(self.tools_cache.read())
            .tools
            .as_slice()
        {
            [] => "No tools currently available.".to_string(),
            tools => tools
                .iter()
                .map(|t| {
                    format!(
                        "- **{}**: {}",
                        t.name,
                        t.description.as_deref().unwrap_or("No description")
                    )
                })
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }

    pub async fn get_tools_for_prompt(&self) -> String {
        let cache = self.tools_cache.read().await;
        if cache.tools.is_empty() {
            return "No tools currently available.".to_string();
        }

        cache
            .tools
            .iter()
            .map(|t| {
                format!(
                    "- **{}**: {}",
                    t.name,
                    t.description.as_deref().unwrap_or("No description")
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Clone for McpClient {
    fn clone(&self) -> Self {
        Self::with_full_config(
            self.endpoint_url.clone(),
            self.name.clone(),
            self.client_info.clone(),
            self.auth.clone(),
            self.cache_ttl.as_secs(),
            DEFAULT_TIMEOUT_SECS,
            self.is_remote,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = McpClient::new("http://localhost:9007/mcp/jsonrpc");
        assert_eq!(client.endpoint_url(), "http://localhost:9007/mcp/jsonrpc");
        assert!(!client.is_remote());
    }

    #[test]
    fn test_remote_detection() {
        let local = McpClient::new("http://localhost:9007/mcp/jsonrpc");
        assert!(!local.is_remote());

        let remote = McpClient::new("https://api.example.com/mcp/jsonrpc");
        assert!(remote.is_remote());

        let also_remote = McpClient::new("http://api.example.com/mcp/jsonrpc");
        assert!(also_remote.is_remote());
    }

    #[test]
    fn test_auth_config() {
        let api_key_auth = AuthConfig::api_key("my-secret-key");
        assert!(api_key_auth.has_auth());
        assert_eq!(api_key_auth.api_key.as_deref(), Some("my-secret-key"));

        let bearer_auth = AuthConfig::bearer_token("my-token");
        assert!(bearer_auth.has_auth());
        assert_eq!(bearer_auth.bearer_token.as_deref(), Some("my-token"));

        let no_auth = AuthConfig::none();
        assert!(!no_auth.has_auth());
    }

    #[test]
    fn test_client_with_auth() {
        let client = McpClient::new("https://api.example.com/mcp").with_api_key("test-key");
        assert!(client.has_auth());
    }

    #[test]
    fn test_server_config() {
        let config = McpServerConfig::new("external-mcp", "https://api.example.com/mcp/jsonrpc")
            .with_api_key("secret")
            .with_description("External MCP server")
            .with_tags(vec!["production".to_string(), "external".to_string()]);

        assert!(config.auth.has_auth());
        assert_eq!(config.description.as_deref(), Some("External MCP server"));
        assert_eq!(config.tags.len(), 2);
    }

    #[test]
    fn test_request_id_increment() {
        let client = McpClient::new("http://localhost:9007/mcp/jsonrpc");
        let id1 = client.next_request_id();
        let id2 = client.next_request_id();
        assert_eq!(id2, id1 + 1);
    }
}
