use axum::{extract::State, Json};
use serde_json::json;
use tracing::{error, info, warn};

use crate::server::AppState;
use super::types::*;
use super::tools::get_all_tools;
use super::handlers::execute_tool;

pub async fn handle_jsonrpc(
    State(state): State<AppState>,
    Json(request): Json<JsonRpcRequest>,
) -> Json<JsonRpcResponse> {
    info!("MCP JSON-RPC request: method={}, id={:?}", request.method, request.id);

    if request.jsonrpc != JSONRPC_VERSION {
        warn!("Invalid JSON-RPC version: {}", request.jsonrpc);
        return Json(JsonRpcResponse::error(
            request.id,
            error_codes::INVALID_REQUEST,
            format!("Invalid JSON-RPC version: {}", request.jsonrpc),
        ));
    }

    let result = match request.method.as_str() {
        "initialize" => handle_initialize(request.params).await,
        "initialized" => {
            info!("Client initialized notification received");
            Ok(json!({}))
        }
        "tools/list" => handle_list_tools().await,
        "tools/call" => handle_call_tool(&state, request.params).await,
        "ping" => {
            info!("Ping received");
            Ok(json!({}))
        }
        "resources/list" => {
            Ok(json!({ "resources": [] }))
        }
        "prompts/list" => {
            Ok(json!({ "prompts": [] }))
        }
        _ => {
            warn!("Unknown method: {}", request.method);
            Err((error_codes::METHOD_NOT_FOUND, format!("Method not found: {}", request.method)))
        }
    };

    let response = match result {
        Ok(value) => {
            info!("MCP JSON-RPC request succeeded");
            JsonRpcResponse::success(request.id, value)
        }
        Err((code, message)) => {
            error!("MCP JSON-RPC request failed: {}", message);
            JsonRpcResponse::error(request.id, code, message)
        }
    };

    Json(response)
}

async fn handle_initialize(params: Option<serde_json::Value>) -> Result<serde_json::Value, (i32, String)> {
    let _init_request: InitializeRequest = match params {
        Some(p) => serde_json::from_value(p).map_err(|e| {
            error!("Failed to parse initialize request: {}", e);
            (error_codes::INVALID_PARAMS, format!("Invalid parameters: {}", e))
        })?,
        None => return Err((error_codes::INVALID_PARAMS, "Missing parameters".to_string())),
    };

    let result = InitializeResult {
        protocol_version: PROTOCOL_VERSION.to_string(),
        capabilities: ServerCapabilities {
            experimental: None,
            logging: None,
            prompts: Some(PromptsCapability { list_changed: false }),
            resources: Some(ResourcesCapability { subscribe: false, list_changed: false }),
            tools: Some(ToolsCapability { list_changed: false }),
        },
        server_info: Implementation {
            name: "arb-farm".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            title: Some("ArbFarm MEV Agent Swarm".to_string()),
        },
        instructions: Some(
            "ArbFarm MCP server providing Solana MEV trading tools. \
            Available tool categories: scanner, edge, strategy, curve, threat, \
            kol, research, engram, consensus, swarm, approval, and learning tools.".to_string()
        ),
    };

    serde_json::to_value(result).map_err(|e| {
        error!("Failed to serialize initialize result: {}", e);
        (error_codes::INTERNAL_ERROR, "Serialization error".to_string())
    })
}

async fn handle_list_tools() -> Result<serde_json::Value, (i32, String)> {
    let mcp_tools = get_all_tools();
    let tools: Vec<Tool> = mcp_tools.iter().map(Tool::from).collect();

    let result = ListToolsResult { tools };

    serde_json::to_value(result).map_err(|e| {
        error!("Failed to serialize tools list: {}", e);
        (error_codes::INTERNAL_ERROR, "Serialization error".to_string())
    })
}

async fn handle_call_tool(
    state: &AppState,
    params: Option<serde_json::Value>,
) -> Result<serde_json::Value, (i32, String)> {
    let call_request: CallToolRequest = match params {
        Some(p) => serde_json::from_value(p).map_err(|e| {
            error!("Failed to parse call tool request: {}", e);
            (error_codes::INVALID_PARAMS, format!("Invalid parameters: {}", e))
        })?,
        None => return Err((error_codes::INVALID_PARAMS, "Missing parameters".to_string())),
    };

    let args = match call_request.arguments {
        Some(map) => serde_json::to_value(map).unwrap_or(json!({})),
        None => json!({}),
    };

    let tool_result = execute_tool(state, &call_request.name, args).await;

    let result = CallToolResult {
        content: vec![ContentBlock::Text {
            text: tool_result.content.first()
                .map(|c| c.text.clone())
                .unwrap_or_default(),
        }],
        is_error: tool_result.is_error,
    };

    serde_json::to_value(result).map_err(|e| {
        error!("Failed to serialize tool result: {}", e);
        (error_codes::INTERNAL_ERROR, "Serialization error".to_string())
    })
}
