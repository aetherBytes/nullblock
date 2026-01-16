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

pub fn create_arb_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        // Health
        .route("/api/arb/health", get(arb_health))

        // Scanner
        .route("/api/arb/scanner/status", get(scanner_status))
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
}
