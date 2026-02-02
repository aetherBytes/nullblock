use crate::resources::wallets::chains::{ChainSignatureVerifier, SolanaSignatureVerifier};
use crate::resources::wallets::traits::{
    ChainType, ChallengeContext, WalletAdapter, WalletError, WalletInfo,
};

#[derive(Debug)]
pub struct PhantomAdapter {
    solana_verifier: SolanaSignatureVerifier,
}

impl PhantomAdapter {
    pub fn new() -> Self {
        Self {
            solana_verifier: SolanaSignatureVerifier::new(),
        }
    }
}

impl Default for PhantomAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl WalletAdapter for PhantomAdapter {
    fn id(&self) -> &'static str {
        "phantom"
    }

    fn info(&self) -> WalletInfo {
        WalletInfo {
            id: "phantom".to_string(),
            name: "Phantom".to_string(),
            description: "Solana Wallet - The #1 wallet for Solana DeFi and NFTs".to_string(),
            icon: "https://phantom.app/img/phantom-icon-purple.svg".to_string(),
            supported_chains: vec![ChainType::Solana],
            install_url: "https://phantom.app/".to_string(),
        }
    }

    fn supported_chains(&self) -> &[ChainType] {
        &[ChainType::Solana]
    }

    fn validate_address(&self, address: &str, chain: &ChainType) -> bool {
        match chain {
            ChainType::Solana => self.solana_verifier.validate_address(address),
            _ => false,
        }
    }

    fn create_challenge_message(&self, context: &ChallengeContext) -> String {
        format!(
            "Nullblock Authentication\n\n\
             Connect your Phantom wallet to unlock agentic workflows.\n\n\
             Wallet: {}\n\
             Challenge: {}\n\
             Timestamp: {}\n\n\
             This signature will not trigger any blockchain transaction.\n\n\
             Welcome to the void, agent.",
            context.wallet_address, context.challenge_id, context.timestamp
        )
    }

    fn verify_signature(
        &self,
        message: &str,
        signature: &str,
        wallet_address: &str,
        chain: &ChainType,
    ) -> Result<bool, WalletError> {
        match chain {
            ChainType::Solana => {
                self.solana_verifier
                    .verify_signature(message, signature, wallet_address)
            }
            _ => Err(WalletError::UnsupportedChain(chain.clone())),
        }
    }

    fn detect_chain_from_address(&self, address: &str) -> Option<ChainType> {
        if self.solana_verifier.validate_address(address) {
            Some(ChainType::Solana)
        } else {
            None
        }
    }

    fn get_network_for_chain(&self, chain: &ChainType) -> &'static str {
        match chain {
            ChainType::Solana => "solana",
            _ => "unknown",
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct NetworkInfo {
    pub name: String,
    pub rpc_url: String,
    pub environment: String,
}

impl PhantomAdapter {
    pub fn get_network_info() -> Vec<NetworkInfo> {
        vec![
            NetworkInfo {
                name: "Mainnet Beta".to_string(),
                rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
                environment: "mainnet-beta".to_string(),
            },
            NetworkInfo {
                name: "Devnet".to_string(),
                rpc_url: "https://api.devnet.solana.com".to_string(),
                environment: "devnet".to_string(),
            },
            NetworkInfo {
                name: "Testnet".to_string(),
                rpc_url: "https://api.testnet.solana.com".to_string(),
                environment: "testnet".to_string(),
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phantom_adapter_info() {
        let adapter = PhantomAdapter::new();
        let info = adapter.info();

        assert_eq!(info.id, "phantom");
        assert_eq!(info.name, "Phantom");
        assert!(info.supported_chains.contains(&ChainType::Solana));
    }

    #[test]
    fn test_phantom_supports_solana_only() {
        let adapter = PhantomAdapter::new();

        assert!(adapter.supports_chain(&ChainType::Solana));
        assert!(!adapter.supports_chain(&ChainType::Evm));
    }

    #[test]
    fn test_phantom_address_validation() {
        let adapter = PhantomAdapter::new();

        // Valid Solana address
        assert!(adapter.validate_address(
            "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM6",
            &ChainType::Solana
        ));

        // Invalid for EVM chain
        assert!(!adapter.validate_address(
            "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM6",
            &ChainType::Evm
        ));
    }

    #[test]
    fn test_phantom_challenge_message() {
        let adapter = PhantomAdapter::new();
        let context = ChallengeContext {
            wallet_address: "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM6".to_string(),
            challenge_id: "test-challenge-123".to_string(),
            chain: ChainType::Solana,
            timestamp: 1234567890,
        };

        let message = adapter.create_challenge_message(&context);
        assert!(message.contains("Phantom"));
        assert!(message.contains(&context.wallet_address));
        assert!(message.contains(&context.challenge_id));
    }
}
