use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct McpRequest {
    pub method: String,
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct McpResponse {
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct McpCapabilities {
    pub resources: Vec<String>,
    pub tools: Vec<String>,
    pub prompts: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct McpServerInfo {
    pub name: String,
    pub version: String,
    pub protocol_version: String,
    pub capabilities: McpCapabilities,
}

impl McpResponse {
    pub fn success(result: serde_json::Value) -> Self {
        Self {
            result: Some(result),
            error: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            result: None,
            error: Some(message),
        }
    }

    pub fn method_not_found() -> Self {
        Self::error("Method not implemented".to_string())
    }
}
