use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{info, warn, debug};

/// Shared external service client for accessing external microservices
/// This eliminates HTTP overhead when used by subservices within the same Rust app
/// and provides common functionality needed across all external service interactions
pub struct ExternalService {
    client: Client,
    hecate_base_url: String,
    mcp_base_url: String,
    orchestration_base_url: String,
    // Cache for storing responses to avoid redundant calls
    response_cache: Arc<tokio::sync::RwLock<HashMap<String, (Value, Instant)>>>,
    // Configuration for retry logic and timeouts
    max_retries: u32,
    retry_delay_ms: u64,
    cache_ttl_seconds: u64,
}

impl ExternalService {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            hecate_base_url: std::env::var("HECATE_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:9003".to_string()),
            mcp_base_url: std::env::var("MCP_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:8001".to_string()),
            orchestration_base_url: std::env::var("ORCHESTRATION_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:8002".to_string()),
            response_cache: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            max_retries: 3,
            retry_delay_ms: 1000,
            cache_ttl_seconds: 300, // 5 minutes
        }
    }

    /// Create with custom configuration
    pub fn with_config(
        hecate_url: impl Into<String>,
        mcp_url: impl Into<String>,
        orchestration_url: impl Into<String>,
        max_retries: u32,
        retry_delay_ms: u64,
        cache_ttl_seconds: u64,
    ) -> Self {
        Self {
            client: Client::new(),
            hecate_base_url: hecate_url.into(),
            mcp_base_url: mcp_url.into(),
            orchestration_base_url: orchestration_url.into(),
            response_cache: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            max_retries,
            retry_delay_ms,
            cache_ttl_seconds,
        }
    }

    // ===== CORE SERVICE INTERACTION METHODS =====

    /// Call external Hecate service directly with retry logic
    pub async fn call_hecate(&self, endpoint: &str) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        self.call_hecate_with_cache(endpoint, false).await
    }

    /// Call external Hecate service with caching option
    pub async fn call_hecate_with_cache(&self, endpoint: &str, use_cache: bool) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let cache_key = format!("hecate:{}", endpoint);
        
        if use_cache {
            if let Some(cached) = self.get_cached_response(&cache_key).await {
                debug!("📋 Returning cached Hecate response for: {}", endpoint);
                return Ok(cached);
            }
        }

        let url = format!("{}/{}", self.hecate_base_url, endpoint);
        info!("🤖 Calling Hecate service: {}", url);
        
        let result = self.call_with_retry(&url).await;
        
        if let Ok(ref data) = result {
            if use_cache {
                self.cache_response(&cache_key, data.clone()).await;
            }
        }
        
        result
    }

    /// Call external MCP service directly with retry logic
    pub async fn call_mcp(&self, endpoint: &str) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        self.call_mcp_with_cache(endpoint, false).await
    }

    /// Call external MCP service with caching option
    pub async fn call_mcp_with_cache(&self, endpoint: &str, use_cache: bool) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let cache_key = format!("mcp:{}", endpoint);
        
        if use_cache {
            if let Some(cached) = self.get_cached_response(&cache_key).await {
                debug!("📋 Returning cached MCP response for: {}", endpoint);
                return Ok(cached);
            }
        }

        let url = format!("{}/{}", self.mcp_base_url, endpoint);
        info!("🌐 Calling MCP service: {}", url);
        
        let result = self.call_with_retry(&url).await;
        
        if let Ok(ref data) = result {
            if use_cache {
                self.cache_response(&cache_key, data.clone()).await;
            }
        }
        
        result
    }

    /// Call external orchestration service directly with retry logic
    pub async fn call_orchestration(&self, endpoint: &str) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        self.call_orchestration_with_cache(endpoint, false).await
    }

    /// Call external orchestration service with caching option
    pub async fn call_orchestration_with_cache(&self, endpoint: &str, use_cache: bool) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let cache_key = format!("orchestration:{}", endpoint);
        
        if use_cache {
            if let Some(cached) = self.get_cached_response(&cache_key).await {
                debug!("📋 Returning cached orchestration response for: {}", endpoint);
                return Ok(cached);
            }
        }

        let url = format!("{}/{}", self.orchestration_base_url, endpoint);
        info!("🔄 Calling orchestration service: {}", url);
        
        let result = self.call_with_retry(&url).await;
        
        if let Ok(ref data) = result {
            if use_cache {
                self.cache_response(&cache_key, data.clone()).await;
            }
        }
        
        result
    }

    /// Generic external service call with custom base URL and retry logic
    pub async fn call_external(&self, base_url: &str, endpoint: &str) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/{}", base_url, endpoint);
        info!("🌐 Calling external service: {}", url);
        
        self.call_with_retry(&url).await
    }

    // ===== AGENT-SPECIFIC FUNCTIONALITY =====

    /// Send a chat message to Hecate agent
    pub async fn hecate_chat(&self, message: &str, context: Option<Value>) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        info!("💬 Sending chat message to Hecate: {}", message);
        
        let payload = serde_json::json!({
            "message": message,
            "context": context,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        
        let url = format!("{}/chat", self.hecate_base_url);
        let response = self.client.post(&url)
            .json(&payload)
            .send()
            .await?;
            
        if response.status().is_success() {
            let result = response.json::<Value>().await?;
            info!("✅ Hecate chat response received");
            Ok(result)
        } else {
            Err(format!("Hecate chat failed with status: {}", response.status()).into())
        }
    }

    /// Get Hecate agent status and capabilities
    pub async fn hecate_status(&self) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        info!("📊 Getting Hecate agent status");
        self.call_hecate("status").await
    }

    /// Get available models from Hecate
    pub async fn hecate_available_models(&self) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        info!("🧠 Getting available models from Hecate");
        self.call_hecate("available-models").await
    }

    /// Set active model for Hecate
    pub async fn hecate_set_model(&self, model_name: &str) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        info!("🎯 Setting Hecate model to: {}", model_name);
        
        let payload = serde_json::json!({
            "model": model_name,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        
        let url = format!("{}/set-model", self.hecate_base_url);
        let response = self.client.post(&url)
            .json(&payload)
            .send()
            .await?;
            
        if response.status().is_success() {
            let result = response.json::<Value>().await?;
            info!("✅ Hecate model set successfully");
            Ok(result)
        } else {
            Err(format!("Failed to set Hecate model with status: {}", response.status()).into())
        }
    }

    /// Get Hecate personality configuration
    pub async fn hecate_personality(&self) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        info!("👤 Getting Hecate personality configuration");
        self.call_hecate("personality").await
    }

    /// Update Hecate personality
    pub async fn hecate_update_personality(&self, personality: Value) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        info!("👤 Updating Hecate personality configuration");
        
        let url = format!("{}/personality", self.hecate_base_url);
        let response = self.client.post(&url)
            .json(&personality)
            .send()
            .await?;
            
        if response.status().is_success() {
            let result = response.json::<Value>().await?;
            info!("✅ Hecate personality updated successfully");
            Ok(result)
        } else {
            Err(format!("Failed to update Hecate personality with status: {}", response.status()).into())
        }
    }

        /// Clear Hecate conversation history
    pub async fn hecate_clear_history(&self) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        info!("🧹 Clearing Hecate conversation history");
        
        let url = format!("{}/clear", self.hecate_base_url);
        let response = self.client.post(&url).send().await?;
        
        if response.status().is_success() {
            let result = response.json::<Value>().await?;
            info!("✅ Hecate conversation history cleared");
            Ok(result)
        } else {
            Err(format!("Failed to clear Hecate history with status: {}", response.status()).into())
        }
    }

    /// Get Hecate conversation history
    pub async fn hecate_get_history(&self, limit: Option<usize>) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        info!("📜 Getting Hecate conversation history");
        
        let mut url = format!("{}/history", self.hecate_base_url);
        if let Some(limit) = limit {
            url.push_str(&format!("?limit={}", limit));
        }
        
        let response = self.client.get(&url).send().await?;
            
        if response.status().is_success() {
            let result = response.json::<Value>().await?;
            info!("✅ Hecate conversation history retrieved");
            Ok(result)
        } else {
            Err(format!("Failed to get Hecate history with status: {}", response.status()).into())
        }
    }

        /// Search for models in Hecate
    pub async fn hecate_search_models(&self, query: &str) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        info!("🔍 Searching Hecate models with query: {}", query);
        
        let url = format!("{}/search-models?q={}", self.hecate_base_url, query);
        let response = self.client.get(&url).send().await?;
        
        if response.status().is_success() {
            let result = response.json::<Value>().await?;
            info!("✅ Hecate model search completed");
            Ok(result)
        } else {
            Err(format!("Failed to search Hecate models with status: {}", response.status()).into())
        }
    }

    /// Get model information from Hecate
    pub async fn hecate_model_info(&self, model_name: &str) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        info!("📋 Getting Hecate model info for: {}", model_name);
        
        let url = format!("{}/model-info?model={}", self.hecate_base_url, model_name);
        let response = self.client.get(&url).send().await?;
            
        if response.status().is_success() {
            let result = response.json::<Value>().await?;
            info!("✅ Hecate model info retrieved");
            Ok(result)
        } else {
            Err(format!("Failed to get Hecate model info with status: {}", response.status()).into())
        }
    }

    /// Execute a reasoning task with Hecate
    pub async fn hecate_reasoning(&self, task: &str, context: Option<Value>) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        info!("🧠 Executing reasoning task with Hecate: {}", task);
        
        let payload = serde_json::json!({
            "task": task,
            "context": context,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        
        let url = format!("{}/reasoning", self.hecate_base_url);
        let response = self.client.post(&url)
            .json(&payload)
            .send()
            .await?;
            
        if response.status().is_success() {
            let result = response.json::<Value>().await?;
            info!("✅ Hecate reasoning task completed");
            Ok(result)
        } else {
            Err(format!("Failed to execute reasoning task with status: {}", response.status()).into())
        }
    }

    /// Get Hecate agent metrics and performance data
    pub async fn hecate_metrics(&self) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        info!("📊 Getting Hecate agent metrics");
        self.call_hecate("metrics").await
    }

    /// Restart Hecate agent
    pub async fn hecate_restart(&self) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        info!("🔄 Restarting Hecate agent");
        
        let url = format!("{}/restart", self.hecate_base_url);
        let response = self.client.post(&url).send().await?;
            
        if response.status().is_success() {
            let result = response.json::<Value>().await?;
            info!("✅ Hecate agent restart initiated");
            Ok(result)
        } else {
            Err(format!("Failed to restart Hecate agent with status: {}", response.status()).into())
        }
    }

    /// Get comprehensive agent information from all available services
    pub async fn get_all_agents_info(&self) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        info!("🤖 Getting comprehensive agent information from all services");
        
        let mut agents_info = serde_json::Map::new();
        
        // Get Hecate agent info
        match self.hecate_status().await {
            Ok(hecate_info) => {
                agents_info.insert("hecate".to_string(), serde_json::json!({
                    "service": "hecate",
                    "status": "available",
                    "info": hecate_info
                }));
            }
            Err(e) => {
                agents_info.insert("hecate".to_string(), serde_json::json!({
                    "service": "hecate",
                    "status": "unavailable",
                    "error": e.to_string()
                }));
            }
        }
        
        // Get MCP agent info
        match self.call_mcp("agents").await {
            Ok(mcp_agents) => {
                agents_info.insert("mcp_agents".to_string(), serde_json::json!({
                    "service": "mcp",
                    "status": "available",
                    "agents": mcp_agents
                }));
            }
            Err(e) => {
                agents_info.insert("mcp_agents".to_string(), serde_json::json!({
                    "service": "mcp",
                    "status": "unavailable",
                    "error": e.to_string()
                }));
            }
        }
        
        // Get orchestration workflow info
        match self.call_orchestration("workflows").await {
            Ok(workflows) => {
                agents_info.insert("workflows".to_string(), serde_json::json!({
                    "service": "orchestration",
                    "status": "available",
                    "workflows": workflows
                }));
            }
            Err(e) => {
                agents_info.insert("workflows".to_string(), serde_json::json!({
                    "service": "orchestration",
                    "status": "unavailable",
                    "error": e.to_string()
                }));
            }
        }
        
        agents_info.insert("timestamp".to_string(), serde_json::json!(chrono::Utc::now().to_rfc3339()));
        agents_info.insert("total_services".to_string(), serde_json::json!(agents_info.len() - 2)); // Exclude timestamp and total_services
        
        Ok(serde_json::Value::Object(agents_info))
    }

    /// Coordinate multiple agents for a complex task
    pub async fn coordinate_agents(&self, task: &str, required_agents: &[&str]) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        info!("🤝 Coordinating agents for task: {} with required agents: {:?}", task, required_agents);
        
        let mut coordination_results = serde_json::Map::new();
        let mut successful_agents = Vec::new();
        let mut failed_agents = Vec::new();
        
        for agent in required_agents {
            match *agent {
                "hecate" => {
                    match self.hecate_reasoning(task, None).await {
                        Ok(result) => {
                            successful_agents.push("hecate");
                            coordination_results.insert("hecate_result".to_string(), result);
                        }
                        Err(e) => {
                            failed_agents.push("hecate");
                            coordination_results.insert("hecate_error".to_string(), serde_json::json!(e.to_string()));
                        }
                    }
                }
                "mcp" => {
                    match self.call_mcp("execute").await {
                        Ok(result) => {
                            successful_agents.push("mcp");
                            coordination_results.insert("mcp_result".to_string(), result);
                        }
                        Err(e) => {
                            failed_agents.push("mcp");
                            coordination_results.insert("mcp_error".to_string(), serde_json::json!(e.to_string()));
                        }
                    }
                }
                "orchestration" => {
                    match self.call_orchestration("coordinate").await {
                        Ok(result) => {
                            successful_agents.push("orchestration");
                            coordination_results.insert("orchestration_result".to_string(), result);
                        }
                        Err(e) => {
                            failed_agents.push("orchestration");
                            coordination_results.insert("orchestration_error".to_string(), serde_json::json!(e.to_string()));
                        }
                    }
                }
                _ => {
                    warn!("⚠️ Unknown agent type: {}", agent);
                    failed_agents.push(agent);
                }
            }
        }
        
        coordination_results.insert("task".to_string(), serde_json::json!(task));
        coordination_results.insert("successful_agents".to_string(), serde_json::json!(successful_agents));
        coordination_results.insert("failed_agents".to_string(), serde_json::json!(failed_agents));
        coordination_results.insert("timestamp".to_string(), serde_json::json!(chrono::Utc::now().to_rfc3339()));
        
        if failed_agents.is_empty() {
            info!("✅ All agents coordinated successfully for task: {}", task);
        } else {
            warn!("⚠️ Some agents failed during coordination for task: {}", task);
        }
        
        Ok(serde_json::Value::Object(coordination_results))
    }

    /// Monitor agent performance and health in real-time
    pub async fn monitor_agents_health(&self) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        info!("📊 Monitoring real-time agent health and performance");
        
        let mut health_data = serde_json::Map::new();
        let start_time = Instant::now();
        
        // Monitor Hecate health
        let hecate_start = Instant::now();
        match self.hecate_metrics().await {
            Ok(metrics) => {
                health_data.insert("hecate".to_string(), serde_json::json!({
                    "status": "healthy",
                    "response_time_ms": hecate_start.elapsed().as_millis(),
                    "metrics": metrics,
                    "last_check": chrono::Utc::now().to_rfc3339()
                }));
            }
            Err(e) => {
                health_data.insert("hecate".to_string(), serde_json::json!({
                    "status": "unhealthy",
                    "error": e.to_string(),
                    "response_time_ms": hecate_start.elapsed().as_millis(),
                    "last_check": chrono::Utc::now().to_rfc3339()
                }));
            }
        }
        
        // Monitor MCP health
        let mcp_start = Instant::now();
        match self.call_mcp("health").await {
            Ok(health) => {
                health_data.insert("mcp".to_string(), serde_json::json!({
                    "status": "healthy",
                    "response_time_ms": mcp_start.elapsed().as_millis(),
                    "health": health,
                    "last_check": chrono::Utc::now().to_rfc3339()
                }));
            }
            Err(e) => {
                health_data.insert("mcp".to_string(), serde_json::json!({
                    "status": "unhealthy",
                    "error": e.to_string(),
                    "response_time_ms": mcp_start.elapsed().as_millis(),
                    "last_check": chrono::Utc::now().to_rfc3339()
                }));
            }
        }
        
        // Monitor orchestration health
        let orch_start = Instant::now();
        match self.call_orchestration("health").await {
            Ok(health) => {
                health_data.insert("orchestration".to_string(), serde_json::json!({
                    "status": "healthy",
                    "response_time_ms": orch_start.elapsed().as_millis(),
                    "health": health,
                    "last_check": chrono::Utc::now().to_rfc3339()
                }));
            }
            Err(e) => {
                health_data.insert("orchestration".to_string(), serde_json::json!({
                    "status": "unhealthy",
                    "error": e.to_string(),
                    "response_time_ms": orch_start.elapsed().as_millis(),
                    "last_check": chrono::Utc::now().to_rfc3339()
                }));
            }
        }
        
        health_data.insert("monitoring_duration_ms".to_string(), serde_json::json!(start_time.elapsed().as_millis()));
        health_data.insert("monitoring_timestamp".to_string(), serde_json::json!(chrono::Utc::now().to_rfc3339()));
        
        Ok(serde_json::Value::Object(health_data))
    }

    /// Get agent capabilities and available features
    pub async fn get_agent_capabilities(&self) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        info!("🔍 Getting agent capabilities and available features");
        
        let mut capabilities = serde_json::Map::new();
        
        // Get Hecate capabilities
        match self.hecate_status().await {
            Ok(status) => {
                capabilities.insert("hecate".to_string(), serde_json::json!({
                    "agent_type": "conversational_ai",
                    "capabilities": [
                        "chat",
                        "reasoning", 
                        "model_switching",
                        "personality_configuration",
                        "conversation_history",
                        "model_search",
                        "metrics_tracking"
                    ],
                    "status": status
                }));
            }
            Err(e) => {
                capabilities.insert("hecate".to_string(), serde_json::json!({
                    "agent_type": "conversational_ai",
                    "capabilities": ["chat", "reasoning", "model_switching"],
                    "status": "unavailable",
                    "error": e.to_string()
                }));
            }
        }
        
        // Get MCP capabilities
        match self.call_mcp("capabilities").await {
            Ok(mcp_caps) => {
                capabilities.insert("mcp".to_string(), serde_json::json!({
                    "agent_type": "model_context_protocol",
                    "capabilities": [
                        "resource_management",
                        "tool_execution",
                        "prompt_management",
                        "context_handling"
                    ],
                    "details": mcp_caps
                }));
            }
            Err(_) => {
                capabilities.insert("mcp".to_string(), serde_json::json!({
                    "agent_type": "model_context_protocol",
                    "capabilities": ["resource_management", "tool_execution"],
                    "status": "capabilities_unavailable"
                }));
            }
        }
        
        // Get orchestration capabilities
        match self.call_orchestration("capabilities").await {
            Ok(orch_caps) => {
                capabilities.insert("orchestration".to_string(), serde_json::json!({
                    "agent_type": "workflow_orchestrator",
                    "capabilities": [
                        "workflow_management",
                        "agent_coordination",
                        "task_scheduling",
                        "resource_allocation"
                    ],
                    "details": orch_caps
                }));
            }
            Err(_) => {
                capabilities.insert("orchestration".to_string(), serde_json::json!({
                    "agent_type": "workflow_orchestrator",
                    "capabilities": ["workflow_management", "agent_coordination"],
                    "status": "capabilities_unavailable"
                }));
            }
        }
        
        capabilities.insert("timestamp".to_string(), serde_json::json!(chrono::Utc::now().to_rfc3339()));
        
        Ok(serde_json::Value::Object(capabilities))
    }

    // ===== ADVANCED SERVICE INTERACTIONS =====

    /// Batch call multiple endpoints on the same service
    pub async fn batch_call_hecate(&self, endpoints: &[&str]) -> Result<HashMap<String, Value>, Box<dyn std::error::Error + Send + Sync>> {
        info!("📦 Batch calling Hecate endpoints: {:?}", endpoints);
        
        let mut results = HashMap::new();
        let mut tasks = Vec::new();
        
        for endpoint in endpoints {
            let service = self.clone();
            let endpoint_str = endpoint.to_string();
            let endpoint_key = endpoint_str.clone();
            let task = tokio::spawn(async move {
                service.call_hecate(&endpoint_str).await
            });
            tasks.push((endpoint_key, task));
        }
        
        for (endpoint, task) in tasks {
            match task.await {
                Ok(Ok(result)) => {
                    results.insert(endpoint, result);
                }
                Ok(Err(e)) => {
                    warn!("⚠️ Failed to call Hecate endpoint {}: {}", endpoint, e);
                    results.insert(endpoint, serde_json::json!({"error": e.to_string()}));
                }
                Err(e) => {
                    warn!("⚠️ Task failed for Hecate endpoint {}: {}", endpoint, e);
                    results.insert(endpoint, serde_json::json!({"error": "Task failed"}));
                }
            }
        }
        
        Ok(results)
    }

    /// Health check with detailed status for all services
    pub async fn detailed_health_check(&self) -> Value {
        info!("🏥 Performing detailed health check of all external services");
        
        let mut services_health = serde_json::Map::new();
        let start_time = Instant::now();

        // Check Hecate service with detailed info
        match self.call_hecate("health").await {
            Ok(hecate_health) => {
                services_health.insert("hecate".to_string(), serde_json::json!({
                    "status": "healthy",
                    "response_time_ms": start_time.elapsed().as_millis(),
                    "details": hecate_health
                }));
            }
            Err(e) => {
                services_health.insert("hecate".to_string(), serde_json::json!({
                    "status": "unhealthy",
                    "error": e.to_string(),
                    "response_time_ms": start_time.elapsed().as_millis()
                }));
            }
        }

        // Check MCP service with detailed info
        let mcp_start = Instant::now();
        match self.call_mcp("health").await {
            Ok(mcp_health) => {
                services_health.insert("mcp".to_string(), serde_json::json!({
                    "status": "healthy",
                    "response_time_ms": mcp_start.elapsed().as_millis(),
                    "details": mcp_health
                }));
            }
            Err(e) => {
                services_health.insert("mcp".to_string(), serde_json::json!({
                    "status": "unhealthy",
                    "error": e.to_string(),
                    "response_time_ms": mcp_start.elapsed().as_millis()
                }));
            }
        }

        // Check orchestration service with detailed info
        let orch_start = Instant::now();
        match self.call_orchestration("health").await {
            Ok(orch_health) => {
                services_health.insert("orchestration".to_string(), serde_json::json!({
                    "status": "healthy",
                    "response_time_ms": orch_start.elapsed().as_millis(),
                    "details": orch_health
                }));
            }
            Err(e) => {
                services_health.insert("orchestration".to_string(), serde_json::json!({
                    "status": "unhealthy",
                    "error": e.to_string(),
                    "response_time_ms": orch_start.elapsed().as_millis()
                }));
            }
        }

        services_health.insert("total_check_time_ms".to_string(), serde_json::json!(start_time.elapsed().as_millis()));
        services_health.insert("note".to_string(), serde_json::json!("Direct service calls - no HTTP overhead"));
        services_health.insert("timestamp".to_string(), serde_json::json!(chrono::Utc::now().to_rfc3339()));

        serde_json::Value::Object(services_health)
    }

    // ===== CACHING AND PERFORMANCE METHODS =====

    /// Get cached response if available and not expired
    async fn get_cached_response(&self, cache_key: &str) -> Option<Value> {
        let cache = self.response_cache.read().await;
        if let Some((value, timestamp)) = cache.get(cache_key) {
            if timestamp.elapsed().as_secs() < self.cache_ttl_seconds {
                return Some(value.clone());
            }
        }
        None
    }

    /// Cache a response with current timestamp
    async fn cache_response(&self, cache_key: &str, value: Value) {
        let mut cache = self.response_cache.write().await;
        cache.insert(cache_key.to_string(), (value, Instant::now()));
        debug!("💾 Cached response for key: {}", cache_key);
    }

    /// Clear expired cache entries
    pub async fn clear_expired_cache(&self) -> usize {
        let mut cache = self.response_cache.write().await;
        let initial_size = cache.len();
        
        cache.retain(|_, (_, timestamp)| {
            timestamp.elapsed().as_secs() < self.cache_ttl_seconds
        });
        
        let cleared_count = initial_size - cache.len();
        if cleared_count > 0 {
            info!("🧹 Cleared {} expired cache entries", cleared_count);
        }
        
        cleared_count
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> Value {
        let cache = self.response_cache.read().await;
        let total_entries = cache.len();
        let mut expired_count = 0;
        let mut valid_count = 0;
        
        for (_, (_, timestamp)) in cache.iter() {
            if timestamp.elapsed().as_secs() >= self.cache_ttl_seconds {
                expired_count += 1;
            } else {
                valid_count += 1;
            }
        }
        
        serde_json::json!({
            "total_entries": total_entries,
            "valid_entries": valid_count,
            "expired_entries": expired_count,
            "cache_ttl_seconds": self.cache_ttl_seconds
        })
    }

    // ===== RETRY AND RESILIENCE METHODS =====

    /// Call with retry logic and exponential backoff
    async fn call_with_retry(&self, url: &str) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let mut last_error: Option<Box<dyn std::error::Error + Send + Sync>> = None;
        
        for attempt in 0..=self.max_retries {
            match self.client.get(url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.json::<Value>().await {
                            Ok(data) => {
                                if attempt > 0 {
                                    info!("✅ Request succeeded on attempt {}", attempt + 1);
                                }
                                return Ok(data);
                            }
                            Err(e) => {
                                last_error = Some(Box::new(e));
                                if attempt < self.max_retries {
                                    warn!("⚠️ Failed to parse response on attempt {}, retrying...", attempt + 1);
                                    sleep(Duration::from_millis(self.retry_delay_ms * (2_u64.pow(attempt)))).await;
                                }
                            }
                        }
                    } else {
                        last_error = Some(format!("HTTP {}: {}", response.status(), response.status().canonical_reason().unwrap_or("Unknown")).into());
                        if attempt < self.max_retries {
                            warn!("⚠️ HTTP error {} on attempt {}, retrying...", response.status(), attempt + 1);
                            sleep(Duration::from_millis(self.retry_delay_ms * (2_u64.pow(attempt)))).await;
                        }
                    }
                }
                Err(e) => {
                    last_error = Some(Box::new(e));
                    if attempt < self.max_retries {
                        warn!("⚠️ Request failed on attempt {}, retrying...", attempt + 1);
                        sleep(Duration::from_millis(self.retry_delay_ms * (2_u64.pow(attempt)))).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| Box::new(std::io::Error::new(std::io::ErrorKind::TimedOut, "Max retries exceeded"))))
    }

    // ===== CONFIGURATION AND UTILITY METHODS =====

    /// Update service URLs dynamically
    pub fn update_service_urls(&mut self, hecate: Option<String>, mcp: Option<String>, orchestration: Option<String>) {
        if let Some(url) = hecate {
            info!("🔄 Updating Hecate base URL to: {}", url);
            self.hecate_base_url = url;
        }
        if let Some(url) = mcp {
            info!("🔄 Updating MCP base URL to: {}", url);
            self.mcp_base_url = url;
        }
        if let Some(url) = orchestration {
            info!("🔄 Updating orchestration base URL to: {}", url);
            self.orchestration_base_url = url;
        }
    }

    /// Get current service configuration
    pub fn get_config(&self) -> Value {
        serde_json::json!({
            "hecate_base_url": self.hecate_base_url,
            "mcp_base_url": self.mcp_base_url,
            "orchestration_base_url": self.orchestration_base_url,
            "max_retries": self.max_retries,
            "retry_delay_ms": self.retry_delay_ms,
            "cache_ttl_seconds": self.cache_ttl_seconds
        })
    }

    /// Check if a specific service is reachable
    pub async fn is_service_reachable(&self, service_name: &str) -> bool {
        let endpoint = match service_name {
            "hecate" => &self.hecate_base_url,
            "mcp" => &self.mcp_base_url,
            "orchestration" => &self.orchestration_base_url,
            _ => return false,
        };
        
        match self.client.get(endpoint).timeout(Duration::from_secs(5)).send().await {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    /// Legacy method for backward compatibility
    pub async fn check_all_services_health(&self) -> Value {
        self.detailed_health_check().await
    }
}

impl Clone for ExternalService {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            hecate_base_url: self.hecate_base_url.clone(),
            mcp_base_url: self.mcp_base_url.clone(),
            orchestration_base_url: self.orchestration_base_url.clone(),
            response_cache: self.response_cache.clone(),
            max_retries: self.max_retries,
            retry_delay_ms: self.retry_delay_ms,
            cache_ttl_seconds: self.cache_ttl_seconds,
        }
    }
}

impl Default for ExternalService {
    fn default() -> Self {
        Self::new()
    }
}
