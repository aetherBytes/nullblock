mod evm;
mod solana;

pub use evm::EvmSignatureVerifier;
pub use solana::SolanaSignatureVerifier;

use super::traits::WalletError;

pub trait ChainSignatureVerifier: Send + Sync {
    fn verify_signature(
        &self,
        message: &str,
        signature: &str,
        wallet_address: &str,
    ) -> Result<bool, WalletError>;

    fn validate_address(&self, address: &str) -> bool;
}
