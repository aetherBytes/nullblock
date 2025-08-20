// Resource modules for wallet interactions, MCP, agents, and other services

pub mod types;
pub mod wallets;
pub mod mcp;
pub mod agents;

// Re-export commonly used types and traits
pub use types::{
    WalletChallengeRequest, WalletChallengeResponse,
    WalletVerifyRequest, WalletVerifyResponse, WalletListResponse
};
pub use wallets::WalletManager;
