use crate::resources::wallets::chains::{ChainSignatureVerifier, EvmSignatureVerifier};
use crate::resources::wallets::traits::{
    ChainType, ChallengeContext, WalletAdapter, WalletError, WalletInfo,
};

#[derive(Debug)]
pub struct MetaMaskAdapter {
    evm_verifier: EvmSignatureVerifier,
}

impl MetaMaskAdapter {
    pub fn new() -> Self {
        Self {
            evm_verifier: EvmSignatureVerifier::new(),
        }
    }
}

impl Default for MetaMaskAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl WalletAdapter for MetaMaskAdapter {
    fn id(&self) -> &'static str {
        "metamask"
    }

    fn info(&self) -> WalletInfo {
        WalletInfo {
            id: "metamask".to_string(),
            name: "MetaMask".to_string(),
            description: "Ethereum Wallet - Industry standard for Ethereum dApps".to_string(),
            icon: "https://raw.githubusercontent.com/MetaMask/brand-resources/master/SVG/SVG_MetaMask_Icon_Color.svg".to_string(),
            supported_chains: vec![ChainType::Evm],
            install_url: "https://metamask.io/".to_string(),
        }
    }

    fn supported_chains(&self) -> &[ChainType] {
        &[ChainType::Evm]
    }

    fn validate_address(&self, address: &str, chain: &ChainType) -> bool {
        match chain {
            ChainType::Evm => self.evm_verifier.validate_address(address),
            _ => false,
        }
    }

    fn create_challenge_message(&self, context: &ChallengeContext) -> String {
        format!(
            "Welcome to Nullblock!\n\n\
             Sign this message to authenticate your MetaMask wallet.\n\n\
             Wallet Address: {}\n\
             Challenge ID: {}\n\
             Timestamp: {}\n\n\
             This action will not trigger any blockchain transaction or cost gas fees.\n\n\
             By signing, you agree to connect your wallet to the Nullblock platform.",
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
            ChainType::Evm => {
                self.evm_verifier
                    .verify_signature(message, signature, wallet_address)
            }
            _ => Err(WalletError::UnsupportedChain(chain.clone())),
        }
    }

    fn detect_chain_from_address(&self, address: &str) -> Option<ChainType> {
        if self.evm_verifier.validate_address(address) {
            Some(ChainType::Evm)
        } else {
            None
        }
    }

    fn get_network_for_chain(&self, chain: &ChainType) -> &'static str {
        match chain {
            ChainType::Evm => "ethereum",
            _ => "unknown",
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct NetworkInfo {
    pub chain_id: String,
    pub name: String,
    pub rpc_url: String,
    pub currency: String,
}

impl MetaMaskAdapter {
    pub fn get_network_info() -> Vec<NetworkInfo> {
        vec![
            NetworkInfo {
                chain_id: "0x1".to_string(),
                name: "Ethereum Mainnet".to_string(),
                rpc_url: "https://mainnet.infura.io".to_string(),
                currency: "ETH".to_string(),
            },
            NetworkInfo {
                chain_id: "0x89".to_string(),
                name: "Polygon Mainnet".to_string(),
                rpc_url: "https://polygon-rpc.com".to_string(),
                currency: "MATIC".to_string(),
            },
            NetworkInfo {
                chain_id: "0xa".to_string(),
                name: "Optimism".to_string(),
                rpc_url: "https://mainnet.optimism.io".to_string(),
                currency: "ETH".to_string(),
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metamask_adapter_info() {
        let adapter = MetaMaskAdapter::new();
        let info = adapter.info();

        assert_eq!(info.id, "metamask");
        assert_eq!(info.name, "MetaMask");
        assert!(info.supported_chains.contains(&ChainType::Evm));
    }

    #[test]
    fn test_metamask_supports_evm_only() {
        let adapter = MetaMaskAdapter::new();

        assert!(adapter.supports_chain(&ChainType::Evm));
        assert!(!adapter.supports_chain(&ChainType::Solana));
    }

    #[test]
    fn test_metamask_address_validation() {
        let adapter = MetaMaskAdapter::new();

        // Valid EVM address
        assert!(adapter.validate_address(
            "0x742d35Cc6634C0532925a3b844Bc454e4438f44e",
            &ChainType::Evm
        ));

        // Invalid for Solana chain
        assert!(!adapter.validate_address(
            "0x742d35Cc6634C0532925a3b844Bc454e4438f44e",
            &ChainType::Solana
        ));
    }

    #[test]
    fn test_metamask_challenge_message() {
        let adapter = MetaMaskAdapter::new();
        let context = ChallengeContext {
            wallet_address: "0x742d35Cc6634C0532925a3b844Bc454e4438f44e".to_string(),
            challenge_id: "test-challenge-123".to_string(),
            chain: ChainType::Evm,
            timestamp: 1234567890,
        };

        let message = adapter.create_challenge_message(&context);
        assert!(message.contains("MetaMask"));
        assert!(message.contains(&context.wallet_address));
        assert!(message.contains(&context.challenge_id));
    }
}
