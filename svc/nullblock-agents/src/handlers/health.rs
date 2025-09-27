use crate::{models::HealthResponse, server::AppState};
use axum::{extract::State, Json};
use chrono::Utc;
use std::collections::HashMap;
use tracing::{info, warn};

pub async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    let mut components = HashMap::new();
    let mut overall_status = "healthy";

    // Check LLM service availability
    let hecate_agent = state.hecate_agent.read().await;

    if let Some(llm_factory) = &hecate_agent.llm_factory {
        let factory = llm_factory.read().await;
        match factory.health_check().await {
            Ok(health_info) => {
                let models_available = health_info["models_available"].as_u64().unwrap_or(0);
                let llm_status = health_info["overall_status"].as_str().unwrap_or("unknown");

                components.insert("llm_service".to_string(), health_info.clone());

                if models_available == 0 || llm_status == "unhealthy" {
                    overall_status = "unhealthy";
                    warn!("🚫 Hecate agent unhealthy: No working LLM models available");
                } else {
                    info!("✅ Hecate agent healthy: {} models available", models_available);
                }
            }
            Err(e) => {
                overall_status = "unhealthy";
                components.insert("llm_service".to_string(), serde_json::json!({
                    "status": "error",
                    "error": e.to_string()
                }));
                warn!("🚫 LLM service health check failed: {}", e);
            }
        }
    } else {
        overall_status = "unhealthy";
        components.insert("llm_service".to_string(), serde_json::json!({
            "status": "not_initialized",
            "message": "LLM service factory not initialized"
        }));
        warn!("🚫 Hecate agent unhealthy: LLM service not initialized");
    }

    // Add agent status
    components.insert("agent_running".to_string(), serde_json::json!(hecate_agent.running));

    drop(hecate_agent);

    Json(HealthResponse {
        status: overall_status.to_string(),
        service: "nullblock-agents".to_string(),
        version: "0.1.0".to_string(),
        timestamp: Utc::now().to_rfc3339(),
        components: Some(components),
    })
}