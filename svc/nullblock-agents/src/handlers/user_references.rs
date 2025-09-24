use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::Utc;
use tracing::{info, warn, error};
use uuid::Uuid;

use crate::{
    database::repositories::user_references::UserReferenceRepository,
    models::{UserReference, UserReferenceResponse, UserReferenceListResponse},
    server::AppState,
};

#[derive(Debug, serde::Deserialize)]
pub struct CreateUserReferenceRequest {
    pub wallet_address: String,
    pub chain: String,
    pub wallet_type: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct UserReferenceQuery {
    wallet_address: Option<String>,
    chain: Option<String>,
    limit: Option<usize>,
}

/// Create a new user reference
pub async fn create_user_reference(
    State(state): State<AppState>,
    Json(request): Json<CreateUserReferenceRequest>,
) -> Result<Json<UserReferenceResponse>, StatusCode> {
    info!("👤 Creating user reference for wallet: {} on chain: {}", request.wallet_address, request.chain);

    // Check if we have database connection
    let database = match &state.database {
        Some(db) => db,
        None => {
            error!("❌ Database connection not available");
            return Ok(Json(UserReferenceResponse {
                success: false,
                data: None,
                error: Some("Database connection not available".to_string()),
                timestamp: Utc::now(),
            }));
        }
    };

    // Create user reference repository
    let user_repo = UserReferenceRepository::new(database.pool().clone());

    // Check if user already exists
    match user_repo.get_by_wallet(&request.wallet_address, &request.chain).await {
        Ok(Some(existing_user)) => {
            info!("✅ User reference already exists: {}", existing_user.id);
            let user_ref = UserReference {
                id: existing_user.id,
                wallet_address: existing_user.wallet_address.unwrap_or_default(),
                chain: existing_user.chain.unwrap_or_default(),
                wallet_type: existing_user.user_type,
                created_at: existing_user.erebus_created_at.unwrap_or(existing_user.synced_at),
                updated_at: existing_user.erebus_updated_at.unwrap_or(existing_user.synced_at),
            };
            Ok(Json(UserReferenceResponse {
                success: true,
                data: Some(user_ref),
                error: None,
                timestamp: Utc::now(),
            }))
        }
        Ok(None) => {
            // Create new user reference
            let new_user = UserReference {
                id: Uuid::new_v4(),
                wallet_address: request.wallet_address.clone(),
                chain: request.chain.clone(),
                wallet_type: request.wallet_type.unwrap_or_else(|| "external".to_string()),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            match user_repo.create(&new_user).await {
                Ok(created_user) => {
                    info!("✅ User reference created successfully: {}", created_user.id);
                    let user_ref = UserReference {
                        id: created_user.id,
                        wallet_address: created_user.wallet_address.unwrap_or_default(),
                        chain: created_user.chain.unwrap_or_default(),
                        wallet_type: created_user.user_type,
                        created_at: created_user.erebus_created_at.unwrap_or(created_user.synced_at),
                        updated_at: created_user.erebus_updated_at.unwrap_or(created_user.synced_at),
                    };
                    Ok(Json(UserReferenceResponse {
                        success: true,
                        data: Some(user_ref),
                        error: None,
                        timestamp: Utc::now(),
                    }))
                }
                Err(e) => {
                    error!("❌ Failed to create user reference: {}", e);
                    Ok(Json(UserReferenceResponse {
                        success: false,
                        data: None,
                        error: Some(format!("Failed to create user reference: {}", e)),
                        timestamp: Utc::now(),
                    }))
                }
            }
        }
        Err(e) => {
            error!("❌ Failed to check existing user reference: {}", e);
            Ok(Json(UserReferenceResponse {
                success: false,
                data: None,
                error: Some(format!("Database error: {}", e)),
                timestamp: Utc::now(),
            }))
        }
    }
}

/// Get user reference by wallet address
pub async fn get_user_reference(
    State(state): State<AppState>,
    Path((wallet_address, chain)): Path<(String, String)>,
) -> Result<Json<UserReferenceResponse>, StatusCode> {
    info!("👤 Getting user reference for wallet: {} on chain: {}", wallet_address, chain);

    // Check if we have database connection
    let database = match &state.database {
        Some(db) => db,
        None => {
            error!("❌ Database connection not available");
            return Ok(Json(UserReferenceResponse {
                success: false,
                data: None,
                error: Some("Database connection not available".to_string()),
                timestamp: Utc::now(),
            }));
        }
    };

    // Create user reference repository
    let user_repo = UserReferenceRepository::new(database.pool().clone());

    match user_repo.get_by_wallet(&wallet_address, &chain).await {
        Ok(Some(user)) => {
            info!("✅ User reference found: {}", user.id);
            let user_ref = UserReference {
                id: user.id,
                wallet_address: user.wallet_address.unwrap_or_default(),
                chain: user.chain.unwrap_or_default(),
                wallet_type: user.user_type,
                created_at: user.erebus_created_at.unwrap_or(user.synced_at),
                updated_at: user.erebus_updated_at.unwrap_or(user.synced_at),
            };
            Ok(Json(UserReferenceResponse {
                success: true,
                data: Some(user_ref),
                error: None,
                timestamp: Utc::now(),
            }))
        }
        Ok(None) => {
            warn!("⚠️ User reference not found for wallet: {} on chain: {}", wallet_address, chain);
            Ok(Json(UserReferenceResponse {
                success: false,
                data: None,
                error: Some("User reference not found".to_string()),
                timestamp: Utc::now(),
            }))
        }
        Err(e) => {
            error!("❌ Failed to get user reference: {}", e);
            Ok(Json(UserReferenceResponse {
                success: false,
                data: None,
                error: Some(format!("Database error: {}", e)),
                timestamp: Utc::now(),
            }))
        }
    }
}

/// List user references with optional filtering
pub async fn list_user_references(
    State(state): State<AppState>,
    Query(query): Query<UserReferenceQuery>,
) -> Result<Json<UserReferenceListResponse>, StatusCode> {
    info!("👤 Listing user references with filters: {:?}", query);

    // Check if we have database connection
    let database = match &state.database {
        Some(db) => db,
        None => {
            error!("❌ Database connection not available");
            return Ok(Json(UserReferenceListResponse {
                success: false,
                data: None,
                total: 0,
                error: Some("Database connection not available".to_string()),
                timestamp: Utc::now(),
            }));
        }
    };

    // Create user reference repository
    let user_repo = UserReferenceRepository::new(database.pool().clone());

    match user_repo.list(query.wallet_address.as_deref(), query.chain.as_deref(), query.limit).await {
        Ok(users) => {
            let user_refs: Vec<UserReference> = users.into_iter().map(|user| UserReference {
                id: user.id,
                wallet_address: user.wallet_address.unwrap_or_default(),
                chain: user.chain.unwrap_or_default(),
                wallet_type: user.user_type,
                created_at: user.erebus_created_at.unwrap_or(user.synced_at),
                updated_at: user.erebus_updated_at.unwrap_or(user.synced_at),
            }).collect();
            info!("✅ Found {} user references", user_refs.len());
            Ok(Json(UserReferenceListResponse {
                success: true,
                data: Some(user_refs.clone()),
                total: user_refs.len(),
                error: None,
                timestamp: Utc::now(),
            }))
        }
        Err(e) => {
            error!("❌ Failed to list user references: {}", e);
            Ok(Json(UserReferenceListResponse {
                success: false,
                data: None,
                total: 0,
                error: Some(format!("Database error: {}", e)),
                timestamp: Utc::now(),
            }))
        }
    }
}
