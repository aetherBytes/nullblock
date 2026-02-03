use super::models::{
    AgentApiKeyResponse, ApiKeyListResponse, ApiKeyResponse, ApiKeySingleResponse,
    CreateAgentApiKeyRequest, CreateApiKeyRequest, DecryptedAgentApiKeyResponse,
    DecryptedApiKeysResponse, RateLimitResponse,
};
use super::service::ApiKeyService;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

pub fn create_api_key_routes<S>(service: Arc<ApiKeyService>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        // User API key management
        .route("/api/users/:user_id/api-keys", post(create_api_key))
        .route("/api/users/:user_id/api-keys", get(list_api_keys))
        .route(
            "/api/users/:user_id/api-keys/:key_id",
            delete(revoke_api_key),
        )
        .route(
            "/internal/users/:user_id/api-keys/decrypted",
            get(get_decrypted_keys),
        )
        // Agent API key management (internal only)
        .route(
            "/internal/agents/:agent_name/api-keys",
            post(create_agent_api_key),
        )
        .route(
            "/internal/agents/:agent_name/api-keys/:provider/decrypted",
            get(get_decrypted_agent_key),
        )
        // Rate limiting (internal only)
        .route(
            "/internal/users/:user_id/rate-limit/:agent_name",
            get(check_rate_limit),
        )
        .route(
            "/internal/users/:user_id/rate-limit/:agent_name/increment",
            post(increment_rate_limit),
        )
        // Check if user has their own API key
        .route(
            "/internal/users/:user_id/has-api-key/:provider",
            get(user_has_api_key),
        )
        .with_state(service)
}

async fn create_api_key(
    State(service): State<Arc<ApiKeyService>>,
    Path(user_id): Path<String>,
    Json(request): Json<CreateApiKeyRequest>,
) -> impl IntoResponse {
    let user_uuid = match Uuid::parse_str(&user_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiKeySingleResponse {
                    success: false,
                    data: None,
                    error: Some("Invalid user ID format".to_string()),
                    timestamp: Utc::now().to_rfc3339(),
                }),
            )
                .into_response();
        }
    };

    match service.create_or_update_api_key(user_uuid, request).await {
        Ok(api_key) => (
            StatusCode::CREATED,
            Json(ApiKeySingleResponse {
                success: true,
                data: Some(api_key.into()),
                error: None,
                timestamp: Utc::now().to_rfc3339(),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiKeySingleResponse {
                success: false,
                data: None,
                error: Some(e),
                timestamp: Utc::now().to_rfc3339(),
            }),
        )
            .into_response(),
    }
}

async fn list_api_keys(
    State(service): State<Arc<ApiKeyService>>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    let user_uuid = match Uuid::parse_str(&user_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiKeyListResponse {
                    success: false,
                    data: None,
                    total: 0,
                    error: Some("Invalid user ID format".to_string()),
                    timestamp: Utc::now().to_rfc3339(),
                }),
            )
                .into_response();
        }
    };

    match service.list_api_keys(user_uuid).await {
        Ok(keys) => {
            let total = keys.len();
            let response_keys: Vec<ApiKeyResponse> = keys.into_iter().map(|k| k.into()).collect();

            (
                StatusCode::OK,
                Json(ApiKeyListResponse {
                    success: true,
                    data: Some(response_keys),
                    total,
                    error: None,
                    timestamp: Utc::now().to_rfc3339(),
                }),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiKeyListResponse {
                success: false,
                data: None,
                total: 0,
                error: Some(e),
                timestamp: Utc::now().to_rfc3339(),
            }),
        )
            .into_response(),
    }
}

async fn revoke_api_key(
    State(service): State<Arc<ApiKeyService>>,
    Path((user_id, key_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let user_uuid = match Uuid::parse_str(&user_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiKeySingleResponse {
                    success: false,
                    data: None,
                    error: Some("Invalid user ID format".to_string()),
                    timestamp: Utc::now().to_rfc3339(),
                }),
            )
                .into_response();
        }
    };

    let key_uuid = match Uuid::parse_str(&key_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiKeySingleResponse {
                    success: false,
                    data: None,
                    error: Some("Invalid key ID format".to_string()),
                    timestamp: Utc::now().to_rfc3339(),
                }),
            )
                .into_response();
        }
    };

    match service.revoke_api_key(user_uuid, key_uuid).await {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiKeySingleResponse {
                success: true,
                data: None,
                error: None,
                timestamp: Utc::now().to_rfc3339(),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiKeySingleResponse {
                success: false,
                data: None,
                error: Some(e),
                timestamp: Utc::now().to_rfc3339(),
            }),
        )
            .into_response(),
    }
}

async fn get_decrypted_keys(
    State(service): State<Arc<ApiKeyService>>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    let user_uuid = match Uuid::parse_str(&user_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(DecryptedApiKeysResponse {
                    success: false,
                    data: None,
                    error: Some("Invalid user ID format".to_string()),
                    timestamp: Utc::now().to_rfc3339(),
                }),
            )
                .into_response();
        }
    };

    match service.get_decrypted_keys(user_uuid).await {
        Ok(keys) => (
            StatusCode::OK,
            Json(DecryptedApiKeysResponse {
                success: true,
                data: Some(keys),
                error: None,
                timestamp: Utc::now().to_rfc3339(),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(DecryptedApiKeysResponse {
                success: false,
                data: None,
                error: Some(e),
                timestamp: Utc::now().to_rfc3339(),
            }),
        )
            .into_response(),
    }
}

// ==================== Agent API Key Handlers ====================

async fn create_agent_api_key(
    State(service): State<Arc<ApiKeyService>>,
    Path(agent_name): Path<String>,
    Json(mut request): Json<CreateAgentApiKeyRequest>,
) -> impl IntoResponse {
    request.agent_name = agent_name;

    match service.create_or_update_agent_api_key(request).await {
        Ok(key) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "success": true,
                "data": AgentApiKeyResponse::from(key),
                "timestamp": Utc::now().to_rfc3339()
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": e,
                "timestamp": Utc::now().to_rfc3339()
            })),
        )
            .into_response(),
    }
}

async fn get_decrypted_agent_key(
    State(service): State<Arc<ApiKeyService>>,
    Path((agent_name, provider)): Path<(String, String)>,
) -> impl IntoResponse {
    match service
        .get_decrypted_agent_key(&agent_name, &provider)
        .await
    {
        Ok(Some(api_key)) => (
            StatusCode::OK,
            Json(DecryptedAgentApiKeyResponse {
                success: true,
                agent_name,
                provider,
                api_key: Some(api_key),
                error: None,
                timestamp: Utc::now().to_rfc3339(),
            }),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(DecryptedAgentApiKeyResponse {
                success: false,
                agent_name,
                provider,
                api_key: None,
                error: Some("No API key found for this agent/provider".to_string()),
                timestamp: Utc::now().to_rfc3339(),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(DecryptedAgentApiKeyResponse {
                success: false,
                agent_name,
                provider,
                api_key: None,
                error: Some(e),
                timestamp: Utc::now().to_rfc3339(),
            }),
        )
            .into_response(),
    }
}

// ==================== Rate Limit Handlers ====================

async fn check_rate_limit(
    State(service): State<Arc<ApiKeyService>>,
    Path((user_id, agent_name)): Path<(String, String)>,
) -> impl IntoResponse {
    let user_uuid = match Uuid::parse_str(&user_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(RateLimitResponse {
                    success: false,
                    data: None,
                    error: Some("Invalid user ID format".to_string()),
                    timestamp: Utc::now().to_rfc3339(),
                }),
            )
                .into_response();
        }
    };

    match service.check_rate_limit(user_uuid, &agent_name).await {
        Ok(status) => (
            StatusCode::OK,
            Json(RateLimitResponse {
                success: true,
                data: Some(status),
                error: None,
                timestamp: Utc::now().to_rfc3339(),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(RateLimitResponse {
                success: false,
                data: None,
                error: Some(e),
                timestamp: Utc::now().to_rfc3339(),
            }),
        )
            .into_response(),
    }
}

async fn increment_rate_limit(
    State(service): State<Arc<ApiKeyService>>,
    Path((user_id, agent_name)): Path<(String, String)>,
) -> impl IntoResponse {
    let user_uuid = match Uuid::parse_str(&user_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(RateLimitResponse {
                    success: false,
                    data: None,
                    error: Some("Invalid user ID format".to_string()),
                    timestamp: Utc::now().to_rfc3339(),
                }),
            )
                .into_response();
        }
    };

    match service.increment_rate_limit(user_uuid, &agent_name).await {
        Ok(status) => (
            StatusCode::OK,
            Json(RateLimitResponse {
                success: true,
                data: Some(status),
                error: None,
                timestamp: Utc::now().to_rfc3339(),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(RateLimitResponse {
                success: false,
                data: None,
                error: Some(e),
                timestamp: Utc::now().to_rfc3339(),
            }),
        )
            .into_response(),
    }
}

async fn user_has_api_key(
    State(service): State<Arc<ApiKeyService>>,
    Path((user_id, provider)): Path<(String, String)>,
) -> impl IntoResponse {
    let user_uuid = match Uuid::parse_str(&user_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "success": false,
                    "has_key": false,
                    "error": "Invalid user ID format",
                    "timestamp": Utc::now().to_rfc3339()
                })),
            )
                .into_response();
        }
    };

    match service.user_has_api_key(user_uuid, &provider).await {
        Ok(has_key) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "has_key": has_key,
                "provider": provider,
                "timestamp": Utc::now().to_rfc3339()
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "has_key": false,
                "error": e,
                "timestamp": Utc::now().to_rfc3339()
            })),
        )
            .into_response(),
    }
}
