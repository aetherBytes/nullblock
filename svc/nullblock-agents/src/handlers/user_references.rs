use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::Utc;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    database::repositories::user_references::UserReferenceRepository,
    models::{UserReference, UserReferenceListResponse, UserReferenceResponse},
    server::AppState,
};

/// Call Erebus user registration API (enforces GOLDEN RULE)
async fn call_erebus_user_api(
    source_identifier: &str,
    network: &str,
    source_type: &serde_json::Value,
    wallet_type: Option<&str>,
) -> Result<UserReference, String> {
    let client = reqwest::Client::new();
    let erebus_url =
        std::env::var("EREBUS_BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

    let request_body = serde_json::json!({
        "source_identifier": source_identifier,
        "network": network,
        "source_type": source_type,
        "wallet_type": wallet_type.unwrap_or("unknown")
    });

    info!(
        "🌐 Calling Erebus user API to enforce GOLDEN RULE: {}/api/users/register",
        erebus_url
    );

    match client
        .post(&format!("{}/api/users/register", erebus_url))
        .json(&request_body)
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(json_response) => {
                        if let (Some(user_id_str), Some(user_data)) = (
                            json_response["user_id"].as_str(),
                            json_response["user"].as_object(),
                        ) {
                            match Uuid::parse_str(user_id_str) {
                                Ok(user_id) => Ok(UserReference {
                                    id: user_id,
                                    source_identifier: user_data
                                        .get("source_identifier")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or(source_identifier)
                                        .to_string(),
                                    network: user_data
                                        .get("network")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or(network)
                                        .to_string(),
                                    source_type: user_data
                                        .get("source_type")
                                        .cloned()
                                        .unwrap_or_else(|| source_type.clone()),
                                    wallet_type: user_data
                                        .get("wallet_type")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_string()),
                                    created_at: chrono::Utc::now(),
                                    updated_at: chrono::Utc::now(),
                                }),
                                Err(e) => Err(format!("Invalid UUID in Erebus response: {}", e)),
                            }
                        } else {
                            Err("Invalid response format from Erebus API".to_string())
                        }
                    }
                    Err(e) => Err(format!("Failed to parse Erebus API response: {}", e)),
                }
            } else {
                let error_text = response.text().await.unwrap_or_default();
                Err(format!("Erebus API error: {}", error_text))
            }
        }
        Err(e) => Err(format!("Failed to call Erebus API: {}", e)),
    }
}

/// Call Erebus user lookup API (enforces GOLDEN RULE)
async fn call_erebus_user_lookup_api(
    source_identifier: &str,
    network: &str,
) -> Result<Option<UserReference>, String> {
    let client = reqwest::Client::new();
    let erebus_url =
        std::env::var("EREBUS_BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

    let request_body = serde_json::json!({
        "source_identifier": source_identifier,
        "network": network
    });

    info!(
        "🔍 Calling Erebus user lookup API to enforce GOLDEN RULE: {}/api/users/lookup",
        erebus_url
    );

    match client
        .post(&format!("{}/api/users/lookup", erebus_url))
        .json(&request_body)
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(json_response) => {
                        if json_response["found"].as_bool() == Some(true) {
                            if let Some(user_data) = json_response["user"].as_object() {
                                if let Some(user_id_str) =
                                    user_data.get("id").and_then(|v| v.as_str())
                                {
                                    match Uuid::parse_str(user_id_str) {
                                        Ok(user_id) => Ok(Some(UserReference {
                                            id: user_id,
                                            source_identifier: user_data
                                                .get("source_identifier")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or(source_identifier)
                                                .to_string(),
                                            network: user_data
                                                .get("network")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or(network)
                                                .to_string(),
                                            source_type: user_data
                                                .get("source_type")
                                                .cloned()
                                                .unwrap_or_else(|| serde_json::json!({})),
                                            wallet_type: user_data
                                                .get("wallet_type")
                                                .and_then(|v| v.as_str())
                                                .map(|s| s.to_string()),
                                            created_at: chrono::Utc::now(),
                                            updated_at: chrono::Utc::now(),
                                        })),
                                        Err(e) => {
                                            Err(format!("Invalid UUID in Erebus response: {}", e))
                                        }
                                    }
                                } else {
                                    Err("No user ID in Erebus response".to_string())
                                }
                            } else {
                                Err("Invalid user data format in Erebus response".to_string())
                            }
                        } else {
                            Ok(None) // User not found
                        }
                    }
                    Err(e) => Err(format!("Failed to parse Erebus API response: {}", e)),
                }
            } else {
                let error_text = response.text().await.unwrap_or_default();
                Err(format!("Erebus API error: {}", error_text))
            }
        }
        Err(e) => Err(format!("Failed to call Erebus API: {}", e)),
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct CreateUserReferenceRequest {
    pub source_identifier: String,
    pub network: String,
    pub source_type: serde_json::Value,
    pub wallet_type: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct SyncUserReferenceRequest {
    pub id: String,
    pub source_identifier: String,
    pub network: String,
    pub source_type: serde_json::Value,
    pub wallet_type: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct UserReferenceQuery {
    source_identifier: Option<String>,
    network: Option<String>,
    limit: Option<usize>,
}

/// Create a new user reference - DEPRECATED: Use Erebus API instead
/// This endpoint now forwards requests to Erebus to maintain GOLDEN RULE
pub async fn create_user_reference(
    State(_state): State<AppState>,
    Json(request): Json<CreateUserReferenceRequest>,
) -> Result<Json<UserReferenceResponse>, StatusCode> {
    info!("👤 [DEPRECATED] User creation request received - forwarding to Erebus API");

    warn!("⚠️ ARCHITECTURAL VIOLATION: Direct user creation in agents service is deprecated");
    warn!("⚠️ Use Erebus /api/users/register endpoint instead - enforcing GOLDEN RULE");

    // Forward to Erebus user registration API
    match call_erebus_user_api(
        &request.source_identifier,
        &request.network,
        &request.source_type,
        request.wallet_type.as_deref(),
    )
    .await
    {
        Ok(user_ref) => {
            info!("✅ User created via Erebus API forwarding: {}", user_ref.id);
            Ok(Json(UserReferenceResponse {
                success: true,
                data: Some(user_ref),
                error: None,
                timestamp: Utc::now(),
            }))
        }
        Err(e) => {
            error!("❌ Failed to create user via Erebus API: {}", e);
            Ok(Json(UserReferenceResponse {
                success: false,
                data: None,
                error: Some(format!("Failed to create user via Erebus API: {}", e)),
                timestamp: Utc::now(),
            }))
        }
    }
}

/// Get user reference by wallet address - DEPRECATED: Use Erebus API
/// This endpoint now forwards lookups to Erebus to maintain GOLDEN RULE
pub async fn get_user_reference(
    State(_state): State<AppState>,
    Path((source_identifier, network)): Path<(String, String)>,
) -> Result<Json<UserReferenceResponse>, StatusCode> {
    info!("👤 [DEPRECATED] User lookup request received - forwarding to Erebus API");

    warn!("⚠️ ARCHITECTURAL VIOLATION: Direct user lookup in agents service is deprecated");
    warn!("⚠️ Use Erebus /api/users/lookup endpoint instead - enforcing GOLDEN RULE");

    // Forward to Erebus user lookup API
    match call_erebus_user_lookup_api(&source_identifier, &network).await {
        Ok(Some(user_ref)) => {
            info!("✅ User found via Erebus API forwarding: {}", user_ref.id);
            Ok(Json(UserReferenceResponse {
                success: true,
                data: Some(user_ref),
                error: None,
                timestamp: Utc::now(),
            }))
        }
        Ok(None) => {
            info!("ℹ️ User not found via Erebus API forwarding");
            Ok(Json(UserReferenceResponse {
                success: false,
                data: None,
                error: Some("User reference not found".to_string()),
                timestamp: Utc::now(),
            }))
        }
        Err(e) => {
            error!("❌ Failed to lookup user via Erebus API: {}", e);
            Ok(Json(UserReferenceResponse {
                success: false,
                data: None,
                error: Some(format!("Failed to lookup user via Erebus API: {}", e)),
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

    match user_repo
        .list(
            query.source_identifier.as_deref(),
            query.network.as_deref(),
            query.limit,
        )
        .await
    {
        Ok(users) => {
            let user_refs: Vec<UserReference> = users
                .into_iter()
                .map(|user| UserReference {
                    id: user.id,
                    source_identifier: user.source_identifier.unwrap_or_default(),
                    network: user.network.unwrap_or_default(),
                    source_type: user.source_type,
                    wallet_type: None,
                    created_at: user.created_at,
                    updated_at: user.updated_at,
                })
                .collect();
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

/// Sync user reference from Erebus
pub async fn sync_user_reference(
    State(state): State<AppState>,
    Json(request): Json<SyncUserReferenceRequest>,
) -> Result<Json<UserReferenceResponse>, StatusCode> {
    info!(
        "🔄 Syncing user reference from Erebus: {} on network: {}",
        request.source_identifier, request.network
    );

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

    // Parse user ID
    let user_id = match Uuid::parse_str(&request.id) {
        Ok(id) => id,
        Err(e) => {
            error!("❌ Invalid user ID format: {}", e);
            return Ok(Json(UserReferenceResponse {
                success: false,
                data: None,
                error: Some(format!("Invalid user ID format: {}", e)),
                timestamp: Utc::now(),
            }));
        }
    };

    // Parse timestamps
    let erebus_created_at = match chrono::DateTime::parse_from_rfc3339(&request.created_at) {
        Ok(dt) => Some(dt.with_timezone(&Utc)),
        Err(e) => {
            warn!("⚠️ Failed to parse erebus_created_at: {}", e);
            None
        }
    };

    let erebus_updated_at = match chrono::DateTime::parse_from_rfc3339(&request.updated_at) {
        Ok(dt) => Some(dt.with_timezone(&Utc)),
        Err(e) => {
            warn!("⚠️ Failed to parse erebus_updated_at: {}", e);
            None
        }
    };

    // Create user reference repository
    let user_repo = UserReferenceRepository::new(database.pool().clone());

    // Sync user from Erebus
    match user_repo
        .upsert_from_kafka_event(
            user_id,
            Some(&request.source_identifier),
            Some(&request.network),
            &request.source_type,
            request.wallet_type.as_deref(),
            None,                   // email
            &serde_json::json!({}), // metadata
            erebus_created_at,
            erebus_updated_at,
        )
        .await
    {
        Ok(user_ref) => {
            info!("✅ User reference synced successfully: {}", user_ref.id);
            let user = UserReference {
                id: user_ref.id,
                source_identifier: user_ref.source_identifier.unwrap_or_default(),
                network: user_ref.network.unwrap_or_default(),
                source_type: user_ref.source_type,
                wallet_type: None,
                created_at: user_ref.created_at,
                updated_at: user_ref.updated_at,
            };
            Ok(Json(UserReferenceResponse {
                success: true,
                data: Some(user),
                error: None,
                timestamp: Utc::now(),
            }))
        }
        Err(e) => {
            error!("❌ Failed to sync user reference: {}", e);
            Ok(Json(UserReferenceResponse {
                success: false,
                data: None,
                error: Some(format!("Database error: {}", e)),
                timestamp: Utc::now(),
            }))
        }
    }
}
