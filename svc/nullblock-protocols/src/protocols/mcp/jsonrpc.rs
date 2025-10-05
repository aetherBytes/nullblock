use axum::{extract::State, http::StatusCode, Json};
use serde_json::json;
use tracing::{error, info, warn};

use crate::server::AppState;
use super::types::*;
use super::handlers;

pub async fn handle_jsonrpc(
    State(state): State<AppState>,
    Json(request): Json<JsonRpcRequest>,
) -> Result<Json<JsonRpcResponse>, StatusCode> {
    info!("📨 MCP JSON-RPC request: method={}, id={:?}", request.method, request.id);

    if request.jsonrpc != JSONRPC_VERSION {
        warn!("⚠️ Invalid JSON-RPC version: {}", request.jsonrpc);
        return Ok(Json(JsonRpcResponse::error(
            request.id,
            error_codes::INVALID_REQUEST,
            format!("Invalid JSON-RPC version: {}", request.jsonrpc),
        )));
    }

    let result = match request.method.as_str() {
        "initialize" => {
            match serde_json::from_value::<InitializeRequest>(request.params.unwrap_or(json!({}))) {
                Ok(init_request) => {
                    match handlers::initialize(State(state.clone()), init_request).await {
                        Ok(Json(result)) => {
                            match serde_json::to_value(result) {
                                Ok(value) => Ok(value),
                                Err(e) => {
                                    error!("❌ Failed to serialize initialize result: {}", e);
                                    Err((error_codes::INTERNAL_ERROR, "Serialization error".to_string()))
                                }
                            }
                        }
                        Err(status) => {
                            error!("❌ Initialize handler failed: {}", status);
                            Err((error_codes::INTERNAL_ERROR, "Initialize failed".to_string()))
                        }
                    }
                }
                Err(e) => {
                    error!("❌ Failed to parse initialize request: {}", e);
                    Err((error_codes::INVALID_PARAMS, format!("Invalid parameters: {}", e)))
                }
            }
        }
        "initialized" => {
            info!("✅ Client initialized notification received");
            Ok(json!({}))
        }
        "resources/list" => {
            match handlers::list_resources(State(state.clone())).await {
                Ok(Json(result)) => {
                    match serde_json::to_value(result) {
                        Ok(value) => Ok(value),
                        Err(e) => {
                            error!("❌ Failed to serialize resources list: {}", e);
                            Err((error_codes::INTERNAL_ERROR, "Serialization error".to_string()))
                        }
                    }
                }
                Err(status) => {
                    error!("❌ List resources handler failed: {}", status);
                    Err((error_codes::INTERNAL_ERROR, "Failed to list resources".to_string()))
                }
            }
        }
        "resources/read" => {
            match serde_json::from_value::<ReadResourceRequest>(request.params.unwrap_or(json!({}))) {
                Ok(read_request) => {
                    match handlers::read_resource(State(state.clone()), read_request).await {
                        Ok(Json(result)) => {
                            match serde_json::to_value(result) {
                                Ok(value) => Ok(value),
                                Err(e) => {
                                    error!("❌ Failed to serialize resource: {}", e);
                                    Err((error_codes::INTERNAL_ERROR, "Serialization error".to_string()))
                                }
                            }
                        }
                        Err(status) => {
                            error!("❌ Read resource handler failed: {}", status);
                            Err((error_codes::INTERNAL_ERROR, "Failed to read resource".to_string()))
                        }
                    }
                }
                Err(e) => {
                    error!("❌ Failed to parse read resource request: {}", e);
                    Err((error_codes::INVALID_PARAMS, format!("Invalid parameters: {}", e)))
                }
            }
        }
        "tools/list" => {
            match handlers::list_tools(State(state.clone())).await {
                Ok(Json(result)) => {
                    match serde_json::to_value(result) {
                        Ok(value) => Ok(value),
                        Err(e) => {
                            error!("❌ Failed to serialize tools list: {}", e);
                            Err((error_codes::INTERNAL_ERROR, "Serialization error".to_string()))
                        }
                    }
                }
                Err(status) => {
                    error!("❌ List tools handler failed: {}", status);
                    Err((error_codes::INTERNAL_ERROR, "Failed to list tools".to_string()))
                }
            }
        }
        "tools/call" => {
            match serde_json::from_value::<CallToolRequest>(request.params.unwrap_or(json!({}))) {
                Ok(call_request) => {
                    match handlers::call_tool(State(state.clone()), call_request).await {
                        Ok(Json(result)) => {
                            match serde_json::to_value(result) {
                                Ok(value) => Ok(value),
                                Err(e) => {
                                    error!("❌ Failed to serialize tool result: {}", e);
                                    Err((error_codes::INTERNAL_ERROR, "Serialization error".to_string()))
                                }
                            }
                        }
                        Err(status) => {
                            error!("❌ Call tool handler failed: {}", status);
                            Err((error_codes::INTERNAL_ERROR, "Failed to call tool".to_string()))
                        }
                    }
                }
                Err(e) => {
                    error!("❌ Failed to parse call tool request: {}", e);
                    Err((error_codes::INVALID_PARAMS, format!("Invalid parameters: {}", e)))
                }
            }
        }
        "prompts/list" => {
            match handlers::list_prompts(State(state.clone())).await {
                Ok(Json(result)) => {
                    match serde_json::to_value(result) {
                        Ok(value) => Ok(value),
                        Err(e) => {
                            error!("❌ Failed to serialize prompts list: {}", e);
                            Err((error_codes::INTERNAL_ERROR, "Serialization error".to_string()))
                        }
                    }
                }
                Err(status) => {
                    error!("❌ List prompts handler failed: {}", status);
                    Err((error_codes::INTERNAL_ERROR, "Failed to list prompts".to_string()))
                }
            }
        }
        "prompts/get" => {
            match serde_json::from_value::<GetPromptRequest>(request.params.unwrap_or(json!({}))) {
                Ok(get_request) => {
                    match handlers::get_prompt(State(state.clone()), get_request).await {
                        Ok(Json(result)) => {
                            match serde_json::to_value(result) {
                                Ok(value) => Ok(value),
                                Err(e) => {
                                    error!("❌ Failed to serialize prompt: {}", e);
                                    Err((error_codes::INTERNAL_ERROR, "Serialization error".to_string()))
                                }
                            }
                        }
                        Err(status) => {
                            error!("❌ Get prompt handler failed: {}", status);
                            Err((error_codes::INTERNAL_ERROR, "Failed to get prompt".to_string()))
                        }
                    }
                }
                Err(e) => {
                    error!("❌ Failed to parse get prompt request: {}", e);
                    Err((error_codes::INVALID_PARAMS, format!("Invalid parameters: {}", e)))
                }
            }
        }
        "ping" => {
            info!("🏓 Ping received");
            Ok(json!({}))
        }
        _ => {
            warn!("⚠️ Unknown method: {}", request.method);
            Err((error_codes::METHOD_NOT_FOUND, format!("Method not found: {}", request.method)))
        }
    };

    let response = match result {
        Ok(value) => {
            info!("✅ MCP JSON-RPC request succeeded");
            JsonRpcResponse::success(request.id, value)
        }
        Err((code, message)) => {
            error!("❌ MCP JSON-RPC request failed: {}", message);
            JsonRpcResponse::error(request.id, code, message)
        }
    };

    Ok(Json(response))
}

