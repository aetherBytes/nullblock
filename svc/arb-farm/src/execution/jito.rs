use reqwest::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, AppResult};

pub struct JitoClient {
    client: Client,
    block_engine_url: String,
    auth_keypair: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleSubmission {
    pub id: Uuid,
    pub transactions: Vec<String>, // Base58 encoded transactions
    pub tip_lamports: u64,
    pub submitted_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleStatus {
    pub bundle_id: String,
    pub status: BundleState,
    pub landed_slot: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BundleState {
    Pending,
    Landed,
    Failed,
    Dropped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JitoTipAccount {
    pub address: String,
    pub balance_lamports: u64,
}

#[derive(Debug, Serialize)]
struct SendBundleRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: Vec<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct SendBundleResponse {
    result: Option<String>,
    error: Option<JitoError>,
}

#[derive(Debug, Deserialize)]
struct JitoError {
    code: i64,
    message: String,
}

#[derive(Debug, Serialize)]
struct GetBundleStatusRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct GetBundleStatusResponse {
    result: Option<BundleStatusResult>,
    error: Option<JitoError>,
}

#[derive(Debug, Deserialize)]
struct BundleStatusResult {
    bundle_id: String,
    status: String,
    landed_slot: Option<u64>,
}

impl JitoClient {
    pub fn new(block_engine_url: String, auth_keypair: Option<String>) -> Self {
        Self {
            client: Client::new(),
            block_engine_url,
            auth_keypair,
        }
    }

    pub async fn get_tip_accounts(&self) -> AppResult<Vec<JitoTipAccount>> {
        let url = format!("{}/api/v1/bundles/tip_accounts", self.block_engine_url);

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Jito tip accounts request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::ExternalApi(format!(
                "Jito returned error: {}",
                response.status()
            )));
        }

        let accounts: Vec<String> = response
            .json()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to parse Jito response: {}", e)))?;

        Ok(accounts
            .into_iter()
            .map(|addr| JitoTipAccount {
                address: addr,
                balance_lamports: 0,
            })
            .collect())
    }

    pub async fn send_bundle(&self, transactions: Vec<String>, tip_lamports: u64) -> AppResult<BundleSubmission> {
        let bundle_id = Uuid::new_v4();

        let request = SendBundleRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "sendBundle".to_string(),
            params: vec![transactions.clone()],
        };

        let url = format!("{}/api/v1/bundles", self.block_engine_url);

        let response = self
            .client
            .post(&url)
            .json(&request)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| AppError::Execution(format!("Jito bundle submission failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::Execution(format!(
                "Jito bundle returned error status: {}",
                response.status()
            )));
        }

        let result: SendBundleResponse = response
            .json()
            .await
            .map_err(|e| AppError::Execution(format!("Failed to parse Jito response: {}", e)))?;

        if let Some(error) = result.error {
            return Err(AppError::Execution(format!(
                "Jito bundle error {}: {}",
                error.code, error.message
            )));
        }

        Ok(BundleSubmission {
            id: bundle_id,
            transactions,
            tip_lamports,
            submitted_at: chrono::Utc::now(),
        })
    }

    pub async fn send_bundle_fast(&self, transactions: &[Vec<u8>]) -> AppResult<String> {
        let encoded: Vec<String> = transactions
            .iter()
            .map(|tx| bs58::encode(tx).into_string())
            .collect();

        let default_tip = 100_000;
        let result = self.send_bundle(encoded, default_tip).await?;
        Ok(result.id.to_string())
    }

    pub async fn get_bundle_status(&self, bundle_id: &str) -> AppResult<BundleStatus> {
        let request = GetBundleStatusRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "getBundleStatus".to_string(),
            params: vec![bundle_id.to_string()],
        };

        let url = format!("{}/api/v1/bundles", self.block_engine_url);

        let response = self
            .client
            .post(&url)
            .json(&request)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Jito status request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::ExternalApi(format!(
                "Jito status returned error: {}",
                response.status()
            )));
        }

        let result: GetBundleStatusResponse = response
            .json()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to parse status response: {}", e)))?;

        if let Some(error) = result.error {
            return Err(AppError::ExternalApi(format!(
                "Jito status error {}: {}",
                error.code, error.message
            )));
        }

        if let Some(status_result) = result.result {
            let state = match status_result.status.as_str() {
                "Pending" => BundleState::Pending,
                "Landed" => BundleState::Landed,
                "Failed" => BundleState::Failed,
                _ => BundleState::Dropped,
            };

            Ok(BundleStatus {
                bundle_id: status_result.bundle_id,
                status: state,
                landed_slot: status_result.landed_slot,
                error: None,
            })
        } else {
            Ok(BundleStatus {
                bundle_id: bundle_id.to_string(),
                status: BundleState::Pending,
                landed_slot: None,
                error: None,
            })
        }
    }

    pub async fn wait_for_bundle(
        &self,
        bundle_id: &str,
        timeout_secs: u64,
    ) -> AppResult<BundleStatus> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(timeout_secs);

        loop {
            if start.elapsed() > timeout {
                return Err(AppError::Execution(format!(
                    "Bundle {} timed out after {}s",
                    bundle_id, timeout_secs
                )));
            }

            let status = self.get_bundle_status(bundle_id).await?;

            match status.status {
                BundleState::Landed | BundleState::Failed | BundleState::Dropped => {
                    return Ok(status);
                }
                BundleState::Pending => {
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleConfig {
    pub max_tip_lamports: u64,
    pub base_tip_lamports: u64,
    pub tip_percentage_of_profit: f64,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
}

impl Default for BundleConfig {
    fn default() -> Self {
        Self {
            max_tip_lamports: 100_000, // 0.0001 SOL max tip
            base_tip_lamports: 1_000,  // 0.000001 SOL base tip
            tip_percentage_of_profit: 0.1, // 10% of profit as tip
            max_retries: 3,
            retry_delay_ms: 1000,
        }
    }
}

impl BundleConfig {
    pub fn calculate_tip(&self, estimated_profit_lamports: i64) -> u64 {
        let profit_based_tip = (estimated_profit_lamports as f64 * self.tip_percentage_of_profit) as u64;
        let tip = self.base_tip_lamports.max(profit_based_tip);
        tip.min(self.max_tip_lamports)
    }
}
