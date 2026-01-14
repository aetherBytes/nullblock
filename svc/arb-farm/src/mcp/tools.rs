use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolResult {
    pub content: Vec<McpContent>,
    #[serde(rename = "isError", skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

impl McpToolResult {
    pub fn success(text: impl Into<String>) -> Self {
        Self {
            content: vec![McpContent {
                content_type: "text".to_string(),
                text: text.into(),
            }],
            is_error: None,
        }
    }

    pub fn error(text: impl Into<String>) -> Self {
        Self {
            content: vec![McpContent {
                content_type: "text".to_string(),
                text: text.into(),
            }],
            is_error: Some(true),
        }
    }
}

pub fn get_scanner_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "scanner_status".to_string(),
            description: "Get scanner status and statistics including venue health".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        McpTool {
            name: "scanner_signals".to_string(),
            description: "Get recent signals with optional filtering by venue type, minimum profit, and confidence".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "venue_type": {
                        "type": "string",
                        "description": "Filter by venue type: dex_amm, bonding_curve, lending, orderbook",
                        "enum": ["dex_amm", "bonding_curve", "lending", "orderbook"]
                    },
                    "min_profit_bps": {
                        "type": "integer",
                        "description": "Minimum estimated profit in basis points"
                    },
                    "min_confidence": {
                        "type": "number",
                        "description": "Minimum confidence score (0.0-1.0)"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of signals to return",
                        "default": 20
                    }
                },
                "required": []
            }),
        },
        McpTool {
            name: "scanner_add_venue".to_string(),
            description: "Add a venue to scan for MEV opportunities".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "venue_type": {
                        "type": "string",
                        "description": "Type of venue",
                        "enum": ["dex_amm", "bonding_curve", "lending", "orderbook"]
                    },
                    "address": {
                        "type": "string",
                        "description": "Contract/pool address if applicable"
                    },
                    "config": {
                        "type": "object",
                        "description": "Venue-specific configuration"
                    }
                },
                "required": ["venue_type"]
            }),
        },
    ]
}

pub fn get_edge_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "edge_list".to_string(),
            description: "List detected edges (opportunities) with filtering".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "status": {
                        "type": "string",
                        "description": "Filter by status",
                        "enum": ["detected", "pending_approval", "executing", "executed", "expired", "failed", "rejected"]
                    },
                    "venue_type": {
                        "type": "string",
                        "description": "Filter by venue type"
                    },
                    "min_profit_lamports": {
                        "type": "integer",
                        "description": "Minimum estimated profit in lamports"
                    },
                    "limit": {
                        "type": "integer",
                        "default": 50
                    }
                },
                "required": []
            }),
        },
        McpTool {
            name: "edge_details".to_string(),
            description: "Get full details for a specific edge".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "edge_id": {
                        "type": "string",
                        "description": "UUID of the edge"
                    }
                },
                "required": ["edge_id"]
            }),
        },
        McpTool {
            name: "edge_approve".to_string(),
            description: "Approve an edge for execution".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "edge_id": {
                        "type": "string",
                        "description": "UUID of the edge to approve"
                    }
                },
                "required": ["edge_id"]
            }),
        },
        McpTool {
            name: "edge_reject".to_string(),
            description: "Reject an edge with a reason (saved as avoidance engram)".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "edge_id": {
                        "type": "string",
                        "description": "UUID of the edge to reject"
                    },
                    "reason": {
                        "type": "string",
                        "description": "Reason for rejection"
                    }
                },
                "required": ["edge_id", "reason"]
            }),
        },
        McpTool {
            name: "edge_classify_atomicity".to_string(),
            description: "Analyze if an edge is atomic (guaranteed profit or revert)".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "edge_id": {
                        "type": "string",
                        "description": "UUID of the edge"
                    }
                },
                "required": ["edge_id"]
            }),
        },
    ]
}

pub fn get_strategy_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "strategy_list".to_string(),
            description: "List active trading strategies".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        McpTool {
            name: "strategy_create".to_string(),
            description: "Create a new trading strategy".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Strategy name"
                    },
                    "venue_types": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Venue types to match"
                    },
                    "execution_mode": {
                        "type": "string",
                        "enum": ["autonomous", "agent_directed", "hybrid"],
                        "description": "How edges should be executed"
                    },
                    "risk_params": {
                        "type": "object",
                        "description": "Risk parameters for the strategy"
                    }
                },
                "required": ["name", "venue_types", "execution_mode"]
            }),
        },
        McpTool {
            name: "strategy_toggle".to_string(),
            description: "Enable or disable a strategy".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "strategy_id": {
                        "type": "string",
                        "description": "UUID of the strategy"
                    },
                    "enabled": {
                        "type": "boolean",
                        "description": "Whether to enable or disable"
                    }
                },
                "required": ["strategy_id", "enabled"]
            }),
        },
    ]
}

pub fn get_threat_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "threat_check_token".to_string(),
            description: "Run full threat analysis on a token using RugCheck, GoPlus, and Birdeye APIs".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "token_mint": {
                        "type": "string",
                        "description": "Token mint address"
                    },
                    "force_refresh": {
                        "type": "boolean",
                        "description": "Force refresh (skip cache)",
                        "default": false
                    }
                },
                "required": ["token_mint"]
            }),
        },
        McpTool {
            name: "threat_check_wallet".to_string(),
            description: "Analyze a wallet for scam history and associations".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "wallet_address": {
                        "type": "string",
                        "description": "Wallet address"
                    }
                },
                "required": ["wallet_address"]
            }),
        },
        McpTool {
            name: "threat_list_blocked".to_string(),
            description: "List blocked tokens/wallets".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "category": {
                        "type": "string",
                        "enum": ["rug_pull", "honeypot", "scam_wallet", "wash_trader", "fake_token", "blacklist_function", "bundle_manipulation"],
                        "description": "Filter by threat category"
                    },
                    "limit": {
                        "type": "integer",
                        "default": 50
                    }
                },
                "required": []
            }),
        },
        McpTool {
            name: "threat_report".to_string(),
            description: "Manually report a threat and add to blocklist".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "entity_type": {
                        "type": "string",
                        "enum": ["token", "wallet", "contract"],
                        "description": "Type of entity"
                    },
                    "address": {
                        "type": "string",
                        "description": "Entity address"
                    },
                    "category": {
                        "type": "string",
                        "enum": ["rug_pull", "honeypot", "scam_wallet", "wash_trader", "fake_token"],
                        "description": "Threat category"
                    },
                    "reason": {
                        "type": "string",
                        "description": "Reason for reporting"
                    },
                    "evidence_url": {
                        "type": "string",
                        "description": "URL with evidence"
                    }
                },
                "required": ["entity_type", "address", "category", "reason"]
            }),
        },
        McpTool {
            name: "threat_whitelist".to_string(),
            description: "Whitelist a trusted token or wallet".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "entity_type": {
                        "type": "string",
                        "enum": ["token", "wallet"],
                        "description": "Type of entity"
                    },
                    "address": {
                        "type": "string",
                        "description": "Entity address"
                    },
                    "reason": {
                        "type": "string",
                        "description": "Reason for whitelisting"
                    }
                },
                "required": ["entity_type", "address", "reason"]
            }),
        },
        McpTool {
            name: "threat_watch_wallet".to_string(),
            description: "Add a wallet to high-alert monitoring (e.g., token creator)".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "wallet_address": {
                        "type": "string",
                        "description": "Wallet address to watch"
                    },
                    "related_token_mint": {
                        "type": "string",
                        "description": "Related token mint address"
                    },
                    "watch_reason": {
                        "type": "string",
                        "description": "Reason for watching"
                    },
                    "alert_on_sell": {
                        "type": "boolean",
                        "description": "Alert when wallet sells tokens",
                        "default": true
                    },
                    "alert_threshold_sol": {
                        "type": "number",
                        "description": "Alert if sells exceed this SOL amount"
                    }
                },
                "required": ["wallet_address"]
            }),
        },
        McpTool {
            name: "threat_alerts".to_string(),
            description: "Get recent threat alerts".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "severity": {
                        "type": "string",
                        "enum": ["low", "medium", "high", "critical"],
                        "description": "Filter by severity"
                    },
                    "limit": {
                        "type": "integer",
                        "default": 50
                    }
                },
                "required": []
            }),
        },
        McpTool {
            name: "threat_score_history".to_string(),
            description: "Get threat score history for a token".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "token_mint": {
                        "type": "string",
                        "description": "Token mint address"
                    }
                },
                "required": ["token_mint"]
            }),
        },
        McpTool {
            name: "threat_stats".to_string(),
            description: "Get threat detection statistics".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        McpTool {
            name: "threat_is_blocked".to_string(),
            description: "Check if a specific address is blocked".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "address": {
                        "type": "string",
                        "description": "Address to check"
                    }
                },
                "required": ["address"]
            }),
        },
        McpTool {
            name: "threat_is_whitelisted".to_string(),
            description: "Check if a specific address is whitelisted".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "address": {
                        "type": "string",
                        "description": "Address to check"
                    }
                },
                "required": ["address"]
            }),
        },
    ]
}

pub fn get_event_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "event_subscribe".to_string(),
            description: "Subscribe to events matching topic patterns".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "topics": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Topic patterns to subscribe to (e.g., 'arb.edge.*')"
                    },
                    "since_timestamp": {
                        "type": "string",
                        "description": "ISO timestamp to replay events from"
                    }
                },
                "required": ["topics"]
            }),
        },
        McpTool {
            name: "event_history".to_string(),
            description: "Query historical events".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "topics": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Topic patterns to query"
                    },
                    "since": {
                        "type": "string",
                        "description": "Start timestamp (ISO format)"
                    },
                    "until": {
                        "type": "string",
                        "description": "End timestamp (ISO format)"
                    },
                    "limit": {
                        "type": "integer",
                        "default": 100
                    }
                },
                "required": []
            }),
        },
    ]
}

pub fn get_curve_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "curve_buy_token".to_string(),
            description: "Get a quote for buying tokens on a bonding curve (pump.fun or moonshot)".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "token_mint": {
                        "type": "string",
                        "description": "Token mint address"
                    },
                    "sol_amount": {
                        "type": "number",
                        "description": "Amount of SOL to spend"
                    },
                    "venue": {
                        "type": "string",
                        "enum": ["pump_fun", "moonshot"],
                        "description": "Bonding curve venue",
                        "default": "pump_fun"
                    }
                },
                "required": ["token_mint", "sol_amount"]
            }),
        },
        McpTool {
            name: "curve_sell_token".to_string(),
            description: "Get a quote for selling tokens on a bonding curve (before or after graduation)".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "token_mint": {
                        "type": "string",
                        "description": "Token mint address"
                    },
                    "token_amount": {
                        "type": "number",
                        "description": "Amount of tokens to sell"
                    },
                    "venue": {
                        "type": "string",
                        "enum": ["pump_fun", "moonshot"],
                        "description": "Bonding curve venue",
                        "default": "pump_fun"
                    }
                },
                "required": ["token_mint", "token_amount"]
            }),
        },
        McpTool {
            name: "curve_check_progress".to_string(),
            description: "Check graduation progress percentage for a bonding curve token".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "token_mint": {
                        "type": "string",
                        "description": "Token mint address"
                    },
                    "venue": {
                        "type": "string",
                        "enum": ["pump_fun", "moonshot"],
                        "description": "Bonding curve venue",
                        "default": "pump_fun"
                    }
                },
                "required": ["token_mint"]
            }),
        },
        McpTool {
            name: "curve_get_holder_stats".to_string(),
            description: "Get holder statistics including top holders and concentration".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "token_mint": {
                        "type": "string",
                        "description": "Token mint address"
                    },
                    "venue": {
                        "type": "string",
                        "enum": ["pump_fun", "moonshot"],
                        "description": "Bonding curve venue",
                        "default": "pump_fun"
                    }
                },
                "required": ["token_mint"]
            }),
        },
        McpTool {
            name: "curve_graduation_eta".to_string(),
            description: "Get estimated time/blocks until graduation based on current volume".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "token_mint": {
                        "type": "string",
                        "description": "Token mint address"
                    },
                    "venue": {
                        "type": "string",
                        "enum": ["pump_fun", "moonshot"],
                        "description": "Bonding curve venue",
                        "default": "pump_fun"
                    }
                },
                "required": ["token_mint"]
            }),
        },
        McpTool {
            name: "curve_list_tokens".to_string(),
            description: "List recent bonding curve tokens with progress and volume info".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "venue": {
                        "type": "string",
                        "enum": ["pump_fun", "moonshot", "all"],
                        "description": "Bonding curve venue to query",
                        "default": "all"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of tokens to return",
                        "default": 50
                    }
                },
                "required": []
            }),
        },
        McpTool {
            name: "curve_graduation_candidates".to_string(),
            description: "List tokens approaching graduation (configurable progress range)".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "min_progress": {
                        "type": "number",
                        "description": "Minimum graduation progress percentage",
                        "default": 50
                    },
                    "max_progress": {
                        "type": "number",
                        "description": "Maximum graduation progress percentage",
                        "default": 95
                    },
                    "venue": {
                        "type": "string",
                        "enum": ["pump_fun", "moonshot", "all"],
                        "description": "Bonding curve venue",
                        "default": "all"
                    },
                    "limit": {
                        "type": "integer",
                        "default": 20
                    }
                },
                "required": []
            }),
        },
        McpTool {
            name: "curve_cross_venue_arb".to_string(),
            description: "Detect cross-venue arbitrage opportunities between bonding curves and DEXes".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "min_diff_percent": {
                        "type": "number",
                        "description": "Minimum price difference percentage to report",
                        "default": 1.0
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum opportunities to return",
                        "default": 20
                    }
                },
                "required": []
            }),
        },
        McpTool {
            name: "curve_get_parameters".to_string(),
            description: "Get bonding curve parameters (curve type, initial/current price, etc.)".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "token_mint": {
                        "type": "string",
                        "description": "Token mint address"
                    }
                },
                "required": ["token_mint"]
            }),
        },
        McpTool {
            name: "curve_venues_health".to_string(),
            description: "Check health status of bonding curve venues (pump.fun, moonshot)".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
    ]
}

pub fn get_research_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "research_ingest_url".to_string(),
            description: "Ingest and analyze a URL for trading strategies (tweets, articles, threads)".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "URL to ingest and analyze"
                    },
                    "context": {
                        "type": "string",
                        "description": "Optional context about what to look for"
                    },
                    "extract_strategy": {
                        "type": "boolean",
                        "description": "Whether to extract trading strategies using LLM",
                        "default": true
                    }
                },
                "required": ["url"]
            }),
        },
        McpTool {
            name: "research_monitor_account".to_string(),
            description: "Add X/Twitter account to monitoring list for alpha or threat intel".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "handle": {
                        "type": "string",
                        "description": "Twitter/X handle (e.g., @ZachXBT)"
                    },
                    "track_type": {
                        "type": "string",
                        "enum": ["alpha", "threat", "both"],
                        "description": "Type of content to track",
                        "default": "both"
                    },
                    "keywords": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Additional keywords to monitor"
                    }
                },
                "required": ["handle"]
            }),
        },
        McpTool {
            name: "research_backtest_strategy".to_string(),
            description: "Backtest a discovered strategy against historical data".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "strategy_id": {
                        "type": "string",
                        "description": "UUID of the strategy to backtest"
                    },
                    "period_days": {
                        "type": "integer",
                        "description": "Number of days to backtest",
                        "default": 30
                    },
                    "initial_capital_sol": {
                        "type": "number",
                        "description": "Starting capital in SOL",
                        "default": 10.0
                    },
                    "max_position_size_sol": {
                        "type": "number",
                        "description": "Maximum position size per trade",
                        "default": 1.0
                    }
                },
                "required": ["strategy_id"]
            }),
        },
        McpTool {
            name: "research_list_discoveries".to_string(),
            description: "List discovered strategies pending review".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "status": {
                        "type": "string",
                        "enum": ["pending", "approved", "rejected", "testing", "live"],
                        "description": "Filter by status"
                    },
                    "limit": {
                        "type": "integer",
                        "default": 20
                    }
                },
                "required": []
            }),
        },
        McpTool {
            name: "research_approve_discovery".to_string(),
            description: "Approve a discovered strategy for testing or live deployment".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "discovery_id": {
                        "type": "string",
                        "description": "UUID of the discovery to approve"
                    },
                    "notes": {
                        "type": "string",
                        "description": "Optional approval notes"
                    }
                },
                "required": ["discovery_id"]
            }),
        },
        McpTool {
            name: "research_reject_discovery".to_string(),
            description: "Reject a discovered strategy with a reason".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "discovery_id": {
                        "type": "string",
                        "description": "UUID of the discovery to reject"
                    },
                    "reason": {
                        "type": "string",
                        "description": "Reason for rejection"
                    }
                },
                "required": ["discovery_id", "reason"]
            }),
        },
        McpTool {
            name: "research_list_sources".to_string(),
            description: "List monitored social media sources (Twitter, Telegram, etc.)".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "source_type": {
                        "type": "string",
                        "enum": ["twitter", "telegram", "discord", "rss"],
                        "description": "Filter by source type"
                    },
                    "track_type": {
                        "type": "string",
                        "enum": ["alpha", "threat", "both"],
                        "description": "Filter by track type"
                    },
                    "active_only": {
                        "type": "boolean",
                        "description": "Only show active sources",
                        "default": true
                    }
                },
                "required": []
            }),
        },
        McpTool {
            name: "research_alerts".to_string(),
            description: "Get recent social monitoring alerts".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "source_id": {
                        "type": "string",
                        "description": "Filter by source UUID"
                    },
                    "alert_type": {
                        "type": "string",
                        "enum": ["trading_alpha", "new_token", "price_alert", "whale_activity", "rug_warning", "scam_alert", "keyword_match"],
                        "description": "Filter by alert type"
                    },
                    "limit": {
                        "type": "integer",
                        "default": 50
                    }
                },
                "required": []
            }),
        },
        McpTool {
            name: "research_stats".to_string(),
            description: "Get research/social monitoring statistics".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
    ]
}

pub fn get_kol_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "kol_track".to_string(),
            description: "Start tracking a KOL wallet or social handle".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "wallet_address": {
                        "type": "string",
                        "description": "Solana wallet address to track"
                    },
                    "twitter_handle": {
                        "type": "string",
                        "description": "Twitter/X handle (e.g., @CryptoWhale)"
                    },
                    "display_name": {
                        "type": "string",
                        "description": "Display name for the KOL"
                    }
                },
                "required": []
            }),
        },
        McpTool {
            name: "kol_list".to_string(),
            description: "List tracked KOLs with trust scores".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "is_active": {
                        "type": "boolean",
                        "description": "Filter by active status"
                    },
                    "copy_enabled": {
                        "type": "boolean",
                        "description": "Filter by copy trading status"
                    },
                    "min_trust_score": {
                        "type": "number",
                        "description": "Minimum trust score (0-100)"
                    },
                    "limit": {
                        "type": "integer",
                        "default": 50
                    }
                },
                "required": []
            }),
        },
        McpTool {
            name: "kol_stats".to_string(),
            description: "Get detailed KOL performance statistics".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "kol_id": {
                        "type": "string",
                        "description": "UUID of the KOL"
                    }
                },
                "required": ["kol_id"]
            }),
        },
        McpTool {
            name: "kol_trades".to_string(),
            description: "Get recent trades made by a KOL".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "kol_id": {
                        "type": "string",
                        "description": "UUID of the KOL"
                    },
                    "trade_type": {
                        "type": "string",
                        "enum": ["buy", "sell"],
                        "description": "Filter by trade type"
                    },
                    "limit": {
                        "type": "integer",
                        "default": 50
                    }
                },
                "required": ["kol_id"]
            }),
        },
        McpTool {
            name: "kol_trust_breakdown".to_string(),
            description: "Get detailed trust score breakdown for a KOL".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "kol_id": {
                        "type": "string",
                        "description": "UUID of the KOL"
                    }
                },
                "required": ["kol_id"]
            }),
        },
        McpTool {
            name: "copy_enable".to_string(),
            description: "Enable copy trading for a KOL".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "kol_id": {
                        "type": "string",
                        "description": "UUID of the KOL"
                    },
                    "max_position_sol": {
                        "type": "number",
                        "description": "Maximum SOL per trade",
                        "default": 0.5
                    },
                    "delay_ms": {
                        "type": "integer",
                        "description": "Delay before copying (ms)",
                        "default": 500
                    },
                    "min_trust_score": {
                        "type": "number",
                        "description": "Auto-disable if trust drops below",
                        "default": 60
                    },
                    "copy_percentage": {
                        "type": "number",
                        "description": "Percentage of KOL position to copy (0.1 = 10%)",
                        "default": 0.1
                    }
                },
                "required": ["kol_id"]
            }),
        },
        McpTool {
            name: "copy_disable".to_string(),
            description: "Disable copy trading for a KOL".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "kol_id": {
                        "type": "string",
                        "description": "UUID of the KOL"
                    }
                },
                "required": ["kol_id"]
            }),
        },
        McpTool {
            name: "copy_active".to_string(),
            description: "List currently active copy positions".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        McpTool {
            name: "copy_stats".to_string(),
            description: "Get copy trading statistics".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        McpTool {
            name: "copy_history".to_string(),
            description: "Get copy trade history for a KOL".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "kol_id": {
                        "type": "string",
                        "description": "UUID of the KOL"
                    },
                    "status": {
                        "type": "string",
                        "enum": ["pending", "executing", "executed", "failed", "skipped"],
                        "description": "Filter by status"
                    },
                    "limit": {
                        "type": "integer",
                        "default": 50
                    }
                },
                "required": ["kol_id"]
            }),
        },
    ]
}

pub fn get_engram_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "engram_create".to_string(),
            description: "Create a new engram (pattern, avoidance, strategy, or intel)".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "key": {
                        "type": "string",
                        "description": "Unique key for the engram (e.g., 'arb.pattern.dex_spread')"
                    },
                    "engram_type": {
                        "type": "string",
                        "enum": ["edge_pattern", "avoidance", "strategy", "threat_intel", "consensus_outcome", "trade_result", "market_condition"],
                        "description": "Type of engram"
                    },
                    "content": {
                        "type": "object",
                        "description": "Engram content (structure depends on type)"
                    },
                    "confidence": {
                        "type": "number",
                        "description": "Confidence score (0.0-1.0)",
                        "default": 0.5
                    },
                    "tags": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Tags for categorization"
                    },
                    "expires_in_hours": {
                        "type": "integer",
                        "description": "Hours until expiry (optional)"
                    }
                },
                "required": ["key", "engram_type", "content"]
            }),
        },
        McpTool {
            name: "engram_get".to_string(),
            description: "Get an engram by its key".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "key": {
                        "type": "string",
                        "description": "Engram key"
                    }
                },
                "required": ["key"]
            }),
        },
        McpTool {
            name: "engram_search".to_string(),
            description: "Search engrams with filtering".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "engram_type": {
                        "type": "string",
                        "enum": ["edge_pattern", "avoidance", "strategy", "threat_intel", "consensus_outcome", "trade_result", "market_condition"],
                        "description": "Filter by engram type"
                    },
                    "key_prefix": {
                        "type": "string",
                        "description": "Filter by key prefix (e.g., 'arb.pattern.')"
                    },
                    "tag": {
                        "type": "string",
                        "description": "Filter by tag"
                    },
                    "min_confidence": {
                        "type": "number",
                        "description": "Minimum confidence score"
                    },
                    "limit": {
                        "type": "integer",
                        "default": 50
                    },
                    "offset": {
                        "type": "integer",
                        "default": 0
                    }
                },
                "required": []
            }),
        },
        McpTool {
            name: "engram_find_patterns".to_string(),
            description: "Find matching patterns for an edge type and venue".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "edge_type": {
                        "type": "string",
                        "description": "Type of edge (e.g., 'dex_arb', 'curve_arb')"
                    },
                    "venue_type": {
                        "type": "string",
                        "description": "Venue type (e.g., 'dex_amm', 'bonding_curve')"
                    },
                    "route_signature": {
                        "type": "string",
                        "description": "Optional route signature to match"
                    },
                    "min_success_rate": {
                        "type": "number",
                        "description": "Minimum success rate threshold",
                        "default": 0.5
                    }
                },
                "required": ["edge_type", "venue_type"]
            }),
        },
        McpTool {
            name: "engram_check_avoidance".to_string(),
            description: "Check if an entity should be avoided based on stored avoidance engrams".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "entity_type": {
                        "type": "string",
                        "description": "Type of entity (e.g., 'token', 'wallet', 'contract')"
                    },
                    "address": {
                        "type": "string",
                        "description": "Entity address"
                    }
                },
                "required": ["entity_type", "address"]
            }),
        },
        McpTool {
            name: "engram_create_avoidance".to_string(),
            description: "Create an avoidance engram for a bad actor or risky entity".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "entity_type": {
                        "type": "string",
                        "description": "Type of entity (e.g., 'token', 'wallet', 'contract')"
                    },
                    "address": {
                        "type": "string",
                        "description": "Entity address"
                    },
                    "reason": {
                        "type": "string",
                        "description": "Reason for avoidance"
                    },
                    "category": {
                        "type": "string",
                        "description": "Category (e.g., 'rug_pull', 'honeypot', 'scam')"
                    },
                    "severity": {
                        "type": "string",
                        "enum": ["low", "medium", "high", "critical"],
                        "description": "Severity level"
                    }
                },
                "required": ["entity_type", "address", "reason", "category", "severity"]
            }),
        },
        McpTool {
            name: "engram_create_pattern".to_string(),
            description: "Create an edge pattern engram from successful trade data".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "edge_type": {
                        "type": "string",
                        "description": "Type of edge"
                    },
                    "venue_type": {
                        "type": "string",
                        "description": "Venue type"
                    },
                    "route_signature": {
                        "type": "string",
                        "description": "Route signature (e.g., 'JUP->RAY->ORCA')"
                    },
                    "success_rate": {
                        "type": "number",
                        "description": "Success rate (0.0-1.0)"
                    },
                    "avg_profit_bps": {
                        "type": "number",
                        "description": "Average profit in basis points"
                    },
                    "sample_count": {
                        "type": "integer",
                        "description": "Number of samples used to calculate stats"
                    }
                },
                "required": ["edge_type", "venue_type", "route_signature", "success_rate", "avg_profit_bps", "sample_count"]
            }),
        },
        McpTool {
            name: "engram_delete".to_string(),
            description: "Delete an engram by its key".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "key": {
                        "type": "string",
                        "description": "Engram key to delete"
                    }
                },
                "required": ["key"]
            }),
        },
        McpTool {
            name: "engram_stats".to_string(),
            description: "Get engram harvester statistics".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
    ]
}

pub fn get_consensus_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "consensus_request".to_string(),
            description: "Request multi-LLM consensus on a trade decision".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "edge_id": {
                        "type": "string",
                        "description": "UUID of the edge to evaluate"
                    },
                    "models": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "LLM models to query (defaults to claude-3.5-sonnet, gpt-4-turbo, llama-3.1-70b)"
                    },
                    "min_agreement": {
                        "type": "number",
                        "description": "Minimum agreement threshold (0.0-1.0)",
                        "default": 0.5
                    },
                    "timeout_secs": {
                        "type": "integer",
                        "description": "Timeout in seconds",
                        "default": 30
                    }
                },
                "required": ["edge_id"]
            }),
        },
        McpTool {
            name: "consensus_result".to_string(),
            description: "Get the result of a consensus request".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "consensus_id": {
                        "type": "string",
                        "description": "UUID of the consensus request"
                    }
                },
                "required": ["consensus_id"]
            }),
        },
        McpTool {
            name: "consensus_history".to_string(),
            description: "Get history of consensus decisions".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "edge_id": {
                        "type": "string",
                        "description": "Filter by edge UUID"
                    },
                    "approved_only": {
                        "type": "boolean",
                        "description": "Only show approved decisions"
                    },
                    "limit": {
                        "type": "integer",
                        "default": 50
                    }
                },
                "required": []
            }),
        },
    ]
}

pub fn get_swarm_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "swarm_status".to_string(),
            description: "Get full swarm status including agents and circuit breakers".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        McpTool {
            name: "swarm_health".to_string(),
            description: "Get swarm health summary".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        McpTool {
            name: "swarm_agents".to_string(),
            description: "List all registered agents with health status".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        McpTool {
            name: "swarm_agent_status".to_string(),
            description: "Get status for a specific agent".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "agent_id": {
                        "type": "string",
                        "description": "UUID of the agent"
                    }
                },
                "required": ["agent_id"]
            }),
        },
        McpTool {
            name: "swarm_pause".to_string(),
            description: "Pause all swarm trading activity".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        McpTool {
            name: "swarm_resume".to_string(),
            description: "Resume swarm trading activity".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        McpTool {
            name: "swarm_heartbeat".to_string(),
            description: "Record agent heartbeat".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "agent_id": {
                        "type": "string",
                        "description": "UUID of the agent"
                    }
                },
                "required": ["agent_id"]
            }),
        },
        McpTool {
            name: "swarm_report_failure".to_string(),
            description: "Report agent failure".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "agent_id": {
                        "type": "string",
                        "description": "UUID of the agent"
                    },
                    "error": {
                        "type": "string",
                        "description": "Error message"
                    }
                },
                "required": ["agent_id", "error"]
            }),
        },
        McpTool {
            name: "circuit_breakers_list".to_string(),
            description: "List all circuit breakers and their states".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        McpTool {
            name: "circuit_breaker_reset".to_string(),
            description: "Reset a specific circuit breaker".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Name of the circuit breaker"
                    }
                },
                "required": ["name"]
            }),
        },
        McpTool {
            name: "circuit_breakers_reset_all".to_string(),
            description: "Reset all circuit breakers".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
    ]
}

pub fn get_all_tools() -> Vec<McpTool> {
    let mut tools = Vec::new();
    tools.extend(get_scanner_tools());
    tools.extend(get_edge_tools());
    tools.extend(get_strategy_tools());
    tools.extend(get_curve_tools());
    tools.extend(get_research_tools());
    tools.extend(get_kol_tools());
    tools.extend(get_threat_tools());
    tools.extend(get_event_tools());
    tools.extend(get_engram_tools());
    tools.extend(get_consensus_tools());
    tools.extend(get_swarm_tools());
    tools
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub tools: Vec<McpTool>,
}

pub fn get_manifest() -> McpToolManifest {
    McpToolManifest {
        name: "arb-farm".to_string(),
        version: "0.1.0".to_string(),
        description: "ArbFarm MEV Agent Swarm - Solana arbitrage and MEV opportunity detection".to_string(),
        tools: get_all_tools(),
    }
}
