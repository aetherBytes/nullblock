// Wallet-specific HTTP routes and handlers
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    Router,
    routing::{get, post},
};

use crate::resources::{
    WalletManager, WalletChallengeRequest, WalletChallengeResponse,
    WalletVerifyRequest, WalletVerifyResponse, WalletListResponse,
};
use crate::resources::types::{
    WalletDetectionRequest, WalletDetectionResponse, WalletConnectionRequest,
    WalletConnectionResponse, WalletStatusResponse
};
use super::wallet_service::WalletService;

/// Create wallet routes for the main router
pub fn create_wallet_routes() -> Router<WalletManager> {
    Router::new()
        .route("/api/wallets", get(get_supported_wallets))
        .route("/api/wallets/detect", post(detect_wallets))
        .route("/api/wallets/connect", post(initiate_wallet_connection))
        .route("/api/wallets/status", get(get_wallet_status))
        .route("/api/wallets/challenge", post(create_wallet_challenge))
        .route("/api/wallets/verify", post(verify_wallet_signature))
        .route("/api/wallets/{wallet_type}/networks", get(get_wallet_networks))
        .route("/api/wallets/sessions/validate", post(validate_session))
}

/// Get all supported wallets endpoint
async fn get_supported_wallets() -> Json<WalletListResponse> {
    Json(WalletManager::get_supported_wallets())
}

/// Detect available wallets endpoint
async fn detect_wallets(
    Json(request): Json<WalletDetectionRequest>,
) -> Json<WalletDetectionResponse> {
    println!("🔍 Wallet detection requested: {:?}", request.available_wallets);
    let response = WalletService::detect_wallets(request.available_wallets);
    Json(response)
}

/// Initiate wallet connection endpoint
async fn initiate_wallet_connection(
    Json(request): Json<WalletConnectionRequest>,
) -> Json<WalletConnectionResponse> {
    println!("🔗 Wallet connection initiated for {}: {}", request.wallet_type, request.wallet_address);
    let response = WalletService::initiate_connection(request);
    Json(response)
}

/// Get wallet status endpoint
async fn get_wallet_status(
    State(wallet_manager): State<WalletManager>,
) -> Json<WalletStatusResponse> {
    // In a real implementation, you'd get the session token from headers or cookies
    // For now, we'll return a disconnected status
    let response = WalletService::get_wallet_status(None, &wallet_manager);
    Json(response)
}

/// Create wallet authentication challenge endpoint
async fn create_wallet_challenge(
    State(wallet_manager): State<WalletManager>,
    Json(request): Json<WalletChallengeRequest>,
) -> Result<Json<WalletChallengeResponse>, StatusCode> {
    match wallet_manager.create_wallet_challenge(request) {
        Ok(response) => Ok(Json(response)),
        Err(error) => {
            println!("❌ Challenge creation failed: {}", error);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// Verify wallet signature endpoint
async fn verify_wallet_signature(
    State(wallet_manager): State<WalletManager>,
    Json(request): Json<WalletVerifyRequest>,
) -> Json<WalletVerifyResponse> {
    Json(wallet_manager.verify_wallet_signature(request))
}

/// Get supported networks for a specific wallet type
async fn get_wallet_networks(
    Path(wallet_type): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let networks = WalletManager::get_wallet_networks(&wallet_type);
    if networks.is_empty() {
        Err(StatusCode::NOT_FOUND)
    } else {
        Ok(Json(serde_json::json!({
            "wallet_type": wallet_type,
            "networks": networks
        })))
    }
}

/// Validate session token endpoint
#[derive(serde::Deserialize)]
struct SessionValidationRequest {
    session_token: String,
}

#[derive(serde::Serialize)]
struct SessionValidationResponse {
    valid: bool,
    session_info: Option<serde_json::Value>,
    message: String,
}

async fn validate_session(
    State(wallet_manager): State<WalletManager>,
    Json(request): Json<SessionValidationRequest>,
) -> Json<SessionValidationResponse> {
    match wallet_manager.validate_session(&request.session_token) {
        Some(session) => Json(SessionValidationResponse {
            valid: true,
            session_info: Some(serde_json::json!({
                "wallet_address": session.wallet_address,
                "wallet_type": session.wallet_type,
                "created_at": session.created_at,
                "expires_at": session.expires_at
            })),
            message: "Session is valid".to_string(),
        }),
        None => Json(SessionValidationResponse {
            valid: false,
            session_info: None,
            message: "Invalid or expired session".to_string(),
        }),
    }
}