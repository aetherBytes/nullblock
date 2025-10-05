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
use crate::user_references::{UserReferenceService, SourceType};
use super::wallet_service::WalletService;
use super::{PhantomWallet, MetaMaskWallet};
use uuid::Uuid;

/// Register user directly in Erebus database after successful wallet verification
async fn register_user_in_database(wallet_address: &str, chain: &str, database: &std::sync::Arc<crate::database::Database>) -> Result<Uuid, String> {
    use tracing::{info, error};

    info!("🗄️ Starting user registration for wallet: {} on chain: {}", wallet_address, chain);

    let user_service = UserReferenceService::new((**database).clone());

    let provider = if chain == "solana" {
        "phantom"
    } else if chain == "ethereum" {
        "metamask"
    } else {
        "unknown"
    };

    info!("🔍 Determined provider: {} for chain: {}", provider, chain);

    let source_type = SourceType::Web3Wallet {
        provider: provider.to_string(),
        network: chain.to_string(),
        metadata: serde_json::json!({}),
    };

    info!("📝 Creating SourceType: {:?}", source_type);
    info!("🗄️ Calling create_or_get_user with identifier: {}, network: {}", wallet_address, chain);

    match user_service.create_or_get_user(wallet_address, chain, source_type, None).await {
        Ok(user_ref) => {
            info!("✅ User registered successfully in Erebus database");
            info!("   User ID: {}", user_ref.id);
            info!("   Source Identifier: {}", user_ref.source_identifier);
            info!("   Network: {}", user_ref.network);
            info!("   Created At: {}", user_ref.created_at);
            Ok(user_ref.id)
        }
        Err(e) => {
            error!("❌ Failed to register user in Erebus database");
            error!("   Wallet Address: {}", wallet_address);
            error!("   Chain: {}", chain);
            error!("   Error: {}", e);
            Err(e)
        }
    }
}

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
    use tracing::{info, warn, error};

    let wallet_address = request.wallet_address.clone();
    info!("🔐 Verifying wallet signature for address: {}", wallet_address);

    let mut verification_response = app_state.wallet_manager.verify_wallet_signature(request);

    // If verification successful, register user via Erebus database
    if verification_response.success {
        info!("✅ Wallet signature verification successful for: {}", wallet_address);
        info!("🎯 Proceeding with user registration in Erebus database");

        // Determine chain based on wallet type
        let chain = if PhantomWallet::validate_solana_address(&wallet_address) {
            info!("🔍 Detected Solana address format");
            "solana"
        } else if MetaMaskWallet::validate_ethereum_address(&wallet_address) {
            info!("🔍 Detected Ethereum address format");
            "ethereum"
        } else {
            warn!("⚠️ Unknown wallet address format: {}", wallet_address);
            "unknown"
        };

        // Register user directly in Erebus database
        match register_user_in_database(&wallet_address, chain, &app_state.database).await {
            Ok(user_id) => {
                info!("✅ User registration completed successfully");
                info!("   User ID: {}", user_id);
                verification_response.user_id = Some(user_id.to_string());
                verification_response.registration_error = None;
            }
            Err(e) => {
                error!("❌ Wallet verification succeeded but user registration failed");
                error!("   Error: {}", e);
                verification_response.user_id = None;
                verification_response.registration_error = Some(e.clone());
                // Note: We still return success=true because wallet verification succeeded
                // Frontend can check registration_error to see if there was a registration issue
                warn!("⚠️ Wallet is verified but user not registered in database");
            }
        }
    } else {
        warn!("❌ Wallet signature verification failed for: {}", wallet_address);
        verification_response.user_id = None;
        verification_response.registration_error = None;
    }

    info!("📤 Returning verification response: success={}, user_id={:?}, registration_error={:?}",
          verification_response.success,
          verification_response.user_id,
          verification_response.registration_error);

    Json(verification_response)
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