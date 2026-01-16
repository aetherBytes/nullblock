use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use tracing::{info, warn};
use uuid::Uuid;
use std::sync::Arc;

use crate::resources::crossroads::models::{
    CreateListingRequest, CreateArbFarmCowRequest, ForkArbFarmCowRequest,
};
use crate::resources::crossroads::repository::ArbFarmRepository;
use crate::resources::crossroads::services::NullblockServiceIntegrator;
use crate::resources::ExternalService;

pub fn create_crossroads_routes(_external_service: &Arc<ExternalService>) -> Router<crate::AppState> {
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

        // ArbFarm COW API - MEV Strategy Marketplace
        .route("/api/marketplace/arbfarm/cows", get(list_arbfarm_cows))
        .route("/api/marketplace/arbfarm/cows", post(create_arbfarm_cow))
        .route("/api/marketplace/arbfarm/cows/:id", get(get_arbfarm_cow))
        .route("/api/marketplace/arbfarm/cows/:id/fork", post(fork_arbfarm_cow))
        .route("/api/marketplace/arbfarm/cows/:id/strategies", get(get_arbfarm_cow_strategies))
        .route("/api/marketplace/arbfarm/cows/:id/forks", get(get_arbfarm_cow_forks))
        .route("/api/marketplace/arbfarm/cows/:id/revenue", get(get_arbfarm_cow_revenue))
        .route("/api/marketplace/arbfarm/earnings/:wallet", get(get_arbfarm_earnings))
        .route("/api/marketplace/arbfarm/stats", get(get_arbfarm_stats))

        // Wallet Stash API - Tool ownership and COW tab unlocks
        .route("/api/marketplace/wallet/:address/stash", get(get_wallet_stash))
        .route("/api/marketplace/wallet/:address/unlocks", get(get_wallet_unlocks))

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
async fn discover_agents(State(app_state): State<crate::AppState>) -> Json<Value> {
    info!("🤖 Discovering available agents");
    
    let start_time = std::time::Instant::now();
    let integrator = NullblockServiceIntegrator::new(app_state.external_service.clone());
    
    let agents = match integrator.discover_agents_from_service().await {
        Ok(agents) => agents,
        Err(e) => {
            warn!("❌ Failed to discover agents: {}", e);
            vec![]
        }
    };
    
    let response = json!({
        "agents": agents,
        "total_count": agents.len(),
        "discovery_time_ms": start_time.elapsed().as_millis(),
        "message": "Agent discovery completed"
    });
    
    Json(response)
}

async fn discover_workflows(State(app_state): State<crate::AppState>) -> Json<Value> {
    info!("🔄 Discovering available workflows");
    
    let start_time = std::time::Instant::now();
    let integrator = NullblockServiceIntegrator::new(app_state.external_service.clone());
    
    let workflows = match integrator.discover_workflows_from_service().await {
        Ok(workflows) => workflows,
        Err(e) => {
            warn!("❌ Failed to discover workflows: {}", e);
            vec![]
        }
    };
    
    let response = json!({
        "workflows": workflows,
        "total_count": workflows.len(),
        "discovery_time_ms": start_time.elapsed().as_millis(),
        "message": "Workflow discovery completed"
    });
    
    Json(response)
}

async fn discover_tools(State(_app_state): State<crate::AppState>) -> Json<Value> {
    info!("🔧 Discovering available tools");
    
    Json(json!({
        "tools": [],
        "total_count": 0,
        "message": "Tool discovery endpoint - integrated with Erebus"
    }))
}

async fn discover_mcp_servers(State(app_state): State<crate::AppState>) -> Json<Value> {
    info!("🌐 Discovering MCP servers");
    
    let start_time = std::time::Instant::now();
    let integrator = NullblockServiceIntegrator::new(app_state.external_service.clone());
    
    let mcp_servers = match integrator.discover_mcp_servers_from_service().await {
        Ok(servers) => servers,
        Err(e) => {
            warn!("❌ Failed to discover MCP servers: {}", e);
            vec![]
        }
    };
    
    let response = json!({
        "mcp_servers": mcp_servers,
        "total_count": mcp_servers.len(),
        "discovery_time_ms": start_time.elapsed().as_millis(),
        "message": "MCP server discovery completed"
    });
    
    Json(response)
}

async fn trigger_discovery_scan(State(_app_state): State<crate::AppState>) -> Json<Value> {
    info!("🔍 Triggering full discovery scan");
    
    Json(json!({
        "scan_id": uuid::Uuid::new_v4(),
        "status": "initiated",
        "message": "Discovery scan initiated"
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

async fn crossroads_health(State(app_state): State<crate::AppState>) -> Json<Value> {
    info!("🏥 Crossroads marketplace health check requested");

    let integrator = NullblockServiceIntegrator::new(app_state.external_service.clone());
    let services_health = integrator.check_services_health().await;

    Json(services_health)
}

// =============================================================================
// ArbFarm COW Endpoints - MEV Strategy Marketplace
// =============================================================================

async fn list_arbfarm_cows(
    State(app_state): State<crate::AppState>,
) -> Result<Json<Value>, StatusCode> {
    info!("🐄 Listing ArbFarm COWs");

    let pool = app_state.database.pool();
    match ArbFarmRepository::list_cows(pool, 20, 0, Some(true), None).await {
        Ok(cows) => {
            let count = cows.len();
            Ok(Json(json!({
                "cows": cows,
                "total_count": count,
                "page": 1,
                "per_page": 20
            })))
        }
        Err(e) => {
            warn!("❌ Failed to list ArbFarm COWs: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn create_arbfarm_cow(
    State(app_state): State<crate::AppState>,
    Json(payload): Json<CreateArbFarmCowRequest>,
) -> Result<Json<Value>, StatusCode> {
    info!("🐄 Creating ArbFarm COW: {}", payload.name);

    let pool = app_state.database.pool();
    let creator_wallet = "anonymous";

    match ArbFarmRepository::create_cow(pool, &payload, creator_wallet).await {
        Ok(cow) => Ok(Json(json!({
            "id": cow.id,
            "listing_id": cow.listing_id,
            "name": cow.name,
            "description": cow.description,
            "strategies": cow.strategies,
            "venue_types": cow.venue_types,
            "risk_profile": cow.risk_profile,
            "is_public": cow.is_public,
            "is_forkable": cow.is_forkable,
            "status": "pending",
            "created_at": cow.created_at,
            "message": "ArbFarm COW created successfully - pending approval"
        }))),
        Err(e) => {
            warn!("❌ Failed to create ArbFarm COW: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_arbfarm_cow(
    State(app_state): State<crate::AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    info!("🐄 Fetching ArbFarm COW: {}", id);

    let pool = app_state.database.pool();
    match ArbFarmRepository::get_cow_by_id(pool, id).await {
        Ok(Some(cow)) => Ok(Json(json!(cow))),
        Ok(None) => {
            warn!("❌ ArbFarm COW not found: {}", id);
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            warn!("❌ Failed to fetch ArbFarm COW: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn fork_arbfarm_cow(
    State(app_state): State<crate::AppState>,
    Path(parent_id): Path<Uuid>,
    Json(payload): Json<ForkArbFarmCowRequest>,
) -> Result<Json<Value>, StatusCode> {
    let fork_name = payload.name.clone().unwrap_or_else(|| format!("Fork of {}", parent_id));
    info!("🍴 Forking ArbFarm COW: {} as {}", parent_id, fork_name);

    let pool = app_state.database.pool();
    let forker_wallet = "anonymous";

    match ArbFarmRepository::fork_cow(
        pool,
        parent_id,
        forker_wallet,
        payload.name,
        payload.description,
        payload.inherit_engrams,
        payload.engram_filters,
    )
    .await
    {
        Ok((cow, fork)) => Ok(Json(json!({
            "fork_id": fork.id,
            "forked_cow_id": cow.id,
            "parent_cow_id": parent_id,
            "name": cow.name,
            "description": cow.description,
            "inherit_engrams": payload.inherit_engrams,
            "inherited_strategies": fork.inherited_strategies,
            "inherited_engrams": fork.inherited_engrams,
            "status": "pending",
            "created_at": cow.created_at,
            "message": "ArbFarm COW forked successfully - pending approval"
        }))),
        Err(e) => {
            warn!("❌ Failed to fork ArbFarm COW: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_arbfarm_cow_strategies(
    State(app_state): State<crate::AppState>,
    Path(cow_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    info!("📊 Fetching strategies for ArbFarm COW: {}", cow_id);

    let pool = app_state.database.pool();
    match ArbFarmRepository::get_strategies_for_cow(pool, cow_id).await {
        Ok(strategies) => {
            let count = strategies.len();
            Ok(Json(json!({
                "cow_id": cow_id,
                "strategies": strategies,
                "total_count": count
            })))
        }
        Err(e) => {
            warn!("❌ Failed to fetch strategies: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_arbfarm_cow_forks(
    State(app_state): State<crate::AppState>,
    Path(cow_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    info!("🍴 Fetching forks of ArbFarm COW: {}", cow_id);

    let pool = app_state.database.pool();
    match ArbFarmRepository::get_forks_for_cow(pool, cow_id).await {
        Ok(forks) => {
            let count = forks.len();
            Ok(Json(json!({
                "parent_cow_id": cow_id,
                "forks": forks,
                "total_count": count
            })))
        }
        Err(e) => {
            warn!("❌ Failed to fetch forks: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_arbfarm_cow_revenue(
    State(app_state): State<crate::AppState>,
    Path(cow_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    info!("💰 Fetching revenue for ArbFarm COW: {}", cow_id);

    let pool = app_state.database.pool();
    match ArbFarmRepository::get_revenue_for_cow(pool, cow_id).await {
        Ok(revenues) => {
            let total: i64 = revenues.iter().map(|r| r.amount_lamports).sum();
            let trading: i64 = revenues
                .iter()
                .filter(|r| matches!(r.revenue_type, crate::resources::crossroads::models::ArbFarmRevenueType::TradingProfit))
                .map(|r| r.amount_lamports)
                .sum();
            let fork_fees: i64 = revenues
                .iter()
                .filter(|r| matches!(r.revenue_type, crate::resources::crossroads::models::ArbFarmRevenueType::ForkFee))
                .map(|r| r.amount_lamports)
                .sum();
            let royalties: i64 = revenues
                .iter()
                .filter(|r| matches!(r.revenue_type, crate::resources::crossroads::models::ArbFarmRevenueType::CreatorRoyalty))
                .map(|r| r.amount_lamports)
                .sum();
            let pending: i64 = revenues
                .iter()
                .filter(|r| !r.is_distributed)
                .map(|r| r.amount_lamports)
                .sum();

            Ok(Json(json!({
                "cow_id": cow_id,
                "revenue_entries": revenues,
                "total_revenue_lamports": total,
                "trading_profit_lamports": trading,
                "fork_fees_lamports": fork_fees,
                "creator_royalties_lamports": royalties,
                "pending_distribution_lamports": pending
            })))
        }
        Err(e) => {
            warn!("❌ Failed to fetch revenue: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_arbfarm_earnings(
    State(app_state): State<crate::AppState>,
    Path(wallet): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    info!("💰 Fetching ArbFarm earnings for wallet: {}", wallet);

    let pool = app_state.database.pool();
    match ArbFarmRepository::get_earnings_for_wallet(pool, &wallet).await {
        Ok((total, trading, fork_fees, royalties, pending, cows_owned, cows_forked)) => {
            Ok(Json(json!({
                "wallet_address": wallet,
                "total_earnings_lamports": total,
                "trading_profit_lamports": trading,
                "fork_fees_earned_lamports": fork_fees,
                "creator_royalties_lamports": royalties,
                "pending_distribution_lamports": pending,
                "cows_owned": cows_owned,
                "cows_forked_from": cows_forked,
                "period": "all_time"
            })))
        }
        Err(e) => {
            warn!("❌ Failed to fetch earnings: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_arbfarm_stats(
    State(app_state): State<crate::AppState>,
) -> Result<Json<Value>, StatusCode> {
    info!("📊 Fetching ArbFarm marketplace statistics");

    let pool = app_state.database.pool();
    match ArbFarmRepository::get_stats(pool).await {
        Ok(stats) => Ok(Json(json!({
            "total_cows": stats.total_cows,
            "active_cows": stats.active_cows,
            "total_forks": stats.total_forks,
            "total_trades_executed": stats.total_trades_executed,
            "total_profit_generated_lamports": stats.total_profit_generated_lamports,
            "total_revenue_distributed_lamports": stats.total_revenue_distributed_lamports,
            "avg_win_rate": stats.avg_win_rate,
            "last_updated": chrono::Utc::now()
        }))),
        Err(e) => {
            warn!("❌ Failed to fetch stats: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_wallet_stash(
    Path(wallet_address): Path<String>,
) -> Json<Value> {
    info!("📦 Fetching stash for wallet: {}", wallet_address);

    let owned_cows: Vec<Value> = vec![];
    let owned_tools: Vec<Value> = vec![];
    let unlocked_tabs = vec!["arbfarm"];

    let unlock_progress = vec![
        json!({
            "cowId": "arbfarm",
            "cowName": "ArbFarm",
            "owned": 5,
            "required": 5,
            "percent": 100,
            "isNullBlockService": true
        }),
        json!({
            "cowId": "polymev",
            "cowName": "PolyMev",
            "owned": 0,
            "required": 5,
            "percent": 0,
            "isNullBlockService": true
        })
    ];

    Json(json!({
        "wallet_address": wallet_address,
        "owned_cows": owned_cows,
        "owned_tools": owned_tools,
        "unlocked_tabs": unlocked_tabs,
        "unlock_progress": unlock_progress
    }))
}

async fn get_wallet_unlocks(
    Path(wallet_address): Path<String>,
) -> Json<Value> {
    info!("🔓 Fetching unlocked tabs for wallet: {}", wallet_address);

    Json(json!({
        "wallet_address": wallet_address,
        "unlocked_tabs": ["arbfarm"],
        "unlock_progress": {
            "arbfarm": { "owned": 5, "required": 5, "percent": 100 },
            "polymev": { "owned": 0, "required": 5, "percent": 0 }
        }
    }))
}