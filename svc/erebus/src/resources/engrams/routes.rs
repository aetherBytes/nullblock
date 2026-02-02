use axum::{
    extract::{Json, Path, Query},
    http::StatusCode,
    response::Json as ResponseJson,
    routing::{delete, get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{error, info};

fn get_engrams_service_url() -> String {
    std::env::var("ENGRAMS_SERVICE_URL").unwrap_or_else(|_| "http://localhost:9004".to_string())
}

#[derive(Debug, Serialize)]
pub struct EngramErrorResponse {
    pub error: String,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

async fn proxy_request(
    method: &str,
    endpoint: &str,
    body: Option<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<EngramErrorResponse>)> {
    let client = reqwest::Client::new();
    let base_url = get_engrams_service_url();
    let url = format!("{}/{}", base_url, endpoint);

    info!("🔗 Proxying {} request to Engram service: {}", method, url);

    let request_builder = match method {
        "GET" => client.get(&url),
        "POST" => client.post(&url),
        "PUT" => client.put(&url),
        "DELETE" => client.delete(&url),
        _ => {
            return Err((
                StatusCode::METHOD_NOT_ALLOWED,
                ResponseJson(EngramErrorResponse {
                    error: "invalid_method".to_string(),
                    code: "INVALID_METHOD".to_string(),
                    message: format!("Method {} not supported", method),
                }),
            ));
        }
    };

    let request_builder = if let Some(body) = body {
        request_builder.json(&body)
    } else {
        request_builder
    };

    match request_builder
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            match response.json::<Value>().await {
                Ok(json_response) => {
                    if status.is_success() {
                        info!("✅ Engram service response successful");
                        Ok(ResponseJson(json_response))
                    } else {
                        error!("❌ Engram service returned error status: {}", status);
                        Err((
                            StatusCode::from_u16(status.as_u16())
                                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                            ResponseJson(EngramErrorResponse {
                                error: "engram_service_error".to_string(),
                                code: "ENGRAM_SERVICE_ERROR".to_string(),
                                message: json_response.to_string(),
                            }),
                        ))
                    }
                }
                Err(e) => {
                    error!("❌ Failed to parse Engram service response: {}", e);
                    Err((
                        StatusCode::BAD_GATEWAY,
                        ResponseJson(EngramErrorResponse {
                            error: "parse_error".to_string(),
                            code: "ENGRAM_PARSE_ERROR".to_string(),
                            message: format!("Failed to parse response: {}", e),
                        }),
                    ))
                }
            }
        }
        Err(e) => {
            error!("❌ Failed to connect to Engram service: {}", e);
            Err((
                StatusCode::SERVICE_UNAVAILABLE,
                ResponseJson(EngramErrorResponse {
                    error: "connection_error".to_string(),
                    code: "ENGRAM_UNAVAILABLE".to_string(),
                    message: format!("Failed to connect to Engram service: {}", e),
                }),
            ))
        }
    }
}

pub async fn engram_health(
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<EngramErrorResponse>)> {
    info!("🏥 Engram service health check requested");
    proxy_request("GET", "health", None).await
}

pub async fn create_engram(
    Json(request): Json<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<EngramErrorResponse>)> {
    info!("📝 Create engram request received");
    info!(
        "📋 Request payload: {}",
        serde_json::to_string_pretty(&request).unwrap_or_default()
    );
    proxy_request("POST", "engrams", Some(request)).await
}

pub async fn list_engrams(
    Query(query): Query<ListQuery>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<EngramErrorResponse>)> {
    info!("📋 List engrams request received");
    let endpoint = format!(
        "engrams?limit={}&offset={}",
        query.limit.unwrap_or(50),
        query.offset.unwrap_or(0)
    );
    proxy_request("GET", &endpoint, None).await
}

pub async fn get_engram(
    Path(id): Path<String>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<EngramErrorResponse>)> {
    info!("📖 Get engram request received for ID: {}", id);
    proxy_request("GET", &format!("engrams/{}", id), None).await
}

pub async fn update_engram(
    Path(id): Path<String>,
    Json(request): Json<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<EngramErrorResponse>)> {
    info!("✏️ Update engram request received for ID: {}", id);
    info!(
        "📋 Request payload: {}",
        serde_json::to_string_pretty(&request).unwrap_or_default()
    );
    proxy_request("PUT", &format!("engrams/{}", id), Some(request)).await
}

pub async fn delete_engram(
    Path(id): Path<String>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<EngramErrorResponse>)> {
    info!("🗑️ Delete engram request received for ID: {}", id);
    proxy_request("DELETE", &format!("engrams/{}", id), None).await
}

pub async fn get_engrams_by_wallet(
    Path(wallet): Path<String>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<EngramErrorResponse>)> {
    info!("👛 Get engrams by wallet request received for: {}", wallet);
    proxy_request("GET", &format!("engrams/wallet/{}", wallet), None).await
}

pub async fn get_engram_by_wallet_key(
    Path((wallet, key)): Path<(String, String)>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<EngramErrorResponse>)> {
    info!(
        "🔑 Get engram by wallet+key request received for: {}/{}",
        wallet, key
    );
    proxy_request("GET", &format!("engrams/wallet/{}/{}", wallet, key), None).await
}

pub async fn search_engrams(
    Json(request): Json<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<EngramErrorResponse>)> {
    info!("🔍 Search engrams request received");
    info!(
        "📋 Request payload: {}",
        serde_json::to_string_pretty(&request).unwrap_or_default()
    );
    proxy_request("POST", "engrams/search", Some(request)).await
}

pub async fn fork_engram(
    Path(id): Path<String>,
    Json(request): Json<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<EngramErrorResponse>)> {
    info!("🔀 Fork engram request received for ID: {}", id);
    info!(
        "📋 Request payload: {}",
        serde_json::to_string_pretty(&request).unwrap_or_default()
    );
    proxy_request("POST", &format!("engrams/{}/fork", id), Some(request)).await
}

pub async fn publish_engram(
    Path(id): Path<String>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<EngramErrorResponse>)> {
    info!("📢 Publish engram request received for ID: {}", id);
    proxy_request("POST", &format!("engrams/{}/publish", id), None).await
}

pub fn create_engram_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/api/engrams/health", get(engram_health))
        .route("/api/engrams", post(create_engram))
        .route("/api/engrams", get(list_engrams))
        .route("/api/engrams/search", post(search_engrams))
        .route("/api/engrams/:id", get(get_engram))
        .route("/api/engrams/:id", put(update_engram))
        .route("/api/engrams/:id", delete(delete_engram))
        .route("/api/engrams/wallet/:wallet", get(get_engrams_by_wallet))
        .route(
            "/api/engrams/wallet/:wallet/:key",
            get(get_engram_by_wallet_key),
        )
        .route("/api/engrams/:id/fork", post(fork_engram))
        .route("/api/engrams/:id/publish", post(publish_engram))
}
