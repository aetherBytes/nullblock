use crate::resources::wallets::chains::{
    ChainSignatureVerifier, EvmSignatureVerifier, SolanaSignatureVerifier,
};
use crate::resources::wallets::traits::{
    ChallengeContext, ChainType, WalletAdapter, WalletError, WalletInfo,
};

#[derive(Debug)]
pub struct BitgetAdapter {
    evm_verifier: EvmSignatureVerifier,
    solana_verifier: SolanaSignatureVerifier,
}

impl BitgetAdapter {
    pub fn new() -> Self {
        Self {
            evm_verifier: EvmSignatureVerifier::new(),
            solana_verifier: SolanaSignatureVerifier::new(),
        }
    }
}

impl Default for BitgetAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl WalletAdapter for BitgetAdapter {
    fn id(&self) -> &'static str {
        "bitget"
    }

    fn info(&self) -> WalletInfo {
        WalletInfo {
            id: "bitget".to_string(),
            name: "Bitget Wallet".to_string(),
            description: "Multi-chain wallet supporting EVM and Solana networks".to_string(),
            icon: "https://web3.bitget.com/favicon.ico".to_string(),
            supported_chains: vec![ChainType::Evm, ChainType::Solana],
            install_url: "https://web3.bitget.com/".to_string(),
        }
    }

    fn supported_chains(&self) -> &[ChainType] {
        &[ChainType::Evm, ChainType::Solana]
    }

    fn validate_address(&self, address: &str, chain: &ChainType) -> bool {
        match chain {
            ChainType::Evm => self.evm_verifier.validate_address(address),
            ChainType::Solana => self.solana_verifier.validate_address(address),
        }
    }

    fn create_challenge_message(&self, context: &ChallengeContext) -> String {
        let chain_name = match context.chain {
            ChainType::Evm => "EVM",
            ChainType::Solana => "Solana",
        };

        format!(
            "Welcome to Nullblock!\n\n\
             Sign this message to authenticate your Bitget Wallet ({}).\n\n\
             Wallet Address: {}\n\
             Challenge ID: {}\n\
             Timestamp: {}\n\n\
             This action will not trigger any blockchain transaction or cost gas fees.\n\n\
             By signing, you agree to connect your wallet to the Nullblock platform.",
            chain_name, context.wallet_address, context.challenge_id, context.timestamp
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
            ChainType::Evm => self
                .evm_verifier
                .verify_signature(message, signature, wallet_address),
            ChainType::Solana => self
                .solana_verifier
                .verify_signature(message, signature, wallet_address),
        }
    }

    fn detect_chain_from_address(&self, address: &str) -> Option<ChainType> {
        // Try EVM first (more common)
        if self.evm_verifier.validate_address(address) {
            Some(ChainType::Evm)
        } else if self.solana_verifier.validate_address(address) {
            Some(ChainType::Solana)
        } else {
            None
        }
    }

    fn get_network_for_chain(&self, chain: &ChainType) -> &'static str {
        match chain {
            ChainType::Evm => "ethereum",
            ChainType::Solana => "solana",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitget_adapter_info() {
        let adapter = BitgetAdapter::new();
        let info = adapter.info();

        assert_eq!(info.id, "bitget");
        assert_eq!(info.name, "Bitget Wallet");
        assert!(info.supported_chains.contains(&ChainType::Evm));
        assert!(info.supported_chains.contains(&ChainType::Solana));
    }

    #[test]
    fn test_bitget_supports_both_chains() {
        let adapter = BitgetAdapter::new();

        assert!(adapter.supports_chain(&ChainType::Evm));
        assert!(adapter.supports_chain(&ChainType::Solana));
    }

    #[test]
    fn test_bitget_evm_address_validation() {
        let adapter = BitgetAdapter::new();

        assert!(adapter.validate_address(
            "0x742d35Cc6634C0532925a3b844Bc454e4438f44e",
            &ChainType::Evm
        ));

        assert!(!adapter.validate_address(
            "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM6",
            &ChainType::Evm
        ));
    }

    #[test]
    fn test_bitget_solana_address_validation() {
        let adapter = BitgetAdapter::new();

        assert!(adapter.validate_address(
            "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM6",
            &ChainType::Solana
        ));

        assert!(!adapter.validate_address(
            "0x742d35Cc6634C0532925a3b844Bc454e4438f44e",
            &ChainType::Solana
        ));
    }

    #[test]
    fn test_bitget_chain_detection() {
        let adapter = BitgetAdapter::new();

        // Detect EVM address
        assert_eq!(
            adapter.detect_chain_from_address("0x742d35Cc6634C0532925a3b844Bc454e4438f44e"),
            Some(ChainType::Evm)
        );

        // Detect Solana address
        assert_eq!(
            adapter.detect_chain_from_address("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM6"),
            Some(ChainType::Solana)
        );

        // Invalid address
        assert_eq!(adapter.detect_chain_from_address("invalid"), None);
    }

    #[test]
    fn test_bitget_challenge_message_includes_chain() {
        let adapter = BitgetAdapter::new();

        let evm_context = ChallengeContext {
            wallet_address: "0x742d35Cc6634C0532925a3b844Bc454e4438f44e".to_string(),
            challenge_id: "test-123".to_string(),
            chain: ChainType::Evm,
            timestamp: 1234567890,
        };
        let evm_message = adapter.create_challenge_message(&evm_context);
        assert!(evm_message.contains("EVM"));

        let solana_context = ChallengeContext {
            wallet_address: "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM6".to_string(),
            challenge_id: "test-456".to_string(),
            chain: ChainType::Solana,
            timestamp: 1234567890,
        };
        let solana_message = adapter.create_challenge_message(&solana_context);
        assert!(solana_message.contains("Solana"));
    }
}
