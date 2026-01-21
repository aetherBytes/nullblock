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
mod erebus;
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
use crate::handlers::{approvals as approval_handlers, autonomous as autonomous_handlers, config_handlers, consensus as consensus_handlers, curves, edges, engram as engram_handlers, graduation as graduation_handlers, health, helius as helius_handlers, kol, positions as position_handlers, research as research_handlers, scanner, settings, sniper as sniper_handlers, sse, strategies, swarm, threat as threat_handlers, trades, wallet as wallet_handlers, webhooks as webhook_handlers};
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

    // Real-time Monitor Status
    println!("\n📡 REAL-TIME MONITOR:");
    let laserstream_configured = state.laserstream_client.is_configured();
    println!("   LaserStream: {}", if laserstream_configured { "✅ Configured" } else { "⚠️ Not configured" });
    let subscribed = state.realtime_monitor.get_subscribed_count().await;
    println!("   Subscriptions: {} bonding curves", subscribed);

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

    // Validate wallet funding - block startup if balance too low for trading
    const MIN_BALANCE_SOL: f64 = 0.05;
    if state.dev_signer.is_configured() {
        if let Some(address) = state.dev_signer.get_address() {
            let balance_result: Result<serde_json::Value, _> = state.helius_rpc_client
                .rpc_call("getBalance", serde_json::json!([address]))
                .await;

            match balance_result {
                Ok(balance_json) => {
                    if let Some(value) = balance_json.get("value").and_then(|v| v.as_u64()) {
                        let sol_balance = value as f64 / 1_000_000_000.0;
                        if sol_balance < MIN_BALANCE_SOL {
                            return Err(anyhow::anyhow!(
                                "❌ STARTUP BLOCKED: Wallet balance ({:.4} SOL) is below minimum ({:.2} SOL). \
                                Fund wallet {} with at least {:.2} SOL to enable trading.",
                                sol_balance, MIN_BALANCE_SOL, address, MIN_BALANCE_SOL
                            ));
                        }
                        info!("✅ Wallet funding validated: {:.4} SOL (min: {:.2} SOL)", sol_balance, MIN_BALANCE_SOL);

                        // Dynamic max position: 1/15th of wallet balance, capped at 10 SOL
                        const MAX_POSITION_CAP_SOL: f64 = 10.0;
                        let dynamic_max_position = (sol_balance / 15.0).min(MAX_POSITION_CAP_SOL);
                        let dynamic_max_position = (dynamic_max_position * 100.0).round() / 100.0; // Round to 2 decimals

                        {
                            let mut risk_config = state.risk_config.write().await;
                            risk_config.max_position_sol = dynamic_max_position;
                            risk_config.max_position_per_token_sol = dynamic_max_position;
                        }
                        info!(
                            "💰 Dynamic max position set: {:.2} SOL (1/15th of {:.2} SOL balance, cap: {} SOL)",
                            dynamic_max_position, sol_balance, MAX_POSITION_CAP_SOL
                        );

                        // Sync all active strategies with the dynamic max position (in-memory + database)
                        use crate::database::repositories::strategies::UpdateStrategyRecord;
                        let strategies = state.strategy_engine.list_strategies().await;
                        let mut synced = 0;
                        for strategy in strategies.iter().filter(|s| s.is_active) {
                            let mut params = strategy.risk_params.clone();
                            params.max_position_sol = dynamic_max_position;

                            // Update in-memory
                            if state.strategy_engine.set_risk_params(strategy.id, params.clone()).await.is_ok() {
                                // Persist to database
                                if let Err(e) = state.strategy_repo.update(strategy.id, UpdateStrategyRecord {
                                    name: None,
                                    venue_types: None,
                                    execution_mode: None,
                                    risk_params: Some(params),
                                    is_active: None,
                                }).await {
                                    warn!("Failed to persist dynamic max position to DB for strategy {}: {}", strategy.id, e);
                                }
                                synced += 1;
                            }
                        }
                        if synced > 0 {
                            info!("✅ Synced {} strategies with dynamic max position: {:.2} SOL (persisted to DB)", synced, dynamic_max_position);
                        }
                    }
                }
                Err(e) => {
                    warn!("⚠️ Could not validate wallet balance: {}. Proceeding with caution.", e);
                }
            }
        }
    }

    // Clone state components for auto-start task BEFORE passing state to router
    let scanner_for_autostart = state.scanner.clone();
    let executor_for_autostart = state.autonomous_executor.clone();
    let position_monitor_for_autostart = state.position_monitor.clone();
    let realtime_monitor_for_autostart = state.realtime_monitor.clone();
    let dev_signer_for_autostart = state.dev_signer.clone();
    let helius_das_for_autostart = state.helius_das.clone();
    let position_manager_for_autostart = state.position_manager.clone();
    let on_chain_fetcher_for_autostart = state.on_chain_fetcher.clone();
    let metrics_collector_for_autostart = state.metrics_collector.clone();
    let jupiter_venue_for_autostart = state.jupiter_venue.clone();
    let risk_config_for_autostart = state.risk_config.clone();
    let graduation_tracker_for_autostart = state.graduation_tracker.clone();
    let graduation_sniper_for_autostart = state.graduation_sniper.clone();
    let consensus_engine_for_analysis = state.consensus_engine.clone();
    let engrams_client_for_analysis = state.engrams_client.clone();
    let db_pool_for_analysis = state.db_pool.clone();
    let event_tx_for_analysis = state.event_tx.clone();
    let dev_signer_for_analysis = state.dev_signer.clone();

    let position_repo_for_metrics = state.position_repo.clone();
    let engrams_client_for_metrics = state.engrams_client.clone();
    let dev_signer_for_metrics = state.dev_signer.clone();

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
    info!("🔫 Sniper API: {}/sniper", base_url);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("✅ Server listening on {}", addr);

    // Auto-start workers after server is ready
    tokio::spawn(async move {
        // Wait for server to be fully ready
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        info!("🚀 Auto-starting workers...");

        // Start scanner
        scanner_for_autostart.start().await;
        info!("✅ Scanner started");

        // Autonomous executor startup is handled in AppState::new() based on strategy config
        // It should NOT be unconditionally started here - only start if user has enabled auto-execution
        let executor_stats = executor_for_autostart.get_stats().await;
        if executor_stats.is_running {
            info!("✅ Autonomous executor already running (auto-execution enabled)");
        } else {
            info!("ℹ️ Autonomous executor NOT started (auto-execution disabled in strategy config)");
            info!("   Enable via: curl -X POST localhost:9007/execution/toggle -d '{{\"enabled\":true}}'");
        }

        // Start position monitor
        let monitor = position_monitor_for_autostart.clone();
        let signer = dev_signer_for_autostart.clone();
        tokio::spawn(async move {
            monitor.start_monitoring(signer).await;
        });
        info!("✅ Position monitor started");

        // Start real-time position monitor (websocket price updates)
        let realtime = realtime_monitor_for_autostart.clone();
        tokio::spawn(async move {
            if let Err(e) = realtime.start().await {
                tracing::error!("❌ Failed to start real-time monitor: {}", e);
            } else {
                info!("✅ Real-time position monitor started (websocket)");
            }
        });

        // Start graduation tracker and sniper
        graduation_tracker_for_autostart.start().await;
        info!("✅ Graduation tracker started (monitoring tracked tokens)");

        graduation_sniper_for_autostart.start().await;
        info!("✅ Graduation sniper started (listening for graduation events)");

        // Run wallet reconciliation to pick up orphaned positions
        if let Some(wallet_address) = dev_signer_for_autostart.get_address() {
            info!("🔄 Running wallet position reconciliation...");
            match helius_das_for_autostart.get_token_accounts_by_owner(&wallet_address).await {
                Ok(token_accounts) => {
                    let wallet_tokens: Vec<crate::execution::WalletTokenHolding> = token_accounts
                        .into_iter()
                        .map(|account| crate::execution::WalletTokenHolding {
                            mint: account.mint,
                            symbol: None,
                            balance: account.ui_amount,
                            decimals: account.decimals,
                        })
                        .collect();

                    info!("📊 Found {} tokens with balance in wallet", wallet_tokens.len());

                    let result = position_manager_for_autostart.reconcile_wallet_tokens(&wallet_tokens).await;

                    for position_id in &result.orphaned_positions {
                        if let Err(e) = position_manager_for_autostart.mark_position_orphaned(*position_id).await {
                            warn!("Failed to mark position {} as orphaned: {}", position_id, e);
                        }
                    }

                    if !result.discovered_tokens.is_empty() {
                        info!("🔍 Discovered {} untracked tokens in wallet - auto-creating exit strategies:", result.discovered_tokens.len());
                        for token in &result.discovered_tokens {
                            if crate::execution::BaseCurrency::is_base_currency(&token.mint) {
                                continue;
                            }

                            let estimated_price = match on_chain_fetcher_for_autostart.get_bonding_curve_state(&token.mint).await {
                                Ok(curve_state) => {
                                    if curve_state.virtual_token_reserves > 0 {
                                        curve_state.virtual_sol_reserves as f64 / curve_state.virtual_token_reserves as f64
                                    } else {
                                        // Try Jupiter as fallback for zero reserves
                                        match jupiter_venue_for_autostart.get_token_price(&token.mint).await {
                                            Ok(price) => {
                                                info!("   📈 {} - using Jupiter price (zero curve reserves)", &token.mint[..12]);
                                                price
                                            }
                                            Err(_) => {
                                                warn!("   ⚠️ {} - zero reserves and Jupiter unavailable, skipping", &token.mint[..12]);
                                                continue;
                                            }
                                        }
                                    }
                                }
                                Err(_) => {
                                    // Bonding curve not found - try Jupiter (likely graduated token)
                                    match jupiter_venue_for_autostart.get_token_price(&token.mint).await {
                                        Ok(price) => {
                                            info!("   📈 {} - using Jupiter price (graduated/DEX token)", &token.mint[..12]);
                                            price
                                        }
                                        Err(e) => {
                                            warn!("   ⚠️ {} - no price source available ({}), skipping", &token.mint[..12], e);
                                            continue;
                                        }
                                    }
                                }
                            };

                            let exit_config = match metrics_collector_for_autostart.calculate_metrics(&token.mint, "pump_fun").await {
                                Ok(metrics) => {
                                    crate::execution::ExitConfig::for_discovered_with_metrics(metrics.volume_24h, metrics.holder_count)
                                }
                                Err(_) => crate::execution::ExitConfig::for_discovered_token(),
                            };

                            // Use current risk config for discovered position entry estimates
                            let max_position_sol = risk_config_for_autostart.read().await.max_position_sol;
                            let raw_estimated_entry = token.balance * estimated_price;
                            let estimated_entry_sol = if raw_estimated_entry > max_position_sol {
                                // Estimated value exceeds max position - cap at max (likely price moved)
                                max_position_sol
                            } else if raw_estimated_entry < 0.001 {
                                // Too small to estimate, use max as conservative default
                                max_position_sol
                            } else {
                                raw_estimated_entry
                            };

                            match position_manager_for_autostart.create_discovered_position_with_config(
                                token,
                                estimated_price,
                                estimated_entry_sol,
                                exit_config,
                            ).await {
                                Ok(position) => {
                                    info!(
                                        "   ✅ {} ({:.6}) - created position {} with SL:{:?}%/TP:{:?}%",
                                        &token.mint[..12],
                                        token.balance,
                                        position.id,
                                        position.exit_config.stop_loss_percent,
                                        position.exit_config.take_profit_percent
                                    );
                                }
                                Err(e) => {
                                    warn!("   ❌ {} - failed to create position: {}", &token.mint[..12], e);
                                }
                            }
                        }
                    }

                    if !result.orphaned_positions.is_empty() {
                        info!("⚠️ Found {} orphaned positions (no longer in wallet)", result.orphaned_positions.len());
                    }

                    info!("✅ Wallet reconciliation complete: {} tracked positions", result.tracked_positions);
                }
                Err(e) => {
                    warn!("⚠️ Failed to reconcile wallet positions: {}", e);
                }
            }
        }

        info!("🎯 All workers auto-started");

        // Start periodic wallet reconciliation (every 10 seconds) to catch orphaned tokens
        let periodic_wallet = dev_signer_for_autostart.get_address().map(|s| s.to_string());
        let periodic_helius = helius_das_for_autostart.clone();
        let periodic_position_manager = position_manager_for_autostart.clone();
        let periodic_on_chain = on_chain_fetcher_for_autostart.clone();
        let periodic_metrics = metrics_collector_for_autostart.clone();
        let periodic_jupiter = jupiter_venue_for_autostart.clone();
        let periodic_risk_config = risk_config_for_autostart.clone();

        if periodic_wallet.is_some() {
            tokio::spawn(async move {
                let wallet_address = periodic_wallet.unwrap();
                let reconcile_interval = std::time::Duration::from_secs(10); // 10 seconds

                loop {
                    tokio::time::sleep(reconcile_interval).await;

                    info!("🔄 [Periodic] Running wallet reconciliation...");

                    match periodic_helius.get_token_accounts_by_owner(&wallet_address).await {
                        Ok(token_accounts) => {
                            let wallet_tokens: Vec<crate::execution::WalletTokenHolding> = token_accounts
                                .into_iter()
                                .map(|account| crate::execution::WalletTokenHolding {
                                    mint: account.mint,
                                    symbol: None,
                                    balance: account.ui_amount,
                                    decimals: account.decimals,
                                })
                                .collect();

                            let result = periodic_position_manager.reconcile_wallet_tokens(&wallet_tokens).await;

                            // Mark orphaned positions
                            for position_id in &result.orphaned_positions {
                                if let Err(e) = periodic_position_manager.mark_position_orphaned(*position_id).await {
                                    warn!("[Periodic] Failed to mark position {} as orphaned: {}", position_id, e);
                                }
                            }

                            // Create positions for discovered tokens
                            for token in &result.discovered_tokens {
                                if crate::execution::BaseCurrency::is_base_currency(&token.mint) {
                                    continue;
                                }

                                // Skip dust amounts
                                if token.balance < 0.001 {
                                    continue;
                                }

                                let estimated_price = match periodic_on_chain.get_bonding_curve_state(&token.mint).await {
                                    Ok(curve_state) => {
                                        if curve_state.virtual_token_reserves > 0 {
                                            curve_state.virtual_sol_reserves as f64 / curve_state.virtual_token_reserves as f64
                                        } else {
                                            // Try Jupiter as fallback for zero reserves
                                            match periodic_jupiter.get_token_price(&token.mint).await {
                                                Ok(price) => {
                                                    info!("[Periodic] 📈 {} - using Jupiter price (zero curve reserves)", &token.mint[..12]);
                                                    price
                                                }
                                                Err(_) => {
                                                    warn!("[Periodic] ⚠️ {} - zero reserves and Jupiter unavailable, skipping", &token.mint[..12]);
                                                    continue;
                                                }
                                            }
                                        }
                                    }
                                    Err(_) => {
                                        // Bonding curve not found - try Jupiter (likely graduated token)
                                        match periodic_jupiter.get_token_price(&token.mint).await {
                                            Ok(price) => {
                                                info!("[Periodic] 📈 {} - using Jupiter price (graduated/DEX token)", &token.mint[..12]);
                                                price
                                            }
                                            Err(e) => {
                                                warn!("[Periodic] ⚠️ {} - no price source available ({}), skipping", &token.mint[..12], e);
                                                continue;
                                            }
                                        }
                                    }
                                };

                                let exit_config = match periodic_metrics.calculate_metrics(&token.mint, "pump_fun").await {
                                    Ok(metrics) => {
                                        crate::execution::ExitConfig::for_discovered_with_metrics(metrics.volume_24h, metrics.holder_count)
                                    }
                                    Err(_) => crate::execution::ExitConfig::for_discovered_token(),
                                };

                                // Use current risk config for discovered position entry estimates
                                let max_position_sol = periodic_risk_config.read().await.max_position_sol;
                                let raw_estimated_entry = token.balance * estimated_price;
                                let estimated_entry_sol = if raw_estimated_entry > max_position_sol {
                                    // Estimated value exceeds max position - cap at max (likely price moved)
                                    max_position_sol
                                } else if raw_estimated_entry < 0.001 {
                                    // Too small to estimate, use max as conservative default
                                    max_position_sol
                                } else {
                                    raw_estimated_entry
                                };

                                match periodic_position_manager.create_discovered_position_with_config(
                                    token,
                                    estimated_price,
                                    estimated_entry_sol,
                                    exit_config,
                                ).await {
                                    Ok(position) => {
                                        info!(
                                            "[Periodic] ✅ Created position for orphaned token {} - SL:{:?}%/TP:{:?}%",
                                            &token.mint[..12],
                                            position.exit_config.stop_loss_percent,
                                            position.exit_config.take_profit_percent
                                        );
                                    }
                                    Err(e) => {
                                        warn!("[Periodic] ❌ Failed to create position for {}: {}", &token.mint[..12], e);
                                    }
                                }
                            }

                            if !result.discovered_tokens.is_empty() || !result.orphaned_positions.is_empty() {
                                info!(
                                    "[Periodic] 📊 Reconciliation: {} tracked, {} discovered, {} orphaned",
                                    result.tracked_positions,
                                    result.discovered_tokens.len(),
                                    result.orphaned_positions.len()
                                );
                            }
                        }
                        Err(e) => {
                            warn!("[Periodic] ⚠️ Failed to fetch wallet tokens: {}", e);
                        }
                    }
                }
            });
            info!("✅ Periodic wallet reconciliation started (every 10 seconds)");
        }

        // Start periodic consensus analysis (every 5 minutes)
        let analysis_wallet = dev_signer_for_analysis.get_address().map(|s| s.to_string());
        let analysis_engrams = engrams_client_for_analysis.clone();
        let analysis_consensus = consensus_engine_for_analysis.clone();
        let analysis_db_pool = db_pool_for_analysis.clone();
        let analysis_event_tx = event_tx_for_analysis.clone();

        if analysis_wallet.is_some() && analysis_engrams.is_configured() && analysis_consensus.is_ready().await {
            tokio::spawn(async move {
                let wallet_address = analysis_wallet.unwrap();
                let analysis_interval = std::time::Duration::from_secs(300); // 5 minutes
                let initial_delay = std::time::Duration::from_secs(60); // 1 minute initial delay

                info!("🧠 [Consensus] Starting periodic analysis scheduler (every 5 minutes, initial delay 60s)");
                tokio::time::sleep(initial_delay).await;

                loop {
                    info!("🧠 [Consensus] Starting periodic consensus analysis...");

                    // Gather trading metrics from database
                    let position_repo = crate::database::PositionRepository::new(analysis_db_pool.clone());
                    let pnl_stats = match position_repo.get_pnl_stats().await {
                        Ok(stats) => stats,
                        Err(e) => {
                            warn!("[Consensus] ⚠️ Failed to get PnL stats: {}", e);
                            tokio::time::sleep(analysis_interval).await;
                            continue;
                        }
                    };

                    // Skip analysis if no trading activity
                    if pnl_stats.total_trades == 0 {
                        info!("[Consensus] ℹ️ No trades yet, skipping analysis");
                        tokio::time::sleep(analysis_interval).await;
                        continue;
                    }

                    // Gather recent errors from engrams
                    let error_history = match analysis_engrams.get_error_history(&wallet_address, Some(20)).await {
                        Ok(errors) => errors,
                        Err(e) => {
                            warn!("[Consensus] ⚠️ Failed to get error history: {}", e);
                            Vec::new()
                        }
                    };

                    // Aggregate error counts by type
                    let mut error_counts: std::collections::HashMap<String, (u32, String)> = std::collections::HashMap::new();
                    for error in &error_history {
                        let error_type_str = serde_json::to_string(&error.error_type)
                            .unwrap_or_else(|_| "unknown".to_string())
                            .trim_matches('"')
                            .to_string();
                        let entry = error_counts.entry(error_type_str).or_insert((0, error.message.clone()));
                        entry.0 += 1;
                    }

                    let recent_errors: Vec<crate::consensus::ErrorSummary> = error_counts
                        .into_iter()
                        .map(|(error_type, (count, last_message))| crate::consensus::ErrorSummary {
                            error_type,
                            count,
                            last_message,
                        })
                        .collect();

                    // Build analysis context
                    let win_rate = if pnl_stats.total_trades > 0 {
                        pnl_stats.take_profits as f64 / pnl_stats.total_trades as f64
                    } else {
                        0.0
                    };

                    // Fetch recent closed trades with detailed context
                    let recent_trades = match position_repo.get_recent_closed_trades(15).await {
                        Ok(trades) => trades.into_iter().map(|t| crate::consensus::DetailedTradeContext {
                            position_id: t.position_id,
                            token_symbol: t.token_symbol,
                            venue: t.venue,
                            entry_sol: t.entry_sol,
                            exit_sol: t.exit_sol,
                            pnl_sol: t.pnl_sol,
                            pnl_percent: t.pnl_percent,
                            hold_minutes: t.hold_minutes,
                            exit_reason: t.exit_reason,
                            stop_loss_pct: t.stop_loss_pct,
                            take_profit_pct: t.take_profit_pct,
                            entry_time: t.entry_time,
                            exit_time: t.exit_time,
                        }).collect(),
                        Err(e) => {
                            warn!("[Consensus] ⚠️ Failed to get recent trades: {}", e);
                            Vec::new()
                        }
                    };

                    let context = crate::consensus::AnalysisContext {
                        total_trades: pnl_stats.total_trades,
                        winning_trades: pnl_stats.take_profits,
                        win_rate,
                        total_pnl_sol: pnl_stats.total_pnl,
                        today_pnl_sol: pnl_stats.today_pnl,
                        week_pnl_sol: pnl_stats.week_pnl,
                        avg_hold_minutes: pnl_stats.avg_hold_minutes,
                        best_trade: pnl_stats.best_trade_symbol.clone().map(|symbol| {
                            crate::consensus::TradeHighlightContext {
                                symbol,
                                pnl_sol: pnl_stats.best_trade_pnl,
                            }
                        }),
                        worst_trade: pnl_stats.worst_trade_symbol.clone().map(|symbol| {
                            crate::consensus::TradeHighlightContext {
                                symbol,
                                pnl_sol: pnl_stats.worst_trade_pnl,
                            }
                        }),
                        take_profit_count: pnl_stats.take_profits,
                        stop_loss_count: pnl_stats.stop_losses,
                        recent_errors,
                        time_period: "Last 7 days".to_string(),
                        recent_trades,
                    };

                    // Store context data needed after move
                    let context_recent_trades = context.recent_trades.clone();

                    // Request consensus analysis
                    match analysis_consensus.request_analysis(context).await {
                        Ok(result) => {
                            info!(
                                "[Consensus] ✅ Analysis complete: {} recommendations, {} risk alerts, confidence: {:.1}%",
                                result.recommendations.len(),
                                result.risk_alerts.len(),
                                result.avg_confidence * 100.0
                            );

                            // Generate conversation log
                            let session_id = uuid::Uuid::new_v4();
                            let conversation = crate::engrams::schemas::ConversationLog {
                                session_id,
                                participants: result.model_votes.clone(),
                                topic: crate::engrams::schemas::ConversationTopic::StrategyReview,
                                context: crate::engrams::schemas::ConversationContext {
                                    trigger: crate::engrams::schemas::ConversationTrigger::Scheduled,
                                    trades_in_scope: Some(pnl_stats.total_trades),
                                    time_period: Some("Last 7 days".to_string()),
                                    additional_context: Some(serde_json::json!({
                                        "total_pnl_sol": pnl_stats.total_pnl,
                                        "win_rate": win_rate,
                                    })),
                                },
                                messages: vec![
                                    crate::engrams::schemas::ConversationMessage {
                                        role: "system".to_string(),
                                        content: "Periodic strategy review analysis".to_string(),
                                        timestamp: chrono::Utc::now(),
                                        tokens_used: None,
                                        latency_ms: Some(result.total_latency_ms),
                                    },
                                    crate::engrams::schemas::ConversationMessage {
                                        role: "assistant".to_string(),
                                        content: result.overall_assessment.clone(),
                                        timestamp: chrono::Utc::now(),
                                        tokens_used: None,
                                        latency_ms: None,
                                    },
                                ],
                                outcome: crate::engrams::schemas::ConversationOutcome {
                                    consensus_reached: !result.recommendations.is_empty(),
                                    recommendations_generated: result.recommendations.len() as u32,
                                    engram_refs: Vec::new(),
                                    summary: Some(result.overall_assessment.clone()),
                                },
                                created_at: chrono::Utc::now(),
                            };

                            // Save conversation log
                            if let Err(e) = analysis_engrams.save_conversation_log(&wallet_address, &conversation).await {
                                warn!("[Consensus] ⚠️ Failed to save conversation log: {}", e);
                            } else {
                                info!("[Consensus] 💾 Saved conversation log: {}", session_id);
                            }

                            // Save each recommendation as an engram and collect IDs
                            let mut recommendation_ids: Vec<uuid::Uuid> = Vec::new();
                            for rec in &result.recommendations {
                                let rec_id = uuid::Uuid::new_v4();
                                recommendation_ids.push(rec_id);
                                let recommendation = crate::engrams::schemas::Recommendation {
                                    recommendation_id: rec_id,
                                    source: crate::engrams::schemas::RecommendationSource::ConsensusLlm,
                                    category: match rec.category.as_str() {
                                        "strategy" => crate::engrams::schemas::RecommendationCategory::Strategy,
                                        "risk" => crate::engrams::schemas::RecommendationCategory::Risk,
                                        "timing" => crate::engrams::schemas::RecommendationCategory::Timing,
                                        "venue" => crate::engrams::schemas::RecommendationCategory::Venue,
                                        "position" => crate::engrams::schemas::RecommendationCategory::Position,
                                        _ => crate::engrams::schemas::RecommendationCategory::Strategy,
                                    },
                                    title: rec.title.clone(),
                                    description: rec.description.clone(),
                                    suggested_action: crate::engrams::schemas::SuggestedAction {
                                        action_type: match rec.action_type.as_str() {
                                            "config_change" => crate::engrams::schemas::SuggestedActionType::ConfigChange,
                                            "strategy_toggle" => crate::engrams::schemas::SuggestedActionType::StrategyToggle,
                                            "risk_adjustment" => crate::engrams::schemas::SuggestedActionType::RiskAdjustment,
                                            "venue_disable" => crate::engrams::schemas::SuggestedActionType::VenueDisable,
                                            "avoid_token" => crate::engrams::schemas::SuggestedActionType::AvoidToken,
                                            _ => crate::engrams::schemas::SuggestedActionType::ConfigChange,
                                        },
                                        target: rec.target.clone(),
                                        current_value: rec.current_value.clone(),
                                        suggested_value: rec.suggested_value.clone(),
                                        reasoning: rec.reasoning.clone(),
                                    },
                                    confidence: rec.confidence,
                                    supporting_data: crate::engrams::schemas::SupportingData {
                                        trades_analyzed: pnl_stats.total_trades,
                                        time_period: "Last 7 days".to_string(),
                                        relevant_engrams: Vec::new(),
                                        metrics: Some(serde_json::json!({
                                            "win_rate": win_rate,
                                            "total_pnl": pnl_stats.total_pnl,
                                        })),
                                    },
                                    status: crate::engrams::schemas::RecommendationStatus::Pending,
                                    created_at: chrono::Utc::now(),
                                    applied_at: None,
                                };

                                if let Err(e) = analysis_engrams.save_recommendation(&wallet_address, &recommendation).await {
                                    warn!("[Consensus] ⚠️ Failed to save recommendation '{}': {}", rec.title, e);
                                } else {
                                    info!("[Consensus] 💡 Saved recommendation: {} (confidence: {:.0}%)", rec.title, rec.confidence * 100.0);
                                }
                            }

                            // Save the full consensus analysis as an engram
                            let analysis_id = uuid::Uuid::new_v4();
                            let consensus_analysis = crate::engrams::schemas::ConsensusAnalysis {
                                analysis_id,
                                analysis_type: crate::engrams::schemas::ConsensusAnalysisType::Scheduled,
                                time_period: "Last 7 days".to_string(),
                                total_trades_analyzed: pnl_stats.total_trades,
                                overall_assessment: result.overall_assessment.clone(),
                                risk_alerts: result.risk_alerts.clone(),
                                recommendations_count: result.recommendations.len() as u32,
                                recommendation_ids,
                                avg_confidence: result.avg_confidence,
                                models_queried: result.model_votes.clone(),
                                total_latency_ms: result.total_latency_ms,
                                context_summary: crate::engrams::schemas::AnalysisContextSummary {
                                    win_rate,
                                    total_pnl_sol: pnl_stats.total_pnl,
                                    top_venue: None,
                                    error_count: 0,
                                },
                                created_at: chrono::Utc::now(),
                            };

                            if let Err(e) = analysis_engrams.save_consensus_analysis(&wallet_address, &consensus_analysis).await {
                                warn!("[Consensus] ⚠️ Failed to save consensus analysis: {}", e);
                            } else {
                                info!("[Consensus] 📊 Saved consensus analysis: {} (confidence: {:.0}%)", analysis_id, result.avg_confidence * 100.0);
                            }

                            // Save trade analyses from the result
                            for trade_analysis in &result.trade_analyses {
                                // Find the matching trade from recent_trades to get additional context
                                let trade_context = context_recent_trades.iter()
                                    .find(|t| t.position_id.to_string() == trade_analysis.position_id);

                                let analysis = crate::engrams::schemas::TradeAnalysis {
                                    analysis_id: uuid::Uuid::new_v4(),
                                    position_id: uuid::Uuid::parse_str(&trade_analysis.position_id)
                                        .unwrap_or_else(|_| uuid::Uuid::nil()),
                                    token_symbol: trade_context.map(|t| t.token_symbol.clone()).unwrap_or_else(|| "Unknown".to_string()),
                                    venue: trade_context.map(|t| t.venue.clone()).unwrap_or_else(|| "pump.fun".to_string()),
                                    pnl_sol: trade_context.map(|t| t.pnl_sol).unwrap_or(0.0),
                                    exit_reason: trade_context.map(|t| t.exit_reason.clone()).unwrap_or_else(|| "Unknown".to_string()),
                                    root_cause: trade_analysis.root_cause.clone(),
                                    config_issue: trade_analysis.config_issue.clone(),
                                    pattern: trade_analysis.pattern.clone(),
                                    suggested_fix: trade_analysis.suggested_fix.clone(),
                                    confidence: 0.7, // Default confidence for trade analyses
                                    created_at: chrono::Utc::now(),
                                };

                                if let Err(e) = analysis_engrams.save_trade_analysis(&wallet_address, &analysis).await {
                                    warn!("[Consensus] ⚠️ Failed to save trade analysis: {}", e);
                                } else {
                                    info!("[Consensus] 🔍 Saved trade analysis for position: {}", trade_analysis.position_id);
                                }
                            }

                            // Save pattern summary if present
                            if let Some(pattern_summary) = &result.pattern_summary {
                                let stored_summary = crate::engrams::schemas::StoredPatternSummary {
                                    summary_id: uuid::Uuid::new_v4(),
                                    losing_patterns: pattern_summary.losing_patterns.clone(),
                                    winning_patterns: pattern_summary.winning_patterns.clone(),
                                    config_recommendations: pattern_summary.config_recommendations.clone(),
                                    trades_analyzed: context_recent_trades.len() as u32,
                                    time_period: "Last 7 days".to_string(),
                                    created_at: chrono::Utc::now(),
                                };

                                if let Err(e) = analysis_engrams.save_pattern_summary(&wallet_address, &stored_summary).await {
                                    warn!("[Consensus] ⚠️ Failed to save pattern summary: {}", e);
                                } else {
                                    info!("[Consensus] 📈 Saved pattern summary with {} losing and {} winning patterns",
                                        stored_summary.losing_patterns.len(),
                                        stored_summary.winning_patterns.len()
                                    );
                                }
                            }

                            // Emit event for frontend real-time update
                            let event = crate::events::ArbEvent::new(
                                "consensus.analysis_complete",
                                crate::events::EventSource::Agent(crate::events::AgentType::StrategyEngine),
                                "arb.consensus.analysis",
                                serde_json::json!({
                                    "session_id": session_id,
                                    "recommendations_count": result.recommendations.len(),
                                    "risk_alerts_count": result.risk_alerts.len(),
                                    "avg_confidence": result.avg_confidence,
                                    "models_queried": result.model_votes,
                                    "timestamp": chrono::Utc::now(),
                                }),
                            );
                            let _ = analysis_event_tx.send(event);
                        }
                        Err(e) => {
                            warn!("[Consensus] ❌ Analysis failed: {}", e);
                        }
                    }

                    tokio::time::sleep(analysis_interval).await;
                }
            });
            info!("✅ Periodic consensus analysis started (every 5 minutes)");
        } else {
            info!("ℹ️ Periodic consensus analysis skipped (wallet/engrams/consensus not configured)");
        }

        // Start daily metrics scheduler (runs at 00:05 UTC)
        let metrics_wallet = dev_signer_for_metrics.get_address().map(|s| s.to_string());
        if metrics_wallet.is_some() && engrams_client_for_metrics.is_configured() {
            let wallet_address = metrics_wallet.unwrap();
            tokio::spawn(async move {
                crate::agents::start_daily_metrics_scheduler(
                    position_repo_for_metrics,
                    engrams_client_for_metrics,
                    wallet_address,
                ).await;
            });
            info!("✅ Daily metrics scheduler started (runs at 00:05 UTC)");
        } else {
            info!("ℹ️ Daily metrics scheduler skipped (wallet/engrams not configured)");
        }
    });

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
        // MCP - Standard JSON-RPC + Crossroads discovery + tool execution
        .route("/mcp/jsonrpc", post(mcp::handle_jsonrpc))
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
        // Graduation Tracker (Token Watchlist with Engram Persistence)
        .route("/graduation/track", post(graduation_handlers::track_token))
        .route("/graduation/untrack/:mint", post(graduation_handlers::untrack_token))
        .route("/graduation/clear", post(graduation_handlers::clear_all_tracked))
        .route("/graduation/tracked", get(graduation_handlers::list_tracked))
        .route("/graduation/tracked/:mint", get(graduation_handlers::is_tracked))
        .route("/graduation/stats", get(graduation_handlers::get_tracker_stats))
        .route("/graduation/start", post(graduation_handlers::start_tracker))
        .route("/graduation/stop", post(graduation_handlers::stop_tracker))
        .route("/graduation/config", get(graduation_handlers::get_tracker_config))
        .route("/graduation/config", axum::routing::put(graduation_handlers::update_tracker_config))
        // Graduation Sniper (Auto-sell positions on graduation)
        .route("/sniper/stats", get(sniper_handlers::get_sniper_stats))
        .route("/sniper/positions", get(sniper_handlers::list_snipe_positions))
        .route("/sniper/positions", post(sniper_handlers::add_snipe_position))
        .route("/sniper/positions/:mint", axum::routing::delete(sniper_handlers::remove_snipe_position))
        .route("/sniper/positions/:mint/sell", post(sniper_handlers::manual_sell_position))
        .route("/sniper/start", post(sniper_handlers::start_sniper))
        .route("/sniper/stop", post(sniper_handlers::stop_sniper))
        .route("/sniper/config", get(sniper_handlers::get_sniper_config))
        .route("/sniper/config", axum::routing::put(sniper_handlers::update_sniper_config))
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
        .route("/consensus/config", get(consensus_handlers::get_consensus_config))
        .route("/consensus/config", axum::routing::put(consensus_handlers::update_consensus_config))
        .route("/consensus/config/reset", post(consensus_handlers::reset_consensus_config))
        .route("/consensus/conversations", get(consensus_handlers::list_conversations))
        .route("/consensus/conversations/:id", get(consensus_handlers::get_conversation_detail))
        .route("/consensus/recommendations", get(consensus_handlers::list_recommendations))
        .route("/consensus/recommendations/:id/status", axum::routing::put(consensus_handlers::update_recommendation_status))
        .route("/consensus/learning", get(consensus_handlers::get_learning_summary))
        .route("/consensus/engrams", get(consensus_handlers::list_engrams))
        .route("/consensus/engrams/:key", get(consensus_handlers::get_engram_detail))
        .route("/consensus/models/discovery", get(consensus_handlers::get_model_discovery_status))
        .route("/consensus/models/refresh", post(consensus_handlers::refresh_models))
        .route("/consensus/models/discovered", get(consensus_handlers::get_discovered_models))
        .route("/consensus/trade-analyses", get(consensus_handlers::list_trade_analyses))
        .route("/consensus/patterns", get(consensus_handlers::get_pattern_summary))
        .route("/consensus/analysis-summary", get(consensus_handlers::get_analysis_summary))
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
        .route("/engram/insights", get(engram_handlers::get_learning_insights))
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
        // Config (Risk Level Presets)
        .route("/config/risk", get(config_handlers::get_risk_level))
        .route("/config/risk", post(config_handlers::set_risk_level))
        .route("/config/risk/custom", post(config_handlers::set_custom_risk))
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
        .route("/helius/sender/send", post(helius_handlers::send_transaction))
        .route("/helius/das/lookup", post(helius_handlers::das_lookup))
        .route("/helius/das/assets", get(helius_handlers::das_assets_by_owner))
        .route("/helius/config", get(helius_handlers::get_helius_config))
        .route("/helius/config", axum::routing::put(helius_handlers::update_helius_config))
        // Positions (Exit Management)
        .route("/positions", get(position_handlers::get_positions))
        .route("/positions/history", get(position_handlers::get_position_history))
        .route("/positions/exposure", get(position_handlers::get_exposure))
        .route("/positions/pnl-summary", get(position_handlers::get_pnl_summary))
        .route("/positions/reconcile", post(position_handlers::reconcile_wallet))
        .route("/positions/monitor/status", get(position_handlers::get_monitor_status))
        .route("/positions/monitor/start", post(position_handlers::start_monitor))
        .route("/positions/monitor/stop", post(position_handlers::stop_monitor))
        .route("/positions/emergency-close", post(position_handlers::emergency_close_all))
        .route("/positions/sell-all", post(position_handlers::sell_all_wallet_tokens))
        .route("/positions/force-clear", post(position_handlers::force_clear_all_positions))
        .route("/positions/exit-config", axum::routing::put(position_handlers::update_all_positions_exit_config))
        .route("/positions/:id", get(position_handlers::get_position))
        .route("/positions/:id/close", post(position_handlers::close_position))
        .route("/positions/:id/exit-config", axum::routing::put(position_handlers::update_position_exit_config))
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
        .route("/positions/stream", get(sse::positions_stream))
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any),
        )
        .layer(TraceLayer::new_for_http())
}
