use super::models::{ApiKey, ApiKeyProvider, CreateApiKeyRequest, DecryptedApiKey, UpdateApiKeyRequest};
use crate::crypto::{EncryptedData, EncryptionService};
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use std::sync::Arc;
use uuid::Uuid;

#[derive(FromRow)]
struct ApiKeyRow {
    id: Uuid,
    user_id: Uuid,
    provider: String,
    encrypted_key: Vec<u8>,
    encryption_iv: Vec<u8>,
    encryption_tag: Vec<u8>,
    key_prefix: Option<String>,
    key_suffix: Option<String>,
    key_name: Option<String>,
    last_used_at: Option<DateTime<Utc>>,
    usage_count: i64,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<ApiKeyRow> for ApiKey {
    type Error = String;

    fn try_from(row: ApiKeyRow) -> Result<Self, Self::Error> {
        Ok(ApiKey {
            id: row.id,
            user_id: row.user_id,
            provider: ApiKeyProvider::from_str(&row.provider)?,
            encrypted_key: row.encrypted_key,
            encryption_iv: row.encryption_iv,
            encryption_tag: row.encryption_tag,
            key_prefix: row.key_prefix,
            key_suffix: row.key_suffix,
            key_name: row.key_name,
            last_used_at: row.last_used_at,
            usage_count: row.usage_count,
            is_active: row.is_active,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[derive(Clone)]
pub struct ApiKeyService {
    pool: Arc<PgPool>,
    encryption: Arc<EncryptionService>,
}

impl ApiKeyService {
    pub fn new(pool: Arc<PgPool>, encryption: Arc<EncryptionService>) -> Self {
        Self { pool, encryption }
    }

    pub async fn create_or_update_api_key(
        &self,
        user_id: Uuid,
        request: CreateApiKeyRequest,
    ) -> Result<ApiKey, String> {
        let provider = ApiKeyProvider::from_str(&request.provider)
            .map_err(|e| format!("Invalid provider: {}", e))?;

        let encrypted = self
            .encryption
            .encrypt(&request.api_key)
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let (key_prefix, key_suffix) = EncryptionService::extract_prefix_suffix(&request.api_key);

        let existing_key = sqlx::query_as::<_, (Uuid,)>(
            "SELECT id FROM user_api_keys WHERE user_id = $1 AND provider = $2"
        )
        .bind(&user_id)
        .bind(provider.as_str())
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        if let Some((existing_id,)) = existing_key {
            self.update_existing_key(
                existing_id,
                &encrypted,
                &key_prefix,
                &key_suffix,
                request.key_name.as_deref(),
            )
            .await
        } else {
            self.create_new_key(
                user_id,
                provider,
                &encrypted,
                &key_prefix,
                &key_suffix,
                request.key_name.as_deref(),
            )
            .await
        }
    }

    async fn create_new_key(
        &self,
        user_id: Uuid,
        provider: ApiKeyProvider,
        encrypted: &EncryptedData,
        key_prefix: &str,
        key_suffix: &str,
        key_name: Option<&str>,
    ) -> Result<ApiKey, String> {
        let row = sqlx::query_as::<_, ApiKeyRow>(
            r#"
            INSERT INTO user_api_keys
            (user_id, provider, encrypted_key, encryption_iv, encryption_tag, key_prefix, key_suffix, key_name)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING
                id, user_id, provider, encrypted_key, encryption_iv, encryption_tag,
                key_prefix, key_suffix, key_name, last_used_at, usage_count, is_active,
                created_at, updated_at
            "#
        )
        .bind(&user_id)
        .bind(provider.as_str())
        .bind(&encrypted.ciphertext)
        .bind(&encrypted.iv)
        .bind(&encrypted.tag)
        .bind(key_prefix)
        .bind(key_suffix)
        .bind(key_name)
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| format!("Failed to create API key: {}", e))?;

        row.try_into()
    }

    async fn update_existing_key(
        &self,
        key_id: Uuid,
        encrypted: &EncryptedData,
        key_prefix: &str,
        key_suffix: &str,
        key_name: Option<&str>,
    ) -> Result<ApiKey, String> {
        let row = sqlx::query_as::<_, ApiKeyRow>(
            r#"
            UPDATE user_api_keys
            SET encrypted_key = $1, encryption_iv = $2, encryption_tag = $3,
                key_prefix = $4, key_suffix = $5, key_name = $6,
                updated_at = NOW()
            WHERE id = $7
            RETURNING
                id, user_id, provider, encrypted_key, encryption_iv, encryption_tag,
                key_prefix, key_suffix, key_name, last_used_at, usage_count, is_active,
                created_at, updated_at
            "#
        )
        .bind(&encrypted.ciphertext)
        .bind(&encrypted.iv)
        .bind(&encrypted.tag)
        .bind(key_prefix)
        .bind(key_suffix)
        .bind(key_name)
        .bind(&key_id)
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| format!("Failed to update API key: {}", e))?;

        row.try_into()
    }

    pub async fn list_api_keys(&self, user_id: Uuid) -> Result<Vec<ApiKey>, String> {
        let rows = sqlx::query_as::<_, ApiKeyRow>(
            r#"
            SELECT
                id, user_id, provider, encrypted_key, encryption_iv, encryption_tag,
                key_prefix, key_suffix, key_name, last_used_at, usage_count, is_active,
                created_at, updated_at
            FROM user_api_keys
            WHERE user_id = $1 AND is_active = true
            ORDER BY created_at DESC
            "#
        )
        .bind(&user_id)
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| format!("Failed to list API keys: {}", e))?;

        rows.into_iter()
            .map(|row| row.try_into())
            .collect()
    }

    pub async fn revoke_api_key(&self, user_id: Uuid, key_id: Uuid) -> Result<(), String> {
        let result = sqlx::query(
            "UPDATE user_api_keys SET is_active = false, updated_at = NOW() WHERE id = $1 AND user_id = $2"
        )
        .bind(&key_id)
        .bind(&user_id)
        .execute(&*self.pool)
        .await
        .map_err(|e| format!("Failed to revoke API key: {}", e))?;

        if result.rows_affected() == 0 {
            return Err("API key not found or already revoked".to_string());
        }

        Ok(())
    }

    pub async fn get_decrypted_keys(&self, user_id: Uuid) -> Result<Vec<DecryptedApiKey>, String> {
        let rows = sqlx::query_as::<_, ApiKeyRow>(
            r#"
            SELECT
                id, user_id, provider, encrypted_key, encryption_iv, encryption_tag,
                key_prefix, key_suffix, key_name, last_used_at, usage_count, is_active,
                created_at, updated_at
            FROM user_api_keys
            WHERE user_id = $1 AND is_active = true
            "#
        )
        .bind(&user_id)
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| format!("Failed to fetch API keys: {}", e))?;

        let mut decrypted_keys = Vec::new();

        for row in rows {
            let encrypted_data = EncryptedData {
                ciphertext: row.encrypted_key,
                iv: row.encryption_iv,
                tag: row.encryption_tag,
            };

            let decrypted_key = self
                .encryption
                .decrypt(&encrypted_data)
                .map_err(|e| format!("Decryption failed for provider {}: {}", row.provider, e))?;

            decrypted_keys.push(DecryptedApiKey {
                provider: row.provider,
                api_key: decrypted_key,
            });
        }

        Ok(decrypted_keys)
    }

    pub async fn increment_usage(&self, key_id: Uuid) -> Result<(), String> {
        sqlx::query(
            "UPDATE user_api_keys SET usage_count = usage_count + 1, last_used_at = NOW() WHERE id = $1"
        )
        .bind(&key_id)
        .execute(&*self.pool)
        .await
        .map_err(|e| format!("Failed to increment usage: {}", e))?;

        Ok(())
    }
}
