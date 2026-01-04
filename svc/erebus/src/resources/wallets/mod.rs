// Wallet-specific implementations, interaction layer, and routes

pub mod adapters;
pub mod chains;
pub mod registry;
pub mod routes;
pub mod traits;
pub mod wallet_interaction;
pub mod wallet_service;

// Keep old modules for backwards compatibility during migration
// TODO: Remove these once migration is complete
pub mod metamask;
pub mod phantom;

// Re-export new adapter system
pub use adapters::{BitgetAdapter, MetaMaskAdapter, PhantomAdapter};
pub use registry::WALLET_REGISTRY;
pub use traits::{ChainType, ChallengeContext, WalletAdapter, WalletError, WalletInfo};
pub use wallet_interaction::WalletManager;

// Re-export old implementations for backwards compatibility
// TODO: Remove these once migration is complete
pub use metamask::MetaMaskWallet;
pub use phantom::PhantomWallet;
