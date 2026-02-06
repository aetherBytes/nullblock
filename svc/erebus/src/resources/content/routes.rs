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

fn get_content_service_url() -> String {
    std::env::var("CONTENT_SERVICE_URL").unwrap_or_else(|_| "http://localhost:8002".to_string())
}

#[derive(Debug, Serialize)]
pub struct ContentErrorResponse {
    pub error: String,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub status: Option<String>,
    pub theme: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

async fn proxy_request(
    method: &str,
    endpoint: &str,
    body: Option<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ContentErrorResponse>)> {
    let client = reqwest::Client::new();
    let base_url = get_content_service_url();
    let url = format!("{}/{}", base_url, endpoint);

    info!("🔗 Proxying {} request to Content service: {}", method, url);

    let request_builder = match method {
        "GET" => client.get(&url),
        "POST" => client.post(&url),
        "PUT" => client.put(&url),
        "DELETE" => client.delete(&url),
        _ => {
            return Err((
                StatusCode::METHOD_NOT_ALLOWED,
                ResponseJson(ContentErrorResponse {
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
                        info!("✅ Content service response successful");
                        Ok(ResponseJson(json_response))
                    } else {
                        error!("❌ Content service returned error status: {}", status);
                        Err((
                            StatusCode::from_u16(status.as_u16())
                                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                            ResponseJson(ContentErrorResponse {
                                error: "service_error".to_string(),
                                code: "SERVICE_ERROR".to_string(),
                                message: format!("Content service returned {}", status),
                            }),
                        ))
                    }
                }
                Err(e) => {
                    error!("❌ Failed to parse Content service response as JSON: {}", e);
                    Err((
                        StatusCode::BAD_GATEWAY,
                        ResponseJson(ContentErrorResponse {
                            error: "invalid_response".to_string(),
                            code: "INVALID_RESPONSE".to_string(),
                            message: format!("Failed to parse response: {}", e),
                        }),
                    ))
                }
            }
        }
        Err(e) => {
            error!("❌ Failed to connect to Content service: {}", e);
            Err((
                StatusCode::BAD_GATEWAY,
                ResponseJson(ContentErrorResponse {
                    error: "service_unavailable".to_string(),
                    code: "SERVICE_UNAVAILABLE".to_string(),
                    message: format!("Content service unavailable: {}", e),
                }),
            ))
        }
    }
}

async fn content_health(
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ContentErrorResponse>)> {
    info!("🏥 Content service health check request received");
    proxy_request("GET", "health", None).await
}

async fn generate_content(
    Json(request): Json<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ContentErrorResponse>)> {
    info!("🎨 Generate content request received");
    proxy_request("POST", "api/content/generate", Some(request)).await
}

async fn list_queue(
    Query(query): Query<ListQuery>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ContentErrorResponse>)> {
    info!("📋 List content queue request received");
    let mut endpoint = "api/content/queue?".to_string();
    let mut params = vec![];

    if let Some(status) = query.status {
        params.push(format!("status={}", status));
    }
    if let Some(theme) = query.theme {
        params.push(format!("theme={}", theme));
    }
    if let Some(limit) = query.limit {
        params.push(format!("limit={}", limit));
    }
    if let Some(offset) = query.offset {
        params.push(format!("offset={}", offset));
    }

    endpoint.push_str(&params.join("&"));
    proxy_request("GET", &endpoint, None).await
}

async fn get_content(
    Path(id): Path<String>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ContentErrorResponse>)> {
    info!("📄 Get content request received for ID: {}", id);
    proxy_request("GET", &format!("api/content/queue/{}", id), None).await
}

async fn update_content(
    Path(id): Path<String>,
    Json(request): Json<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ContentErrorResponse>)> {
    info!("✏️ Update content request received for ID: {}", id);
    proxy_request("PUT", &format!("api/content/queue/{}", id), Some(request)).await
}

async fn delete_content(
    Path(id): Path<String>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ContentErrorResponse>)> {
    info!("🗑️ Delete content request received for ID: {}", id);
    proxy_request("DELETE", &format!("api/content/queue/{}", id), None).await
}

async fn get_metrics(
    Path(id): Path<String>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ContentErrorResponse>)> {
    info!("📊 Get metrics request received for content ID: {}", id);
    proxy_request("GET", &format!("api/content/metrics/{}", id), None).await
}

async fn list_templates(
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ContentErrorResponse>)> {
    info!("📚 List templates request received");
    proxy_request("GET", "api/content/templates", None).await
}

pub fn create_content_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/api/content/health", get(content_health))
        .route("/api/content/generate", post(generate_content))
        .route("/api/content/queue", get(list_queue))
        .route("/api/content/queue/:id", get(get_content))
        .route("/api/content/queue/:id", put(update_content))
        .route("/api/content/queue/:id", delete(delete_content))
        .route("/api/content/metrics/:id", get(get_metrics))
        .route("/api/content/templates", get(list_templates))
}
