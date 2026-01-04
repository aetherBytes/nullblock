use std::collections::HashMap;
use std::sync::LazyLock;

use super::adapters::{BitgetAdapter, MetaMaskAdapter, PhantomAdapter};
use super::traits::{ChainType, WalletAdapter, WalletInfo};

pub struct WalletRegistry {
    adapters: HashMap<&'static str, Box<dyn WalletAdapter>>,
}

impl WalletRegistry {
    fn new() -> Self {
        let mut adapters: HashMap<&'static str, Box<dyn WalletAdapter>> = HashMap::new();

        // Compile-time registration - add new wallets here
        adapters.insert("metamask", Box::new(MetaMaskAdapter::new()));
        adapters.insert("phantom", Box::new(PhantomAdapter::new()));
        adapters.insert("bitget", Box::new(BitgetAdapter::new()));

        Self { adapters }
    }

    pub fn get(&self, wallet_id: &str) -> Option<&dyn WalletAdapter> {
        self.adapters.get(wallet_id).map(|a| a.as_ref())
    }

    pub fn list_all(&self) -> Vec<WalletInfo> {
        self.adapters.values().map(|a| a.info()).collect()
    }

    pub fn list_ids(&self) -> Vec<&'static str> {
        self.adapters.keys().copied().collect()
    }

    pub fn wallets_for_chain(&self, chain: &ChainType) -> Vec<&dyn WalletAdapter> {
        self.adapters
            .values()
            .filter(|a| a.supports_chain(chain))
            .map(|a| a.as_ref())
            .collect()
    }

    pub fn detect_wallet_from_address(
        &self,
        address: &str,
    ) -> Option<(&dyn WalletAdapter, ChainType)> {
        for adapter in self.adapters.values() {
            if let Some(chain) = adapter.detect_chain_from_address(address) {
                return Some((adapter.as_ref(), chain));
            }
        }
        None
    }

    pub fn detect_chain_from_address(&self, address: &str) -> Option<ChainType> {
        // Try each adapter to detect the chain
        for adapter in self.adapters.values() {
            if let Some(chain) = adapter.detect_chain_from_address(address) {
                return Some(chain);
            }
        }
        None
    }
}

pub static WALLET_REGISTRY: LazyLock<WalletRegistry> = LazyLock::new(WalletRegistry::new);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_get_wallet() {
        let registry = &*WALLET_REGISTRY;

        assert!(registry.get("metamask").is_some());
        assert!(registry.get("phantom").is_some());
        assert!(registry.get("bitget").is_some());
        assert!(registry.get("unknown").is_none());
    }

    #[test]
    fn test_registry_list_all() {
        let registry = &*WALLET_REGISTRY;
        let wallets = registry.list_all();

        assert!(wallets.len() >= 3);
        assert!(wallets.iter().any(|w| w.id == "metamask"));
        assert!(wallets.iter().any(|w| w.id == "phantom"));
        assert!(wallets.iter().any(|w| w.id == "bitget"));
    }

    #[test]
    fn test_registry_wallets_for_chain() {
        let registry = &*WALLET_REGISTRY;

        let evm_wallets = registry.wallets_for_chain(&ChainType::Evm);
        assert!(evm_wallets.len() >= 2); // MetaMask and Bitget

        let solana_wallets = registry.wallets_for_chain(&ChainType::Solana);
        assert!(solana_wallets.len() >= 2); // Phantom and Bitget
    }

    #[test]
    fn test_detect_chain_from_address() {
        let registry = &*WALLET_REGISTRY;

        // EVM address
        let evm_chain =
            registry.detect_chain_from_address("0x742d35Cc6634C0532925a3b844Bc454e4438f44e");
        assert_eq!(evm_chain, Some(ChainType::Evm));

        // Solana address
        let solana_chain =
            registry.detect_chain_from_address("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM6");
        assert_eq!(solana_chain, Some(ChainType::Solana));

        // Invalid address
        let invalid = registry.detect_chain_from_address("invalid");
        assert!(invalid.is_none());
    }
}
