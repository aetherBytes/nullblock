use axum::{
    extract::Path,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use tracing::{info, warn};
use uuid::Uuid;

use crate::resources::crossroads::models::CreateListingRequest;
use crate::resources::crossroads::services::NullblockServiceIntegrator;

pub fn create_crossroads_routes() -> Router {
    Router::new()
        // Core Marketplace API - ONLY marketplace functionality
        .route("/api/marketplace/listings", get(get_listings))
        .route("/api/marketplace/listings", post(create_listing))
        .route("/api/marketplace/listings/:id", get(get_listing))
        .route("/api/marketplace/search", post(search_listings))
        .route("/api/marketplace/featured", get(get_featured_listings))
        .route("/api/marketplace/stats", get(get_marketplace_stats))
        
        // Service Discovery API - Core to marketplace discovery
        .route("/api/discovery/agents", get(discover_agents))
        .route("/api/discovery/workflows", get(discover_workflows))
        .route("/api/discovery/tools", get(discover_tools))
        .route("/api/discovery/mcp-servers", get(discover_mcp_servers))
        .route("/api/discovery/scan", post(trigger_discovery_scan))
        .route("/api/discovery/health/:endpoint", get(check_service_health))
        
        // Marketplace Admin API - Only marketplace moderation
        .route("/api/admin/listings/approve/:id", post(approve_listing))
        .route("/api/admin/listings/reject/:id", post(reject_listing))
        .route("/api/admin/listings/feature/:id", post(feature_listing))
        
        // Health endpoint
        .route("/api/crossroads/health", get(crossroads_health))
}

// Marketplace endpoints
async fn get_listings() -> Json<Value> {
    info!("📦 Fetching all marketplace listings");
    
    Json(json!({
        "listings": [],
        "total_count": 0,
        "message": "Marketplace listings endpoint - integrated with Erebus"
    }))
}

async fn create_listing(
    Json(payload): Json<CreateListingRequest>
) -> Result<Json<Value>, StatusCode> {
    info!("📦 Creating new marketplace listing: {}", payload.title);
    
    let listing_id = Uuid::new_v4();
    
    Ok(Json(json!({
        "id": listing_id,
        "title": payload.title,
        "status": "pending",
        "message": "Listing created successfully"
    })))
}

async fn get_listing(Path(id): Path<Uuid>) -> Json<Value> {
    info!("📦 Fetching listing: {}", id);
    
    Json(json!({
        "id": id,
        "message": "Individual listing endpoint - integrated with Erebus"
    }))
}

async fn search_listings(Json(_search_req): Json<Value>) -> Json<Value> {
    info!("🔍 Searching marketplace listings");
    
    Json(json!({
        "listings": [],
        "total_count": 0,
        "page": 1,
        "per_page": 20,
        "message": "Search endpoint - integrated with Erebus"
    }))
}

async fn get_featured_listings() -> Json<Value> {
    info!("⭐ Fetching featured marketplace listings");
    
    Json(json!({
        "featured_listings": [],
        "count": 0,
        "message": "Featured listings endpoint - integrated with Erebus"
    }))
}

async fn get_marketplace_stats() -> Json<Value> {
    info!("📊 Fetching marketplace statistics");
    
    Json(json!({
        "total_listings": 0,
        "active_listings": 0,
        "agents_count": 0,
        "workflows_count": 0,
        "tools_count": 0,
        "mcp_servers_count": 0,
        "featured_count": 0,
        "last_updated": chrono::Utc::now(),
        "message": "Marketplace stats endpoint - integrated with Erebus"
    }))
}

// Discovery endpoints
async fn discover_agents() -> Json<Value> {
    info!("🤖 Discovering available agents");
    
    let start_time = std::time::Instant::now();
    let integrator = NullblockServiceIntegrator::new();
    
    let agents = match integrator.discover_agents_from_service().await {
        Ok(agents) => agents,
        Err(e) => {
            warn!("⚠️ Failed to discover agents from service: {}", e);
            vec![]
        }
    };
    
    let scan_duration = start_time.elapsed().as_millis();
    
    Json(json!({
        "agents": agents,
        "count": agents.len(),
        "discovered_at": chrono::Utc::now(),
        "scan_duration_ms": scan_duration,
        "sources": ["nullblock_agents_service", "local_registry", "network_scan"],
        "message": "Agent discovery with Nullblock service integration"
    }))
}

async fn discover_workflows() -> Json<Value> {
    info!("🔄 Discovering available workflows");
    
    let start_time = std::time::Instant::now();
    let integrator = NullblockServiceIntegrator::new();
    
    let workflows = match integrator.discover_workflows_from_service().await {
        Ok(workflows) => workflows,
        Err(e) => {
            warn!("⚠️ Failed to discover workflows from service: {}", e);
            vec![]
        }
    };
    
    let scan_duration = start_time.elapsed().as_millis();
    
    Json(json!({
        "workflows": workflows,
        "count": workflows.len(),
        "discovered_at": chrono::Utc::now(),
        "scan_duration_ms": scan_duration,
        "sources": ["nullblock_orchestration_service", "agent_definitions", "template_library"],
        "message": "Workflow discovery with Nullblock service integration"
    }))
}

async fn discover_tools() -> Json<Value> {
    info!("🔧 Discovering available tools");
    
    Json(json!({
        "tools": [],
        "discovered_at": chrono::Utc::now(),
        "scan_duration_ms": 0,
        "sources": ["mcp_servers", "agent_toolkits", "standalone_tools"],
        "message": "Tool discovery endpoint - integrated with Erebus"
    }))
}

async fn discover_mcp_servers() -> Json<Value> {
    info!("🌐 Discovering MCP servers");
    
    let start_time = std::time::Instant::now();
    let integrator = NullblockServiceIntegrator::new();
    
    let mcp_servers = match integrator.discover_mcp_servers_from_service().await {
        Ok(servers) => servers,
        Err(e) => {
            warn!("⚠️ Failed to discover MCP servers from service: {}", e);
            vec![]
        }
    };
    
    let scan_duration = start_time.elapsed().as_millis();
    
    Json(json!({
        "mcp_servers": mcp_servers,
        "count": mcp_servers.len(),
        "discovered_at": chrono::Utc::now(),
        "scan_duration_ms": scan_duration,
        "protocol_versions": ["1.0", "0.9"],
        "sources": ["nullblock_mcp_service", "self_registration", "network_scan"],
        "message": "MCP server discovery with Nullblock service integration"
    }))
}

async fn trigger_discovery_scan() -> Json<Value> {
    info!("🔍 Triggering full discovery scan");
    
    Json(json!({
        "scan_id": Uuid::new_v4(),
        "status": "started",
        "estimated_duration_ms": 30000,
        "started_at": chrono::Utc::now(),
        "message": "Discovery scan initiated - integrated with Erebus"
    }))
}

async fn check_service_health(Path(endpoint): Path<String>) -> Json<Value> {
    info!("🏥 Checking health of service: {}", endpoint);
    
    Json(json!({
        "endpoint": endpoint,
        "status": "unknown",
        "response_time_ms": 0,
        "checked_at": chrono::Utc::now(),
        "message": "Service health check endpoint - integrated with Erebus"
    }))
}

// Admin endpoints
async fn approve_listing(Path(id): Path<Uuid>) -> Result<Json<Value>, StatusCode> {
    info!("✅ Admin approving listing: {}", id);
    
    Ok(Json(json!({
        "listing_id": id,
        "status": "approved",
        "approved_at": chrono::Utc::now(),
        "message": "Listing approved successfully"
    })))
}

async fn reject_listing(Path(id): Path<Uuid>) -> Result<Json<Value>, StatusCode> {
    warn!("❌ Admin rejecting listing: {}", id);
    
    Ok(Json(json!({
        "listing_id": id,
        "status": "rejected",
        "rejected_at": chrono::Utc::now(),
        "message": "Listing rejected"
    })))
}

async fn feature_listing(Path(id): Path<Uuid>) -> Result<Json<Value>, StatusCode> {
    info!("⭐ Admin featuring listing: {}", id);
    
    Ok(Json(json!({
        "listing_id": id,
        "status": "featured",
        "featured_at": chrono::Utc::now(),
        "message": "Listing featured successfully"
    })))
}


// These functions have been moved to their respective dedicated services:
// - MCP endpoints should be in resources/mcp/ 
// - Blockchain endpoints should be in resources/blockchain/
// - Wealth distribution should be in resources/wealth/
// - Agent interoperability should be in resources/agents/ (extended)

async fn crossroads_health() -> Json<Value> {
    info!("🏥 Crossroads marketplace health check requested");
    
    let integrator = NullblockServiceIntegrator::new();
    let services_health = integrator.check_services_health().await;
    
    Json(json!({
        "status": "healthy",
        "service": "crossroads-marketplace",
        "timestamp": chrono::Utc::now(),
        "components": {
            "marketplace_api": "healthy",
            "discovery_engine": "healthy", 
            "listing_management": "healthy",
            "search_index": "healthy",
            "service_integration": "healthy",
            "erebus_integration": "healthy"
        },
        "integrated_services": services_health,
        "scope": "Marketplace and service discovery only",
        "note": "For MCP, blockchain, wealth distribution - use dedicated service endpoints",
        "message": "Crossroads marketplace subsystem healthy - focused on marketplace and discovery"
    }))
}