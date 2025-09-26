use crate::database::Database;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tracing::{info, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserReference {
    pub id: Uuid,
    pub source_identifier: String,
    pub chain: String,
    pub source_type: serde_json::Value,
    pub wallet_type: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserReferenceRequest {
    pub source_identifier: String,
    pub chain: String,
    pub source_type: serde_json::Value,
    pub wallet_type: Option<String>,
}

pub struct UserReferenceService {
    database: Database,
}

impl UserReferenceService {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    /// Create or get user reference by source identifier
    pub async fn create_or_get_user(&self, source_identifier: &str, chain: &str, source_type: serde_json::Value, wallet_type: Option<&str>) -> Result<UserReference, String> {
        info!("👤 Creating or getting user reference for source: {} on chain: {}", source_identifier, chain);

        // First, try to get existing user
        match self.get_user_by_source(source_identifier, chain).await {
            Ok(Some(user)) => {
                info!("✅ User reference already exists: {}", user.id);
                return Ok(user);
            }
            Ok(None) => {
                // User doesn't exist, create new one
                info!("📝 Creating new user reference for source: {}", source_identifier);
            }
            Err(e) => {
                error!("❌ Failed to check existing user: {}", e);
                return Err(format!("Database error: {}", e));
            }
        }

        // Create new user reference
        let user_id = Uuid::new_v4();
        let now = Utc::now();
        let wallet_type_str = wallet_type.unwrap_or("external");

        let result = sqlx::query_as::<_, (uuid::Uuid, Option<String>, Option<String>, String, serde_json::Value, Option<String>, Option<chrono::DateTime<chrono::Utc>>, Option<chrono::DateTime<chrono::Utc>>)>(
            r#"
            INSERT INTO user_references (
                id, source_identifier, chain, user_type, source_type, wallet_type, email, metadata,
                preferences, additional_metadata, is_active, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            ON CONFLICT (source_identifier, chain) WHERE (source_identifier IS NOT NULL)
            DO UPDATE SET
                source_type = EXCLUDED.source_type,
                wallet_type = EXCLUDED.wallet_type,
                updated_at = NOW()
            RETURNING id, source_identifier, chain, user_type, source_type, wallet_type, created_at, updated_at
            "#
        )
        .bind(user_id)
        .bind(source_identifier)
        .bind(chain)
        .bind("external") // user_type
        .bind(&source_type)
        .bind(wallet_type_str)
        .bind(None::<String>) // email
        .bind(serde_json::json!({})) // metadata
        .bind(serde_json::json!({})) // preferences
        .bind(serde_json::json!({})) // additional_metadata
        .bind(true) // is_active
        .bind(now) // created_at
        .bind(now) // updated_at
        .fetch_one(self.database.pool())
        .await;

        match result {
            Ok((id, source_identifier, chain, _user_type, source_type, wallet_type, created_at, updated_at)) => {
                let user_ref = UserReference {
                    id,
                    source_identifier: source_identifier.unwrap_or_default(),
                    chain: chain.unwrap_or_default(),
                    source_type: source_type,
                    wallet_type: wallet_type,
                    created_at: created_at.unwrap_or(now),
                    updated_at: updated_at.unwrap_or(now),
                };
                info!("✅ User reference created successfully: {}", user_ref.id);
                Ok(user_ref)
            }
            Err(e) => {
                error!("❌ Failed to create user reference: {}", e);
                Err(format!("Failed to create user reference: {}", e))
            }
        }
    }

    /// Get user reference by source identifier and chain
    pub async fn get_user_by_source(&self, source_identifier: &str, chain: &str) -> Result<Option<UserReference>, String> {
        let result = sqlx::query_as::<_, (uuid::Uuid, Option<String>, Option<String>, String, serde_json::Value, Option<String>, Option<chrono::DateTime<chrono::Utc>>, Option<chrono::DateTime<chrono::Utc>>)>(
            "SELECT id, source_identifier, chain, user_type, source_type, wallet_type, created_at, updated_at FROM user_references WHERE source_identifier = $1 AND chain = $2 AND is_active = true"
        )
        .bind(source_identifier)
        .bind(chain)
        .fetch_optional(self.database.pool())
        .await;

        match result {
            Ok(Some((id, source_identifier, chain, _user_type, source_type, wallet_type, created_at, updated_at))) => {
                let user_ref = UserReference {
                    id,
                    source_identifier: source_identifier.unwrap_or_default(),
                    chain: chain.unwrap_or_default(),
                    source_type: source_type,
                    wallet_type: wallet_type,
                    created_at: created_at.unwrap_or(Utc::now()),
                    updated_at: updated_at.unwrap_or(Utc::now()),
                };
                Ok(Some(user_ref))
            }
            Ok(None) => Ok(None),
            Err(e) => {
                error!("❌ Failed to get user reference: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    /// Get user reference by ID
    pub async fn get_user_by_id(&self, user_id: &Uuid) -> Result<Option<UserReference>, String> {
        let result = sqlx::query_as::<_, (uuid::Uuid, Option<String>, Option<String>, String, serde_json::Value, Option<String>, Option<chrono::DateTime<chrono::Utc>>, Option<chrono::DateTime<chrono::Utc>>)>(
            "SELECT id, source_identifier, chain, user_type, source_type, wallet_type, created_at, updated_at FROM user_references WHERE id = $1 AND is_active = true"
        )
        .bind(user_id)
        .fetch_optional(self.database.pool())
        .await;

        match result {
            Ok(Some((id, source_identifier, chain, _user_type, source_type, wallet_type, created_at, updated_at))) => {
                let user_ref = UserReference {
                    id,
                    source_identifier: source_identifier.unwrap_or_default(),
                    chain: chain.unwrap_or_default(),
                    source_type: source_type,
                    wallet_type: wallet_type,
                    created_at: created_at.unwrap_or(Utc::now()),
                    updated_at: updated_at.unwrap_or(Utc::now()),
                };
                Ok(Some(user_ref))
            }
            Ok(None) => Ok(None),
            Err(e) => {
                error!("❌ Failed to get user reference: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }
}
