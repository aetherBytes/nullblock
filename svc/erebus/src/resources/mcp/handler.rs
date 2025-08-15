use super::types::{McpRequest, McpResponse, McpServerInfo, McpCapabilities};
use super::worker::McpWorkerFactory;

#[derive(Clone)]
pub struct McpHandler {
    worker_factory: std::sync::Arc<McpWorkerFactory>,
}

impl McpHandler {
    pub fn new() -> Self {
        Self {
            worker_factory: std::sync::Arc::new(McpWorkerFactory::new()),
        }
    }

    /// Handle MCP protocol requests
    pub fn handle_request(&self, request: McpRequest) -> McpResponse {
        match request.method.as_str() {
            "ping" => self.handle_ping(&request.params),
            "initialize" => self.handle_initialize(&request.params),
            "resources/list" => self.handle_list_resources(&request.params),
            "tools/list" => self.handle_list_tools(&request.params),
            "tools/call" => self.handle_tool_call(&request.params),
            "prompts/list" => self.handle_list_prompts(&request.params),
            "prompts/get" => self.handle_get_prompt(&request.params),
            "resources/read" => self.handle_read_resource(&request.params),
            "wallets/list" => self.handle_wallet_integration(&request.params),
            "social_trading/analyze" => self.handle_social_trading(&request.params),
            "arbitrage/execute" => self.handle_arbitrage(&request.params),
            "worker/status" => self.handle_worker_status(&request.params),
            "worker/list" => self.handle_worker_list(&request.params),
            "worker/stats" => self.handle_worker_stats(&request.params),
            _ => McpResponse::method_not_found(),
        }
    }

    fn handle_ping(&self, params: &Option<serde_json::Value>) -> McpResponse {
        let empty_json = serde_json::json!({});
        let ping_data = params.as_ref().and_then(|p| p.get("data")).unwrap_or(&empty_json);
        println!("🏓 MCP ping received with data: {}", ping_data);
        McpResponse::success(serde_json::json!({
            "message": "pong",
            "timestamp": chrono::Utc::now().timestamp(),
            "echo": ping_data
        }))
    }

    fn handle_initialize(&self, params: &Option<serde_json::Value>) -> McpResponse {
        let default_client = serde_json::json!({"name": "unknown", "version": "unknown"});
        let client_info = params.as_ref()
            .and_then(|p| p.get("clientInfo"))
            .unwrap_or(&default_client);
        
        println!("🚀 MCP initialization requested from client: {}", client_info);
        let server_info = McpServerInfo {
            name: "erebus".to_string(),
            version: "0.1.0".to_string(),
            protocol_version: "2024-11-05".to_string(),
            capabilities: McpCapabilities {
                resources: vec![
                    "wallets".to_string(),
                    "sessions".to_string(),
                    "trading_agents".to_string(),
                    "social_signals".to_string(),
                ],
                tools: vec![
                    "wallet_challenge".to_string(),
                    "wallet_verify".to_string(),
                    "session_validate".to_string(),
                    "social_trading_analyze".to_string(),
                    "arbitrage_execute".to_string(),
                ],
                prompts: vec![
                    "wallet_connection_flow".to_string(),
                    "trading_strategy".to_string(),
                    "risk_assessment".to_string(),
                ],
            },
        };
        McpResponse::success(serde_json::to_value(server_info).unwrap())
    }

    fn handle_list_resources(&self, params: &Option<serde_json::Value>) -> McpResponse {
        let filter = params.as_ref().and_then(|p| p.get("filter")).and_then(|f| f.as_str());
        println!("📋 MCP resources list requested with filter: {:?}", filter);
        
        let mut resources = vec![
            serde_json::json!({
                "uri": "erebus://wallets",
                "name": "Wallet Management",
                "description": "Multi-wallet authentication and session management",
                "mimeType": "application/json",
                "category": "authentication"
            }),
            serde_json::json!({
                "uri": "erebus://sessions", 
                "name": "Session Management",
                "description": "Active wallet session validation and cleanup",
                "mimeType": "application/json",
                "category": "authentication"
            }),
            serde_json::json!({
                "uri": "erebus://trading_agents",
                "name": "Trading Agents",
                "description": "Social trading and arbitrage agent workflows",
                "mimeType": "application/json",
                "category": "trading"
            }),
        ];

        // Apply filter if provided
        if let Some(category) = filter {
            resources.retain(|r| r.get("category").and_then(|c| c.as_str()) == Some(category));
        }

        McpResponse::success(serde_json::json!(resources))
    }

    fn handle_list_tools(&self, params: &Option<serde_json::Value>) -> McpResponse {
        let category = params.as_ref().and_then(|p| p.get("category")).and_then(|c| c.as_str());
        println!("🔧 MCP tools list requested for category: {:?}", category);
        
        let mut tools = vec![
            serde_json::json!({
                "name": "wallet_challenge",
                "description": "Create authentication challenge for any supported wallet",
                "category": "wallet",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "wallet_address": {"type": "string"},
                        "wallet_type": {"type": "string", "enum": ["phantom", "metamask"]}
                    },
                    "required": ["wallet_address", "wallet_type"]
                }
            }),
            serde_json::json!({
                "name": "social_trading_analyze",
                "description": "Analyze social signals for token trading decisions",
                "category": "trading",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "token": {"type": "string", "description": "Token symbol (e.g., BONK, SOL)"},
                        "timeframe": {"type": "string", "enum": ["1h", "4h", "24h"], "default": "24h"},
                        "sources": {"type": "array", "items": {"type": "string"}, "default": ["twitter", "telegram"]}
                    },
                    "required": ["token"]
                }
            }),
            serde_json::json!({
                "name": "arbitrage_execute",
                "description": "Execute arbitrage opportunity across DEXes with MEV protection",
                "category": "trading",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "token_pair": {"type": "string", "description": "Trading pair (e.g., SOL/USDC)"},
                        "amount": {"type": "number", "description": "Amount to trade"},
                        "max_slippage": {"type": "number", "default": 0.5, "description": "Maximum slippage %"},
                        "use_mev_protection": {"type": "boolean", "default": true}
                    },
                    "required": ["token_pair", "amount"]
                }
            })
        ];

        // Apply category filter if provided
        if let Some(cat) = category {
            tools.retain(|t| t.get("category").and_then(|c| c.as_str()) == Some(cat));
        }

        McpResponse::success(serde_json::json!(tools))
    }

    fn handle_list_prompts(&self, params: &Option<serde_json::Value>) -> McpResponse {
        let context = params.as_ref().and_then(|p| p.get("context")).and_then(|c| c.as_str());
        println!("💬 MCP prompts list requested for context: {:?}", context);
        
        let prompts = serde_json::json!([
            {
                "name": "wallet_connection_flow",
                "description": "Guide users through secure wallet connection process",
                "context": "authentication",
                "arguments": [
                    {
                        "name": "wallet_type",
                        "description": "Type of wallet to connect (phantom/metamask)",
                        "required": false
                    },
                    {
                        "name": "user_experience_level",
                        "description": "User's Web3 experience level (beginner/intermediate/advanced)",
                        "required": false
                    }
                ]
            },
            {
                "name": "trading_strategy",
                "description": "Generate comprehensive trading strategy based on market conditions",
                "context": "trading",
                "arguments": [
                    {
                        "name": "risk_tolerance",
                        "description": "Risk tolerance level (conservative/moderate/aggressive)",
                        "required": true
                    },
                    {
                        "name": "portfolio_size",
                        "description": "Portfolio size in USD",
                        "required": true
                    }
                ]
            }
        ]);
        McpResponse::success(prompts)
    }

    fn handle_tool_call(&self, params: &Option<serde_json::Value>) -> McpResponse {
        let tool_name = params.as_ref()
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("unknown");
        
        let empty_args = serde_json::json!({});
        let arguments = params.as_ref()
            .and_then(|p| p.get("arguments"))
            .unwrap_or(&empty_args);

        println!("🔧 MCP tool call: {} with args: {}", tool_name, arguments);

        match tool_name {
            "social_trading_analyze" => self.delegate_to_nullblock_mcp("social_trading", arguments),
            "arbitrage_execute" => self.delegate_to_nullblock_mcp("arbitrage", arguments),
            _ => McpResponse::error(format!("Tool '{}' requires delegation to nullblock.mcp", tool_name))
        }
    }

    fn handle_get_prompt(&self, params: &Option<serde_json::Value>) -> McpResponse {
        let prompt_name = params.as_ref()
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("unknown");
        
        let empty_args = serde_json::json!({});
        let arguments = params.as_ref()
            .and_then(|p| p.get("arguments"))
            .unwrap_or(&empty_args);

        println!("💬 MCP prompt requested: {} with args: {}", prompt_name, arguments);

        match prompt_name {
            "wallet_connection_flow" => self.generate_wallet_flow_prompt(arguments),
            "trading_strategy" => self.delegate_to_nullblock_mcp("prompt_trading_strategy", arguments),
            _ => McpResponse::error(format!("Unknown prompt: {}", prompt_name))
        }
    }

    fn handle_read_resource(&self, params: &Option<serde_json::Value>) -> McpResponse {
        let uri = params.as_ref()
            .and_then(|p| p.get("uri"))
            .and_then(|u| u.as_str())
            .unwrap_or("unknown");

        println!("📖 MCP resource read requested: {}", uri);

        match uri {
            "erebus://wallets" => self.get_wallet_resource_data(),
            "erebus://sessions" => self.get_session_resource_data(),
            "erebus://trading_agents" => self.delegate_to_nullblock_mcp("resource_trading_agents", &serde_json::json!({})),
            _ => McpResponse::error(format!("Unknown resource URI: {}", uri))
        }
    }

    fn handle_social_trading(&self, params: &Option<serde_json::Value>) -> McpResponse {
        println!("📱 Social trading analysis requested: {}", params.as_ref().unwrap_or(&serde_json::json!({})));
        self.delegate_to_nullblock_mcp("social_trading", params.as_ref().unwrap_or(&serde_json::json!({})))
    }

    fn handle_arbitrage(&self, params: &Option<serde_json::Value>) -> McpResponse {
        println!("💹 Arbitrage execution requested: {}", params.as_ref().unwrap_or(&serde_json::json!({})));
        self.delegate_to_nullblock_mcp("arbitrage", params.as_ref().unwrap_or(&serde_json::json!({})))
    }

    fn handle_wallet_integration(&self, _params: &Option<serde_json::Value>) -> McpResponse {
        println!("🔗 MCP wallet integration requested");
        let wallet_info = serde_json::json!({
            "supported_wallets": [
                {
                    "id": "phantom",
                    "name": "Phantom", 
                    "description": "Solana Wallet",
                    "icon": "👻",
                    "networks": ["solana-mainnet", "solana-devnet", "solana-testnet"]
                },
                {
                    "id": "metamask",
                    "name": "MetaMask",
                    "description": "Ethereum Wallet", 
                    "icon": "🦊",
                    "networks": ["ethereum", "polygon", "optimism"]
                }
            ],
            "mcp_integration": {
                "tools_available": true,
                "session_management": true,
                "multi_wallet_support": true
            }
        });
        McpResponse::success(wallet_info)
    }

    /// Delegate complex MCP operations to nullblock.mcp service
    fn delegate_to_nullblock_mcp(&self, operation: &str, params: &serde_json::Value) -> McpResponse {
        println!("🔄 Delegating '{}' to nullblock.mcp service with params: {}", operation, params);
        
        // Create worker for the operation
        let worker = self.worker_factory.create_worker(operation, params.clone());
        
        // For now, return worker information - in production this would be async
        // and we'd use tokio::spawn to handle the nullblock.mcp communication
        McpResponse::success(serde_json::json!({
            "status": "worker_created",
            "worker_id": worker.worker_id,
            "operation": operation,
            "params": params,
            "message": format!("Worker created for operation '{}'. Use /mcp/worker/{} to check status.", operation, worker.worker_id),
            "worker_status": format!("{:?}", worker.status),
            "created_at": worker.created_at,
            "check_status_url": format!("/mcp/worker/{}", worker.worker_id),
            "estimated_completion": "30-60 seconds"
        }))
    }

    fn generate_wallet_flow_prompt(&self, arguments: &serde_json::Value) -> McpResponse {
        let wallet_type = arguments.get("wallet_type").and_then(|w| w.as_str()).unwrap_or("any");
        let experience = arguments.get("user_experience_level").and_then(|e| e.as_str()).unwrap_or("intermediate");

        let prompt = match (wallet_type, experience) {
            ("phantom", "beginner") => "I'll guide you through connecting your Phantom wallet step-by-step. First, make sure you have the Phantom browser extension installed...",
            ("metamask", "beginner") => "Let's connect your MetaMask wallet safely. I'll walk you through each step to ensure your security...",
            (_, "advanced") => "Quick wallet connection: Click the lock icon, select your wallet, and sign the authentication challenge.",
            _ => "To connect your wallet: 1) Click the connect button, 2) Choose your wallet type, 3) Sign the authentication message."
        };

        McpResponse::success(serde_json::json!({
            "messages": [
                {
                    "role": "assistant",
                    "content": {
                        "type": "text",
                        "text": prompt
                    }
                }
            ]
        }))
    }

    fn get_wallet_resource_data(&self) -> McpResponse {
        // This would typically integrate with the WalletManager
        McpResponse::success(serde_json::json!({
            "total_supported_wallets": 2,
            "active_sessions": 0,
            "supported_networks": ["solana", "ethereum", "polygon"],
            "security_features": ["challenge_response_auth", "session_expiration", "signature_verification"]
        }))
    }

    fn get_session_resource_data(&self) -> McpResponse {
        McpResponse::success(serde_json::json!({
            "active_sessions": 0,
            "session_timeout_minutes": 30,
            "cleanup_frequency": "every_hour",
            "max_concurrent_sessions": 1000
        }))
    }

    fn handle_worker_status(&self, params: &Option<serde_json::Value>) -> McpResponse {
        let worker_id = params.as_ref()
            .and_then(|p| p.get("worker_id"))
            .and_then(|w| w.as_str())
            .unwrap_or("unknown");

        println!("🔍 MCP worker status requested for: {}", worker_id);

        match self.worker_factory.get_worker_status(worker_id) {
            Some(worker) => McpResponse::success(serde_json::json!({
                "worker_id": worker.worker_id,
                "operation": worker.operation,
                "status": format!("{:?}", worker.status),
                "created_at": worker.created_at,
                "updated_at": worker.updated_at,
                "params": worker.params,
                "result": worker.result,
                "error": worker.error
            })),
            None => McpResponse::error(format!("Worker '{}' not found", worker_id))
        }
    }

    fn handle_worker_list(&self, _params: &Option<serde_json::Value>) -> McpResponse {
        println!("📋 MCP worker list requested");
        let workers = self.worker_factory.get_all_workers();
        
        let worker_summaries: Vec<_> = workers.into_iter().map(|w| serde_json::json!({
            "worker_id": w.worker_id,
            "operation": w.operation,
            "status": format!("{:?}", w.status),
            "created_at": w.created_at,
            "updated_at": w.updated_at
        })).collect();

        McpResponse::success(serde_json::json!({
            "total_workers": worker_summaries.len(),
            "workers": worker_summaries
        }))
    }

    fn handle_worker_stats(&self, _params: &Option<serde_json::Value>) -> McpResponse {
        println!("📊 MCP worker stats requested");
        let stats = self.worker_factory.get_worker_stats();
        McpResponse::success(stats)
    }
}