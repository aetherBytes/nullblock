pub mod config;
mod engine;
pub mod model_discovery;
mod openrouter;
pub mod providers;
mod voting;

pub use config::{
    ConsensusConfig, ConsensusConfigManager, ConsensusModelConfig, UpdateConsensusConfigRequest,
    get_all_available_models, get_dev_wallet_models, get_models_for_wallet, get_standard_models,
    is_dev_wallet, AVAILABLE_MODELS as CONFIG_AVAILABLE_MODELS,
};
pub use engine::*;
pub use model_discovery::{
    discover_best_reasoning_models, get_discovered_models, get_fallback_models,
    refresh_models, get_discovery_status, ModelDiscoveryStatus,
};
pub use openrouter::{get_default_models, get_model_weight, OpenRouterClient, AVAILABLE_MODELS, quick_llm_call};
pub use voting::*;
