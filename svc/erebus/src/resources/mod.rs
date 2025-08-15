// Resource modules for wallet interactions, MCP, and other services

pub mod types;
pub mod wallets;
pub mod mcp;

// Re-export commonly used types and traits
pub use types::{
    WalletChallengeRequest, WalletChallengeResponse,
    WalletVerifyRequest, WalletVerifyResponse, WalletListResponse
};
pub use wallets::WalletManager;
pub use mcp::{McpHandler, McpRequest, McpResponse};