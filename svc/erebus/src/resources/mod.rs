// Resource modules for wallet interactions, MCP, agents, crossroads marketplace, and other services

pub mod types;
pub mod wallets;
pub mod mcp;
pub mod agents;
pub mod crossroads;

// Re-export commonly used types and traits
pub use types::{
    WalletChallengeRequest, WalletChallengeResponse,
    WalletVerifyRequest, WalletVerifyResponse, WalletListResponse
};
pub use wallets::WalletManager;
pub use crossroads::routes::create_crossroads_routes;
