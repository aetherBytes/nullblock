// Agent routing endpoints for Erebus
use axum::{
    extract::{Path, Json},
    response::Json as ResponseJson,
    http::StatusCode,
};
use serde_json::Value;
use tracing::{info, error, warn};

use super::proxy::{AgentProxy, AgentRequest, AgentResponse, AgentStatus, AgentErrorResponse};

/// Hecate agent proxy instance
fn get_hecate_proxy() -> AgentProxy {
    let hecate_url = std::env::var("HECATE_AGENT_URL")
        .unwrap_or_else(|_| "http://localhost:9002".to_string());
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