use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::events::ArbEvent;
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

    match state.helius_client.create_webhook(&config).await {
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

    match state.helius_client.list_webhooks().await {
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
    match state.helius_client.delete_webhook(&request.webhook_id).await {
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
    Json(payload): Json<Vec<EnhancedTransactionEvent>>,
) -> StatusCode {
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

                let kol_name = state
                    .kol_tracker
                    .get_kol_name(&swap.wallet_address)
                    .await;
                let trust_score = state
                    .kol_tracker
                    .get_trust_score(&swap.wallet_address)
                    .await
                    .unwrap_or(0.5);

                let signal = KOLTradeSignal::from_swap(swap.clone(), kol_name.clone(), trust_score);

                let event = ArbEvent::new(
                    "kol.trade_detected",
                    crate::events::EventSource::Agent(crate::events::AgentType::Scanner),
                    "arb.kol.trade",
                    serde_json::json!({
                        "kol_address": signal.kol_address,
                        "kol_name": signal.kol_name,
                        "signature": signal.swap.signature,
                        "input_mint": signal.swap.input_mint,
                        "output_mint": signal.swap.output_mint,
                        "value_sol": signal.value_sol,
                        "copy_recommended": signal.copy_recommended,
                        "dex": signal.swap.dex_source,
                    }),
                );

                let _ = state.event_tx.send(event);

                if let Err(e) = state.kol_tracker.record_trade(swap).await {
                    tracing::error!("Failed to record KOL trade: {}", e);
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
