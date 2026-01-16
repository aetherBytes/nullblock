use std::env;
use std::net::SocketAddr;

use axum::{routing::{get, post}, Router};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, warn};

mod agents;
mod config;
mod consensus;
mod database;
mod error;
mod events;
mod execution;
mod handlers;
mod mcp;
mod models;
mod research;
mod resilience;
mod server;
mod threat;
mod venues;
mod wallet;
mod webhooks;

use axum::Json;
use crate::config::Config;
use crate::handlers::{consensus as consensus_handlers, curves, edges, engram as engram_handlers, health, kol, research as research_handlers, scanner, settings, sse, swarm, threat as threat_handlers, trades, wallet as wallet_handlers, webhooks as webhook_handlers};
use crate::mcp::{get_manifest, get_all_tools};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if let Err(e) = dotenvy::from_filename(".env.dev") {
        warn!("⚠️ Could not load .env.dev file: {}", e);
    } else {
        println!("✅ Loaded configuration from .env.dev");
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,arb_farm=debug".into()),
        )
        .init();

    let config = Config::from_env()?;
    info!("🔧 Configuration loaded: {}", config.service_name);

    let state = server::AppState::new(config.clone()).await?;

    let app = create_router(state);

    let port = env::var("ARB_FARM_PORT")
        .unwrap_or_else(|_| "9007".to_string())
        .parse::<u16>()
        .unwrap_or(9007);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let base_url =
        env::var("ARB_FARM_SERVICE_URL").unwrap_or_else(|_| format!("http://localhost:{}", port));

    info!("🚀 ArbFarm MEV Agent Swarm starting...");
    info!("📡 Server will bind to: {}", addr);
    info!("🏥 Health check: {}/health", base_url);
    info!("📊 Scanner API: {}/scanner", base_url);
    info!("💹 Edges API: {}/edges", base_url);
    info!("🎯 Strategies API: {}/strategies", base_url);
    info!("📈 Curves API: {}/curves", base_url);
    info!("🔬 Research API: {}/research", base_url);
    info!("👥 KOL Tracking API: {}/kol", base_url);
    info!("🛡️ Threat Detection API: {}/threat", base_url);
    info!("🧠 Engram API: {}/engram", base_url);
    info!("🤝 Consensus API: {}/consensus", base_url);
    info!("🐝 Swarm Management API: {}/swarm", base_url);
    info!("💰 Wallet API: {}/wallet", base_url);
    info!("⚙️ Settings API: {}/settings", base_url);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("✅ Server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn mcp_manifest() -> Json<mcp::McpToolManifest> {
    Json(get_manifest())
}

async fn mcp_tools() -> Json<Vec<mcp::McpTool>> {
    Json(get_all_tools())
}

fn create_router(state: server::AppState) -> Router {
    Router::new()
        // Health
        .route("/health", get(health::health_check))
        // MCP - Crossroads discovery
        .route("/mcp/manifest", get(mcp_manifest))
        .route("/mcp/tools", get(mcp_tools))
        // Scanner
        .route("/scanner/status", get(scanner::get_scanner_status))
        .route("/scanner/start", post(scanner::start_scanner))
        .route("/scanner/stop", post(scanner::stop_scanner))
        .route("/scanner/signals", get(scanner::get_signals))
        // Edges (Opportunities)
        .route("/edges", get(edges::list_edges))
        .route("/edges/atomic", get(edges::list_atomic_edges))
        .route("/edges/:id", get(edges::get_edge))
        .route("/edges/:id/approve", post(edges::approve_edge))
        .route("/edges/:id/reject", post(edges::reject_edge))
        .route("/edges/:id/execute", post(edges::execute_edge))
        .route("/edges/:id/execute-auto", post(edges::execute_edge_auto))
        .route("/edges/:id/simulate", post(edges::simulate_edge))
        // Trades
        .route("/trades", get(trades::list_trades))
        .route("/trades/stats", get(trades::get_trade_stats))
        .route("/trades/daily", get(trades::get_daily_stats))
        .route("/trades/:id", get(trades::get_trade))
        // Bonding Curves (pump.fun, moonshot)
        .route("/curves/tokens", get(curves::list_curve_tokens))
        .route("/curves/health", get(curves::get_venues_health))
        .route("/curves/graduation-candidates", get(curves::list_graduation_candidates))
        .route("/curves/cross-venue-arb", get(curves::detect_cross_venue_arb))
        .route("/curves/:mint/progress", get(curves::get_graduation_progress))
        .route("/curves/:mint/holders", get(curves::get_holder_stats))
        .route("/curves/:mint/quote", post(curves::get_curve_quote))
        .route("/curves/:mint/parameters", get(curves::get_curve_parameters))
        // Research/DD
        .route("/research/ingest", post(research_handlers::ingest_url))
        .route("/research/discoveries", get(research_handlers::list_discoveries))
        .route("/research/discoveries/:id", get(research_handlers::get_discovery))
        .route("/research/discoveries/:id/approve", post(research_handlers::approve_discovery))
        .route("/research/discoveries/:id/reject", post(research_handlers::reject_discovery))
        .route("/research/backtest", post(research_handlers::run_backtest))
        .route("/research/backtest/:id", get(research_handlers::get_backtest_result))
        .route("/research/sources", get(research_handlers::list_sources))
        .route("/research/sources", post(research_handlers::add_source))
        .route("/research/sources/:id", get(research_handlers::get_source))
        .route("/research/sources/:id", axum::routing::delete(research_handlers::delete_source))
        .route("/research/sources/:id/toggle", post(research_handlers::toggle_source))
        .route("/research/alerts", get(research_handlers::list_alerts))
        .route("/research/stats", get(research_handlers::get_monitor_stats))
        .route("/research/monitor", post(research_handlers::monitor_account))
        // Consensus (Multi-LLM voting)
        .route("/consensus", get(consensus_handlers::list_consensus_history))
        .route("/consensus/stats", get(consensus_handlers::get_consensus_stats))
        .route("/consensus/models", get(consensus_handlers::list_available_models))
        .route("/consensus/request", post(consensus_handlers::request_consensus))
        .route("/consensus/:id", get(consensus_handlers::get_consensus_detail))
        // KOL Tracking + Copy Trading
        .route("/kol", get(kol::list_kols))
        .route("/kol", post(kol::add_kol))
        .route("/kol/copies/active", get(kol::list_active_copies))
        .route("/kol/copies/stats", get(kol::get_copy_stats))
        .route("/kol/:id", get(kol::get_kol))
        .route("/kol/:id", axum::routing::put(kol::update_kol))
        .route("/kol/:id", axum::routing::delete(kol::delete_kol))
        .route("/kol/:id/trades", get(kol::get_kol_trades))
        .route("/kol/:id/stats", get(kol::get_kol_stats))
        .route("/kol/:id/trust", get(kol::get_trust_breakdown))
        .route("/kol/:id/copy/enable", post(kol::enable_copy_trading))
        .route("/kol/:id/copy/disable", post(kol::disable_copy_trading))
        .route("/kol/:id/copy/history", get(kol::get_copy_history))
        // Threat Detection
        .route("/threat/check/:mint", get(threat_handlers::check_token))
        .route("/threat/wallet/:address", get(threat_handlers::check_wallet))
        .route("/threat/blocked", get(threat_handlers::list_blocked))
        .route("/threat/blocked/:address", axum::routing::delete(threat_handlers::remove_from_blocklist))
        .route("/threat/blocked/:address/status", get(threat_handlers::is_blocked))
        .route("/threat/whitelist", get(threat_handlers::list_whitelisted))
        .route("/threat/whitelist", post(threat_handlers::whitelist_entity))
        .route("/threat/whitelist/:address", axum::routing::delete(threat_handlers::remove_from_whitelist))
        .route("/threat/whitelist/:address/status", get(threat_handlers::is_whitelisted))
        .route("/threat/watch", get(threat_handlers::list_watched))
        .route("/threat/watch", post(threat_handlers::add_watch))
        .route("/threat/report", post(threat_handlers::report_threat))
        .route("/threat/alerts", get(threat_handlers::get_alerts))
        .route("/threat/score/:mint/history", get(threat_handlers::get_score_history))
        .route("/threat/stats", get(threat_handlers::get_stats))
        // Engrams (Pattern Learning)
        .route("/engram", post(engram_handlers::create_engram))
        .route("/engram/search", get(engram_handlers::search_engrams))
        .route("/engram/patterns", post(engram_handlers::find_patterns))
        .route("/engram/avoidance", post(engram_handlers::create_avoidance))
        .route("/engram/avoidance/:entity_type/:address", get(engram_handlers::check_avoidance))
        .route("/engram/pattern", post(engram_handlers::create_pattern))
        .route("/engram/stats", get(engram_handlers::get_harvester_stats))
        .route("/engram/:key", get(engram_handlers::get_engram))
        .route("/engram/:key", axum::routing::delete(engram_handlers::delete_engram))
        // Swarm Management
        .route("/swarm/status", get(swarm::get_swarm_status))
        .route("/swarm/health", get(swarm::get_swarm_health))
        .route("/swarm/pause", post(swarm::pause_swarm))
        .route("/swarm/resume", post(swarm::resume_swarm))
        .route("/swarm/agents", get(swarm::list_agents))
        .route("/swarm/agents/:id", get(swarm::get_agent_status))
        .route("/swarm/heartbeat", post(swarm::record_heartbeat))
        .route("/swarm/failure", post(swarm::report_failure))
        .route("/swarm/circuit-breakers", get(swarm::list_circuit_breakers))
        .route("/swarm/circuit-breakers/:name/reset", post(swarm::reset_circuit_breaker))
        .route("/swarm/circuit-breakers/reset-all", post(swarm::reset_all_circuit_breakers))
        // Wallet (Turnkey delegation)
        .route("/wallet/status", get(wallet_handlers::get_wallet_status))
        .route("/wallet/setup", post(wallet_handlers::setup_wallet))
        .route("/wallet/policy", post(wallet_handlers::update_policy))
        .route("/wallet/balance", get(wallet_handlers::get_balance))
        .route("/wallet/disconnect", post(wallet_handlers::disconnect_wallet))
        .route("/wallet/usage", get(wallet_handlers::get_daily_usage))
        .route("/wallet/test-sign", post(wallet_handlers::test_sign))
        // Settings
        .route("/settings", get(settings::get_all_settings))
        .route("/settings/risk", get(settings::get_risk_settings))
        .route("/settings/risk", post(settings::update_risk_settings))
        .route("/settings/venues", get(settings::get_venue_settings))
        .route("/settings/api-keys", get(settings::get_api_key_status))
        // Webhooks (Helius)
        .route("/webhooks/status", get(webhook_handlers::get_webhook_status))
        .route("/webhooks/register", post(webhook_handlers::register_webhook))
        .route("/webhooks", get(webhook_handlers::list_webhooks))
        .route("/webhooks", axum::routing::delete(webhook_handlers::delete_webhook))
        .route("/webhooks/helius", post(webhook_handlers::receive_helius_webhook))
        .route("/webhooks/events", get(webhook_handlers::get_recent_webhook_events))
        // SSE Streams
        .route("/scanner/stream", get(sse::scanner_stream))
        .route("/edges/stream", get(sse::edges_stream))
        .route("/events/stream", get(sse::all_events_stream))
        .route("/threat/stream", get(sse::threat_stream))
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any),
        )
        .layer(TraceLayer::new_for_http())
}
