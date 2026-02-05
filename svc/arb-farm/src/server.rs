use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

use crate::agents::{
    spawn_autonomous_executor, spawn_hecate_notifier, start_autonomous_executor,
    AutonomousExecutor, CurveMetricsCollector, CurveOpportunityScorer, EngramHarvester,
    GraduationSniper, KolDiscoveryAgent, OverseerConfig, ResilienceOverseer, ScannerAgent,
    StrategyEngine,
};
use crate::config::Config;
use crate::consensus::{ConsensusConfig, ConsensusEngine};
use crate::database::repositories::ConsensusRepository;
use crate::database::repositories::KolRepository;
use crate::database::{EdgeRepository, PositionRepository, StrategyRepository, TradeRepository};
use crate::engrams::EngramsClient;
use crate::events::{ArbEvent, EventBus};
use crate::execution::risk::RiskConfig;
use crate::execution::{
    ApprovalManager, CapitalManager, CurveTransactionBuilder, ExecutorAgent, ExecutorConfig,
    JitoClient, MonitorConfig, PositionCommand, PositionExecutor, PositionMonitor,
    RealtimePositionMonitor, TransactionBuilder, TransactionSimulator,
};
use crate::handlers::engram::init_harvester;
use crate::handlers::swarm::{init_circuit_breakers, init_overseer};
use crate::helius::{
    priority_fee::PriorityFeeMonitor, DasClient, HeliusClient, HeliusSender, LaserStreamClient,
};
use crate::models::KOLTracker;
use crate::resilience::CircuitBreakerRegistry;
use crate::venues::curves::{HolderAnalyzer, OnChainFetcher};
use crate::venues::curves::{MoonshotVenue, PumpFunVenue};
use crate::venues::dex::JupiterVenue;
use crate::wallet::turnkey::{TurnkeyConfig, TurnkeySigner};
use crate::wallet::DevWalletSigner;
use crate::webhooks::helius::HeliusWebhookClient;
use nullblock_mcp_client::McpClient;

pub const DEFAULT_EVENT_CHANNEL_CAPACITY: usize = 1024;
pub const DEFAULT_SCAN_INTERVAL_MS: u64 = 5000;

pub fn get_event_channel_capacity() -> usize {
    std::env::var("ARB_EVENT_CHANNEL_CAPACITY")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_EVENT_CHANNEL_CAPACITY)
}

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db_pool: PgPool,
    pub event_tx: broadcast::Sender<ArbEvent>,
    pub event_bus: Arc<EventBus>,
    pub scanner: Arc<ScannerAgent>,
    pub executor: Arc<ExecutorAgent>,
    pub simulator: Arc<TransactionSimulator>,
    pub tx_builder: Arc<TransactionBuilder>,
    pub edge_repo: Arc<EdgeRepository>,
    pub strategy_repo: Arc<StrategyRepository>,
    pub trade_repo: Arc<TradeRepository>,
    pub jupiter_venue: Arc<JupiterVenue>,
    pub pump_fun_venue: Arc<PumpFunVenue>,
    pub moonshot_venue: Arc<MoonshotVenue>,
    pub turnkey_signer: Arc<TurnkeySigner>,
    pub risk_config: Arc<RwLock<RiskConfig>>,
    pub helius_webhook_client: Arc<HeliusWebhookClient>,
    pub helius_rpc_client: Arc<HeliusClient>,
    pub helius_sender: Arc<HeliusSender>,
    pub helius_das: Arc<DasClient>,
    pub priority_fee_monitor: Arc<PriorityFeeMonitor>,
    pub kol_tracker: Arc<KOLTracker>,
    pub strategy_engine: Arc<StrategyEngine>,
    pub consensus_engine: Arc<ConsensusEngine>,
    pub consensus_config: Arc<RwLock<ConsensusConfig>>,
    pub engrams_client: Arc<EngramsClient>,
    pub position_repo: Arc<PositionRepository>,
    pub consensus_repo: Arc<ConsensusRepository>,
    pub settings_repo: Arc<crate::database::SettingsRepository>,
    pub kol_repo: Arc<KolRepository>,
    pub laserstream_client: Arc<LaserStreamClient>,
    pub kol_discovery: Arc<KolDiscoveryAgent>,
    pub dev_signer: Arc<DevWalletSigner>,
    pub position_manager: Arc<crate::execution::PositionManager>,
    pub position_monitor: Arc<PositionMonitor>,
    pub jito_client: Arc<JitoClient>,
    pub approval_manager: Arc<ApprovalManager>,
    pub capital_manager: Arc<CapitalManager>,
    pub curve_builder: Arc<CurveTransactionBuilder>,
    pub on_chain_fetcher: Arc<OnChainFetcher>,
    pub metrics_collector: Arc<CurveMetricsCollector>,
    pub holder_analyzer: Arc<HolderAnalyzer>,
    pub curve_scorer: Arc<CurveOpportunityScorer>,
    pub autonomous_executor: Arc<AutonomousExecutor>,
    pub position_executor: Arc<PositionExecutor>,
    pub realtime_monitor: Arc<RealtimePositionMonitor>,
    pub graduation_sniper: Arc<GraduationSniper>,
    pub wallet_max_position_sol: Arc<RwLock<f64>>,
    pub consensus_scheduler_paused: Arc<AtomicBool>,
    pub consensus_last_queried: Arc<RwLock<Option<chrono::DateTime<chrono::Utc>>>>,
}

impl AppState {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        let db_pool = PgPoolOptions::new()
            .max_connections(30) // Consolidated with database/mod.rs
            .acquire_timeout(std::time::Duration::from_secs(30))
            .connect(&config.database_url)
            .await?;

        sqlx::query("SELECT 1")
            .execute(&db_pool)
            .await
            .map_err(|e| {
                anyhow::anyhow!(
                    "Database health check failed: {}. Ensure PostgreSQL is running.",
                    e
                )
            })?;

        tracing::info!("✅ Database connection pool created and health check passed");

        let channel_capacity = get_event_channel_capacity();
        let (event_tx, _) = broadcast::channel(channel_capacity);
        let event_bus = Arc::new(EventBus::new(event_tx.clone(), db_pool.clone()));
        tracing::info!("✅ Event bus initialized (capacity: {})", channel_capacity);

        let scanner = Arc::new(ScannerAgent::new(
            event_tx.clone(),
            DEFAULT_SCAN_INTERVAL_MS,
        ));

        // Initialize venues (shared between scanner and direct access)
        let jupiter_venue = Arc::new(JupiterVenue::new(config.jupiter_api_url.clone()));
        let pump_fun_venue = Arc::new(PumpFunVenue::new(
            config.pump_fun_api_url.clone(),
            config.dexscreener_api_url.clone(),
        ));
        let moonshot_venue = Arc::new(MoonshotVenue::new(config.moonshot_api_url.clone()));

        // Add venues to scanner (cloning the Arc)
        scanner
            .add_venue(Box::new(JupiterVenue::new(config.jupiter_api_url.clone())))
            .await;
        scanner
            .add_venue(Box::new(PumpFunVenue::new(
                config.pump_fun_api_url.clone(),
                config.dexscreener_api_url.clone(),
            )))
            .await;
        scanner
            .add_venue(Box::new(MoonshotVenue::new(
                config.moonshot_api_url.clone(),
            )))
            .await;

        tracing::info!("✅ Scanner agent initialized with 3 venues (Jupiter, pump.fun, moonshot)");

        // Initialize repositories
        let edge_repo = Arc::new(EdgeRepository::new(db_pool.clone()));
        let strategy_repo = Arc::new(StrategyRepository::new(db_pool.clone()));
        let trade_repo = Arc::new(TradeRepository::new(db_pool.clone()));
        tracing::info!("✅ Database repositories initialized");

        // Initialize simulator, transaction builder, and executor
        let simulator = Arc::new(TransactionSimulator::new(config.rpc_url.clone()));
        let tx_builder = Arc::new(TransactionBuilder::new(
            config.jupiter_api_url.clone(),
            config.rpc_url.clone(),
        )?);
        let executor = Arc::new(ExecutorAgent::new(
            config.jito_block_engine_url.clone(),
            config.rpc_url.clone(),
            Default::default(),
            event_tx.clone(),
        ));
        tracing::info!("✅ Executor agent initialized (Jito + Simulation + TransactionBuilder)");

        // Initialize Turnkey signer for wallet delegation
        let turnkey_config = TurnkeyConfig {
            api_url: config.turnkey_api_url.clone(),
            organization_id: config.turnkey_organization_id.clone().unwrap_or_default(),
            api_public_key: config.turnkey_api_public_key.clone(),
            api_private_key: config.turnkey_api_private_key.clone(),
        };
        let is_dev_mode = std::env::var("ARBFARM_DEV_MODE").is_ok();
        let turnkey_signer = Arc::new(if is_dev_mode {
            tracing::info!("🔓 Dev mode: no daily volume limits");
            TurnkeySigner::new_dev(turnkey_config)
        } else {
            TurnkeySigner::new(turnkey_config)
        });

        // Initialize DevWalletSigner for development mode signing
        let dev_signer = match DevWalletSigner::new(
            config.wallet_private_key.as_deref(),
            config.wallet_address.as_deref(),
        ) {
            Ok(signer) => {
                if signer.is_configured() {
                    tracing::info!(
                        "✅ Dev wallet signer initialized: {}",
                        signer.get_address().unwrap_or("unknown")
                    );
                } else {
                    tracing::info!("✅ Dev wallet signer initialized (no private key)");
                }
                Arc::new(signer)
            }
            Err(e) => {
                tracing::warn!("⚠️ Failed to initialize dev signer: {}", e);
                Arc::new(DevWalletSigner::new(None, None).unwrap())
            }
        };

        // Auto-initialize Turnkey (production) or dev wallet
        if dev_signer.is_configured() {
            // Connect the dev signer to enable transaction signing
            if let Err(e) = dev_signer.connect().await {
                tracing::warn!("⚠️ Failed to connect dev signer: {}", e);
            }

            // Use dev signer - auto-configure Turnkey with dev wallet address for status tracking
            if let Some(ref wallet_address) = config.wallet_address {
                let dev_wallet_id = format!(
                    "dev_wallet_{}",
                    wallet_address.chars().take(8).collect::<String>()
                );
                if let Err(e) = turnkey_signer
                    .set_wallet(wallet_address.clone(), dev_wallet_id)
                    .await
                {
                    tracing::warn!("⚠️ Failed to auto-configure turnkey status: {}", e);
                }
            }
            tracing::info!("✅ Wallet signing mode: DEV (private key from env, auto-connected)");
        } else if config.turnkey_api_public_key.is_some()
            && config.turnkey_api_private_key.is_some()
        {
            tracing::info!("✅ Wallet signing mode: PRODUCTION (Turnkey delegation)");
        } else {
            tracing::warn!("⚠️ No wallet signing configured - transactions will fail");
        }

        // Initialize risk config with MEDIUM profile - balanced risk/reward
        let mut initial_risk = RiskConfig::medium();
        initial_risk.daily_loss_limit_sol = config.default_daily_loss_limit_sol;
        let risk_config = Arc::new(RwLock::new(initial_risk));
        tracing::info!(
            "✅ Risk config initialized (MEDIUM profile: {:.2} SOL daily loss limit)",
            config.default_daily_loss_limit_sol
        );

        // Initialize Helius clients (webhook + RPC + sender + DAS)
        let helius_webhook_client = Arc::new(HeliusWebhookClient::new(
            config.helius_api_url.clone(),
            config.helius_api_key.clone(),
        ));
        if helius_webhook_client.is_configured() {
            tracing::info!("✅ Helius webhook client initialized");
        } else {
            tracing::warn!("⚠️ Helius API key not configured - webhooks disabled");
        }

        // Initialize comprehensive Helius RPC client
        let helius_rpc_client =
            Arc::new(HeliusClient::new(&config).with_event_bus(event_bus.clone()));
        tracing::info!(
            "✅ Helius RPC client initialized (url: {})",
            config.helius_api_url
        );

        // Initialize Helius Sender for fast TX submission
        let helius_sender = Arc::new(HeliusSender::new(
            helius_rpc_client.clone(),
            event_bus.clone(),
        ));
        tracing::info!(
            "✅ Helius Sender initialized (url: {})",
            config.helius_sender_url
        );

        // Initialize DAS (Digital Asset Standard) client
        let helius_das = Arc::new(DasClient::new(helius_rpc_client.clone(), event_bus.clone()));
        tracing::info!("✅ Helius DAS client initialized");

        // Initialize Priority Fee Monitor
        let priority_fee_monitor = Arc::new(PriorityFeeMonitor::new(
            helius_rpc_client.clone(),
            event_bus.clone(),
        ));
        tracing::info!("✅ Priority Fee Monitor initialized");

        // Initialize KOL tracker
        let kol_tracker = Arc::new(KOLTracker::new());
        tracing::info!("✅ KOL tracker initialized");

        // Initialize Strategy Engine with default strategies
        let strategy_engine = Arc::new(StrategyEngine::new(event_tx.clone()));

        // Register default strategies for each venue type (in DB first, then in memory)
        use crate::database::repositories::strategies::CreateStrategyRecord;
        use crate::models::{RiskParams, Strategy};

        let default_wallet = config
            .wallet_address
            .clone()
            .unwrap_or_else(|| "default".to_string());

        // Helper to create or get strategy from DB
        async fn get_or_create_strategy(
            strategy_repo: &StrategyRepository,
            name: &str,
            wallet_address: &str,
            strategy_type: &str,
            venue_types: Vec<String>,
            execution_mode: &str,
            risk_params: RiskParams,
        ) -> Option<Strategy> {
            // First check if it exists
            if let Ok(existing) = strategy_repo.list(None, 100, 0).await {
                if let Some(record) = existing.iter().find(|s| s.name == name) {
                    return Some(Strategy {
                        id: record.id,
                        wallet_address: record.wallet_address.clone(),
                        name: record.name.clone(),
                        strategy_type: record.strategy_type.clone(),
                        venue_types: record.venue_types.clone(),
                        execution_mode: record.execution_mode.clone(),
                        risk_params: serde_json::from_value(record.risk_params.clone())
                            .unwrap_or_default(),
                        is_active: record.is_active,
                        created_at: record.created_at,
                        updated_at: record.updated_at,
                        last_tested_at: None,
                        last_executed_at: None,
                        test_results: None,
                    });
                }
            }

            // Create new
            match strategy_repo
                .create(CreateStrategyRecord {
                    wallet_address: wallet_address.to_string(),
                    name: name.to_string(),
                    strategy_type: strategy_type.to_string(),
                    venue_types,
                    execution_mode: execution_mode.to_string(),
                    risk_params,
                })
                .await
            {
                Ok(record) => Some(Strategy {
                    id: record.id,
                    wallet_address: record.wallet_address,
                    name: record.name,
                    strategy_type: record.strategy_type,
                    venue_types: record.venue_types,
                    execution_mode: record.execution_mode,
                    risk_params: serde_json::from_value(record.risk_params).unwrap_or_default(),
                    is_active: record.is_active,
                    created_at: record.created_at,
                    updated_at: record.updated_at,
                    last_tested_at: None,
                    last_executed_at: None,
                    test_results: None,
                }),
                Err(e) => {
                    tracing::warn!("Failed to create strategy {}: {}", name, e);
                    None
                }
            }
        }

        // TODO: WIP - re-enable when copy trading strategy is ready
        // // KOL Copy Trade Strategy - use preset for copy trading
        // // Only match DEX signals - bonding curve signals should go to Curve Graduation strategy
        // if let Some(kol_strategy) = get_or_create_strategy(
        //     &strategy_repo,
        //     "KOL Copy Trading",
        //     &default_wallet,
        //     "copy_trade",
        //     vec!["dexamm".to_string(), "DexAmm".to_string()],
        //     "agent_directed",
        //     RiskParams::for_copy_trade(),
        // ).await {
        //     strategy_engine.add_strategy(kol_strategy).await;
        // }

        // Graduation Snipe Strategy - Buy at high graduation progress, sell on Raydium migration
        // Listens for graduation_imminent events (95%+) and creates entry signals
        if let Some(snipe_strategy) = get_or_create_strategy(
            &strategy_repo,
            "Graduation Snipe",
            &default_wallet,
            "graduation_snipe",
            vec!["bondingcurve".to_string(), "BondingCurve".to_string()],
            "autonomous", // Autonomous execution for speed
            RiskParams {
                // Synced with global config
                max_position_sol: 0.08, // Reduced from 0.3 per LLM consensus
                daily_loss_limit_sol: 1.0, // Matches global default
                min_profit_bps: 25,     // Lower profit threshold (0.25%)
                max_slippage_bps: 300,  // Higher slippage tolerance for speed
                max_risk_score: 50,     // Moderate risk tolerance
                require_simulation: false, // Skip simulation for speed
                auto_execute_atomic: true, // Auto-execute atomic ops
                auto_execute_enabled: true, // ON by default - snipes must be fast
                require_consensus: false, // No consensus for time-sensitive snipes
                require_confirmation: false, // No confirmation needed
                staleness_threshold_hours: 1, // Short staleness window
                stop_loss_percent: Some(13.0), // DEFENSIVE: 13% stop (widened from 10%)
                take_profit_percent: Some(15.0), // DEFENSIVE: 15% TP (strong momentum extends)
                trailing_stop_percent: Some(8.0), // DEFENSIVE: 8% trailing stop
                time_limit_minutes: Some(5), // DEFENSIVE: 5 min
                base_currency: "sol".to_string(),
                max_capital_allocation_percent: 5.0, // Conservative 5% allocation
                concurrent_positions: Some(3),       // Up to 3 snipe positions
                momentum_adaptive_exits: true,       // Enable for graduation snipes
                let_winners_run: true,               // Let winners run post-graduation
            },
        )
        .await
        {
            strategy_engine.add_strategy(snipe_strategy).await;
        }

        if let Some(raydium_strategy) = get_or_create_strategy(
            &strategy_repo,
            "Raydium Snipe",
            &default_wallet,
            "raydium_snipe",
            vec!["dexamm".to_string(), "DexAmm".to_string()],
            "autonomous",
            RiskParams {
                max_position_sol: 0.08,
                daily_loss_limit_sol: 1.0,
                min_profit_bps: 25,
                max_slippage_bps: 500,
                max_risk_score: 50,
                require_simulation: false,
                auto_execute_atomic: true,
                auto_execute_enabled: true,
                require_consensus: false,
                require_confirmation: false,
                staleness_threshold_hours: 1,
                stop_loss_percent: Some(15.0),
                take_profit_percent: Some(30.0),
                trailing_stop_percent: Some(10.0),
                time_limit_minutes: Some(5),
                base_currency: "sol".to_string(),
                max_capital_allocation_percent: 5.0,
                concurrent_positions: Some(2),
                momentum_adaptive_exits: false,
                let_winners_run: false,
            },
        )
        .await
        {
            strategy_engine.add_strategy(raydium_strategy).await;
        }

        let strategy_count = strategy_engine.list_strategies().await.len();
        tracing::info!(
            "✅ Strategy engine initialized with {} default strategies (persisted to DB)",
            strategy_count
        );

        // AUTO-SYNC: Ensure all strategies match global RiskConfig
        // This handles the case where strategies exist in DB with old values
        {
            let global_config = risk_config.read().await;
            let strategies = strategy_engine.list_strategies().await;
            let mut synced_count = 0;

            for strategy in strategies
                .iter()
                .filter(|s| s.strategy_type == "graduation_snipe")
            {
                // Only sync if strategy has smaller max_position_sol than global
                if strategy.risk_params.max_position_sol < global_config.max_position_sol {
                    let mut updated_params = strategy.risk_params.clone();
                    updated_params.max_position_sol = global_config.max_position_sol;
                    updated_params.daily_loss_limit_sol = global_config.daily_loss_limit_sol;

                    // Update in-memory
                    if let Err(e) = strategy_engine
                        .set_risk_params(strategy.id, updated_params.clone())
                        .await
                    {
                        tracing::warn!(strategy_id = %strategy.id, error = %e, "Failed to sync strategy on startup");
                        continue;
                    }

                    // Persist to database
                    use crate::database::repositories::strategies::UpdateStrategyRecord;
                    if let Err(e) = strategy_repo
                        .update(
                            strategy.id,
                            UpdateStrategyRecord {
                                name: None,
                                venue_types: None,
                                execution_mode: None,
                                risk_params: Some(updated_params),
                                is_active: None,
                            },
                        )
                        .await
                    {
                        tracing::warn!(strategy_id = %strategy.id, error = %e, "Failed to persist strategy sync");
                    }

                    synced_count += 1;
                    tracing::info!(
                        strategy_id = %strategy.id,
                        strategy_name = %strategy.name,
                        old_max = strategy.risk_params.max_position_sol,
                        new_max = global_config.max_position_sol,
                        "🔄 Auto-synced strategy to global config"
                    );
                }
            }

            if synced_count > 0 {
                tracing::info!(
                    "✅ Auto-synced {} strategies to global RiskConfig (0.3 SOL max position)",
                    synced_count
                );
            }
        }

        // NOTE: execution_mode reconciliation moved to AFTER engrams restoration (below)
        // to ensure persisted auto_execute_enabled state is honored

        // Connect scanner to strategy engine for automatic signal processing
        scanner.set_strategy_engine(strategy_engine.clone()).await;
        tracing::info!("✅ Scanner connected to strategy engine (auto-processing enabled)");

        // Initialize Erebus client for fetching agent API keys from DB
        let erebus_client = crate::erebus::ErebusClient::new(&config.erebus_url);

        // Fetch OpenRouter API key from Erebus (DB) first, fall back to env var
        let openrouter_api_key = match erebus_client.get_openrouter_key().await {
            Some(key) => {
                tracing::info!(
                    "✅ Retrieved OpenRouter API key from Erebus (agent_api_keys table)"
                );
                Some(key)
            }
            None => {
                if config.openrouter_api_key.is_some() {
                    tracing::info!(
                        "📝 Using OpenRouter API key from environment variable (fallback)"
                    );
                }
                config.openrouter_api_key.clone()
            }
        };

        // Initialize LLM consensus engine with best reasoning models from OpenRouter
        let consensus_engine = if let Some(ref api_key) = openrouter_api_key {
            tracing::info!("🔍 Discovering best reasoning models from OpenRouter...");
            let discovered_models = crate::consensus::discover_best_reasoning_models(api_key).await;

            let base_engine = if !discovered_models.is_empty() {
                let model_ids: Vec<String> = discovered_models
                    .iter()
                    .map(|m| m.model_id.clone())
                    .collect();
                tracing::info!(
                    "✅ Discovered {} best reasoning models:",
                    discovered_models.len()
                );
                for model in &discovered_models {
                    tracing::info!("   - {} (weight: {:.1})", model.display_name, model.weight);
                }
                ConsensusEngine::new(api_key.clone())
                    .with_models(model_ids)
                    .with_event_tx(event_tx.clone())
            } else {
                ConsensusEngine::new(api_key.clone()).with_event_tx(event_tx.clone())
            };

            // Try to initialize MCP client for agentic tool calling (read-only tools)
            let mcp_url = "http://localhost:9007/mcp/jsonrpc";
            let mcp_client = McpClient::new(mcp_url);
            let engine_with_mcp = match mcp_client.connect().await {
                Ok(_) => match mcp_client.list_tools().await {
                    Ok(tools) => {
                        let read_only_count =
                            nullblock_mcp_client::filter_read_only(tools.clone()).len();
                        tracing::info!(
                            "✅ MCP client connected ({} tools, {} read-only for consensus)",
                            tools.len(),
                            read_only_count
                        );
                        base_engine.with_mcp_client(Arc::new(mcp_client), tools)
                    }
                    Err(e) => {
                        tracing::warn!("⚠️ MCP tools fetch failed: {} - consensus will operate without tool calling", e);
                        base_engine
                    }
                },
                Err(e) => {
                    tracing::warn!("⚠️ MCP client connection failed: {} - consensus will operate without tool calling", e);
                    base_engine
                }
            };

            Arc::new(engine_with_mcp)
        } else {
            tracing::error!("⚠️⚠️⚠️ NO OPENROUTER API KEY CONFIGURED ⚠️⚠️⚠️");
            tracing::error!("Consensus engine will be DISABLED. LLM analysis and recommendations will NOT work.");
            tracing::error!("To enable: Set OPENROUTER_API_KEY env var or add key to agent_api_keys table in Erebus DB");
            Arc::new(ConsensusEngine::new_disabled())
        };

        if openrouter_api_key.is_some() {
            if consensus_engine.is_agentic_enabled() {
                tracing::info!(
                    "✅ Consensus engine initialized (OpenRouter + agentic tool calling enabled)"
                );
            } else {
                tracing::info!(
                    "✅ Consensus engine initialized (OpenRouter multi-LLM, tool calling disabled)"
                );
            }
        } else {
            tracing::warn!("⚠️ OpenRouter API key not configured - consensus disabled");
        }

        let consensus_config = Arc::new(RwLock::new(ConsensusConfig::default()));
        tracing::info!(
            "✅ Consensus config initialized (execution_gating: {}, fail_open: {})",
            ConsensusConfig::default().consensus_enabled_for_execution,
            ConsensusConfig::default().fail_open_on_consensus_error
        );

        // Initialize Engrams client for persistent memory
        let engrams_client = Arc::new(EngramsClient::new(config.engrams_url.clone()));
        if engrams_client.is_configured() {
            tracing::info!(
                "✅ Engrams client initialized (url: {})",
                config.engrams_url
            );
        } else {
            tracing::warn!("⚠️ Engrams service URL not configured - persistence disabled");
        }

        // Initialize EngramHarvester for local pattern learning with remote sync
        let engram_harvester = EngramHarvester::new(event_tx.clone())
            .with_engrams_client(engrams_client.clone(), default_wallet.clone());

        // Restore patterns from remote engrams service on startup
        if engrams_client.is_configured() {
            let restored_count = engram_harvester.restore_from_remote().await;
            if restored_count > 0 {
                tracing::info!(
                    "✅ Engram Harvester initialized with {} restored patterns from remote",
                    restored_count
                );
            } else {
                tracing::info!("✅ Engram Harvester initialized (no prior patterns to restore)");
            }
        } else {
            tracing::info!(
                "✅ Engram Harvester initialized (local-only mode - remote sync disabled)"
            );
        }

        init_harvester(engram_harvester);

        // Initialize Resilience Overseer for swarm health monitoring
        let overseer_config = OverseerConfig::default();
        let resilience_overseer = ResilienceOverseer::new(overseer_config, event_tx.clone());
        init_overseer(resilience_overseer);
        tracing::info!("✅ Resilience Overseer initialized (swarm health monitoring)");

        // Initialize Circuit Breakers for fault tolerance
        let circuit_registry = CircuitBreakerRegistry::default();
        init_circuit_breakers(circuit_registry);
        tracing::info!("✅ Circuit Breakers initialized");

        // Initialize LaserStream client for real-time Solana data
        let laserstream_client = Arc::new(LaserStreamClient::new(
            config.helius_laserstream_url.clone(),
            config.helius_api_key.clone(),
            event_bus.clone(),
        ));
        if laserstream_client.is_configured() {
            tracing::info!(
                "✅ LaserStream client initialized (endpoint: {})",
                config.helius_laserstream_url
            );
            // Start reconnection monitor for auto-reconnect on disconnect
            laserstream_client.start_reconnect_monitor();
        } else {
            tracing::warn!("⚠️ LaserStream not configured - real-time streaming disabled");
        }

        // Initialize KOL discovery agent
        let kol_discovery = Arc::new(
            KolDiscoveryAgent::new()
                .with_engrams_client(engrams_client.clone())
                .with_owner_wallet(default_wallet.clone()),
        );
        tracing::info!("✅ KOL Discovery Agent initialized");

        // Restore workflow state from engrams (persisted data from previous sessions)
        if engrams_client.is_configured() {
            tracing::info!(
                "🔄 Starting engrams restoration for wallet: {}",
                default_wallet
            );
            let workflow_state = engrams_client.restore_workflow_state(&default_wallet).await;

            tracing::info!(
                "📦 Engrams returned: {} strategies, {} KOLs, {} avoidances, {} patterns",
                workflow_state.strategies.len(),
                workflow_state.discovered_kols.len(),
                workflow_state.avoidances.len(),
                workflow_state.patterns.len()
            );

            // Log each strategy from engrams
            for es in &workflow_state.strategies {
                if let Ok(risk_params) =
                    serde_json::from_value::<RiskParams>(es.risk_params.clone())
                {
                    tracing::info!(
                        "  └─ Engram strategy: id={}, name={}, execution_mode={}, auto_execute_enabled={}",
                        es.strategy_id, es.name, es.execution_mode, risk_params.auto_execute_enabled
                    );
                }
            }

            // Restore discovered KOLs
            if !workflow_state.discovered_kols.is_empty() {
                kol_discovery
                    .restore_kols(workflow_state.discovered_kols)
                    .await;
            }

            // Restore strategies from engrams (authoritative source for state)
            for engram_strategy in &workflow_state.strategies {
                if let Ok(strategy_id) = uuid::Uuid::parse_str(&engram_strategy.strategy_id) {
                    if let Some(existing) = strategy_engine.get_strategy(strategy_id).await {
                        // Update existing strategy's is_active from engram
                        if existing.is_active != engram_strategy.is_active {
                            tracing::info!(
                                "Restoring strategy '{}' active state from engrams: {}",
                                existing.name,
                                engram_strategy.is_active
                            );
                            if let Err(e) = strategy_engine
                                .toggle_strategy(strategy_id, engram_strategy.is_active)
                                .await
                            {
                                tracing::warn!("Failed to apply strategy update: {}", e);
                            }
                        }

                        // Restore execution_mode from engrams (authoritative source)
                        if existing.execution_mode != engram_strategy.execution_mode {
                            tracing::info!(
                                "Restoring strategy '{}' execution_mode from engrams: {} -> {}",
                                existing.name,
                                existing.execution_mode,
                                engram_strategy.execution_mode
                            );
                            if let Err(e) = strategy_engine
                                .set_execution_mode(
                                    strategy_id,
                                    engram_strategy.execution_mode.clone(),
                                )
                                .await
                            {
                                tracing::warn!("Failed to apply strategy update: {}", e);
                            }
                        }

                        // Restore risk_params from engrams (authoritative source)
                        if let Ok(engram_risk_params) = serde_json::from_value::<RiskParams>(
                            engram_strategy.risk_params.clone(),
                        ) {
                            // Check if risk params differ (compare key fields)
                            if existing.risk_params.max_position_sol
                                != engram_risk_params.max_position_sol
                                || existing.risk_params.max_risk_score
                                    != engram_risk_params.max_risk_score
                                || existing.risk_params.auto_execute_enabled
                                    != engram_risk_params.auto_execute_enabled
                            {
                                tracing::info!(
                                    "Restoring strategy '{}' risk params from engrams (max_position: {} SOL, max_risk: {}, auto_execute: {})",
                                    existing.name,
                                    engram_risk_params.max_position_sol,
                                    engram_risk_params.max_risk_score,
                                    engram_risk_params.auto_execute_enabled
                                );
                                if let Err(e) = strategy_engine
                                    .set_risk_params(strategy_id, engram_risk_params)
                                    .await
                                {
                                    tracing::warn!("Failed to apply strategy update: {}", e);
                                }
                            }
                        }
                    } else {
                        // Strategy not in engine - create it from engram
                        let strategy = Strategy {
                            id: strategy_id,
                            wallet_address: default_wallet.clone(),
                            name: engram_strategy.name.clone(),
                            strategy_type: engram_strategy.strategy_type.clone(),
                            venue_types: engram_strategy.venue_types.clone(),
                            execution_mode: engram_strategy.execution_mode.clone(),
                            risk_params: serde_json::from_value(
                                engram_strategy.risk_params.clone(),
                            )
                            .unwrap_or_default(),
                            is_active: engram_strategy.is_active,
                            created_at: chrono::Utc::now(),
                            updated_at: chrono::Utc::now(),
                            last_tested_at: None,
                            last_executed_at: None,
                            test_results: None,
                        };
                        strategy_engine.add_strategy(strategy).await;
                        tracing::info!(
                            "Restored strategy '{}' from engrams (active: {})",
                            engram_strategy.name,
                            engram_strategy.is_active
                        );
                    }
                }
            }

            // Restore avoidance list to strategy engine
            for avoidance in &workflow_state.avoidances {
                tracing::debug!(
                    "Restored avoidance: {} ({}) - {}",
                    avoidance.address,
                    avoidance.entity_type,
                    avoidance.reason
                );
            }

            tracing::info!(
                "✅ Workflow state restored: {} KOLs, {} strategies, {} avoidances, {} patterns",
                kol_discovery.get_stats().await.total_kols_discovered,
                workflow_state.strategies.len(),
                workflow_state.avoidances.len(),
                workflow_state.patterns.len()
            );
        } else {
            tracing::warn!("⚠️ Engrams client not configured - skipping workflow state restoration. Strategies will use DB defaults.");
        }

        // Log strategy states BEFORE reconciliation for debugging
        tracing::info!("📋 Pre-reconciliation strategy states:");
        let strategies = strategy_engine.list_strategies().await;
        for strategy in strategies.iter() {
            tracing::info!(
                strategy_id = %strategy.id,
                strategy_name = %strategy.name,
                strategy_type = %strategy.strategy_type,
                execution_mode = %strategy.execution_mode,
                auto_execute_enabled = strategy.risk_params.auto_execute_enabled,
                is_active = strategy.is_active,
                "  └─ Strategy state"
            );
        }

        // Reconcile curve strategies to be ACTIVE by default (unless sniper disabled)
        // This ensures sniper strategies are ready to execute immediately on startup
        let sniper_disabled_env = std::env::var("ARBFARM_DISABLE_SNIPER")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false);
        let sniper_disabled_file = std::path::Path::new("/tmp/arb-no-snipe").exists();
        let sniper_disabled = sniper_disabled_env || sniper_disabled_file;
        if !sniper_disabled {
            let strategies_to_activate: Vec<_> = strategies
                .iter()
                .filter(|s| s.strategy_type == "graduation_snipe" && !s.is_active)
                .map(|s| s.id)
                .collect();

            for strategy_id in strategies_to_activate {
                if let Some(strategy) = strategies.iter().find(|s| s.id == strategy_id) {
                    tracing::info!(
                        strategy_id = %strategy_id,
                        strategy_name = %strategy.name,
                        strategy_type = %strategy.strategy_type,
                        "🔫 Activating curve strategy (default behavior)"
                    );

                    use crate::database::repositories::strategies::UpdateStrategyRecord;
                    if let Err(e) = strategy_repo
                        .update(
                            strategy_id,
                            UpdateStrategyRecord {
                                name: None,
                                venue_types: None,
                                execution_mode: None,
                                risk_params: None,
                                is_active: Some(true),
                            },
                        )
                        .await
                    {
                        tracing::warn!(error = %e, "Failed to activate curve strategy");
                    } else {
                        if let Err(e) = strategy_engine.toggle_strategy(strategy_id, true).await {
                            tracing::warn!("Failed to apply strategy update: {}", e);
                        }

                        // Persist to engrams so the active state survives restarts
                        let risk_params_json =
                            serde_json::to_value(&strategy.risk_params).unwrap_or_default();
                        if let Err(e) = engrams_client
                            .save_strategy_full(
                                &default_wallet,
                                &strategy.id.to_string(),
                                &strategy.name,
                                &strategy.strategy_type,
                                &strategy.venue_types,
                                &strategy.execution_mode,
                                &risk_params_json,
                                true, // is_active = true
                            )
                            .await
                        {
                            tracing::warn!(error = %e, "Failed to persist curve strategy activation to engrams");
                        }
                    }
                }
            }
        } else {
            let reason = if sniper_disabled_env {
                "ARBFARM_DISABLE_SNIPER=1"
            } else {
                "/tmp/arb-no-snipe"
            };
            tracing::warn!(
                "⚠️ Sniper DISABLED ({}) - skipping curve strategy activation",
                reason
            );
        }

        // Refresh strategies list after activation
        let strategies = strategy_engine.list_strategies().await;

        // Reconcile graduation_snipe to have auto_execute_enabled=true
        let strategies_to_enable: Vec<_> = strategies
            .iter()
            .filter(|s| {
                s.strategy_type == "graduation_snipe" && !s.risk_params.auto_execute_enabled
            })
            .map(|s| s.id)
            .collect();

        for strategy_id in strategies_to_enable {
            if let Some(strategy) = strategies.iter().find(|s| s.id == strategy_id) {
                tracing::info!(
                    strategy_id = %strategy_id,
                    strategy_name = %strategy.name,
                    strategy_type = %strategy.strategy_type,
                    "🔫 Enabling auto_execute for curve strategy (autonomous trading)"
                );

                let mut updated_risk_params = strategy.risk_params.clone();
                updated_risk_params.auto_execute_enabled = true;

                use crate::database::repositories::strategies::UpdateStrategyRecord;
                if let Err(e) = strategy_repo
                    .update(
                        strategy_id,
                        UpdateStrategyRecord {
                            name: None,
                            venue_types: None,
                            execution_mode: Some("autonomous".to_string()),
                            risk_params: Some(updated_risk_params.clone()),
                            is_active: None,
                        },
                    )
                    .await
                {
                    tracing::warn!(error = %e, "Failed to enable auto_execute for graduation_snipe");
                } else {
                    // Update in-memory
                    if let Err(e) = strategy_engine
                        .set_risk_params(strategy_id, updated_risk_params)
                        .await
                    {
                        tracing::warn!("Failed to apply strategy update: {}", e);
                    }
                    if let Err(e) = strategy_engine
                        .set_execution_mode(strategy_id, "autonomous".to_string())
                        .await
                    {
                        tracing::warn!("Failed to apply strategy update: {}", e);
                    }
                }
            }
        }

        // Refresh strategies list after snipe reconciliation
        let strategies = strategy_engine.list_strategies().await;

        // Reconcile execution_mode with auto_execute_enabled for all strategies
        // IMPORTANT: This runs AFTER engrams restoration to ensure persisted state is honored
        // This ensures that if auto_execute_enabled is true, execution_mode is "autonomous"
        let mut reconciliation_count = 0;
        for strategy in strategies.iter() {
            let expected_mode = if strategy.risk_params.auto_execute_enabled {
                "autonomous"
            } else {
                "agent_directed"
            };

            if strategy.execution_mode != expected_mode {
                reconciliation_count += 1;
                tracing::info!(
                    strategy_id = %strategy.id,
                    strategy_name = %strategy.name,
                    old_mode = %strategy.execution_mode,
                    new_mode = expected_mode,
                    auto_execute_enabled = strategy.risk_params.auto_execute_enabled,
                    "🔧 Reconciling execution_mode to match auto_execute_enabled (post-engrams)"
                );

                // Update in-memory
                if let Err(e) = strategy_engine
                    .set_execution_mode(strategy.id, expected_mode.to_string())
                    .await
                {
                    tracing::warn!(error = %e, "Failed to update strategy execution_mode in memory");
                }

                // Update in database
                use crate::database::repositories::strategies::UpdateStrategyRecord;
                if let Err(e) = strategy_repo
                    .update(
                        strategy.id,
                        UpdateStrategyRecord {
                            name: None,
                            venue_types: None,
                            execution_mode: Some(expected_mode.to_string()),
                            risk_params: None,
                            is_active: None,
                        },
                    )
                    .await
                {
                    tracing::warn!(error = %e, "Failed to persist execution_mode reconciliation to database");
                }
            }
        }

        if reconciliation_count == 0 {
            tracing::info!(
                "✅ All strategies already have correct execution_mode (no reconciliation needed)"
            );
        } else {
            tracing::info!(
                "✅ Reconciled {} strategies' execution_mode",
                reconciliation_count
            );
        }

        // Log final strategy states AFTER reconciliation
        tracing::info!("📋 Post-reconciliation strategy states:");
        let final_strategies = strategy_engine.list_strategies().await;
        for strategy in final_strategies.iter() {
            tracing::info!(
                strategy_id = %strategy.id,
                strategy_name = %strategy.name,
                execution_mode = %strategy.execution_mode,
                auto_execute_enabled = strategy.risk_params.auto_execute_enabled,
                "  └─ Final state"
            );
        }

        // Initialize Position Repository and Manager for tracking open positions and exit conditions
        let position_repo = Arc::new(PositionRepository::new(db_pool.clone()));
        let position_manager = Arc::new(crate::execution::PositionManager::with_repository(
            position_repo.clone(),
        ));

        // Initialize Consensus Repository for persisting LLM consensus decisions
        let consensus_repo = Arc::new(ConsensusRepository::new(db_pool.clone()));
        tracing::info!("✅ Consensus repository initialized (persisting to PostgreSQL)");

        let settings_repo = Arc::new(crate::database::SettingsRepository::new(db_pool.clone()));

        let kol_repo = Arc::new(KolRepository::new(db_pool.clone()));
        tracing::info!("✅ KOL repository initialized (PostgreSQL persistence)");

        // Restore any open positions from database
        match position_manager.load_positions_from_db().await {
            Ok(count) => {
                if count > 0 {
                    tracing::info!(
                        "✅ Position Manager initialized with {} restored positions",
                        count
                    );
                } else {
                    tracing::info!(
                        "✅ Position Manager initialized (no prior positions to restore)"
                    );
                }
            }
            Err(e) => {
                tracing::warn!(
                    "⚠️ Position Manager initialized but failed to load prior positions: {}",
                    e
                );
            }
        }

        // Restore any pending exit signals from database (critical for exit recovery)
        match position_manager.load_pending_exits_from_db().await {
            Ok(count) => {
                if count > 0 {
                    tracing::info!("✅ Restored {} pending exit signals for retry", count);
                }
            }
            Err(e) => {
                tracing::warn!("⚠️ Failed to load pending exit signals: {}", e);
            }
        }

        // Initialize Jito client for bundle submission (shared by executor and position monitor)
        let jito_client = Arc::new(JitoClient::new(config.jito_block_engine_url.clone(), None));
        tracing::info!(
            "✅ Jito client initialized (block engine: {})",
            config.jito_block_engine_url
        );

        // Create command channel for PositionMonitor → PositionExecutor communication
        let (command_tx, command_rx) = tokio::sync::mpsc::channel::<PositionCommand>(256);

        // Initialize Position Monitor for automated exit management (curve support added later)
        let position_monitor_base = PositionMonitor::new(
            position_manager.clone(),
            tx_builder.clone(),
            event_tx.clone(),
            command_tx.clone(),
            MonitorConfig::default(),
        );
        tracing::info!(
            "✅ Position Monitor base initialized (curve support added after curve_builder)"
        );

        // Initialize Approval Manager for execution controls
        let approval_manager = Arc::new(ApprovalManager::new(event_tx.clone()));

        // Sync global execution config from strategy states (persisted from previous session)
        let strategies_for_sync = strategy_engine.list_strategies().await;
        let any_autonomous = strategies_for_sync
            .iter()
            .filter(|s| s.is_active)
            .any(|s| s.execution_mode == "autonomous" || s.risk_params.auto_execute_enabled);
        approval_manager.sync_from_strategies(any_autonomous).await;
        tracing::info!("✅ Approval Manager initialized (execution controls + Hecate integration, synced from strategies: auto={})", any_autonomous);

        // Spawn HecateNotifier to forward approval events to Hecate for recommendations
        let hecate_event_rx = event_tx.subscribe();
        spawn_hecate_notifier(config.agents_service_url.clone(), hecate_event_rx);
        tracing::info!("✅ Hecate Notifier spawned (listening for approval events)");

        // Initialize Capital Manager for per-strategy allocation tracking
        let capital_manager = Arc::new(CapitalManager::new().with_db_pool(db_pool.clone()));

        // Load existing reservations from database (recovery after restart)
        match capital_manager.load_reservations_from_db().await {
            Ok(count) if count > 0 => {
                tracing::info!(
                    "✅ Capital Manager loaded {} existing reservations from DB",
                    count
                );
            }
            Ok(_) => {
                tracing::debug!("Capital Manager: no existing reservations to load");
            }
            Err(e) => {
                tracing::warn!(
                    "⚠️ Failed to load capital reservations from DB: {} - starting fresh",
                    e
                );
            }
        }

        // Register each strategy with capital manager
        for strategy in strategy_engine.list_strategies().await {
            let max_positions = strategy.risk_params.concurrent_positions.unwrap_or(1);
            capital_manager
                .register_strategy(
                    strategy.id,
                    strategy.risk_params.max_capital_allocation_percent,
                    max_positions,
                )
                .await;
        }
        tracing::info!(
            "✅ Capital Manager initialized (per-strategy allocation tracking + DB persistence)"
        );

        // Initialize on-chain fetcher and curve transaction builder for bonding curve operations
        let on_chain_fetcher = Arc::new(OnChainFetcher::new(&config.rpc_url));
        let curve_builder = Arc::new(
            CurveTransactionBuilder::new(&config.rpc_url)
                .with_on_chain_fetcher(on_chain_fetcher.clone()),
        );
        tracing::info!("✅ Curve execution engine initialized (on-chain state + tx builder)");

        // Add curve state checker to position monitor (for curve price lookups only)
        let position_monitor =
            Arc::new(position_monitor_base.with_curve_state_checker(curve_builder.clone()));
        tracing::info!("✅ Position Monitor initialized with curve support (monitoring only, execution via PositionExecutor)");

        // Initialize PositionExecutor for centralized sell execution
        let position_executor = Arc::new(
            PositionExecutor::new(
                command_rx,
                position_manager.clone(),
                tx_builder.clone(),
                jito_client.clone(),
                event_tx.clone(),
                dev_signer.clone(),
                ExecutorConfig::default(),
            )
            .with_curve_support(curve_builder.clone(), helius_sender.clone())
            .with_helius_client(helius_rpc_client.clone())
            .with_engrams(engrams_client.clone())
            .with_trade_repo(trade_repo.clone())
            .with_capital_manager(capital_manager.clone()),
        );
        tracing::info!("✅ Position Executor initialized (centralized sell execution: curve + DEX + engrams + capital)");

        // Initialize curve metrics collector, holder analyzer, and opportunity scorer
        let metrics_collector = Arc::new(CurveMetricsCollector::new(on_chain_fetcher.clone()));
        let holder_analyzer = Arc::new(HolderAnalyzer::new(helius_rpc_client.clone()));
        let curve_scorer = Arc::new(CurveOpportunityScorer::new(
            metrics_collector.clone(),
            holder_analyzer.clone(),
            on_chain_fetcher.clone(),
        ));
        tracing::info!("✅ Curve scoring engine initialized (metrics + holders + scorer)");

        // Register behavioral strategies with the scanner for the Strategy Factory pattern
        // These strategies generate signals independently and share capital equally
        // Note: Strategies are registered but execution requires enabling via UI or env vars
        use crate::agents::strategies::{GraduationSniperStrategy, RaydiumSnipeStrategy};
        use crate::execution::CopyTradeExecutor;

        let graduation_sniper_strategy = Arc::new(GraduationSniperStrategy::new());
        scanner
            .register_behavioral_strategy(graduation_sniper_strategy)
            .await;

        let raydium_snipe_strategy = Arc::new(RaydiumSnipeStrategy::new());
        scanner
            .register_behavioral_strategy(raydium_snipe_strategy.clone())
            .await;
        scanner
            .set_raydium_snipe_strategy(raydium_snipe_strategy)
            .await;
        scanner.set_pump_fun_venue(pump_fun_venue.clone()).await;
        tracing::info!("✅ Behavioral strategies registered (Graduation Sniper, Raydium Snipe)");

        // Rebalance capital manager to give all strategies equal allocation
        capital_manager.rebalance_equal().await;
        tracing::info!("✅ Capital allocation rebalanced: all strategies have equal share");

        // Create CopyTradeExecutor for KOL copy trading (OFF by default for observation mode)
        let copy_executor = Arc::new(CopyTradeExecutor::new(
            kol_repo.clone(),
            curve_builder.clone(),
            dev_signer.clone(),
            helius_sender.clone(),
            position_manager.clone(),
            engrams_client.clone(),
            event_tx.clone(),
            command_tx.clone(),
            default_wallet.clone(),
        ));

        // Enable copy trading via env var: ARBFARM_COPY_TRADING=1
        let copy_trading_enabled = std::env::var("ARBFARM_COPY_TRADING")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false);

        if copy_trading_enabled {
            let mut copy_config = copy_executor.get_config().await;
            copy_config.enabled = true;
            copy_executor.update_config(copy_config).await;
            tracing::info!(
                "✅ Copy Trade Executor initialized (ENABLED via ARBFARM_COPY_TRADING=1)"
            );
        } else {
            tracing::info!("✅ Copy Trade Executor initialized (DISABLED - observation mode)");
        }

        // Create Autonomous Executor (does NOT auto-start - respects user preference)
        let default_wallet_for_executor = config
            .wallet_address
            .clone()
            .unwrap_or_else(|| "default".to_string());
        let autonomous_executor = spawn_autonomous_executor(
            strategy_engine.clone(),
            curve_builder.clone(),
            dev_signer.clone(),
            helius_sender.clone(),
            position_manager.clone(),
            risk_config.clone(),
            engrams_client.clone(),
            Some(consensus_engine.clone()),
            consensus_config.clone(),
            event_tx.clone(),
            default_wallet_for_executor,
            Some(trade_repo.clone()),
            Some(helius_rpc_client.clone()),
        );

        // Connect CopyTradeExecutor to AutonomousExecutor for KOL copy trading
        autonomous_executor
            .set_copy_executor(copy_executor.clone())
            .await;
        tracing::info!("✅ Copy Trade Executor connected to Autonomous Executor");

        // Executor state: env var override > DB saved state > default (OFF)
        let executor_env_override = std::env::var("ARBFARM_ENABLE_EXECUTOR")
            .ok()
            .map(|v| v == "1" || v.to_lowercase() == "true");
        let executor_persisted = settings_repo
            .get_bool("execution_enabled")
            .await
            .unwrap_or(None);
        let executor_enabled = executor_env_override
            .or(executor_persisted)
            .unwrap_or(false);

        if executor_enabled {
            start_autonomous_executor(autonomous_executor.clone());
            approval_manager.toggle_execution(true).await;

            let strategies = strategy_engine.list_strategies().await;
            for strategy in strategies.iter().filter(|s| s.is_active) {
                let mut updated_params = strategy.risk_params.clone();
                updated_params.auto_execute_enabled = true;
                if let Err(e) = strategy_engine
                    .set_risk_params(strategy.id, updated_params.clone())
                    .await
                {
                    tracing::warn!("Failed to apply strategy update: {}", e);
                }
                if let Err(e) = strategy_engine
                    .set_execution_mode(strategy.id, "autonomous".to_string())
                    .await
                {
                    tracing::warn!("Failed to apply strategy update: {}", e);
                }

                use crate::database::repositories::strategies::UpdateStrategyRecord;
                let _ = strategy_repo
                    .update(
                        strategy.id,
                        UpdateStrategyRecord {
                            name: None,
                            venue_types: None,
                            execution_mode: Some("autonomous".to_string()),
                            risk_params: Some(updated_params),
                            is_active: None,
                        },
                    )
                    .await;
            }

            if executor_env_override.is_some() {
                tracing::info!("✅ Autonomous Executor started (ARBFARM_ENABLE_EXECUTOR env var) - {} strategies set to autonomous", strategies.iter().filter(|s| s.is_active).count());
            } else {
                tracing::info!("✅ Autonomous Executor started (restored from saved state) - {} strategies set to autonomous", strategies.iter().filter(|s| s.is_active).count());
            }
        } else {
            tracing::info!("✅ Autonomous Executor initialized but OFF by default");
            tracing::info!("   Enable: set ARBFARM_ENABLE_EXECUTOR=1 or use UI toggle");
        }

        // Initialize Real-time Position Monitor for websocket-based price updates
        let realtime_monitor = Arc::new(RealtimePositionMonitor::new(
            laserstream_client.clone(),
            position_manager.clone(),
            position_monitor.clone(),
            event_tx.clone(),
        ));
        tracing::info!("✅ Real-time Position Monitor initialized (websocket price updates)");

        // Initialize Graduation Sniper for automated sell on graduation (execution-only)
        let graduation_sniper = Arc::new(
            GraduationSniper::new(
                curve_builder.clone(),
                jito_client.clone(),
                on_chain_fetcher.clone(),
                event_tx.clone(),
                default_wallet.clone(),
            )
            .with_jupiter_api_url(config.jupiter_api_url.clone())
            .with_strategy_engine(strategy_engine.clone())
            .with_transaction_support(dev_signer.clone(), helius_sender.clone())
            .with_position_manager(position_manager.clone())
            .with_risk_config(risk_config.clone()),
        );
        tracing::info!("✅ Graduation Sniper initialized (strategy engine + Jupiter + PositionManager + RiskConfig for exit monitoring)");

        Ok(Self {
            config,
            db_pool,
            event_tx,
            event_bus,
            scanner,
            executor,
            simulator,
            tx_builder,
            edge_repo,
            strategy_repo,
            trade_repo,
            jupiter_venue,
            pump_fun_venue,
            moonshot_venue,
            turnkey_signer,
            risk_config,
            helius_webhook_client,
            helius_rpc_client,
            helius_sender,
            helius_das,
            priority_fee_monitor,
            kol_tracker,
            strategy_engine,
            consensus_engine,
            consensus_config,
            engrams_client,
            position_repo,
            consensus_repo,
            settings_repo,
            kol_repo,
            laserstream_client,
            kol_discovery,
            dev_signer,
            position_manager,
            position_monitor,
            jito_client,
            approval_manager,
            capital_manager,
            curve_builder,
            on_chain_fetcher,
            metrics_collector,
            holder_analyzer,
            curve_scorer,
            autonomous_executor,
            position_executor,
            realtime_monitor,
            graduation_sniper,
            wallet_max_position_sol: Arc::new(RwLock::new(10.0)),
            consensus_scheduler_paused: Arc::new(AtomicBool::new(true)), // ALWAYS start paused - manual trigger only
            consensus_last_queried: Arc::new(RwLock::new(None)),
        })
    }

    pub fn start_position_monitor(&self) {
        let monitor = self.position_monitor.clone();
        tokio::spawn(async move {
            monitor.start_monitoring().await;
        });
        tracing::info!("🔭 Position monitor background task started");

        let executor = self.position_executor.clone();
        tokio::spawn(async move {
            executor.run().await;
        });
        tracing::info!("⚡ Position executor background task started");
    }

    pub fn start_realtime_monitor(&self) {
        let realtime = self.realtime_monitor.clone();

        tokio::spawn(async move {
            if let Err(e) = realtime.start().await {
                tracing::error!("Failed to start real-time monitor: {}", e);
            }
        });

        tracing::info!("📡 Real-time position monitor background task started");
    }

    pub fn start_daily_metrics_scheduler(&self, wallet_address: String) {
        use crate::agents::start_daily_metrics_scheduler;

        let position_repo = self.position_repo.clone();
        let engrams_client = self.engrams_client.clone();

        tokio::spawn(async move {
            start_daily_metrics_scheduler(position_repo, engrams_client, wallet_address).await;
        });

        tracing::info!("📊 Daily metrics scheduler started (runs at 00:05 UTC)");
    }

    pub fn subscribe_events(&self) -> broadcast::Receiver<ArbEvent> {
        self.event_tx.subscribe()
    }
}
