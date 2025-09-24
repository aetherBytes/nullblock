use crate::database::Database;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tracing::{info, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserReference {
    pub id: Uuid,
    pub wallet_address: String,
    pub chain: String,
    pub wallet_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserReferenceRequest {
    pub wallet_address: String,
    pub chain: String,
    pub wallet_type: Option<String>,
}

pub struct UserReferenceService {
    database: Database,
}

impl UserReferenceService {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    /// Create or get user reference by wallet address
    pub async fn create_or_get_user(&self, wallet_address: &str, chain: &str, wallet_type: Option<&str>) -> Result<UserReference, String> {
        info!("👤 Creating or getting user reference for wallet: {} on chain: {}", wallet_address, chain);

        // First, try to get existing user
        match self.get_user_by_wallet(wallet_address, chain).await {
            Ok(Some(user)) => {
                info!("✅ User reference already exists: {}", user.id);
                return Ok(user);
            }
            Ok(None) => {
                // User doesn't exist, create new one
                info!("📝 Creating new user reference for wallet: {}", wallet_address);
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

        let result = sqlx::query_as::<_, (uuid::Uuid, Option<String>, Option<String>, String, Option<chrono::DateTime<chrono::Utc>>, Option<chrono::DateTime<chrono::Utc>>)>(
            r#"
            INSERT INTO user_references (
                id, wallet_address, chain, user_type, email, metadata,
                preferences, synced_at, is_active, erebus_created_at, erebus_updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, wallet_address, chain, user_type, erebus_created_at, erebus_updated_at
            "#
        )
        .bind(user_id)
        .bind(wallet_address)
        .bind(chain)
        .bind(wallet_type_str)
        .bind(None::<String>) // email
        .bind(serde_json::json!({})) // metadata
        .bind(serde_json::json!({})) // preferences
        .bind(now)
        .bind(true) // is_active
        .bind(Some(now))
        .bind(Some(now))
        .fetch_one(self.database.pool())
        .await;

        match result {
            Ok((id, wallet_address, chain, user_type, erebus_created_at, erebus_updated_at)) => {
                let user_ref = UserReference {
                    id,
                    wallet_address: wallet_address.unwrap_or_default(),
                    chain: chain.unwrap_or_default(),
                    wallet_type: user_type,
                    created_at: erebus_created_at.unwrap_or(now),
                    updated_at: erebus_updated_at.unwrap_or(now),
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

    /// Get user reference by wallet address and chain
    pub async fn get_user_by_wallet(&self, wallet_address: &str, chain: &str) -> Result<Option<UserReference>, String> {
        let result = sqlx::query_as::<_, (uuid::Uuid, Option<String>, Option<String>, String, Option<chrono::DateTime<chrono::Utc>>, Option<chrono::DateTime<chrono::Utc>>)>(
            "SELECT id, wallet_address, chain, user_type, erebus_created_at, erebus_updated_at FROM user_references WHERE wallet_address = $1 AND chain = $2 AND is_active = true"
        )
        .bind(wallet_address)
        .bind(chain)
        .fetch_optional(self.database.pool())
        .await;

        match result {
            Ok(Some((id, wallet_address, chain, user_type, erebus_created_at, erebus_updated_at))) => {
                let user_ref = UserReference {
                    id,
                    wallet_address: wallet_address.unwrap_or_default(),
                    chain: chain.unwrap_or_default(),
                    wallet_type: user_type,
                    created_at: erebus_created_at.unwrap_or(Utc::now()),
                    updated_at: erebus_updated_at.unwrap_or(Utc::now()),
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
        let result = sqlx::query_as::<_, (uuid::Uuid, Option<String>, Option<String>, String, Option<chrono::DateTime<chrono::Utc>>, Option<chrono::DateTime<chrono::Utc>>)>(
            "SELECT id, wallet_address, chain, user_type, erebus_created_at, erebus_updated_at FROM user_references WHERE id = $1 AND is_active = true"
        )
        .bind(user_id)
        .fetch_optional(self.database.pool())
        .await;

        match result {
            Ok(Some((id, wallet_address, chain, user_type, erebus_created_at, erebus_updated_at))) => {
                let user_ref = UserReference {
                    id,
                    wallet_address: wallet_address.unwrap_or_default(),
                    chain: chain.unwrap_or_default(),
                    wallet_type: user_type,
                    created_at: erebus_created_at.unwrap_or(Utc::now()),
                    updated_at: erebus_updated_at.unwrap_or(Utc::now()),
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
