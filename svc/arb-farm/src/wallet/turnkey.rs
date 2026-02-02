use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::policy::{ArbFarmPolicy, DailyUsage, PolicyViolation};
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnkeyConfig {
    pub api_url: String,
    pub organization_id: String,
    pub api_public_key: Option<String>,
    pub api_private_key: Option<String>,
}

impl Default for TurnkeyConfig {
    fn default() -> Self {
        Self {
            api_url: "https://api.turnkey.com".to_string(),
            organization_id: String::new(),
            api_public_key: None,
            api_private_key: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletStatus {
    pub is_connected: bool,
    pub wallet_address: Option<String>,
    pub turnkey_wallet_id: Option<String>,
    pub balance_lamports: Option<u64>,
    pub daily_usage: DailyUsage,
    pub policy: ArbFarmPolicy,
    pub delegation_status: DelegationStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DelegationStatus {
    NotConfigured,
    Pending,
    Active,
    Revoked,
    Error,
}

impl Default for WalletStatus {
    fn default() -> Self {
        Self {
            is_connected: false,
            wallet_address: None,
            turnkey_wallet_id: None,
            balance_lamports: None,
            daily_usage: DailyUsage::new(),
            policy: ArbFarmPolicy::default(),
            delegation_status: DelegationStatus::NotConfigured,
        }
    }
}

impl WalletStatus {
    pub fn dev() -> Self {
        Self {
            policy: ArbFarmPolicy::dev_testing(),
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignRequest {
    pub transaction_base64: String,
    pub estimated_amount_lamports: u64,
    pub estimated_profit_lamports: Option<i64>,
    pub edge_id: Option<uuid::Uuid>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignResult {
    pub success: bool,
    pub signed_transaction_base64: Option<String>,
    pub signature: Option<String>,
    pub error: Option<String>,
    pub policy_violation: Option<PolicyViolation>,
}

impl SignResult {
    pub fn success(signed_tx: String, signature: String) -> Self {
        Self {
            success: true,
            signed_transaction_base64: Some(signed_tx),
            signature: Some(signature),
            error: None,
            policy_violation: None,
        }
    }

    pub fn policy_error(violation: PolicyViolation) -> Self {
        Self {
            success: false,
            signed_transaction_base64: None,
            signature: None,
            error: Some(violation.message.clone()),
            policy_violation: Some(violation),
        }
    }

    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            signed_transaction_base64: None,
            signature: None,
            error: Some(msg.into()),
            policy_violation: None,
        }
    }
}

pub struct TurnkeySigner {
    client: reqwest::Client,
    config: TurnkeyConfig,
    wallet_status: Arc<RwLock<WalletStatus>>,
}

impl TurnkeySigner {
    pub fn new(config: TurnkeyConfig) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            config,
            wallet_status: Arc::new(RwLock::new(WalletStatus::default())),
        }
    }

    pub fn new_dev(config: TurnkeyConfig) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            config,
            wallet_status: Arc::new(RwLock::new(WalletStatus::dev())),
        }
    }

    pub async fn get_status(&self) -> WalletStatus {
        self.wallet_status.read().await.clone()
    }

    pub async fn set_wallet(
        &self,
        wallet_address: String,
        turnkey_wallet_id: String,
    ) -> AppResult<()> {
        let mut status = self.wallet_status.write().await;
        status.wallet_address = Some(wallet_address);
        status.turnkey_wallet_id = Some(turnkey_wallet_id);
        status.is_connected = true;
        status.delegation_status = DelegationStatus::Active;
        Ok(())
    }

    pub async fn update_policy(&self, policy: ArbFarmPolicy) -> AppResult<()> {
        let mut status = self.wallet_status.write().await;
        status.policy = policy;
        Ok(())
    }

    pub async fn update_balance(&self, balance_lamports: u64) -> AppResult<()> {
        let mut status = self.wallet_status.write().await;
        status.balance_lamports = Some(balance_lamports);
        Ok(())
    }

    pub async fn validate_transaction(&self, request: &SignRequest) -> Result<(), PolicyViolation> {
        let status = self.wallet_status.read().await;
        let policy = &status.policy;

        // Check amount limit
        if request.estimated_amount_lamports > policy.max_transaction_amount_lamports {
            return Err(PolicyViolation::amount_exceeded(
                request.estimated_amount_lamports,
                policy.max_transaction_amount_lamports,
            ));
        }

        // Check daily usage
        let mut daily_usage = status.daily_usage.clone();
        daily_usage.reset_if_new_day();
        daily_usage.can_execute(request.estimated_amount_lamports, policy)?;

        // Check profit threshold if applicable
        if let Some(profit) = request.estimated_profit_lamports {
            if profit < policy.min_profit_threshold_lamports as i64 && profit > 0 {
                return Err(PolicyViolation {
                    violation_type: super::policy::PolicyViolationType::ProfitBelowThreshold,
                    message: format!(
                        "Estimated profit {} lamports below threshold of {} lamports",
                        profit, policy.min_profit_threshold_lamports
                    ),
                    details: Some(serde_json::json!({
                        "profit": profit,
                        "threshold": policy.min_profit_threshold_lamports,
                    })),
                });
            }
        }

        Ok(())
    }

    pub async fn sign_transaction(&self, request: SignRequest) -> AppResult<SignResult> {
        // Validate against policy first
        if let Err(violation) = self.validate_transaction(&request).await {
            return Ok(SignResult::policy_error(violation));
        }

        let status = self.wallet_status.read().await;

        // Check wallet is connected
        if !status.is_connected {
            return Ok(SignResult::error("Wallet not connected"));
        }

        let wallet_id = match &status.turnkey_wallet_id {
            Some(id) => id.clone(),
            None => return Ok(SignResult::error("No Turnkey wallet configured")),
        };

        drop(status);

        // Call Turnkey API to sign
        let sign_result = self
            .call_turnkey_sign(&wallet_id, &request.transaction_base64)
            .await?;

        // Record the transaction in daily usage
        if sign_result.success {
            let mut status = self.wallet_status.write().await;
            status.daily_usage.reset_if_new_day();
            status
                .daily_usage
                .record_transaction(request.estimated_amount_lamports);
        }

        Ok(sign_result)
    }

    async fn call_turnkey_sign(
        &self,
        wallet_id: &str,
        transaction_base64: &str,
    ) -> AppResult<SignResult> {
        // Check if API keys are configured
        let (api_public_key, api_private_key) = {
            match (&self.config.api_public_key, &self.config.api_private_key) {
                (Some(pub_key), Some(priv_key)) => (pub_key.clone(), priv_key.clone()),
                _ => {
                    // API keys not configured - return error instead of fake success
                    tracing::error!(
                        "❌ CRITICAL: Turnkey API keys not configured - cannot sign transactions"
                    );
                    return Ok(SignResult {
                        success: false,
                        signed_transaction_base64: None,
                        signature: None,
                        error: Some("Turnkey API keys not configured - cannot sign transactions. Set TURNKEY_API_PUBLIC_KEY and TURNKEY_API_PRIVATE_KEY.".to_string()),
                        policy_violation: None,
                    });
                }
            }
        };

        // Build Turnkey sign request
        let turnkey_request = serde_json::json!({
            "type": "ACTIVITY_TYPE_SIGN_RAW_PAYLOAD",
            "organizationId": self.config.organization_id,
            "parameters": {
                "signWith": wallet_id,
                "payload": transaction_base64,
                "encoding": "PAYLOAD_ENCODING_BASE64",
                "hashFunction": "HASH_FUNCTION_NO_OP",
            },
            "timestampMs": chrono::Utc::now().timestamp_millis().to_string(),
        });

        // Sign the request with Turnkey API stamp
        let request_body = serde_json::to_string(&turnkey_request)
            .map_err(|e| AppError::Internal(format!("Failed to serialize request: {}", e)))?;

        let stamp = self.create_turnkey_stamp(&request_body, &api_public_key, &api_private_key)?;

        let response = self
            .client
            .post(format!(
                "{}/public/v1/submit/sign_raw_payload",
                self.config.api_url
            ))
            .header("Content-Type", "application/json")
            .header("X-Stamp", stamp)
            .body(request_body)
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Turnkey API error: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Ok(SignResult::error(format!(
                "Turnkey API error: {}",
                error_text
            )));
        }

        let result: TurnkeySignResponse = response.json().await.map_err(|e| {
            AppError::ExternalApi(format!("Failed to parse Turnkey response: {}", e))
        })?;

        match result.activity.status.as_str() {
            "ACTIVITY_STATUS_COMPLETED" => {
                let signed_tx = result
                    .activity
                    .result
                    .and_then(|r| r.sign_raw_payload_result)
                    .map(|r| r.encoded_signature)
                    .unwrap_or_default();

                Ok(SignResult::success(signed_tx.clone(), signed_tx))
            }
            "ACTIVITY_STATUS_FAILED" => Ok(SignResult::error("Turnkey signing failed")),
            status => Ok(SignResult::error(format!(
                "Unexpected Turnkey status: {}",
                status
            ))),
        }
    }

    fn create_turnkey_stamp(
        &self,
        _body: &str,
        _public_key: &str,
        _private_key: &str,
    ) -> AppResult<String> {
        // Turnkey uses a specific stamping mechanism
        // For now, return a placeholder - full implementation requires their SDK
        // In production, this would use P-256 ECDSA signing
        Ok(format!(
            r#"{{"publicKey":"{}","scheme":"SIGNATURE_SCHEME_TK_API_P256","signature":"placeholder"}}"#,
            _public_key
        ))
    }

    pub async fn disconnect(&self) -> AppResult<()> {
        let mut status = self.wallet_status.write().await;
        status.is_connected = false;
        status.wallet_address = None;
        status.turnkey_wallet_id = None;
        status.delegation_status = DelegationStatus::Revoked;
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct TurnkeySignResponse {
    activity: TurnkeyActivity,
}

#[derive(Debug, Deserialize)]
struct TurnkeyActivity {
    status: String,
    result: Option<TurnkeyActivityResult>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TurnkeyActivityResult {
    sign_raw_payload_result: Option<SignRawPayloadResult>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SignRawPayloadResult {
    encoded_signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletSetupRequest {
    pub user_wallet_address: String,
    pub wallet_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletSetupResponse {
    pub success: bool,
    pub turnkey_wallet_id: Option<String>,
    pub turnkey_wallet_address: Option<String>,
    pub error: Option<String>,
}

impl TurnkeySigner {
    pub async fn setup_wallet(
        &self,
        request: WalletSetupRequest,
    ) -> AppResult<WalletSetupResponse> {
        // Check if keys are configured
        if self.config.api_public_key.is_none() || self.config.api_private_key.is_none() {
            // Mock response for development
            tracing::warn!("Turnkey API keys not configured - returning mock setup");
            let mock_wallet_id = format!("mock_wallet_{}", uuid::Uuid::new_v4());

            self.set_wallet(request.user_wallet_address.clone(), mock_wallet_id.clone())
                .await?;

            return Ok(WalletSetupResponse {
                success: true,
                turnkey_wallet_id: Some(mock_wallet_id),
                turnkey_wallet_address: Some(request.user_wallet_address),
                error: None,
            });
        }

        // In production, this would:
        // 1. Create a Turnkey sub-organization for the user
        // 2. Create a new Solana wallet within that sub-org
        // 3. Return the wallet details

        // For now, placeholder implementation
        let mock_wallet_id = format!("tk_wallet_{}", uuid::Uuid::new_v4());

        self.set_wallet(request.user_wallet_address.clone(), mock_wallet_id.clone())
            .await?;

        Ok(WalletSetupResponse {
            success: true,
            turnkey_wallet_id: Some(mock_wallet_id),
            turnkey_wallet_address: Some(request.user_wallet_address),
            error: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_wallet_status_default() {
        let signer = TurnkeySigner::new(TurnkeyConfig::default());
        let status = signer.get_status().await;
        assert!(!status.is_connected);
        assert_eq!(status.delegation_status, DelegationStatus::NotConfigured);
    }

    #[tokio::test]
    async fn test_set_wallet() {
        let signer = TurnkeySigner::new(TurnkeyConfig::default());
        signer
            .set_wallet(
                "So11111111111111111111111111111111111111112".to_string(),
                "tk_wallet_123".to_string(),
            )
            .await
            .unwrap();

        let status = signer.get_status().await;
        assert!(status.is_connected);
        assert_eq!(status.delegation_status, DelegationStatus::Active);
    }

    #[tokio::test]
    async fn test_policy_validation() {
        let signer = TurnkeySigner::new(TurnkeyConfig::default());
        signer
            .set_wallet(
                "So11111111111111111111111111111111111111112".to_string(),
                "tk_wallet_123".to_string(),
            )
            .await
            .unwrap();

        // Valid request
        let valid_request = SignRequest {
            transaction_base64: "test".to_string(),
            estimated_amount_lamports: 1_000_000_000, // 1 SOL
            estimated_profit_lamports: Some(10_000_000),
            edge_id: None,
            description: "Test transaction".to_string(),
        };
        assert!(signer.validate_transaction(&valid_request).await.is_ok());

        // Over limit
        let over_limit_request = SignRequest {
            transaction_base64: "test".to_string(),
            estimated_amount_lamports: 10_000_000_000, // 10 SOL - over 5 SOL limit
            estimated_profit_lamports: Some(10_000_000),
            edge_id: None,
            description: "Test transaction".to_string(),
        };
        assert!(signer
            .validate_transaction(&over_limit_request)
            .await
            .is_err());
    }
}
