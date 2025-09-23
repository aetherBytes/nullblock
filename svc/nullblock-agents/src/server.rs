use crate::{agents::HecateAgent, config::Config, database::Database, kafka::{KafkaConfig, KafkaProducer}, error::AppResult};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub hecate_agent: Arc<RwLock<HecateAgent>>,
    pub database: Option<Arc<Database>>,
    pub kafka_producer: Option<Arc<KafkaProducer>>,
}

impl AppState {
    pub async fn new(config: Config) -> AppResult<Self> {
        // Initialize database connection if URL is provided
        let database = if let Ok(database_url) = std::env::var("DATABASE_URL") {
            let db = Database::new(&database_url).await?;

            // Run migrations
            if let Err(e) = db.run_migrations().await {
                tracing::warn!("Failed to run database migrations: {}", e);
            } else {
                tracing::info!("✅ Database migrations completed successfully");
            }

            Some(Arc::new(db))
        } else {
            tracing::warn!("⚠️ DATABASE_URL not set, running without persistent storage");
            None
        };

        // Initialize Hecate agent
        let mut hecate_agent = HecateAgent::new(None);

        // Get API keys from config
        let api_keys = config.get_api_keys();

        // Start the agent
        hecate_agent.start(&api_keys).await?;

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
            database,
            kafka_producer,
        })
    }
}