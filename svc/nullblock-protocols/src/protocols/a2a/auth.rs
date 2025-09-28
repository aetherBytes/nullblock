use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};

pub async fn auth_middleware(
    _headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // For now, just pass through all requests
    // TODO: Implement actual authentication based on security schemes

    // Optional: Log incoming requests for debugging
    tracing::debug!("A2A request to: {}", request.uri());

    Ok(next.run(request).await)
}

pub fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|auth_header| {
            if auth_header.starts_with("Bearer ") {
                Some(auth_header[7..].to_string())
            } else {
                None
            }
        })
}

pub fn extract_api_key(headers: &HeaderMap, key_name: &str) -> Option<String> {
    headers
        .get(key_name)
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string())
}