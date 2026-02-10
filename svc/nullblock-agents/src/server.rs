use crate::{
    agents::{siren_marketing::MarketingAgent, HecateAgent, MorosAgent},
    config::{ApiKeys, Config},
    database::{repositories::AgentRepository, Database},
    engrams::EngramsClient,
    error::AppResult,
    kafka::{KafkaConfig, KafkaProducer},
    services::ErebusClient,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

#[derive(Clone)]
pub struct AppState {
    #[allow(dead_code)]
    pub config: Config,
    pub api_keys: ApiKeys,
    pub hecate_agent: Arc<RwLock<HecateAgent>>,
    pub moros_agent: Arc<RwLock<MorosAgent>>,
    pub marketing_agent: Arc<RwLock<MarketingAgent>>,
    pub database: Option<Arc<Database>>,
    pub kafka_producer: Option<Arc<KafkaProducer>>,
    pub erebus_client: Arc<ErebusClient>,
    pub engrams_client: Arc<EngramsClient>,
    pub agent_openrouter_keys: HashMap<String, String>,
    pub agent_model_preferences: Arc<RwLock<HashMap<String, String>>>,
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

        let engrams_base_url = std::env::var("ENGRAMS_BASE_URL")
            .or_else(|_| std::env::var("ENGRAMS_SERVICE_URL"))
            .unwrap_or_else(|_| "http://localhost:9004".to_string());
        info!("🧠 Connecting to Engrams at {}", engrams_base_url);
        let engrams_client = Arc::new(EngramsClient::new(engrams_base_url));

        // Initialize Hecate agent with default model from config
        let default_model = config.llm.default_model.clone();
        let mut hecate_agent = HecateAgent::new(None);
        hecate_agent.preferred_model = default_model.clone();
        hecate_agent.set_engrams_client(engrams_client.clone());

        // Initialize Moros agent with default model from config
        let mut moros_agent = MorosAgent::new(None);
        moros_agent.preferred_model = default_model.clone();
        moros_agent.set_engrams_client(engrams_client.clone());

        // Initialize Marketing agent with default model from config
        let mut marketing_agent = MarketingAgent::new(None);
        marketing_agent.preferred_model = default_model;

        // Fetch API keys from Erebus (agent keys) with fallback to env vars
        let api_keys = Self::resolve_api_keys(&erebus_client, &config).await;

        // Start the agents
        hecate_agent.start(&api_keys).await?;
        moros_agent.start(&api_keys).await?;
        marketing_agent.start(&api_keys).await?;

        // Register agents in database if available
        if let Some(ref db) = database {
            let agent_repo = AgentRepository::new(db.pool().clone());
            if let Err(e) = hecate_agent.register_agent(&agent_repo).await {
                tracing::warn!("⚠️ Failed to register Hecate agent: {}", e);
            }
            if let Err(e) = moros_agent.register_agent(&agent_repo).await {
                tracing::warn!("⚠️ Failed to register Moros agent: {}", e);
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

        let agent_openrouter_keys = Self::resolve_agent_openrouter_keys(&erebus_client).await;

        // Seed First Encounter engrams in background (non-blocking)
        let seed_client = engrams_client.clone();
        tokio::spawn(async move {
            seed_first_encounter_engram(&seed_client).await;
        });

        let seed_moros_client = engrams_client.clone();
        tokio::spawn(async move {
            seed_moros_first_encounter_engram(&seed_moros_client).await;
        });

        Ok(Self {
            config,
            api_keys,
            hecate_agent: Arc::new(RwLock::new(hecate_agent)),
            moros_agent: Arc::new(RwLock::new(moros_agent)),
            marketing_agent: Arc::new(RwLock::new(marketing_agent)),
            database,
            kafka_producer,
            erebus_client,
            engrams_client,
            agent_openrouter_keys,
            agent_model_preferences: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    async fn resolve_api_keys(erebus_client: &ErebusClient, config: &Config) -> ApiKeys {
        info!("🔑 Resolving API keys from Erebus...");

        const MAX_RETRIES: u32 = 5;
        const INITIAL_DELAY_MS: u64 = 1000;

        let mut openrouter_key = None;

        // Try to fetch agent keys from Erebus with retry logic
        for attempt in 1..=MAX_RETRIES {
            match erebus_client
                .get_agent_api_key("hecate", "openrouter")
                .await
            {
                Ok(Some(key)) => {
                    info!(
                        "✅ Retrieved HECATE's OpenRouter API key from Erebus (attempt {})",
                        attempt
                    );
                    openrouter_key = Some(key);
                    break;
                }
                Ok(None) => {
                    warn!("⚠️ No OpenRouter key found for HECATE in Erebus");
                    break; // Key doesn't exist, no point retrying
                }
                Err(e) => {
                    if attempt < MAX_RETRIES {
                        let delay = INITIAL_DELAY_MS * 2_u64.pow(attempt - 1);
                        warn!(
                            "⚠️ Failed to fetch API key (attempt {}/{}): {}. Retrying in {}ms...",
                            attempt, MAX_RETRIES, e, delay
                        );
                        tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                    } else {
                        error!(
                            "❌ Failed to fetch API key after {} attempts: {}",
                            MAX_RETRIES, e
                        );
                        warn!("⚠️ Falling back to environment variables for API keys");
                    }
                }
            }
        }

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

    async fn resolve_agent_openrouter_keys(erebus_client: &ErebusClient) -> HashMap<String, String> {
        let mut keys = HashMap::new();
        let agent_names = ["clawros"];

        for agent_name in &agent_names {
            const MAX_RETRIES: u32 = 5;
            const INITIAL_DELAY_MS: u64 = 1000;

            for attempt in 1..=MAX_RETRIES {
                match erebus_client
                    .get_agent_api_key(agent_name, "openrouter")
                    .await
                {
                    Ok(Some(key)) => {
                        info!(
                            "✅ Retrieved {} OpenRouter key from Erebus (attempt {})",
                            agent_name, attempt
                        );
                        keys.insert(agent_name.to_string(), key);
                        break;
                    }
                    Ok(None) => {
                        warn!("⚠️ No OpenRouter key found for {} in Erebus", agent_name);
                        break;
                    }
                    Err(e) => {
                        if attempt < MAX_RETRIES {
                            let delay = INITIAL_DELAY_MS * 2_u64.pow(attempt - 1);
                            warn!(
                                "⚠️ Failed to fetch {} key (attempt {}/{}): {}. Retrying in {}ms...",
                                agent_name, attempt, MAX_RETRIES, e, delay
                            );
                            tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                        } else {
                            error!(
                                "❌ Failed to fetch {} key after {} attempts: {}",
                                agent_name, MAX_RETRIES, e
                            );
                        }
                    }
                }
            }
        }

        if !keys.is_empty() {
            info!("🔑 Loaded {} agent OpenRouter key(s): {:?}", keys.len(), keys.keys().collect::<Vec<_>>());
        }

        keys
    }
}

async fn seed_first_encounter_engram(client: &EngramsClient) {
    use crate::config::dev_wallet::DEV_WALLETS;
    use crate::engrams::CreateEngramRequest;

    let wallet = DEV_WALLETS[0];
    let key = "hecate.persona.architect";

    match client.get_engram_by_wallet_key(wallet, key).await {
        Ok(Some(_)) => {
            info!("🧠 First Encounter engram already exists, skipping seed");
            return;
        }
        Ok(None) => {}
        Err(e) => {
            warn!(
                "⚠️ Could not check First Encounter engram (engrams service may be down): {}",
                e
            );
            return;
        }
    }

    let content = include_str!("agents/hecate_sage_fist_encounter.md");

    let request = CreateEngramRequest {
        wallet_address: wallet.to_string(),
        engram_type: "persona".to_string(),
        key: key.to_string(),
        content: content.to_string(),
        metadata: Some(serde_json::json!({
            "type": "architect_persona",
            "source": "first_encounter",
            "relationship": "architect"
        })),
        tags: Some(vec![
            "persona".to_string(),
            "architect".to_string(),
            "first_encounter".to_string(),
            "hecate".to_string(),
        ]),
        is_public: Some(false),
    };

    match client.create_engram(request).await {
        Ok(engram) => {
            info!(
                "🧠 First Encounter engram seeded: {} ({})",
                engram.key, engram.id
            );
        }
        Err(e) => {
            warn!("⚠️ Failed to seed First Encounter engram: {}", e);
        }
    }
}

async fn seed_moros_first_encounter_engram(client: &EngramsClient) {
    use crate::config::dev_wallet::DEV_WALLETS;
    use crate::engrams::CreateEngramRequest;

    let wallet = DEV_WALLETS[0];
    let key = "moros.persona.architect";

    match client.get_engram_by_wallet_key(wallet, key).await {
        Ok(Some(_)) => {
            info!("🌑 Moros First Encounter engram already exists, skipping seed");
            return;
        }
        Ok(None) => {}
        Err(e) => {
            warn!(
                "⚠️ Could not check Moros First Encounter engram (engrams service may be down): {}",
                e
            );
            return;
        }
    }

    let content = include_str!("agents/moros_first_encounter.md");

    let request = CreateEngramRequest {
        wallet_address: wallet.to_string(),
        engram_type: "persona".to_string(),
        key: key.to_string(),
        content: content.to_string(),
        metadata: Some(serde_json::json!({
            "type": "architect_persona",
            "source": "first_encounter",
            "relationship": "architect"
        })),
        tags: Some(vec![
            "persona".to_string(),
            "architect".to_string(),
            "first_encounter".to_string(),
            "moros".to_string(),
        ]),
        is_public: Some(false),
    };

    match client.create_engram(request).await {
        Ok(engram) => {
            info!(
                "🌑 Moros First Encounter engram seeded: {} ({})",
                engram.key, engram.id
            );
        }
        Err(e) => {
            warn!("⚠️ Failed to seed Moros First Encounter engram: {}", e);
        }
    }
}
