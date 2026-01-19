use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::mcp::tools::{get_manifest, McpToolResult};
use crate::server::AppState;

#[derive(Debug, Deserialize)]
pub struct ToolCallRequest {
    pub name: String,
    #[serde(default)]
    pub arguments: Value,
}

#[derive(Debug, Serialize)]
pub struct ToolCallResponse {
    pub content: Vec<ContentItem>,
    #[serde(rename = "isError", skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ContentItem {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

impl From<McpToolResult> for ToolCallResponse {
    fn from(result: McpToolResult) -> Self {
        Self {
            content: result.content.into_iter().map(|c| ContentItem {
                content_type: c.content_type,
                text: c.text,
            }).collect(),
            is_error: result.is_error,
        }
    }
}

pub async fn list_tools() -> impl IntoResponse {
    let manifest = get_manifest();
    (StatusCode::OK, Json(manifest))
}

pub async fn call_tool(
    State(state): State<AppState>,
    Json(request): Json<ToolCallRequest>,
) -> impl IntoResponse {
    let result = execute_tool(&state, &request.name, request.arguments).await;
    let response: ToolCallResponse = result.into();

    if response.is_error.unwrap_or(false) {
        (StatusCode::BAD_REQUEST, Json(response))
    } else {
        (StatusCode::OK, Json(response))
    }
}

pub async fn execute_tool(state: &AppState, name: &str, args: Value) -> McpToolResult {
    match name {
        // Scanner tools
        "scanner_status" => scanner_status(state).await,
        "scanner_signals" => scanner_signals(state, args).await,

        // Edge tools
        "edge_list" => edge_list(state, args).await,
        "edge_details" => edge_details(state, args).await,

        // Strategy tools
        "strategy_list" => strategy_list(state).await,
        "strategy_toggle" => strategy_toggle(state, args).await,
        "strategy_kill" => strategy_kill(state, args).await,

        // Curve tools
        "curve_list_tokens" => curve_list_tokens(state, args).await,
        "curve_graduation_candidates" => curve_graduation_candidates(state, args).await,
        "curve_check_progress" => curve_check_progress(state, args).await,
        "curve_venues_health" => curve_venues_health(state).await,

        // KOL tools
        "kol_list" => kol_list(state, args).await,
        "kol_stats" => kol_stats(state, args).await,

        // Swarm tools
        "swarm_status" => swarm_status(state).await,
        "swarm_health" => swarm_health(state).await,

        // Engram tools (via engrams client)
        "engram_search" => engram_search(state, args).await,
        "engram_stats" => engram_stats(state).await,

        // Learning engram tools (A2A tagged)
        "engram_get_arbfarm_learning" => engram_get_arbfarm_learning(state, args).await,
        "engram_acknowledge_recommendation" => engram_acknowledge_recommendation(state, args).await,
        "engram_get_trade_history" => engram_get_trade_history(state, args).await,
        "engram_get_errors" => engram_get_errors(state, args).await,
        "engram_request_analysis" => engram_request_analysis(state, args).await,
        "engram_get_by_ids" => engram_get_by_ids(state, args).await,

        // Position tools
        "position_list" => position_list(state).await,
        "position_details" => position_details(state, args).await,

        // Wallet tools
        "wallet_balance" => wallet_balance(state).await,
        "wallet_status" => wallet_status(state).await,

        // Approval tools
        "approval_list" => approval_list(state).await,
        "approval_details" => approval_details(state, args).await,
        "approval_approve" => approval_approve(state, args).await,
        "approval_reject" => approval_reject(state, args).await,
        "execution_config_get" => execution_config_get(state).await,
        "execution_toggle" => execution_toggle(state, args).await,
        "approval_recommend" => approval_recommend(state, args).await,

        // Consensus tools
        "consensus_request" => consensus_request_mcp(state, args).await,
        "consensus_result" => consensus_result_mcp(state, args).await,
        "consensus_history" => consensus_history_mcp(state, args).await,
        "consensus_stats" => consensus_stats_mcp(state).await,
        "consensus_config_get" => consensus_config_get_mcp(state).await,
        "consensus_config_update" => consensus_config_update_mcp(state, args).await,
        "consensus_models_list" => consensus_models_list_mcp(state).await,
        "consensus_models_discovered" => consensus_models_discovered_mcp(state).await,
        "consensus_learning_summary" => consensus_learning_summary_mcp(state).await,

        _ => McpToolResult::error(format!("Unknown tool: {}", name)),
    }
}

// Scanner tool implementations
async fn scanner_status(state: &AppState) -> McpToolResult {
    let status = state.scanner.get_status().await;

    // Convert to JSON manually since ScannerStatus doesn't implement Serialize
    let status_json = serde_json::json!({
        "id": status.id.to_string(),
        "is_running": status.is_running,
        "scan_interval_ms": status.scan_interval_ms,
        "stats": {
            "total_scans": status.stats.total_scans,
            "total_signals_detected": status.stats.total_signals_detected,
            "last_scan_at": status.stats.last_scan_at.map(|t| t.to_rfc3339())
        },
        "venues": status.venue_statuses.iter().map(|v| serde_json::json!({
            "id": v.id.to_string(),
            "name": v.name,
            "venue_type": format!("{:?}", v.venue_type),
            "is_healthy": v.is_healthy
        })).collect::<Vec<_>>()
    });

    McpToolResult::success(serde_json::to_string_pretty(&status_json).unwrap_or_default())
}

async fn scanner_signals(state: &AppState, args: Value) -> McpToolResult {
    let min_confidence = args.get("min_confidence").and_then(|v| v.as_f64()).unwrap_or(0.5);

    match state.scanner.get_high_confidence_signals(min_confidence).await {
        Ok(signals) => McpToolResult::success(serde_json::to_string_pretty(&signals).unwrap_or_default()),
        Err(e) => McpToolResult::error(format!("Failed to get signals: {}", e)),
    }
}

// Edge tool implementations
async fn edge_list(state: &AppState, args: Value) -> McpToolResult {
    let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(50);
    let offset = args.get("offset").and_then(|v| v.as_i64()).unwrap_or(0);
    let status = args.get("status").and_then(|v| v.as_str());

    match state.edge_repo.list(None, status, limit, offset).await {
        Ok(edges) => McpToolResult::success(serde_json::to_string_pretty(&edges).unwrap_or_default()),
        Err(e) => McpToolResult::error(format!("Failed to list edges: {}", e)),
    }
}

async fn edge_details(state: &AppState, args: Value) -> McpToolResult {
    let edge_id = match args.get("edge_id").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => return McpToolResult::error("edge_id is required"),
    };

    let uuid = match uuid::Uuid::parse_str(edge_id) {
        Ok(id) => id,
        Err(_) => return McpToolResult::error("Invalid edge_id format"),
    };

    match state.edge_repo.get_by_id(uuid).await {
        Ok(Some(edge)) => McpToolResult::success(serde_json::to_string_pretty(&edge).unwrap_or_default()),
        Ok(None) => McpToolResult::error("Edge not found"),
        Err(e) => McpToolResult::error(format!("Failed to get edge: {}", e)),
    }
}

// Strategy tool implementations
async fn strategy_list(state: &AppState) -> McpToolResult {
    let strategies = state.strategy_engine.list_strategies().await;
    McpToolResult::success(serde_json::to_string_pretty(&strategies).unwrap_or_default())
}

async fn strategy_toggle(state: &AppState, args: Value) -> McpToolResult {
    let strategy_id = match args.get("strategy_id").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => return McpToolResult::error("strategy_id is required"),
    };

    let enabled = match args.get("enabled").and_then(|v| v.as_bool()) {
        Some(e) => e,
        None => return McpToolResult::error("enabled is required"),
    };

    let uuid = match uuid::Uuid::parse_str(strategy_id) {
        Ok(id) => id,
        Err(_) => return McpToolResult::error("Invalid strategy_id format"),
    };

    match state.strategy_engine.toggle_strategy(uuid, enabled).await {
        Ok(_) => McpToolResult::success(serde_json::json!({
            "success": true,
            "strategy_id": strategy_id,
            "enabled": enabled
        }).to_string()),
        Err(e) => McpToolResult::error(format!("Failed to toggle strategy: {}", e)),
    }
}

async fn strategy_kill(state: &AppState, args: Value) -> McpToolResult {
    let strategy_id = match args.get("strategy_id").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => return McpToolResult::error("strategy_id is required"),
    };

    let uuid = match uuid::Uuid::parse_str(strategy_id) {
        Ok(id) => id,
        Err(_) => return McpToolResult::error("Invalid strategy_id format"),
    };

    match state.strategy_engine.kill_strategy(uuid).await {
        Ok(strategy_name) => {
            // Also cancel pending approvals
            let _ = state.approval_manager.cancel_by_strategy(uuid).await;

            McpToolResult::success(serde_json::json!({
                "success": true,
                "strategy_id": strategy_id,
                "strategy_name": strategy_name,
                "action": "emergency_stop",
                "message": "Strategy killed - all operations halted"
            }).to_string())
        },
        Err(e) => McpToolResult::error(format!("Failed to kill strategy: {}", e)),
    }
}

// Curve tool implementations
async fn curve_list_tokens(state: &AppState, args: Value) -> McpToolResult {
    let venue = args.get("venue").and_then(|v| v.as_str()).unwrap_or("all");
    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(50) as u32;

    let mut tokens: Vec<serde_json::Value> = Vec::new();

    if venue == "all" || venue == "pump_fun" {
        if let Ok(pump_tokens) = state.pump_fun_venue.get_new_tokens(limit).await {
            for token in pump_tokens {
                tokens.push(serde_json::json!({
                    "mint": token.mint,
                    "symbol": token.symbol,
                    "name": token.name,
                    "market_cap": token.market_cap,
                    "volume_24h": token.volume_24h,
                    "bonding_curve_complete": token.bonding_curve_complete,
                    "venue": "pump_fun"
                }));
            }
        }
    }

    if venue == "all" || venue == "moonshot" {
        if let Ok(moonshot_tokens) = state.moonshot_venue.get_new_tokens(limit).await {
            for token in moonshot_tokens {
                tokens.push(serde_json::json!({
                    "mint": token.mint,
                    "symbol": token.symbol,
                    "name": token.name,
                    "market_cap": token.market_cap_usd,
                    "price_usd": token.price_usd,
                    "volume_24h": token.volume_24h_usd,
                    "bonding_curve_complete": token.is_graduated,
                    "venue": "moonshot"
                }));
            }
        }
    }

    McpToolResult::success(serde_json::to_string_pretty(&tokens).unwrap_or_default())
}

async fn curve_graduation_candidates(state: &AppState, args: Value) -> McpToolResult {
    let min_progress = args.get("min_progress").and_then(|v| v.as_f64()).unwrap_or(50.0);
    let max_progress = args.get("max_progress").and_then(|v| v.as_f64()).unwrap_or(95.0);
    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(20) as usize;

    let mut candidates = Vec::new();

    if let Ok(tokens) = state.pump_fun_venue.get_new_tokens(100).await {
        for token in tokens {
            let progress = (token.market_cap / token.graduation_threshold.unwrap_or(69000.0)) * 100.0;
            if progress >= min_progress && progress <= max_progress {
                candidates.push(serde_json::json!({
                    "mint": token.mint,
                    "symbol": token.symbol,
                    "name": token.name,
                    "progress": progress,
                    "market_cap": token.market_cap,
                    "venue": "pump_fun"
                }));
            }
            if candidates.len() >= limit {
                break;
            }
        }
    }

    McpToolResult::success(serde_json::to_string_pretty(&candidates).unwrap_or_default())
}

async fn curve_check_progress(state: &AppState, args: Value) -> McpToolResult {
    let token_mint = match args.get("token_mint").and_then(|v| v.as_str()) {
        Some(mint) => mint,
        None => return McpToolResult::error("token_mint is required"),
    };
    let venue = args.get("venue").and_then(|v| v.as_str()).unwrap_or("pump_fun");

    match venue {
        "pump_fun" => {
            match state.pump_fun_venue.get_token_info(token_mint).await {
                Ok(info) => McpToolResult::success(serde_json::to_string_pretty(&info).unwrap_or_default()),
                Err(e) => McpToolResult::error(format!("Failed to get token info: {}", e)),
            }
        }
        "moonshot" => {
            match state.moonshot_venue.get_token_info(token_mint).await {
                Ok(info) => McpToolResult::success(serde_json::to_string_pretty(&info).unwrap_or_default()),
                Err(e) => McpToolResult::error(format!("Failed to get token info: {}", e)),
            }
        }
        _ => McpToolResult::error("Invalid venue. Use 'pump_fun' or 'moonshot'"),
    }
}

async fn curve_venues_health(state: &AppState) -> McpToolResult {
    let pump_health = state.pump_fun_venue.get_new_tokens(1).await;
    let moonshot_health = state.moonshot_venue.get_new_tokens(1).await;

    let health = serde_json::json!({
        "pump_fun": {
            "healthy": pump_health.is_ok(),
            "error": pump_health.err().map(|e| e.to_string())
        },
        "moonshot": {
            "healthy": moonshot_health.is_ok(),
            "error": moonshot_health.err().map(|e| e.to_string())
        }
    });

    McpToolResult::success(health.to_string())
}

// KOL tool implementations
async fn kol_list(state: &AppState, args: Value) -> McpToolResult {
    let limit = args.get("limit").and_then(|v| v.as_u64()).map(|l| l as usize);
    let kols = state.kol_discovery.get_discovered_kols(limit).await;
    McpToolResult::success(serde_json::to_string_pretty(&kols).unwrap_or_default())
}

async fn kol_stats(state: &AppState, _args: Value) -> McpToolResult {
    let stats = state.kol_discovery.get_stats().await;
    McpToolResult::success(serde_json::to_string_pretty(&stats).unwrap_or_default())
}

// Swarm tool implementations
async fn swarm_status(state: &AppState) -> McpToolResult {
    let scanner_status = state.scanner.get_status().await;
    let strategies = state.strategy_engine.list_strategies().await;
    let positions = state.position_manager.get_open_positions().await;

    let status = serde_json::json!({
        "scanner": {
            "id": scanner_status.id.to_string(),
            "is_running": scanner_status.is_running,
            "scan_interval_ms": scanner_status.scan_interval_ms,
            "total_scans": scanner_status.stats.total_scans,
            "signals_detected": scanner_status.stats.total_signals_detected,
            "venue_count": scanner_status.venue_statuses.len()
        },
        "strategies": {
            "total": strategies.len(),
            "active": strategies.iter().filter(|s| s.is_active).count()
        },
        "positions": {
            "open": positions.len()
        },
        "consensus_engine": "ready"
    });

    McpToolResult::success(status.to_string())
}

async fn swarm_health(state: &AppState) -> McpToolResult {
    let scanner_status = state.scanner.get_status().await;
    let venues_healthy = scanner_status.venue_statuses.iter().all(|v| v.is_healthy);

    let health = serde_json::json!({
        "status": "healthy",
        "scanner": {
            "running": scanner_status.is_running,
            "venues_healthy": venues_healthy
        },
        "database": "connected",
        "helius": state.helius_rpc_client.is_configured(),
        "engrams": state.engrams_client.is_configured()
    });

    McpToolResult::success(health.to_string())
}

// Engram tool implementations (via engrams client)
async fn engram_search(state: &AppState, args: Value) -> McpToolResult {
    if !state.engrams_client.is_configured() {
        return McpToolResult::error("Engrams service not configured");
    }

    let wallet = state.dev_signer.get_address().unwrap_or_default();
    let engram_type = args.get("engram_type").and_then(|v| v.as_str());

    match engram_type {
        Some("pattern") => {
            match state.engrams_client.get_patterns(&wallet).await {
                Ok(patterns) => McpToolResult::success(serde_json::to_string_pretty(&patterns).unwrap_or_default()),
                Err(e) => McpToolResult::error(format!("Failed to search patterns: {}", e)),
            }
        }
        Some("avoidance") => {
            match state.engrams_client.get_avoidances(&wallet).await {
                Ok(avoidances) => McpToolResult::success(serde_json::to_string_pretty(&avoidances).unwrap_or_default()),
                Err(e) => McpToolResult::error(format!("Failed to search avoidances: {}", e)),
            }
        }
        Some("strategy") => {
            match state.engrams_client.get_saved_strategies(&wallet).await {
                Ok(strategies) => McpToolResult::success(serde_json::to_string_pretty(&strategies).unwrap_or_default()),
                Err(e) => McpToolResult::error(format!("Failed to search strategies: {}", e)),
            }
        }
        Some("kol") => {
            match state.engrams_client.get_discovered_kols(&wallet).await {
                Ok(kols) => McpToolResult::success(serde_json::to_string_pretty(&kols).unwrap_or_default()),
                Err(e) => McpToolResult::error(format!("Failed to search KOLs: {}", e)),
            }
        }
        _ => {
            match state.engrams_client.get_engrams_by_wallet(&wallet).await {
                Ok(engrams) => McpToolResult::success(serde_json::to_string_pretty(&engrams).unwrap_or_default()),
                Err(e) => McpToolResult::error(format!("Failed to search engrams: {}", e)),
            }
        }
    }
}

async fn engram_stats(state: &AppState) -> McpToolResult {
    if !state.engrams_client.is_configured() {
        return McpToolResult::error("Engrams service not configured");
    }

    let wallet = state.dev_signer.get_address().unwrap_or_default();

    let patterns = state.engrams_client.get_patterns(&wallet).await.map(|p| p.len()).unwrap_or(0);
    let avoidances = state.engrams_client.get_avoidances(&wallet).await.map(|a| a.len()).unwrap_or(0);
    let strategies = state.engrams_client.get_saved_strategies(&wallet).await.map(|s| s.len()).unwrap_or(0);
    let kols = state.engrams_client.get_discovered_kols(&wallet).await.map(|k| k.len()).unwrap_or(0);

    let stats = serde_json::json!({
        "wallet": wallet,
        "patterns": patterns,
        "avoidances": avoidances,
        "strategies": strategies,
        "discovered_kols": kols,
        "total": patterns + avoidances + strategies + kols
    });

    McpToolResult::success(stats.to_string())
}

// Position tool implementations
async fn position_list(state: &AppState) -> McpToolResult {
    let positions = state.position_manager.get_open_positions().await;
    McpToolResult::success(serde_json::to_string_pretty(&positions).unwrap_or_default())
}

async fn position_details(state: &AppState, args: Value) -> McpToolResult {
    let position_id = match args.get("position_id").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => return McpToolResult::error("position_id is required"),
    };

    let uuid = match uuid::Uuid::parse_str(position_id) {
        Ok(id) => id,
        Err(_) => return McpToolResult::error("Invalid position_id format"),
    };

    match state.position_manager.get_position(uuid).await {
        Some(position) => McpToolResult::success(serde_json::to_string_pretty(&position).unwrap_or_default()),
        None => McpToolResult::error("Position not found"),
    }
}

// Wallet tool implementations
async fn wallet_balance(state: &AppState) -> McpToolResult {
    let wallet_address = match state.dev_signer.get_address() {
        Some(addr) => addr,
        None => return McpToolResult::error("No wallet configured"),
    };

    #[derive(serde::Deserialize)]
    struct BalanceResult {
        value: u64,
    }

    let params = serde_json::json!([wallet_address]);

    match state.helius_rpc_client.rpc_call::<BalanceResult>("getBalance", params).await {
        Ok(result) => {
            McpToolResult::success(serde_json::json!({
                "address": wallet_address,
                "balance_lamports": result.value,
                "balance_sol": result.value as f64 / 1_000_000_000.0
            }).to_string())
        }
        Err(e) => McpToolResult::error(format!("Failed to get balance: {}", e)),
    }
}

async fn wallet_status(state: &AppState) -> McpToolResult {
    let dev_configured = state.dev_signer.is_configured();
    let dev_address = state.dev_signer.get_address();
    let turnkey_status = state.turnkey_signer.get_status().await;

    let status = serde_json::json!({
        "dev_wallet": {
            "configured": dev_configured,
            "address": dev_address
        },
        "turnkey": {
            "connected": turnkey_status.is_connected,
            "wallet_address": turnkey_status.wallet_address,
            "delegation_status": format!("{:?}", turnkey_status.delegation_status)
        }
    });

    McpToolResult::success(status.to_string())
}

// Approval tool implementations
async fn approval_list(state: &AppState) -> McpToolResult {
    let approvals = state.approval_manager.list_pending().await;

    let response = serde_json::json!({
        "pending_count": approvals.len(),
        "approvals": approvals.iter().map(|a| serde_json::json!({
            "id": a.id.to_string(),
            "approval_type": format!("{}", a.approval_type),
            "status": format!("{}", a.status),
            "edge_id": a.edge_id.map(|id| id.to_string()),
            "position_id": a.position_id.map(|id| id.to_string()),
            "token_symbol": a.token_symbol,
            "amount_sol": a.amount_sol,
            "estimated_profit_lamports": a.estimated_profit_lamports,
            "risk_score": a.risk_score,
            "expires_at": a.expires_at.to_rfc3339(),
            "time_remaining_secs": a.time_remaining_secs(),
            "hecate_decision": a.hecate_decision,
            "hecate_confidence": a.hecate_confidence,
            "created_at": a.created_at.to_rfc3339()
        })).collect::<Vec<_>>()
    });

    McpToolResult::success(serde_json::to_string_pretty(&response).unwrap_or_default())
}

async fn approval_details(state: &AppState, args: Value) -> McpToolResult {
    let approval_id = match args.get("approval_id").and_then(|v| v.as_str()) {
        Some(id) => match uuid::Uuid::parse_str(id) {
            Ok(uuid) => uuid,
            Err(_) => return McpToolResult::error("Invalid approval_id format"),
        },
        None => return McpToolResult::error("approval_id is required"),
    };

    match state.approval_manager.get_approval(approval_id).await {
        Some(approval) => {
            let response = serde_json::json!({
                "id": approval.id.to_string(),
                "approval_type": format!("{}", approval.approval_type),
                "status": format!("{}", approval.status),
                "edge_id": approval.edge_id.map(|id| id.to_string()),
                "position_id": approval.position_id.map(|id| id.to_string()),
                "strategy_id": approval.strategy_id.map(|id| id.to_string()),
                "token_mint": approval.token_mint,
                "token_symbol": approval.token_symbol,
                "amount_sol": approval.amount_sol,
                "estimated_profit_lamports": approval.estimated_profit_lamports,
                "risk_score": approval.risk_score,
                "context": approval.context,
                "expires_at": approval.expires_at.to_rfc3339(),
                "time_remaining_secs": approval.time_remaining_secs(),
                "is_expired": approval.is_expired(),
                "hecate_decision": approval.hecate_decision,
                "hecate_reasoning": approval.hecate_reasoning,
                "hecate_confidence": approval.hecate_confidence,
                "user_decision": approval.user_decision,
                "user_decided_at": approval.user_decided_at.map(|t| t.to_rfc3339()),
                "created_at": approval.created_at.to_rfc3339()
            });
            McpToolResult::success(serde_json::to_string_pretty(&response).unwrap_or_default())
        }
        None => McpToolResult::error(format!("Approval {} not found", approval_id)),
    }
}

async fn approval_approve(state: &AppState, args: Value) -> McpToolResult {
    let approval_id = match args.get("approval_id").and_then(|v| v.as_str()) {
        Some(id) => match uuid::Uuid::parse_str(id) {
            Ok(uuid) => uuid,
            Err(_) => return McpToolResult::error("Invalid approval_id format"),
        },
        None => return McpToolResult::error("approval_id is required"),
    };

    let notes = args.get("notes").and_then(|v| v.as_str()).map(String::from);

    match state.approval_manager.approve(approval_id, notes).await {
        Ok(approval) => McpToolResult::success(serde_json::json!({
            "success": true,
            "approval_id": approval.id.to_string(),
            "status": format!("{}", approval.status),
            "message": "Approval approved successfully"
        }).to_string()),
        Err(e) => McpToolResult::error(format!("Failed to approve: {}", e)),
    }
}

async fn approval_reject(state: &AppState, args: Value) -> McpToolResult {
    let approval_id = match args.get("approval_id").and_then(|v| v.as_str()) {
        Some(id) => match uuid::Uuid::parse_str(id) {
            Ok(uuid) => uuid,
            Err(_) => return McpToolResult::error("Invalid approval_id format"),
        },
        None => return McpToolResult::error("approval_id is required"),
    };

    let reason = match args.get("reason").and_then(|v| v.as_str()) {
        Some(r) => r.to_string(),
        None => return McpToolResult::error("reason is required"),
    };

    match state.approval_manager.reject(approval_id, reason).await {
        Ok(approval) => McpToolResult::success(serde_json::json!({
            "success": true,
            "approval_id": approval.id.to_string(),
            "status": format!("{}", approval.status),
            "message": "Approval rejected successfully"
        }).to_string()),
        Err(e) => McpToolResult::error(format!("Failed to reject: {}", e)),
    }
}

async fn execution_config_get(state: &AppState) -> McpToolResult {
    let config = state.approval_manager.get_config().await;

    let response = serde_json::json!({
        "auto_execution_enabled": config.auto_execution_enabled,
        "default_approval_timeout_secs": config.default_approval_timeout_secs,
        "notify_hecate_on_pending": config.notify_hecate_on_pending,
        "require_hecate_approval": config.require_hecate_approval,
        "max_pending_approvals": config.max_pending_approvals,
        "auto_approve_atomic": config.auto_approve_atomic,
        "auto_approve_min_profit_bps": config.auto_approve_min_profit_bps,
        "auto_approve_max_risk_score": config.auto_approve_max_risk_score,
        "emergency_exit_enabled": config.emergency_exit_enabled,
        "updated_at": config.updated_at.to_rfc3339()
    });

    McpToolResult::success(serde_json::to_string_pretty(&response).unwrap_or_default())
}

async fn execution_toggle(state: &AppState, args: Value) -> McpToolResult {
    let enabled = match args.get("enabled").and_then(|v| v.as_bool()) {
        Some(e) => e,
        None => return McpToolResult::error("enabled is required"),
    };

    let config = state.approval_manager.toggle_execution(enabled).await;

    McpToolResult::success(serde_json::json!({
        "success": true,
        "auto_execution_enabled": config.auto_execution_enabled,
        "message": if config.auto_execution_enabled {
            "Auto-execution enabled"
        } else {
            "Auto-execution disabled"
        }
    }).to_string())
}

async fn approval_recommend(state: &AppState, args: Value) -> McpToolResult {
    let approval_id = match args.get("approval_id").and_then(|v| v.as_str()) {
        Some(id) => match uuid::Uuid::parse_str(id) {
            Ok(uuid) => uuid,
            Err(_) => return McpToolResult::error("Invalid approval_id format"),
        },
        None => return McpToolResult::error("approval_id is required"),
    };

    let decision = match args.get("decision").and_then(|v| v.as_bool()) {
        Some(d) => d,
        None => return McpToolResult::error("decision is required"),
    };

    let reasoning = match args.get("reasoning").and_then(|v| v.as_str()) {
        Some(r) => r.to_string(),
        None => return McpToolResult::error("reasoning is required"),
    };

    let confidence = args.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.5);

    let recommendation = crate::models::HecateRecommendation {
        approval_id,
        decision,
        reasoning,
        confidence,
    };

    match state.approval_manager.add_hecate_recommendation(recommendation).await {
        Ok(approval) => McpToolResult::success(serde_json::json!({
            "success": true,
            "approval_id": approval.id.to_string(),
            "hecate_decision": approval.hecate_decision,
            "hecate_confidence": approval.hecate_confidence,
            "message": "Recommendation added successfully"
        }).to_string()),
        Err(e) => McpToolResult::error(format!("Failed to add recommendation: {}", e)),
    }
}

// Learning engram tool implementations (A2A tagged)
async fn engram_get_arbfarm_learning(state: &AppState, args: Value) -> McpToolResult {
    if !state.engrams_client.is_configured() {
        return McpToolResult::error("Engrams service not configured");
    }

    let wallet = state.dev_signer.get_address().unwrap_or_default();
    let category = args.get("category").and_then(|v| v.as_str()).unwrap_or("all");
    let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(20);
    let status_filter = args.get("status").and_then(|v| v.as_str()).unwrap_or("all");

    let mut result = serde_json::json!({
        "wallet": wallet,
        "category": category,
        "engrams": {}
    });

    match category {
        "recommendations" | "all" => {
            let status = match status_filter {
                "pending" => Some(crate::engrams::schemas::RecommendationStatus::Pending),
                "acknowledged" => Some(crate::engrams::schemas::RecommendationStatus::Acknowledged),
                "applied" => Some(crate::engrams::schemas::RecommendationStatus::Applied),
                "rejected" => Some(crate::engrams::schemas::RecommendationStatus::Rejected),
                _ => None,
            };

            match state.engrams_client.get_recommendations_with_metadata(&wallet, status.as_ref(), Some(limit)).await {
                Ok(recs) => {
                    result["engrams"]["recommendations"] = serde_json::to_value(&recs).unwrap_or_default();
                    result["engrams"]["recommendations_count"] = serde_json::json!(recs.len());
                }
                Err(e) => {
                    result["engrams"]["recommendations_error"] = serde_json::json!(e);
                }
            }
        }
        _ => {}
    }

    match category {
        "conversations" | "all" => {
            match state.engrams_client.get_conversations(&wallet, Some(limit)).await {
                Ok(convos) => {
                    result["engrams"]["conversations"] = serde_json::to_value(&convos).unwrap_or_default();
                    result["engrams"]["conversations_count"] = serde_json::json!(convos.len());
                }
                Err(e) => {
                    result["engrams"]["conversations_error"] = serde_json::json!(e);
                }
            }
        }
        _ => {}
    }

    match category {
        "patterns" | "all" => {
            match state.engrams_client.get_patterns(&wallet).await {
                Ok(patterns) => {
                    let limited: Vec<_> = patterns.into_iter().take(limit as usize).collect();
                    result["engrams"]["patterns"] = serde_json::to_value(&limited).unwrap_or_default();
                    result["engrams"]["patterns_count"] = serde_json::json!(limited.len());
                }
                Err(e) => {
                    result["engrams"]["patterns_error"] = serde_json::json!(e);
                }
            }
        }
        _ => {}
    }

    McpToolResult::success(serde_json::to_string_pretty(&result).unwrap_or_default())
}

async fn engram_acknowledge_recommendation(state: &AppState, args: Value) -> McpToolResult {
    if !state.engrams_client.is_configured() {
        return McpToolResult::error("Engrams service not configured");
    }

    let recommendation_id = match args.get("recommendation_id").and_then(|v| v.as_str()) {
        Some(id) => match uuid::Uuid::parse_str(id) {
            Ok(uuid) => uuid,
            Err(_) => return McpToolResult::error("Invalid recommendation_id format"),
        },
        None => return McpToolResult::error("recommendation_id is required"),
    };

    let status_str = match args.get("status").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return McpToolResult::error("status is required"),
    };

    let new_status = match status_str {
        "acknowledged" => crate::engrams::schemas::RecommendationStatus::Acknowledged,
        "applied" => crate::engrams::schemas::RecommendationStatus::Applied,
        "rejected" => crate::engrams::schemas::RecommendationStatus::Rejected,
        _ => return McpToolResult::error("Invalid status. Must be 'acknowledged', 'applied', or 'rejected'"),
    };

    let wallet = state.dev_signer.get_address().unwrap_or_default();

    match state.engrams_client.update_recommendation_status(&wallet, &recommendation_id, new_status.clone()).await {
        Ok(_) => McpToolResult::success(serde_json::json!({
            "success": true,
            "recommendation_id": recommendation_id.to_string(),
            "new_status": status_str,
            "message": format!("Recommendation status updated to {}", status_str)
        }).to_string()),
        Err(e) => McpToolResult::error(format!("Failed to update recommendation status: {}", e)),
    }
}

async fn engram_get_trade_history(state: &AppState, args: Value) -> McpToolResult {
    if !state.engrams_client.is_configured() {
        return McpToolResult::error("Engrams service not configured");
    }

    let wallet = state.dev_signer.get_address().unwrap_or_default();
    let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(50);
    let profitable_only = args.get("profitable_only").and_then(|v| v.as_bool()).unwrap_or(false);

    match state.engrams_client.get_trade_history_with_metadata(&wallet, Some(limit)).await {
        Ok(trades) => {
            let filtered: Vec<_> = if profitable_only {
                trades.into_iter().filter(|t| t.trade.pnl_sol.unwrap_or(0.0) > 0.0).collect()
            } else {
                trades
            };

            let total_pnl: f64 = filtered.iter().filter_map(|t| t.trade.pnl_sol).sum();
            let winning_trades = filtered.iter().filter(|t| t.trade.pnl_sol.unwrap_or(0.0) > 0.0).count();
            let win_rate = if !filtered.is_empty() {
                (winning_trades as f64 / filtered.len() as f64) * 100.0
            } else {
                0.0
            };

            let result = serde_json::json!({
                "wallet": wallet,
                "trade_count": filtered.len(),
                "total_pnl_sol": total_pnl,
                "winning_trades": winning_trades,
                "win_rate_percent": win_rate,
                "trades": filtered
            });

            McpToolResult::success(serde_json::to_string_pretty(&result).unwrap_or_default())
        }
        Err(e) => McpToolResult::error(format!("Failed to get trade history: {}", e)),
    }
}

async fn engram_get_errors(state: &AppState, args: Value) -> McpToolResult {
    if !state.engrams_client.is_configured() {
        return McpToolResult::error("Engrams service not configured");
    }

    let wallet = state.dev_signer.get_address().unwrap_or_default();
    let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(50);
    let error_type_filter = args.get("error_type").and_then(|v| v.as_str());

    match state.engrams_client.get_error_history_with_metadata(&wallet, Some(limit)).await {
        Ok(errors) => {
            let filtered: Vec<_> = if let Some(et) = error_type_filter {
                errors.into_iter().filter(|e| {
                    let type_str = serde_json::to_string(&e.error.error_type)
                        .unwrap_or_default()
                        .trim_matches('"')
                        .to_lowercase();
                    type_str == et.to_lowercase()
                }).collect()
            } else {
                errors
            };

            let recoverable_count = filtered.iter().filter(|e| e.error.recoverable).count();
            let fatal_count = filtered.len() - recoverable_count;

            let mut by_type: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
            for error in &filtered {
                let type_str = serde_json::to_string(&error.error.error_type)
                    .unwrap_or_else(|_| "unknown".to_string())
                    .trim_matches('"')
                    .to_string();
                *by_type.entry(type_str).or_insert(0) += 1;
            }

            let result = serde_json::json!({
                "wallet": wallet,
                "error_count": filtered.len(),
                "recoverable_count": recoverable_count,
                "fatal_count": fatal_count,
                "by_type": by_type,
                "errors": filtered
            });

            McpToolResult::success(serde_json::to_string_pretty(&result).unwrap_or_default())
        }
        Err(e) => McpToolResult::error(format!("Failed to get error history: {}", e)),
    }
}

async fn engram_request_analysis(state: &AppState, args: Value) -> McpToolResult {
    if !state.engrams_client.is_configured() {
        return McpToolResult::error("Engrams service not configured");
    }

    let analysis_type = args.get("analysis_type").and_then(|v| v.as_str()).unwrap_or("trade_review");
    let time_period = args.get("time_period").and_then(|v| v.as_str()).unwrap_or("24h");
    let wallet = state.dev_signer.get_address().unwrap_or_default();

    // Gather data for analysis based on time period
    let limit = match time_period {
        "24h" => 50,
        "7d" => 200,
        "30d" => 500,
        _ => 50,
    };

    // Get trade history and errors for context
    let trades = state.engrams_client.get_trade_history(&wallet, Some(limit)).await.unwrap_or_default();
    let errors = state.engrams_client.get_error_history(&wallet, Some(limit / 2)).await.unwrap_or_default();

    // Calculate summary stats
    let total_trades = trades.len();
    let winning_trades = trades.iter().filter(|t| t.pnl_sol.unwrap_or(0.0) > 0.0).count();
    let total_pnl: f64 = trades.iter().filter_map(|t| t.pnl_sol).sum();
    let win_rate = if total_trades > 0 { (winning_trades as f64 / total_trades as f64) * 100.0 } else { 0.0 };

    // Check if consensus engine is available
    let consensus_available = state.consensus_engine.is_ready().await;

    let result = if consensus_available {
        // Trigger consensus analysis (async, won't block)
        let topic = match analysis_type {
            "risk_assessment" => crate::engrams::schemas::ConversationTopic::RiskAssessment,
            "strategy_optimization" => crate::engrams::schemas::ConversationTopic::StrategyReview,
            "pattern_discovery" => crate::engrams::schemas::ConversationTopic::PatternDiscovery,
            _ => crate::engrams::schemas::ConversationTopic::TradeAnalysis,
        };

        // Queue analysis request
        if let Err(e) = state.consensus_engine.request_analysis(topic, time_period.to_string()).await {
            return McpToolResult::error(format!("Failed to request analysis: {}", e));
        }

        serde_json::json!({
            "success": true,
            "message": format!("Analysis requested for {} over {}", analysis_type, time_period),
            "analysis_type": analysis_type,
            "time_period": time_period,
            "consensus_status": "analysis_queued",
            "data_summary": {
                "trades_in_scope": total_trades,
                "winning_trades": winning_trades,
                "win_rate_percent": win_rate,
                "total_pnl_sol": total_pnl,
                "errors_in_scope": errors.len()
            },
            "note": "Analysis results will be saved as recommendation engrams. Check back using engram_get_arbfarm_learning with category='recommendations'"
        })
    } else {
        serde_json::json!({
            "success": false,
            "message": "Consensus engine not available",
            "analysis_type": analysis_type,
            "time_period": time_period,
            "data_summary": {
                "trades_in_scope": total_trades,
                "winning_trades": winning_trades,
                "win_rate_percent": win_rate,
                "total_pnl_sol": total_pnl,
                "errors_in_scope": errors.len()
            },
            "note": "Manual analysis data provided above. Consensus LLM analysis unavailable."
        })
    };

    McpToolResult::success(serde_json::to_string_pretty(&result).unwrap_or_default())
}

async fn engram_get_by_ids(state: &AppState, args: Value) -> McpToolResult {
    if !state.engrams_client.is_configured() {
        return McpToolResult::error("Engrams service not configured");
    }

    let wallet = state.dev_signer.get_address().unwrap_or_default();

    let engram_ids: Vec<String> = match args.get("engram_ids") {
        Some(ids) => {
            match serde_json::from_value(ids.clone()) {
                Ok(v) => v,
                Err(_) => return McpToolResult::error("engram_ids must be an array of strings"),
            }
        }
        None => return McpToolResult::error("engram_ids is required"),
    };

    if engram_ids.is_empty() {
        return McpToolResult::error("engram_ids array cannot be empty");
    }

    match state.engrams_client.get_engrams_by_ids(&wallet, &engram_ids).await {
        Ok(engrams) => {
            let result = serde_json::json!({
                "wallet": wallet,
                "requested_count": engram_ids.len(),
                "found_count": engrams.len(),
                "engrams": engrams.iter().map(|e| serde_json::json!({
                    "engram_id": e.id.to_string(),
                    "engram_key": e.key,
                    "engram_type": format!("{:?}", e.engram_type),
                    "tags": e.tags,
                    "content": e.content,
                    "created_at": e.created_at,
                    "updated_at": e.updated_at,
                    "version": e.version
                })).collect::<Vec<_>>()
            });
            McpToolResult::success(serde_json::to_string_pretty(&result).unwrap_or_default())
        }
        Err(e) => McpToolResult::error(format!("Failed to get engrams by IDs: {}", e)),
    }
}

// Consensus MCP tool implementations
async fn consensus_request_mcp(state: &AppState, args: Value) -> McpToolResult {
    if state.config.openrouter_api_key.is_none() {
        return McpToolResult::error("Consensus engine not configured - OpenRouter API key missing");
    }

    let edge_type = args.get("edge_type").and_then(|v| v.as_str()).unwrap_or("unknown");
    let venue = args.get("venue").and_then(|v| v.as_str()).unwrap_or("unknown");
    let token_pair: Vec<String> = args.get("token_pair")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();
    let estimated_profit_lamports = args.get("estimated_profit_lamports").and_then(|v| v.as_i64()).unwrap_or(0);
    let risk_score = args.get("risk_score").and_then(|v| v.as_i64()).unwrap_or(50) as i32;
    let route_data = args.get("route_data").cloned().unwrap_or_else(|| serde_json::json!({}));
    let models: Option<Vec<String>> = args.get("models").and_then(|v| serde_json::from_value(v.clone()).ok());

    let edge_id = args.get("edge_id")
        .and_then(|v| v.as_str())
        .and_then(|s| uuid::Uuid::parse_str(s).ok())
        .unwrap_or_else(uuid::Uuid::new_v4);

    let edge_context = crate::consensus::format_edge_context(
        edge_type,
        venue,
        &token_pair,
        estimated_profit_lamports,
        risk_score,
        &route_data,
    );

    match state.consensus_engine.request_consensus(edge_id, &edge_context, models).await {
        Ok(result) => {
            let event = state.consensus_engine.create_consensus_event(edge_id, &result);
            let _ = state.event_tx.send(event);

            let response = serde_json::json!({
                "edge_id": edge_id.to_string(),
                "approved": result.approved,
                "agreement_score": result.agreement_score,
                "weighted_confidence": result.weighted_confidence,
                "reasoning_summary": result.reasoning_summary,
                "model_votes": result.model_votes.iter().map(|v| serde_json::json!({
                    "model": v.model,
                    "approved": v.approved,
                    "confidence": v.confidence,
                    "reasoning": v.reasoning,
                    "latency_ms": v.latency_ms
                })).collect::<Vec<_>>(),
                "total_latency_ms": result.total_latency_ms
            });
            McpToolResult::success(serde_json::to_string_pretty(&response).unwrap_or_default())
        }
        Err(e) => McpToolResult::error(format!("Consensus request failed: {}", e)),
    }
}

async fn consensus_result_mcp(state: &AppState, args: Value) -> McpToolResult {
    let consensus_id = match args.get("consensus_id").and_then(|v| v.as_str()) {
        Some(id) => match uuid::Uuid::parse_str(id) {
            Ok(uuid) => uuid,
            Err(_) => return McpToolResult::error("Invalid consensus_id format"),
        },
        None => return McpToolResult::error("consensus_id is required"),
    };

    // Use the consensus history from handlers/consensus.rs
    // For now, return a message that this requires the REST endpoint
    McpToolResult::success(serde_json::json!({
        "message": "Use GET /consensus/history/:id endpoint for detailed consensus results",
        "consensus_id": consensus_id.to_string(),
        "note": "Consensus results are stored in the consensus history and can be fetched via consensus_history tool"
    }).to_string())
}

async fn consensus_history_mcp(state: &AppState, args: Value) -> McpToolResult {
    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(50) as usize;
    let approved_only = args.get("approved_only").and_then(|v| v.as_bool());
    let edge_id = args.get("edge_id").and_then(|v| v.as_str());

    // Build query string for REST endpoint
    let mut query_parts = vec![format!("limit={}", limit)];
    if let Some(approved) = approved_only {
        query_parts.push(format!("approved_only={}", approved));
    }

    let result = serde_json::json!({
        "message": "Consensus history is available via REST endpoint",
        "endpoint": "/consensus/history",
        "query_params": {
            "limit": limit,
            "approved_only": approved_only,
            "edge_id": edge_id
        },
        "note": "Use the REST API or the consensus_request tool to create new consensus decisions"
    });

    McpToolResult::success(serde_json::to_string_pretty(&result).unwrap_or_default())
}

async fn consensus_stats_mcp(state: &AppState) -> McpToolResult {
    let is_configured = state.config.openrouter_api_key.is_some();
    let is_ready = state.consensus_engine.is_ready().await;

    let result = serde_json::json!({
        "consensus_engine": {
            "configured": is_configured,
            "ready": is_ready,
        },
        "note": "Use GET /consensus/stats endpoint for detailed statistics"
    });

    McpToolResult::success(serde_json::to_string_pretty(&result).unwrap_or_default())
}

async fn consensus_config_get_mcp(state: &AppState) -> McpToolResult {
    let wallet = state.config.wallet_address.clone().unwrap_or_default();
    let is_dev = crate::consensus::is_dev_wallet(&wallet);
    let models = crate::consensus::get_models_for_wallet(&wallet);
    let available = crate::consensus::get_all_available_models();

    let result = serde_json::json!({
        "is_dev_wallet": is_dev,
        "wallet": wallet,
        "active_models": models.iter().map(|m| serde_json::json!({
            "model_id": m.model_id,
            "weight": m.weight,
            "enabled": m.enabled,
            "max_tokens": m.max_tokens
        })).collect::<Vec<_>>(),
        "available_models_count": available.len(),
        "note": "Use PUT /consensus/config endpoint to update configuration"
    });

    McpToolResult::success(serde_json::to_string_pretty(&result).unwrap_or_default())
}

async fn consensus_config_update_mcp(state: &AppState, args: Value) -> McpToolResult {
    // This is a read-only MCP tool - configuration should be updated via REST API
    McpToolResult::success(serde_json::json!({
        "message": "Configuration updates should be performed via REST API",
        "endpoint": "PUT /consensus/config",
        "available_fields": ["enabled", "models", "min_consensus_threshold", "auto_apply_recommendations", "review_interval_hours"],
        "note": "For security, configuration changes require direct REST API access"
    }).to_string())
}

async fn consensus_models_list_mcp(state: &AppState) -> McpToolResult {
    let available = crate::consensus::get_all_available_models();
    let default_models = crate::consensus::get_default_models();

    let result = serde_json::json!({
        "available_models": available.iter().map(|m| serde_json::json!({
            "model_id": m.model_id,
            "weight": m.weight,
            "enabled": m.enabled,
            "max_tokens": m.max_tokens
        })).collect::<Vec<_>>(),
        "default_models": default_models,
        "total_count": available.len()
    });

    McpToolResult::success(serde_json::to_string_pretty(&result).unwrap_or_default())
}

async fn consensus_models_discovered_mcp(state: &AppState) -> McpToolResult {
    let models = crate::consensus::get_discovered_models().await;
    let status = crate::consensus::get_discovery_status().await;

    let result = serde_json::json!({
        "discovery_status": status,
        "discovered_models": models.iter().map(|m| serde_json::json!({
            "model_id": m.model_id,
            "weight": m.weight,
            "enabled": m.enabled,
            "max_tokens": m.max_tokens
        })).collect::<Vec<_>>(),
        "total_discovered": models.len()
    });

    McpToolResult::success(serde_json::to_string_pretty(&result).unwrap_or_default())
}

async fn consensus_learning_summary_mcp(state: &AppState) -> McpToolResult {
    let wallet = state.config.wallet_address.clone().unwrap_or_else(|| "default".to_string());

    let recommendations = state.engrams_client.get_recommendations(&wallet, None, Some(100)).await.unwrap_or_default();
    let conversations = state.engrams_client.get_conversations(&wallet, Some(20)).await.unwrap_or_default();

    let pending = recommendations.iter()
        .filter(|r| r.status == crate::engrams::schemas::RecommendationStatus::Pending)
        .count();
    let applied = recommendations.iter()
        .filter(|r| r.status == crate::engrams::schemas::RecommendationStatus::Applied)
        .count();

    let result = serde_json::json!({
        "wallet": wallet,
        "recommendations": {
            "total": recommendations.len(),
            "pending": pending,
            "applied": applied,
            "recent": recommendations.into_iter().take(5).collect::<Vec<_>>()
        },
        "conversations": {
            "total": conversations.len(),
            "recent": conversations.into_iter().take(5).collect::<Vec<_>>()
        }
    });

    McpToolResult::success(serde_json::to_string_pretty(&result).unwrap_or_default())
}
