// User reference system for multi-source authentication
// Contains scaffolding methods for future authentication providers

#![allow(dead_code)]

use crate::database::Database;
use serde::{Deserialize, Serialize};
use serde_json::{Value as JsonValue, Map as JsonMap};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tracing::{info, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SourceType {
    Web3Wallet {
        provider: String,      // metamask, phantom, coinbase, etc.
        network: String,       // ethereum, solana, etc.
        #[serde(default)]
        metadata: JsonValue,   // Additional wallet-specific data
    },
    ApiKey {
        name: String,          // API service name or identifier
        #[serde(default)]
        scope: Vec<String>,    // API permissions/scopes
        #[serde(default)]
        metadata: JsonValue,   // Additional API-specific data
    },
    EmailAuth {
        verified: bool,        // Email verification status
        provider: String,      // email service provider (gmail, outlook, etc.)
        #[serde(default)]
        metadata: JsonValue,   // Additional email-specific data
    },
    SystemAgent {
        agent_type: String,    // Type of system agent (task_runner, monitor, etc.)
        #[serde(default)]
        capabilities: Vec<String>, // Agent capabilities
        #[serde(default)]
        metadata: JsonValue,   // Additional agent-specific data
    },
    OAuth {
        provider: String,      // OAuth provider (google, github, discord, etc.)
        user_id: String,       // User ID from OAuth provider
        #[serde(default)]
        metadata: JsonValue,   // Additional OAuth-specific data
    },
}

impl SourceType {
    pub fn get_type(&self) -> &str {
        match self {
            SourceType::Web3Wallet { .. } => "web3_wallet",
            SourceType::ApiKey { .. } => "api_key",
            SourceType::EmailAuth { .. } => "email_auth",
            SourceType::SystemAgent { .. } => "system_agent",
            SourceType::OAuth { .. } => "oauth",
        }
    }

    pub fn get_provider(&self) -> Option<&str> {
        match self {
            SourceType::Web3Wallet { provider, .. } => Some(provider),
            SourceType::EmailAuth { provider, .. } => Some(provider),
            SourceType::OAuth { provider, .. } => Some(provider),
            _ => None,
        }
    }

    pub fn get_network(&self) -> Option<&str> {
        match self {
            SourceType::Web3Wallet { network, .. } => Some(network),
            _ => None,
        }
    }

    pub fn from_json(json: JsonValue) -> Result<Self, String> {
        serde_json::from_value(json).map_err(|e| format!("Failed to parse SourceType: {}", e))
    }

    pub fn to_json(&self) -> JsonValue {
        serde_json::to_value(self).unwrap_or(JsonValue::Null)
    }
}

impl Default for SourceType {
    fn default() -> Self {
        SourceType::Web3Wallet {
            provider: "unknown".to_string(),
            network: "unknown".to_string(),
            metadata: JsonValue::Object(JsonMap::new()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserReference {
    pub id: Uuid,
    pub source_identifier: String,
    pub network: String,                    // Renamed from 'chain' to be more generic
    pub user_type: String,                  // external, system, agent, api
    pub source_type: SourceType,            // Strongly typed instead of raw JSONB
    pub email: Option<String>,              // Email address if available
    pub metadata: JsonValue,                // General user metadata
    pub preferences: JsonValue,             // User preferences
    pub is_active: bool,                    // Soft delete flag
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserReferenceRequest {
    pub source_identifier: String,
    pub network: String,
    pub source_type: SourceType,
    pub user_type: Option<String>,              // defaults to "external"
    pub email: Option<String>,
    pub metadata: Option<JsonValue>,
    pub preferences: Option<JsonValue>,
}

pub struct UserReferenceService {
    database: Database,
}

impl UserReferenceService {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    /// Create or get user reference by source identifier
    pub async fn create_or_get_user(&self, source_identifier: &str, network: &str, source_type: SourceType, user_type: Option<&str>) -> Result<UserReference, String> {
        info!("👤 Creating or getting user reference for source: {} on network: {}", source_identifier, network);

        // First, try to get existing user
        match self.get_user_by_source(source_identifier, network).await {
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
        let user_type_str = user_type.unwrap_or("external");
        let source_type_json = source_type.to_json();

        let result = sqlx::query_as::<_, (uuid::Uuid, String, String, String, serde_json::Value, Option<String>, serde_json::Value, serde_json::Value, bool, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
            r#"
            INSERT INTO user_references (
                id, source_identifier, network, user_type, source_type, email, metadata,
                preferences, is_active, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (source_identifier, network) WHERE (source_identifier IS NOT NULL AND is_active = true)
            DO UPDATE SET
                source_type = EXCLUDED.source_type,
                user_type = EXCLUDED.user_type,
                updated_at = NOW()
            RETURNING id, source_identifier, network, user_type, source_type, email, metadata, preferences, is_active, created_at, updated_at
            "#
        )
        .bind(user_id)
        .bind(source_identifier)
        .bind(network)
        .bind(user_type_str)
        .bind(&source_type_json)
        .bind(None::<String>) // email
        .bind(serde_json::json!({})) // metadata
        .bind(serde_json::json!({})) // preferences
        .bind(true) // is_active
        .bind(now) // created_at
        .bind(now) // updated_at
        .fetch_one(self.database.pool())
        .await;

        match result {
            Ok((id, source_identifier, network, user_type, source_type_json, email, metadata, preferences, is_active, created_at, updated_at)) => {
                let source_type = SourceType::from_json(source_type_json)
                    .unwrap_or_else(|_| SourceType::default());

                let user_ref = UserReference {
                    id,
                    source_identifier,
                    network,
                    user_type,
                    source_type,
                    email,
                    metadata,
                    preferences,
                    is_active,
                    created_at,
                    updated_at,
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

    /// Get user reference by source identifier and network
    pub async fn get_user_by_source(&self, source_identifier: &str, network: &str) -> Result<Option<UserReference>, String> {
        let result = sqlx::query_as::<_, (uuid::Uuid, String, String, String, serde_json::Value, Option<String>, serde_json::Value, serde_json::Value, bool, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
            "SELECT id, source_identifier, network, user_type, source_type, email, metadata, preferences, is_active, created_at, updated_at FROM user_references WHERE source_identifier = $1 AND network = $2 AND is_active = true"
        )
        .bind(source_identifier)
        .bind(network)
        .fetch_optional(self.database.pool())
        .await;

        match result {
            Ok(Some((id, source_identifier, network, user_type, source_type_json, email, metadata, preferences, is_active, created_at, updated_at))) => {
                let source_type = SourceType::from_json(source_type_json)
                    .unwrap_or_else(|_| SourceType::default());

                let user_ref = UserReference {
                    id,
                    source_identifier,
                    network,
                    user_type,
                    source_type,
                    email,
                    metadata,
                    preferences,
                    is_active,
                    created_at,
                    updated_at,
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
        let result = sqlx::query_as::<_, (uuid::Uuid, String, String, String, serde_json::Value, Option<String>, serde_json::Value, serde_json::Value, bool, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
            "SELECT id, source_identifier, network, user_type, source_type, email, metadata, preferences, is_active, created_at, updated_at FROM user_references WHERE id = $1 AND is_active = true"
        )
        .bind(user_id)
        .fetch_optional(self.database.pool())
        .await;

        match result {
            Ok(Some((id, source_identifier, network, user_type, source_type_json, email, metadata, preferences, is_active, created_at, updated_at))) => {
                let source_type = SourceType::from_json(source_type_json)
                    .unwrap_or_else(|_| SourceType::default());

                let user_ref = UserReference {
                    id,
                    source_identifier,
                    network,
                    user_type,
                    source_type,
                    email,
                    metadata,
                    preferences,
                    is_active,
                    created_at,
                    updated_at,
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

    /// Create a web3 wallet user (backward compatibility helper)
    pub async fn create_web3_user(&self, source_identifier: &str, network: &str, provider: &str) -> Result<UserReference, String> {
        let source_type = SourceType::Web3Wallet {
            provider: provider.to_string(),
            network: network.to_string(),
            metadata: JsonValue::Object(JsonMap::new()),
        };

        self.create_or_get_user(source_identifier, network, source_type, None).await
    }

    /// Create an API key user
    pub async fn create_api_user(&self, source_identifier: &str, api_name: &str, scope: Vec<String>) -> Result<UserReference, String> {
        let source_type = SourceType::ApiKey {
            name: api_name.to_string(),
            scope,
            metadata: JsonValue::Object(JsonMap::new()),
        };

        self.create_or_get_user(source_identifier, "api", source_type, Some("external")).await
    }

    /// Create an email authenticated user
    pub async fn create_email_user(&self, email: &str, provider: &str, verified: bool) -> Result<UserReference, String> {
        let source_type = SourceType::EmailAuth {
            verified,
            provider: provider.to_string(),
            metadata: JsonValue::Object(JsonMap::new()),
        };

        self.create_or_get_user(email, "email", source_type, Some("external")).await
    }

    /// Create a system agent user
    pub async fn create_system_agent(&self, agent_id: &str, agent_type: &str, capabilities: Vec<String>) -> Result<UserReference, String> {
        let source_type = SourceType::SystemAgent {
            agent_type: agent_type.to_string(),
            capabilities,
            metadata: JsonValue::Object(JsonMap::new()),
        };

        self.create_or_get_user(agent_id, "system", source_type, Some("system")).await
    }

    /// Create an OAuth user
    pub async fn create_oauth_user(&self, oauth_user_id: &str, provider: &str, metadata: JsonValue) -> Result<UserReference, String> {
        let source_type = SourceType::OAuth {
            provider: provider.to_string(),
            user_id: oauth_user_id.to_string(),
            metadata,
        };

        self.create_or_get_user(oauth_user_id, "oauth", source_type, Some("external")).await
    }

    /// Get users by source type
    pub async fn get_users_by_source_type(&self, source_type: &str) -> Result<Vec<UserReference>, String> {
        let result = sqlx::query_as::<_, (uuid::Uuid, String, String, String, serde_json::Value, Option<String>, serde_json::Value, serde_json::Value, bool, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
            "SELECT id, source_identifier, network, user_type, source_type, email, metadata, preferences, is_active, created_at, updated_at FROM user_references WHERE source_type->>'type' = $1 AND is_active = true ORDER BY created_at DESC"
        )
        .bind(source_type)
        .fetch_all(self.database.pool())
        .await;

        match result {
            Ok(rows) => {
                let users = rows.into_iter().map(|(id, source_identifier, network, user_type, source_type_json, email, metadata, preferences, is_active, created_at, updated_at)| {
                    let source_type = SourceType::from_json(source_type_json)
                        .unwrap_or_else(|_| SourceType::default());

                    UserReference {
                        id,
                        source_identifier,
                        network,
                        user_type,
                        source_type,
                        email,
                        metadata,
                        preferences,
                        is_active,
                        created_at,
                        updated_at,
                    }
                }).collect();
                Ok(users)
            }
            Err(e) => {
                error!("❌ Failed to get users by source type: {}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    /// Update user metadata
    pub async fn update_user_metadata(&self, user_id: &Uuid, metadata: JsonValue, preferences: Option<JsonValue>) -> Result<UserReference, String> {
        let update_query = if let Some(_prefs) = &preferences {
            "UPDATE user_references SET metadata = $2, preferences = $3, updated_at = NOW() WHERE id = $1 AND is_active = true RETURNING id, source_identifier, network, user_type, source_type, email, metadata, preferences, is_active, created_at, updated_at"
        } else {
            "UPDATE user_references SET metadata = $2, updated_at = NOW() WHERE id = $1 AND is_active = true RETURNING id, source_identifier, network, user_type, source_type, email, metadata, preferences, is_active, created_at, updated_at"
        };

        let result = if let Some(prefs) = &preferences {
            sqlx::query_as::<_, (uuid::Uuid, String, String, String, serde_json::Value, Option<String>, serde_json::Value, serde_json::Value, bool, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(update_query)
                .bind(user_id)
                .bind(&metadata)
                .bind(&prefs)
                .fetch_one(self.database.pool())
                .await
        } else {
            sqlx::query_as::<_, (uuid::Uuid, String, String, String, serde_json::Value, Option<String>, serde_json::Value, serde_json::Value, bool, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(update_query)
                .bind(user_id)
                .bind(&metadata)
                .fetch_one(self.database.pool())
                .await
        };

        match result {
            Ok((id, source_identifier, network, user_type, source_type_json, email, metadata, preferences, is_active, created_at, updated_at)) => {
                let source_type = SourceType::from_json(source_type_json)
                    .unwrap_or_else(|_| SourceType::default());

                let user_ref = UserReference {
                    id,
                    source_identifier,
                    network,
                    user_type,
                    source_type,
                    email,
                    metadata,
                    preferences,
                    is_active,
                    created_at,
                    updated_at,
                };
                info!("✅ User metadata updated successfully: {}", user_ref.id);
                Ok(user_ref)
            }
            Err(e) => {
                error!("❌ Failed to update user metadata: {}", e);
                Err(format!("Failed to update user metadata: {}", e))
            }
        }
    }
}
