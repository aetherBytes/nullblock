// Resource modules for wallet interactions, MCP, agents, crossroads marketplace, engrams, and other services

pub mod types;
pub mod wallets;
pub mod mcp;
pub mod agents;
pub mod users;
pub mod crossroads;
pub mod logs;
pub mod external_service;
pub mod api_keys;
pub mod engrams;
pub mod arb;

// Re-export commonly used types and traits
pub use types::{
    WalletChallengeRequest, WalletChallengeResponse,
    WalletVerifyRequest, WalletVerifyResponse, WalletListResponse
};
pub use wallets::WalletManager;
pub use crossroads::routes::create_crossroads_routes;
pub use engrams::create_engram_routes;
pub use mcp::routes::create_mcp_routes;
pub use arb::create_arb_routes;
pub use external_service::ExternalService;
