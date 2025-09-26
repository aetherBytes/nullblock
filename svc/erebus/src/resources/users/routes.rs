use axum::{
    extract::{Path, Json},
    response::Json as ResponseJson,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use tracing::{info, error};
use uuid::Uuid;

use crate::database::Database;
use crate::user_references::{UserReferenceService, UserReference};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub source_identifier: String,
    pub chain: String,
    pub source_type: Option<serde_json::Value>,
    pub wallet_type: Option<String>,
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
    pub chain: String,
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
        .unwrap_or_else(|_| "postgresql://postgres:REDACTED_DB_PASS@localhost:5440/erebus".to_string());

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

    let source_type = request.source_type.unwrap_or_else(|| serde_json::json!({
        "type": "web3_wallet",
        "provider": request.wallet_type.as_deref().unwrap_or("unknown"),
        "metadata": {}
    }));

    match user_service
        .create_or_get_user(
            &request.source_identifier,
            &request.chain,
            source_type,
            request.wallet_type.as_deref(),
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
        .unwrap_or_else(|_| "postgresql://postgres:REDACTED_DB_PASS@localhost:5440/erebus".to_string());

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

    match user_service
        .get_user_by_source(&request.source_identifier, &request.chain)
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
        .unwrap_or_else(|_| "postgresql://postgres:REDACTED_DB_PASS@localhost:5440/erebus".to_string());

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