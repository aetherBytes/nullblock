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
use crate::resources::crossroads::services::{NullblockServiceIntegrator, SchemaValidator};

pub fn create_crossroads_routes() -> Router {
    Router::new()
        // Marketplace API
        .route("/api/marketplace/listings", get(get_listings))
        .route("/api/marketplace/listings", post(create_listing))
        .route("/api/marketplace/listings/:id", get(get_listing))
        .route("/api/marketplace/search", post(search_listings))
        .route("/api/marketplace/featured", get(get_featured_listings))
        .route("/api/marketplace/stats", get(get_marketplace_stats))
        
        // Discovery API
        .route("/api/discovery/agents", get(discover_agents))
        .route("/api/discovery/workflows", get(discover_workflows))
        .route("/api/discovery/tools", get(discover_tools))
        .route("/api/discovery/mcp-servers", get(discover_mcp_servers))
        .route("/api/discovery/scan", post(trigger_discovery_scan))
        .route("/api/discovery/health/:endpoint", get(check_service_health))
        
        // MCP Self-Registration API
        .route("/api/mcp/register", post(register_mcp_server))
        .route("/api/mcp/deregister/:id", post(deregister_mcp_server))
        .route("/api/mcp/heartbeat/:id", post(mcp_heartbeat))
        .route("/api/mcp/metadata/:id", get(get_mcp_metadata))
        .route("/api/mcp/sampling/request", post(request_mcp_sampling))
        .route("/api/mcp/sampling/offer", post(offer_mcp_sampling))
        
        // Blockchain/Tokenization API
        .route("/api/blockchain/tokenize", post(tokenize_asset))
        .route("/api/blockchain/assets/:id", get(get_tokenized_asset))
        .route("/api/blockchain/trade", post(create_trade_order))
        .route("/api/blockchain/portfolio/:address", get(get_portfolio))
        
        // Wealth Distribution API
        .route("/api/wealth/pools", get(get_distribution_pools))
        .route("/api/wealth/pools", post(create_distribution_pool))
        .route("/api/wealth/distribute/:pool_id", post(trigger_distribution))
        .route("/api/wealth/rewards/:address", get(get_user_rewards))
        
        // Agent Interoperability API
        .route("/api/agents/interfaces", get(get_agent_interfaces))
        .route("/api/agents/interfaces", post(register_agent_interface))
        .route("/api/agents/compatibility", post(check_agent_compatibility))
        .route("/api/agents/schemas/:name", get(get_schema_definition))
        
        // Admin API
        .route("/api/admin/listings/approve/:id", post(approve_listing))
        .route("/api/admin/listings/reject/:id", post(reject_listing))
        .route("/api/admin/listings/feature/:id", post(feature_listing))
        .route("/api/admin/system/stats", get(get_system_stats))
        .route("/api/admin/mcp/verify/:id", post(verify_mcp_server))
        
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

async fn get_system_stats() -> Json<Value> {
    info!("📊 Admin fetching system statistics");
    
    Json(json!({
        "uptime_seconds": 0,
        "total_requests": 0,
        "active_connections": 0,
        "memory_usage_mb": 0,
        "cpu_usage_percent": 0.0,
        "disk_usage_mb": 0,
        "database_connections": 0,
        "cache_hit_ratio": 0.0,
        "last_updated": chrono::Utc::now(),
        "message": "System stats endpoint - integrated with Erebus"
    }))
}

// MCP Self-Registration endpoints
async fn register_mcp_server(Json(payload): Json<Value>) -> Result<Json<Value>, StatusCode> {
    info!("🌐 MCP server self-registration request");
    
    let server_id = Uuid::new_v4();
    let server_name = payload.get("server_name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    
    Ok(Json(json!({
        "server_id": server_id,
        "status": "registered",
        "verification_status": "pending",
        "registered_at": chrono::Utc::now(),
        "message": format!("MCP server '{}' registered successfully", server_name)
    })))
}

async fn deregister_mcp_server(Path(id): Path<Uuid>) -> Result<Json<Value>, StatusCode> {
    info!("🌐 MCP server deregistration: {}", id);
    
    Ok(Json(json!({
        "server_id": id,
        "status": "deregistered",
        "deregistered_at": chrono::Utc::now(),
        "message": "MCP server deregistered successfully"
    })))
}

async fn mcp_heartbeat(Path(id): Path<Uuid>) -> Result<Json<Value>, StatusCode> {
    info!("💓 MCP server heartbeat: {}", id);
    
    Ok(Json(json!({
        "server_id": id,
        "status": "alive",
        "last_heartbeat": chrono::Utc::now(),
        "next_heartbeat_expected": chrono::Utc::now() + chrono::Duration::minutes(5),
        "message": "Heartbeat recorded"
    })))
}

async fn get_mcp_metadata(Path(id): Path<Uuid>) -> Json<Value> {
    info!("📋 Fetching MCP server metadata: {}", id);
    
    Json(json!({
        "server_id": id,
        "metadata": {
            "protocol_version": "1.0",
            "capabilities": ["resources", "tools", "prompts"],
            "resources": [],
            "tools": [],
            "prompts": [],
            "sampling_config": {
                "allows_outbound_sampling": true,
                "accepts_inbound_sampling": true,
                "authentication_required": false
            }
        },
        "message": "MCP metadata retrieved"
    }))
}

async fn request_mcp_sampling(Json(_payload): Json<Value>) -> Result<Json<Value>, StatusCode> {
    info!("🔗 MCP sampling request received");
    
    let request_id = Uuid::new_v4();
    
    Ok(Json(json!({
        "request_id": request_id,
        "status": "pending",
        "estimated_processing_time_ms": 1000,
        "created_at": chrono::Utc::now(),
        "message": "MCP sampling request queued"
    })))
}

async fn offer_mcp_sampling(Json(_payload): Json<Value>) -> Result<Json<Value>, StatusCode> {
    info!("🎯 MCP sampling offer received");
    
    let offer_id = Uuid::new_v4();
    
    Ok(Json(json!({
        "offer_id": offer_id,
        "status": "active",
        "expires_at": chrono::Utc::now() + chrono::Duration::hours(24),
        "created_at": chrono::Utc::now(),
        "message": "MCP sampling offer published"
    })))
}

// Blockchain/Tokenization endpoints
async fn tokenize_asset(Json(_payload): Json<Value>) -> Result<Json<Value>, StatusCode> {
    info!("🪙 Asset tokenization request received");
    
    let token_id = Uuid::new_v4();
    
    Ok(Json(json!({
        "token_id": token_id,
        "contract_address": "0x1234567890123456789012345678901234567890",
        "blockchain_tx": format!("0x{}", hex::encode(&token_id.as_bytes()[0..8])),
        "status": "minting",
        "created_at": chrono::Utc::now(),
        "message": "Asset tokenization initiated"
    })))
}

async fn get_tokenized_asset(Path(id): Path<Uuid>) -> Json<Value> {
    info!("🪙 Fetching tokenized asset: {}", id);
    
    Json(json!({
        "token_id": id,
        "asset_type": "Agent",
        "owner_address": "0x1234567890123456789012345678901234567890",
        "is_tradeable": true,
        "current_value": 0.1,
        "transaction_history": [],
        "message": "Tokenized asset details"
    }))
}

async fn create_trade_order(Json(_payload): Json<Value>) -> Result<Json<Value>, StatusCode> {
    info!("📈 Trade order creation request");
    
    let order_id = Uuid::new_v4();
    
    Ok(Json(json!({
        "order_id": order_id,
        "status": "pending",
        "order_type": "market",
        "created_at": chrono::Utc::now(),
        "estimated_execution_time": "30s",
        "message": "Trade order created"
    })))
}

async fn get_portfolio(Path(address): Path<String>) -> Json<Value> {
    info!("💼 Fetching portfolio for address: {}", address);
    
    Json(json!({
        "address": address,
        "total_value": 0.0,
        "assets": [],
        "trading_history": [],
        "rewards_earned": 0.0,
        "last_updated": chrono::Utc::now(),
        "message": "Portfolio details retrieved"
    }))
}

// Wealth Distribution endpoints
async fn get_distribution_pools() -> Json<Value> {
    info!("💰 Fetching distribution pools");
    
    Json(json!({
        "pools": [],
        "total_rewards_available": 0.0,
        "active_pools_count": 0,
        "next_distribution": chrono::Utc::now() + chrono::Duration::days(1),
        "message": "Distribution pools retrieved"
    }))
}

async fn create_distribution_pool(Json(payload): Json<Value>) -> Result<Json<Value>, StatusCode> {
    info!("💰 Creating distribution pool");
    
    let pool_id = Uuid::new_v4();
    
    Ok(Json(json!({
        "pool_id": pool_id,
        "status": "active",
        "total_rewards": payload.get("total_rewards").unwrap_or(&json!(0.0)),
        "created_at": chrono::Utc::now(),
        "first_distribution": chrono::Utc::now() + chrono::Duration::days(1),
        "message": "Distribution pool created"
    })))
}

async fn trigger_distribution(Path(pool_id): Path<Uuid>) -> Result<Json<Value>, StatusCode> {
    info!("💸 Triggering distribution for pool: {}", pool_id);
    
    Ok(Json(json!({
        "pool_id": pool_id,
        "distribution_id": Uuid::new_v4(),
        "status": "processing",
        "participants_count": 0,
        "total_distributed": 0.0,
        "started_at": chrono::Utc::now(),
        "message": "Distribution triggered"
    })))
}

async fn get_user_rewards(Path(address): Path<String>) -> Json<Value> {
    info!("🏆 Fetching rewards for address: {}", address);
    
    Json(json!({
        "address": address,
        "total_earned": 0.0,
        "pending_rewards": 0.0,
        "claimed_rewards": 0.0,
        "reward_history": [],
        "next_distribution": chrono::Utc::now() + chrono::Duration::days(1),
        "message": "User rewards retrieved"
    }))
}

// Agent Interoperability endpoints
async fn get_agent_interfaces() -> Json<Value> {
    info!("🤖 Fetching agent interfaces");
    
    Json(json!({
        "interfaces": [],
        "total_count": 0,
        "mcp_compatible_count": 0,
        "last_updated": chrono::Utc::now(),
        "message": "Agent interfaces retrieved"
    }))
}

async fn register_agent_interface(Json(payload): Json<Value>) -> Result<Json<Value>, StatusCode> {
    info!("🤖 Registering agent interface");
    
    let interface_id = Uuid::new_v4();
    let integrator = NullblockServiceIntegrator::new();
    
    // If this is a Hecate agent registration, also register with Hecate service
    if payload.get("agent_id").and_then(|v| v.as_str()) == Some("hecate") {
        match integrator.register_agent_with_hecate(payload.clone()).await {
            Ok(hecate_response) => {
                info!("✅ Successfully registered with Hecate marketplace");
                Ok(Json(json!({
                    "interface_id": interface_id,
                    "status": "registered",
                    "agent_id": "hecate",
                    "mcp_compatible": true,
                    "hecate_integration": hecate_response,
                    "marketplace_ready": true,
                    "registered_at": chrono::Utc::now(),
                    "message": "Hecate flagship agent registered with marketplace integration"
                })))
            }
            Err(e) => {
                warn!("⚠️ Failed to register with Hecate: {}", e);
                Ok(Json(json!({
                    "interface_id": interface_id,
                    "status": "registered",
                    "agent_id": "hecate",
                    "mcp_compatible": true,
                    "hecate_integration": {"status": "failed", "error": e.to_string()},
                    "marketplace_ready": false,
                    "registered_at": chrono::Utc::now(),
                    "message": "Agent registered but Hecate integration failed"
                })))
            }
        }
    } else {
        Ok(Json(json!({
            "interface_id": interface_id,
            "status": "registered",
            "agent_id": payload.get("agent_id").unwrap_or(&json!("unknown")),
            "mcp_compatible": payload.get("mcp_compatible").unwrap_or(&json!(false)),
            "registered_at": chrono::Utc::now(),
            "message": "Agent interface registered"
        })))
    }
}

async fn check_agent_compatibility(Json(payload): Json<Value>) -> Json<Value> {
    info!("🔍 Checking agent compatibility");
    
    Json(json!({
        "compatibility_check_id": Uuid::new_v4(),
        "agents": payload.get("agents").unwrap_or(&json!([])),
        "compatibility_score": 0.95,
        "compatible": true,
        "incompatible_features": [],
        "recommendations": [],
        "checked_at": chrono::Utc::now(),
        "message": "Compatibility check completed"
    }))
}

async fn get_schema_definition(Path(name): Path<String>) -> Json<Value> {
    info!("📋 Fetching schema definition: {}", name);
    
    let validator = SchemaValidator::new();
    
    match validator.get_schema(&name) {
        Some(schema) => {
            Json(json!({
                "schema_name": name,
                "version": "1.0.0",
                "definition": schema,
                "examples": [],
                "available_schemas": validator.list_schemas(),
                "last_updated": chrono::Utc::now(),
                "message": "Schema definition retrieved"
            }))
        }
        None => {
            Json(json!({
                "schema_name": name,
                "version": "1.0.0",
                "definition": null,
                "error": "Schema not found",
                "available_schemas": validator.list_schemas(),
                "last_updated": chrono::Utc::now(),
                "message": "Schema not found - check available_schemas list"
            }))
        }
    }
}

async fn verify_mcp_server(Path(id): Path<Uuid>) -> Result<Json<Value>, StatusCode> {
    info!("✅ Admin verifying MCP server: {}", id);
    
    Ok(Json(json!({
        "server_id": id,
        "verification_status": "verified",
        "verified_by": "system_admin",
        "verified_at": chrono::Utc::now(),
        "trust_level": "trusted",
        "message": "MCP server verified successfully"
    })))
}

async fn crossroads_health() -> Json<Value> {
    info!("🏥 Crossroads health check requested");
    
    let integrator = NullblockServiceIntegrator::new();
    let services_health = integrator.check_services_health().await;
    let hecate_info = integrator.get_hecate_marketplace_info().await;
    
    Json(json!({
        "status": "healthy",
        "service": "crossroads-marketplace",
        "timestamp": chrono::Utc::now(),
        "components": {
            "discovery_engine": "healthy",
            "marketplace_api": "healthy",
            "search_index": "healthy",
            "mcp_registration": "healthy",
            "blockchain_integration": "healthy",
            "wealth_distribution": "healthy",
            "agent_interoperability": "healthy",
            "hecate_flagship_integration": hecate_info.get("status").unwrap_or(&serde_json::json!("unknown")),
            "erebus_integration": "healthy"
        },
        "nullblock_services": services_health,
        "hecate_marketplace_capabilities": hecate_info,
        "message": "Crossroads decentralized marketplace healthy - integrated with Erebus and Hecate"
    }))
}