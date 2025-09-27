use axum::{
    extract::{Path, Json},
    response::Json as ResponseJson,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use tracing::{info, error};
use uuid::Uuid;

use crate::database::Database;
use crate::user_references::{UserReferenceService, UserReference, SourceType};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub source_identifier: String,
    pub network: String,                        // Renamed from 'chain' for source agnostic terminology
    pub source_type: SourceType,                // Use proper SourceType enum instead of raw JSON
    pub user_type: Option<String>,              // Optional user type (defaults to "external")
    pub email: Option<String>,                  // Email address if available
    pub metadata: Option<serde_json::Value>,    // General user metadata
    pub preferences: Option<serde_json::Value>, // User preferences

    // Legacy fields for backward compatibility (will be removed in future version)
    #[serde(default)]
    pub chain: Option<String>,                  // Legacy field name - maps to network
    #[serde(default)]
    pub wallet_type: Option<String>,            // Legacy wallet type - handled by source_type conversion
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserResponse {
    pub success: bool,
    pub user_id: Uuid,
    pub message: String,
    pub user: UserReference,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LookupUserRequest {
    pub source_identifier: String,
    pub network: String,                        // Renamed from 'chain' for source agnostic terminology

    // Legacy field for backward compatibility
    #[serde(default)]
    pub chain: Option<String>,                  // Legacy field name - maps to network
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LookupUserResponse {
    pub found: bool,
    pub user: Option<UserReference>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
    pub code: String,
}

pub async fn create_user_endpoint(
    Json(request): Json<CreateUserRequest>
) -> Result<ResponseJson<CreateUserResponse>, (StatusCode, ResponseJson<ErrorResponse>)> {
    info!("🏗️ User creation request received");
    info!("📝 Request payload: {}", serde_json::to_string_pretty(&request).unwrap_or_default());

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres_secure_pass@localhost:5440/erebus".to_string());

    let database = match Database::new(&database_url).await {
        Ok(db) => db,
        Err(e) => {
            error!("❌ Failed to connect to Erebus database: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(ErrorResponse {
                    success: false,
                    error: format!("Database connection failed: {}", e),
                    code: "DB_CONNECTION_ERROR".to_string(),
                }),
            ));
        }
    };

    let user_service = UserReferenceService::new(database);

    // Handle backward compatibility for legacy 'chain' field
    let network = if !request.network.is_empty() {
        request.network.clone()
    } else if let Some(legacy_chain) = request.chain.as_ref() {
        info!("🔄 Using legacy 'chain' field value: {}", legacy_chain);
        legacy_chain.clone()
    } else {
        return Err((
            StatusCode::BAD_REQUEST,
            ResponseJson(ErrorResponse {
                success: false,
                error: "Either 'network' or legacy 'chain' field must be provided".to_string(),
                code: "MISSING_NETWORK_FIELD".to_string(),
            }),
        ));
    };

    // Handle legacy wallet_type conversion to proper SourceType if needed
    let source_type = if let Some(wallet_type) = request.wallet_type.as_ref() {
        info!("🔄 Converting legacy wallet_type '{}' to SourceType::Web3Wallet", wallet_type);
        SourceType::Web3Wallet {
            provider: wallet_type.clone(),
            network: network.clone(),
            metadata: request.metadata.clone().unwrap_or_default(),
        }
    } else {
        request.source_type
    };

    match user_service
        .create_or_get_user(
            &request.source_identifier,
            &network,
            source_type,
            request.user_type.as_deref(),
        )
        .await
    {
        Ok(user_ref) => {
            info!("✅ User created/retrieved successfully: {}", user_ref.id);
            Ok(ResponseJson(CreateUserResponse {
                success: true,
                user_id: user_ref.id,
                message: "User created or retrieved successfully".to_string(),
                user: user_ref,
            }))
        }
        Err(e) => {
            error!("❌ User creation failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(ErrorResponse {
                    success: false,
                    error: format!("User creation failed: {}", e),
                    code: "USER_CREATION_ERROR".to_string(),
                }),
            ))
        }
    }
}

pub async fn lookup_user_endpoint(
    Json(request): Json<LookupUserRequest>
) -> Result<ResponseJson<LookupUserResponse>, (StatusCode, ResponseJson<ErrorResponse>)> {
    info!("🔍 User lookup request received");
    info!("📝 Request payload: {}", serde_json::to_string_pretty(&request).unwrap_or_default());

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres_secure_pass@localhost:5440/erebus".to_string());

    let database = match Database::new(&database_url).await {
        Ok(db) => db,
        Err(e) => {
            error!("❌ Failed to connect to Erebus database: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(ErrorResponse {
                    success: false,
                    error: format!("Database connection failed: {}", e),
                    code: "DB_CONNECTION_ERROR".to_string(),
                }),
            ));
        }
    };

    let user_service = UserReferenceService::new(database);

    // Handle backward compatibility for legacy 'chain' field
    let network = if !request.network.is_empty() {
        request.network.clone()
    } else if let Some(legacy_chain) = request.chain.as_ref() {
        info!("🔄 Using legacy 'chain' field value: {}", legacy_chain);
        legacy_chain.clone()
    } else {
        return Err((
            StatusCode::BAD_REQUEST,
            ResponseJson(ErrorResponse {
                success: false,
                error: "Either 'network' or legacy 'chain' field must be provided".to_string(),
                code: "MISSING_NETWORK_FIELD".to_string(),
            }),
        ));
    };

    match user_service
        .get_user_by_source(&request.source_identifier, &network)
        .await
    {
        Ok(user_opt) => {
            info!("✅ User lookup completed: found={}", user_opt.is_some());
            Ok(ResponseJson(LookupUserResponse {
                found: user_opt.is_some(),
                user: user_opt,
            }))
        }
        Err(e) => {
            error!("❌ User lookup failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(ErrorResponse {
                    success: false,
                    error: format!("User lookup failed: {}", e),
                    code: "USER_LOOKUP_ERROR".to_string(),
                }),
            ))
        }
    }
}

pub async fn get_user_endpoint(
    Path(user_id): Path<Uuid>
) -> Result<ResponseJson<LookupUserResponse>, (StatusCode, ResponseJson<ErrorResponse>)> {
    info!("👤 Get user by ID request received: {}", user_id);

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres_secure_pass@localhost:5440/erebus".to_string());

    let database = match Database::new(&database_url).await {
        Ok(db) => db,
        Err(e) => {
            error!("❌ Failed to connect to Erebus database: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(ErrorResponse {
                    success: false,
                    error: format!("Database connection failed: {}", e),
                    code: "DB_CONNECTION_ERROR".to_string(),
                }),
            ));
        }
    };

    let user_service = UserReferenceService::new(database);

    match user_service.get_user_by_id(&user_id).await {
        Ok(user_opt) => {
            info!("✅ Get user by ID completed: found={}", user_opt.is_some());
            Ok(ResponseJson(LookupUserResponse {
                found: user_opt.is_some(),
                user: user_opt,
            }))
        }
        Err(e) => {
            error!("❌ Get user by ID failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(ErrorResponse {
                    success: false,
                    error: format!("Get user by ID failed: {}", e),
                    code: "USER_GET_ERROR".to_string(),
                }),
            ))
        }
    }
}