use rand::Rng;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::error::{AppError, AppResult};

const BUNDLE_RETRY_BASE_DELAY_MS: u64 = 500;
const BUNDLE_RETRY_MAX_DELAY_MS: u64 = 4000;
const BUNDLE_MAX_RETRIES: u32 = 3;

fn calculate_backoff_with_jitter(attempt: u32) -> u64 {
    let base_delay = (BUNDLE_RETRY_BASE_DELAY_MS * (1 << attempt)).min(BUNDLE_RETRY_MAX_DELAY_MS);
    let mut rng = rand::thread_rng();
    let jitter_factor: f64 = rng.gen_range(0.5..1.5);
    ((base_delay as f64) * jitter_factor) as u64
}

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
            .map_err(|e| {
                AppError::ExternalApi(format!("Jito tip accounts request failed: {}", e))
            })?;

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

    pub async fn send_bundle(
        &self,
        transactions: Vec<String>,
        tip_lamports: u64,
    ) -> AppResult<BundleSubmission> {
        self.send_bundle_with_retry(transactions, tip_lamports, BUNDLE_MAX_RETRIES)
            .await
    }

    pub async fn send_bundle_with_retry(
        &self,
        transactions: Vec<String>,
        tip_lamports: u64,
        max_retries: u32,
    ) -> AppResult<BundleSubmission> {
        let bundle_id = Uuid::new_v4();
        let url = format!("{}/api/v1/bundles", self.block_engine_url);
        let mut last_error = String::new();

        for attempt in 0..=max_retries {
            if attempt > 0 {
                let delay_ms = calculate_backoff_with_jitter(attempt);
                warn!(
                    bundle_id = %bundle_id,
                    attempt = attempt,
                    delay_ms = delay_ms,
                    "Retrying Jito bundle submission after error: {}",
                    last_error
                );
                tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
            }

            let request = SendBundleRequest {
                jsonrpc: "2.0".to_string(),
                id: 1,
                method: "sendBundle".to_string(),
                params: vec![transactions.clone()],
            };

            let response = match self
                .client
                .post(&url)
                .json(&request)
                .timeout(std::time::Duration::from_secs(30))
                .send()
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    last_error = format!("Jito bundle submission failed: {}", e);
                    error!(bundle_id = %bundle_id, attempt = attempt, "{}", last_error);
                    continue;
                }
            };

            if !response.status().is_success() {
                last_error = format!("Jito bundle returned error status: {}", response.status());
                error!(bundle_id = %bundle_id, attempt = attempt, "{}", last_error);
                continue;
            }

            let result: SendBundleResponse = match response.json().await {
                Ok(r) => r,
                Err(e) => {
                    last_error = format!("Failed to parse Jito response: {}", e);
                    error!(bundle_id = %bundle_id, attempt = attempt, "{}", last_error);
                    continue;
                }
            };

            if let Some(error) = result.error {
                last_error = format!("Jito bundle error {}: {}", error.code, error.message);
                error!(bundle_id = %bundle_id, attempt = attempt, "{}", last_error);

                // Don't retry on bundle-level errors that indicate the tx was processed or is invalid
                // These errors mean retrying with the same transaction won't help
                let error_msg_lower = error.message.to_lowercase();
                let is_non_retryable = error_msg_lower.contains("duplicate")
                    || error_msg_lower.contains("already processed")
                    || error_msg_lower.contains("already been processed")
                    || error_msg_lower.contains("blockhash not found")
                    || error_msg_lower.contains("blockhash expired")
                    || error_msg_lower.contains("invalid transaction")
                    || error_msg_lower.contains("signature verification")
                    || error.code == -32002  // Transaction simulation failed
                    || error.code == -32003; // Transaction precompile verification failed

                if is_non_retryable {
                    warn!(
                        bundle_id = %bundle_id,
                        "Jito error is non-retryable (tx needs fresh blockhash): {}",
                        error.message
                    );
                    return Err(AppError::Execution(format!(
                        "Jito bundle rejected (non-retryable): {} - {}",
                        error.code, error.message
                    )));
                }

                continue;
            }

            info!(
                bundle_id = %bundle_id,
                tip_lamports = tip_lamports,
                tx_count = transactions.len(),
                "Jito bundle submitted successfully"
            );

            return Ok(BundleSubmission {
                id: bundle_id,
                transactions,
                tip_lamports,
                submitted_at: chrono::Utc::now(),
            });
        }

        Err(AppError::Execution(format!(
            "Jito bundle submission failed after {} retries: {}",
            max_retries, last_error
        )))
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

        let result: GetBundleStatusResponse = response.json().await.map_err(|e| {
            AppError::ExternalApi(format!("Failed to parse status response: {}", e))
        })?;

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
        let mut poll_interval_ms: u64 = 500;
        let max_poll_interval_ms: u64 = 2000;
        let mut poll_count: u32 = 0;
        const MAX_POLLS: u32 = 120; // Safety limit

        debug!(bundle_id = %bundle_id, timeout_secs = timeout_secs, "Waiting for bundle status");

        loop {
            if start.elapsed() > timeout {
                warn!(bundle_id = %bundle_id, poll_count = poll_count, "Bundle timed out");
                return Err(AppError::Execution(format!(
                    "Bundle {} timed out after {}s",
                    bundle_id, timeout_secs
                )));
            }

            if poll_count >= MAX_POLLS {
                warn!(bundle_id = %bundle_id, "Bundle exceeded max poll count");
                return Err(AppError::Execution(format!(
                    "Bundle {} exceeded max poll attempts ({})",
                    bundle_id, MAX_POLLS
                )));
            }

            let status = match self.get_bundle_status(bundle_id).await {
                Ok(s) => s,
                Err(e) => {
                    warn!(bundle_id = %bundle_id, poll_count = poll_count, "Failed to get bundle status: {}", e);
                    poll_count += 1;
                    tokio::time::sleep(std::time::Duration::from_millis(poll_interval_ms)).await;
                    poll_interval_ms = (poll_interval_ms * 3 / 2).min(max_poll_interval_ms);
                    continue;
                }
            };

            match status.status {
                BundleState::Landed => {
                    info!(bundle_id = %bundle_id, slot = ?status.landed_slot, "Bundle landed");
                    return Ok(status);
                }
                BundleState::Failed => {
                    warn!(bundle_id = %bundle_id, error = ?status.error, "Bundle failed");
                    return Ok(status);
                }
                BundleState::Dropped => {
                    warn!(bundle_id = %bundle_id, "Bundle dropped");
                    return Ok(status);
                }
                BundleState::Pending => {
                    poll_count += 1;
                    tokio::time::sleep(std::time::Duration::from_millis(poll_interval_ms)).await;
                    poll_interval_ms = (poll_interval_ms * 3 / 2).min(max_poll_interval_ms);
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
            max_tip_lamports: 5_000_000, // 0.005 SOL max tip (increased for competitive bundle inclusion)
            base_tip_lamports: 50_000,   // 0.00005 SOL base tip
            tip_percentage_of_profit: 0.15, // 15% of profit as tip
            max_retries: 3,
            retry_delay_ms: 1000,
        }
    }
}

impl BundleConfig {
    pub fn calculate_tip(&self, estimated_profit_lamports: i64) -> u64 {
        let profit_based_tip =
            (estimated_profit_lamports as f64 * self.tip_percentage_of_profit) as u64;
        let tip = self.base_tip_lamports.max(profit_based_tip);
        tip.min(self.max_tip_lamports)
    }
}
