use std::env;
use std::net::SocketAddr;

use axum::{routing::{get, post}, Router};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, warn};

mod agents;
mod config;
mod consensus;
mod database;
mod engrams;
mod error;
mod events;
mod execution;
mod handlers;
mod helius;
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
use crate::handlers::{approvals as approval_handlers, autonomous as autonomous_handlers, consensus as consensus_handlers, curves, edges, engram as engram_handlers, health, helius as helius_handlers, kol, positions as position_handlers, research as research_handlers, scanner, settings, sse, strategies, swarm, threat as threat_handlers, trades, wallet as wallet_handlers, webhooks as webhook_handlers};
use crate::mcp::{get_manifest, get_all_tools, handlers as mcp_handlers};

async fn print_startup_summary(state: &server::AppState) {
    println!("\n{}", "=".repeat(70));
    println!("                    🌾 ARBFARM STARTUP SUMMARY 🌾");
    println!("{}\n", "=".repeat(70));

    // Wallet Status
    println!("💰 WALLET STATUS:");
    if state.dev_signer.is_configured() {
        let address = state.dev_signer.get_address().unwrap_or("unknown");
        println!("   Address: {}", address);
        println!("   Signer:  ✅ DEV MODE (private key from env)");

        // Try to get balance via RPC
        let balance_result: Result<serde_json::Value, _> = state.helius_rpc_client
            .rpc_call("getBalance", serde_json::json!([address]))
            .await;
        match balance_result {
            Ok(balance_json) => {
                if let Some(value) = balance_json.get("value").and_then(|v| v.as_u64()) {
                    let sol_balance = value as f64 / 1_000_000_000.0;
                    println!("   Balance: {:.4} SOL", sol_balance);
                    if sol_balance < 0.1 {
                        println!("   ⚠️  WARNING: Low balance - consider adding more SOL");
                    }
                } else {
                    println!("   Balance: ❌ Unexpected response format");
                }
            }
            Err(e) => println!("   Balance: ❌ Failed to fetch ({})", e),
        }
    } else {
        println!("   ❌ NO WALLET CONFIGURED");
        println!("   Set ARB_FARM_WALLET_PRIVATE_KEY in .env.dev");
    }

    // Strategies
    println!("\n📊 STRATEGIES:");
    let strategies = state.strategy_engine.list_strategies().await;
    for strategy in &strategies {
        let status = if strategy.is_active { "🟢 ON" } else { "⚪ OFF" };
        let mode = match strategy.execution_mode.as_str() {
            "autonomous" => "🤖 AUTO",
            "hybrid" => "🔀 HYBRID",
            _ => "👤 MANUAL",
        };
        println!("   {} {} - {} | {} | max:{:.2} SOL | risk:{}",
            status,
            strategy.name,
            strategy.strategy_type,
            mode,
            strategy.risk_params.max_position_sol,
            strategy.risk_params.max_risk_score
        );
    }

    // Risk Config
    println!("\n⚠️  RISK CONFIG:");
    let risk = state.risk_config.read().await;
    println!("   Max Position:      {:.2} SOL", risk.max_position_sol);
    println!("   Daily Loss Limit:  {:.2} SOL", risk.daily_loss_limit_sol);
    println!("   Max Drawdown:      {:.1}%", risk.max_drawdown_percent);
    println!("   Max Concurrent:    {} positions", risk.max_concurrent_positions);
    drop(risk);

    // Autonomous Executor
    println!("\n🤖 AUTONOMOUS EXECUTOR:");
    let executor_stats = state.autonomous_executor.get_stats().await;
    println!("   Running:    {}", if executor_stats.is_running { "✅ YES" } else { "⏸️ NO" });
    println!("   Attempted:  {}", executor_stats.executions_attempted);
    println!("   Succeeded:  {}", executor_stats.executions_succeeded);
    println!("   Failed:     {}", executor_stats.executions_failed);
    println!("   SOL Deployed: {:.4}", executor_stats.total_sol_deployed);

    // Scanner Status
    println!("\n📡 SCANNER:");
    let scanner_status = state.scanner.get_status().await;
    println!("   Running:  {}", if scanner_status.is_running { "🟢 YES" } else { "⚪ NO" });
    println!("   Venues:   {}/{} healthy", scanner_status.stats.healthy_venues, scanner_status.stats.total_venues);
    println!("   Signals:  {} detected", scanner_status.stats.total_signals_detected);

    // API Key Status
    println!("\n🔑 API KEYS:");
    println!("   Helius:     {}", if state.config.helius_api_key.is_some() { "✅" } else { "❌" });
    println!("   Birdeye:    {}", if state.config.birdeye_api_key.is_some() { "✅" } else { "⚠️ Optional" });
    println!("   OpenRouter: {}", if state.config.openrouter_api_key.is_some() { "✅" } else { "⚠️ Optional" });

    println!("\n{}", "=".repeat(70));
    println!("📋 QUICK COMMANDS:");
    println!("   Start scanner:    curl -X POST localhost:9007/scanner/start");
    println!("   Start executor:   curl -X POST localhost:9007/executor/start");
    println!("   Check candidates: curl localhost:9007/curves/graduation-candidates");
    println!("   Check balance:    curl localhost:9007/wallet/balance");
    println!("   Get strategies:   curl localhost:9007/strategies");
    println!("{}\n", "=".repeat(70));
}

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

    // Print comprehensive startup summary for tmuxinator pane
    print_startup_summary(&state).await;

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
    info!("🔍 KOL Discovery API: {}/kol/discovery", base_url);
    info!("🛡️ Threat Detection API: {}/threat", base_url);
    info!("🧠 Engram API: {}/engram", base_url);
    info!("🤝 Consensus API: {}/consensus", base_url);
    info!("🐝 Swarm Management API: {}/swarm", base_url);
    info!("💰 Wallet API: {}/wallet", base_url);
    info!("⚡ Helius API: {}/helius", base_url);
    info!("⚙️ Settings API: {}/settings", base_url);
    info!("🔐 Approvals API: {}/approvals", base_url);
    info!("🎮 Execution Config API: {}/execution", base_url);

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
        // MCP - Crossroads discovery + tool execution
        .route("/mcp/manifest", get(mcp_manifest))
        .route("/mcp/tools", get(mcp_tools))
        .route("/mcp/call", post(mcp_handlers::call_tool))
        // Scanner
        .route("/scanner/status", get(scanner::get_scanner_status))
        .route("/scanner/start", post(scanner::start_scanner))
        .route("/scanner/stop", post(scanner::stop_scanner))
        .route("/scanner/signals", get(scanner::get_signals))
        .route("/scanner/process", post(scanner::process_signals))
        // Edges (Opportunities)
        .route("/edges", get(edges::list_edges))
        .route("/edges/atomic", get(edges::list_atomic_edges))
        .route("/edges/:id", get(edges::get_edge))
        .route("/edges/:id/approve", post(edges::approve_edge))
        .route("/edges/:id/reject", post(edges::reject_edge))
        .route("/edges/:id/execute", post(edges::execute_edge))
        .route("/edges/:id/execute-auto", post(edges::execute_edge_auto))
        .route("/edges/:id/simulate", post(edges::simulate_edge))
        // Strategies
        .route("/strategies", get(strategies::list_strategies))
        .route("/strategies", post(strategies::create_strategy))
        .route("/strategies/:id", get(strategies::get_strategy))
        .route("/strategies/:id", axum::routing::put(strategies::update_strategy))
        .route("/strategies/:id", axum::routing::delete(strategies::delete_strategy))
        .route("/strategies/:id/toggle", post(strategies::toggle_strategy))
        .route("/strategies/:id/risk-profile", post(strategies::set_risk_profile))
        .route("/strategies/:id/kill", post(strategies::kill_strategy))
        .route("/strategies/:id/reset-stats", post(strategies::reset_strategy_stats))
        .route("/strategies/batch-toggle", post(strategies::batch_toggle_strategies))
        .route("/strategies/save-to-engrams", post(strategies::save_strategies_to_engrams))
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
        .route("/curves/:mint/state", get(curves::get_on_chain_state))
        .route("/curves/:mint/buy", post(curves::buy_curve_token))
        .route("/curves/:mint/sell", post(curves::sell_curve_token))
        .route("/curves/:mint/post-graduation-pool", get(curves::get_post_graduation_pool))
        .route("/curves/:mint/addresses", get(curves::get_curve_addresses))
        // Curve Metrics & Scoring
        .route("/curves/:mint/metrics", get(curves::get_curve_metrics))
        .route("/curves/:mint/holder-analysis", get(curves::get_holder_analysis))
        .route("/curves/:mint/score", get(curves::get_opportunity_score))
        .route("/curves/top-opportunities", get(curves::get_top_opportunities))
        .route("/curves/scoring-config", get(curves::get_scoring_config))
        // Research/DD
        .route("/research/ingest", post(research_handlers::ingest_url))
        .route("/research/extract", post(research_handlers::extract_strategy_from_text))
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
        // KOL Discovery
        .route("/kol/discovery/status", get(kol::get_discovery_status))
        .route("/kol/discovery/start", post(kol::start_discovery))
        .route("/kol/discovery/stop", post(kol::stop_discovery))
        .route("/kol/discovery/scan", post(kol::scan_for_kols_now))
        .route("/kol/discovery/discovered", get(kol::list_discovered_kols))
        .route("/kol/discovery/promote/:wallet_address", post(kol::promote_discovered_kol))
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
        // Wallet (Turnkey delegation + dev mode)
        .route("/wallet/status", get(wallet_handlers::get_wallet_status))
        .route("/wallet/setup", post(wallet_handlers::setup_wallet))
        .route("/wallet/policy", post(wallet_handlers::update_policy))
        .route("/wallet/balance", get(wallet_handlers::get_balance))
        .route("/wallet/disconnect", post(wallet_handlers::disconnect_wallet))
        .route("/wallet/usage", get(wallet_handlers::get_daily_usage))
        .route("/wallet/test-sign", post(wallet_handlers::test_sign))
        .route("/wallet/sign", post(wallet_handlers::sign_transaction))
        .route("/wallet/dev-mode", get(wallet_handlers::get_dev_mode))
        .route("/wallet/dev-connect", post(wallet_handlers::connect_dev_wallet))
        .route("/wallet/capital", get(wallet_handlers::get_capital_usage))
        .route("/wallet/capital/sync", post(wallet_handlers::sync_capital_balance))
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
        // Helius Integration (⚡ LaserStream, Priority Fee, Sender, DAS)
        .route("/helius/status", get(helius_handlers::get_helius_status))
        .route("/helius/laserstream", get(helius_handlers::get_laserstream_status))
        .route("/helius/priority-fees", get(helius_handlers::get_priority_fees))
        .route("/helius/priority-fees/cached", get(helius_handlers::get_cached_priority_fees))
        .route("/helius/sender/stats", get(helius_handlers::get_sender_stats))
        .route("/helius/sender/ping", post(helius_handlers::ping_sender))
        .route("/helius/das/lookup", post(helius_handlers::das_lookup))
        .route("/helius/das/assets", get(helius_handlers::das_assets_by_owner))
        .route("/helius/config", get(helius_handlers::get_helius_config))
        .route("/helius/config", axum::routing::put(helius_handlers::update_helius_config))
        // Positions (Exit Management)
        .route("/positions", get(position_handlers::get_positions))
        .route("/positions/exposure", get(position_handlers::get_exposure))
        .route("/positions/monitor/status", get(position_handlers::get_monitor_status))
        .route("/positions/monitor/start", post(position_handlers::start_monitor))
        .route("/positions/emergency-close", post(position_handlers::emergency_close_all))
        .route("/positions/:id", get(position_handlers::get_position))
        .route("/positions/:id/close", post(position_handlers::close_position))
        // Approvals (Execution Controls)
        .route("/approvals", get(approval_handlers::list_approvals))
        .route("/approvals/pending", get(approval_handlers::list_pending_approvals))
        .route("/approvals/cleanup", post(approval_handlers::cleanup_expired))
        .route("/approvals/:id", get(approval_handlers::get_approval))
        .route("/approvals/:id/approve", post(approval_handlers::approve_approval))
        .route("/approvals/:id/reject", post(approval_handlers::reject_approval))
        .route("/approvals/hecate-recommendation", post(approval_handlers::add_hecate_recommendation))
        .route("/execution/config", get(approval_handlers::get_execution_config))
        .route("/execution/config", axum::routing::put(approval_handlers::update_execution_config))
        .route("/execution/toggle", post(approval_handlers::toggle_execution))
        // Autonomous Executor (Auto-execution for autonomous strategies)
        .route("/executor/stats", get(autonomous_handlers::get_autonomous_executor_stats))
        .route("/executor/executions", get(autonomous_handlers::list_autonomous_executions))
        .route("/executor/start", post(autonomous_handlers::start_autonomous_executor))
        .route("/executor/stop", post(autonomous_handlers::stop_autonomous_executor))
        // SSE Streams
        .route("/scanner/stream", get(sse::scanner_stream))
        .route("/edges/stream", get(sse::edges_stream))
        .route("/events/stream", get(sse::all_events_stream))
        .route("/threat/stream", get(sse::threat_stream))
        .route("/helius/stream", get(sse::helius_stream))
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any),
        )
        .layer(TraceLayer::new_for_http())
}
