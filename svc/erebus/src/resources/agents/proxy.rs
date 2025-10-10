// Agent proxy service for routing requests to agent backends
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, error, warn};

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentRequest {
    pub message: String,
    pub user_context: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentResponse {
    pub content: String,
    pub model_used: String,
    pub latency_ms: f64,
    pub confidence_score: f64,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentStatus {
    pub status: String,
    pub current_model: Option<String>,
    pub health: HashMap<String, serde_json::Value>,
    pub stats: HashMap<String, serde_json::Value>,
    pub conversation_length: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentErrorResponse {
    pub error: String,
    pub code: String,
    pub message: String,
    pub agent_available: bool,
}

pub struct AgentProxy {
    agent_base_url: String,
    timeout_seconds: u64,
}

impl AgentProxy {
    pub fn new(agent_base_url: String) -> Self {
        Self {
            agent_base_url,
            timeout_seconds: 300,
        }
    }

    pub fn agent_base_url(&self) -> &str {
        &self.agent_base_url
    }

    /// Proxy chat request to Hecate agent backend
    pub async fn proxy_chat(&self, request: AgentRequest) -> Result<AgentResponse, AgentErrorResponse> {
        let client = reqwest::Client::new();
        let url = format!("{}/hecate/chat", self.agent_base_url);
        
        info!("🤖 Proxying chat request to agent: {}", url);
        
        match client
            .post(&url)
            .json(&request)
            .timeout(std::time::Duration::from_secs(self.timeout_seconds))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<AgentResponse>().await {
                        Ok(agent_response) => {
                            info!("✅ Agent response received: {} chars", agent_response.content.len());
                            Ok(agent_response)
                        }
                        Err(e) => {
                            error!("❌ Failed to parse agent response: {}", e);
                            Err(AgentErrorResponse {
                                error: "parse_error".to_string(),
                                code: "AGENT_PARSE_ERROR".to_string(),
                                message: format!("Failed to parse agent response: {}", e),
                                agent_available: true,
                            })
                        }
                    }
                } else {
                    let status = response.status();
                    warn!("⚠️ Agent returned error status: {}", status);

                    // Try to parse the error response body to get detailed error message
                    match response.json::<serde_json::Value>().await {
                        Ok(error_json) => {
                            // Extract the error message from the JSON response
                            let error_message = error_json.get("message")
                                .and_then(|m| m.as_str())
                                .unwrap_or_else(|| "Agent returned an error");

                            let error_type = error_json.get("error")
                                .and_then(|e| e.as_str())
                                .unwrap_or("agent_error");

                            info!("📤 Agent error message: {}", error_message);

                            Err(AgentErrorResponse {
                                error: error_type.to_string(),
                                code: "AGENT_HTTP_ERROR".to_string(),
                                message: error_message.to_string(),
                                agent_available: true,
                            })
                        }
                        Err(e) => {
                            error!("❌ Failed to parse agent error response: {}", e);
                            Err(AgentErrorResponse {
                                error: "agent_error".to_string(),
                                code: "AGENT_HTTP_ERROR".to_string(),
                                message: format!("Agent returned status: {}", status),
                                agent_available: true,
                            })
                        }
                    }
                }
            }
            Err(e) => {
                error!("❌ Failed to connect to agent: {}", e);

                // Check if this is likely an API key issue vs agent unavailable
                let error_str = e.to_string().to_lowercase();
                let (message, code) = if error_str.contains("connection refused") {
                    // Agent is not running - likely API key configuration issue
                    (
                        "🔑 Hecate agent is not running. This is usually caused by missing or invalid LLM API keys. Please check your OpenRouter API key configuration in .env.dev and restart the service. Visit https://openrouter.ai/ to get a free API key.".to_string(),
                        "AGENT_CONFIG_REQUIRED".to_string()
                    )
                } else if error_str.contains("timeout") {
                    (
                        "⏰ The agent service is taking too long to respond. This may indicate an API key or network issue. Please check your configuration and try again.".to_string(),
                        "AGENT_TIMEOUT".to_string()
                    )
                } else {
                    (
                        format!("🌐 Unable to connect to the agent service: {}. Please check that your API keys are configured in .env.dev and the service is running.", e),
                        "AGENT_UNAVAILABLE".to_string()
                    )
                };

                Err(AgentErrorResponse {
                    error: "connection_error".to_string(),
                    code,
                    message,
                    agent_available: false,
                })
            }
        }
    }

    /// Proxy chat request to Siren agent backend
    pub async fn proxy_siren_chat(&self, request: AgentRequest) -> Result<AgentResponse, AgentErrorResponse> {
        let client = reqwest::Client::new();
        let url = format!("{}/siren/chat", self.agent_base_url);

        info!("🎭 Proxying chat request to Siren agent: {}", url);

        match client
            .post(&url)
            .json(&request)
            .timeout(std::time::Duration::from_secs(self.timeout_seconds))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<AgentResponse>().await {
                        Ok(agent_response) => {
                            info!("✅ Marketing agent response received: {} chars", agent_response.content.len());
                            Ok(agent_response)
                        }
                        Err(e) => {
                            error!("❌ Failed to parse marketing agent response: {}", e);
                            Err(AgentErrorResponse {
                                error: "parse_error".to_string(),
                                code: "AGENT_PARSE_ERROR".to_string(),
                                message: format!("Failed to parse marketing agent response: {}", e),
                                agent_available: true,
                            })
                        }
                    }
                } else {
                    let status = response.status();
                    warn!("⚠️ Marketing agent returned error status: {}", status);

                    // Try to parse the error response body to get detailed error message
                    match response.json::<serde_json::Value>().await {
                        Ok(error_json) => {
                            // Extract the error message from the JSON response
                            let error_message = error_json.get("message")
                                .and_then(|m| m.as_str())
                                .unwrap_or_else(|| "Marketing agent returned an error");

                            let error_type = error_json.get("error")
                                .and_then(|e| e.as_str())
                                .unwrap_or("agent_error");

                            info!("📤 Marketing agent error message: {}", error_message);

                            Err(AgentErrorResponse {
                                error: error_type.to_string(),
                                code: "AGENT_HTTP_ERROR".to_string(),
                                message: error_message.to_string(),
                                agent_available: true,
                            })
                        }
                        Err(e) => {
                            error!("❌ Failed to parse marketing agent error response: {}", e);
                            Err(AgentErrorResponse {
                                error: "agent_error".to_string(),
                                code: "AGENT_HTTP_ERROR".to_string(),
                                message: format!("Marketing agent returned status: {}", status),
                                agent_available: true,
                            })
                        }
                    }
                }
            }
            Err(e) => {
                error!("❌ Failed to connect to marketing agent: {}", e);

                // Check if this is likely an API key issue vs agent unavailable
                let error_str = e.to_string().to_lowercase();
                let (message, code) = if error_str.contains("connection refused") {
                    (
                        "🔑 Marketing agent is not running. This is usually caused by missing or invalid LLM API keys. Please check your OpenRouter API key configuration in .env.dev and restart the service.".to_string(),
                        "AGENT_CONFIG_REQUIRED".to_string()
                    )
                } else if error_str.contains("timeout") {
                    (
                        "⏰ The marketing agent service is taking too long to respond. This may indicate an API key or network issue. Please check your configuration and try again.".to_string(),
                        "AGENT_TIMEOUT".to_string()
                    )
                } else {
                    (
                        format!("🌐 Unable to connect to the marketing agent service: {}. Please check that your API keys are configured in .env.dev and the service is running.", e),
                        "AGENT_UNAVAILABLE".to_string()
                    )
                };

                Err(AgentErrorResponse {
                    error: "connection_error".to_string(),
                    code,
                    message,
                    agent_available: false,
                })
            }
        }
    }

    /// Get agent status and health
    pub async fn get_agent_status(&self) -> Result<AgentStatus, AgentErrorResponse> {
        let client = reqwest::Client::new();
        let url = format!("{}/hecate/model-status", self.agent_base_url);

        info!("🔍 Checking agent status: {}", url);

        match client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<AgentStatus>().await {
                        Ok(status) => {
                            info!("✅ Agent status retrieved: {}", status.status);
                            Ok(status)
                        }
                        Err(e) => {
                            error!("❌ Failed to parse agent status: {}", e);
                            Err(AgentErrorResponse {
                                error: "parse_error".to_string(),
                                code: "STATUS_PARSE_ERROR".to_string(),
                                message: format!("Failed to parse status: {}", e),
                                agent_available: true,
                            })
                        }
                    }
                } else {
                    warn!("⚠️ Agent status endpoint error: {}", response.status());
                    Err(AgentErrorResponse {
                        error: "status_error".to_string(),
                        code: "STATUS_HTTP_ERROR".to_string(),
                        message: format!("Status endpoint error: {}", response.status()),
                        agent_available: false,
                    })
                }
            }
            Err(e) => {
                error!("❌ Failed to connect to agent for status: {}", e);
                Err(AgentErrorResponse {
                    error: "connection_error".to_string(),
                    code: "STATUS_UNAVAILABLE".to_string(),
                    message: format!("Agent status unavailable: {}", e),
                    agent_available: false,
                })
            }
        }
    }

    /// Get Siren agent status and health
    pub async fn get_siren_status(&self) -> Result<AgentStatus, AgentErrorResponse> {
        let client = reqwest::Client::new();
        let url = format!("{}/siren/model-status", self.agent_base_url);

        info!("🔍 Checking Siren agent status: {}", url);

        match client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<AgentStatus>().await {
                        Ok(status) => {
                            info!("✅ Siren agent status retrieved: {}", status.status);
                            Ok(status)
                        }
                        Err(e) => {
                            error!("❌ Failed to parse Siren agent status: {}", e);
                            Err(AgentErrorResponse {
                                error: "parse_error".to_string(),
                                code: "STATUS_PARSE_ERROR".to_string(),
                                message: format!("Failed to parse Siren status: {}", e),
                                agent_available: true,
                            })
                        }
                    }
                } else {
                    warn!("⚠️ Siren agent status endpoint error: {}", response.status());
                    Err(AgentErrorResponse {
                        error: "status_error".to_string(),
                        code: "STATUS_HTTP_ERROR".to_string(),
                        message: format!("Siren status endpoint error: {}", response.status()),
                        agent_available: false,
                    })
                }
            }
            Err(e) => {
                error!("❌ Failed to connect to Siren agent for status: {}", e);
                Err(AgentErrorResponse {
                    error: "connection_error".to_string(),
                    code: "STATUS_UNAVAILABLE".to_string(),
                    message: format!("Siren agent status unavailable: {}", e),
                    agent_available: false,
                })
            }
        }
    }

    /// Check if agent is healthy
    pub async fn health_check(&self) -> bool {
        let client = reqwest::Client::new();
        let url = format!("{}/hecate/health", self.agent_base_url);
        
        match client
            .get(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
        {
            Ok(response) => {
                let is_healthy = response.status().is_success();
                if is_healthy {
                    info!("✅ Agent health check passed: {}", url);
                } else {
                    warn!("⚠️ Agent health check failed: {} -> {}", url, response.status());
                }
                is_healthy
            }
            Err(e) => {
                error!("❌ Agent health check connection failed: {} -> {}", url, e);
                false
            }
        }
    }

    /// Proxy generic request to agent
    pub async fn proxy_request(&self, endpoint: &str, method: &str, body: Option<serde_json::Value>, headers: Option<&axum::http::HeaderMap>) -> Result<serde_json::Value, AgentErrorResponse> {
        let client = reqwest::Client::new();
        // Task endpoints are at root level, Hecate-specific endpoints are under /hecate
        let url = if endpoint.starts_with("tasks") {
            format!("{}/{}", self.agent_base_url, endpoint)
        } else {
            format!("{}/hecate/{}", self.agent_base_url, endpoint)
        };

        info!("🔗 Proxying {} request to agent: {}", method, url);

        let mut request_builder = match method.to_uppercase().as_str() {
            "GET" => client.get(&url),
            "POST" => client.post(&url),
            "PUT" => client.put(&url),
            "DELETE" => client.delete(&url),
            _ => return Err(AgentErrorResponse {
                error: "invalid_method".to_string(),
                code: "INVALID_HTTP_METHOD".to_string(),
                message: format!("Unsupported HTTP method: {}", method),
                agent_available: false,
            }),
        };

        if let Some(json_body) = body {
            request_builder = request_builder.json(&json_body);
        }

        // Forward wallet context headers if provided
        if let Some(header_map) = headers {
            if let Some(wallet_address) = header_map.get("x-wallet-address") {
                if let Ok(addr_str) = wallet_address.to_str() {
                    request_builder = request_builder.header("x-wallet-address", addr_str);
                    info!("🔗 Forwarding x-wallet-address: {}", addr_str);
                }
            }
            if let Some(wallet_chain) = header_map.get("x-wallet-chain") {
                if let Ok(chain_str) = wallet_chain.to_str() {
                    request_builder = request_builder.header("x-wallet-chain", chain_str);
                    info!("🔗 Forwarding x-wallet-chain: {}", chain_str);
                }
            }
        }

        match request_builder
            .timeout(std::time::Duration::from_secs(self.timeout_seconds))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<serde_json::Value>().await {
                        Ok(json_response) => {
                            info!("✅ Agent proxy response received");
                            Ok(json_response)
                        }
                        Err(e) => {
                            error!("❌ Failed to parse agent proxy response: {}", e);
                            Err(AgentErrorResponse {
                                error: "parse_error".to_string(),
                                code: "AGENT_PARSE_ERROR".to_string(),
                                message: format!("Failed to parse response: {}", e),
                                agent_available: true,
                            })
                        }
                    }
                } else {
                    warn!("⚠️ Agent proxy returned error status: {}", response.status());
                    Err(AgentErrorResponse {
                        error: "agent_error".to_string(),
                        code: "AGENT_HTTP_ERROR".to_string(),
                        message: format!("Agent returned status: {}", response.status()),
                        agent_available: true,
                    })
                }
            }
            Err(e) => {
                error!("❌ Failed to connect to agent: {}", e);

                // Check if this is likely an API key issue vs agent unavailable
                let error_str = e.to_string().to_lowercase();
                let (message, code) = if error_str.contains("connection refused") {
                    // Agent is not running - likely API key configuration issue
                    (
                        "🔑 Hecate agent is not running. This is usually caused by missing or invalid LLM API keys. Please check your OpenRouter API key configuration in .env.dev and restart the service. Visit https://openrouter.ai/ to get a free API key.".to_string(),
                        "AGENT_CONFIG_REQUIRED".to_string()
                    )
                } else if error_str.contains("timeout") {
                    (
                        "⏰ The agent service is taking too long to respond. This may indicate an API key or network issue. Please check your configuration and try again.".to_string(),
                        "AGENT_TIMEOUT".to_string()
                    )
                } else {
                    (
                        format!("🌐 Unable to connect to the agent service: {}. Please check that your API keys are configured in .env.dev and the service is running.", e),
                        "AGENT_UNAVAILABLE".to_string()
                    )
                };

                Err(AgentErrorResponse {
                    error: "connection_error".to_string(),
                    code,
                    message,
                    agent_available: false,
                })
            }
        }
    }
}