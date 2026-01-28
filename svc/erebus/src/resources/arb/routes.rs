use axum::{
    body::Body,
    extract::{Path, Query, Json},
    http::StatusCode,
    response::{Json as ResponseJson, IntoResponse, Response},
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{info, error};

fn get_arb_service_url() -> String {
    std::env::var("ARB_FARM_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:9007".to_string())
}

#[derive(Debug, Serialize)]
pub struct ArbErrorResponse {
    pub error: String,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub status: Option<String>,
    pub edge_type: Option<String>,
}

async fn proxy_request(
    method: &str,
    endpoint: &str,
    body: Option<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    let client = reqwest::Client::new();
    let base_url = get_arb_service_url();
    let url = format!("{}/{}", base_url, endpoint);

    info!("🔗 Proxying {} request to ArbFarm service: {}", method, url);

    let request_builder = match method {
        "GET" => client.get(&url),
        "POST" => client.post(&url),
        "PUT" => client.put(&url),
        "DELETE" => client.delete(&url),
        _ => {
            return Err((
                StatusCode::METHOD_NOT_ALLOWED,
                ResponseJson(ArbErrorResponse {
                    error: "invalid_method".to_string(),
                    code: "INVALID_METHOD".to_string(),
                    message: format!("Method {} not supported", method),
                }),
            ));
        }
    };

    let request_builder = if let Some(body) = body {
        request_builder.json(&body)
    } else {
        request_builder
    };

    match request_builder
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            match response.json::<Value>().await {
                Ok(json_response) => {
                    if status.is_success() {
                        info!("✅ ArbFarm service response successful");
                        Ok(ResponseJson(json_response))
                    } else {
                        error!("❌ ArbFarm service returned error status: {}", status);
                        Err((
                            StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                            ResponseJson(ArbErrorResponse {
                                error: "arb_service_error".to_string(),
                                code: "ARB_SERVICE_ERROR".to_string(),
                                message: json_response.to_string(),
                            }),
                        ))
                    }
                }
                Err(e) => {
                    error!("❌ Failed to parse ArbFarm service response: {}", e);
                    Err((
                        StatusCode::BAD_GATEWAY,
                        ResponseJson(ArbErrorResponse {
                            error: "parse_error".to_string(),
                            code: "ARB_PARSE_ERROR".to_string(),
                            message: format!("Failed to parse response: {}", e),
                        }),
                    ))
                }
            }
        }
        Err(e) => {
            error!("❌ Failed to connect to ArbFarm service: {}", e);
            Err((
                StatusCode::SERVICE_UNAVAILABLE,
                ResponseJson(ArbErrorResponse {
                    error: "connection_error".to_string(),
                    code: "ARB_UNAVAILABLE".to_string(),
                    message: format!("Failed to connect to ArbFarm service: {}", e),
                }),
            ))
        }
    }
}

async fn proxy_sse(endpoint: &str) -> Result<Response, StatusCode> {
    let base_url = get_arb_service_url();
    let url = format!("{}/{}", base_url, endpoint);

    info!("🔌 Proxying SSE request to ArbFarm: {}", url);

    let client = reqwest::Client::new();
    match client.get(&url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                let stream = response.bytes_stream();
                Ok((
                    [(axum::http::header::CONTENT_TYPE, "text/event-stream")],
                    Body::from_stream(stream)
                ).into_response())
            } else {
                error!("❌ SSE proxy failed: {}", response.status());
                Err(StatusCode::BAD_GATEWAY)
            }
        }
        Err(e) => {
            error!("❌ SSE proxy connection failed: {}", e);
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }
}

// Health
pub async fn arb_health() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("🏥 ArbFarm health check requested");
    proxy_request("GET", "health", None).await
}

// Scanner
pub async fn scanner_status() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📊 Scanner status requested");
    proxy_request("GET", "scanner/status", None).await
}

pub async fn scanner_stream() -> Result<Response, StatusCode> {
    proxy_sse("scanner/stream").await
}

pub async fn scanner_start(
    Json(request): Json<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("▶️ Start scanner requested");
    proxy_request("POST", "scanner/start", Some(request)).await
}

pub async fn scanner_stop() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("⏹️ Stop scanner requested");
    proxy_request("POST", "scanner/stop", None).await
}

#[derive(Debug, Deserialize)]
pub struct SignalQuery {
    pub limit: Option<i64>,
    pub venue_type: Option<String>,
    pub min_profit_bps: Option<i32>,
    pub min_confidence: Option<f64>,
}

pub async fn scanner_signals(
    Query(query): Query<SignalQuery>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📡 Scanner signals requested");
    let mut endpoint = "scanner/signals".to_string();
    let mut params = Vec::new();

    if let Some(limit) = query.limit {
        params.push(format!("limit={}", limit));
    }
    if let Some(venue_type) = &query.venue_type {
        params.push(format!("venue_type={}", venue_type));
    }
    if let Some(min_profit_bps) = query.min_profit_bps {
        params.push(format!("min_profit_bps={}", min_profit_bps));
    }
    if let Some(min_confidence) = query.min_confidence {
        params.push(format!("min_confidence={}", min_confidence));
    }

    if !params.is_empty() {
        endpoint = format!("{}?{}", endpoint, params.join("&"));
    }

    proxy_request("GET", &endpoint, None).await
}

// Venues
pub async fn list_venues(
    Query(query): Query<ListQuery>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📋 List venues requested");
    let endpoint = format!(
        "venues?limit={}&offset={}",
        query.limit.unwrap_or(50),
        query.offset.unwrap_or(0)
    );
    proxy_request("GET", &endpoint, None).await
}

pub async fn create_venue(
    Json(request): Json<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("➕ Create venue requested");
    proxy_request("POST", "venues", Some(request)).await
}

pub async fn get_venue(
    Path(id): Path<String>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📖 Get venue {} requested", id);
    proxy_request("GET", &format!("venues/{}", id), None).await
}

pub async fn delete_venue(
    Path(id): Path<String>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("🗑️ Delete venue {} requested", id);
    proxy_request("DELETE", &format!("venues/{}", id), None).await
}

// Edges (Opportunities)
pub async fn list_edges(
    Query(query): Query<ListQuery>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📋 List edges requested");
    let mut endpoint = format!(
        "edges?limit={}&offset={}",
        query.limit.unwrap_or(50),
        query.offset.unwrap_or(0)
    );
    if let Some(status) = &query.status {
        endpoint.push_str(&format!("&status={}", status));
    }
    if let Some(edge_type) = &query.edge_type {
        endpoint.push_str(&format!("&edge_type={}", edge_type));
    }
    proxy_request("GET", &endpoint, None).await
}

pub async fn get_edge(
    Path(id): Path<String>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📖 Get edge {} requested", id);
    proxy_request("GET", &format!("edges/{}", id), None).await
}

pub async fn approve_edge(
    Path(id): Path<String>,
    Json(request): Json<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("✅ Approve edge {} requested", id);
    proxy_request("POST", &format!("edges/{}/approve", id), Some(request)).await
}

pub async fn reject_edge(
    Path(id): Path<String>,
    Json(request): Json<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("❌ Reject edge {} requested", id);
    proxy_request("POST", &format!("edges/{}/reject", id), Some(request)).await
}

pub async fn execute_edge(
    Path(id): Path<String>,
    Json(request): Json<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("🚀 Execute edge {} requested", id);
    proxy_request("POST", &format!("edges/{}/execute", id), Some(request)).await
}

pub async fn edges_stream() -> Result<Response, StatusCode> {
    proxy_sse("edges/stream").await
}

// Strategies
pub async fn list_strategies(
    Query(query): Query<ListQuery>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📋 List strategies requested");
    let endpoint = format!(
        "strategies?limit={}&offset={}",
        query.limit.unwrap_or(50),
        query.offset.unwrap_or(0)
    );
    proxy_request("GET", &endpoint, None).await
}

pub async fn create_strategy(
    Json(request): Json<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("➕ Create strategy requested");
    proxy_request("POST", "strategies", Some(request)).await
}

pub async fn get_strategy(
    Path(id): Path<String>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📖 Get strategy {} requested", id);
    proxy_request("GET", &format!("strategies/{}", id), None).await
}

pub async fn update_strategy(
    Path(id): Path<String>,
    Json(request): Json<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("✏️ Update strategy {} requested", id);
    proxy_request("PUT", &format!("strategies/{}", id), Some(request)).await
}

pub async fn delete_strategy(
    Path(id): Path<String>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("🗑️ Delete strategy {} requested", id);
    proxy_request("DELETE", &format!("strategies/{}", id), None).await
}

pub async fn toggle_strategy(
    Path(id): Path<String>,
    Json(request): Json<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("🔄 Toggle strategy {} requested", id);
    proxy_request("POST", &format!("strategies/{}/toggle", id), Some(request)).await
}

pub async fn strategy_stats(
    Path(id): Path<String>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📊 Get strategy {} stats requested", id);
    proxy_request("GET", &format!("strategies/{}/stats", id), None).await
}

// Consensus
pub async fn request_consensus(
    Json(request): Json<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("🤖 Request consensus");
    proxy_request("POST", "consensus/request", Some(request)).await
}

pub async fn get_consensus(
    Path(id): Path<String>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📖 Get consensus {} requested", id);
    proxy_request("GET", &format!("consensus/{}", id), None).await
}

pub async fn consensus_history(
    Query(query): Query<ListQuery>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📜 Consensus history requested");
    let endpoint = format!(
        "consensus/history?limit={}&offset={}",
        query.limit.unwrap_or(50),
        query.offset.unwrap_or(0)
    );
    proxy_request("GET", &endpoint, None).await
}

// Research/DD
pub async fn research_ingest(
    Json(request): Json<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("🔗 Research ingest URL requested");
    proxy_request("POST", "research/ingest", Some(request)).await
}

pub async fn research_discoveries(
    Query(query): Query<ListQuery>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📋 Research discoveries requested");
    let mut endpoint = format!(
        "research/discoveries?limit={}&offset={}",
        query.limit.unwrap_or(50),
        query.offset.unwrap_or(0)
    );
    if let Some(status) = &query.status {
        endpoint.push_str(&format!("&status={}", status));
    }
    proxy_request("GET", &endpoint, None).await
}

pub async fn research_approve(
    Path(id): Path<String>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("✅ Approve discovery {} requested", id);
    proxy_request("POST", &format!("research/discoveries/{}/approve", id), None).await
}

pub async fn research_reject(
    Path(id): Path<String>,
    Json(request): Json<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("❌ Reject discovery {} requested", id);
    proxy_request("POST", &format!("research/discoveries/{}/reject", id), Some(request)).await
}

pub async fn research_sources(
    Query(query): Query<ListQuery>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📋 Research sources requested");
    let endpoint = format!(
        "research/sources?limit={}&offset={}",
        query.limit.unwrap_or(50),
        query.offset.unwrap_or(0)
    );
    proxy_request("GET", &endpoint, None).await
}

pub async fn add_research_source(
    Json(request): Json<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("➕ Add research source requested");
    proxy_request("POST", "research/sources", Some(request)).await
}

pub async fn delete_research_source(
    Path(id): Path<String>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("🗑️ Delete research source {} requested", id);
    proxy_request("DELETE", &format!("research/sources/{}", id), None).await
}

// KOL Tracking
pub async fn list_kols(
    Query(query): Query<ListQuery>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📋 List KOLs requested");
    let endpoint = format!(
        "kol?limit={}&offset={}",
        query.limit.unwrap_or(50),
        query.offset.unwrap_or(0)
    );
    proxy_request("GET", &endpoint, None).await
}

pub async fn add_kol(
    Json(request): Json<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("➕ Add KOL requested");
    proxy_request("POST", "kol", Some(request)).await
}

pub async fn get_kol(
    Path(id): Path<String>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📖 Get KOL {} requested", id);
    proxy_request("GET", &format!("kol/{}", id), None).await
}

pub async fn update_kol(
    Path(id): Path<String>,
    Json(request): Json<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("✏️ Update KOL {} requested", id);
    proxy_request("PUT", &format!("kol/{}", id), Some(request)).await
}

pub async fn delete_kol(
    Path(id): Path<String>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("🗑️ Delete KOL {} requested", id);
    proxy_request("DELETE", &format!("kol/{}", id), None).await
}

pub async fn kol_trades(
    Path(id): Path<String>,
    Query(query): Query<ListQuery>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📋 Get KOL {} trades requested", id);
    let endpoint = format!(
        "kol/{}/trades?limit={}&offset={}",
        id,
        query.limit.unwrap_or(50),
        query.offset.unwrap_or(0)
    );
    proxy_request("GET", &endpoint, None).await
}

pub async fn enable_copy_trading(
    Path(id): Path<String>,
    Json(request): Json<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("✅ Enable copy trading for KOL {} requested", id);
    proxy_request("POST", &format!("kol/{}/copy/enable", id), Some(request)).await
}

pub async fn disable_copy_trading(
    Path(id): Path<String>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("⛔ Disable copy trading for KOL {} requested", id);
    proxy_request("POST", &format!("kol/{}/copy/disable", id), None).await
}

// Threat Detection
pub async fn threat_check_token(
    Path(mint): Path<String>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("🔍 Threat check for token {} requested", mint);
    proxy_request("GET", &format!("threat/check/{}", mint), None).await
}

pub async fn threat_check_wallet(
    Path(address): Path<String>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("🔍 Threat check for wallet {} requested", address);
    proxy_request("GET", &format!("threat/wallet/{}", address), None).await
}

pub async fn threat_blocked(
    Query(query): Query<ListQuery>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📋 Blocked entities requested");
    let endpoint = format!(
        "threat/blocked?limit={}&offset={}",
        query.limit.unwrap_or(50),
        query.offset.unwrap_or(0)
    );
    proxy_request("GET", &endpoint, None).await
}

pub async fn threat_report(
    Json(request): Json<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("🚨 Threat report submitted");
    proxy_request("POST", "threat/report", Some(request)).await
}

pub async fn threat_score(
    Path(mint): Path<String>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📊 Threat score for {} requested", mint);
    proxy_request("GET", &format!("threat/score/{}", mint), None).await
}

pub async fn threat_score_history(
    Path(mint): Path<String>,
    Query(query): Query<ListQuery>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📜 Threat score history for {} requested", mint);
    let endpoint = format!(
        "threat/score/{}/history?limit={}&offset={}",
        mint,
        query.limit.unwrap_or(50),
        query.offset.unwrap_or(0)
    );
    proxy_request("GET", &endpoint, None).await
}

pub async fn threat_watch(
    Json(request): Json<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("👁️ Add wallet to threat watch");
    proxy_request("POST", "threat/watch", Some(request)).await
}

pub async fn threat_alerts(
    Query(query): Query<ListQuery>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("🚨 Threat alerts requested");
    let endpoint = format!(
        "threat/alerts?limit={}&offset={}",
        query.limit.unwrap_or(50),
        query.offset.unwrap_or(0)
    );
    proxy_request("GET", &endpoint, None).await
}

pub async fn threat_whitelist(
    Json(request): Json<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("✅ Add to whitelist");
    proxy_request("POST", "threat/whitelist", Some(request)).await
}

// Swarm Management
pub async fn swarm_status() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📊 Swarm status requested");
    proxy_request("GET", "swarm/status", None).await
}

pub async fn swarm_pause() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("⏸️ Pause swarm requested");
    proxy_request("POST", "swarm/pause", None).await
}

pub async fn swarm_resume() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("▶️ Resume swarm requested");
    proxy_request("POST", "swarm/resume", None).await
}

pub async fn swarm_agents() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📋 List swarm agents requested");
    proxy_request("GET", "swarm/agents", None).await
}

pub async fn restart_agent(
    Path(agent_type): Path<String>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("🔄 Restart agent {} requested", agent_type);
    proxy_request("POST", &format!("swarm/agents/{}/restart", agent_type), None).await
}

// Trades
pub async fn list_trades(
    Query(query): Query<ListQuery>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📋 List trades requested");
    let endpoint = format!(
        "trades?limit={}&offset={}",
        query.limit.unwrap_or(50),
        query.offset.unwrap_or(0)
    );
    proxy_request("GET", &endpoint, None).await
}

pub async fn get_trade(
    Path(id): Path<String>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📖 Get trade {} requested", id);
    proxy_request("GET", &format!("trades/{}", id), None).await
}

pub async fn trade_stats(
    Query(query): Query<ListQuery>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📊 Trade stats requested");
    let endpoint = format!(
        "trades/stats?limit={}",
        query.limit.unwrap_or(50)
    );
    proxy_request("GET", &endpoint, None).await
}

// Consensus Stats & Models
pub async fn consensus_stats() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📊 Consensus stats requested");
    proxy_request("GET", "consensus/stats", None).await
}

pub async fn consensus_models() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("🤖 Consensus models requested");
    proxy_request("GET", "consensus/models", None).await
}

// Wallet Management
pub async fn wallet_status() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("👛 Wallet status requested");
    proxy_request("GET", "wallet/status", None).await
}

pub async fn wallet_setup(
    Json(request): Json<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("👛 Wallet setup requested");
    proxy_request("POST", "wallet/setup", Some(request)).await
}

pub async fn wallet_policy(
    Json(request): Json<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📜 Wallet policy update requested");
    proxy_request("PUT", "wallet/policy", Some(request)).await
}

pub async fn wallet_balance() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("💰 Wallet balance requested");
    proxy_request("GET", "wallet/balance", None).await
}

pub async fn wallet_disconnect() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("🔌 Wallet disconnect requested");
    proxy_request("POST", "wallet/disconnect", None).await
}

pub async fn wallet_usage() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📈 Wallet usage requested");
    proxy_request("GET", "wallet/usage", None).await
}

pub async fn wallet_test_sign(
    Json(request): Json<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("🔏 Wallet test sign requested");
    proxy_request("POST", "wallet/test-sign", Some(request)).await
}

// Settings
pub async fn get_risk_settings() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("⚙️ Get risk settings requested");
    proxy_request("GET", "settings/risk", None).await
}

pub async fn update_risk_settings(
    Json(request): Json<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("⚙️ Update risk settings requested");
    proxy_request("PUT", "settings/risk", Some(request)).await
}

pub async fn get_venue_settings() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("⚙️ Get venue settings requested");
    proxy_request("GET", "settings/venues", None).await
}

pub async fn get_api_keys_status() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("🔑 Get API keys status requested");
    proxy_request("GET", "settings/api-keys", None).await
}

// Events Stream
pub async fn events_stream() -> Result<Response, StatusCode> {
    proxy_sse("events/stream").await
}

// Threat Stream
pub async fn threat_stream() -> Result<Response, StatusCode> {
    proxy_sse("threat/stream").await
}

// Positions Stream
pub async fn positions_stream() -> Result<Response, StatusCode> {
    proxy_sse("positions/stream").await
}

// Positions
pub async fn list_positions() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📊 List positions requested");
    proxy_request("GET", "positions", None).await
}

pub async fn positions_history() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📊 Positions history requested");
    proxy_request("GET", "positions/history", None).await
}

pub async fn positions_exposure() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📊 Positions exposure requested");
    proxy_request("GET", "positions/exposure", None).await
}

pub async fn positions_pnl_summary() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📊 Positions PnL summary requested");
    proxy_request("GET", "positions/pnl-summary", None).await
}

pub async fn positions_reconcile() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("🔄 Positions reconcile requested");
    proxy_request("POST", "positions/reconcile", None).await
}

pub async fn positions_monitor_status() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📊 Positions monitor status requested");
    proxy_request("GET", "positions/monitor/status", None).await
}

pub async fn positions_monitor_start() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("▶️ Positions monitor start requested");
    proxy_request("POST", "positions/monitor/start", None).await
}

pub async fn positions_monitor_stop() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("⏹️ Positions monitor stop requested");
    proxy_request("POST", "positions/monitor/stop", None).await
}

pub async fn positions_emergency_close() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("🚨 Positions emergency close requested");
    proxy_request("POST", "positions/emergency-close", None).await
}

pub async fn positions_sell_all() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("💰 Positions sell all requested");
    proxy_request("POST", "positions/sell-all", None).await
}

pub async fn positions_force_clear() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("🧹 Positions force clear requested");
    proxy_request("POST", "positions/force-clear", None).await
}

pub async fn get_position(
    Path(id): Path<String>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📖 Get position {} requested", id);
    proxy_request("GET", &format!("positions/{}", id), None).await
}

pub async fn close_position(
    Path(id): Path<String>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("✖️ Close position {} requested", id);
    proxy_request("POST", &format!("positions/{}/close", id), None).await
}

// Curves
pub async fn curves_tokens() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📊 Curves tokens requested");
    proxy_request("GET", "curves/tokens", None).await
}

pub async fn curves_health() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("🏥 Curves health requested");
    proxy_request("GET", "curves/health", None).await
}

pub async fn curves_graduation_candidates() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("🎓 Curves graduation candidates requested");
    proxy_request("GET", "curves/graduation-candidates", None).await
}

pub async fn curves_top_opportunities() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("🔝 Curves top opportunities requested");
    proxy_request("GET", "curves/top-opportunities", None).await
}

pub async fn curves_progress(
    Path(mint): Path<String>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📈 Curves progress for {} requested", mint);
    proxy_request("GET", &format!("curves/{}/progress", mint), None).await
}

pub async fn curves_metrics(
    Path(mint): Path<String>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📊 Curves metrics for {} requested", mint);
    proxy_request("GET", &format!("curves/{}/metrics", mint), None).await
}

pub async fn curves_state(
    Path(mint): Path<String>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📊 Curves state for {} requested", mint);
    proxy_request("GET", &format!("curves/{}/state", mint), None).await
}

pub async fn curves_quote(
    Path(mint): Path<String>,
    Json(request): Json<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("💱 Curves quote for {} requested", mint);
    proxy_request("POST", &format!("curves/{}/quote", mint), Some(request)).await
}

pub async fn curves_buy(
    Path(mint): Path<String>,
    Json(request): Json<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("💰 Curves buy for {} requested", mint);
    proxy_request("POST", &format!("curves/{}/buy", mint), Some(request)).await
}

pub async fn curves_sell(
    Path(mint): Path<String>,
    Json(request): Json<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("💸 Curves sell for {} requested", mint);
    proxy_request("POST", &format!("curves/{}/sell", mint), Some(request)).await
}

pub async fn curves_score(
    Path(mint): Path<String>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📊 Curves score for {} requested", mint);
    proxy_request("GET", &format!("curves/{}/score", mint), None).await
}

// Config
pub async fn config_risk_get() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("⚙️ Get risk config requested");
    proxy_request("GET", "config/risk", None).await
}

pub async fn config_risk_set(
    Json(request): Json<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("⚙️ Set risk config requested");
    proxy_request("POST", "config/risk", Some(request)).await
}

pub async fn config_risk_custom(
    Json(request): Json<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("⚙️ Set custom risk config requested");
    proxy_request("POST", "config/risk/custom", Some(request)).await
}

// Executor
pub async fn executor_stats() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📊 Executor stats requested");
    proxy_request("GET", "executor/stats", None).await
}

pub async fn executor_start() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("▶️ Executor start requested");
    proxy_request("POST", "executor/start", None).await
}

pub async fn executor_stop() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("⏹️ Executor stop requested");
    proxy_request("POST", "executor/stop", None).await
}

// Graduation Sniper
pub async fn sniper_stats() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📊 Sniper stats requested");
    proxy_request("GET", "sniper/stats", None).await
}

pub async fn sniper_positions() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📋 Sniper positions requested");
    proxy_request("GET", "sniper/positions", None).await
}

pub async fn sniper_add_position(
    Json(body): Json<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("➕ Sniper add position requested");
    proxy_request("POST", "sniper/positions", Some(body)).await
}

pub async fn sniper_remove_position(
    Path(mint): Path<String>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("➖ Sniper remove position requested: {}", mint);
    proxy_request("DELETE", &format!("sniper/positions/{}", mint), None).await
}

pub async fn sniper_manual_sell(
    Path(mint): Path<String>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("💰 Sniper manual sell requested: {}", mint);
    proxy_request("POST", &format!("sniper/positions/{}/sell", mint), None).await
}

pub async fn sniper_start() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("▶️ Sniper start requested");
    proxy_request("POST", "sniper/start", None).await
}

pub async fn sniper_stop() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("⏹️ Sniper stop requested");
    proxy_request("POST", "sniper/stop", None).await
}

pub async fn sniper_config_get() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("⚙️ Sniper config get requested");
    proxy_request("GET", "sniper/config", None).await
}

pub async fn sniper_config_update(
    Json(body): Json<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("⚙️ Sniper config update requested");
    proxy_request("PUT", "sniper/config", Some(body)).await
}

// Helius
pub async fn helius_status() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📊 Helius status requested");
    proxy_request("GET", "helius/status", None).await
}

pub async fn helius_laserstream() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📊 Helius laserstream status requested");
    proxy_request("GET", "helius/laserstream", None).await
}

pub async fn helius_priority_fees() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("💰 Helius priority fees requested");
    proxy_request("GET", "helius/priority-fees", None).await
}

pub async fn helius_config() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("⚙️ Helius config requested");
    proxy_request("GET", "helius/config", None).await
}

pub async fn helius_das_lookup(
    Json(request): Json<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("🔍 Helius DAS lookup requested");
    proxy_request("POST", "helius/das/lookup", Some(request)).await
}

// Approvals
pub async fn list_approvals() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📋 List approvals requested");
    proxy_request("GET", "approvals", None).await
}

pub async fn list_pending_approvals() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("📋 List pending approvals requested");
    proxy_request("GET", "approvals/pending", None).await
}

pub async fn execution_config() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("⚙️ Execution config requested");
    proxy_request("GET", "execution/config", None).await
}

pub async fn update_execution_config(
    Json(request): Json<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("⚙️ Update execution config requested");
    proxy_request("PUT", "execution/config", Some(request)).await
}

pub async fn execution_toggle(
    Json(request): Json<Value>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("🔄 Execution toggle requested");
    proxy_request("POST", "execution/toggle", Some(request)).await
}

// Engrams
pub async fn engram_search(
    Query(query): Query<ListQuery>,
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("🔍 Engram search requested");
    let endpoint = format!(
        "engram/search?limit={}&offset={}",
        query.limit.unwrap_or(50),
        query.offset.unwrap_or(0)
    );
    proxy_request("GET", &endpoint, None).await
}

pub async fn engram_insights() -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<ArbErrorResponse>)> {
    info!("💡 Engram insights requested");
    proxy_request("GET", "engram/insights", None).await
}

pub fn create_arb_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        // Health
        .route("/api/arb/health", get(arb_health))

        // Scanner
        .route("/api/arb/scanner/status", get(scanner_status))
        .route("/api/arb/scanner/signals", get(scanner_signals))
        .route("/api/arb/scanner/stream", get(scanner_stream))
        .route("/api/arb/scanner/start", post(scanner_start))
        .route("/api/arb/scanner/stop", post(scanner_stop))

        // Venues
        .route("/api/arb/venues", get(list_venues))
        .route("/api/arb/venues", post(create_venue))
        .route("/api/arb/venues/:id", get(get_venue))
        .route("/api/arb/venues/:id", delete(delete_venue))

        // Edges (Opportunities)
        .route("/api/arb/edges", get(list_edges))
        .route("/api/arb/edges/stream", get(edges_stream))
        .route("/api/arb/edges/:id", get(get_edge))
        .route("/api/arb/edges/:id/approve", post(approve_edge))
        .route("/api/arb/edges/:id/reject", post(reject_edge))
        .route("/api/arb/edges/:id/execute", post(execute_edge))

        // Strategies
        .route("/api/arb/strategies", get(list_strategies))
        .route("/api/arb/strategies", post(create_strategy))
        .route("/api/arb/strategies/:id", get(get_strategy))
        .route("/api/arb/strategies/:id", put(update_strategy))
        .route("/api/arb/strategies/:id", delete(delete_strategy))
        .route("/api/arb/strategies/:id/toggle", post(toggle_strategy))
        .route("/api/arb/strategies/:id/stats", get(strategy_stats))

        // Consensus
        .route("/api/arb/consensus/request", post(request_consensus))
        .route("/api/arb/consensus/history", get(consensus_history))
        .route("/api/arb/consensus/stats", get(consensus_stats))
        .route("/api/arb/consensus/models", get(consensus_models))
        .route("/api/arb/consensus/:id", get(get_consensus))

        // Research/DD
        .route("/api/arb/research/ingest", post(research_ingest))
        .route("/api/arb/research/discoveries", get(research_discoveries))
        .route("/api/arb/research/discoveries/:id/approve", post(research_approve))
        .route("/api/arb/research/discoveries/:id/reject", post(research_reject))
        .route("/api/arb/research/sources", get(research_sources))
        .route("/api/arb/research/sources", post(add_research_source))
        .route("/api/arb/research/sources/:id", delete(delete_research_source))

        // KOL Tracking
        .route("/api/arb/kol", get(list_kols))
        .route("/api/arb/kol", post(add_kol))
        .route("/api/arb/kol/:id", get(get_kol))
        .route("/api/arb/kol/:id", put(update_kol))
        .route("/api/arb/kol/:id", delete(delete_kol))
        .route("/api/arb/kol/:id/trades", get(kol_trades))
        .route("/api/arb/kol/:id/copy/enable", post(enable_copy_trading))
        .route("/api/arb/kol/:id/copy/disable", post(disable_copy_trading))

        // Threat Detection
        .route("/api/arb/threat/check/:mint", get(threat_check_token))
        .route("/api/arb/threat/wallet/:address", get(threat_check_wallet))
        .route("/api/arb/threat/blocked", get(threat_blocked))
        .route("/api/arb/threat/report", post(threat_report))
        .route("/api/arb/threat/score/:mint", get(threat_score))
        .route("/api/arb/threat/score/:mint/history", get(threat_score_history))
        .route("/api/arb/threat/watch", post(threat_watch))
        .route("/api/arb/threat/alerts", get(threat_alerts))
        .route("/api/arb/threat/whitelist", post(threat_whitelist))

        // Swarm Management
        .route("/api/arb/swarm/status", get(swarm_status))
        .route("/api/arb/swarm/pause", post(swarm_pause))
        .route("/api/arb/swarm/resume", post(swarm_resume))
        .route("/api/arb/swarm/agents", get(swarm_agents))
        .route("/api/arb/swarm/agents/:type/restart", post(restart_agent))

        // Trades
        .route("/api/arb/trades", get(list_trades))
        .route("/api/arb/trades/stats", get(trade_stats))
        .route("/api/arb/trades/:id", get(get_trade))

        // Wallet Management
        .route("/api/arb/wallet/status", get(wallet_status))
        .route("/api/arb/wallet/setup", post(wallet_setup))
        .route("/api/arb/wallet/policy", put(wallet_policy))
        .route("/api/arb/wallet/balance", get(wallet_balance))
        .route("/api/arb/wallet/disconnect", post(wallet_disconnect))
        .route("/api/arb/wallet/usage", get(wallet_usage))
        .route("/api/arb/wallet/test-sign", post(wallet_test_sign))

        // Settings
        .route("/api/arb/settings/risk", get(get_risk_settings))
        .route("/api/arb/settings/risk", put(update_risk_settings))
        .route("/api/arb/settings/venues", get(get_venue_settings))
        .route("/api/arb/settings/api-keys", get(get_api_keys_status))

        // Event Streams
        .route("/api/arb/events/stream", get(events_stream))
        .route("/api/arb/threat/stream", get(threat_stream))
        .route("/api/arb/positions/stream", get(positions_stream))

        // Positions
        .route("/api/arb/positions", get(list_positions))
        .route("/api/arb/positions/history", get(positions_history))
        .route("/api/arb/positions/exposure", get(positions_exposure))
        .route("/api/arb/positions/pnl-summary", get(positions_pnl_summary))
        .route("/api/arb/positions/reconcile", post(positions_reconcile))
        .route("/api/arb/positions/monitor/status", get(positions_monitor_status))
        .route("/api/arb/positions/monitor/start", post(positions_monitor_start))
        .route("/api/arb/positions/monitor/stop", post(positions_monitor_stop))
        .route("/api/arb/positions/emergency-close", post(positions_emergency_close))
        .route("/api/arb/positions/sell-all", post(positions_sell_all))
        .route("/api/arb/positions/force-clear", post(positions_force_clear))
        .route("/api/arb/positions/:id", get(get_position))
        .route("/api/arb/positions/:id/close", post(close_position))

        // Curves
        .route("/api/arb/curves/tokens", get(curves_tokens))
        .route("/api/arb/curves/health", get(curves_health))
        .route("/api/arb/curves/graduation-candidates", get(curves_graduation_candidates))
        .route("/api/arb/curves/top-opportunities", get(curves_top_opportunities))
        .route("/api/arb/curves/:mint/progress", get(curves_progress))
        .route("/api/arb/curves/:mint/metrics", get(curves_metrics))
        .route("/api/arb/curves/:mint/state", get(curves_state))
        .route("/api/arb/curves/:mint/quote", post(curves_quote))
        .route("/api/arb/curves/:mint/buy", post(curves_buy))
        .route("/api/arb/curves/:mint/sell", post(curves_sell))
        .route("/api/arb/curves/:mint/score", get(curves_score))

        // Config
        .route("/api/arb/config/risk", get(config_risk_get))
        .route("/api/arb/config/risk", post(config_risk_set))
        .route("/api/arb/config/risk/custom", post(config_risk_custom))

        // Executor
        .route("/api/arb/executor/stats", get(executor_stats))
        .route("/api/arb/executor/start", post(executor_start))
        .route("/api/arb/executor/stop", post(executor_stop))

        // Helius
        .route("/api/arb/helius/status", get(helius_status))
        .route("/api/arb/helius/laserstream", get(helius_laserstream))
        .route("/api/arb/helius/priority-fees", get(helius_priority_fees))
        .route("/api/arb/helius/config", get(helius_config))
        .route("/api/arb/helius/das/lookup", post(helius_das_lookup))

        // Approvals
        .route("/api/arb/approvals", get(list_approvals))
        .route("/api/arb/approvals/pending", get(list_pending_approvals))
        .route("/api/arb/execution/config", get(execution_config).put(update_execution_config))
        .route("/api/arb/execution/toggle", post(execution_toggle))

        // Engrams
        .route("/api/arb/engram/search", get(engram_search))
        .route("/api/arb/engram/insights", get(engram_insights))

        // Graduation Sniper
        .route("/api/arb/sniper/stats", get(sniper_stats))
        .route("/api/arb/sniper/positions", get(sniper_positions).post(sniper_add_position))
        .route("/api/arb/sniper/positions/:mint", delete(sniper_remove_position))
        .route("/api/arb/sniper/positions/:mint/sell", post(sniper_manual_sell))
        .route("/api/arb/sniper/start", post(sniper_start))
        .route("/api/arb/sniper/stop", post(sniper_stop))
        .route("/api/arb/sniper/config", get(sniper_config_get).put(sniper_config_update))

        // Graduation Tracker
}
