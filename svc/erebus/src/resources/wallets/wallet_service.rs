// Wallet service for multi-chain wallet support
// Contains utility methods for wallet validation and information

#![allow(dead_code)]

use super::registry::WALLET_REGISTRY;
use crate::resources::types::{
    DetectedWallet, InstallPrompt, WalletConnectionRequest, WalletConnectionResponse,
    WalletDetectionResponse, WalletInfo, WalletStatusResponse,
};

pub struct WalletService;

impl WalletService {
    /// Detect available wallets and provide installation guidance
    pub fn detect_wallets(available_wallets: Vec<String>) -> WalletDetectionResponse {
        let mut detected_wallets = Vec::new();
        let mut install_prompts = Vec::new();

        // Get all wallets from registry
        for wallet_info in WALLET_REGISTRY.list_all() {
            let is_available = available_wallets.contains(&wallet_info.id);

            detected_wallets.push(DetectedWallet {
                id: wallet_info.id.clone(),
                name: wallet_info.name.clone(),
                description: wallet_info.description.clone(),
                icon: wallet_info.icon.clone(),
                is_available,
                install_url: Some(wallet_info.install_url.clone()),
            });

            if !is_available {
                install_prompts.push(InstallPrompt {
                    wallet_id: wallet_info.id.clone(),
                    wallet_name: wallet_info.name.clone(),
                    install_url: wallet_info.install_url.clone(),
                    description: format!("Install {} for blockchain access", wallet_info.name),
                });
            }
        }

        // Determine recommended wallet (prefer first available)
        let recommended_wallet = available_wallets.first().cloned();

        WalletDetectionResponse {
            available_wallets: detected_wallets,
            recommended_wallet,
            install_prompts,
        }
    }

    /// Initiate wallet connection process
    pub fn initiate_connection(request: WalletConnectionRequest) -> WalletConnectionResponse {
        // Get adapter from registry
        let adapter = match WALLET_REGISTRY.get(&request.wallet_type) {
            Some(a) => a,
            None => {
                return WalletConnectionResponse {
                    success: false,
                    session_token: None,
                    wallet_info: None,
                    message: format!("Unsupported wallet type: {}", request.wallet_type),
                    next_step: None,
                };
            }
        };

        let info = adapter.info();

        // Detect chain from address
        let chain = match adapter.detect_chain_from_address(&request.wallet_address) {
            Some(c) => c,
            None => {
                return WalletConnectionResponse {
                    success: false,
                    session_token: None,
                    wallet_info: Some(WalletInfo {
                        id: info.id,
                        name: info.name,
                        description: info.description,
                        icon: info.icon,
                    }),
                    message: format!(
                        "Unable to detect chain from address: {}",
                        request.wallet_address
                    ),
                    next_step: None,
                };
            }
        };

        // Validate wallet address format
        if !adapter.validate_address(&request.wallet_address, &chain) {
            return WalletConnectionResponse {
                success: false,
                session_token: None,
                wallet_info: Some(WalletInfo {
                    id: info.id,
                    name: info.name,
                    description: info.description,
                    icon: info.icon,
                }),
                message: format!("Invalid {} address format", request.wallet_type),
                next_step: None,
            };
        }

        // Connection initiated successfully - next step is to create challenge
        WalletConnectionResponse {
            success: true,
            session_token: None,
            wallet_info: Some(WalletInfo {
                id: info.id,
                name: info.name,
                description: info.description,
                icon: info.icon,
            }),
            message: "Wallet connection initiated. Proceed to create authentication challenge."
                .to_string(),
            next_step: Some("create_challenge".to_string()),
        }
    }

    /// Get wallet status information
    pub fn get_wallet_status(
        session_token: Option<&str>,
        wallet_manager: &super::WalletManager,
    ) -> WalletStatusResponse {
        if let Some(token) = session_token {
            if let Some(session) = wallet_manager.validate_session(token) {
                return WalletStatusResponse {
                    connected: true,
                    wallet_type: Some(session.wallet_type.clone()),
                    wallet_address: Some(session.wallet_address.clone()),
                    session_valid: !session.is_expired(),
                    session_expires_at: Some(session.expires_at.to_rfc3339()),
                };
            }
        }

        WalletStatusResponse {
            connected: false,
            wallet_type: None,
            wallet_address: None,
            session_valid: false,
            session_expires_at: None,
        }
    }

    /// Get comprehensive wallet information for frontend
    pub fn get_wallet_information() -> Vec<WalletInfo> {
        WALLET_REGISTRY
            .list_all()
            .into_iter()
            .map(|w| WalletInfo {
                id: w.id,
                name: w.name,
                description: w.description,
                icon: w.icon,
            })
            .collect()
    }

    /// Validate wallet address format
    pub fn validate_wallet_address(wallet_type: &str, address: &str) -> bool {
        let adapter = match WALLET_REGISTRY.get(wallet_type) {
            Some(a) => a,
            None => return false,
        };

        // Try to detect chain and validate
        if let Some(chain) = adapter.detect_chain_from_address(address) {
            adapter.validate_address(address, &chain)
        } else {
            false
        }
    }

    /// Get wallet-specific connection instructions
    pub fn get_connection_instructions(wallet_type: &str) -> Option<String> {
        let adapter = WALLET_REGISTRY.get(wallet_type)?;
        let info = adapter.info();

        Some(format!(
            "1. Click 'Connect {}' in your browser extension\n\
             2. Approve the connection request\n\
             3. Sign the authentication message when prompted",
            info.name
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_wallets() {
        let available = vec!["metamask".to_string()];
        let response = WalletService::detect_wallets(available);

        assert!(response.available_wallets.len() >= 3);
        assert!(response
            .available_wallets
            .iter()
            .any(|w| w.id == "metamask" && w.is_available));
        assert!(response
            .available_wallets
            .iter()
            .any(|w| w.id == "phantom" && !w.is_available));
    }

    #[test]
    fn test_get_wallet_information() {
        let wallets = WalletService::get_wallet_information();
        assert!(wallets.len() >= 3);
        assert!(wallets.iter().any(|w| w.id == "bitget"));
    }

    #[test]
    fn test_validate_wallet_address() {
        // Valid EVM address
        assert!(WalletService::validate_wallet_address(
            "metamask",
            "0x742d35Cc6634C0532925a3b844Bc454e4438f44e"
        ));

        // Valid Solana address
        assert!(WalletService::validate_wallet_address(
            "phantom",
            "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM6"
        ));

        // Bitget with EVM address
        assert!(WalletService::validate_wallet_address(
            "bitget",
            "0x742d35Cc6634C0532925a3b844Bc454e4438f44e"
        ));

        // Bitget with Solana address
        assert!(WalletService::validate_wallet_address(
            "bitget",
            "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM6"
        ));

        // Invalid
        assert!(!WalletService::validate_wallet_address("unknown", "test"));
    }

    #[test]
    fn test_connection_instructions() {
        let metamask_instructions = WalletService::get_connection_instructions("metamask");
        assert!(metamask_instructions.is_some());
        assert!(metamask_instructions.unwrap().contains("MetaMask"));

        let bitget_instructions = WalletService::get_connection_instructions("bitget");
        assert!(bitget_instructions.is_some());
        assert!(bitget_instructions.unwrap().contains("Bitget"));
    }
}
