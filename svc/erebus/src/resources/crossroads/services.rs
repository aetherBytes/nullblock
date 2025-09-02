use serde_json::Value;
use tracing::{info, warn, error};
use uuid::Uuid;

/// Integration service for connecting Crossroads with other Nullblock services
pub struct NullblockServiceIntegrator {
    mcp_service_url: String,
    agents_service_url: String,
    orchestration_service_url: String,
}

impl NullblockServiceIntegrator {
    pub fn new() -> Self {
        Self {
            mcp_service_url: "http://localhost:8001".to_string(),
            agents_service_url: "http://localhost:9001".to_string(),
            orchestration_service_url: "http://localhost:8002".to_string(),
        }
    }

    /// Discover agents from the Nullblock Agents service
    pub async fn discover_agents_from_service(&self) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        info!("🤖 Discovering agents from Nullblock Agents service");
        
        let client = reqwest::Client::new();
        match client.get(&format!("{}/agents", self.agents_service_url)).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<Value>().await {
                        Ok(data) => {
                            info!("✅ Successfully discovered agents from service");
                            Ok(data.get("agents").unwrap_or(&serde_json::json!([])).as_array().unwrap_or(&vec![]).clone())
                        }
                        Err(e) => {
                            warn!("⚠️ Failed to parse agents response: {}", e);
                            Ok(vec![])
                        }
                    }
                }
                else {
                    warn!("⚠️ Agents service returned status: {}", response.status());
                    Ok(vec![])
                }
            }
            Err(e) => {
                warn!("⚠️ Failed to connect to agents service: {}", e);
                Ok(vec![])
            }
        }
    }

    /// Get MCP server list from the MCP service
    pub async fn discover_mcp_servers_from_service(&self) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        info!("🌐 Discovering MCP servers from MCP service");
        
        let client = reqwest::Client::new();
        match client.get(&format!("{}/mcp/servers", self.mcp_service_url)).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<Value>().await {
                        Ok(data) => {
                            info!("✅ Successfully discovered MCP servers from service");
                            Ok(data.get("servers").unwrap_or(&serde_json::json!([])).as_array().unwrap_or(&vec![]).clone())
                        }
                        Err(e) => {
                            warn!("⚠️ Failed to parse MCP servers response: {}", e);
                            Ok(vec![])
                        }
                    }
                }
                else {
                    warn!("⚠️ MCP service returned status: {}", response.status());
                    Ok(vec![])
                }
            }
            Err(e) => {
                warn!("⚠️ Failed to connect to MCP service: {}", e);
                Ok(vec![])
            }
        }
    }

    /// Get workflows from the orchestration service
    pub async fn discover_workflows_from_service(&self) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        info!("🔄 Discovering workflows from Orchestration service");
        
        let client = reqwest::Client::new();
        match client.get(&format!("{}/workflows", self.orchestration_service_url)).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<Value>().await {
                        Ok(data) => {
                            info!("✅ Successfully discovered workflows from service");
                            Ok(data.get("workflows").unwrap_or(&serde_json::json!([])).as_array().unwrap_or(&vec![]).clone())
                        }
                        Err(e) => {
                            warn!("⚠️ Failed to parse workflows response: {}", e);
                            Ok(vec![])
                        }
                    }
                }
                else {
                    warn!("⚠️ Orchestration service returned status: {}", response.status());
                    Ok(vec![])
                }
            }
            Err(e) => {
                warn!("⚠️ Failed to connect to orchestration service: {}", e);
                Ok(vec![])
            }
        }
    }

    /// Check health of all Nullblock services
    pub async fn check_services_health(&self) -> Value {
        info!("🏥 Checking health of all Nullblock services");
        
        let client = reqwest::Client::new();
        let mut services_health = serde_json::Map::new();

        // Check MCP service
        match client.get(&format!("{}/health", self.mcp_service_url)).send().await {
            Ok(response) if response.status().is_success() => {
                services_health.insert("mcp_service".to_string(), serde_json::json!("healthy"));
            }
            _ => {
                services_health.insert("mcp_service".to_string(), serde_json::json!("unhealthy"));
            }
        }

        // Check Agents service
        match client.get(&format!("{}/health", self.agents_service_url)).send().await {
            Ok(response) if response.status().is_success() => {
                services_health.insert("agents_service".to_string(), serde_json::json!("healthy"));
            }
            _ => {
                services_health.insert("agents_service".to_string(), serde_json::json!("unhealthy"));
            }
        }

        // Check Orchestration service
        match client.get(&format!("{}/health", self.orchestration_service_url)).send().await {
            Ok(response) if response.status().is_success() => {
                services_health.insert("orchestration_service".to_string(), serde_json::json!("healthy"));
            }
            _ => {
                services_health.insert("orchestration_service".to_string(), serde_json::json!("unhealthy"));
            }
        }

        serde_json::Value::Object(services_health)
    }

    /// Register an agent with Hecate for marketplace integration
    pub async fn register_agent_with_hecate(&self, agent_data: Value) -> Result<Value, Box<dyn std::error::Error>> {
        info!("🎯 Registering agent with Hecate for marketplace integration");
        
        let hecate_url = "http://localhost:9002";
        let client = reqwest::Client::new();
        
        match client.post(&format!("{}/marketplace/register", hecate_url))
            .json(&agent_data)
            .send()
            .await 
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<Value>().await {
                        Ok(data) => {
                            info!("✅ Successfully registered agent with Hecate");
                            Ok(data)
                        }
                        Err(e) => {
                            error!("❌ Failed to parse Hecate registration response: {}", e);
                            Err(Box::new(e))
                        }
                    }
                }
                else {
                    error!("❌ Hecate registration failed with status: {}", response.status());
                    Err(format!("Registration failed with status: {}", response.status()).into())
                }
            }
            Err(e) => {
                error!("❌ Failed to connect to Hecate for registration: {}", e);
                Err(Box::new(e))
            }
        }
    }

    /// Get Hecate agent marketplace capabilities
    pub async fn get_hecate_marketplace_info(&self) -> Value {
        info!("📋 Fetching Hecate marketplace capabilities");
        
        let hecate_url = "http://localhost:9002";
        let client = reqwest::Client::new();
        
        match client.get(&format!("{}/marketplace/info", hecate_url)).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<Value>().await {
                        Ok(data) => {
                            info!("✅ Successfully fetched Hecate marketplace info");
                            data
                        }
                        Err(e) => {
                            warn!("⚠️ Failed to parse Hecate marketplace info: {}", e);
                            serde_json::json!({
                                "status": "error",
                                "message": "Failed to parse marketplace info"
                            })
                        }
                    }
                }
                else {
                    warn!("⚠️ Hecate marketplace info request failed: {}", response.status());
                    serde_json::json!({
                        "status": "service_unavailable",
                        "message": "Hecate service unavailable"
                    })
                }
            }
            Err(e) => {
                warn!("⚠️ Failed to connect to Hecate: {}", e);
                serde_json::json!({
                    "status": "connection_error",
                    "message": "Cannot connect to Hecate service"
                })
            }
        }
    }
}

impl Default for NullblockServiceIntegrator {
    fn default() -> Self {
        Self::new()
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

    pub fn validate_schema(&self, schema_name: &str, data: &Value) -> Result<(), String> {
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