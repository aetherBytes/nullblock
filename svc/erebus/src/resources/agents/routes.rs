// Agent routing endpoints for Erebus
use axum::{
    extract::{Path, Json, Query},
    response::Json as ResponseJson,
    http::StatusCode,
};
use std::collections::HashMap;
use serde_json::Value;
use tracing::{info, error, warn};

use super::proxy::{AgentProxy, AgentRequest, AgentResponse, AgentStatus, AgentErrorResponse};

/// Hecate agent proxy instance - now points to Rust service
fn get_hecate_proxy() -> AgentProxy {
    let hecate_url = std::env::var("HECATE_AGENT_URL")
        .unwrap_or_else(|_| "http://localhost:9003".to_string());
    AgentProxy::new(hecate_url)
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
pub async fn create_task(Json(request): Json<Value>) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<AgentErrorResponse>)> {
    info!("📋 Task creation request received");
    info!("📝 Request payload: {}", serde_json::to_string_pretty(&request).unwrap_or_default());

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