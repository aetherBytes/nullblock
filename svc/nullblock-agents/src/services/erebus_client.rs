// Erebus client for API key and user management
// Contains scaffolding for future API key decryption features

#![allow(dead_code)]

use crate::error::{AppError, AppResult};
use serde::Deserialize;
use tracing::{info, warn};

#[derive(Debug, Clone, Deserialize)]
pub struct DecryptedAgentApiKeyResponse {
    pub success: bool,
    pub agent_name: String,
    pub provider: String,
    pub api_key: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitStatus {
    pub allowed: bool,
    pub remaining: i32,
    pub limit: i32,
    pub resets_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitResponse {
    pub success: bool,
    pub data: Option<RateLimitStatus>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HasApiKeyResponse {
    pub success: bool,
    pub has_key: bool,
    pub provider: Option<String>,
    pub error: Option<String>,
}

#[derive(Clone)]
pub struct ErebusClient {
    base_url: String,
    client: reqwest::Client,
}

impl ErebusClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_agent_api_key(&self, agent_name: &str, provider: &str) -> AppResult<Option<String>> {
        let url = format!(
            "{}/internal/agents/{}/api-keys/{}/decrypted",
            self.base_url, agent_name, provider
        );

        info!("🔑 Fetching API key for agent '{}' provider '{}' from Erebus", agent_name, provider);

        let response = self.client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| AppError::ApiKeyResolutionFailed(format!("Failed to connect to Erebus: {}", e)))?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            warn!("⚠️ No API key found for agent '{}' provider '{}'", agent_name, provider);
            return Ok(None);
        }

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ApiKeyResolutionFailed(format!(
                "Erebus returned error {}: {}",
                status, error_text
            )));
        }

        let data: DecryptedAgentApiKeyResponse = response
            .json()
            .await
            .map_err(|e| AppError::ApiKeyResolutionFailed(format!("Failed to parse response: {}", e)))?;

        if data.success {
            if let Some(api_key) = data.api_key {
                info!("✅ Retrieved API key for agent '{}' ({}...{})",
                    agent_name,
                    &api_key[..10.min(api_key.len())],
                    if api_key.len() > 10 { &api_key[api_key.len()-4..] } else { "" }
                );
                return Ok(Some(api_key));
            }
        }

        if let Some(error) = data.error {
            warn!("⚠️ Erebus error: {}", error);
        }

        Ok(None)
    }

    pub async fn check_rate_limit(&self, user_id: &str, agent_name: &str) -> AppResult<RateLimitStatus> {
        let url = format!(
            "{}/internal/users/{}/rate-limit/{}",
            self.base_url, user_id, agent_name
        );

        let response = self.client
            .get(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| AppError::RateLimitCheckFailed(format!("Failed to connect to Erebus: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::RateLimitCheckFailed(format!(
                "Erebus returned error {}: {}",
                status, error_text
            )));
        }

        let data: RateLimitResponse = response
            .json()
            .await
            .map_err(|e| AppError::RateLimitCheckFailed(format!("Failed to parse response: {}", e)))?;

        if data.success {
            if let Some(status) = data.data {
                return Ok(status);
            }
        }

        Err(AppError::RateLimitCheckFailed(
            data.error.unwrap_or_else(|| "Unknown error".to_string())
        ))
    }

    pub async fn increment_rate_limit(&self, user_id: &str, agent_name: &str) -> AppResult<RateLimitStatus> {
        let url = format!(
            "{}/internal/users/{}/rate-limit/{}/increment",
            self.base_url, user_id, agent_name
        );

        let response = self.client
            .post(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| AppError::RateLimitCheckFailed(format!("Failed to connect to Erebus: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::RateLimitCheckFailed(format!(
                "Erebus returned error {}: {}",
                status, error_text
            )));
        }

        let data: RateLimitResponse = response
            .json()
            .await
            .map_err(|e| AppError::RateLimitCheckFailed(format!("Failed to parse response: {}", e)))?;

        if data.success {
            if let Some(status) = data.data {
                return Ok(status);
            }
        }

        Err(AppError::RateLimitCheckFailed(
            data.error.unwrap_or_else(|| "Unknown error".to_string())
        ))
    }

    pub async fn user_has_api_key(&self, user_id: &str, provider: &str) -> AppResult<bool> {
        let url = format!(
            "{}/internal/users/{}/has-api-key/{}",
            self.base_url, user_id, provider
        );

        let response = self.client
            .get(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| AppError::ApiKeyResolutionFailed(format!("Failed to connect to Erebus: {}", e)))?;

        if !response.status().is_success() {
            return Ok(false);
        }

        let data: HasApiKeyResponse = response
            .json()
            .await
            .map_err(|e| AppError::ApiKeyResolutionFailed(format!("Failed to parse response: {}", e)))?;

        Ok(data.success && data.has_key)
    }

    pub async fn get_user_decrypted_keys(&self, user_id: &str) -> AppResult<Vec<DecryptedApiKey>> {
        let url = format!(
            "{}/internal/users/{}/api-keys/decrypted",
            self.base_url, user_id
        );

        let response = self.client
            .get(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| AppError::ApiKeyResolutionFailed(format!("Failed to connect to Erebus: {}", e)))?;

        if !response.status().is_success() {
            return Ok(vec![]);
        }

        let data: DecryptedUserApiKeysResponse = response
            .json()
            .await
            .map_err(|e| AppError::ApiKeyResolutionFailed(format!("Failed to parse response: {}", e)))?;

        if data.success {
            return Ok(data.data.unwrap_or_default());
        }

        Ok(vec![])
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DecryptedApiKey {
    pub provider: String,
    pub api_key: String,
}

#[derive(Debug, Clone, Deserialize)]
struct DecryptedUserApiKeysResponse {
    pub success: bool,
    pub data: Option<Vec<DecryptedApiKey>>,
    pub error: Option<String>,
}
