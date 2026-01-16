use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub service_name: String,
    pub port: u16,
    pub database_url: String,
    pub erebus_url: String,
    pub engrams_url: String,

    // Solana RPC
    pub rpc_url: String,
    pub jito_block_engine_url: String,

    // External API URLs
    pub jupiter_api_url: String,
    pub raydium_api_url: String,
    pub pump_fun_api_url: String,
    pub moonshot_api_url: String,
    pub helius_api_url: String,
    pub helius_api_key: Option<String>,
    pub birdeye_api_url: String,
    pub birdeye_api_key: Option<String>,
    pub jito_api_url: String,
    pub rugcheck_api_url: String,
    pub goplus_api_url: String,

    // Lending protocols
    pub marginfi_api_url: String,
    pub kamino_api_url: String,

    // OpenRouter for multi-LLM consensus
    pub openrouter_api_url: String,
    pub openrouter_api_key: Option<String>,

    // Turnkey wallet delegation
    pub turnkey_api_url: String,
    pub turnkey_organization_id: Option<String>,
    pub turnkey_api_public_key: Option<String>,
    pub turnkey_api_private_key: Option<String>,

    // Risk defaults
    pub default_max_position_sol: f64,
    pub default_daily_loss_limit_sol: f64,
    pub default_min_profit_bps: u16,
    pub default_max_slippage_bps: u16,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            service_name: env::var("SERVICE_NAME")
                .unwrap_or_else(|_| "arb-farm".to_string()),
            port: env::var("ARB_FARM_PORT")
                .unwrap_or_else(|_| "9007".to_string())
                .parse()
                .unwrap_or(9007),
            database_url: env::var("ARB_FARM_DATABASE_URL")
                .or_else(|_| env::var("DATABASE_URL"))
                .unwrap_or_else(|_| {
                    "postgresql://postgres:postgres_secure_pass@localhost:5441/agents".to_string()
                }),
            erebus_url: env::var("EREBUS_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
            engrams_url: env::var("ENGRAMS_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:9004".to_string()),

            // Solana RPC
            rpc_url: env::var("SOLANA_RPC_URL")
                .or_else(|_| env::var("HELIUS_RPC_URL"))
                .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string()),
            jito_block_engine_url: env::var("JITO_BLOCK_ENGINE_URL")
                .unwrap_or_else(|_| "https://mainnet.block-engine.jito.wtf".to_string()),

            // External APIs
            jupiter_api_url: env::var("JUPITER_API_URL")
                .unwrap_or_else(|_| "https://quote-api.jup.ag/v6".to_string()),
            raydium_api_url: env::var("RAYDIUM_API_URL")
                .unwrap_or_else(|_| "https://api-v3.raydium.io".to_string()),
            pump_fun_api_url: env::var("PUMP_FUN_API_URL")
                .unwrap_or_else(|_| "https://pumpportal.fun/api".to_string()),
            moonshot_api_url: env::var("MOONSHOT_API_URL")
                .unwrap_or_else(|_| "https://api.moonshot.cc/v1".to_string()),
            helius_api_url: env::var("HELIUS_API_URL")
                .unwrap_or_else(|_| "https://mainnet.helius-rpc.com".to_string()),
            helius_api_key: env::var("HELIUS_API_KEY").ok(),
            birdeye_api_url: env::var("BIRDEYE_API_URL")
                .unwrap_or_else(|_| "https://public-api.birdeye.so".to_string()),
            birdeye_api_key: env::var("BIRDEYE_API_KEY").ok(),
            jito_api_url: env::var("JITO_API_URL")
                .unwrap_or_else(|_| "https://mainnet.block-engine.jito.wtf".to_string()),
            rugcheck_api_url: env::var("RUGCHECK_API_URL")
                .unwrap_or_else(|_| "https://api.rugcheck.xyz/v1".to_string()),
            goplus_api_url: env::var("GOPLUS_API_URL")
                .unwrap_or_else(|_| "https://api.gopluslabs.io/api/v1".to_string()),

            // Lending protocols
            marginfi_api_url: env::var("MARGINFI_API_URL")
                .unwrap_or_else(|_| "https://api.marginfi.com/v1".to_string()),
            kamino_api_url: env::var("KAMINO_API_URL")
                .unwrap_or_else(|_| "https://api.kamino.finance/v1".to_string()),

            // OpenRouter
            openrouter_api_url: env::var("OPENROUTER_API_URL")
                .unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string()),
            openrouter_api_key: env::var("OPENROUTER_API_KEY").ok(),

            // Turnkey wallet delegation
            turnkey_api_url: env::var("TURNKEY_API_URL")
                .unwrap_or_else(|_| "https://api.turnkey.com".to_string()),
            turnkey_organization_id: env::var("TURNKEY_ORGANIZATION_ID").ok(),
            turnkey_api_public_key: env::var("TURNKEY_API_PUBLIC_KEY").ok(),
            turnkey_api_private_key: env::var("TURNKEY_API_PRIVATE_KEY").ok(),

            // Risk defaults (dev_testing profile)
            default_max_position_sol: env::var("DEFAULT_MAX_POSITION_SOL")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(5.0),
            default_daily_loss_limit_sol: env::var("DEFAULT_DAILY_LOSS_LIMIT_SOL")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(2.0),
            default_min_profit_bps: env::var("DEFAULT_MIN_PROFIT_BPS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(50), // 0.5%
            default_max_slippage_bps: env::var("DEFAULT_MAX_SLIPPAGE_BPS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100), // 1%
        })
    }
}
