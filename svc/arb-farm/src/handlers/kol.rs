use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;

use crate::database::repositories::kol::{
    CreateCopyTradeRecord, CreateKolEntityRecord, CreateKolTradeRecord,
    UpdateCopyTradeRecord, UpdateKolEntityRecord,
};
use crate::events::{ArbEvent, topics::kol as kol_topics};
use crate::models::{
    AddKolRequest, CopyTradeConfig, CopyTradeStatus, EnableCopyRequest,
    KolEntity, KolEntityType, KolStats, KolTrade, KolTradeType, TrustScoreBreakdown,
    UpdateKolRequest,
};
use crate::server::AppState;
use crate::webhooks::helius::{WebhookConfig, TransactionType, WebhookType};

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
    State(state): State<AppState>,
    Query(query): Query<ListKolQuery>,
) -> impl IntoResponse {
    let offset = query.offset.unwrap_or(0) as i64;
    let limit = query.limit.unwrap_or(50) as i64;

    match state.kol_repo.list_entities(
        query.is_active,
        query.copy_enabled,
        query.min_trust_score,
        limit,
        offset,
    ).await {
        Ok(records) => {
            let total = match state.kol_repo.count_entities(
                query.is_active,
                query.copy_enabled,
                query.min_trust_score,
            ).await {
                Ok(count) => count as usize,
                Err(_) => records.len(),
            };

            let kols: Vec<KolEntity> = records.into_iter().map(|r| {
                let copy_config = r.copy_config_parsed();
                let entity_type = r.entity_type_enum();
                KolEntity {
                    id: r.id,
                    entity_type,
                    identifier: r.identifier,
                    display_name: r.display_name,
                    linked_wallet: r.linked_wallet,
                    trust_score: r.trust_score,
                    total_trades_tracked: r.total_trades_tracked,
                    profitable_trades: r.profitable_trades,
                    avg_profit_percent: r.avg_profit_percent,
                    max_drawdown: r.max_drawdown,
                    copy_trading_enabled: r.copy_trading_enabled,
                    copy_config,
                    is_active: r.is_active,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                }
            }).collect();

            (StatusCode::OK, Json(KolListResponse { kols, total }))
        }
        Err(e) => {
            warn!("Failed to list KOLs: {}", e);
            (StatusCode::OK, Json(KolListResponse { kols: vec![], total: 0 }))
        }
    }
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

    if let Ok(Some(_)) = state.kol_repo.get_by_identifier(&identifier).await {
        return (
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "error": "KOL with this identifier already exists"
            })),
        )
            .into_response();
    }

    let create_record = CreateKolEntityRecord {
        entity_type: entity_type.clone(),
        identifier: identifier.clone(),
        display_name: request.display_name.clone(),
        linked_wallet: wallet_address.clone(),
        discovery_source: None,
    };

    match state.kol_repo.create_entity(create_record).await {
        Ok(record) => {
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
                        }
                        Err(e) => {
                            warn!("Failed to register Helius webhook: {}. KOL added without webhook.", e);
                        }
                    }
                }

                state.kol_tracker.add_kol(
                    &wallet,
                    request.display_name.clone(),
                    record.trust_score.try_into().unwrap_or(50.0),
                ).await;
            }

            let _ = state.event_tx.send(ArbEvent::new(
                "kol_added",
                crate::events::EventSource::Agent(crate::events::AgentType::CopyTrade),
                kol_topics::ADDED,
                serde_json::json!({
                    "entity_id": record.id,
                    "identifier": record.identifier,
                    "display_name": record.display_name,
                }),
            ));

            let copy_config = record.copy_config_parsed();
            let entity_type = record.entity_type_enum();
            let kol = KolEntity {
                id: record.id,
                entity_type,
                identifier: record.identifier,
                display_name: record.display_name,
                linked_wallet: record.linked_wallet,
                trust_score: record.trust_score,
                total_trades_tracked: record.total_trades_tracked,
                profitable_trades: record.profitable_trades,
                avg_profit_percent: record.avg_profit_percent,
                max_drawdown: record.max_drawdown,
                copy_trading_enabled: record.copy_trading_enabled,
                copy_config,
                is_active: record.is_active,
                created_at: record.created_at,
                updated_at: record.updated_at,
            };

            (StatusCode::CREATED, Json(serde_json::to_value(kol).unwrap_or_default())).into_response()
        }
        Err(e) => {
            warn!("Failed to create KOL: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            ).into_response()
        }
    }
}

pub async fn get_kol(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.kol_repo.get_entity(id).await {
        Ok(Some(record)) => {
            let copy_config = record.copy_config_parsed();
            let entity_type = record.entity_type_enum();
            let kol = KolEntity {
                id: record.id,
                entity_type,
                identifier: record.identifier,
                display_name: record.display_name,
                linked_wallet: record.linked_wallet,
                trust_score: record.trust_score,
                total_trades_tracked: record.total_trades_tracked,
                profitable_trades: record.profitable_trades,
                avg_profit_percent: record.avg_profit_percent,
                max_drawdown: record.max_drawdown,
                copy_trading_enabled: record.copy_trading_enabled,
                copy_config,
                is_active: record.is_active,
                created_at: record.created_at,
                updated_at: record.updated_at,
            };
            (StatusCode::OK, Json(serde_json::to_value(kol).unwrap_or_default())).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "KOL not found"})),
        ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        ).into_response(),
    }
}

pub async fn update_kol(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateKolRequest>,
) -> impl IntoResponse {
    let update = UpdateKolEntityRecord {
        display_name: request.display_name,
        linked_wallet: request.linked_wallet,
        is_active: request.is_active,
        ..Default::default()
    };

    match state.kol_repo.update_entity(id, update).await {
        Ok(record) => {
            let copy_config = record.copy_config_parsed();
            let entity_type = record.entity_type_enum();
            let kol = KolEntity {
                id: record.id,
                entity_type,
                identifier: record.identifier,
                display_name: record.display_name,
                linked_wallet: record.linked_wallet,
                trust_score: record.trust_score,
                total_trades_tracked: record.total_trades_tracked,
                profitable_trades: record.profitable_trades,
                avg_profit_percent: record.avg_profit_percent,
                max_drawdown: record.max_drawdown,
                copy_trading_enabled: record.copy_trading_enabled,
                copy_config,
                is_active: record.is_active,
                created_at: record.created_at,
                updated_at: record.updated_at,
            };
            (StatusCode::OK, Json(serde_json::to_value(kol).unwrap_or_default())).into_response()
        }
        Err(crate::error::AppError::NotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "KOL not found"})),
        ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        ).into_response(),
    }
}

pub async fn delete_kol(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // First, get the KOL entity to retrieve wallet address for webhook cleanup
    let wallet_to_cleanup = match state.kol_repo.get_entity(id).await {
        Ok(Some(record)) => record.linked_wallet,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "KOL not found"})),
            ).into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            ).into_response();
        }
    };

    match state.kol_repo.delete_entity(id).await {
        Ok(_) => {
            // Delete associated Helius webhook if wallet was tracked
            if let Some(wallet) = wallet_to_cleanup {
                if state.helius_webhook_client.is_configured() {
                    match state.helius_webhook_client.find_webhook_for_address(&wallet).await {
                        Ok(Some(webhook_id)) => {
                            match state.helius_webhook_client.delete_webhook(&webhook_id).await {
                                Ok(()) => {
                                    info!(
                                        kol_id = %id,
                                        wallet = %wallet,
                                        webhook_id = %webhook_id,
                                        "🗑️ Helius webhook deleted for removed KOL"
                                    );
                                }
                                Err(e) => {
                                    warn!(
                                        kol_id = %id,
                                        wallet = %wallet,
                                        webhook_id = %webhook_id,
                                        error = %e,
                                        "⚠️ Failed to delete Helius webhook for removed KOL"
                                    );
                                }
                            }
                        }
                        Ok(None) => {
                            // No webhook found - that's fine
                        }
                        Err(e) => {
                            warn!(
                                kol_id = %id,
                                wallet = %wallet,
                                error = %e,
                                "⚠️ Failed to lookup webhook for removed KOL"
                            );
                        }
                    }
                }

                // Also remove from in-memory tracker
                state.kol_tracker.remove_kol(&wallet).await;
            }

            let _ = state.event_tx.send(ArbEvent::new(
                "kol_removed",
                crate::events::EventSource::Agent(crate::events::AgentType::CopyTrade),
                kol_topics::REMOVED,
                serde_json::json!({
                    "entity_id": id,
                }),
            ));
            (StatusCode::OK, Json(serde_json::json!({"deleted": true}))).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        ).into_response(),
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
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<KolTradesQuery>,
) -> impl IntoResponse {
    if let Ok(None) = state.kol_repo.get_entity(id).await {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "KOL not found"})),
        ).into_response();
    }

    let trade_type = query.trade_type.as_ref().map(|t| {
        match t.to_lowercase().as_str() {
            "buy" => KolTradeType::Buy,
            "sell" => KolTradeType::Sell,
            _ => KolTradeType::Buy,
        }
    });

    let offset = query.offset.unwrap_or(0) as i64;
    let limit = query.limit.unwrap_or(50) as i64;

    match state.kol_repo.get_trades(id, trade_type, limit, offset).await {
        Ok(records) => {
            let trades: Vec<KolTrade> = records.into_iter().map(|r| {
                let trade_type = r.trade_type_enum();
                KolTrade {
                    id: r.id,
                    entity_id: r.entity_id,
                    tx_signature: r.tx_signature,
                    trade_type,
                    token_mint: r.token_mint,
                    token_symbol: r.token_symbol,
                    amount_sol: r.amount_sol.unwrap_or_default(),
                    token_amount: r.token_amount,
                    price_at_trade: r.price_at_trade,
                    detected_at: r.detected_at,
                }
            }).collect();

            let total = trades.len();
            (StatusCode::OK, Json(KolTradesResponse { trades, total })).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        ).into_response(),
    }
}

pub async fn get_kol_stats(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.kol_repo.get_entity_stats(id).await {
        Ok(Some(stats)) => {
            let kol_stats = KolStats {
                entity_id: stats.entity_id,
                display_name: stats.display_name,
                identifier: stats.identifier,
                trust_score: stats.trust_score,
                total_trades: stats.total_trades as i32,
                profitable_trades: stats.profitable_trades as i32,
                win_rate: stats.win_rate,
                avg_profit_percent: stats.avg_profit_percent,
                max_drawdown_percent: stats.max_drawdown,
                total_volume_sol: stats.total_volume_sol,
                our_copy_count: stats.copy_count as i32,
                our_copy_profit_sol: Decimal::new(stats.copy_profit_lamports, 9),
                copy_trading_enabled: stats.copy_trading_enabled,
                last_trade_at: stats.last_trade_at,
            };
            (StatusCode::OK, Json(kol_stats)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "KOL not found"})),
        ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        ).into_response(),
    }
}

pub async fn get_trust_breakdown(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.kol_repo.get_entity(id).await {
        Ok(Some(record)) => {
            let copy_config = record.copy_config_parsed();
            let entity_type = record.entity_type_enum();
            let kol = KolEntity {
                id: record.id,
                entity_type,
                identifier: record.identifier,
                display_name: record.display_name,
                linked_wallet: record.linked_wallet,
                trust_score: record.trust_score,
                total_trades_tracked: record.total_trades_tracked,
                profitable_trades: record.profitable_trades,
                avg_profit_percent: record.avg_profit_percent,
                max_drawdown: record.max_drawdown,
                copy_trading_enabled: record.copy_trading_enabled,
                copy_config,
                is_active: record.is_active,
                created_at: record.created_at,
                updated_at: record.updated_at,
            };
            let breakdown = kol.trust_score_breakdown();
            (StatusCode::OK, Json(breakdown)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "KOL not found"})),
        ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        ).into_response(),
    }
}

pub async fn enable_copy_trading(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<EnableCopyRequest>,
) -> impl IntoResponse {
    let current = match state.kol_repo.get_entity(id).await {
        Ok(Some(r)) => r,
        Ok(None) => return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "KOL not found"})),
        ).into_response(),
        Err(e) => return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        ).into_response(),
    };

    let mut copy_config = current.copy_config_parsed();
    if let Some(max_pos) = request.max_position_sol {
        copy_config.max_position_sol = max_pos;
    }
    if let Some(delay) = request.delay_ms {
        copy_config.delay_ms = delay;
    }
    if let Some(min_trust) = request.min_trust_score {
        copy_config.min_trust_score = min_trust;
    }
    if let Some(pct) = request.copy_percentage {
        copy_config.copy_percentage = pct;
    }
    if let Some(whitelist) = request.token_whitelist {
        copy_config.token_whitelist = Some(whitelist);
    }
    if let Some(blacklist) = request.token_blacklist {
        copy_config.token_blacklist = Some(blacklist);
    }

    let update = UpdateKolEntityRecord {
        copy_trading_enabled: Some(true),
        copy_config: Some(copy_config),
        ..Default::default()
    };

    match state.kol_repo.update_entity(id, update).await {
        Ok(record) => {
            let copy_config = record.copy_config_parsed();
            let entity_type = record.entity_type_enum();
            let kol = KolEntity {
                id: record.id,
                entity_type,
                identifier: record.identifier,
                display_name: record.display_name,
                linked_wallet: record.linked_wallet,
                trust_score: record.trust_score,
                total_trades_tracked: record.total_trades_tracked,
                profitable_trades: record.profitable_trades,
                avg_profit_percent: record.avg_profit_percent,
                max_drawdown: record.max_drawdown,
                copy_trading_enabled: record.copy_trading_enabled,
                copy_config,
                is_active: record.is_active,
                created_at: record.created_at,
                updated_at: record.updated_at,
            };
            (StatusCode::OK, Json(serde_json::to_value(kol).unwrap_or_default())).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        ).into_response(),
    }
}

pub async fn disable_copy_trading(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let update = UpdateKolEntityRecord {
        copy_trading_enabled: Some(false),
        ..Default::default()
    };

    match state.kol_repo.update_entity(id, update).await {
        Ok(record) => {
            let copy_config = record.copy_config_parsed();
            let entity_type = record.entity_type_enum();
            let kol = KolEntity {
                id: record.id,
                entity_type,
                identifier: record.identifier,
                display_name: record.display_name,
                linked_wallet: record.linked_wallet,
                trust_score: record.trust_score,
                total_trades_tracked: record.total_trades_tracked,
                profitable_trades: record.profitable_trades,
                avg_profit_percent: record.avg_profit_percent,
                max_drawdown: record.max_drawdown,
                copy_trading_enabled: record.copy_trading_enabled,
                copy_config,
                is_active: record.is_active,
                created_at: record.created_at,
                updated_at: record.updated_at,
            };
            (StatusCode::OK, Json(serde_json::to_value(kol).unwrap_or_default())).into_response()
        }
        Err(crate::error::AppError::NotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "KOL not found"})),
        ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        ).into_response(),
    }
}

#[derive(Debug, Serialize)]
pub struct ActiveCopiesResponse {
    pub active_copies: Vec<crate::models::CopyTrade>,
    pub total: usize,
}

pub async fn list_active_copies(State(state): State<AppState>) -> impl IntoResponse {
    let kols = match state.kol_repo.list_entities(
        Some(true),
        Some(true),
        None,
        1000,
        0,
    ).await {
        Ok(kols) => kols,
        Err(_) => vec![],
    };

    let mut active: Vec<crate::models::CopyTrade> = Vec::new();
    for kol in kols {
        if let Ok(copies) = state.kol_repo.get_copy_history(kol.id, Some(CopyTradeStatus::Pending), 100, 0).await {
            for record in copies {
                let status = record.status_enum();
                active.push(crate::models::CopyTrade {
                    id: record.id,
                    entity_id: record.entity_id,
                    kol_trade_id: record.kol_trade_id,
                    our_tx_signature: record.our_tx_signature,
                    copy_amount_sol: record.copy_amount_sol.unwrap_or_default(),
                    delay_ms: record.delay_ms.unwrap_or(0) as u64,
                    profit_loss_lamports: record.profit_loss_lamports,
                    status,
                    skip_reason: None,
                    executed_at: record.executed_at,
                    created_at: record.created_at,
                });
            }
        }
        if let Ok(copies) = state.kol_repo.get_copy_history(kol.id, Some(CopyTradeStatus::Executing), 100, 0).await {
            for record in copies {
                let status = record.status_enum();
                active.push(crate::models::CopyTrade {
                    id: record.id,
                    entity_id: record.entity_id,
                    kol_trade_id: record.kol_trade_id,
                    our_tx_signature: record.our_tx_signature,
                    copy_amount_sol: record.copy_amount_sol.unwrap_or_default(),
                    delay_ms: record.delay_ms.unwrap_or(0) as u64,
                    profit_loss_lamports: record.profit_loss_lamports,
                    status,
                    skip_reason: None,
                    executed_at: record.executed_at,
                    created_at: record.created_at,
                });
            }
        }
    }

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
    pub copies: Vec<crate::models::CopyTrade>,
    pub total: usize,
}

pub async fn get_copy_history(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<CopyHistoryQuery>,
) -> impl IntoResponse {
    if let Ok(None) = state.kol_repo.get_entity(id).await {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "KOL not found"})),
        ).into_response();
    }

    let status = query.status.as_ref().map(|s| {
        match s.to_lowercase().as_str() {
            "pending" => CopyTradeStatus::Pending,
            "executing" => CopyTradeStatus::Executing,
            "executed" => CopyTradeStatus::Executed,
            "failed" => CopyTradeStatus::Failed,
            "skipped" => CopyTradeStatus::Skipped,
            _ => CopyTradeStatus::Pending,
        }
    });

    let offset = query.offset.unwrap_or(0) as i64;
    let limit = query.limit.unwrap_or(50) as i64;

    match state.kol_repo.get_copy_history(id, status, limit, offset).await {
        Ok(records) => {
            let copies: Vec<crate::models::CopyTrade> = records.into_iter().map(|r| {
                let status = r.status_enum();
                crate::models::CopyTrade {
                    id: r.id,
                    entity_id: r.entity_id,
                    kol_trade_id: r.kol_trade_id,
                    our_tx_signature: r.our_tx_signature,
                    copy_amount_sol: r.copy_amount_sol.unwrap_or_default(),
                    delay_ms: r.delay_ms.unwrap_or(0) as u64,
                    profit_loss_lamports: r.profit_loss_lamports,
                    status,
                    skip_reason: None,
                    executed_at: r.executed_at,
                    created_at: r.created_at,
                }
            }).collect();

            let total = copies.len();
            (StatusCode::OK, Json(CopyHistoryResponse { copies, total })).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        ).into_response(),
    }
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

pub async fn get_copy_stats(State(state): State<AppState>) -> impl IntoResponse {
    match state.kol_repo.get_copy_stats().await {
        Ok(stats) => (
            StatusCode::OK,
            Json(CopyStatsResponse {
                total_copies: stats.total_copies,
                executed: stats.executed,
                failed: stats.failed,
                skipped: stats.skipped,
                total_profit_lamports: stats.total_profit_lamports,
                avg_delay_ms: stats.avg_delay_ms,
            }),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(CopyStatsResponse {
                total_copies: 0,
                executed: 0,
                failed: 0,
                skipped: 0,
                total_profit_lamports: 0,
                avg_delay_ms: 0.0,
            }),
        ),
    }
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
            if let Ok(Some(_)) = state.kol_repo.get_by_identifier(&discovered_kol.wallet_address).await {
                return (
                    StatusCode::CONFLICT,
                    Json(serde_json::json!({
                        "error": "KOL already promoted",
                        "wallet_address": wallet_address
                    })),
                ).into_response();
            }

            let create_record = CreateKolEntityRecord {
                entity_type: KolEntityType::Wallet,
                identifier: discovered_kol.wallet_address.clone(),
                display_name: discovered_kol.display_name.clone(),
                linked_wallet: Some(discovered_kol.wallet_address.clone()),
                discovery_source: Some(discovered_kol.source.clone()),
            };

            match state.kol_repo.create_entity(create_record).await {
                Ok(record) => {
                    let trust_score = Decimal::from_f64_retain(discovered_kol.trust_score)
                        .unwrap_or(Decimal::new(500, 1));

                    let update = UpdateKolEntityRecord {
                        trust_score: Some(trust_score),
                        total_trades_tracked: Some(discovered_kol.total_trades as i32),
                        profitable_trades: Some(discovered_kol.winning_trades as i32),
                        ..Default::default()
                    };

                    let _ = state.kol_repo.update_entity(record.id, update).await;

                    // Register Helius webhook for the promoted KOL's wallet
                    if state.helius_webhook_client.is_configured() {
                        info!(
                            kol_id = %record.id,
                            wallet = %discovered_kol.wallet_address,
                            "Registering Helius webhook for promoted KOL"
                        );
                        let webhook_url = format!(
                            "{}/api/arb/webhooks/helius",
                            state.config.erebus_url
                        );

                        let config = WebhookConfig {
                            webhook_url,
                            account_addresses: vec![discovered_kol.wallet_address.clone()],
                            transaction_types: vec![TransactionType::Swap],
                            webhook_type: WebhookType::Enhanced,
                            auth_header: state.config.helius_webhook_auth_token.clone(),
                        };

                        match state.helius_webhook_client.create_webhook(&config).await {
                            Ok(registration) => {
                                info!(
                                    kol_id = %record.id,
                                    wallet = %discovered_kol.wallet_address,
                                    webhook_id = %registration.webhook_id,
                                    "✅ Helius webhook registered for promoted KOL"
                                );
                            }
                            Err(e) => {
                                warn!(
                                    kol_id = %record.id,
                                    wallet = %discovered_kol.wallet_address,
                                    error = %e,
                                    "⚠️ Failed to register Helius webhook for promoted KOL. Copy trading may not work."
                                );
                            }
                        }
                    } else {
                        warn!(
                            kol_id = %record.id,
                            wallet = %discovered_kol.wallet_address,
                            "⚠️ Helius webhooks not configured - promoted KOL trades will not be detected"
                        );
                    }

                    state.kol_tracker.add_kol(
                        &discovered_kol.wallet_address,
                        discovered_kol.display_name.clone(),
                        discovered_kol.trust_score,
                    ).await;

                    let _ = state.event_tx.send(ArbEvent::new(
                        "kol_promoted",
                        crate::events::EventSource::Agent(crate::events::AgentType::CopyTrade),
                        kol_topics::PROMOTED,
                        serde_json::json!({
                            "entity_id": record.id,
                            "wallet_address": discovered_kol.wallet_address,
                            "trust_score": discovered_kol.trust_score,
                            "source": discovered_kol.source,
                        }),
                    ));

                    let copy_config = record.copy_config_parsed();
                    let entity_type = record.entity_type_enum();
                    let kol = KolEntity {
                        id: record.id,
                        entity_type,
                        identifier: record.identifier,
                        display_name: record.display_name,
                        linked_wallet: record.linked_wallet,
                        trust_score: trust_score,
                        total_trades_tracked: discovered_kol.total_trades as i32,
                        profitable_trades: discovered_kol.winning_trades as i32,
                        avg_profit_percent: None,
                        max_drawdown: None,
                        copy_trading_enabled: record.copy_trading_enabled,
                        copy_config,
                        is_active: record.is_active,
                        created_at: record.created_at,
                        updated_at: record.updated_at,
                    };

                    (StatusCode::CREATED, Json(serde_json::to_value(kol).unwrap_or_default())).into_response()
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": e.to_string()})),
                ).into_response(),
            }
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

pub async fn record_kol_trade(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(trade): Json<CreateKolTradeRecord>,
) -> impl IntoResponse {
    if let Ok(None) = state.kol_repo.get_entity(id).await {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "KOL not found"})),
        ).into_response();
    }

    match state.kol_repo.record_trade(trade).await {
        Ok(record) => {
            let trade_type = record.trade_type_enum();
            let trade = KolTrade {
                id: record.id,
                entity_id: record.entity_id,
                tx_signature: record.tx_signature,
                trade_type,
                token_mint: record.token_mint,
                token_symbol: record.token_symbol,
                amount_sol: record.amount_sol.unwrap_or_default(),
                token_amount: record.token_amount,
                price_at_trade: record.price_at_trade,
                detected_at: record.detected_at,
            };
            (StatusCode::CREATED, Json(serde_json::to_value(trade).unwrap_or_default())).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        ).into_response(),
    }
}
