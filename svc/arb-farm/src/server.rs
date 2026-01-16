use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

use crate::agents::ScannerAgent;
use crate::config::Config;
use crate::consensus::ConsensusEngine;
use crate::database::{EdgeRepository, StrategyRepository, TradeRepository};
use crate::events::ArbEvent;
use crate::execution::{ExecutorAgent, TransactionSimulator, TransactionBuilder};
use crate::execution::risk::RiskConfig;
use crate::models::KOLTracker;
use crate::venues::curves::{MoonshotVenue, PumpFunVenue};
use crate::venues::dex::{JupiterVenue, RaydiumVenue};
use crate::wallet::turnkey::{TurnkeySigner, TurnkeyConfig};
use crate::webhooks::helius::HeliusWebhookClient;

pub const EVENT_CHANNEL_CAPACITY: usize = 1024;
pub const DEFAULT_SCAN_INTERVAL_MS: u64 = 5000;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db_pool: PgPool,
    pub event_tx: broadcast::Sender<ArbEvent>,
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
    pub helius_client: Arc<HeliusWebhookClient>,
    pub kol_tracker: Arc<KOLTracker>,
    pub consensus_engine: Arc<ConsensusEngine>,
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
        tracing::info!("✅ Turnkey wallet signer initialized");

        // Initialize risk config with dev_testing profile
        let risk_config = Arc::new(RwLock::new(RiskConfig::dev_testing()));
        tracing::info!("✅ Risk config initialized (dev_testing profile: {} SOL max position)",
            config.default_max_position_sol);

        // Initialize Helius webhook client
        let helius_client = Arc::new(HeliusWebhookClient::new(
            config.helius_api_url.clone(),
            config.helius_api_key.clone(),
        ));
        if helius_client.is_configured() {
            tracing::info!("✅ Helius webhook client initialized");
        } else {
            tracing::warn!("⚠️ Helius API key not configured - webhooks disabled");
        }

        // Initialize KOL tracker
        let kol_tracker = Arc::new(KOLTracker::new());
        tracing::info!("✅ KOL tracker initialized");

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

        Ok(Self {
            config,
            db_pool,
            event_tx,
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
            helius_client,
            kol_tracker,
            consensus_engine,
        })
    }

    pub fn subscribe_events(&self) -> broadcast::Receiver<ArbEvent> {
        self.event_tx.subscribe()
    }
}
