use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

use crate::error::AppResult;
use crate::events::ArbEvent;
use crate::models::{
    AddKolRequest, CopyTrade, CopyTradeConfig, CopyTradeStatus, EnableCopyRequest,
    KolEntity, KolEntityType, KolStats, KolTrade, KolTradeType, TrustScoreBreakdown,
    UpdateKolRequest,
};
use crate::server::AppState;
use crate::webhooks::helius::{WebhookConfig, TransactionType, WebhookType};

lazy_static::lazy_static! {
    static ref KOL_STORE: RwLock<HashMap<Uuid, KolEntity>> = RwLock::new(HashMap::new());
    static ref KOL_TRADES: RwLock<HashMap<Uuid, Vec<KolTrade>>> = RwLock::new(HashMap::new());
    static ref COPY_TRADES: RwLock<HashMap<Uuid, Vec<CopyTrade>>> = RwLock::new(HashMap::new());
}

#[derive(Debug, Deserialize)]
pub struct ListKolQuery {
    pub is_active: Option<bool>,
    pub copy_enabled: Option<bool>,
    pub min_trust_score: Option<f64>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct KolListResponse {
    pub kols: Vec<KolEntity>,
    pub total: usize,
}

pub async fn list_kols(
    State(_state): State<AppState>,
    Query(query): Query<ListKolQuery>,
) -> impl IntoResponse {
    let store = KOL_STORE.read().unwrap();
    let mut kols: Vec<KolEntity> = store
        .values()
        .filter(|k| {
            if let Some(active) = query.is_active {
                if k.is_active != active {
                    return false;
                }
            }
            if let Some(copy_enabled) = query.copy_enabled {
                if k.copy_trading_enabled != copy_enabled {
                    return false;
                }
            }
            if let Some(min_score) = query.min_trust_score {
                let score: f64 = k.trust_score.try_into().unwrap_or(0.0);
                if score < min_score {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect();

    kols.sort_by(|a, b| b.trust_score.cmp(&a.trust_score));

    let total = kols.len();
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(50);
    let kols: Vec<_> = kols.into_iter().skip(offset).take(limit).collect();

    (StatusCode::OK, Json(KolListResponse { kols, total }))
}

pub async fn add_kol(
    State(state): State<AppState>,
    Json(request): Json<AddKolRequest>,
) -> impl IntoResponse {
    let (entity_type, identifier, wallet_address) = if let Some(wallet) = request.wallet_address.clone() {
        (KolEntityType::Wallet, wallet.clone(), Some(wallet))
    } else if let Some(handle) = request.twitter_handle {
        let handle = if handle.starts_with('@') {
            handle
        } else {
            format!("@{}", handle)
        };
        (KolEntityType::TwitterHandle, handle, None)
    } else {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Either wallet_address or twitter_handle is required"
            })),
        )
            .into_response();
    };

    {
        let store = KOL_STORE.read().unwrap();
        if store.values().any(|k| k.identifier == identifier) {
            return (
                StatusCode::CONFLICT,
                Json(serde_json::json!({
                    "error": "KOL with this identifier already exists"
                })),
            )
                .into_response();
        }
    }

    let mut kol = KolEntity::new(entity_type, identifier, request.display_name);
    let id = kol.id;

    if let Some(wallet) = wallet_address {
        if state.helius_webhook_client.is_configured() {
            info!("Registering Helius webhook for KOL wallet: {}", wallet);
            let webhook_url = format!(
                "{}/webhooks/helius",
                std::env::var("ARB_FARM_SERVICE_URL").unwrap_or_else(|_| "http://localhost:9007".to_string())
            );

            let config = WebhookConfig {
                webhook_url,
                account_addresses: vec![wallet.clone()],
                transaction_types: vec![TransactionType::Swap],
                webhook_type: WebhookType::Enhanced,
                auth_header: None,
            };

            match state.helius_webhook_client.create_webhook(&config).await {
                Ok(registration) => {
                    info!("Helius webhook registered: {}", registration.webhook_id);
                    kol.linked_wallet = Some(wallet);
                }
                Err(e) => {
                    warn!("Failed to register Helius webhook: {}. KOL added without webhook.", e);
                    kol.linked_wallet = Some(wallet);
                }
            }
        } else {
            kol.linked_wallet = Some(wallet);
        }

        state.kol_tracker.add_kol(
            kol.linked_wallet.as_deref().unwrap_or(&kol.identifier),
            kol.display_name.clone(),
            kol.trust_score.try_into().unwrap_or(50.0),
        ).await;
    }

    {
        let mut store = KOL_STORE.write().unwrap();
        store.insert(id, kol.clone());
    }

    let _ = state.event_tx.send(ArbEvent::new(
        "kol_added",
        crate::events::EventSource::Agent(crate::events::AgentType::CopyTrade),
        "kol",
        serde_json::json!({
            "entity_id": id,
            "identifier": kol.identifier,
            "display_name": kol.display_name,
        }),
    ));

    (StatusCode::CREATED, Json(kol)).into_response()
}

pub async fn get_kol(
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let store = KOL_STORE.read().unwrap();

    match store.get(&id) {
        Some(kol) => (StatusCode::OK, Json(kol.clone())).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "KOL not found"})),
        )
            .into_response(),
    }
}

pub async fn update_kol(
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateKolRequest>,
) -> impl IntoResponse {
    let mut store = KOL_STORE.write().unwrap();

    match store.get_mut(&id) {
        Some(kol) => {
            if let Some(name) = request.display_name {
                kol.display_name = Some(name);
            }
            if let Some(wallet) = request.linked_wallet {
                kol.linked_wallet = Some(wallet);
            }
            if let Some(active) = request.is_active {
                kol.is_active = active;
            }
            kol.updated_at = chrono::Utc::now();

            (StatusCode::OK, Json(kol.clone())).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "KOL not found"})),
        )
            .into_response(),
    }
}

pub async fn delete_kol(
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let mut store = KOL_STORE.write().unwrap();

    match store.remove(&id) {
        Some(_) => {
            let mut trades = KOL_TRADES.write().unwrap();
            trades.remove(&id);
            let mut copies = COPY_TRADES.write().unwrap();
            copies.remove(&id);

            (StatusCode::OK, Json(serde_json::json!({"deleted": true}))).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "KOL not found"})),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct KolTradesQuery {
    pub trade_type: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct KolTradesResponse {
    pub trades: Vec<KolTrade>,
    pub total: usize,
}

pub async fn get_kol_trades(
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<KolTradesQuery>,
) -> impl IntoResponse {
    let store = KOL_STORE.read().unwrap();
    if !store.contains_key(&id) {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "KOL not found"})),
        )
            .into_response();
    }

    let trades_store = KOL_TRADES.read().unwrap();
    let trades = trades_store.get(&id).cloned().unwrap_or_default();

    let filtered: Vec<_> = trades
        .into_iter()
        .filter(|t| {
            if let Some(ref tt) = query.trade_type {
                let trade_type_str = match t.trade_type {
                    KolTradeType::Buy => "buy",
                    KolTradeType::Sell => "sell",
                };
                return trade_type_str == tt;
            }
            true
        })
        .collect();

    let total = filtered.len();
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(50);
    let trades: Vec<_> = filtered.into_iter().skip(offset).take(limit).collect();

    (StatusCode::OK, Json(KolTradesResponse { trades, total })).into_response()
}

pub async fn get_kol_stats(
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let store = KOL_STORE.read().unwrap();

    match store.get(&id) {
        Some(kol) => {
            let trades_store = KOL_TRADES.read().unwrap();
            let trades = trades_store.get(&id).cloned().unwrap_or_default();
            let total_volume: Decimal = trades.iter().map(|t| t.amount_sol).sum();
            let last_trade = trades.last().map(|t| t.detected_at);

            let copies_store = COPY_TRADES.read().unwrap();
            let copies = copies_store.get(&id).cloned().unwrap_or_default();
            let our_copy_count = copies
                .iter()
                .filter(|c| c.status == CopyTradeStatus::Executed)
                .count() as i32;
            let our_copy_profit: i64 = copies
                .iter()
                .filter_map(|c| c.profit_loss_lamports)
                .sum();
            let our_copy_profit_sol =
                Decimal::new(our_copy_profit, 9);

            let stats = KolStats {
                entity_id: kol.id,
                display_name: kol.display_name.clone(),
                identifier: kol.identifier.clone(),
                trust_score: kol.trust_score,
                total_trades: kol.total_trades_tracked,
                profitable_trades: kol.profitable_trades,
                win_rate: kol.win_rate(),
                avg_profit_percent: kol.avg_profit_percent,
                max_drawdown_percent: kol.max_drawdown,
                total_volume_sol: total_volume,
                our_copy_count,
                our_copy_profit_sol,
                copy_trading_enabled: kol.copy_trading_enabled,
                last_trade_at: last_trade,
            };

            (StatusCode::OK, Json(stats)).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "KOL not found"})),
        )
            .into_response(),
    }
}

pub async fn get_trust_breakdown(
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let store = KOL_STORE.read().unwrap();

    match store.get(&id) {
        Some(kol) => {
            let breakdown = kol.trust_score_breakdown();
            (StatusCode::OK, Json(breakdown)).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "KOL not found"})),
        )
            .into_response(),
    }
}

pub async fn enable_copy_trading(
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<EnableCopyRequest>,
) -> impl IntoResponse {
    let mut store = KOL_STORE.write().unwrap();

    match store.get_mut(&id) {
        Some(kol) => {
            kol.copy_trading_enabled = true;

            if let Some(max_pos) = request.max_position_sol {
                kol.copy_config.max_position_sol = max_pos;
            }
            if let Some(delay) = request.delay_ms {
                kol.copy_config.delay_ms = delay;
            }
            if let Some(min_trust) = request.min_trust_score {
                kol.copy_config.min_trust_score = min_trust;
            }
            if let Some(pct) = request.copy_percentage {
                kol.copy_config.copy_percentage = pct;
            }
            if let Some(whitelist) = request.token_whitelist {
                kol.copy_config.token_whitelist = Some(whitelist);
            }
            if let Some(blacklist) = request.token_blacklist {
                kol.copy_config.token_blacklist = Some(blacklist);
            }

            kol.updated_at = chrono::Utc::now();

            (StatusCode::OK, Json(kol.clone())).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "KOL not found"})),
        )
            .into_response(),
    }
}

pub async fn disable_copy_trading(
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let mut store = KOL_STORE.write().unwrap();

    match store.get_mut(&id) {
        Some(kol) => {
            kol.copy_trading_enabled = false;
            kol.updated_at = chrono::Utc::now();

            (StatusCode::OK, Json(kol.clone())).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "KOL not found"})),
        )
            .into_response(),
    }
}

#[derive(Debug, Serialize)]
pub struct ActiveCopiesResponse {
    pub active_copies: Vec<CopyTrade>,
    pub total: usize,
}

pub async fn list_active_copies(State(_state): State<AppState>) -> impl IntoResponse {
    let copies_store = COPY_TRADES.read().unwrap();
    let active: Vec<_> = copies_store
        .values()
        .flatten()
        .filter(|c| {
            c.status == CopyTradeStatus::Pending || c.status == CopyTradeStatus::Executing
        })
        .cloned()
        .collect();

    let total = active.len();

    (
        StatusCode::OK,
        Json(ActiveCopiesResponse {
            active_copies: active,
            total,
        }),
    )
}

#[derive(Debug, Deserialize)]
pub struct CopyHistoryQuery {
    pub status: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct CopyHistoryResponse {
    pub copies: Vec<CopyTrade>,
    pub total: usize,
}

pub async fn get_copy_history(
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<CopyHistoryQuery>,
) -> impl IntoResponse {
    let store = KOL_STORE.read().unwrap();
    if !store.contains_key(&id) {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "KOL not found"})),
        )
            .into_response();
    }

    let copies_store = COPY_TRADES.read().unwrap();
    let copies = copies_store.get(&id).cloned().unwrap_or_default();

    let filtered: Vec<_> = copies
        .into_iter()
        .filter(|c| {
            if let Some(ref status) = query.status {
                let status_str = match c.status {
                    CopyTradeStatus::Pending => "pending",
                    CopyTradeStatus::Executing => "executing",
                    CopyTradeStatus::Executed => "executed",
                    CopyTradeStatus::Failed => "failed",
                    CopyTradeStatus::Skipped => "skipped",
                };
                return status_str == status;
            }
            true
        })
        .collect();

    let total = filtered.len();
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(50);
    let copies: Vec<_> = filtered.into_iter().skip(offset).take(limit).collect();

    (StatusCode::OK, Json(CopyHistoryResponse { copies, total })).into_response()
}

#[derive(Debug, Serialize)]
pub struct CopyStatsResponse {
    pub total_copies: usize,
    pub executed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub total_profit_lamports: i64,
    pub avg_delay_ms: f64,
}

pub async fn get_copy_stats(State(_state): State<AppState>) -> impl IntoResponse {
    let copies_store = COPY_TRADES.read().unwrap();
    let all_copies: Vec<_> = copies_store.values().flatten().cloned().collect();

    let total_copies = all_copies.len();
    let executed = all_copies
        .iter()
        .filter(|c| c.status == CopyTradeStatus::Executed)
        .count();
    let failed = all_copies
        .iter()
        .filter(|c| c.status == CopyTradeStatus::Failed)
        .count();
    let skipped = all_copies
        .iter()
        .filter(|c| c.status == CopyTradeStatus::Skipped)
        .count();

    let total_profit: i64 = all_copies
        .iter()
        .filter_map(|c| c.profit_loss_lamports)
        .sum();

    let avg_delay = if executed > 0 {
        all_copies
            .iter()
            .filter(|c| c.status == CopyTradeStatus::Executed)
            .map(|c| c.delay_ms as f64)
            .sum::<f64>()
            / executed as f64
    } else {
        0.0
    };

    (
        StatusCode::OK,
        Json(CopyStatsResponse {
            total_copies,
            executed,
            failed,
            skipped,
            total_profit_lamports: total_profit,
            avg_delay_ms: avg_delay,
        }),
    )
}

#[derive(Debug, Serialize)]
pub struct DiscoveryStatusResponse {
    pub is_running: bool,
    pub total_wallets_analyzed: u64,
    pub total_kols_discovered: u64,
    pub last_scan_at: Option<String>,
    pub scan_interval_ms: u64,
}

pub async fn get_discovery_status(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let stats = state.kol_discovery.get_stats().await;

    Json(DiscoveryStatusResponse {
        is_running: stats.is_running,
        total_wallets_analyzed: stats.total_wallets_analyzed,
        total_kols_discovered: stats.total_kols_discovered,
        last_scan_at: stats.last_scan_at.map(|t| t.to_rfc3339()),
        scan_interval_ms: stats.scan_interval_ms,
    })
}

pub async fn start_discovery(
    State(state): State<AppState>,
) -> impl IntoResponse {
    state.kol_discovery.start().await;

    (StatusCode::OK, Json(serde_json::json!({
        "status": "started",
        "message": "KOL discovery agent started"
    })))
}

pub async fn stop_discovery(
    State(state): State<AppState>,
) -> impl IntoResponse {
    state.kol_discovery.stop().await;

    (StatusCode::OK, Json(serde_json::json!({
        "status": "stopped",
        "message": "KOL discovery agent stopped"
    })))
}

#[derive(Debug, Deserialize)]
pub struct DiscoveredKolsQuery {
    pub min_trust_score: Option<f64>,
    pub min_win_rate: Option<f64>,
    pub source: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct DiscoveredKolsResponse {
    pub discovered: Vec<crate::agents::DiscoveredKol>,
    pub total: usize,
}

pub async fn list_discovered_kols(
    State(state): State<AppState>,
    Query(query): Query<DiscoveredKolsQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(50);
    let mut discovered = state.kol_discovery.get_discovered_kols(Some(limit)).await;

    if let Some(min_trust) = query.min_trust_score {
        discovered.retain(|k| k.trust_score >= min_trust);
    }
    if let Some(min_wr) = query.min_win_rate {
        discovered.retain(|k| k.win_rate >= min_wr);
    }
    if let Some(ref source) = query.source {
        discovered.retain(|k| k.source == *source);
    }

    let total = discovered.len();

    Json(DiscoveredKolsResponse { discovered, total })
}

pub async fn scan_for_kols_now(
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.kol_discovery.scan_once().await {
        Ok(discovered) => {
            let count = discovered.len();

            for kol in &discovered {
                state.kol_tracker.add_kol(
                    &kol.wallet_address,
                    kol.display_name.clone(),
                    kol.trust_score,
                ).await;
            }

            (StatusCode::OK, Json(serde_json::json!({
                "discovered": discovered,
                "count": count,
                "message": format!("Discovered {} new KOLs", count)
            }))).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": e.to_string()
        }))).into_response(),
    }
}

pub async fn promote_discovered_kol(
    State(state): State<AppState>,
    Path(wallet_address): Path<String>,
) -> impl IntoResponse {
    let discovered = state.kol_discovery.get_discovered_kols(None).await;

    let kol = discovered.iter().find(|k| k.wallet_address == wallet_address);

    match kol {
        Some(discovered_kol) => {
            let request = AddKolRequest {
                wallet_address: Some(discovered_kol.wallet_address.clone()),
                twitter_handle: None,
                display_name: discovered_kol.display_name.clone(),
            };

            let (entity_type, identifier) = (KolEntityType::Wallet, discovered_kol.wallet_address.clone());

            {
                let store = KOL_STORE.read().unwrap();
                if store.values().any(|k| k.identifier == identifier) {
                    return (
                        StatusCode::CONFLICT,
                        Json(serde_json::json!({
                            "error": "KOL already promoted",
                            "wallet_address": wallet_address
                        })),
                    ).into_response();
                }
            }

            let mut kol_entity = KolEntity::new(entity_type, identifier.clone(), discovered_kol.display_name.clone());
            kol_entity.linked_wallet = Some(discovered_kol.wallet_address.clone());
            kol_entity.trust_score = rust_decimal::Decimal::from_f64_retain(discovered_kol.trust_score)
                .unwrap_or(rust_decimal::Decimal::new(500, 1));
            kol_entity.total_trades_tracked = discovered_kol.total_trades as i32;
            kol_entity.profitable_trades = discovered_kol.winning_trades as i32;

            let id = kol_entity.id;

            {
                let mut store = KOL_STORE.write().unwrap();
                store.insert(id, kol_entity.clone());
            }

            state.kol_tracker.add_kol(
                &discovered_kol.wallet_address,
                discovered_kol.display_name.clone(),
                discovered_kol.trust_score,
            ).await;

            let _ = state.event_tx.send(ArbEvent::new(
                "kol_promoted",
                crate::events::EventSource::Agent(crate::events::AgentType::CopyTrade),
                "kol",
                serde_json::json!({
                    "entity_id": id,
                    "wallet_address": discovered_kol.wallet_address,
                    "trust_score": discovered_kol.trust_score,
                    "source": discovered_kol.source,
                }),
            ));

            (StatusCode::CREATED, Json(kol_entity)).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Discovered KOL not found",
                "wallet_address": wallet_address
            })),
        ).into_response(),
    }
}
