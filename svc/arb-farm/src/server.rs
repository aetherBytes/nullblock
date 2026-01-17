use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

use crate::agents::{
    AutonomousExecutor, CurveMetricsCollector, CurveOpportunityScorer, EngramHarvester,
    KolDiscoveryAgent, OverseerConfig, ResilienceOverseer, ScannerAgent, StrategyEngine,
    spawn_autonomous_executor, spawn_hecate_notifier,
};
use crate::handlers::engram::init_harvester;
use crate::handlers::swarm::{init_overseer, init_circuit_breakers};
use crate::resilience::CircuitBreakerRegistry;
use crate::config::Config;
use crate::consensus::ConsensusEngine;
use crate::database::{EdgeRepository, StrategyRepository, TradeRepository};
use crate::engrams::EngramsClient;
use crate::events::{ArbEvent, EventBus};
use crate::execution::{ApprovalManager, CapitalManager, CurveTransactionBuilder, ExecutorAgent, TransactionSimulator, TransactionBuilder, PositionMonitor, MonitorConfig, JitoClient};
use crate::venues::curves::{HolderAnalyzer, OnChainFetcher};
use crate::execution::risk::RiskConfig;
use crate::helius::{HeliusClient, DasClient, HeliusSender, LaserStreamClient, priority_fee::PriorityFeeMonitor};
use crate::models::KOLTracker;
use crate::venues::curves::{MoonshotVenue, PumpFunVenue};
use crate::venues::dex::{JupiterVenue, RaydiumVenue};
use crate::wallet::turnkey::{TurnkeySigner, TurnkeyConfig};
use crate::wallet::DevWalletSigner;
use crate::webhooks::helius::HeliusWebhookClient;

pub const EVENT_CHANNEL_CAPACITY: usize = 1024;
pub const DEFAULT_SCAN_INTERVAL_MS: u64 = 5000;

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
    pub raydium_venue: Arc<RaydiumVenue>,
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
    pub engrams_client: Arc<EngramsClient>,
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
}

impl AppState {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        let db_pool = PgPoolOptions::new()
            .max_connections(20)
            .connect(&config.database_url)
            .await?;

        tracing::info!("✅ Database connection pool created");
        tracing::info!("✅ Database ready (migrations handled externally)");

        let (event_tx, _) = broadcast::channel(EVENT_CHANNEL_CAPACITY);
        let event_bus = Arc::new(EventBus::new(event_tx.clone(), db_pool.clone()));
        tracing::info!("✅ Event bus initialized (capacity: {})", EVENT_CHANNEL_CAPACITY);

        let scanner = Arc::new(ScannerAgent::new(event_tx.clone(), DEFAULT_SCAN_INTERVAL_MS));

        // Initialize venues (shared between scanner and direct access)
        let jupiter_venue = Arc::new(JupiterVenue::new(config.jupiter_api_url.clone()));
        let raydium_venue = Arc::new(RaydiumVenue::new(config.raydium_api_url.clone()));
        let pump_fun_venue = Arc::new(PumpFunVenue::new(config.pump_fun_api_url.clone()));
        let moonshot_venue = Arc::new(MoonshotVenue::new(config.moonshot_api_url.clone()));

        // Add venues to scanner (cloning the Arc)
        scanner.add_venue(Box::new(JupiterVenue::new(config.jupiter_api_url.clone()))).await;
        scanner.add_venue(Box::new(RaydiumVenue::new(config.raydium_api_url.clone()))).await;
        scanner.add_venue(Box::new(PumpFunVenue::new(config.pump_fun_api_url.clone()))).await;
        scanner.add_venue(Box::new(MoonshotVenue::new(config.moonshot_api_url.clone()))).await;

        tracing::info!("✅ Scanner agent initialized with 4 venues (Jupiter, Raydium, pump.fun, moonshot)");

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
        ));
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
        let turnkey_signer = Arc::new(TurnkeySigner::new(turnkey_config));

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
                let dev_wallet_id = format!("dev_wallet_{}", wallet_address.chars().take(8).collect::<String>());
                if let Err(e) = turnkey_signer.set_wallet(wallet_address.clone(), dev_wallet_id).await {
                    tracing::warn!("⚠️ Failed to auto-configure turnkey status: {}", e);
                }
            }
            tracing::info!("✅ Wallet signing mode: DEV (private key from env, auto-connected)");
        } else if config.turnkey_api_public_key.is_some() && config.turnkey_api_private_key.is_some() {
            tracing::info!("✅ Wallet signing mode: PRODUCTION (Turnkey delegation)");
        } else {
            tracing::warn!("⚠️ No wallet signing configured - transactions will fail");
        }

        // Initialize risk config with dev_testing profile
        let risk_config = Arc::new(RwLock::new(RiskConfig::dev_testing()));
        tracing::info!("✅ Risk config initialized (dev_testing profile: {} SOL max position)",
            config.default_max_position_sol);

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
        let helius_rpc_client = Arc::new(
            HeliusClient::new(&config).with_event_bus(event_bus.clone())
        );
        tracing::info!("✅ Helius RPC client initialized (url: {})", config.helius_api_url);

        // Initialize Helius Sender for fast TX submission
        let helius_sender = Arc::new(HeliusSender::new(
            helius_rpc_client.clone(),
            event_bus.clone(),
        ));
        tracing::info!("✅ Helius Sender initialized (url: {})", config.helius_sender_url);

        // Initialize DAS (Digital Asset Standard) client
        let helius_das = Arc::new(DasClient::new(
            helius_rpc_client.clone(),
            event_bus.clone(),
        ));
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
        use crate::models::{Strategy, RiskParams};
        use crate::database::repositories::strategies::CreateStrategyRecord;

        let default_wallet = config.wallet_address.clone().unwrap_or_else(|| "default".to_string());

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
                        risk_params: serde_json::from_value(record.risk_params.clone()).unwrap_or_default(),
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
            match strategy_repo.create(CreateStrategyRecord {
                wallet_address: wallet_address.to_string(),
                name: name.to_string(),
                strategy_type: strategy_type.to_string(),
                venue_types,
                execution_mode: execution_mode.to_string(),
                risk_params,
            }).await {
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

        // Bonding Curve Strategy - Default to manual mode (safe by default)
        // Users can toggle to autonomous in the UI
        // Risk appetite configured separately via UI
        if let Some(curve_strategy) = get_or_create_strategy(
            &strategy_repo,
            "Curve Graduation",
            &default_wallet,
            "curve_arb",
            vec!["bondingcurve".to_string(), "BondingCurve".to_string()],
            "agent_directed",  // Manual approval required by default
            RiskParams {
                // Moderate risk profile by default
                max_position_sol: config.default_max_position_sol.min(0.01), // Cap at 0.01 SOL for testing
                daily_loss_limit_sol: 1.0,              // Max 1 SOL loss per day
                min_profit_bps: 50,                     // Require 0.5% min profit
                max_slippage_bps: 150,                  // 1.5% max slippage
                max_risk_score: 60,                     // Moderate risk tolerance
                require_simulation: true,               // Always simulate first
                auto_execute_atomic: false,             // No auto-execute
                auto_execute_enabled: false,            // Disabled by default
                require_confirmation: true,             // Always require confirmation
                staleness_threshold_hours: 24,
                stop_loss_percent: Some(10.0),          // 10% stop loss
                take_profit_percent: Some(30.0),        // 30% take profit
                trailing_stop_percent: None,
                time_limit_minutes: Some(60),           // 1 hour max hold
                base_currency: "sol".to_string(),
                max_capital_allocation_percent: 15.0,   // Conservative 15% allocation
                concurrent_positions: Some(1),          // One position at a time
            },
        ).await {
            strategy_engine.add_strategy(curve_strategy).await;
        }

        // DEX Arbitrage Strategy
        if let Some(dex_strategy) = get_or_create_strategy(
            &strategy_repo,
            "DEX Arbitrage",
            &default_wallet,
            "dex_arb",
            vec!["dexamm".to_string(), "DexAmm".to_string()],
            "manual",
            RiskParams::default(),
        ).await {
            strategy_engine.add_strategy(dex_strategy).await;
        }

        // KOL Copy Trade Strategy - use preset for copy trading
        if let Some(kol_strategy) = get_or_create_strategy(
            &strategy_repo,
            "KOL Copy Trading",
            &default_wallet,
            "copy_trade",
            vec!["bondingcurve".to_string(), "dexamm".to_string()],
            "agent_directed",
            RiskParams::for_copy_trade(),
        ).await {
            strategy_engine.add_strategy(kol_strategy).await;
        }

        let strategy_count = strategy_engine.list_strategies().await.len();
        tracing::info!("✅ Strategy engine initialized with {} default strategies (persisted to DB)", strategy_count);

        // Connect scanner to strategy engine for automatic signal processing
        scanner.set_strategy_engine(strategy_engine.clone()).await;
        tracing::info!("✅ Scanner connected to strategy engine (auto-processing enabled)");

        // Initialize LLM consensus engine
        let consensus_engine = if let Some(ref api_key) = config.openrouter_api_key {
            Arc::new(ConsensusEngine::new(api_key.clone()))
        } else {
            Arc::new(ConsensusEngine::new(""))
        };
        if config.openrouter_api_key.is_some() {
            tracing::info!("✅ Consensus engine initialized (OpenRouter multi-LLM)");
        } else {
            tracing::warn!("⚠️ OpenRouter API key not configured - consensus disabled");
        }

        // Initialize Engrams client for persistent memory
        let engrams_client = Arc::new(EngramsClient::new(config.engrams_url.clone()));
        if engrams_client.is_configured() {
            tracing::info!("✅ Engrams client initialized (url: {})", config.engrams_url);
        } else {
            tracing::warn!("⚠️ Engrams service URL not configured - persistence disabled");
        }

        // Initialize EngramHarvester for local pattern learning
        let engram_harvester = EngramHarvester::new(event_tx.clone());
        init_harvester(engram_harvester);
        tracing::info!("✅ Engram Harvester initialized (local pattern store)");

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
            tracing::info!("✅ LaserStream client initialized (endpoint: {})", config.helius_laserstream_url);
        } else {
            tracing::warn!("⚠️ LaserStream not configured - real-time streaming disabled");
        }

        // Initialize KOL discovery agent
        let kol_discovery = Arc::new(
            KolDiscoveryAgent::new()
                .with_engrams_client(engrams_client.clone())
                .with_owner_wallet(default_wallet.clone())
        );
        tracing::info!("✅ KOL Discovery Agent initialized");

        // Restore workflow state from engrams (persisted data from previous sessions)
        if engrams_client.is_configured() {
            let workflow_state = engrams_client.restore_workflow_state(&default_wallet).await;

            // Restore discovered KOLs
            if !workflow_state.discovered_kols.is_empty() {
                kol_discovery.restore_kols(workflow_state.discovered_kols).await;
            }

            // Restore strategies from engrams (authoritative source for state)
            for engram_strategy in &workflow_state.strategies {
                // Check if strategy exists in engine
                if let Ok(strategy_id) = uuid::Uuid::parse_str(&engram_strategy.strategy_id) {
                    if let Some(existing) = strategy_engine.get_strategy(strategy_id).await {
                        // Update existing strategy's is_active from engram
                        if existing.is_active != engram_strategy.is_active {
                            tracing::info!(
                                "Restoring strategy '{}' active state from engrams: {}",
                                existing.name,
                                engram_strategy.is_active
                            );
                            let _ = strategy_engine.toggle_strategy(strategy_id, engram_strategy.is_active).await;
                        }

                        // Restore risk_params from engrams (authoritative source)
                        if let Ok(engram_risk_params) = serde_json::from_value::<RiskParams>(engram_strategy.risk_params.clone()) {
                            // Check if risk params differ (compare key fields)
                            if existing.risk_params.max_position_sol != engram_risk_params.max_position_sol
                                || existing.risk_params.max_risk_score != engram_risk_params.max_risk_score
                                || existing.risk_params.auto_execute_enabled != engram_risk_params.auto_execute_enabled
                            {
                                tracing::info!(
                                    "Restoring strategy '{}' risk params from engrams (max_position: {} SOL, max_risk: {}, auto_execute: {})",
                                    existing.name,
                                    engram_risk_params.max_position_sol,
                                    engram_risk_params.max_risk_score,
                                    engram_risk_params.auto_execute_enabled
                                );
                                let _ = strategy_engine.set_risk_params(strategy_id, engram_risk_params).await;
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
                            risk_params: serde_json::from_value(engram_strategy.risk_params.clone()).unwrap_or_default(),
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
        }

        // Initialize Position Manager for tracking open positions and exit conditions
        let position_manager = Arc::new(crate::execution::PositionManager::new());
        tracing::info!("✅ Position Manager initialized (exit tracking: SL/TP/trailing/time)");

        // Initialize Jito client for bundle submission (shared by executor and position monitor)
        let jito_client = Arc::new(JitoClient::new(config.jito_block_engine_url.clone(), None));
        tracing::info!("✅ Jito client initialized (block engine: {})", config.jito_block_engine_url);

        // Initialize Position Monitor for automated exit management
        let position_monitor = Arc::new(PositionMonitor::new(
            position_manager.clone(),
            tx_builder.clone(),
            jito_client.clone(),
            event_tx.clone(),
            MonitorConfig::default(),
        ));
        tracing::info!("✅ Position Monitor initialized (auto-exit: SL/TP/trailing/time-based)");

        // Initialize Approval Manager for execution controls
        let approval_manager = Arc::new(ApprovalManager::new(event_tx.clone()));
        tracing::info!("✅ Approval Manager initialized (execution controls + Hecate integration)");

        // Spawn HecateNotifier to forward approval events to Hecate for recommendations
        let hecate_event_rx = event_tx.subscribe();
        spawn_hecate_notifier(config.agents_service_url.clone(), hecate_event_rx);
        tracing::info!("✅ Hecate Notifier spawned (listening for approval events)");

        // Initialize Capital Manager for per-strategy allocation tracking
        let capital_manager = Arc::new(CapitalManager::new());

        // Register each strategy with capital manager
        for strategy in strategy_engine.list_strategies().await {
            let max_positions = strategy.risk_params.concurrent_positions.unwrap_or(1);
            capital_manager.register_strategy(
                strategy.id,
                strategy.risk_params.max_capital_allocation_percent,
                max_positions,
            ).await;
        }
        tracing::info!("✅ Capital Manager initialized (per-strategy allocation tracking)");

        // Initialize on-chain fetcher and curve transaction builder for bonding curve operations
        let on_chain_fetcher = Arc::new(OnChainFetcher::new(&config.rpc_url));
        let curve_builder = Arc::new(
            CurveTransactionBuilder::new(&config.rpc_url)
                .with_on_chain_fetcher(on_chain_fetcher.clone())
        );
        tracing::info!("✅ Curve execution engine initialized (on-chain state + tx builder)");

        // Initialize curve metrics collector, holder analyzer, and opportunity scorer
        let metrics_collector = Arc::new(CurveMetricsCollector::new(on_chain_fetcher.clone()));
        let holder_analyzer = Arc::new(HolderAnalyzer::new(helius_rpc_client.clone()));
        let curve_scorer = Arc::new(CurveOpportunityScorer::new(
            metrics_collector.clone(),
            holder_analyzer.clone(),
            on_chain_fetcher.clone(),
        ));
        tracing::info!("✅ Curve scoring engine initialized (metrics + holders + scorer)");

        // Spawn Autonomous Executor for auto-execution of edges in autonomous mode
        let default_wallet_for_executor = config.wallet_address.clone().unwrap_or_else(|| "default".to_string());
        let autonomous_executor = spawn_autonomous_executor(
            strategy_engine.clone(),
            curve_builder.clone(),
            dev_signer.clone(),
            helius_sender.clone(),
            position_manager.clone(),
            event_tx.clone(),
            default_wallet_for_executor,
        );
        tracing::info!("✅ Autonomous Executor spawned (auto-execution for autonomous strategies)");

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
            raydium_venue,
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
            engrams_client,
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
        })
    }

    pub fn start_position_monitor(&self) {
        let monitor = self.position_monitor.clone();
        let signer = self.dev_signer.clone();

        tokio::spawn(async move {
            monitor.start_monitoring(signer).await;
        });

        tracing::info!("🔭 Position monitor background task started");
    }

    pub fn subscribe_events(&self) -> broadcast::Receiver<ArbEvent> {
        self.event_tx.subscribe()
    }
}
