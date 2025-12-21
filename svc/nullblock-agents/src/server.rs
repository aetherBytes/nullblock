use crate::{
    agents::{HecateAgent, siren_marketing::MarketingAgent},
    config::{ApiKeys, Config},
    database::{Database, repositories::AgentRepository},
    kafka::{KafkaConfig, KafkaProducer},
    error::AppResult,
    services::ErebusClient,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub hecate_agent: Arc<RwLock<HecateAgent>>,
    pub marketing_agent: Arc<RwLock<MarketingAgent>>,
    pub database: Option<Arc<Database>>,
    pub kafka_producer: Option<Arc<KafkaProducer>>,
    pub erebus_client: Arc<ErebusClient>,
}

impl AppState {
    pub async fn new(config: Config) -> AppResult<Self> {
        // Initialize Erebus client for API key resolution
        let erebus_base_url = std::env::var("EREBUS_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:3000".to_string());
        info!("🔗 Connecting to Erebus at {}", erebus_base_url);
        let erebus_client = Arc::new(ErebusClient::new(&erebus_base_url));

        // Initialize database connection if URL is provided
        let database = if let Ok(database_url) = std::env::var("DATABASE_URL") {
            let db = Database::new(&database_url).await?;

            // Run migrations
            if let Err(e) = db.run_migrations().await {
                warn!("Failed to run database migrations: {}", e);
            } else {
                info!("✅ Database migrations completed successfully");
            }

            Some(Arc::new(db))
        } else {
            warn!("⚠️ DATABASE_URL not set, running without persistent storage");
            None
        };

        // Initialize Hecate agent with default model from config
        let default_model = config.llm.default_model.clone();
        let mut hecate_agent = HecateAgent::new(None);
        hecate_agent.preferred_model = default_model.clone();

        // Initialize Marketing agent with default model from config
        let mut marketing_agent = MarketingAgent::new(None);
        marketing_agent.preferred_model = default_model;

        // Fetch API keys from Erebus (agent keys) with fallback to env vars
        let api_keys = Self::resolve_api_keys(&erebus_client, &config).await;

        // Start the agents
        hecate_agent.start(&api_keys).await?;
        marketing_agent.start(&api_keys).await?;

        // Register agents in database if available
        if let Some(ref db) = database {
            let agent_repo = AgentRepository::new(db.pool().clone());
            if let Err(e) = hecate_agent.register_agent(&agent_repo).await {
                tracing::warn!("⚠️ Failed to register Hecate agent: {}", e);
            }
            if let Err(e) = marketing_agent.register_agent(&agent_repo).await {
                tracing::warn!("⚠️ Failed to register Marketing agent: {}", e);
            }
        }

        // Initialize Kafka producer
        let kafka_producer = if std::env::var("KAFKA_BOOTSTRAP_SERVERS").is_ok() {
            let kafka_config = KafkaConfig::default();
            match KafkaProducer::new(&kafka_config) {
                Ok(producer) => {
                    tracing::info!("✅ Kafka producer initialized");
                    Some(Arc::new(producer))
                }
                Err(e) => {
                    tracing::warn!("⚠️ Failed to initialize Kafka producer: {}", e);
                    None
                }
            }
        } else {
            tracing::warn!("⚠️ KAFKA_BOOTSTRAP_SERVERS not set, running without event streaming");
            None
        };

        Ok(Self {
            config,
            hecate_agent: Arc::new(RwLock::new(hecate_agent)),
            marketing_agent: Arc::new(RwLock::new(marketing_agent)),
            database,
            kafka_producer,
            erebus_client,
        })
    }

    async fn resolve_api_keys(erebus_client: &ErebusClient, config: &Config) -> ApiKeys {
        info!("🔑 Resolving API keys from Erebus...");

        // Try to fetch agent keys from Erebus (OpenRouter only for now)
        let openrouter_key = match erebus_client.get_agent_api_key("hecate", "openrouter").await {
            Ok(Some(key)) => {
                info!("✅ Retrieved HECATE's OpenRouter API key from Erebus");
                Some(key)
            }
            Ok(None) => {
                warn!("⚠️ No OpenRouter key found for HECATE in Erebus, checking env vars");
                None
            }
            Err(e) => {
                error!("❌ Failed to fetch API key from Erebus: {}", e);
                warn!("⚠️ Falling back to environment variables for API keys");
                None
            }
        };

        // Fall back to env vars if Erebus didn't have the key
        let env_keys = config.get_api_keys();

        ApiKeys {
            openai: env_keys.openai,
            anthropic: env_keys.anthropic,
            groq: env_keys.groq,
            huggingface: env_keys.huggingface,
            openrouter: openrouter_key.or(env_keys.openrouter),
        }
    }
}