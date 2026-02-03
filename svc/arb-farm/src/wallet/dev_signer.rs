use solana_sdk::{
    message::VersionedMessage,
    signature::{Keypair, Signature, Signer},
    transaction::{Transaction, VersionedTransaction},
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use super::policy::{ArbFarmPolicy, DailyUsage, PolicyViolation};
use super::turnkey::{DelegationStatus, SignRequest, SignResult, WalletStatus};
use crate::error::{AppError, AppResult};

pub struct DevWalletSigner {
    keypair: Option<Keypair>,
    wallet_address: Option<String>,
    wallet_status: Arc<RwLock<WalletStatus>>,
}

impl DevWalletSigner {
    pub fn new(private_key_base58: Option<&str>, wallet_address: Option<&str>) -> AppResult<Self> {
        let keypair = if let Some(key) = private_key_base58 {
            let key_bytes = bs58::decode(key)
                .into_vec()
                .map_err(|e| AppError::Internal(format!("Invalid private key base58: {}", e)))?;

            let keypair = Keypair::try_from(key_bytes.as_slice())
                .map_err(|e| AppError::Internal(format!("Invalid keypair bytes: {}", e)))?;

            let derived_address = keypair.pubkey().to_string();

            if let Some(expected_address) = wallet_address {
                if derived_address != expected_address {
                    warn!(
                        "Wallet address mismatch: expected {} but derived {}",
                        expected_address, derived_address
                    );
                }
            }

            info!("🔐 Dev wallet signer initialized: {}", derived_address);
            Some(keypair)
        } else {
            warn!("No private key provided - dev signer not available");
            None
        };

        let address = keypair.as_ref().map(|k| k.pubkey().to_string());

        Ok(Self {
            keypair,
            wallet_address: address,
            wallet_status: Arc::new(RwLock::new(WalletStatus::default())),
        })
    }

    pub fn is_configured(&self) -> bool {
        self.keypair.is_some()
    }

    pub fn get_address(&self) -> Option<&str> {
        self.wallet_address.as_deref()
    }

    pub async fn get_status(&self) -> WalletStatus {
        self.wallet_status.read().await.clone()
    }

    pub async fn connect(&self) -> AppResult<()> {
        if self.keypair.is_none() {
            return Err(AppError::Internal(
                "Cannot connect: private key not configured".into(),
            ));
        }

        let mut status = self.wallet_status.write().await;
        status.is_connected = true;
        status.wallet_address = self.wallet_address.clone();
        status.turnkey_wallet_id = Some(format!(
            "dev_{}",
            self.wallet_address
                .as_ref()
                .map(|a| &a[..8])
                .unwrap_or("unknown")
        ));
        status.delegation_status = DelegationStatus::Active;

        info!(
            "✅ Dev wallet connected: {}",
            self.wallet_address.as_deref().unwrap_or("unknown")
        );
        Ok(())
    }

    pub async fn disconnect(&self) -> AppResult<()> {
        let mut status = self.wallet_status.write().await;
        status.is_connected = false;
        status.delegation_status = DelegationStatus::Revoked;
        info!("🔌 Dev wallet disconnected");
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

        if request.estimated_amount_lamports > policy.max_transaction_amount_lamports {
            return Err(PolicyViolation::amount_exceeded(
                request.estimated_amount_lamports,
                policy.max_transaction_amount_lamports,
            ));
        }

        let mut daily_usage = status.daily_usage.clone();
        daily_usage.reset_if_new_day();
        daily_usage.can_execute(request.estimated_amount_lamports, policy)?;

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
        if let Err(violation) = self.validate_transaction(&request).await {
            return Ok(SignResult::policy_error(violation));
        }

        let keypair = match &self.keypair {
            Some(kp) => kp,
            None => return Ok(SignResult::error("Dev wallet private key not configured")),
        };

        let status = self.wallet_status.read().await;
        if !status.is_connected {
            return Ok(SignResult::error("Dev wallet not connected"));
        }
        drop(status);

        let tx_bytes = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            &request.transaction_base64,
        )
        .map_err(|e| AppError::Internal(format!("Invalid transaction base64: {}", e)))?;

        let (signed_tx_bytes, signature) = if let Ok(mut versioned_tx) =
            bincode::deserialize::<VersionedTransaction>(&tx_bytes)
        {
            debug!(
                "📝 Signing versioned transaction: {} signatures needed",
                versioned_tx.signatures.len()
            );

            let message_bytes = versioned_tx.message.serialize();
            let sig = keypair.sign_message(&message_bytes);

            if !versioned_tx.signatures.is_empty() {
                versioned_tx.signatures[0] = sig;
            }

            let bytes = bincode::serialize(&versioned_tx).map_err(|e| {
                AppError::Internal(format!("Failed to serialize versioned tx: {}", e))
            })?;
            (bytes, sig)
        } else if let Ok(mut legacy_tx) = bincode::deserialize::<Transaction>(&tx_bytes) {
            debug!(
                "📝 Signing legacy transaction: {} signatures needed",
                legacy_tx.signatures.len()
            );

            let message_bytes = legacy_tx.message.serialize();
            let sig = keypair.sign_message(&message_bytes);

            if !legacy_tx.signatures.is_empty() {
                legacy_tx.signatures[0] = sig;
            }

            let bytes = bincode::serialize(&legacy_tx)
                .map_err(|e| AppError::Internal(format!("Failed to serialize legacy tx: {}", e)))?;
            (bytes, sig)
        } else {
            return Ok(SignResult::error(
                "Failed to deserialize transaction - not a valid versioned or legacy transaction",
            ));
        };

        let signed_tx_base64 =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &signed_tx_bytes);

        let signature_str = signature.to_string();

        let mut status = self.wallet_status.write().await;
        status.daily_usage.reset_if_new_day();
        status
            .daily_usage
            .record_transaction(request.estimated_amount_lamports);

        info!(
            "✅ Transaction signed: {} (amount: {} lamports)",
            &signature_str[..16],
            request.estimated_amount_lamports
        );

        Ok(SignResult::success(signed_tx_base64, signature_str))
    }

    pub async fn sign_message(&self, message: &[u8]) -> AppResult<Signature> {
        let keypair = match &self.keypair {
            Some(kp) => kp,
            None => {
                return Err(AppError::Internal(
                    "Dev wallet private key not configured".into(),
                ))
            }
        };

        Ok(keypair.sign_message(message))
    }

    pub fn get_pubkey(&self) -> Option<solana_sdk::pubkey::Pubkey> {
        self.keypair.as_ref().map(|kp| kp.pubkey())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signer_without_key() {
        let signer = DevWalletSigner::new(None, None).unwrap();
        assert!(!signer.is_configured());
        assert!(signer.get_address().is_none());
    }

    #[tokio::test]
    async fn test_connect_without_key_fails() {
        let signer = DevWalletSigner::new(None, None).unwrap();
        assert!(signer.connect().await.is_err());
    }
}
