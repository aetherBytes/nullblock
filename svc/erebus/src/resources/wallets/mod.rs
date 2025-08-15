// Wallet-specific implementations, interaction layer, and routes

pub mod metamask;
pub mod phantom;
pub mod routes;
pub mod wallet_interaction;
pub mod wallet_service;

// Re-export wallet implementations and manager
pub use metamask::MetaMaskWallet;
pub use phantom::PhantomWallet;
pub use wallet_interaction::WalletManager;