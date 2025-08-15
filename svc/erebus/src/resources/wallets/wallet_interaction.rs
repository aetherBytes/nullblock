// Generic layer for agnostic wallet interaction
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::resources::types::{
    WalletInfo, WalletProvider, WalletChallengeRequest, WalletChallengeResponse,
    WalletVerifyRequest, WalletVerifyResponse, WalletListResponse, WalletSession
};
use super::{MetaMaskWallet, PhantomWallet};

// Storage for challenges and sessions (in production, use Redis or database)
pub type ChallengeStorage = Arc<Mutex<HashMap<String, (String, String)>>>;  // challenge_id -> (message, wallet_address)
pub type SessionStorage = Arc<Mutex<HashMap<String, WalletSession>>>;  // session_token -> WalletSession

#[derive(Clone)]
pub struct WalletManager {
    challenge_storage: ChallengeStorage,
    session_storage: SessionStorage,
}

impl WalletManager {
    pub fn new() -> Self {
        Self {
            challenge_storage: Arc::new(Mutex::new(HashMap::new())),
            session_storage: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_challenge_storage(&self) -> ChallengeStorage {
        Arc::clone(&self.challenge_storage)
    }

    pub fn get_session_storage(&self) -> SessionStorage {
        Arc::clone(&self.session_storage)
    }

    /// Get all supported wallets for MCP exposure
    pub fn get_supported_wallets() -> WalletListResponse {
        WalletListResponse {
            supported_wallets: vec![
                PhantomWallet::get_wallet_info(),
                MetaMaskWallet::get_wallet_info(),
            ],
        }
    }

    /// Create authentication challenge for any supported wallet
    pub fn create_wallet_challenge(
        &self,
        request: WalletChallengeRequest,
    ) -> Result<WalletChallengeResponse, String> {
        let challenge_id = Uuid::new_v4().to_string();
        
        // Generate wallet-specific challenge message
        let message = match request.wallet_type.as_str() {
            "phantom" => PhantomWallet::create_challenge_message(&request.wallet_address, &challenge_id),
            "metamask" => MetaMaskWallet::create_challenge_message(&request.wallet_address, &challenge_id),
            _ => return Err(format!("Unsupported wallet type: {}", request.wallet_type)),
        };

        // Validate wallet address format
        let is_valid = match request.wallet_type.as_str() {
            "phantom" => PhantomWallet::validate_solana_address(&request.wallet_address),
            "metamask" => MetaMaskWallet::validate_ethereum_address(&request.wallet_address),
            _ => false,
        };

        if !is_valid {
            return Err(format!("Invalid {} address format", request.wallet_type));
        }

        // Store challenge
        {
            let mut challenges = self.challenge_storage.lock().unwrap();
            challenges.insert(challenge_id.clone(), (message.clone(), request.wallet_address.clone()));
        }

        println!("🔐 Created {} wallet challenge for {}: {}", 
                 request.wallet_type, request.wallet_address, challenge_id);

        Ok(WalletChallengeResponse {
            challenge_id,
            message,
            wallet_address: request.wallet_address,
        })
    }

    /// Verify wallet signature and create session
    pub fn verify_wallet_signature(
        &self,
        request: WalletVerifyRequest,
    ) -> WalletVerifyResponse {
        // Retrieve and remove challenge
        let challenge_data = {
            let mut challenges = self.challenge_storage.lock().unwrap();
            challenges.remove(&request.challenge_id)
        };

        let (message, expected_address) = match challenge_data {
            Some(data) => data,
            None => {
                return WalletVerifyResponse {
                    success: false,
                    session_token: None,
                    message: "Invalid or expired challenge".to_string(),
                };
            }
        };

        // Verify address matches
        if expected_address != request.wallet_address {
            return WalletVerifyResponse {
                success: false,
                session_token: None,
                message: "Wallet address mismatch".to_string(),
            };
        }

        // Determine wallet type from address format
        let wallet_type = if PhantomWallet::validate_solana_address(&request.wallet_address) {
            "phantom"
        } else if MetaMaskWallet::validate_ethereum_address(&request.wallet_address) {
            "metamask"
        } else {
            return WalletVerifyResponse {
                success: false,
                session_token: None,
                message: "Unable to determine wallet type".to_string(),
            };
        };

        // Verify signature using appropriate wallet provider
        let verification_result = match wallet_type {
            "phantom" => PhantomWallet::verify_signature(&message, &request.signature, &request.wallet_address),
            "metamask" => MetaMaskWallet::verify_signature(&message, &request.signature, &request.wallet_address),
            _ => Err("Unsupported wallet type".to_string()),
        };

        match verification_result {
            Ok(true) => {
                // Create session
                let session_token = Uuid::new_v4().to_string();
                let session = WalletSession::new(
                    request.wallet_address.clone(),
                    wallet_type.to_string(),
                    session_token.clone(),
                );

                // Store session
                {
                    let mut sessions = self.session_storage.lock().unwrap();
                    sessions.insert(session_token.clone(), session);
                }

                println!("✅ {} wallet authenticated successfully for {}, session: {}", 
                         wallet_type, request.wallet_address, session_token);

                WalletVerifyResponse {
                    success: true,
                    session_token: Some(session_token),
                    message: "Wallet authenticated successfully".to_string(),
                }
            }
            Ok(false) => WalletVerifyResponse {
                success: false,
                session_token: None,
                message: "Signature verification failed".to_string(),
            },
            Err(e) => WalletVerifyResponse {
                success: false,
                session_token: None,
                message: format!("Verification error: {}", e),
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

    /// Get wallet info by type for MCP integration
    pub fn get_wallet_info(wallet_type: &str) -> Option<WalletInfo> {
        match wallet_type {
            "phantom" => Some(PhantomWallet::get_wallet_info()),
            "metamask" => Some(MetaMaskWallet::get_wallet_info()),
            _ => None,
        }
    }

    /// Clean up expired sessions (should be called periodically)
    pub fn cleanup_expired_sessions(&self) {
        let mut sessions = self.session_storage.lock().unwrap();
        sessions.retain(|_, session| !session.is_expired());
    }

    /// Get active session count for monitoring
    pub fn get_active_sessions_count(&self) -> usize {
        let sessions = self.session_storage.lock().unwrap();
        sessions.len()
    }

    /// Get supported networks for a wallet type
    pub fn get_wallet_networks(wallet_type: &str) -> Vec<serde_json::Value> {
        match wallet_type {
            "phantom" => PhantomWallet::get_network_info()
                .into_iter()
                .map(|n| serde_json::to_value(n).unwrap_or_default())
                .collect(),
            "metamask" => MetaMaskWallet::get_network_info()
                .into_iter()
                .map(|n| serde_json::to_value(n).unwrap_or_default())
                .collect(),
            _ => vec![],
        }
    }
}
