// Resource modules for wallet interactions, MCP, agents, crossroads marketplace, engrams, and other services

pub mod agents;
pub mod api_keys;
pub mod arb;
pub mod content;
pub mod crossroads;
pub mod discovery;
pub mod engrams;
pub mod external_service;
pub mod mcp;
pub mod types;
pub mod users;
pub mod wallets;

// Re-export commonly used types and traits
pub use arb::create_arb_routes;
pub use content::create_content_routes;
pub use crossroads::routes::create_crossroads_routes;
pub use discovery::create_discovery_routes;
pub use engrams::create_engram_routes;
pub use external_service::ExternalService;
pub use mcp::routes::create_mcp_routes;
pub use types::{
    WalletChallengeRequest, WalletChallengeResponse, WalletListResponse, WalletVerifyRequest,
    WalletVerifyResponse,
};
pub use wallets::WalletManager;
