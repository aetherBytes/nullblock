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

    pub async fn get_agent_api_key(
        &self,
        agent_name: &str,
        provider: &str,
    ) -> Result<Option<String>, String> {
        let url = format!(
            "{}/internal/agents/{}/api-keys/{}/decrypted",
            self.base_url, agent_name, provider
        );

        info!(
            "🔑 Fetching API key for agent '{}' provider '{}' from Erebus",
            agent_name, provider
        );

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| format!("Failed to connect to Erebus: {}", e))?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            warn!(
                "⚠️ No API key found for agent '{}' provider '{}'",
                agent_name, provider
            );
            return Ok(None);
        }

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("Erebus returned error {}: {}", status, error_text));
        }

        let data: DecryptedAgentApiKeyResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if data.success {
            if let Some(api_key) = data.api_key {
                let prefix = if api_key.len() > 10 {
                    &api_key[..10]
                } else {
                    &api_key
                };
                let suffix = if api_key.len() > 10 {
                    &api_key[api_key.len() - 4..]
                } else {
                    ""
                };
                info!(
                    "✅ Retrieved API key for agent '{}' ({}...{})",
                    agent_name, prefix, suffix
                );
                return Ok(Some(api_key));
            }
        }

        if let Some(error) = data.error {
            warn!("⚠️ Erebus error: {}", error);
        }

        Ok(None)
    }

    pub async fn get_openrouter_key(&self) -> Option<String> {
        match self.get_agent_api_key("arb-farm", "openrouter").await {
            Ok(Some(key)) => Some(key),
            Ok(None) => {
                warn!("⚠️ No OpenRouter API key found for arb-farm agent in DB");
                None
            }
            Err(e) => {
                warn!("⚠️ Failed to fetch OpenRouter API key from Erebus: {}", e);
                None
            }
        }
    }
}
