use crate::{agents::HecateAgent, config::Config, error::AppResult};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub hecate_agent: Arc<RwLock<HecateAgent>>,
}

impl AppState {
    pub async fn new(config: Config) -> AppResult<Self> {
        // Initialize Hecate agent
        let mut hecate_agent = HecateAgent::new(None);
        
        // Get API keys from config
        let api_keys = config.get_api_keys();
        
        // Start the agent
        hecate_agent.start(&api_keys).await?;

        Ok(Self {
            config,
            hecate_agent: Arc::new(RwLock::new(hecate_agent)),
        })
    }
}