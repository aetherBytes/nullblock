pub mod client;
#[cfg(feature = "config-file")]
pub mod config;
pub mod error;
pub mod filter;
pub mod registry;
pub mod types;

pub use client::{AuthConfig, McpClient, McpServerConfig};
pub use error::{McpError, McpResult};
pub use filter::{
    filter_by_tag, filter_idempotent, filter_not_destructive, filter_read_only, ToolFilter,
};
pub use registry::{RegistryStats, ServiceEndpoint, ServiceRegistry};
pub use types::*;

#[cfg(feature = "config-file")]
pub use config::{
    load_config_from_file_or_default, load_registry_from_config, McpServicesConfig, ServiceConfig,
};
