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
pub fn create_wallet_routes() -> Router<crate::AppState> {
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
    State(app_state): State<crate::AppState>,
) -> Json<WalletStatusResponse> {
    // In a real implementation, you'd get the session token from headers or cookies
    // For now, we'll return a disconnected status
    let response = WalletService::get_wallet_status(None, &app_state.wallet_manager);
    Json(response)
}

/// Create wallet authentication challenge endpoint
async fn create_wallet_challenge(
    State(app_state): State<crate::AppState>,
    Json(request): Json<WalletChallengeRequest>,
) -> Result<Json<WalletChallengeResponse>, StatusCode> {
    match app_state.wallet_manager.create_wallet_challenge(request) {
        Ok(response) => Ok(Json(response)),
        Err(error) => {
            println!("❌ Challenge creation failed: {}", error);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// Verify wallet signature endpoint
async fn verify_wallet_signature(
    State(app_state): State<crate::AppState>,
    Json(request): Json<WalletVerifyRequest>,
) -> Json<WalletVerifyResponse> {
    Json(app_state.wallet_manager.verify_wallet_signature(request))
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

/// Validate wallet session endpoint
async fn validate_session() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "valid": false,
        "message": "Session validation not implemented yet"
    }))
}