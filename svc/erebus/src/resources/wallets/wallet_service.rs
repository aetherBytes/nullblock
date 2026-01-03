// Wallet service for multi-chain wallet support
// Contains utility methods for wallet validation and information

#![allow(dead_code)]

use crate::resources::types::{
    DetectedWallet, InstallPrompt, WalletDetectionResponse, WalletConnectionRequest,
    WalletConnectionResponse, WalletInfo, WalletStatusResponse, WalletProvider,
};
use super::{MetaMaskWallet, PhantomWallet};

pub struct WalletService;

impl WalletService {
    /// Detect available wallets and provide installation guidance
    pub fn detect_wallets(available_wallets: Vec<String>) -> WalletDetectionResponse {
        let mut detected_wallets = Vec::new();
        let mut install_prompts = Vec::new();

        // Check for Phantom
        let phantom_available = available_wallets.contains(&"phantom".to_string());
        let phantom_wallet = DetectedWallet {
            id: "phantom".to_string(),
            name: "Phantom".to_string(),
            description: "Solana Wallet - Fast, secure, and user-friendly".to_string(),
            icon: "👻".to_string(),
            is_available: phantom_available,
            install_url: Some("https://phantom.app/".to_string()),
        };
        detected_wallets.push(phantom_wallet.clone());

        if !phantom_available {
            install_prompts.push(InstallPrompt {
                wallet_id: "phantom".to_string(),
                wallet_name: "Phantom".to_string(),
                install_url: "https://phantom.app/".to_string(),
                description: "Install Phantom for Solana blockchain access".to_string(),
            });
        }

        // Check for MetaMask
        let metamask_available = available_wallets.contains(&"metamask".to_string());
        let metamask_wallet = DetectedWallet {
            id: "metamask".to_string(),
            name: "MetaMask".to_string(),
            description: "Ethereum Wallet - Industry standard for Ethereum dApps".to_string(),
            icon: "🦊".to_string(),
            is_available: metamask_available,
            install_url: Some("https://metamask.io/".to_string()),
        };
        detected_wallets.push(metamask_wallet.clone());

        if !metamask_available {
            install_prompts.push(InstallPrompt {
                wallet_id: "metamask".to_string(),
                wallet_name: "MetaMask".to_string(),
                install_url: "https://metamask.io/".to_string(),
                description: "Install MetaMask for Ethereum blockchain access".to_string(),
            });
        }

        // Determine recommended wallet
        let recommended_wallet = if phantom_available {
            Some("phantom".to_string())
        } else if metamask_available {
            Some("metamask".to_string())
        } else {
            None
        };

        WalletDetectionResponse {
            available_wallets: detected_wallets,
            recommended_wallet,
            install_prompts,
        }
    }

    /// Initiate wallet connection process
    pub fn initiate_connection(request: WalletConnectionRequest) -> WalletConnectionResponse {
        // Validate wallet type
        let wallet_info = match request.wallet_type.as_str() {
            "phantom" => Some(PhantomWallet::get_wallet_info()),
            "metamask" => Some(MetaMaskWallet::get_wallet_info()),
            _ => None,
        };

        if wallet_info.is_none() {
            return WalletConnectionResponse {
                success: false,
                session_token: None,
                wallet_info: None,
                message: format!("Unsupported wallet type: {}", request.wallet_type),
                next_step: None,
            };
        }

        // Validate wallet address format
        let is_valid_address = match request.wallet_type.as_str() {
            "phantom" => PhantomWallet::validate_solana_address(&request.wallet_address),
            "metamask" => MetaMaskWallet::validate_ethereum_address(&request.wallet_address),
            _ => false,
        };

        if !is_valid_address {
            return WalletConnectionResponse {
                success: false,
                session_token: None,
                wallet_info: wallet_info.clone(),
                message: format!("Invalid {} address format", request.wallet_type),
                next_step: None,
            };
        }

        // Connection initiated successfully - next step is to create challenge
        WalletConnectionResponse {
            success: true,
            session_token: None,
            wallet_info,
            message: "Wallet connection initiated. Proceed to create authentication challenge.".to_string(),
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
        vec![
            PhantomWallet::get_wallet_info(),
            MetaMaskWallet::get_wallet_info(),
        ]
    }

    /// Validate wallet address format
    pub fn validate_wallet_address(wallet_type: &str, address: &str) -> bool {
        match wallet_type {
            "phantom" => PhantomWallet::validate_solana_address(address),
            "metamask" => MetaMaskWallet::validate_ethereum_address(address),
            _ => false,
        }
    }

    /// Get wallet-specific connection instructions
    pub fn get_connection_instructions(wallet_type: &str) -> Option<String> {
        match wallet_type {
            "phantom" => Some(
                "1. Click 'Connect Phantom' in your browser extension\n\
                 2. Approve the connection request\n\
                 3. Sign the authentication message when prompted"
                    .to_string(),
            ),
            "metamask" => Some(
                "1. Click 'Connect MetaMask' in your browser extension\n\
                 2. Approve the connection request\n\
                 3. Sign the authentication message when prompted"
                    .to_string(),
            ),
            _ => None,
        }
    }
}
