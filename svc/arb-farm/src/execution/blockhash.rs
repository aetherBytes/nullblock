use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use crate::error::{AppError, AppResult};

pub struct BlockhashCache {
    client: reqwest::Client,
    rpc_url: String,
    cached: Arc<RwLock<CachedBlockhash>>,
    ttl: Duration,
}

#[derive(Debug, Clone, Default)]
struct CachedBlockhash {
    blockhash: String,
    last_valid_block_height: u64,
    fetched_at: Option<Instant>,
}

impl BlockhashCache {
    pub fn new(rpc_url: String) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client"),
            rpc_url,
            cached: Arc::new(RwLock::new(CachedBlockhash::default())),
            ttl: Duration::from_secs(10), // Blockhashes are valid for ~60 seconds, refresh every 10
        }
    }

    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }

    pub async fn get_blockhash(&self) -> AppResult<RecentBlockhash> {
        // Check cache first
        {
            let cached = self.cached.read().await;
            if let Some(fetched_at) = cached.fetched_at {
                if fetched_at.elapsed() < self.ttl && !cached.blockhash.is_empty() {
                    return Ok(RecentBlockhash {
                        blockhash: cached.blockhash.clone(),
                        last_valid_block_height: cached.last_valid_block_height,
                    });
                }
            }
        }

        // Fetch fresh blockhash
        let fresh = self.fetch_blockhash().await?;

        // Update cache
        {
            let mut cached = self.cached.write().await;
            cached.blockhash = fresh.blockhash.clone();
            cached.last_valid_block_height = fresh.last_valid_block_height;
            cached.fetched_at = Some(Instant::now());
        }

        Ok(fresh)
    }

    async fn fetch_blockhash(&self) -> AppResult<RecentBlockhash> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getLatestBlockhash",
            "params": [{
                "commitment": "confirmed"
            }]
        });

        let response = self
            .client
            .post(&self.rpc_url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("RPC request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::ExternalApi(format!(
                "RPC returned error status: {}",
                response.status()
            )));
        }

        let rpc_response: RpcBlockhashResponse = response
            .json()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to parse RPC response: {}", e)))?;

        if let Some(error) = rpc_response.error {
            return Err(AppError::ExternalApi(format!(
                "RPC error: {} - {}",
                error.code, error.message
            )));
        }

        let result = rpc_response.result.ok_or_else(|| {
            AppError::ExternalApi("Missing result in RPC response".to_string())
        })?;

        Ok(RecentBlockhash {
            blockhash: result.value.blockhash,
            last_valid_block_height: result.value.last_valid_block_height,
        })
    }

    pub async fn invalidate(&self) {
        let mut cached = self.cached.write().await;
        cached.fetched_at = None;
    }
}

#[derive(Debug, Clone)]
pub struct RecentBlockhash {
    pub blockhash: String,
    pub last_valid_block_height: u64,
}

#[derive(Debug, serde::Deserialize)]
struct RpcBlockhashResponse {
    result: Option<RpcBlockhashResult>,
    error: Option<RpcError>,
}

#[derive(Debug, serde::Deserialize)]
struct RpcBlockhashResult {
    value: BlockhashValue,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct BlockhashValue {
    blockhash: String,
    last_valid_block_height: u64,
}

#[derive(Debug, serde::Deserialize)]
struct RpcError {
    code: i64,
    message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_blockhash_cache_creation() {
        let cache = BlockhashCache::new("https://api.mainnet-beta.solana.com".to_string());
        // Just verify it can be created without panic
        assert_eq!(cache.ttl, Duration::from_secs(10));
    }

    #[tokio::test]
    async fn test_cache_with_custom_ttl() {
        let cache = BlockhashCache::new("https://api.mainnet-beta.solana.com".to_string())
            .with_ttl(Duration::from_secs(5));
        assert_eq!(cache.ttl, Duration::from_secs(5));
    }
}
