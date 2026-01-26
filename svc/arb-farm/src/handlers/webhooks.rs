use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::database::repositories::kol::{CreateKolTradeRecord};
use crate::error::{AppError, AppResult};
use crate::events::{kol as kol_topics, ArbEvent};
use crate::models::KolTradeType;
use crate::server::AppState;
use crate::webhooks::helius::{
    EnhancedTransactionEvent, HeliusWebhookPayload, TransactionType, WebhookConfig, WebhookType,
};
use crate::webhooks::parser::{KOLTradeSignal, ParsedSwap, TransactionParser};

#[derive(Debug, Serialize)]
pub struct WebhookStatusResponse {
    pub helius_configured: bool,
    pub active_webhooks: usize,
    pub processed_events: u64,
    pub last_event_at: Option<String>,
}

pub async fn get_webhook_status(
    State(state): State<AppState>,
) -> AppResult<Json<WebhookStatusResponse>> {
    let is_configured = state.config.helius_api_key.is_some();

    Ok(Json(WebhookStatusResponse {
        helius_configured: is_configured,
        active_webhooks: 0,
        processed_events: 0,
        last_event_at: None,
    }))
}

#[derive(Debug, Deserialize)]
pub struct RegisterWebhookRequest {
    pub wallet_addresses: Vec<String>,
    pub webhook_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RegisterWebhookResponse {
    pub success: bool,
    pub webhook_id: Option<String>,
    pub registered_addresses: Vec<String>,
    pub error: Option<String>,
}

pub async fn register_webhook(
    State(state): State<AppState>,
    Json(request): Json<RegisterWebhookRequest>,
) -> AppResult<Json<RegisterWebhookResponse>> {
    if state.config.helius_api_key.is_none() {
        return Ok(Json(RegisterWebhookResponse {
            success: false,
            webhook_id: None,
            registered_addresses: vec![],
            error: Some("Helius API key not configured".to_string()),
        }));
    }

    let webhook_url = request.webhook_url.unwrap_or_else(|| {
        format!("{}/api/arb/webhooks/helius", state.config.erebus_url)
    });

    let config = WebhookConfig {
        webhook_url,
        webhook_type: WebhookType::Enhanced,
        transaction_types: vec![TransactionType::Swap, TransactionType::Transfer],
        account_addresses: request.wallet_addresses.clone(),
        auth_header: None,
    };

    match state.helius_webhook_client.create_webhook(&config).await {
        Ok(registration) => Ok(Json(RegisterWebhookResponse {
            success: true,
            webhook_id: Some(registration.webhook_id),
            registered_addresses: request.wallet_addresses,
            error: None,
        })),
        Err(e) => Ok(Json(RegisterWebhookResponse {
            success: false,
            webhook_id: None,
            registered_addresses: vec![],
            error: Some(e.to_string()),
        })),
    }
}

#[derive(Debug, Serialize)]
pub struct ListWebhooksResponse {
    pub webhooks: Vec<WebhookSummary>,
}

#[derive(Debug, Serialize)]
pub struct WebhookSummary {
    pub webhook_id: String,
    pub wallet_address: String,
    pub is_active: bool,
    pub created_at: String,
}

pub async fn list_webhooks(
    State(state): State<AppState>,
) -> AppResult<Json<ListWebhooksResponse>> {
    if state.config.helius_api_key.is_none() {
        return Ok(Json(ListWebhooksResponse { webhooks: vec![] }));
    }

    match state.helius_webhook_client.list_webhooks().await {
        Ok(registrations) => {
            let webhooks = registrations
                .into_iter()
                .map(|r| WebhookSummary {
                    webhook_id: r.webhook_id,
                    wallet_address: r.wallet_address,
                    is_active: r.is_active,
                    created_at: r.created_at.to_rfc3339(),
                })
                .collect();

            Ok(Json(ListWebhooksResponse { webhooks }))
        }
        Err(e) => {
            tracing::error!("Failed to list webhooks: {}", e);
            Ok(Json(ListWebhooksResponse { webhooks: vec![] }))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct DeleteWebhookRequest {
    pub webhook_id: String,
}

#[derive(Debug, Serialize)]
pub struct DeleteWebhookResponse {
    pub success: bool,
    pub error: Option<String>,
}

pub async fn delete_webhook(
    State(state): State<AppState>,
    Json(request): Json<DeleteWebhookRequest>,
) -> AppResult<Json<DeleteWebhookResponse>> {
    match state.helius_webhook_client.delete_webhook(&request.webhook_id).await {
        Ok(()) => Ok(Json(DeleteWebhookResponse {
            success: true,
            error: None,
        })),
        Err(e) => Ok(Json(DeleteWebhookResponse {
            success: false,
            error: Some(e.to_string()),
        })),
    }
}

pub async fn receive_helius_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<Vec<EnhancedTransactionEvent>>,
) -> StatusCode {
    // Verify webhook authentication if configured
    if let Some(expected_token) = &state.config.helius_webhook_auth_token {
        let auth_header = headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.trim_start_matches("Bearer ").trim());

        match auth_header {
            Some(token) if token == expected_token => {
                // Auth successful
            }
            Some(_) => {
                tracing::warn!("🔒 Webhook auth failed: invalid token");
                return StatusCode::UNAUTHORIZED;
            }
            None => {
                tracing::warn!("🔒 Webhook auth failed: no Authorization header");
                return StatusCode::UNAUTHORIZED;
            }
        }
    } else {
        tracing::error!("🔒 HELIUS_WEBHOOK_AUTH_TOKEN not configured - rejecting webhook");
        return StatusCode::UNAUTHORIZED;
    }

    tracing::info!("Received Helius webhook with {} events", payload.len());

    for event in payload {
        if TransactionParser::is_swap_transaction(&event) {
            if let Some(swap) = TransactionParser::parse_swap(&event) {
                tracing::info!(
                    "Parsed swap: {} -> {} ({} -> {})",
                    swap.input_mint,
                    swap.output_mint,
                    swap.input_amount,
                    swap.output_amount
                );

                // Look up the KOL entity from DB by wallet address
                let kol_entity = match state.kol_repo.get_by_identifier(&swap.wallet_address).await {
                    Ok(Some(entity)) => entity,
                    Ok(None) => {
                        tracing::debug!(
                            wallet = %swap.wallet_address,
                            "Swap from untracked wallet, skipping copy trade event"
                        );
                        // Still record in memory tracker for stats
                        if let Err(e) = state.kol_tracker.record_trade(swap).await {
                            tracing::error!("Failed to record KOL trade: {}", e);
                        }
                        continue;
                    }
                    Err(e) => {
                        tracing::error!("Failed to lookup KOL entity: {}", e);
                        continue;
                    }
                };

                // Determine trade type: buy if SOL input, sell if SOL output
                let (trade_type, token_mint) = if swap.is_native_input {
                    (KolTradeType::Buy, swap.output_mint.clone())
                } else if swap.is_native_output {
                    (KolTradeType::Sell, swap.input_mint.clone())
                } else {
                    tracing::debug!(
                        signature = %swap.signature,
                        "Token-to-token swap (no SOL), skipping"
                    );
                    continue;
                };

                let value_sol = TransactionParser::calculate_swap_value_sol(&swap);

                // Record the KOL trade in DB to get kol_trade_id
                let kol_trade = match state.kol_repo.record_trade(CreateKolTradeRecord {
                    entity_id: kol_entity.id,
                    tx_signature: swap.signature.clone(),
                    trade_type: trade_type.clone(),
                    token_mint: token_mint.clone(),
                    token_symbol: None,
                    amount_sol: Decimal::from_f64_retain(value_sol).unwrap_or_default(),
                    token_amount: if trade_type == KolTradeType::Buy {
                        Some(Decimal::from(swap.output_amount))
                    } else {
                        Some(Decimal::from(swap.input_amount))
                    },
                    price_at_trade: None,
                }).await {
                    Ok(trade) => trade,
                    Err(e) => {
                        tracing::error!("Failed to record KOL trade in DB: {}", e);
                        continue;
                    }
                };

                let trade_type_str = match trade_type {
                    KolTradeType::Buy => "buy",
                    KolTradeType::Sell => "sell",
                };

                let trust_score: f64 = kol_entity.trust_score.try_into().unwrap_or(50.0);
                let copy_recommended = kol_entity.copy_trading_enabled && trust_score >= 60.0;

                tracing::info!(
                    kol_id = %kol_entity.id,
                    kol_name = ?kol_entity.display_name,
                    trade_type = trade_type_str,
                    token_mint = %token_mint,
                    value_sol = value_sol,
                    trust_score = trust_score,
                    copy_enabled = kol_entity.copy_trading_enabled,
                    copy_recommended = copy_recommended,
                    "🔗 KOL trade detected, emitting copy trade event"
                );

                // Emit event with correct topic and all required fields
                let arb_event = ArbEvent::new(
                    "kol.trade_detected",
                    crate::events::EventSource::Agent(crate::events::AgentType::Scanner),
                    kol_topics::TRADE_DETECTED,
                    serde_json::json!({
                        "kol_id": kol_entity.id.to_string(),
                        "kol_trade_id": kol_trade.id.to_string(),
                        "kol_address": swap.wallet_address,
                        "kol_name": kol_entity.display_name,
                        "token_mint": token_mint,
                        "trade_type": trade_type_str,
                        "kol_trust_score": trust_score,
                        "signature": swap.signature,
                        "input_mint": swap.input_mint,
                        "output_mint": swap.output_mint,
                        "value_sol": value_sol,
                        "copy_recommended": copy_recommended,
                        "copy_trading_enabled": kol_entity.copy_trading_enabled,
                        "dex": swap.dex_source,
                    }),
                );

                if let Err(e) = state.event_tx.send(arb_event) {
                    tracing::error!("Failed to send KOL trade event: {}", e);
                }

                // Also record in memory tracker for quick lookups
                if let Err(e) = state.kol_tracker.record_trade(swap).await {
                    tracing::error!("Failed to record KOL trade in memory: {}", e);
                }
            }
        }
    }

    StatusCode::OK
}

#[derive(Debug, Serialize)]
pub struct RecentWebhookEventsResponse {
    pub events: Vec<WebhookEventSummary>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct WebhookEventSummary {
    pub id: String,
    pub wallet_address: String,
    pub transaction_type: String,
    pub signature: String,
    pub timestamp: String,
    pub value_sol: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct RecentEventsQuery {
    pub wallet_address: Option<String>,
    pub limit: Option<usize>,
}

pub async fn get_recent_webhook_events(
    State(state): State<AppState>,
    Query(query): Query<RecentEventsQuery>,
) -> AppResult<Json<RecentWebhookEventsResponse>> {
    let limit = query.limit.unwrap_or(50);

    let trades = state
        .kol_tracker
        .get_recent_trades(query.wallet_address.as_deref(), limit)
        .await;

    let events: Vec<WebhookEventSummary> = trades
        .into_iter()
        .map(|t| {
            let value_sol = TransactionParser::calculate_swap_value_sol(&t);
            WebhookEventSummary {
                id: t.id.to_string(),
                wallet_address: t.wallet_address,
                transaction_type: "SWAP".to_string(),
                signature: t.signature,
                timestamp: t.timestamp.to_rfc3339(),
                value_sol: Some(value_sol),
            }
        })
        .collect();

    let total = events.len();

    Ok(Json(RecentWebhookEventsResponse { events, total }))
}
