use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tracing::{info, warn};

#[derive(Clone)]
pub struct AuthConfig {
    pub api_keys: HashSet<String>,
    pub enable_bearer_tokens: bool,
    pub require_auth: bool,
}

impl Default for AuthConfig {
    fn default() -> Self {
        let api_keys_str = std::env::var("API_KEYS")
            .unwrap_or_else(|_| String::new());
        
        let api_keys: HashSet<String> = api_keys_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let require_auth = std::env::var("REQUIRE_AUTH")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);

        let enable_bearer_tokens = std::env::var("ENABLE_BEARER_TOKENS")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        Self {
            api_keys,
            enable_bearer_tokens,
            require_auth,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
    pub authenticated: bool,
    pub auth_type: Option<AuthType>,
    pub identity: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthType {
    ApiKey,
    BearerToken,
    ServiceToService,
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

pub fn validate_api_key(api_key: &str, config: &AuthConfig) -> bool {
    if config.api_keys.is_empty() {
        return true;
    }
    config.api_keys.contains(api_key)
}

pub fn validate_bearer_token(token: &str) -> Result<AuthContext, String> {
    if token.is_empty() {
        return Err("Empty token".to_string());
    }
    
    Ok(AuthContext {
        authenticated: true,
        auth_type: Some(AuthType::BearerToken),
        identity: Some(token[..8.min(token.len())].to_string()),
    })
}

pub fn validate_service_token(token: &str) -> bool {
    let expected_token = std::env::var("SERVICE_SECRET")
        .unwrap_or_else(|_| "nullblock-service-secret-dev".to_string());
    
    token == expected_token
}

pub async fn auth_middleware(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let config = AuthConfig::default();
    
    if !config.require_auth {
        tracing::debug!("Auth not required, allowing request: {}", request.uri());
        return Ok(next.run(request).await);
    }

    if let Some(service_token) = extract_service_token(&headers) {
        if validate_service_token(&service_token) {
            info!("✅ Service-to-service auth successful");
            return Ok(next.run(request).await);
        } else {
            warn!("❌ Invalid service token");
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    if let Some(api_key) = extract_api_key(&headers) {
        if validate_api_key(&api_key, &config) {
            info!("✅ API key auth successful");
            return Ok(next.run(request).await);
        } else {
            warn!("❌ Invalid API key");
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    if config.enable_bearer_tokens {
        if let Some(bearer_token) = extract_bearer_token(&headers) {
            match validate_bearer_token(&bearer_token) {
                Ok(auth_ctx) => {
                    info!("✅ Bearer token auth successful: {:?}", auth_ctx.identity);
                    return Ok(next.run(request).await);
                }
                Err(e) => {
                    warn!("❌ Invalid bearer token: {}", e);
                    return Err(StatusCode::UNAUTHORIZED);
                }
            }
        }
    }

    warn!("❌ No valid authentication provided for: {}", request.uri());
    Err(StatusCode::UNAUTHORIZED)
}

pub async fn optional_auth_middleware(
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let config = AuthConfig::default();
    let mut auth_ctx = AuthContext {
        authenticated: false,
        auth_type: None,
        identity: None,
    };

    if let Some(service_token) = extract_service_token(&headers) {
        if validate_service_token(&service_token) {
            info!("✅ Optional auth: Service-to-service");
            auth_ctx = AuthContext {
                authenticated: true,
                auth_type: Some(AuthType::ServiceToService),
                identity: Some("service".to_string()),
            };
        }
    } else if let Some(api_key) = extract_api_key(&headers) {
        if validate_api_key(&api_key, &config) {
            info!("✅ Optional auth: API key");
            auth_ctx = AuthContext {
                authenticated: true,
                auth_type: Some(AuthType::ApiKey),
                identity: Some("api_key_user".to_string()),
            };
        }
    } else if config.enable_bearer_tokens {
        if let Some(bearer_token) = extract_bearer_token(&headers) {
            if let Ok(ctx) = validate_bearer_token(&bearer_token) {
                info!("✅ Optional auth: Bearer token");
                auth_ctx = ctx;
            }
        }
    }

    request.extensions_mut().insert(auth_ctx);
    Ok(next.run(request).await)
}

