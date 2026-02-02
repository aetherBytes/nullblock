use crate::client::{AuthConfig, McpServerConfig};
use crate::error::{McpError, McpResult};
use crate::registry::ServiceRegistry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::Path;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServicesConfig {
    #[serde(default)]
    pub services: HashMap<String, ServiceConfig>,
    #[serde(default)]
    pub defaults: DefaultConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DefaultConfig {
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl_secs: u64,
    #[serde(default)]
    pub auto_refresh: bool,
}

fn default_timeout() -> u64 {
    30
}

fn default_cache_ttl() -> u64 {
    300
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub url: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub auto_refresh: bool,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub auth: AuthConfigToml,
    #[serde(default)]
    pub timeout_secs: Option<u64>,
    #[serde(default)]
    pub cache_ttl_secs: Option<u64>,
    #[serde(default)]
    pub health_check_interval_secs: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthConfigToml {
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub api_key_env: Option<String>,
    #[serde(default)]
    pub api_key_header: Option<String>,
    #[serde(default)]
    pub bearer_token: Option<String>,
    #[serde(default)]
    pub bearer_token_env: Option<String>,
    #[serde(default)]
    pub custom_headers: HashMap<String, String>,
}

impl AuthConfigToml {
    pub fn resolve(&self) -> AuthConfig {
        let api_key = self.api_key.clone().or_else(|| {
            self.api_key_env.as_ref().and_then(|env_var| {
                env::var(env_var).ok().map(|v| {
                    debug!(env_var = %env_var, "Loaded API key from environment");
                    v
                })
            })
        });

        let bearer_token = self.bearer_token.clone().or_else(|| {
            self.bearer_token_env.as_ref().and_then(|env_var| {
                env::var(env_var).ok().map(|v| {
                    debug!(env_var = %env_var, "Loaded bearer token from environment");
                    v
                })
            })
        });

        AuthConfig {
            api_key,
            api_key_header: self.api_key_header.clone(),
            bearer_token,
            custom_headers: self.custom_headers.clone(),
        }
    }
}

impl ServiceConfig {
    pub fn to_server_config(&self, name: &str, defaults: &DefaultConfig) -> McpServerConfig {
        McpServerConfig {
            name: name.to_string(),
            url: resolve_url_env(&self.url),
            auth: self.auth.resolve(),
            enabled: self.enabled,
            description: self.description.clone(),
            tags: self.tags.clone(),
            timeout_secs: self.timeout_secs.unwrap_or(defaults.timeout_secs),
            cache_ttl_secs: self.cache_ttl_secs.unwrap_or(defaults.cache_ttl_secs),
            health_check_interval_secs: self.health_check_interval_secs,
        }
    }
}

fn resolve_url_env(url: &str) -> String {
    if url.starts_with("${") && url.ends_with("}") {
        let env_var = &url[2..url.len() - 1];
        match env::var(env_var) {
            Ok(value) => {
                debug!(env_var = %env_var, "Resolved URL from environment");
                value
            }
            Err(_) => {
                warn!(env_var = %env_var, "Environment variable not found for URL, using as-is");
                url.to_string()
            }
        }
    } else {
        url.to_string()
    }
}

fn default_true() -> bool {
    true
}

impl McpServicesConfig {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> McpResult<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| McpError::ConfigError(format!("Failed to read config file: {}", e)))?;

        Self::load_from_str(&content)
    }

    pub fn load_from_str(content: &str) -> McpResult<Self> {
        toml::from_str(content)
            .map_err(|e| McpError::ConfigError(format!("Failed to parse config: {}", e)))
    }

    pub fn get_server_configs(&self) -> Vec<McpServerConfig> {
        self.services
            .iter()
            .filter(|(_, cfg)| cfg.enabled)
            .map(|(name, cfg)| cfg.to_server_config(name, &self.defaults))
            .collect()
    }

    pub fn to_registry(&self) -> ServiceRegistry {
        let configs = self.get_server_configs();
        ServiceRegistry::from_server_configs(configs)
    }

    pub async fn into_registry_with_discovery(self) -> McpResult<ServiceRegistry> {
        let registry = self.to_registry();
        registry.discover_all_tools().await?;
        Ok(registry)
    }

    pub fn merge_from_env(&mut self) {
        if let Ok(services_json) = env::var("MCP_SERVICES") {
            match serde_json::from_str::<HashMap<String, ServiceConfig>>(&services_json) {
                Ok(env_services) => {
                    info!(
                        count = env_services.len(),
                        "Loaded MCP services from MCP_SERVICES env"
                    );
                    self.services.extend(env_services);
                }
                Err(e) => {
                    warn!(error = %e, "Failed to parse MCP_SERVICES env var");
                }
            }
        }
    }

    pub fn add_service(&mut self, name: impl Into<String>, config: ServiceConfig) {
        self.services.insert(name.into(), config);
    }

    pub fn remove_service(&mut self, name: &str) -> Option<ServiceConfig> {
        self.services.remove(name)
    }

    pub fn enable_service(&mut self, name: &str) -> bool {
        if let Some(config) = self.services.get_mut(name) {
            config.enabled = true;
            true
        } else {
            false
        }
    }

    pub fn disable_service(&mut self, name: &str) -> bool {
        if let Some(config) = self.services.get_mut(name) {
            config.enabled = false;
            true
        } else {
            false
        }
    }
}

impl Default for McpServicesConfig {
    fn default() -> Self {
        let mut services = HashMap::new();

        services.insert(
            "arbfarm".to_string(),
            ServiceConfig {
                url: "http://localhost:9007/mcp/jsonrpc".to_string(),
                enabled: true,
                auto_refresh: true,
                description: Some("ArbFarm MEV agent swarm".to_string()),
                tags: vec!["trading".to_string(), "mev".to_string()],
                auth: AuthConfigToml::default(),
                timeout_secs: None,
                cache_ttl_secs: None,
                health_check_interval_secs: None,
            },
        );

        services.insert(
            "engrams".to_string(),
            ServiceConfig {
                url: "http://localhost:9004/mcp/jsonrpc".to_string(),
                enabled: true,
                auto_refresh: true,
                description: Some("Memory and context layer".to_string()),
                tags: vec!["memory".to_string(), "context".to_string()],
                auth: AuthConfigToml::default(),
                timeout_secs: None,
                cache_ttl_secs: None,
                health_check_interval_secs: None,
            },
        );

        Self {
            services,
            defaults: DefaultConfig::default(),
        }
    }
}

impl ServiceConfig {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            enabled: true,
            auto_refresh: false,
            description: None,
            tags: Vec::new(),
            auth: AuthConfigToml::default(),
            timeout_secs: None,
            cache_ttl_secs: None,
            health_check_interval_secs: None,
        }
    }

    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.auth.api_key = Some(key.into());
        self
    }

    pub fn with_api_key_env(mut self, env_var: impl Into<String>) -> Self {
        self.auth.api_key_env = Some(env_var.into());
        self
    }

    pub fn with_bearer_token(mut self, token: impl Into<String>) -> Self {
        self.auth.bearer_token = Some(token.into());
        self
    }

    pub fn with_bearer_token_env(mut self, env_var: impl Into<String>) -> Self {
        self.auth.bearer_token_env = Some(env_var.into());
        self
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

pub async fn load_registry_from_config<P: AsRef<Path>>(
    config_path: Option<P>,
) -> McpResult<ServiceRegistry> {
    let mut config = match config_path {
        Some(path) => {
            info!(path = ?path.as_ref(), "Loading MCP services config from file");
            McpServicesConfig::load_from_file(path)?
        }
        None => {
            info!("Using default MCP services config");
            McpServicesConfig::default()
        }
    };

    config.merge_from_env();
    config.into_registry_with_discovery().await
}

pub fn load_config_from_file_or_default<P: AsRef<Path>>(
    config_path: Option<P>,
) -> McpResult<McpServicesConfig> {
    let mut config = match config_path {
        Some(path) => {
            info!(path = ?path.as_ref(), "Loading MCP services config from file");
            McpServicesConfig::load_from_file(path)?
        }
        None => {
            info!("Using default MCP services config");
            McpServicesConfig::default()
        }
    };

    config.merge_from_env();
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config() {
        let toml = r#"
[services.arbfarm]
url = "http://localhost:9007/mcp/jsonrpc"
enabled = true
auto_refresh = true

[services.engrams]
url = "http://localhost:9004/mcp/jsonrpc"
enabled = true

[services.disabled_service]
url = "http://localhost:9999/mcp/jsonrpc"
enabled = false
"#;

        let config = McpServicesConfig::load_from_str(toml).unwrap();
        assert_eq!(config.services.len(), 3);
        assert!(config.services.get("arbfarm").unwrap().enabled);
        assert!(!config.services.get("disabled_service").unwrap().enabled);
    }

    #[test]
    fn test_parse_config_with_auth() {
        let toml = r#"
[defaults]
timeout_secs = 60
cache_ttl_secs = 600

[services.external]
url = "https://api.example.com/mcp/jsonrpc"
enabled = true
description = "External MCP server"
tags = ["external", "production"]

[services.external.auth]
api_key = "my-secret-key"
api_key_header = "X-Custom-Key"

[services.another]
url = "${MCP_ANOTHER_URL}"
enabled = true

[services.another.auth]
bearer_token_env = "ANOTHER_TOKEN"
"#;

        let config = McpServicesConfig::load_from_str(toml).unwrap();
        assert_eq!(config.services.len(), 2);

        let external = config.services.get("external").unwrap();
        assert_eq!(external.auth.api_key.as_deref(), Some("my-secret-key"));
        assert_eq!(
            external.auth.api_key_header.as_deref(),
            Some("X-Custom-Key")
        );
        assert_eq!(external.description.as_deref(), Some("External MCP server"));
        assert_eq!(external.tags.len(), 2);

        let another = config.services.get("another").unwrap();
        assert_eq!(
            another.auth.bearer_token_env.as_deref(),
            Some("ANOTHER_TOKEN")
        );

        assert_eq!(config.defaults.timeout_secs, 60);
        assert_eq!(config.defaults.cache_ttl_secs, 600);
    }

    #[test]
    fn test_default_config() {
        let config = McpServicesConfig::default();
        assert!(config.services.contains_key("arbfarm"));
        assert!(config.services.contains_key("engrams"));
    }

    #[test]
    fn test_service_config_builder() {
        let config = ServiceConfig::new("https://api.example.com/mcp")
            .with_api_key("test-key")
            .with_description("Test server")
            .with_tags(vec!["test".to_string()]);

        assert_eq!(config.auth.api_key.as_deref(), Some("test-key"));
        assert_eq!(config.description.as_deref(), Some("Test server"));
    }

    #[test]
    fn test_get_server_configs() {
        let toml = r#"
[services.enabled]
url = "http://localhost:9007/mcp/jsonrpc"
enabled = true

[services.disabled]
url = "http://localhost:9008/mcp/jsonrpc"
enabled = false
"#;
        let config = McpServicesConfig::load_from_str(toml).unwrap();
        let server_configs = config.get_server_configs();

        assert_eq!(server_configs.len(), 1);
        assert_eq!(server_configs[0].name, "enabled");
    }

    #[test]
    fn test_auth_config_resolve() {
        let auth = AuthConfigToml {
            api_key: Some("direct-key".to_string()),
            api_key_env: Some("UNUSED_ENV".to_string()),
            api_key_header: Some("X-API-Key".to_string()),
            bearer_token: None,
            bearer_token_env: None,
            custom_headers: HashMap::new(),
        };

        let resolved = auth.resolve();
        assert_eq!(resolved.api_key.as_deref(), Some("direct-key"));
    }
}
