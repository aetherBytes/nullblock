// Agent routing endpoints for Erebus
use axum::{
    extract::{Path, Json, Query},
    response::Json as ResponseJson,
    http::{StatusCode, HeaderMap},
};
use std::collections::HashMap;
use serde_json::Value;
use tracing::{info, error, warn};
use uuid::Uuid;

use super::proxy::{AgentProxy, AgentRequest, AgentResponse, AgentStatus, AgentErrorResponse};

/// Hecate agent proxy instance - now points to Rust service
fn get_hecate_proxy() -> AgentProxy {
    let hecate_url = std::env::var("HECATE_AGENT_URL")
        .unwrap_or_else(|_| "http://localhost:9003".to_string());
    AgentProxy::new(hecate_url)
}

/// Marketing agent proxy instance - also uses the Rust service
fn get_marketing_proxy() -> AgentProxy {
    let marketing_url = std::env::var("MARKETING_AGENT_URL")
        .unwrap_or_else(|_| "http://localhost:9003".to_string());
    AgentProxy::new(marketing_url)
}

/// Extract wallet address from request headers and create user reference if needed
async fn extract_wallet_and_create_user(headers: &HeaderMap) -> Option<Uuid> {
    let wallet_address = headers.get("x-wallet-address")
        .and_then(|h| h.to_str().ok())?;
    let wallet_chain = headers.get("x-wallet-chain")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");

    info!("🔍 Extracted wallet: {} on chain: {}", wallet_address, wallet_chain);

    // Call Erebus user registration API instead of direct database access
    let default_source_type = serde_json::json!({
        "type": "web3_wallet",
        "provider": "unknown",
        "metadata": {}
    });
    match call_erebus_user_registration_api(wallet_address, wallet_chain, Some(default_source_type)).await {
        Ok(user_id) => {
            info!("✅ User reference created/updated via Erebus API: {}", user_id);
            Some(user_id)
        }
        Err(e) => {
            error!("❌ Failed to create user reference via Erebus API: {}", e);
            None
        }
    }
}

/// Call Erebus user registration API (replaces direct database access)
async fn call_erebus_user_registration_api(wallet_address: &str, chain: &str, source_type: Option<serde_json::Value>) -> Result<Uuid, String> {
    let erebus_url = "http://localhost:3000";
    let client = reqwest::Client::new();

    let request_body = serde_json::json!({
        "source_identifier": wallet_address,
        "chain": chain,
        "source_type": source_type.unwrap_or_else(|| serde_json::json!({
            "type": "web3_wallet",
            "provider": "unknown",
            "metadata": {}
        })),
        "wallet_type": "unknown"
    });

    info!("🌐 Calling Erebus user registration API: {}/api/users/register", erebus_url);

    match client
        .post(&format!("{}/api/users/register", erebus_url))
        .json(&request_body)
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(json_response) => {
                        if let Some(user_id_str) = json_response["user_id"].as_str() {
                            match Uuid::parse_str(user_id_str) {
                                Ok(user_id) => Ok(user_id),
                                Err(e) => Err(format!("Invalid UUID in response: {}", e))
                            }
                        } else {
                            Err("No user_id in response".to_string())
                        }
                    }
                    Err(e) => Err(format!("Failed to parse response JSON: {}", e))
                }
            } else {
                let error_text = response.text().await.unwrap_or_default();
                Err(format!("Erebus API error: {}", error_text))
            }
        }
        Err(e) => Err(format!("Failed to call Erebus API: {}", e))
    }
}


/// Register user endpoint - called when wallet connects
pub async fn register_user(
    headers: HeaderMap,
    Json(request): Json<Value>
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<AgentErrorResponse>)> {
    info!("👤 User registration request received");
    info!("📝 Request payload: {}", serde_json::to_string_pretty(&request).unwrap_or_default());

    // Extract wallet information from headers or request body
    let wallet_address = headers.get("x-wallet-address")
        .and_then(|h| h.to_str().ok())
        .or_else(|| request["source_identifier"].as_str());
    
    let wallet_chain = headers.get("x-wallet-chain")
        .and_then(|h| h.to_str().ok())
        .or_else(|| request["chain"].as_str())
        .unwrap_or("unknown");

    if let Some(wallet_address) = wallet_address {
        info!("🔍 Registering user with wallet: {} on chain: {}", wallet_address, wallet_chain);
        
        // Extract source_type from request
        let source_type = request["source_type"].as_object().map(|obj| serde_json::Value::Object(obj.clone()));
        
        match call_erebus_user_registration_api(wallet_address, wallet_chain, source_type).await {
            Ok(user_id) => {
                info!("✅ User registered successfully via Erebus API: {}", user_id);

                // Note: User sync to Agents database is now handled automatically by Erebus
                // via Kafka events and database triggers - no manual sync needed

                let response = serde_json::json!({
                    "success": true,
                    "data": {
                        "user_id": user_id,
                        "wallet_address": wallet_address,
                        "chain": wallet_chain
                    },
                    "message": "User registered successfully via Erebus API"
                });
                Ok(ResponseJson(response))
            }
            Err(e) => {
                error!("❌ User registration failed: {}", e);
                let error_response = AgentErrorResponse {
                    error: "user_registration_failed".to_string(),
                    code: "USER_REGISTRATION_ERROR".to_string(),
                    message: format!("Failed to register user via Erebus API: {}", e),
                    agent_available: true,
                };
                Err((StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error_response)))
            }
        }
    } else {
        error!("❌ No wallet address provided for user registration");
        let error_response = AgentErrorResponse {
            error: "missing_wallet_address".to_string(),
            code: "MISSING_WALLET_ADDRESS".to_string(),
            message: "Wallet address is required for user registration".to_string(),
            agent_available: true,
        };
        Err((StatusCode::BAD_REQUEST, ResponseJson(error_response)))
    }
}

/// Health check for agent routing subsystem
pub async fn agent_health() -> ResponseJson<Value> {
    info!("🏥 Agent routing health check requested");
    
    let hecate_proxy = get_hecate_proxy();
    let hecate_healthy = hecate_proxy.health_check().await;
    
    let health_data = serde_json::json!({
        "status": if hecate_healthy { "healthy" } else { "degraded" },
        "service": "erebus_agent_routing",
        "version": "0.1.0",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "agents": {
            "hecate": {
                "status": if hecate_healthy { "healthy" } else { "unavailable" },
                "url": hecate_proxy.agent_base_url()
            }
        }
    });
    
    info!("📊 Agent health response: {}", serde_json::to_string_pretty(&health_data).unwrap_or_default());
    ResponseJson(health_data)
}

/// Proxy chat request to Hecate agent
pub async fn hecate_chat(Json(request): Json<AgentRequest>) -> Result<ResponseJson<AgentResponse>, (StatusCode, ResponseJson<AgentErrorResponse>)> {
    info!("💬 Hecate chat request received");
    info!("📝 Request payload: {}", serde_json::to_string_pretty(&request).unwrap_or_default());
    
    let proxy = get_hecate_proxy();
    
    match proxy.proxy_chat(request).await {
        Ok(response) => {
            info!("✅ Hecate chat response successful");
            info!("📤 Response payload: {}", serde_json::to_string_pretty(&response).unwrap_or_default());
            Ok(ResponseJson(response))
        }
        Err(error) => {
            error!("❌ Hecate chat request failed");
            error!("📤 Error response: {}", serde_json::to_string_pretty(&error).unwrap_or_default());
            
            let status_code = match error.code.as_str() {
                "AGENT_UNAVAILABLE" => StatusCode::SERVICE_UNAVAILABLE,
                "AGENT_HTTP_ERROR" => StatusCode::BAD_GATEWAY,
                "AGENT_PARSE_ERROR" => StatusCode::BAD_GATEWAY,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            
            Err((status_code, ResponseJson(error)))
        }
    }
}

/// Proxy chat request to Marketing agent
pub async fn marketing_chat(Json(request): Json<AgentRequest>) -> Result<ResponseJson<AgentResponse>, (StatusCode, ResponseJson<AgentErrorResponse>)> {
    info!("🎭 Marketing chat request received");
    info!("📝 Request payload: {}", serde_json::to_string_pretty(&request).unwrap_or_default());

    let proxy = get_marketing_proxy();

    match proxy.proxy_marketing_chat(request).await {
        Ok(response) => {
            info!("✅ Marketing chat response successful");
            info!("📤 Response payload: {}", serde_json::to_string_pretty(&response).unwrap_or_default());
            Ok(ResponseJson(response))
        }
        Err(error) => {
            error!("❌ Marketing chat request failed");
            error!("📤 Error response: {}", serde_json::to_string_pretty(&error).unwrap_or_default());

            let status_code = match error.code.as_str() {
                "AGENT_UNAVAILABLE" => StatusCode::SERVICE_UNAVAILABLE,
                "AGENT_HTTP_ERROR" => StatusCode::BAD_GATEWAY,
                "AGENT_PARSE_ERROR" => StatusCode::BAD_GATEWAY,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };

            Err((status_code, ResponseJson(error)))
        }
    }
}

/// Get Hecate agent status
pub async fn hecate_status() -> Result<ResponseJson<AgentStatus>, (StatusCode, ResponseJson<AgentErrorResponse>)> {
    info!("📊 Hecate status request received");
    
    let proxy = get_hecate_proxy();
    
    match proxy.get_agent_status().await {
        Ok(status) => {
            info!("✅ Hecate status retrieved successfully");
            info!("📤 Status payload: {}", serde_json::to_string_pretty(&status).unwrap_or_default());
            Ok(ResponseJson(status))
        }
        Err(error) => {
            error!("❌ Hecate status request failed");
            error!("📤 Error response: {}", serde_json::to_string_pretty(&error).unwrap_or_default());
            
            let status_code = match error.code.as_str() {
                "STATUS_UNAVAILABLE" => StatusCode::SERVICE_UNAVAILABLE,
                "STATUS_HTTP_ERROR" => StatusCode::BAD_GATEWAY,
                "STATUS_PARSE_ERROR" => StatusCode::BAD_GATEWAY,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            
            Err((status_code, ResponseJson(error)))
        }
    }
}

/// Generic agent proxy for future agents
pub async fn agent_chat(
    Path(agent_name): Path<String>,
    Json(request): Json<AgentRequest>
) -> Result<ResponseJson<AgentResponse>, (StatusCode, ResponseJson<AgentErrorResponse>)> {
    info!("🤖 Generic agent chat request for: {}", agent_name);
    info!("📝 Request payload: {}", serde_json::to_string_pretty(&request).unwrap_or_default());
    
    match agent_name.as_str() {
        "hecate" => hecate_chat(Json(request)).await,
        "marketing" => marketing_chat(Json(request)).await,
        _ => {
            let error = AgentErrorResponse {
                error: "agent_not_found".to_string(),
                code: "AGENT_NOT_SUPPORTED".to_string(),
                message: format!("Agent '{}' is not supported", agent_name),
                agent_available: false,
            };
            
            warn!("⚠️ Unsupported agent requested: {}", agent_name);
            error!("📤 Error response: {}", serde_json::to_string_pretty(&error).unwrap_or_default());
            
            Err((StatusCode::NOT_FOUND, ResponseJson(error)))
        }
    }
}

/// Generic agent status for future agents
pub async fn agent_status(
    Path(agent_name): Path<String>
) -> Result<ResponseJson<AgentStatus>, (StatusCode, ResponseJson<AgentErrorResponse>)> {
    info!("📊 Generic agent status request for: {}", agent_name);
    
    match agent_name.as_str() {
        "hecate" => hecate_status().await,
        _ => {
            let error = AgentErrorResponse {
                error: "agent_not_found".to_string(),
                code: "AGENT_NOT_SUPPORTED".to_string(),
                message: format!("Agent '{}' is not supported", agent_name),
                agent_available: false,
            };
            
            warn!("⚠️ Unsupported agent status requested: {}", agent_name);
            error!("📤 Error response: {}", serde_json::to_string_pretty(&error).unwrap_or_default());
            
            Err((StatusCode::NOT_FOUND, ResponseJson(error)))
        }
    }
}

/// Set Hecate agent personality
pub async fn hecate_personality(Json(request): Json<Value>) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<AgentErrorResponse>)> {
    info!("⚙️ Hecate personality request received");
    info!("📝 Request payload: {}", serde_json::to_string_pretty(&request).unwrap_or_default());
    
    let proxy = get_hecate_proxy();
    
    match proxy.proxy_request("personality", "POST", Some(request)).await {
        Ok(response) => {
            info!("✅ Hecate personality set successfully");
            info!("📤 Response payload: {}", serde_json::to_string_pretty(&response).unwrap_or_default());
            Ok(ResponseJson(response))
        }
        Err(error) => {
            error!("❌ Hecate personality request failed");
            error!("📤 Error response: {}", serde_json::to_string_pretty(&error).unwrap_or_default());
            
            let status_code = match error.code.as_str() {
                "AGENT_UNAVAILABLE" => StatusCode::SERVICE_UNAVAILABLE,
                "AGENT_HTTP_ERROR" => StatusCode::BAD_GATEWAY,
                "AGENT_PARSE_ERROR" => StatusCode::BAD_GATEWAY,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            
            Err((status_code, ResponseJson(error)))
        }
    }
}

/// Clear Hecate conversation history
pub async fn hecate_clear() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<AgentErrorResponse>)> {
    info!("🧹 Hecate clear conversation request received");
    
    let proxy = get_hecate_proxy();
    
    match proxy.proxy_request("clear", "POST", None).await {
        Ok(response) => {
            info!("✅ Hecate conversation cleared successfully");
            info!("📤 Response payload: {}", serde_json::to_string_pretty(&response).unwrap_or_default());
            Ok(ResponseJson(response))
        }
        Err(error) => {
            error!("❌ Hecate clear conversation request failed");
            error!("📤 Error response: {}", serde_json::to_string_pretty(&error).unwrap_or_default());
            
            let status_code = match error.code.as_str() {
                "AGENT_UNAVAILABLE" => StatusCode::SERVICE_UNAVAILABLE,
                "AGENT_HTTP_ERROR" => StatusCode::BAD_GATEWAY,
                "AGENT_PARSE_ERROR" => StatusCode::BAD_GATEWAY,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            
            Err((status_code, ResponseJson(error)))
        }
    }
}

/// Get Hecate conversation history
pub async fn hecate_history() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<AgentErrorResponse>)> {
    info!("📜 Hecate history request received");
    
    let proxy = get_hecate_proxy();
    
    match proxy.proxy_request("history", "GET", None).await {
        Ok(response) => {
            info!("✅ Hecate history retrieved successfully");
            info!("📤 Response payload: {}", serde_json::to_string_pretty(&response).unwrap_or_default());
            Ok(ResponseJson(response))
        }
        Err(error) => {
            error!("❌ Hecate history request failed");
            error!("📤 Error response: {}", serde_json::to_string_pretty(&error).unwrap_or_default());
            
            let status_code = match error.code.as_str() {
                "AGENT_UNAVAILABLE" => StatusCode::SERVICE_UNAVAILABLE,
                "AGENT_HTTP_ERROR" => StatusCode::BAD_GATEWAY,
                "AGENT_PARSE_ERROR" => StatusCode::BAD_GATEWAY,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            
            Err((status_code, ResponseJson(error)))
        }
    }
}

/// Get available models from Hecate agent
pub async fn hecate_available_models() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<AgentErrorResponse>)> {
    info!("🧠 Hecate available models request received");
    
    let proxy = get_hecate_proxy();
    
    match proxy.proxy_request("available-models", "GET", None).await {
        Ok(response) => {
            info!("✅ Hecate available models retrieved successfully");
            info!("📤 Response payload: {}", serde_json::to_string_pretty(&response).unwrap_or_default());
            Ok(ResponseJson(response))
        }
        Err(error) => {
            error!("❌ Hecate available models request failed");
            error!("📤 Error response: {}", serde_json::to_string_pretty(&error).unwrap_or_default());
            
            let status_code = match error.code.as_str() {
                "AGENT_UNAVAILABLE" => StatusCode::SERVICE_UNAVAILABLE,
                "AGENT_HTTP_ERROR" => StatusCode::BAD_GATEWAY,
                "AGENT_PARSE_ERROR" => StatusCode::BAD_GATEWAY,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            
            Err((status_code, ResponseJson(error)))
        }
    }
}

/// Set Hecate model selection
pub async fn hecate_set_model(Json(request): Json<Value>) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<AgentErrorResponse>)> {
    info!("🎯 Hecate set model request received");
    info!("📝 Request payload: {}", serde_json::to_string_pretty(&request).unwrap_or_default());
    
    let proxy = get_hecate_proxy();
    
    match proxy.proxy_request("set-model", "POST", Some(request)).await {
        Ok(response) => {
            info!("✅ Hecate model set successfully");
            info!("📤 Response payload: {}", serde_json::to_string_pretty(&response).unwrap_or_default());
            Ok(ResponseJson(response))
        }
        Err(error) => {
            error!("❌ Hecate set model request failed");
            error!("📤 Error response: {}", serde_json::to_string_pretty(&error).unwrap_or_default());
            
            let status_code = match error.code.as_str() {
                "AGENT_UNAVAILABLE" => StatusCode::SERVICE_UNAVAILABLE,
                "AGENT_HTTP_ERROR" => StatusCode::BAD_GATEWAY,
                "AGENT_PARSE_ERROR" => StatusCode::BAD_GATEWAY,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            
            Err((status_code, ResponseJson(error)))
        }
    }
}

/// Get detailed model information from Hecate agent
pub async fn hecate_model_info() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<AgentErrorResponse>)> {
    info!("📋 Hecate model info request received");
    
    let proxy = get_hecate_proxy();
    
    match proxy.proxy_request("model-info", "GET", None).await {
        Ok(response) => {
            info!("✅ Hecate model info retrieved successfully");
            info!("📤 Response payload: {}", serde_json::to_string_pretty(&response).unwrap_or_default());
            Ok(ResponseJson(response))
        }
        Err(error) => {
            error!("❌ Hecate model info request failed");
            error!("📤 Error response: {}", serde_json::to_string_pretty(&error).unwrap_or_default());
            
            let status_code = match error.code.as_str() {
                "AGENT_UNAVAILABLE" => StatusCode::SERVICE_UNAVAILABLE,
                "AGENT_HTTP_ERROR" => StatusCode::BAD_GATEWAY,
                "AGENT_PARSE_ERROR" => StatusCode::BAD_GATEWAY,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            
            Err((status_code, ResponseJson(error)))
        }
    }
}

/// Search models via Hecate agent
pub async fn hecate_search_models(Query(params): Query<HashMap<String, String>>) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<AgentErrorResponse>)> {
    info!("🔍 Hecate search models request received");
    info!("📝 Query parameters: {:?}", params);

    let proxy = get_hecate_proxy();

    let query_string = params.iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&");

    let endpoint = if query_string.is_empty() {
        "search-models".to_string()
    } else {
        format!("search-models?{}", query_string)
    };

    match proxy.proxy_request(&endpoint, "GET", None).await {
        Ok(response) => {
            info!("✅ Hecate search models retrieved successfully");
            info!("📤 Response payload: {}", serde_json::to_string_pretty(&response).unwrap_or_default());
            Ok(ResponseJson(response))
        }
        Err(error) => {
            error!("❌ Hecate search models request failed");
            error!("📤 Error response: {}", serde_json::to_string_pretty(&error).unwrap_or_default());

            let status_code = match error.code.as_str() {
                "AGENT_UNAVAILABLE" => StatusCode::SERVICE_UNAVAILABLE,
                "AGENT_HTTP_ERROR" => StatusCode::BAD_GATEWAY,
                "AGENT_PARSE_ERROR" => StatusCode::BAD_GATEWAY,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };

            Err((status_code, ResponseJson(error)))
        }
    }
}

// ================================
// TASK MANAGEMENT ENDPOINTS
// ================================

/// Create a new task (user-initiated or API/MCP-triggered)
pub async fn create_task(
    headers: HeaderMap,
    Json(request): Json<Value>
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<AgentErrorResponse>)> {
    info!("📋 Task creation request received");
    info!("📝 Request payload: {}", serde_json::to_string_pretty(&request).unwrap_or_default());

    // Extract wallet information and create user reference if needed
    let user_id = extract_wallet_and_create_user(&headers).await;
    if let Some(user_id) = user_id {
        info!("👤 Task will be associated with user: {}", user_id);
    } else {
        info!("👤 No wallet information provided, task will be created without user association");
    }

    let proxy = get_hecate_proxy();

    match proxy.proxy_request("tasks", "POST", Some(request)).await {
        Ok(response) => {
            info!("✅ Task created successfully");
            info!("📤 Response payload: {}", serde_json::to_string_pretty(&response).unwrap_or_default());
            Ok(ResponseJson(response))
        }
        Err(error) => {
            error!("❌ Task creation failed");
            error!("📤 Error response: {}", serde_json::to_string_pretty(&error).unwrap_or_default());

            let status_code = match error.code.as_str() {
                "AGENT_UNAVAILABLE" => StatusCode::SERVICE_UNAVAILABLE,
                "AGENT_HTTP_ERROR" => StatusCode::BAD_GATEWAY,
                "AGENT_PARSE_ERROR" => StatusCode::BAD_GATEWAY,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };

            Err((status_code, ResponseJson(error)))
        }
    }
}

/// Get all tasks with optional filtering
pub async fn get_tasks(Query(params): Query<HashMap<String, String>>) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<AgentErrorResponse>)> {
    info!("📋 Get tasks request received");
    info!("📝 Query parameters: {:?}", params);

    let proxy = get_hecate_proxy();

    let query_string = params.iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&");

    let endpoint = if query_string.is_empty() {
        "tasks".to_string()
    } else {
        format!("tasks?{}", query_string)
    };

    match proxy.proxy_request(&endpoint, "GET", None).await {
        Ok(response) => {
            info!("✅ Tasks retrieved successfully");
            Ok(ResponseJson(response))
        }
        Err(error) => {
            error!("❌ Get tasks request failed");
            error!("📤 Error response: {}", serde_json::to_string_pretty(&error).unwrap_or_default());

            let status_code = match error.code.as_str() {
                "AGENT_UNAVAILABLE" => StatusCode::SERVICE_UNAVAILABLE,
                "AGENT_HTTP_ERROR" => StatusCode::BAD_GATEWAY,
                "AGENT_PARSE_ERROR" => StatusCode::BAD_GATEWAY,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };

            Err((status_code, ResponseJson(error)))
        }
    }
}

/// Get a specific task by ID
pub async fn get_task(Path(task_id): Path<String>) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<AgentErrorResponse>)> {
    info!("📋 Get task request received for ID: {}", task_id);

    let proxy = get_hecate_proxy();
    let endpoint = format!("tasks/{}", task_id);

    match proxy.proxy_request(&endpoint, "GET", None).await {
        Ok(response) => {
            info!("✅ Task retrieved successfully");
            Ok(ResponseJson(response))
        }
        Err(error) => {
            error!("❌ Get task request failed");
            error!("📤 Error response: {}", serde_json::to_string_pretty(&error).unwrap_or_default());

            let status_code = match error.code.as_str() {
                "AGENT_UNAVAILABLE" => StatusCode::SERVICE_UNAVAILABLE,
                "AGENT_HTTP_ERROR" => StatusCode::BAD_GATEWAY,
                "AGENT_PARSE_ERROR" => StatusCode::BAD_GATEWAY,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };

            Err((status_code, ResponseJson(error)))
        }
    }
}

/// Update a task
pub async fn update_task(Path(task_id): Path<String>, Json(request): Json<Value>) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<AgentErrorResponse>)> {
    info!("📋 Task update request received for ID: {}", task_id);
    info!("📝 Request payload: {}", serde_json::to_string_pretty(&request).unwrap_or_default());

    let proxy = get_hecate_proxy();
    let endpoint = format!("tasks/{}", task_id);

    match proxy.proxy_request(&endpoint, "PUT", Some(request)).await {
        Ok(response) => {
            info!("✅ Task updated successfully");
            info!("📤 Response payload: {}", serde_json::to_string_pretty(&response).unwrap_or_default());
            Ok(ResponseJson(response))
        }
        Err(error) => {
            error!("❌ Task update failed");
            error!("📤 Error response: {}", serde_json::to_string_pretty(&error).unwrap_or_default());

            let status_code = match error.code.as_str() {
                "AGENT_UNAVAILABLE" => StatusCode::SERVICE_UNAVAILABLE,
                "AGENT_HTTP_ERROR" => StatusCode::BAD_GATEWAY,
                "AGENT_PARSE_ERROR" => StatusCode::BAD_GATEWAY,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };

            Err((status_code, ResponseJson(error)))
        }
    }
}

/// Delete a task
pub async fn delete_task(Path(task_id): Path<String>) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<AgentErrorResponse>)> {
    info!("📋 Task deletion request received for ID: {}", task_id);

    let proxy = get_hecate_proxy();
    let endpoint = format!("tasks/{}", task_id);

    match proxy.proxy_request(&endpoint, "DELETE", None).await {
        Ok(response) => {
            info!("✅ Task deleted successfully");
            Ok(ResponseJson(response))
        }
        Err(error) => {
            error!("❌ Task deletion failed");
            error!("📤 Error response: {}", serde_json::to_string_pretty(&error).unwrap_or_default());

            let status_code = match error.code.as_str() {
                "AGENT_UNAVAILABLE" => StatusCode::SERVICE_UNAVAILABLE,
                "AGENT_HTTP_ERROR" => StatusCode::BAD_GATEWAY,
                "AGENT_PARSE_ERROR" => StatusCode::BAD_GATEWAY,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };

            Err((status_code, ResponseJson(error)))
        }
    }
}

/// Start a task
pub async fn start_task(Path(task_id): Path<String>) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<AgentErrorResponse>)> {
    info!("▶️ Task start request received for ID: {}", task_id);

    let proxy = get_hecate_proxy();
    let endpoint = format!("tasks/{}/start", task_id);

    match proxy.proxy_request(&endpoint, "POST", None).await {
        Ok(response) => {
            info!("✅ Task started successfully");
            Ok(ResponseJson(response))
        }
        Err(error) => {
            error!("❌ Task start failed");
            error!("📤 Error response: {}", serde_json::to_string_pretty(&error).unwrap_or_default());

            let status_code = match error.code.as_str() {
                "AGENT_UNAVAILABLE" => StatusCode::SERVICE_UNAVAILABLE,
                "AGENT_HTTP_ERROR" => StatusCode::BAD_GATEWAY,
                "AGENT_PARSE_ERROR" => StatusCode::BAD_GATEWAY,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };

            Err((status_code, ResponseJson(error)))
        }
    }
}

/// Pause a task
pub async fn pause_task(Path(task_id): Path<String>) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<AgentErrorResponse>)> {
    info!("⏸️ Task pause request received for ID: {}", task_id);

    let proxy = get_hecate_proxy();
    let endpoint = format!("tasks/{}/pause", task_id);

    match proxy.proxy_request(&endpoint, "POST", None).await {
        Ok(response) => {
            info!("✅ Task paused successfully");
            Ok(ResponseJson(response))
        }
        Err(error) => {
            error!("❌ Task pause failed");
            error!("📤 Error response: {}", serde_json::to_string_pretty(&error).unwrap_or_default());

            let status_code = match error.code.as_str() {
                "AGENT_UNAVAILABLE" => StatusCode::SERVICE_UNAVAILABLE,
                "AGENT_HTTP_ERROR" => StatusCode::BAD_GATEWAY,
                "AGENT_PARSE_ERROR" => StatusCode::BAD_GATEWAY,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };

            Err((status_code, ResponseJson(error)))
        }
    }
}

/// Resume a task
pub async fn resume_task(Path(task_id): Path<String>) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<AgentErrorResponse>)> {
    info!("▶️ Task resume request received for ID: {}", task_id);

    let proxy = get_hecate_proxy();
    let endpoint = format!("tasks/{}/resume", task_id);

    match proxy.proxy_request(&endpoint, "POST", None).await {
        Ok(response) => {
            info!("✅ Task resumed successfully");
            Ok(ResponseJson(response))
        }
        Err(error) => {
            error!("❌ Task resume failed");
            error!("📤 Error response: {}", serde_json::to_string_pretty(&error).unwrap_or_default());

            let status_code = match error.code.as_str() {
                "AGENT_UNAVAILABLE" => StatusCode::SERVICE_UNAVAILABLE,
                "AGENT_HTTP_ERROR" => StatusCode::BAD_GATEWAY,
                "AGENT_PARSE_ERROR" => StatusCode::BAD_GATEWAY,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };

            Err((status_code, ResponseJson(error)))
        }
    }
}

/// Cancel a task
pub async fn cancel_task(Path(task_id): Path<String>) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<AgentErrorResponse>)> {
    info!("❌ Task cancel request received for ID: {}", task_id);

    let proxy = get_hecate_proxy();
    let endpoint = format!("tasks/{}/cancel", task_id);

    match proxy.proxy_request(&endpoint, "POST", None).await {
        Ok(response) => {
            info!("✅ Task cancelled successfully");
            Ok(ResponseJson(response))
        }
        Err(error) => {
            error!("❌ Task cancel failed");
            error!("📤 Error response: {}", serde_json::to_string_pretty(&error).unwrap_or_default());

            let status_code = match error.code.as_str() {
                "AGENT_UNAVAILABLE" => StatusCode::SERVICE_UNAVAILABLE,
                "AGENT_HTTP_ERROR" => StatusCode::BAD_GATEWAY,
                "AGENT_PARSE_ERROR" => StatusCode::BAD_GATEWAY,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };

            Err((status_code, ResponseJson(error)))
        }
    }
}

/// Retry a failed task
pub async fn retry_task(Path(task_id): Path<String>) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<AgentErrorResponse>)> {
    info!("🔄 Task retry request received for ID: {}", task_id);

    let proxy = get_hecate_proxy();
    let endpoint = format!("tasks/{}/retry", task_id);

    match proxy.proxy_request(&endpoint, "POST", None).await {
        Ok(response) => {
            info!("✅ Task retry initiated successfully");
            Ok(ResponseJson(response))
        }
        Err(error) => {
            error!("❌ Task retry failed");
            error!("📤 Error response: {}", serde_json::to_string_pretty(&error).unwrap_or_default());

            let status_code = match error.code.as_str() {
                "AGENT_UNAVAILABLE" => StatusCode::SERVICE_UNAVAILABLE,
                "AGENT_HTTP_ERROR" => StatusCode::BAD_GATEWAY,
                "AGENT_PARSE_ERROR" => StatusCode::BAD_GATEWAY,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };

            Err((status_code, ResponseJson(error)))
        }
    }
}

/// Get task queues
pub async fn get_task_queues() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<AgentErrorResponse>)> {
    info!("📋 Get task queues request received");

    let proxy = get_hecate_proxy();

    match proxy.proxy_request("tasks/queues", "GET", None).await {
        Ok(response) => {
            info!("✅ Task queues retrieved successfully");
            Ok(ResponseJson(response))
        }
        Err(error) => {
            error!("❌ Get task queues failed");
            error!("📤 Error response: {}", serde_json::to_string_pretty(&error).unwrap_or_default());

            let status_code = match error.code.as_str() {
                "AGENT_UNAVAILABLE" => StatusCode::SERVICE_UNAVAILABLE,
                "AGENT_HTTP_ERROR" => StatusCode::BAD_GATEWAY,
                "AGENT_PARSE_ERROR" => StatusCode::BAD_GATEWAY,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };

            Err((status_code, ResponseJson(error)))
        }
    }
}

/// Get task templates
pub async fn get_task_templates() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<AgentErrorResponse>)> {
    info!("📋 Get task templates request received");

    let proxy = get_hecate_proxy();

    match proxy.proxy_request("tasks/templates", "GET", None).await {
        Ok(response) => {
            info!("✅ Task templates retrieved successfully");
            Ok(ResponseJson(response))
        }
        Err(error) => {
            error!("❌ Get task templates failed");
            error!("📤 Error response: {}", serde_json::to_string_pretty(&error).unwrap_or_default());

            let status_code = match error.code.as_str() {
                "AGENT_UNAVAILABLE" => StatusCode::SERVICE_UNAVAILABLE,
                "AGENT_HTTP_ERROR" => StatusCode::BAD_GATEWAY,
                "AGENT_PARSE_ERROR" => StatusCode::BAD_GATEWAY,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };

            Err((status_code, ResponseJson(error)))
        }
    }
}

/// Create task from template
pub async fn create_task_from_template(Json(request): Json<Value>) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<AgentErrorResponse>)> {
    info!("📋 Create task from template request received");
    info!("📝 Request payload: {}", serde_json::to_string_pretty(&request).unwrap_or_default());

    let proxy = get_hecate_proxy();

    match proxy.proxy_request("tasks/from-template", "POST", Some(request)).await {
        Ok(response) => {
            info!("✅ Task created from template successfully");
            Ok(ResponseJson(response))
        }
        Err(error) => {
            error!("❌ Create task from template failed");
            error!("📤 Error response: {}", serde_json::to_string_pretty(&error).unwrap_or_default());

            let status_code = match error.code.as_str() {
                "AGENT_UNAVAILABLE" => StatusCode::SERVICE_UNAVAILABLE,
                "AGENT_HTTP_ERROR" => StatusCode::BAD_GATEWAY,
                "AGENT_PARSE_ERROR" => StatusCode::BAD_GATEWAY,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };

            Err((status_code, ResponseJson(error)))
        }
    }
}

/// Get task statistics
pub async fn get_task_stats(Query(params): Query<HashMap<String, String>>) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<AgentErrorResponse>)> {
    info!("📊 Get task stats request received");
    info!("📝 Query parameters: {:?}", params);

    let proxy = get_hecate_proxy();

    let query_string = params.iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&");

    let endpoint = if query_string.is_empty() {
        "tasks/stats".to_string()
    } else {
        format!("tasks/stats?{}", query_string)
    };

    match proxy.proxy_request(&endpoint, "GET", None).await {
        Ok(response) => {
            info!("✅ Task stats retrieved successfully");
            Ok(ResponseJson(response))
        }
        Err(error) => {
            error!("❌ Get task stats failed");
            error!("📤 Error response: {}", serde_json::to_string_pretty(&error).unwrap_or_default());

            let status_code = match error.code.as_str() {
                "AGENT_UNAVAILABLE" => StatusCode::SERVICE_UNAVAILABLE,
                "AGENT_HTTP_ERROR" => StatusCode::BAD_GATEWAY,
                "AGENT_PARSE_ERROR" => StatusCode::BAD_GATEWAY,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };

            Err((status_code, ResponseJson(error)))
        }
    }
}

/// Get task notifications
pub async fn get_task_notifications() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<AgentErrorResponse>)> {
    info!("🔔 Get task notifications request received");

    let proxy = get_hecate_proxy();

    match proxy.proxy_request("tasks/notifications", "GET", None).await {
        Ok(response) => {
            info!("✅ Task notifications retrieved successfully");
            Ok(ResponseJson(response))
        }
        Err(error) => {
            error!("❌ Get task notifications failed");
            error!("📤 Error response: {}", serde_json::to_string_pretty(&error).unwrap_or_default());

            let status_code = match error.code.as_str() {
                "AGENT_UNAVAILABLE" => StatusCode::SERVICE_UNAVAILABLE,
                "AGENT_HTTP_ERROR" => StatusCode::BAD_GATEWAY,
                "AGENT_PARSE_ERROR" => StatusCode::BAD_GATEWAY,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };

            Err((status_code, ResponseJson(error)))
        }
    }
}

/// Mark notification as read
pub async fn mark_notification_read(Path(notification_id): Path<String>) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<AgentErrorResponse>)> {
    info!("🔔 Mark notification read request received for ID: {}", notification_id);

    let proxy = get_hecate_proxy();
    let endpoint = format!("tasks/notifications/{}/read", notification_id);

    match proxy.proxy_request(&endpoint, "POST", None).await {
        Ok(response) => {
            info!("✅ Notification marked as read successfully");
            Ok(ResponseJson(response))
        }
        Err(error) => {
            error!("❌ Mark notification read failed");
            error!("📤 Error response: {}", serde_json::to_string_pretty(&error).unwrap_or_default());

            let status_code = match error.code.as_str() {
                "AGENT_UNAVAILABLE" => StatusCode::SERVICE_UNAVAILABLE,
                "AGENT_HTTP_ERROR" => StatusCode::BAD_GATEWAY,
                "AGENT_PARSE_ERROR" => StatusCode::BAD_GATEWAY,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };

            Err((status_code, ResponseJson(error)))
        }
    }
}

/// Handle notification action
pub async fn handle_notification_action(Path(notification_id): Path<String>, Json(request): Json<Value>) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<AgentErrorResponse>)> {
    info!("🔔 Handle notification action request received for ID: {}", notification_id);
    info!("📝 Request payload: {}", serde_json::to_string_pretty(&request).unwrap_or_default());

    let proxy = get_hecate_proxy();
    let endpoint = format!("tasks/notifications/{}/action", notification_id);

    match proxy.proxy_request(&endpoint, "POST", Some(request)).await {
        Ok(response) => {
            info!("✅ Notification action handled successfully");
            Ok(ResponseJson(response))
        }
        Err(error) => {
            error!("❌ Handle notification action failed");
            error!("📤 Error response: {}", serde_json::to_string_pretty(&error).unwrap_or_default());

            let status_code = match error.code.as_str() {
                "AGENT_UNAVAILABLE" => StatusCode::SERVICE_UNAVAILABLE,
                "AGENT_HTTP_ERROR" => StatusCode::BAD_GATEWAY,
                "AGENT_PARSE_ERROR" => StatusCode::BAD_GATEWAY,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };

            Err((status_code, ResponseJson(error)))
        }
    }
}

/// Get task events
pub async fn get_task_events(Query(params): Query<HashMap<String, String>>) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<AgentErrorResponse>)> {
    info!("⚡ Get task events request received");
    info!("📝 Query parameters: {:?}", params);

    let proxy = get_hecate_proxy();

    let query_string = params.iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&");

    let endpoint = if query_string.is_empty() {
        "tasks/events".to_string()
    } else {
        format!("tasks/events?{}", query_string)
    };

    match proxy.proxy_request(&endpoint, "GET", None).await {
        Ok(response) => {
            info!("✅ Task events retrieved successfully");
            Ok(ResponseJson(response))
        }
        Err(error) => {
            error!("❌ Get task events failed");
            error!("📤 Error response: {}", serde_json::to_string_pretty(&error).unwrap_or_default());

            let status_code = match error.code.as_str() {
                "AGENT_UNAVAILABLE" => StatusCode::SERVICE_UNAVAILABLE,
                "AGENT_HTTP_ERROR" => StatusCode::BAD_GATEWAY,
                "AGENT_PARSE_ERROR" => StatusCode::BAD_GATEWAY,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };

            Err((status_code, ResponseJson(error)))
        }
    }
}

/// Publish task event (for automation/MCP hooks)
pub async fn publish_task_event(Json(request): Json<Value>) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<AgentErrorResponse>)> {
    info!("⚡ Publish task event request received");
    info!("📝 Request payload: {}", serde_json::to_string_pretty(&request).unwrap_or_default());

    let proxy = get_hecate_proxy();

    match proxy.proxy_request("tasks/events", "POST", Some(request)).await {
        Ok(response) => {
            info!("✅ Task event published successfully");
            Ok(ResponseJson(response))
        }
        Err(error) => {
            error!("❌ Publish task event failed");
            error!("📤 Error response: {}", serde_json::to_string_pretty(&error).unwrap_or_default());

            let status_code = match error.code.as_str() {
                "AGENT_UNAVAILABLE" => StatusCode::SERVICE_UNAVAILABLE,
                "AGENT_HTTP_ERROR" => StatusCode::BAD_GATEWAY,
                "AGENT_PARSE_ERROR" => StatusCode::BAD_GATEWAY,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };

            Err((status_code, ResponseJson(error)))
        }
    }
}

/// Get Hecate motivation state
pub async fn get_motivation_state() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<AgentErrorResponse>)> {
    info!("🧠 Get motivation state request received");

    let proxy = get_hecate_proxy();

    match proxy.proxy_request("tasks/motivation", "GET", None).await {
        Ok(response) => {
            info!("✅ Motivation state retrieved successfully");
            Ok(ResponseJson(response))
        }
        Err(error) => {
            error!("❌ Get motivation state failed");
            error!("📤 Error response: {}", serde_json::to_string_pretty(&error).unwrap_or_default());

            let status_code = match error.code.as_str() {
                "AGENT_UNAVAILABLE" => StatusCode::SERVICE_UNAVAILABLE,
                "AGENT_HTTP_ERROR" => StatusCode::BAD_GATEWAY,
                "AGENT_PARSE_ERROR" => StatusCode::BAD_GATEWAY,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };

            Err((status_code, ResponseJson(error)))
        }
    }
}

/// Update Hecate motivation state
pub async fn update_motivation_state(Json(request): Json<Value>) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<AgentErrorResponse>)> {
    info!("🧠 Update motivation state request received");
    info!("📝 Request payload: {}", serde_json::to_string_pretty(&request).unwrap_or_default());

    let proxy = get_hecate_proxy();

    match proxy.proxy_request("tasks/motivation", "PUT", Some(request)).await {
        Ok(response) => {
            info!("✅ Motivation state updated successfully");
            Ok(ResponseJson(response))
        }
        Err(error) => {
            error!("❌ Update motivation state failed");
            error!("📤 Error response: {}", serde_json::to_string_pretty(&error).unwrap_or_default());

            let status_code = match error.code.as_str() {
                "AGENT_UNAVAILABLE" => StatusCode::SERVICE_UNAVAILABLE,
                "AGENT_HTTP_ERROR" => StatusCode::BAD_GATEWAY,
                "AGENT_PARSE_ERROR" => StatusCode::BAD_GATEWAY,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };

            Err((status_code, ResponseJson(error)))
        }
    }
}

/// Get task suggestions based on context
pub async fn get_task_suggestions(Json(request): Json<Value>) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<AgentErrorResponse>)> {
    info!("💡 Get task suggestions request received");
    info!("📝 Request payload: {}", serde_json::to_string_pretty(&request).unwrap_or_default());

    let proxy = get_hecate_proxy();

    match proxy.proxy_request("tasks/suggestions", "POST", Some(request)).await {
        Ok(response) => {
            info!("✅ Task suggestions retrieved successfully");
            Ok(ResponseJson(response))
        }
        Err(error) => {
            error!("❌ Get task suggestions failed");
            error!("📤 Error response: {}", serde_json::to_string_pretty(&error).unwrap_or_default());

            let status_code = match error.code.as_str() {
                "AGENT_UNAVAILABLE" => StatusCode::SERVICE_UNAVAILABLE,
                "AGENT_HTTP_ERROR" => StatusCode::BAD_GATEWAY,
                "AGENT_PARSE_ERROR" => StatusCode::BAD_GATEWAY,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };

            Err((status_code, ResponseJson(error)))
        }
    }
}

/// Learn from task outcome
pub async fn learn_from_task(Path(task_id): Path<String>, Json(request): Json<Value>) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<AgentErrorResponse>)> {
    info!("🎓 Learn from task request received for ID: {}", task_id);
    info!("📝 Request payload: {}", serde_json::to_string_pretty(&request).unwrap_or_default());

    let proxy = get_hecate_proxy();
    let endpoint = format!("tasks/{}/learn", task_id);

    match proxy.proxy_request(&endpoint, "POST", Some(request)).await {
        Ok(response) => {
            info!("✅ Task learning completed successfully");
            Ok(ResponseJson(response))
        }
        Err(error) => {
            error!("❌ Learn from task failed");
            error!("📤 Error response: {}", serde_json::to_string_pretty(&error).unwrap_or_default());

            let status_code = match error.code.as_str() {
                "AGENT_UNAVAILABLE" => StatusCode::SERVICE_UNAVAILABLE,
                "AGENT_HTTP_ERROR" => StatusCode::BAD_GATEWAY,
                "AGENT_PARSE_ERROR" => StatusCode::BAD_GATEWAY,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };

            Err((status_code, ResponseJson(error)))
        }
    }
}

/// Process task with Hecate agent
pub async fn process_task(Path(task_id): Path<String>) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<AgentErrorResponse>)> {
    info!("⚡ Process task request received for ID: {}", task_id);

    let proxy = get_hecate_proxy();
    let endpoint = format!("tasks/{}/process", task_id);

    match proxy.proxy_request(&endpoint, "POST", None).await {
        Ok(response) => {
            info!("✅ Task processed successfully");
            info!("📤 Response payload: {}", serde_json::to_string_pretty(&response).unwrap_or_default());
            Ok(ResponseJson(response))
        }
        Err(error) => {
            error!("❌ Task processing failed");
            error!("📤 Error response: {}", serde_json::to_string_pretty(&error).unwrap_or_default());

            let status_code = match error.code.as_str() {
                "AGENT_UNAVAILABLE" => StatusCode::SERVICE_UNAVAILABLE,
                "AGENT_HTTP_ERROR" => StatusCode::BAD_GATEWAY,
                "AGENT_PARSE_ERROR" => StatusCode::BAD_GATEWAY,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };

            Err((status_code, ResponseJson(error)))
        }
    }
}