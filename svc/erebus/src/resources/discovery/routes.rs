use axum::{
    extract::{Path, State},
    response::Json,
    routing::{delete, get, post},
    Router,
};
use nullblock_mcp_client::McpServerConfig;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::info;

use super::aggregator::DiscoveryAggregator;
use super::models::{
    AgentsResponse, HealthResponse, HotItemsResponse, ProtocolsResponse, ToolsResponse,
};
use super::providers::{AgentsProvider, ArbFarmProvider, ExternalMcpProvider, ProtocolsProvider};

lazy_static::lazy_static! {
    static ref EXTERNAL_MCP_PROVIDER: Arc<RwLock<Option<Arc<ExternalMcpProvider>>>> = Arc::new(RwLock::new(None));
}

pub fn create_discovery_routes() -> Router<crate::AppState> {
    Router::new()
        .route("/api/discovery/tools", get(discover_tools))
        .route("/api/discovery/agents", get(discover_agents_new))
        .route("/api/discovery/protocols", get(discover_protocols))
        .route("/api/discovery/all", get(discover_all))
        .route("/api/discovery/hot", get(discover_hot))
        .route("/api/discovery/health", get(discovery_health))
        .route("/api/discovery/external", get(list_external_services))
        .route("/api/discovery/external", post(register_external_service))
        .route(
            "/api/discovery/external/:name",
            delete(unregister_external_service),
        )
        .route(
            "/api/discovery/external/refresh",
            post(refresh_external_tools),
        )
}

async fn get_or_init_external_provider() -> Arc<ExternalMcpProvider> {
    {
        let guard = EXTERNAL_MCP_PROVIDER.read().await;
        if let Some(ref provider) = *guard {
            return provider.clone();
        }
    }

    let mut guard = EXTERNAL_MCP_PROVIDER.write().await;
    if guard.is_none() {
        let provider = Arc::new(ExternalMcpProvider::new());
        if let Err(e) = provider.initialize().await {
            tracing::warn!(error = %e, "Failed to initialize external MCP provider");
        }
        *guard = Some(provider);
    }
    guard.as_ref().unwrap().clone()
}

fn create_aggregator() -> DiscoveryAggregator {
    let providers: Vec<Arc<dyn super::aggregator::DiscoveryProvider>> = vec![
        Arc::new(ArbFarmProvider::new()),
        Arc::new(AgentsProvider::new()),
        Arc::new(ProtocolsProvider::new()),
    ];

    DiscoveryAggregator::new(providers)
}

async fn create_aggregator_with_external() -> DiscoveryAggregator {
    let external_provider = get_or_init_external_provider().await;

    let providers: Vec<Arc<dyn super::aggregator::DiscoveryProvider>> = vec![
        Arc::new(ArbFarmProvider::new()),
        Arc::new(AgentsProvider::new()),
        Arc::new(ProtocolsProvider::new()),
        external_provider,
    ];

    DiscoveryAggregator::new(providers)
}

async fn discover_tools(State(_app_state): State<crate::AppState>) -> Json<Value> {
    let start = Instant::now();
    info!("🔧 Discovery: Fetching all MCP tools (including external)");

    let aggregator = create_aggregator_with_external().await;
    let tools = aggregator.discover_tools().await;
    let categories = DiscoveryAggregator::get_category_summary(&tools);

    let hot_count = tools.iter().filter(|t| t.is_hot).count();
    let external_count = tools
        .iter()
        .filter(|t| t.provider.starts_with("external/"))
        .count();

    let response = ToolsResponse {
        total_count: tools.len(),
        hot_count,
        tools,
        categories,
        discovery_time_ms: start.elapsed().as_millis() as u64,
    };

    info!(
        total = response.total_count,
        external = external_count,
        "Discovery complete"
    );

    Json(json!(response))
}

async fn discover_agents_new(State(_app_state): State<crate::AppState>) -> Json<Value> {
    let start = Instant::now();
    info!("🤖 Discovery: Fetching all agents");

    let aggregator = create_aggregator_with_external().await;
    let agents = aggregator.discover_agents().await;

    let healthy_count = agents
        .iter()
        .filter(|a| matches!(a.status, super::models::HealthStatus::Healthy))
        .count();

    let response = AgentsResponse {
        total_count: agents.len(),
        healthy_count,
        agents,
        discovery_time_ms: start.elapsed().as_millis() as u64,
    };

    Json(json!(response))
}

async fn discover_protocols(State(_app_state): State<crate::AppState>) -> Json<Value> {
    let start = Instant::now();
    info!("🔌 Discovery: Fetching all protocols (including external)");

    let aggregator = create_aggregator_with_external().await;
    let protocols = aggregator.discover_protocols().await;

    let response = ProtocolsResponse {
        total_count: protocols.len(),
        protocols,
        discovery_time_ms: start.elapsed().as_millis() as u64,
    };

    Json(json!(response))
}

async fn discover_all(State(_app_state): State<crate::AppState>) -> Json<Value> {
    info!("🌐 Discovery: Fetching all discoverable items (including external)");

    let aggregator = create_aggregator_with_external().await;
    let response = aggregator.discover_all().await;

    Json(json!(response))
}

async fn discover_hot(State(_app_state): State<crate::AppState>) -> Json<Value> {
    let start = Instant::now();
    info!("🔥 Discovery: Fetching hot items");

    let aggregator = create_aggregator_with_external().await;
    let hot_tools = aggregator.get_hot_items().await;

    let response = HotItemsResponse {
        total_count: hot_tools.len(),
        tools: hot_tools,
        discovery_time_ms: start.elapsed().as_millis() as u64,
    };

    Json(json!(response))
}

async fn discovery_health(State(_app_state): State<crate::AppState>) -> Json<Value> {
    info!("🏥 Discovery: Checking provider health (including external)");

    let aggregator = create_aggregator_with_external().await;
    let providers = aggregator.get_provider_health().await;
    let overall_status = DiscoveryAggregator::get_overall_health(&providers);

    let response = HealthResponse {
        providers,
        overall_status,
        checked_at: chrono::Utc::now(),
    };

    Json(json!(response))
}

#[derive(Debug, Serialize, Deserialize)]
struct ExternalServiceInfo {
    name: String,
    url: String,
    description: Option<String>,
    tags: Vec<String>,
    is_remote: bool,
    has_auth: bool,
    tool_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExternalServicesResponse {
    services: Vec<ExternalServiceInfo>,
    total_count: usize,
    stats: ExternalRegistryStats,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExternalRegistryStats {
    service_count: usize,
    enabled_service_count: usize,
    remote_service_count: usize,
    authenticated_service_count: usize,
    total_tools: usize,
    read_only_tools: usize,
}

async fn list_external_services(State(_app_state): State<crate::AppState>) -> Json<Value> {
    info!("📋 Listing external MCP services");

    let provider = get_or_init_external_provider().await;
    let services = provider.list_services().await;
    let stats = provider.get_registry_stats().await;

    let service_infos: Vec<ExternalServiceInfo> = services
        .into_iter()
        .map(|s| {
            let tool_count = s.tool_count();
            ExternalServiceInfo {
                name: s.name,
                url: s.url,
                description: s.description,
                tags: s.tags,
                is_remote: s.is_remote,
                has_auth: s.has_auth,
                tool_count,
            }
        })
        .collect();

    let response = ExternalServicesResponse {
        total_count: service_infos.len(),
        services: service_infos,
        stats: ExternalRegistryStats {
            service_count: stats.service_count,
            enabled_service_count: stats.enabled_service_count,
            remote_service_count: stats.remote_service_count,
            authenticated_service_count: stats.authenticated_service_count,
            total_tools: stats.total_tools,
            read_only_tools: stats.read_only_tools,
        },
    };

    Json(json!(response))
}

#[derive(Debug, Deserialize)]
struct RegisterServiceRequest {
    name: String,
    url: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    api_key: Option<String>,
    #[serde(default)]
    api_key_header: Option<String>,
    #[serde(default)]
    bearer_token: Option<String>,
}

async fn register_external_service(
    State(_app_state): State<crate::AppState>,
    Json(req): Json<RegisterServiceRequest>,
) -> Json<Value> {
    info!(name = %req.name, url = %req.url, "Registering external MCP service");

    let provider = get_or_init_external_provider().await;

    let mut auth = nullblock_mcp_client::AuthConfig::default();
    if let Some(key) = req.api_key {
        auth.api_key = Some(key);
        auth.api_key_header = req.api_key_header.or(Some("X-API-Key".to_string()));
    }
    if let Some(token) = req.bearer_token {
        auth.bearer_token = Some(token);
    }

    let config = McpServerConfig {
        name: req.name.clone(),
        url: req.url.clone(),
        auth,
        enabled: true,
        description: req.description,
        tags: req.tags,
        timeout_secs: 30,
        cache_ttl_secs: 300,
        health_check_interval_secs: None,
    };

    match provider.register_service(config).await {
        Ok(()) => {
            info!(name = %req.name, "External MCP service registered successfully");
            Json(json!({
                "success": true,
                "message": format!("Service '{}' registered successfully", req.name)
            }))
        }
        Err(e) => {
            info!(name = %req.name, error = %e, "Failed to register external MCP service");
            Json(json!({
                "success": false,
                "error": e.to_string()
            }))
        }
    }
}

async fn unregister_external_service(
    State(_app_state): State<crate::AppState>,
    Path(name): Path<String>,
) -> Json<Value> {
    info!(name = %name, "Unregistering external MCP service");

    let provider = get_or_init_external_provider().await;
    provider.unregister_service(&name).await;

    Json(json!({
        "success": true,
        "message": format!("Service '{}' unregistered", name)
    }))
}

async fn refresh_external_tools(State(_app_state): State<crate::AppState>) -> Json<Value> {
    info!("🔄 Refreshing external MCP tools");

    let provider = get_or_init_external_provider().await;

    match provider.refresh_tools().await {
        Ok(count) => {
            info!(tool_count = count, "External tools refreshed");
            Json(json!({
                "success": true,
                "tool_count": count,
                "message": format!("Refreshed {} tools from external services", count)
            }))
        }
        Err(e) => {
            info!(error = %e, "Failed to refresh external tools");
            Json(json!({
                "success": false,
                "error": e.to_string()
            }))
        }
    }
}
