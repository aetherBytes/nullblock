// Authentication middleware and utilities
// Contains scaffolding functions for various auth patterns

#![allow(dead_code)]

use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use tracing::{info, warn};

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

pub fn extract_api_key(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-api-key")
        .or_else(|| headers.get("api-key"))
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string())
}

pub fn extract_service_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-service-token")
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string())
}

pub fn extract_wallet_address(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-wallet-address")
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string())
}

pub fn extract_wallet_chain(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-wallet-chain")
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string())
}

pub fn extract_session_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-session-token")
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string())
}

pub fn validate_service_token(token: &str) -> bool {
    let expected_token = std::env::var("SERVICE_SECRET")
        .unwrap_or_else(|_| "nullblock-service-secret-dev".to_string());
    
    token == expected_token
}

pub fn validate_api_key(api_key: &str) -> bool {
    let api_keys_str = std::env::var("API_KEYS")
        .unwrap_or_else(|_| String::new());
    
    if api_keys_str.is_empty() {
        return true;
    }

    api_keys_str
        .split(',')
        .map(|s| s.trim())
        .any(|key| key == api_key)
}

pub async fn service_auth_middleware(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let require_service_auth = std::env::var("REQUIRE_SERVICE_AUTH")
        .unwrap_or_else(|_| "false".to_string())
        .parse()
        .unwrap_or(false);

    if !require_service_auth {
        tracing::debug!("Service auth not required, allowing request: {}", request.uri());
        return Ok(next.run(request).await);
    }

    if let Some(service_token) = extract_service_token(&headers) {
        if validate_service_token(&service_token) {
            info!("✅ Service-to-service auth successful from protocols service");
            return Ok(next.run(request).await);
        } else {
            warn!("❌ Invalid service token from: {}", request.uri());
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    warn!("❌ No service token provided for: {}", request.uri());
    Err(StatusCode::UNAUTHORIZED)
}

pub async fn optional_auth_middleware(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if let Some(service_token) = extract_service_token(&headers) {
        if validate_service_token(&service_token) {
            info!("✅ Service token validated");
        } else {
            warn!("⚠️ Invalid service token, continuing without auth");
        }
    } else if let Some(api_key) = extract_api_key(&headers) {
        if validate_api_key(&api_key) {
            info!("✅ API key validated");
        } else {
            warn!("⚠️ Invalid API key, continuing without auth");
        }
    } else if let Some(bearer_token) = extract_bearer_token(&headers) {
        info!("✅ Bearer token detected: {}...", &bearer_token[..8.min(bearer_token.len())]);
    } else if let Some(wallet) = extract_wallet_address(&headers) {
        let chain = extract_wallet_chain(&headers).unwrap_or_else(|| "unknown".to_string());
        info!("✅ Wallet detected: {} on {}", wallet, chain);
    } else if let Some(session) = extract_session_token(&headers) {
        info!("✅ Session token detected: {}...", &session[..8.min(session.len())]);
    }

    Ok(next.run(request).await)
}


