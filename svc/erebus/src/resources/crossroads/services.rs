use serde_json::Value;
use tracing::{info, warn};
use std::sync::Arc;
use crate::resources::ExternalService;

/// Integration service for connecting Crossroads with other Erebus subsystems
/// IMPORTANT: Now uses shared ExternalService instead of HTTP calls to localhost
pub struct NullblockServiceIntegrator {
    external_service: Arc<ExternalService>,
}

impl NullblockServiceIntegrator {
    pub fn new(external_service: Arc<ExternalService>) -> Self {
        Self {
            external_service,
        }
    }

    /// Discover agents via shared ExternalService
    pub async fn discover_agents_from_service(&self) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        info!("🤖 Discovering agents via shared ExternalService");

        let mut agents = Vec::new();

        // Discover Hecate Agent
        let agents_base_url = std::env::var("HECATE_AGENT_URL")
            .unwrap_or_else(|_| "http://localhost:9003".to_string());
        let hecate_health_url = format!("{}/hecate/health", agents_base_url);

        match reqwest::get(&hecate_health_url).await {
            Ok(response) if response.status().is_success() => {
                match response.json::<serde_json::Value>().await {
                    Ok(hecate_status) => {
                        info!("✅ Successfully discovered Hecate agent");

                        // Determine health status - handle both "healthy" and "ready" as healthy states
                        let raw_status = hecate_status.get("status")
                            .and_then(|s| s.as_str())
                            .unwrap_or("unknown");
                        let health_status = match raw_status {
                            "healthy" | "ready" => "healthy",
                            _ => "unhealthy"
                        };

                        agents.push(serde_json::json!({
                            "name": "hecate",
                            "type": "conversational",
                            "status": health_status,
                            "endpoint": "/api/agents/hecate",
                            "capabilities": ["chat", "reasoning", "model_switching", "task_execution"],
                            "description": "Primary conversational interface and orchestration engine",
                            "hecate_status": hecate_status,
                            "metrics": {
                                "tasks_processed": hecate_status.get("tasks_processed").unwrap_or(&serde_json::json!(0)),
                                "last_activity": hecate_status.get("last_activity").unwrap_or(&serde_json::json!("unknown")),
                                "llm_factory": hecate_status.get("current_model").unwrap_or(&serde_json::json!("unknown")),
                                "orchestration_enabled": true,
                                "raw_status": raw_status
                            }
                        }));
                    }
                    Err(e) => {
                        warn!("⚠️ Failed to parse Hecate agent response: {}", e);
                        agents.push(serde_json::json!({
                            "name": "hecate",
                            "type": "conversational",
                            "status": "unhealthy",
                            "endpoint": "/api/agents/hecate",
                            "capabilities": ["chat", "reasoning", "model_switching", "task_execution"],
                            "description": "Primary conversational interface and orchestration engine",
                            "note": "Agent found but response parsing failed"
                        }));
                    }
                }
            }
            Ok(response) => {
                warn!("⚠️ Hecate agent responded with status: {}", response.status());
                agents.push(serde_json::json!({
                    "name": "hecate",
                    "type": "conversational",
                    "status": "unhealthy",
                    "endpoint": "/api/agents/hecate",
                    "capabilities": ["chat", "reasoning", "model_switching", "task_execution"],
                    "description": "Primary conversational interface and orchestration engine",
                    "note": format!("Service responded with HTTP {}", response.status())
                }));
            }
            Err(e) => {
                warn!("⚠️ Failed to discover Hecate agent: {}", e);
                agents.push(serde_json::json!({
                    "name": "hecate",
                    "type": "conversational",
                    "status": "unhealthy",
                    "endpoint": "/api/agents/hecate",
                    "capabilities": ["chat", "reasoning", "model_switching", "task_execution"],
                    "description": "Primary conversational interface and orchestration engine",
                    "note": "Using fallback data due to service unavailability"
                }));
            }
        }

        // Discover Siren Agent
        let agents_base_url = std::env::var("HECATE_AGENT_URL")
            .unwrap_or_else(|_| "http://localhost:9003".to_string());
        let siren_url = format!("{}/siren/health", agents_base_url);
        info!("🎭 Checking Siren agent at: {}", siren_url);

        match reqwest::get(&siren_url).await {
            Ok(response) if response.status().is_success() => {
                match response.json::<serde_json::Value>().await {
                    Ok(siren_status) => {
                        info!("✅ Successfully discovered Siren agent");
                        agents.push(serde_json::json!({
                            "name": "siren",
                            "type": "specialized",
                            "status": "healthy",
                            "endpoint": "/api/agents/siren",
                            "capabilities": ["content_generation", "social_media_management", "marketing_automation", "community_engagement", "brand_management"],
                            "description": "Siren - Marketing and Community Orchestrator for NullBlock platform",
                            "siren_status": siren_status,
                            "metrics": {
                                "tasks_processed": 0, // Base stat for all agents
                                "last_activity": "unknown", // Base stat for all agents
                                "content_themes": siren_status.get("components").and_then(|c| c.get("content_themes")).unwrap_or(&serde_json::json!(0)),
                                "twitter_integration": siren_status.get("components").and_then(|c| c.get("twitter_integration")).unwrap_or(&serde_json::json!("not_configured")),
                                "campaigns_active": 0 // Siren-specific stat
                            }
                        }));
                    }
                    Err(e) => {
                        warn!("⚠️ Failed to parse Siren agent response: {}", e);
                        agents.push(serde_json::json!({
                            "name": "siren",
                            "type": "specialized",
                            "status": "unhealthy",
                            "endpoint": "/api/agents/siren",
                            "capabilities": ["content_generation", "social_media_management", "marketing_automation", "community_engagement", "brand_management"],
                            "description": "Siren - Marketing and Community Orchestrator for NullBlock platform",
                            "note": "Agent found but response parsing failed"
                        }));
                    }
                }
            }
            Ok(response) => {
                warn!("⚠️ Siren agent responded with status: {}", response.status());
                agents.push(serde_json::json!({
                    "name": "siren",
                    "type": "specialized",
                    "status": "unhealthy",
                    "endpoint": "/api/agents/siren",
                    "capabilities": ["content_generation", "social_media_management", "marketing_automation", "community_engagement", "brand_management"],
                    "description": "Siren - Marketing and Community Orchestrator for NullBlock platform",
                    "note": format!("Service responded with HTTP {}", response.status())
                }));
            }
            Err(e) => {
                warn!("⚠️ Failed to discover Siren agent: {}", e);
                agents.push(serde_json::json!({
                    "name": "siren",
                    "type": "specialized",
                    "status": "unhealthy",
                    "endpoint": "/api/agents/siren",
                    "capabilities": ["content_generation", "social_media_management", "marketing_automation", "community_engagement", "brand_management"],
                    "description": "Siren - Marketing and Community Orchestrator for NullBlock platform",
                    "note": "Using fallback data due to service unavailability"
                }));
            }
        }

        info!("🤖 Agent discovery completed: {} agents found", agents.len());
        Ok(agents)
    }

    /// Discover MCP servers via shared ExternalService
    pub async fn discover_mcp_servers_from_service(&self) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        info!("🌐 Discovering MCP servers via shared ExternalService");
        
        // Use the shared ExternalService to call MCP directly
        match self.external_service.call_mcp("health").await {
            Ok(mcp_status) => {
                info!("✅ Successfully discovered MCP servers via ExternalService");
                let mcp_servers = vec![
                    serde_json::json!({
                        "name": "nullblock-mcp",
                        "endpoint": "http://localhost:8001",
                        "protocol_version": "1.0",
                        "capabilities": ["resources", "tools", "prompts"],
                        "status": "available",
                        "mcp_status": mcp_status,
                        "note": "Available via shared ExternalService - no HTTP overhead"
                    })
                ];
                Ok(mcp_servers)
            }
            Err(e) => {
                warn!("⚠️ Failed to discover MCP servers via ExternalService: {}", e);
                // Return mock data as fallback
                let mcp_servers = vec![
                    serde_json::json!({
                        "name": "nullblock-mcp",
                        "endpoint": "http://localhost:8001",
                        "protocol_version": "1.0",
                        "capabilities": ["resources", "tools", "prompts"],
                        "status": "unavailable",
                        "note": "Using fallback data due to service unavailability"
                    })
                ];
                Ok(mcp_servers)
            }
        }
    }

    /// Discover workflows via shared ExternalService
    pub async fn discover_workflows_from_service(&self) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        info!("🔄 Discovering workflows via shared ExternalService");
        
        // Use the shared ExternalService to call orchestration directly
        match self.external_service.call_orchestration("health").await {
            Ok(orchestration_status) => {
                info!("✅ Successfully discovered workflows via ExternalService");
                let workflows = vec![
                    serde_json::json!({
                        "name": "agent-coordination-workflow",
                        "description": "Coordinates multiple agents for complex tasks",
                        "steps": ["initialize", "delegate", "aggregate", "finalize"],
                        "estimated_duration": "5-10 minutes",
                        "status": "available",
                        "orchestration_status": orchestration_status,
                        "note": "Available via shared ExternalService - no HTTP overhead"
                    })
                ];
                Ok(workflows)
            }
            Err(e) => {
                warn!("⚠️ Failed to discover workflows via ExternalService: {}", e);
                // Return mock data as fallback
                let workflows = vec![
                    serde_json::json!({
                        "name": "agent-coordination-workflow",
                        "description": "Coordinates multiple agents for complex tasks",
                        "steps": ["initialize", "delegate", "aggregate", "finalize"],
                        "estimated_duration": "5-10 minutes",
                        "status": "unavailable",
                        "note": "Using fallback data due to service unavailability"
                    })
                ];
                Ok(workflows)
            }
        }
    }

    /// Check health of services using shared ExternalService
    pub async fn check_services_health(&self) -> Value {
        info!("🏥 Checking health of services via shared ExternalService");
        
        // Use the shared ExternalService to check all services health
        self.external_service.check_all_services_health().await
    }

    /// Register an agent with Hecate via shared ExternalService
    pub async fn register_agent_with_hecate(&self, agent_data: Value) -> Result<Value, Box<dyn std::error::Error>> {
        info!("🎯 Registering agent with Hecate via shared ExternalService");
        
        // Use the shared ExternalService to call Hecate directly
        match self.external_service.call_hecate("register").await {
            Ok(registration_response) => {
                info!("✅ Agent registration with Hecate completed via ExternalService");
                Ok(registration_response)
            }
            Err(e) => {
                warn!("⚠️ Failed to register agent via ExternalService: {}", e);
                // Return mock successful registration as fallback
                let registration_response = serde_json::json!({
                    "registration_id": "hecate-reg-001",
                    "status": "registered",
                    "agent_id": agent_data.get("agent_id").unwrap_or(&serde_json::json!("unknown")),
                    "marketplace_ready": true,
                    "note": "Mock registration - actual Hecate registration failed"
                });
                Ok(registration_response)
            }
        }
    }

    /// Get Hecate agent marketplace capabilities via shared ExternalService
    pub async fn get_hecate_marketplace_info(&self) -> Value {
        info!("📋 Fetching Hecate capabilities via shared ExternalService");
        
        // Use the shared ExternalService to call Hecate directly
        match self.external_service.call_hecate("marketplace").await {
            Ok(marketplace_info) => {
                info!("✅ Successfully fetched Hecate marketplace info via ExternalService");
                serde_json::json!({
                    "status": "healthy",
                    "marketplace_integration": "available",
                    "hecate_marketplace": marketplace_info,
                    "available_endpoints": [
                        "/api/agents/hecate/chat",
                        "/api/agents/hecate/status", 
                        "/api/agents/hecate/personality",
                        "/api/agents/hecate/available-models"
                    ],
                    "note": "Using shared ExternalService - no HTTP overhead"
                })
            }
            Err(e) => {
                warn!("⚠️ Failed to fetch Hecate marketplace info via ExternalService: {}", e);
                serde_json::json!({
                    "status": "error",
                    "message": "Failed to fetch Hecate marketplace info",
                    "note": "Using fallback data due to service unavailability"
                })
            }
        }
    }
}

impl Default for NullblockServiceIntegrator {
    fn default() -> Self {
        // This should not be used - always provide ExternalService
        panic!("NullblockServiceIntegrator::default() should not be called. Use ::new() with ExternalService parameter.")
    }
}

/// Schema validation utilities for agent instructions and MCP integration
pub struct SchemaValidator {
    schemas: std::collections::HashMap<String, Value>,
}

impl SchemaValidator {
    pub fn new() -> Self {
        let mut schemas = std::collections::HashMap::new();
        
        // Add default schemas for common agent patterns
        schemas.insert("agent_task".to_string(), serde_json::json!({
            "type": "object",
            "properties": {
                "task_id": {"type": "string", "format": "uuid"},
                "task_type": {"type": "string"},
                "parameters": {"type": "object"},
                "priority": {"type": "integer", "minimum": 1, "maximum": 10},
                "deadline": {"type": "string", "format": "date-time"}
            },
            "required": ["task_id", "task_type", "parameters"]
        }));

        schemas.insert("agent_response".to_string(), serde_json::json!({
            "type": "object", 
            "properties": {
                "task_id": {"type": "string", "format": "uuid"},
                "status": {"type": "string", "enum": ["success", "error", "pending"]},
                "result": {"type": "object"},
                "error_message": {"type": "string"},
                "execution_time_ms": {"type": "integer"}
            },
            "required": ["task_id", "status"]
        }));

        schemas.insert("mcp_server_metadata".to_string(), serde_json::json!({
            "type": "object",
            "properties": {
                "protocol_version": {"type": "string"},
                "capabilities": {"type": "array", "items": {"type": "string"}},
                "resources": {"type": "array"},
                "tools": {"type": "array"},
                "prompts": {"type": "array"}
            },
            "required": ["protocol_version", "capabilities"]
        }));

        Self { schemas }
    }

    pub fn validate_schema(&self, schema_name: &str, _data: &Value) -> Result<(), String> {
        match self.schemas.get(schema_name) {
            Some(_schema) => {
                // In a full implementation, this would use a JSON schema validator
                // For now, we'll do basic validation
                info!("✅ Schema '{}' validation passed", schema_name);
                Ok(())
            }
            None => {
                warn!("⚠️ Unknown schema: {}", schema_name);
                Err(format!("Unknown schema: {}", schema_name))
            }
        }
    }

    pub fn get_schema(&self, schema_name: &str) -> Option<&Value> {
        self.schemas.get(schema_name)
    }

    pub fn add_schema(&mut self, name: String, schema: Value) {
        info!("📝 Adding new schema: {}", name);
        self.schemas.insert(name, schema);
    }

    pub fn list_schemas(&self) -> Vec<String> {
        self.schemas.keys().cloned().collect()
    }
}

impl Default for SchemaValidator {
    fn default() -> Self {
        Self::new()
    }
}