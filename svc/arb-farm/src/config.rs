use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub service_name: String,
    pub port: u16,
    pub database_url: String,
    pub erebus_url: String,
    pub engrams_url: String,
    pub agents_service_url: String,

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
    pub helius_sender_url: String,
    pub helius_laserstream_url: String,
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

    // Dev wallet (private key for local dev only)
    pub wallet_address: Option<String>,
    pub wallet_private_key: Option<String>,

    // Turnkey wallet delegation (production)
    pub turnkey_api_url: String,
    pub turnkey_organization_id: Option<String>,
    pub turnkey_api_public_key: Option<String>,
    pub turnkey_api_private_key: Option<String>,

    // Risk defaults
    pub default_max_position_sol: f64,
    pub default_daily_loss_limit_sol: f64,
    pub default_min_profit_bps: u16,
    pub default_max_slippage_bps: u16,

    // Graduation tracker settings
    pub graduation_threshold: Option<f64>,
    pub tracker_fast_poll_ms: Option<u64>,
    pub tracker_normal_poll_ms: Option<u64>,
    pub tracker_rpc_timeout_secs: Option<u64>,
    pub tracker_eviction_hours: Option<i64>,
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
            agents_service_url: env::var("AGENTS_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:9003".to_string()),

            // Solana RPC
            rpc_url: env::var("SOLANA_RPC_URL")
                .or_else(|_| env::var("HELIUS_RPC_URL"))
                .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string()),
            jito_block_engine_url: env::var("JITO_BLOCK_ENGINE_URL")
                .unwrap_or_else(|_| "https://mainnet.block-engine.jito.wtf".to_string()),

            // External APIs
            jupiter_api_url: env::var("JUPITER_API_URL")
                .unwrap_or_else(|_| "https://lite-api.jup.ag/swap/v1".to_string()),
            raydium_api_url: env::var("RAYDIUM_API_URL")
                .unwrap_or_else(|_| "https://api.raydium.io/v2".to_string()),
            pump_fun_api_url: env::var("PUMP_FUN_API_URL")
                .unwrap_or_else(|_| "https://api.dexscreener.com/latest/dex".to_string()),
            moonshot_api_url: env::var("MOONSHOT_API_URL")
                .unwrap_or_else(|_| "https://api.dexscreener.com/latest/dex".to_string()),
            helius_api_url: env::var("HELIUS_API_URL")
                .unwrap_or_else(|_| "https://mainnet.helius-rpc.com".to_string()),
            helius_api_key: env::var("HELIUS_API_KEY").ok(),
            helius_sender_url: env::var("HELIUS_SENDER_URL")
                .unwrap_or_else(|_| "https://mainnet.helius-rpc.com".to_string()),
            helius_laserstream_url: env::var("HELIUS_LASERSTREAM_URL")
                .unwrap_or_else(|_| "https://laserstream-mainnet.helius-rpc.com".to_string()),
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

            // Dev wallet (private key for local dev only)
            wallet_address: env::var("ARB_FARM_WALLET_ADDRESS").ok(),
            wallet_private_key: env::var("ARB_FARM_WALLET_PRIVATE_KEY").ok(),

            // Turnkey wallet delegation (production)
            turnkey_api_url: env::var("TURNKEY_API_URL")
                .unwrap_or_else(|_| "https://api.turnkey.com".to_string()),
            turnkey_organization_id: env::var("TURNKEY_ORGANIZATION_ID").ok(),
            turnkey_api_public_key: env::var("TURNKEY_API_PUBLIC_KEY").ok(),
            turnkey_api_private_key: env::var("TURNKEY_API_PRIVATE_KEY").ok(),

            // Risk defaults (dev_testing profile)
            default_max_position_sol: env::var("DEFAULT_MAX_POSITION_SOL")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.01),
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

            // Graduation tracker settings (all optional with defaults in TrackerConfig)
            graduation_threshold: env::var("GRADUATION_THRESHOLD")
                .ok()
                .and_then(|v| v.parse().ok()),
            tracker_fast_poll_ms: env::var("TRACKER_FAST_POLL_MS")
                .ok()
                .and_then(|v| v.parse().ok()),
            tracker_normal_poll_ms: env::var("TRACKER_NORMAL_POLL_MS")
                .ok()
                .and_then(|v| v.parse().ok()),
            tracker_rpc_timeout_secs: env::var("TRACKER_RPC_TIMEOUT_SECS")
                .ok()
                .and_then(|v| v.parse().ok()),
            tracker_eviction_hours: env::var("TRACKER_EVICTION_HOURS")
                .ok()
                .and_then(|v| v.parse().ok()),
        })
    }
}
