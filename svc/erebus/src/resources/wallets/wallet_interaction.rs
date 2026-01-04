// Generic layer for agnostic wallet interaction
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::resources::types::{
    WalletChallengeRequest, WalletChallengeResponse, WalletInfo as ApiWalletInfo,
    WalletListResponse, WalletSession, WalletVerifyRequest, WalletVerifyResponse,
};
use super::registry::WALLET_REGISTRY;
use super::traits::{ChallengeContext, ChainType};

// Challenge data structure with chain info
#[derive(Debug, Clone)]
pub struct ChallengeData {
    pub message: String,
    pub wallet_address: String,
    pub wallet_type: String,
    pub chain: ChainType,
}

// Storage for challenges and sessions (in production, use Redis or database)
pub type ChallengeStorage = Arc<Mutex<HashMap<String, ChallengeData>>>;
pub type SessionStorage = Arc<Mutex<HashMap<String, WalletSession>>>;

#[derive(Clone)]
pub struct WalletManager {
    challenge_storage: ChallengeStorage,
    session_storage: SessionStorage,
}

impl Default for WalletManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WalletManager {
    pub fn new() -> Self {
        Self {
            challenge_storage: Arc::new(Mutex::new(HashMap::new())),
            session_storage: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get all supported wallets for API exposure
    pub fn get_supported_wallets() -> WalletListResponse {
        let wallets = WALLET_REGISTRY
            .list_all()
            .into_iter()
            .map(|w| ApiWalletInfo {
                id: w.id,
                name: w.name,
                description: w.description,
                icon: w.icon,
            })
            .collect();

        WalletListResponse {
            supported_wallets: wallets,
        }
    }

    /// Create authentication challenge for any supported wallet
    pub fn create_wallet_challenge(
        &self,
        request: WalletChallengeRequest,
    ) -> Result<WalletChallengeResponse, String> {
        // Get adapter from registry
        let adapter = WALLET_REGISTRY
            .get(&request.wallet_type)
            .ok_or_else(|| format!("Unsupported wallet type: {}", request.wallet_type))?;

        // Determine chain from request or auto-detect from address
        let chain = if let Some(chain_str) = &request.chain {
            chain_str
                .parse::<ChainType>()
                .map_err(|e| format!("Invalid chain type: {}", e))?
        } else {
            adapter
                .detect_chain_from_address(&request.wallet_address)
                .ok_or_else(|| {
                    format!(
                        "Unable to detect chain type for address: {}",
                        request.wallet_address
                    )
                })?
        };

        // Validate address format for the chain
        if !adapter.validate_address(&request.wallet_address, &chain) {
            return Err(format!(
                "Invalid {} address format for {} chain",
                request.wallet_type, chain
            ));
        }

        // Generate challenge
        let challenge_id = Uuid::new_v4().to_string();
        let context = ChallengeContext {
            wallet_address: request.wallet_address.clone(),
            challenge_id: challenge_id.clone(),
            chain: chain.clone(),
            timestamp: chrono::Utc::now().timestamp(),
        };

        let message = adapter.create_challenge_message(&context);

        // Store challenge with chain info
        {
            let mut challenges = self.challenge_storage.lock().unwrap();
            challenges.insert(
                challenge_id.clone(),
                ChallengeData {
                    message: message.clone(),
                    wallet_address: request.wallet_address.clone(),
                    wallet_type: request.wallet_type.clone(),
                    chain,
                },
            );
        }

        println!(
            "Created {} wallet challenge ({}) for {}: {}",
            request.wallet_type,
            context.chain,
            request.wallet_address,
            challenge_id
        );

        Ok(WalletChallengeResponse {
            challenge_id,
            message,
            wallet_address: request.wallet_address,
        })
    }

    /// Verify wallet signature and create session
    pub fn verify_wallet_signature(&self, request: WalletVerifyRequest) -> WalletVerifyResponse {
        // Retrieve and remove challenge
        let challenge_data = {
            let mut challenges = self.challenge_storage.lock().unwrap();
            challenges.remove(&request.challenge_id)
        };

        let data = match challenge_data {
            Some(d) => d,
            None => {
                return WalletVerifyResponse {
                    success: false,
                    session_token: None,
                    message: "Invalid or expired challenge".to_string(),
                    user_id: None,
                    registration_error: None,
                    network: None,
                };
            }
        };

        // Verify address matches
        if data.wallet_address != request.wallet_address {
            return WalletVerifyResponse {
                success: false,
                session_token: None,
                message: "Wallet address mismatch".to_string(),
                user_id: None,
                registration_error: None,
                network: None,
            };
        }

        // Get adapter from registry
        let adapter = match WALLET_REGISTRY.get(&data.wallet_type) {
            Some(a) => a,
            None => {
                return WalletVerifyResponse {
                    success: false,
                    session_token: None,
                    message: format!("Unknown wallet type: {}", data.wallet_type),
                    user_id: None,
                    registration_error: None,
                    network: None,
                };
            }
        };

        // Verify signature using adapter
        let verification_result = adapter.verify_signature(
            &data.message,
            &request.signature,
            &request.wallet_address,
            &data.chain,
        );

        match verification_result {
            Ok(true) => {
                // Create session
                let session_token = Uuid::new_v4().to_string();
                let network = adapter.get_network_for_chain(&data.chain);

                let session = WalletSession::new(
                    request.wallet_address.clone(),
                    data.wallet_type.clone(),
                    session_token.clone(),
                );

                // Store session
                {
                    let mut sessions = self.session_storage.lock().unwrap();
                    sessions.insert(session_token.clone(), session);
                }

                println!(
                    "{} wallet ({}) authenticated successfully for {}, session: {}",
                    data.wallet_type, data.chain, request.wallet_address, session_token
                );

                WalletVerifyResponse {
                    success: true,
                    session_token: Some(session_token),
                    message: "Wallet authenticated successfully".to_string(),
                    user_id: None,
                    registration_error: None,
                    network: Some(network.to_string()),
                }
            }
            Ok(false) => WalletVerifyResponse {
                success: false,
                session_token: None,
                message: "Signature verification failed".to_string(),
                user_id: None,
                registration_error: None,
                network: None,
            },
            Err(e) => WalletVerifyResponse {
                success: false,
                session_token: None,
                message: format!("Verification error: {}", e),
                user_id: None,
                registration_error: None,
                network: None,
            },
        }
    }

    /// Validate session token
    pub fn validate_session(&self, session_token: &str) -> Option<WalletSession> {
        let mut sessions = self.session_storage.lock().unwrap();

        if let Some(session) = sessions.get(session_token) {
            if session.is_expired() {
                sessions.remove(session_token);
                None
            } else {
                Some(session.clone())
            }
        } else {
            None
        }
    }

    /// Get supported networks for a wallet type
    pub fn get_wallet_networks(wallet_type: &str) -> Vec<serde_json::Value> {
        // Return networks based on supported chains
        let adapter = match WALLET_REGISTRY.get(wallet_type) {
            Some(a) => a,
            None => return vec![],
        };

        adapter
            .supported_chains()
            .iter()
            .map(|chain| {
                let network_name = adapter.get_network_for_chain(chain);
                serde_json::json!({
                    "chain": chain.to_string(),
                    "network": network_name,
                })
            })
            .collect()
    }

    /// Detect chain type from wallet address
    pub fn detect_chain_from_address(address: &str) -> Option<ChainType> {
        WALLET_REGISTRY.detect_chain_from_address(address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_challenge_evm() {
        let manager = WalletManager::new();
        let request = WalletChallengeRequest {
            wallet_address: "0x742d35Cc6634C0532925a3b844Bc454e4438f44e".to_string(),
            wallet_type: "metamask".to_string(),
            chain: None,
        };

        let result = manager.create_wallet_challenge(request);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(!response.challenge_id.is_empty());
        assert!(response.message.contains("MetaMask"));
    }

    #[test]
    fn test_create_challenge_solana() {
        let manager = WalletManager::new();
        let request = WalletChallengeRequest {
            wallet_address: "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM6".to_string(),
            wallet_type: "phantom".to_string(),
            chain: None,
        };

        let result = manager.create_wallet_challenge(request);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(!response.challenge_id.is_empty());
        assert!(response.message.contains("Phantom"));
    }

    #[test]
    fn test_create_challenge_bitget_evm() {
        let manager = WalletManager::new();
        let request = WalletChallengeRequest {
            wallet_address: "0x742d35Cc6634C0532925a3b844Bc454e4438f44e".to_string(),
            wallet_type: "bitget".to_string(),
            chain: Some("evm".to_string()),
        };

        let result = manager.create_wallet_challenge(request);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.message.contains("Bitget"));
        assert!(response.message.contains("EVM"));
    }

    #[test]
    fn test_create_challenge_bitget_solana() {
        let manager = WalletManager::new();
        let request = WalletChallengeRequest {
            wallet_address: "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM6".to_string(),
            wallet_type: "bitget".to_string(),
            chain: Some("solana".to_string()),
        };

        let result = manager.create_wallet_challenge(request);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.message.contains("Bitget"));
        assert!(response.message.contains("Solana"));
    }

    #[test]
    fn test_unsupported_wallet() {
        let manager = WalletManager::new();
        let request = WalletChallengeRequest {
            wallet_address: "0x742d35Cc6634C0532925a3b844Bc454e4438f44e".to_string(),
            wallet_type: "unknown_wallet".to_string(),
            chain: None,
        };

        let result = manager.create_wallet_challenge(request);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unsupported"));
    }

    #[test]
    fn test_get_supported_wallets() {
        let wallets = WalletManager::get_supported_wallets();
        assert!(wallets.supported_wallets.len() >= 3);

        let ids: Vec<_> = wallets.supported_wallets.iter().map(|w| &w.id).collect();
        assert!(ids.contains(&&"metamask".to_string()));
        assert!(ids.contains(&&"phantom".to_string()));
        assert!(ids.contains(&&"bitget".to_string()));
    }

    #[test]
    fn test_detect_chain_from_address() {
        // EVM address
        let evm = WalletManager::detect_chain_from_address(
            "0x742d35Cc6634C0532925a3b844Bc454e4438f44e",
        );
        assert_eq!(evm, Some(ChainType::Evm));

        // Solana address
        let solana = WalletManager::detect_chain_from_address(
            "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM6",
        );
        assert_eq!(solana, Some(ChainType::Solana));

        // Invalid address
        let invalid = WalletManager::detect_chain_from_address("invalid");
        assert!(invalid.is_none());
    }
}
