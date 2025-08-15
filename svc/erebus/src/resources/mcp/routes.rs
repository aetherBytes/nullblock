// MCP-specific HTTP routes and handlers
use axum::{
    extract::State,
    response::Json,
    Router,
    routing::post,
};

use super::{types::{McpRequest, McpResponse}, handler::McpHandler};

/// Create MCP routes for the main router
pub fn create_mcp_routes() -> Router<McpHandler> {
    Router::new()
        .route("/mcp", post(handle_mcp_request))
        .route("/mcp/initialize", post(handle_mcp_initialize))
        .route("/mcp/resources", post(handle_mcp_resources))
        .route("/mcp/tools", post(handle_mcp_tools))
        .route("/mcp/prompts", post(handle_mcp_prompts))
}

/// Main MCP protocol endpoint
async fn handle_mcp_request(
    State(handler): State<McpHandler>,
    Json(request): Json<McpRequest>,
) -> Json<McpResponse> {
    println!("🔥 MCP request received: {}", request.method);
    Json(handler.handle_request(request))
}

/// MCP initialization endpoint
async fn handle_mcp_initialize(
    State(handler): State<McpHandler>,
) -> Json<McpResponse> {
    let init_request = McpRequest {
        method: "initialize".to_string(),
        params: None,
    };
    Json(handler.handle_request(init_request))
}

/// MCP resources listing endpoint
async fn handle_mcp_resources(
    State(handler): State<McpHandler>,
) -> Json<McpResponse> {
    let resources_request = McpRequest {
        method: "resources/list".to_string(), 
        params: None,
    };
    Json(handler.handle_request(resources_request))
}

/// MCP tools listing endpoint
async fn handle_mcp_tools(
    State(handler): State<McpHandler>,
) -> Json<McpResponse> {
    let tools_request = McpRequest {
        method: "tools/list".to_string(),
        params: None,
    };
    Json(handler.handle_request(tools_request))
}

/// MCP prompts listing endpoint  
async fn handle_mcp_prompts(
    State(handler): State<McpHandler>,
) -> Json<McpResponse> {
    let prompts_request = McpRequest {
        method: "prompts/list".to_string(),
        params: None,
    };
    Json(handler.handle_request(prompts_request))
}