use axum::{
    extract::State,
    response::Json,
    routing::get,
    Router,
};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Instant;
use tracing::info;

use super::aggregator::DiscoveryAggregator;
use super::models::{
    AgentsResponse, HealthResponse, HotItemsResponse, ProtocolsResponse, ToolsResponse,
};
use super::providers::{AgentsProvider, ArbFarmProvider, ProtocolsProvider};

pub fn create_discovery_routes() -> Router<crate::AppState> {
    Router::new()
        .route("/api/discovery/tools", get(discover_tools))
        .route("/api/discovery/agents", get(discover_agents_new))
        .route("/api/discovery/protocols", get(discover_protocols))
        .route("/api/discovery/all", get(discover_all))
        .route("/api/discovery/hot", get(discover_hot))
        .route("/api/discovery/health", get(discovery_health))
}

fn create_aggregator() -> DiscoveryAggregator {
    let providers: Vec<Arc<dyn super::aggregator::DiscoveryProvider>> = vec![
        Arc::new(ArbFarmProvider::new()),
        Arc::new(AgentsProvider::new()),
        Arc::new(ProtocolsProvider::new()),
    ];

    DiscoveryAggregator::new(providers)
}

async fn discover_tools(State(_app_state): State<crate::AppState>) -> Json<Value> {
    let start = Instant::now();
    info!("🔧 Discovery: Fetching all MCP tools");

    let aggregator = create_aggregator();
    let tools = aggregator.discover_tools().await;
    let categories = DiscoveryAggregator::get_category_summary(&tools);

    let hot_count = tools.iter().filter(|t| t.is_hot).count();

    let response = ToolsResponse {
        total_count: tools.len(),
        hot_count,
        tools,
        categories,
        discovery_time_ms: start.elapsed().as_millis() as u64,
    };

    Json(json!(response))
}

async fn discover_agents_new(State(_app_state): State<crate::AppState>) -> Json<Value> {
    let start = Instant::now();
    info!("🤖 Discovery: Fetching all agents");

    let aggregator = create_aggregator();
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
    info!("🔌 Discovery: Fetching all protocols");

    let aggregator = create_aggregator();
    let protocols = aggregator.discover_protocols().await;

    let response = ProtocolsResponse {
        total_count: protocols.len(),
        protocols,
        discovery_time_ms: start.elapsed().as_millis() as u64,
    };

    Json(json!(response))
}

async fn discover_all(State(_app_state): State<crate::AppState>) -> Json<Value> {
    info!("🌐 Discovery: Fetching all discoverable items");

    let aggregator = create_aggregator();
    let response = aggregator.discover_all().await;

    Json(json!(response))
}

async fn discover_hot(State(_app_state): State<crate::AppState>) -> Json<Value> {
    let start = Instant::now();
    info!("🔥 Discovery: Fetching hot items");

    let aggregator = create_aggregator();
    let hot_tools = aggregator.get_hot_items().await;

    let response = HotItemsResponse {
        total_count: hot_tools.len(),
        tools: hot_tools,
        discovery_time_ms: start.elapsed().as_millis() as u64,
    };

    Json(json!(response))
}

async fn discovery_health(State(_app_state): State<crate::AppState>) -> Json<Value> {
    info!("🏥 Discovery: Checking provider health");

    let aggregator = create_aggregator();
    let providers = aggregator.get_provider_health().await;
    let overall_status = DiscoveryAggregator::get_overall_health(&providers);

    let response = HealthResponse {
        providers,
        overall_status,
        checked_at: chrono::Utc::now(),
    };

    Json(json!(response))
}
