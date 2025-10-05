use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use tracing::{info, warn};

use crate::auth::{
    extract_bearer_token, extract_api_key, extract_service_token,
    validate_api_key, validate_bearer_token, validate_service_token, AuthConfig,
};

pub async fn mcp_auth_middleware(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let config = AuthConfig::default();
    
    let require_mcp_auth = std::env::var("REQUIRE_MCP_AUTH")
        .unwrap_or_else(|_| "false".to_string())
        .parse()
        .unwrap_or(false);

    if !require_mcp_auth && !config.require_auth {
        tracing::debug!("MCP auth not required, allowing request: {}", request.uri());
        return Ok(next.run(request).await);
    }

    if let Some(service_token) = extract_service_token(&headers) {
        if validate_service_token(&service_token) {
            info!("✅ MCP: Service-to-service auth successful");
            return Ok(next.run(request).await);
        } else {
            warn!("❌ MCP: Invalid service token");
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    if let Some(api_key) = extract_api_key(&headers) {
        if validate_api_key(&api_key, &config) {
            info!("✅ MCP: API key auth successful");
            return Ok(next.run(request).await);
        } else {
            warn!("❌ MCP: Invalid API key");
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    if config.enable_bearer_tokens {
        if let Some(bearer_token) = extract_bearer_token(&headers) {
            match validate_bearer_token(&bearer_token) {
                Ok(auth_ctx) => {
                    info!("✅ MCP: Bearer token auth successful: {:?}", auth_ctx.identity);
                    return Ok(next.run(request).await);
                }
                Err(e) => {
                    warn!("❌ MCP: Invalid bearer token: {}", e);
                    return Err(StatusCode::UNAUTHORIZED);
                }
            }
        }
    }

    if require_mcp_auth || config.require_auth {
        warn!("❌ MCP: No valid authentication provided for: {}", request.uri());
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(next.run(request).await)
}


