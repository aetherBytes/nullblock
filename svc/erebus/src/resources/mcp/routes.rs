use axum::{
    extract::Json,
    response::Json as ResponseJson,
    http::StatusCode,
    routing::{get, post},
    Router,
};
use serde::Serialize;
use serde_json::{json, Value};
use tracing::{info, error};

fn get_protocols_service_url() -> String {
    std::env::var("PROTOCOLS_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:8001".to_string())
}

#[derive(Debug, Serialize)]
pub struct McpErrorResponse {
    pub error: String,
    pub code: String,
    pub message: String,
}

async fn mcp_proxy_request(
    endpoint: &str,
    body: Option<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<McpErrorResponse>)> {
    let client = reqwest::Client::new();
    let base_url = get_protocols_service_url();
    let url = format!("{}/{}", base_url, endpoint);

    info!("🔗 Proxying MCP request to Protocols service: {}", url);

    let request_builder = client.post(&url);

    let request_builder = if let Some(body) = body {
        info!("📤 MCP request body: {}", serde_json::to_string_pretty(&body).unwrap_or_default());
        request_builder.json(&body)
    } else {
        request_builder
    };

    match request_builder
        .header("Content-Type", "application/json")
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            match response.json::<Value>().await {
                Ok(json_response) => {
                    if status.is_success() {
                        info!("✅ MCP proxy response successful");
                        Ok(ResponseJson(json_response))
                    } else {
                        error!("❌ Protocols service returned error status: {}", status);
                        Err((
                            StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                            ResponseJson(McpErrorResponse {
                                error: "protocols_service_error".to_string(),
                                code: "MCP_SERVICE_ERROR".to_string(),
                                message: json_response.to_string(),
                            }),
                        ))
                    }
                }
                Err(e) => {
                    error!("❌ Failed to parse Protocols service response: {}", e);
                    Err((
                        StatusCode::BAD_GATEWAY,
                        ResponseJson(McpErrorResponse {
                            error: "parse_error".to_string(),
                            code: "MCP_PARSE_ERROR".to_string(),
                            message: format!("Failed to parse response: {}", e),
                        }),
                    ))
                }
            }
        }
        Err(e) => {
            error!("❌ Failed to connect to Protocols service: {}", e);
            Err((
                StatusCode::SERVICE_UNAVAILABLE,
                ResponseJson(McpErrorResponse {
                    error: "connection_error".to_string(),
                    code: "MCP_UNAVAILABLE".to_string(),
                    message: format!("Failed to connect to Protocols service: {}", e),
                }),
            ))
        }
    }
}

pub async fn mcp_jsonrpc(
    Json(request): Json<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<McpErrorResponse>)> {
    info!("🔌 MCP JSON-RPC request received");
    mcp_proxy_request("mcp/jsonrpc", Some(request)).await
}

pub async fn mcp_tools() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<McpErrorResponse>)> {
    info!("🛠️ MCP tools list requested");
    let request = json!({
        "jsonrpc": "2.0",
        "method": "tools/list",
        "id": 1
    });
    mcp_proxy_request("mcp/jsonrpc", Some(request)).await
}

pub async fn mcp_resources() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<McpErrorResponse>)> {
    info!("📚 MCP resources list requested");
    let request = json!({
        "jsonrpc": "2.0",
        "method": "resources/list",
        "id": 1
    });
    mcp_proxy_request("mcp/jsonrpc", Some(request)).await
}

pub async fn mcp_prompts() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<McpErrorResponse>)> {
    info!("💬 MCP prompts list requested");
    let request = json!({
        "jsonrpc": "2.0",
        "method": "prompts/list",
        "id": 1
    });
    mcp_proxy_request("mcp/jsonrpc", Some(request)).await
}

pub async fn mcp_health() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<McpErrorResponse>)> {
    info!("🏥 MCP health check requested");
    let protocols_url = get_protocols_service_url();
    let client = reqwest::Client::new();

    match client
        .get(format!("{}/health", protocols_url))
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => {
            Ok(ResponseJson(json!({
                "status": "healthy",
                "service": "mcp-proxy",
                "protocols_service": protocols_url,
                "message": "MCP proxy connected to Protocols service"
            })))
        }
        Ok(response) => {
            Err((
                StatusCode::BAD_GATEWAY,
                ResponseJson(McpErrorResponse {
                    error: "protocols_unhealthy".to_string(),
                    code: "MCP_PROTOCOLS_UNHEALTHY".to_string(),
                    message: format!("Protocols service returned status: {}", response.status()),
                }),
            ))
        }
        Err(e) => {
            Err((
                StatusCode::SERVICE_UNAVAILABLE,
                ResponseJson(McpErrorResponse {
                    error: "protocols_unavailable".to_string(),
                    code: "MCP_PROTOCOLS_UNAVAILABLE".to_string(),
                    message: format!("Cannot reach Protocols service: {}", e),
                }),
            ))
        }
    }
}

pub fn create_mcp_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/mcp/jsonrpc", post(mcp_jsonrpc))
        .route("/mcp/tools", get(mcp_tools))
        .route("/mcp/resources", get(mcp_resources))
        .route("/mcp/prompts", get(mcp_prompts))
        .route("/mcp/health", get(mcp_health))
        .route("/api/tools", get(mcp_tools))
}
