use crate::error::{AppError, AppResult};
use crate::mcp::types::*;
use serde_json::json;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

const MCP_CACHE_TTL_SECS: u64 = 300;

pub struct McpClient {
    erebus_url: String,
    http_client: reqwest::Client,
    request_id: AtomicU64,
    initialized: RwLock<bool>,
    server_capabilities: RwLock<Option<ServerCapabilities>>,
    server_info: RwLock<Option<ServerInfo>>,
    protocol_version: RwLock<Option<String>>,
    tools_cache: RwLock<ToolsCache>,
    resources_cache: RwLock<ResourcesCache>,
    prompts_cache: RwLock<PromptsCache>,
}

struct ToolsCache {
    tools: Vec<McpTool>,
    last_fetched: Instant,
}

struct ResourcesCache {
    resources: Vec<McpResource>,
    last_fetched: Instant,
}

struct PromptsCache {
    prompts: Vec<McpPrompt>,
    last_fetched: Instant,
}

impl McpClient {
    pub fn new(erebus_url: &str) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            erebus_url: erebus_url.to_string(),
            http_client,
            request_id: AtomicU64::new(1),
            initialized: RwLock::new(false),
            server_capabilities: RwLock::new(None),
            server_info: RwLock::new(None),
            protocol_version: RwLock::new(None),
            tools_cache: RwLock::new(ToolsCache {
                tools: Vec::new(),
                last_fetched: Instant::now() - Duration::from_secs(MCP_CACHE_TTL_SECS + 1),
            }),
            resources_cache: RwLock::new(ResourcesCache {
                resources: Vec::new(),
                last_fetched: Instant::now() - Duration::from_secs(MCP_CACHE_TTL_SECS + 1),
            }),
            prompts_cache: RwLock::new(PromptsCache {
                prompts: Vec::new(),
                last_fetched: Instant::now() - Duration::from_secs(MCP_CACHE_TTL_SECS + 1),
            }),
        }
    }

    fn next_request_id(&self) -> u64 {
        self.request_id.fetch_add(1, Ordering::SeqCst)
    }

    async fn send_request(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> AppResult<serde_json::Value> {
        let id = self.next_request_id();
        let mcp_url = format!("{}/mcp/jsonrpc", self.erebus_url);

        let mut request_body = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method
        });

        if let Some(params) = params {
            request_body["params"] = params;
        }

        let response = self
            .http_client
            .post(&mcp_url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AppError::InternalError(format!("MCP request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::InternalError(format!(
                "MCP {} failed with status: {}",
                method,
                response.status()
            )));
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to parse MCP response: {}", e)))?;

        if let Some(error) = data.get("error") {
            let code = error.get("code").and_then(|c| c.as_i64()).unwrap_or(-1);
            let message = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            return Err(AppError::InternalError(format!(
                "MCP error {}: {}",
                code, message
            )));
        }

        data.get("result")
            .cloned()
            .ok_or_else(|| AppError::InternalError("MCP response missing result".to_string()))
    }

    async fn send_notification(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> AppResult<()> {
        let mcp_url = format!("{}/mcp/jsonrpc", self.erebus_url);

        let mut request_body = json!({
            "jsonrpc": "2.0",
            "method": method
        });

        if let Some(params) = params {
            request_body["params"] = params;
        }

        let response = self
            .http_client
            .post(&mcp_url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AppError::InternalError(format!("MCP notification failed: {}", e)))?;

        if !response.status().is_success() {
            warn!(
                "MCP notification {} returned status: {}",
                method,
                response.status()
            );
        }

        Ok(())
    }

    pub async fn connect(&self) -> AppResult<()> {
        info!("🔌 Initializing MCP connection to {}...", self.erebus_url);

        let init_params = json!({
            "protocolVersion": MCP_PROTOCOL_VERSION,
            "capabilities": {
                "experimental": {},
                "sampling": {}
            },
            "clientInfo": {
                "name": MCP_CLIENT_NAME,
                "version": MCP_CLIENT_VERSION
            }
        });

        let result = self.send_request("initialize", Some(init_params)).await?;

        let init_result: InitializeResult = serde_json::from_value(result).map_err(|e| {
            AppError::InternalError(format!("Failed to parse initialize result: {}", e))
        })?;

        if init_result.protocol_version != MCP_PROTOCOL_VERSION {
            warn!(
                "⚠️ Server protocol version {} differs from client {}",
                init_result.protocol_version, MCP_PROTOCOL_VERSION
            );
        }

        {
            let mut caps = self.server_capabilities.write().await;
            *caps = Some(init_result.capabilities);
        }
        {
            let mut info = self.server_info.write().await;
            *info = Some(init_result.server_info.clone());
        }
        {
            let mut version = self.protocol_version.write().await;
            *version = Some(init_result.protocol_version);
        }

        self.send_notification("initialized", Some(json!({})))
            .await?;

        {
            let mut initialized = self.initialized.write().await;
            *initialized = true;
        }

        info!(
            "✅ MCP connection established with {} v{}",
            init_result.server_info.name, init_result.server_info.version
        );

        Ok(())
    }

    pub async fn ensure_connected(&self) -> AppResult<()> {
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

    pub async fn list_tools(&self) -> AppResult<Vec<McpTool>> {
        self.ensure_connected().await?;

        {
            let cache = self.tools_cache.read().await;
            if cache.last_fetched.elapsed() < Duration::from_secs(MCP_CACHE_TTL_SECS) {
                return Ok(cache.tools.clone());
            }
        }

        info!("🔄 Fetching MCP tools list...");
        let result = self.send_request("tools/list", None).await?;

        let tools_array = result
            .get("tools")
            .and_then(|t| t.as_array())
            .ok_or_else(|| AppError::InternalError("Invalid tools/list response".to_string()))?;

        let tools: Vec<McpTool> = tools_array
            .iter()
            .filter_map(|t| serde_json::from_value(t.clone()).ok())
            .collect();

        {
            let mut cache = self.tools_cache.write().await;
            cache.tools = tools.clone();
            cache.last_fetched = Instant::now();
        }

        info!("✅ Cached {} MCP tools", tools.len());
        Ok(tools)
    }

    pub async fn call_tool(
        &self,
        name: &str,
        arguments: HashMap<String, serde_json::Value>,
    ) -> AppResult<CallToolResult> {
        self.ensure_connected().await?;

        info!("🔧 Calling MCP tool: {}", name);

        let params = json!({
            "name": name,
            "arguments": arguments
        });

        let result = self.send_request("tools/call", Some(params)).await?;

        let tool_result: CallToolResult = serde_json::from_value(result).map_err(|e| {
            AppError::InternalError(format!("Failed to parse tools/call result: {}", e))
        })?;

        if tool_result.is_error == Some(true) {
            let error_text = tool_result
                .content
                .iter()
                .filter_map(|block| {
                    if let ContentBlock::Text { text } = block {
                        Some(text.as_str())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            error!("❌ Tool {} returned error: {}", name, error_text);
        } else {
            info!("✅ Tool {} executed successfully", name);
        }

        Ok(tool_result)
    }

    pub async fn list_resources(&self) -> AppResult<Vec<McpResource>> {
        self.ensure_connected().await?;

        {
            let cache = self.resources_cache.read().await;
            if cache.last_fetched.elapsed() < Duration::from_secs(MCP_CACHE_TTL_SECS) {
                return Ok(cache.resources.clone());
            }
        }

        info!("🔄 Fetching MCP resources list...");
        let result = self.send_request("resources/list", None).await?;

        let resources_array = result
            .get("resources")
            .and_then(|r| r.as_array())
            .ok_or_else(|| {
                AppError::InternalError("Invalid resources/list response".to_string())
            })?;

        let resources: Vec<McpResource> = resources_array
            .iter()
            .filter_map(|r| serde_json::from_value(r.clone()).ok())
            .collect();

        {
            let mut cache = self.resources_cache.write().await;
            cache.resources = resources.clone();
            cache.last_fetched = Instant::now();
        }

        info!("✅ Cached {} MCP resources", resources.len());
        Ok(resources)
    }

    pub async fn read_resource(&self, uri: &str) -> AppResult<Vec<ResourceContents>> {
        self.ensure_connected().await?;

        info!("📖 Reading MCP resource: {}", uri);

        let params = json!({
            "uri": uri
        });

        let result = self.send_request("resources/read", Some(params)).await?;

        let contents_array = result
            .get("contents")
            .and_then(|c| c.as_array())
            .ok_or_else(|| {
                AppError::InternalError("Invalid resources/read response".to_string())
            })?;

        let contents: Vec<ResourceContents> = contents_array
            .iter()
            .filter_map(|c| serde_json::from_value(c.clone()).ok())
            .collect();

        info!("✅ Read {} content blocks from {}", contents.len(), uri);
        Ok(contents)
    }

    pub async fn list_prompts(&self) -> AppResult<Vec<McpPrompt>> {
        self.ensure_connected().await?;

        {
            let cache = self.prompts_cache.read().await;
            if cache.last_fetched.elapsed() < Duration::from_secs(MCP_CACHE_TTL_SECS) {
                return Ok(cache.prompts.clone());
            }
        }

        info!("🔄 Fetching MCP prompts list...");
        let result = self.send_request("prompts/list", None).await?;

        let prompts_array = result
            .get("prompts")
            .and_then(|p| p.as_array())
            .ok_or_else(|| AppError::InternalError("Invalid prompts/list response".to_string()))?;

        let prompts: Vec<McpPrompt> = prompts_array
            .iter()
            .filter_map(|p| serde_json::from_value(p.clone()).ok())
            .collect();

        {
            let mut cache = self.prompts_cache.write().await;
            cache.prompts = prompts.clone();
            cache.last_fetched = Instant::now();
        }

        info!("✅ Cached {} MCP prompts", prompts.len());
        Ok(prompts)
    }

    pub async fn get_prompt(
        &self,
        name: &str,
        arguments: Option<HashMap<String, String>>,
    ) -> AppResult<GetPromptResult> {
        self.ensure_connected().await?;

        info!("💬 Getting MCP prompt: {}", name);

        let mut params = json!({
            "name": name
        });

        if let Some(args) = arguments {
            params["arguments"] = json!(args);
        }

        let result = self.send_request("prompts/get", Some(params)).await?;

        let prompt_result: GetPromptResult = serde_json::from_value(result).map_err(|e| {
            AppError::InternalError(format!("Failed to parse prompts/get result: {}", e))
        })?;

        info!(
            "✅ Retrieved prompt {} with {} messages",
            name,
            prompt_result.messages.len()
        );
        Ok(prompt_result)
    }

    pub async fn ping(&self) -> AppResult<()> {
        self.ensure_connected().await?;

        let _ = self.send_request("ping", None).await?;
        info!("🏓 MCP ping successful");
        Ok(())
    }

    pub fn get_tools_for_prompt(&self) -> String {
        let tools_guard = futures::executor::block_on(self.tools_cache.read());
        if tools_guard.tools.is_empty() {
            return "No tools currently available.".to_string();
        }

        let mut result = String::new();
        for tool in &tools_guard.tools {
            result.push_str(&format!("- **{}**", tool.name));
            if let Some(desc) = &tool.description {
                result.push_str(&format!(": {}", desc));
            }
            result.push('\n');
        }
        result
    }

    pub async fn get_tools_for_prompt_async(&self) -> String {
        let cache = self.tools_cache.read().await;
        if cache.tools.is_empty() {
            return "No tools currently available.".to_string();
        }

        let mut result = String::new();
        for tool in &cache.tools {
            result.push_str(&format!("- **{}**", tool.name));
            if let Some(desc) = &tool.description {
                result.push_str(&format!(": {}", desc));
            }
            result.push('\n');
        }
        result
    }

    pub async fn to_json(&self) -> serde_json::Value {
        let tools = self.tools_cache.read().await;
        let resources = self.resources_cache.read().await;
        let prompts = self.prompts_cache.read().await;
        let capabilities = self.server_capabilities.read().await;
        let server_info = self.server_info.read().await;
        let initialized = *self.initialized.read().await;

        json!({
            "initialized": initialized,
            "erebus_url": self.erebus_url,
            "protocol_version": MCP_PROTOCOL_VERSION,
            "server_info": *server_info,
            "capabilities": *capabilities,
            "tools": {
                "count": tools.tools.len(),
                "cache_age_seconds": tools.last_fetched.elapsed().as_secs(),
                "items": tools.tools
            },
            "resources": {
                "count": resources.resources.len(),
                "cache_age_seconds": resources.last_fetched.elapsed().as_secs(),
                "items": resources.resources
            },
            "prompts": {
                "count": prompts.prompts.len(),
                "cache_age_seconds": prompts.last_fetched.elapsed().as_secs(),
                "items": prompts.prompts
            },
            "ttl_seconds": MCP_CACHE_TTL_SECS
        })
    }

    pub async fn refresh_all_caches(&self) -> AppResult<()> {
        self.ensure_connected().await?;

        info!("🔄 Refreshing all MCP caches...");

        let _ = self.list_tools().await?;
        let _ = self.list_resources().await?;
        let _ = self.list_prompts().await?;

        info!("✅ All MCP caches refreshed");
        Ok(())
    }

    pub async fn invalidate_caches(&self) {
        {
            let mut cache = self.tools_cache.write().await;
            cache.last_fetched = Instant::now() - Duration::from_secs(MCP_CACHE_TTL_SECS + 1);
        }
        {
            let mut cache = self.resources_cache.write().await;
            cache.last_fetched = Instant::now() - Duration::from_secs(MCP_CACHE_TTL_SECS + 1);
        }
        {
            let mut cache = self.prompts_cache.write().await;
            cache.last_fetched = Instant::now() - Duration::from_secs(MCP_CACHE_TTL_SECS + 1);
        }
        info!("🗑️ MCP caches invalidated");
    }
}
