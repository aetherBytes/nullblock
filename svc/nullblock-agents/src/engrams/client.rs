use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEngramRequest {
    pub wallet_address: String,
    pub engram_type: String,
    pub key: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_public: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Engram {
    pub id: String,
    pub wallet_address: String,
    pub engram_type: String,
    pub key: String,
    pub content: String,
    #[serde(default)]
    pub metadata: Option<Value>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub is_public: bool,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub version: Option<i32>,
    #[serde(default)]
    pub parent_id: Option<String>,
    #[serde(default)]
    pub lineage_root_id: Option<String>,
    #[serde(default)]
    pub is_mintable: Option<bool>,
    #[serde(default)]
    pub nft_token_id: Option<String>,
    #[serde(default)]
    pub price_mon: Option<String>,
    #[serde(default)]
    pub royalty_percent: Option<i32>,
    #[serde(default)]
    pub priority: Option<i32>,
    #[serde(default)]
    pub ttl_seconds: Option<i64>,
    #[serde(default)]
    pub created_by: Option<String>,
    #[serde(default)]
    pub accessed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngramResponse {
    pub success: bool,
    pub data: Option<Engram>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngramListResponse {
    pub success: bool,
    pub data: Option<Vec<Engram>>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallet_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engram_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
}

pub struct EngramsClient {
    base_url: String,
    http_client: Client,
}

impl EngramsClient {
    pub fn new(base_url: String) -> Self {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            base_url,
            http_client,
        }
    }

    pub fn is_configured(&self) -> bool {
        !self.base_url.is_empty()
    }

    pub async fn create_engram(&self, request: CreateEngramRequest) -> Result<Engram, String> {
        if !self.is_configured() {
            return Err("Engrams service not configured".to_string());
        }

        let url = format!("{}/engrams", self.base_url);
        debug!("Creating engram: {} at {}", request.key, url);

        match self.http_client.post(&url).json(&request).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<EngramResponse>().await {
                        Ok(resp) => {
                            if resp.success {
                                if let Some(engram) = resp.data {
                                    info!("Created engram: {} ({})", engram.key, engram.id);
                                    Ok(engram)
                                } else {
                                    Err("Engrams service returned success but no data".to_string())
                                }
                            } else {
                                let err_msg =
                                    resp.error.unwrap_or_else(|| "Unknown error".to_string());
                                error!("Engrams service error: {}", err_msg);
                                Err(err_msg)
                            }
                        }
                        Err(e) => {
                            error!("Failed to parse engram response: {}", e);
                            Err(format!("Failed to parse response: {}", e))
                        }
                    }
                } else {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    error!("Engrams service error {}: {}", status, body);
                    Err(format!("Engrams service error {}: {}", status, body))
                }
            }
            Err(e) => {
                error!("Failed to connect to Engrams service: {}", e);
                Err(format!("Connection error: {}", e))
            }
        }
    }

    pub async fn get_engram_by_wallet_key(
        &self,
        wallet: &str,
        key: &str,
    ) -> Result<Option<Engram>, String> {
        if !self.is_configured() {
            return Err("Engrams service not configured".to_string());
        }

        let url = format!("{}/engrams/wallet/{}/{}", self.base_url, wallet, key);
        debug!("Fetching engram by wallet/key: {}/{}", wallet, key);

        match self.http_client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<EngramResponse>().await {
                        Ok(resp) => {
                            if resp.success && resp.data.is_some() {
                                return Ok(resp.data);
                            }
                            debug!(
                                "Direct lookup failed for {}/{}, trying search fallback",
                                wallet, key
                            );
                        }
                        Err(_) => {}
                    }
                }

                let search = SearchRequest {
                    wallet_address: Some(wallet.to_string()),
                    engram_type: None,
                    query: None,
                    tags: None,
                    limit: Some(100),
                    offset: None,
                };

                if let Ok(engrams) = self.search_engrams(search).await {
                    for engram in engrams {
                        if engram.key == key {
                            debug!("Found engram via search fallback: {} ({})", key, engram.id);
                            return Ok(Some(engram));
                        }
                    }
                }

                Ok(None)
            }
            Err(e) => {
                error!("Failed to connect to Engrams service: {}", e);
                Err(format!("Connection error: {}", e))
            }
        }
    }

    pub async fn search_engrams(&self, request: SearchRequest) -> Result<Vec<Engram>, String> {
        if !self.is_configured() {
            return Err("Engrams service not configured".to_string());
        }

        let url = format!("{}/engrams/search", self.base_url);
        debug!("Searching engrams with query: {:?}", request.query);

        match self.http_client.post(&url).json(&request).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<EngramListResponse>().await {
                        Ok(resp) => {
                            if resp.success {
                                let engrams = resp.data.unwrap_or_default();
                                debug!("Found {} engrams matching query", engrams.len());
                                Ok(engrams)
                            } else {
                                let err_msg =
                                    resp.error.unwrap_or_else(|| "Unknown error".to_string());
                                error!("Engrams search error: {}", err_msg);
                                Err(err_msg)
                            }
                        }
                        Err(e) => {
                            error!("Failed to parse search response: {}", e);
                            Err(format!("Failed to parse response: {}", e))
                        }
                    }
                } else {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    Err(format!("Engrams service error {}: {}", status, body))
                }
            }
            Err(e) => {
                error!("Failed to connect to Engrams service: {}", e);
                Err(format!("Connection error: {}", e))
            }
        }
    }

    pub async fn update_engram(
        &self,
        id: &str,
        content: &str,
        tags: Option<Vec<String>>,
    ) -> Result<Engram, String> {
        if !self.is_configured() {
            return Err("Engrams service not configured".to_string());
        }

        let url = format!("{}/engrams/{}", self.base_url, id);
        debug!("Updating engram: {} at {}", id, url);

        let body = serde_json::json!({
            "content": content,
            "tags": tags,
        });

        match self.http_client.put(&url).json(&body).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<EngramResponse>().await {
                        Ok(resp) => {
                            if resp.success {
                                if let Some(engram) = resp.data {
                                    info!("Updated engram: {} ({})", engram.key, engram.id);
                                    Ok(engram)
                                } else {
                                    Err("Engrams service returned success but no data".to_string())
                                }
                            } else {
                                let err_msg =
                                    resp.error.unwrap_or_else(|| "Unknown error".to_string());
                                error!("Engrams update error: {}", err_msg);
                                Err(err_msg)
                            }
                        }
                        Err(e) => {
                            error!("Failed to parse engram response: {}", e);
                            Err(format!("Failed to parse response: {}", e))
                        }
                    }
                } else {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    error!("Engrams service error {}: {}", status, body);
                    Err(format!("Engrams service error {}: {}", status, body))
                }
            }
            Err(e) => {
                error!("Failed to connect to Engrams service: {}", e);
                Err(format!("Connection error: {}", e))
            }
        }
    }

    pub async fn delete_engram(&self, id: &str) -> Result<(), String> {
        if !self.is_configured() {
            return Err("Engrams service not configured".to_string());
        }

        let url = format!("{}/engrams/{}", self.base_url, id);
        debug!("Deleting engram: {} at {}", id, url);

        match self.http_client.delete(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    info!("Deleted engram: {}", id);
                    Ok(())
                } else {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    error!("Engrams delete error {}: {}", status, body);
                    Err(format!("Engrams service error {}: {}", status, body))
                }
            }
            Err(e) => {
                error!("Failed to connect to Engrams service: {}", e);
                Err(format!("Connection error: {}", e))
            }
        }
    }

    pub async fn get_engram_by_id(&self, id: &str) -> Result<Option<Engram>, String> {
        if !self.is_configured() {
            return Err("Engrams service not configured".to_string());
        }

        let url = format!("{}/engrams/{}", self.base_url, id);
        debug!("Fetching engram by id: {}", id);

        match self.http_client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<EngramResponse>().await {
                        Ok(resp) => {
                            if resp.success {
                                Ok(resp.data)
                            } else {
                                Ok(None)
                            }
                        }
                        Err(_) => Ok(None),
                    }
                } else if response.status() == reqwest::StatusCode::NOT_FOUND {
                    Ok(None)
                } else {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    Err(format!("Engrams service error {}: {}", status, body))
                }
            }
            Err(e) => {
                error!("Failed to connect to Engrams service: {}", e);
                Err(format!("Connection error: {}", e))
            }
        }
    }

    pub async fn upsert_engram(&self, request: CreateEngramRequest) -> Result<Engram, String> {
        if let Ok(Some(existing)) = self
            .get_engram_by_wallet_key(&request.wallet_address, &request.key)
            .await
        {
            return self
                .update_engram(&existing.id, &request.content, request.tags)
                .await;
        }

        match self.create_engram(request.clone()).await {
            Ok(engram) => Ok(engram),
            Err(e) if e.contains("duplicate key") || e.contains("unique constraint") => {
                warn!("Engram already exists but wasn't found initially - retrying fetch");
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;

                if let Ok(Some(existing)) = self
                    .get_engram_by_wallet_key(&request.wallet_address, &request.key)
                    .await
                {
                    return self
                        .update_engram(&existing.id, &request.content, request.tags)
                        .await;
                }

                warn!(
                    "Engram {} still not found after duplicate key error - treating as success",
                    request.key
                );
                Ok(Engram {
                    id: String::new(),
                    wallet_address: request.wallet_address,
                    engram_type: request.engram_type,
                    key: request.key,
                    content: request.content,
                    metadata: request.metadata,
                    tags: request.tags.unwrap_or_default(),
                    is_public: request.is_public.unwrap_or(false),
                    created_at: chrono::Utc::now().to_rfc3339(),
                    updated_at: chrono::Utc::now().to_rfc3339(),
                    summary: None,
                    version: None,
                    parent_id: None,
                    lineage_root_id: None,
                    is_mintable: None,
                    nft_token_id: None,
                    price_mon: None,
                    royalty_percent: None,
                    priority: None,
                    ttl_seconds: None,
                    created_by: None,
                    accessed_at: None,
                })
            }
            Err(e) => Err(e),
        }
    }
}
