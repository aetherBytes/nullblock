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
    pub dexscreener_api_url: String,
    pub moonshot_api_url: String,
    pub helius_api_url: String,
    pub helius_api_key: Option<String>,
    pub helius_webhook_auth_token: Option<String>,
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

    // Serper for web search
    pub serper_api_url: String,
    pub serper_api_key: Option<String>,

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
                .map_err(|_| anyhow::anyhow!("DATABASE_URL or ARB_FARM_DATABASE_URL must be set"))?,
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
                .unwrap_or_else(|_| "https://frontend-api-v3.pump.fun".to_string()),
            dexscreener_api_url: env::var("DEXSCREENER_API_URL")
                .unwrap_or_else(|_| "https://api.dexscreener.com/latest/dex".to_string()),
            moonshot_api_url: env::var("MOONSHOT_API_URL")
                .unwrap_or_else(|_| "https://api.dexscreener.com/latest/dex".to_string()),
            helius_api_url: env::var("HELIUS_API_URL")
                .unwrap_or_else(|_| "https://mainnet.helius-rpc.com".to_string()),
            helius_api_key: env::var("HELIUS_API_KEY").ok(),
            helius_webhook_auth_token: env::var("HELIUS_WEBHOOK_AUTH_TOKEN").ok(),
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

            // Serper for web search
            serper_api_url: env::var("SERPER_API_URL")
                .unwrap_or_else(|_| "https://google.serper.dev".to_string()),
            serper_api_key: env::var("SERPER_API_KEY").ok(),

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
                .unwrap_or(2.0),
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

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Validate RPC URL format
        if !self.rpc_url.starts_with("http://") && !self.rpc_url.starts_with("https://") {
            errors.push(format!("Invalid RPC URL: {} (must start with http:// or https://)", self.rpc_url));
        }

        // Validate database URL format
        if !self.database_url.starts_with("postgres") {
            errors.push(format!("Invalid database URL: must start with 'postgres'"));
        }

        // Validate Helius API key if URL is configured
        if self.helius_api_url.contains("helius") && self.helius_api_key.is_none() {
            tracing::warn!("⚠️ Helius API URL configured but HELIUS_API_KEY is not set - rate limits may apply");
        }

        // Warn if webhook auth token not set - webhooks will be REJECTED
        if self.helius_webhook_auth_token.is_none() {
            tracing::warn!("⚠️ HELIUS_WEBHOOK_AUTH_TOKEN not set - webhooks will be REJECTED (401)");
            tracing::warn!("   Copy trading requires this token to be set for webhook authentication");
        }

        // Validate risk parameters are within sensible bounds
        if self.default_max_position_sol <= 0.0 {
            errors.push("default_max_position_sol must be > 0".to_string());
        }
        if self.default_max_position_sol > 100.0 {
            tracing::warn!("⚠️ default_max_position_sol is very high ({} SOL) - is this intentional?", self.default_max_position_sol);
        }

        if self.default_daily_loss_limit_sol <= 0.0 {
            errors.push("default_daily_loss_limit_sol must be > 0".to_string());
        }

        if self.default_max_slippage_bps > 5000 {
            tracing::warn!("⚠️ default_max_slippage_bps is very high ({} bps = {}%) - is this intentional?",
                self.default_max_slippage_bps, self.default_max_slippage_bps as f64 / 100.0);
        }

        // Validate Turnkey config is complete if any Turnkey env var is set
        let turnkey_partially_configured = self.turnkey_organization_id.is_some()
            || self.turnkey_api_public_key.is_some()
            || self.turnkey_api_private_key.is_some();

        if turnkey_partially_configured {
            if self.turnkey_organization_id.is_none() {
                errors.push("TURNKEY_ORGANIZATION_ID required when Turnkey is configured".to_string());
            }
            if self.turnkey_api_public_key.is_none() {
                errors.push("TURNKEY_API_PUBLIC_KEY required when Turnkey is configured".to_string());
            }
            if self.turnkey_api_private_key.is_none() {
                errors.push("TURNKEY_API_PRIVATE_KEY required when Turnkey is configured".to_string());
            }
        }

        // Validate dev wallet config is complete if any dev wallet env var is set
        if self.wallet_private_key.is_some() && self.wallet_address.is_none() {
            tracing::warn!("⚠️ ARB_FARM_WALLET_PRIVATE_KEY set but ARB_FARM_WALLET_ADDRESS not set - address will be derived");
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
