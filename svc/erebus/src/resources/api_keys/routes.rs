use super::models::{
    ApiKeyListResponse, ApiKeyResponse, ApiKeySingleResponse, CreateApiKeyRequest,
    DecryptedApiKeysResponse,
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
        .route("/api/users/:user_id/api-keys", post(create_api_key))
        .route("/api/users/:user_id/api-keys", get(list_api_keys))
        .route("/api/users/:user_id/api-keys/:key_id", delete(revoke_api_key))
        .route(
            "/internal/users/:user_id/api-keys/decrypted",
            get(get_decrypted_keys),
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
