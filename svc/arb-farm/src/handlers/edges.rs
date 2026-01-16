use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::server::AppState;

#[derive(Debug, Deserialize)]
pub struct ListEdgesQuery {
    pub status: Option<String>,
    pub edge_type: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct EdgeResponse {
    pub id: Uuid,
    pub strategy_id: Option<Uuid>,
    pub edge_type: String,
    pub execution_mode: String,
    pub atomicity: String,
    pub simulated_profit_guaranteed: bool,
    pub estimated_profit_lamports: Option<i64>,
    pub risk_score: Option<i32>,
    pub status: String,
    pub created_at: String,
    pub expires_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ListEdgesResponse {
    pub edges: Vec<EdgeResponse>,
    pub total: usize,
}

pub async fn list_edges(
    State(state): State<AppState>,
    Query(query): Query<ListEdgesQuery>,
) -> AppResult<Json<ListEdgesResponse>> {
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let records = state
        .edge_repo
        .list(
            query.status.as_deref(),
            query.edge_type.as_deref(),
            limit,
            offset,
        )
        .await?;

    let edges: Vec<EdgeResponse> = records
        .iter()
        .map(|r| EdgeResponse {
            id: r.id,
            strategy_id: r.strategy_id,
            edge_type: r.edge_type.clone(),
            execution_mode: r.execution_mode.clone(),
            atomicity: r.atomicity.clone(),
            simulated_profit_guaranteed: r.simulated_profit_guaranteed,
            estimated_profit_lamports: r.estimated_profit_lamports,
            risk_score: r.risk_score,
            status: r.status.clone(),
            created_at: r.created_at.to_rfc3339(),
            expires_at: r.expires_at.map(|t| t.to_rfc3339()),
        })
        .collect();

    let total = edges.len();

    Ok(Json(ListEdgesResponse { edges, total }))
}

pub async fn get_edge(
    State(state): State<AppState>,
    Path(edge_id): Path<Uuid>,
) -> AppResult<Json<EdgeDetailResponse>> {
    let record = state
        .edge_repo
        .get_by_id(edge_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Edge {} not found", edge_id)))?;

    let trade = state.trade_repo.get_by_edge_id(edge_id).await?;

    Ok(Json(EdgeDetailResponse {
        edge: EdgeResponse {
            id: record.id,
            strategy_id: record.strategy_id,
            edge_type: record.edge_type.clone(),
            execution_mode: record.execution_mode.clone(),
            atomicity: record.atomicity.clone(),
            simulated_profit_guaranteed: record.simulated_profit_guaranteed,
            estimated_profit_lamports: record.estimated_profit_lamports,
            risk_score: record.risk_score,
            status: record.status.clone(),
            created_at: record.created_at.to_rfc3339(),
            expires_at: record.expires_at.map(|t| t.to_rfc3339()),
        },
        route_data: record.route_data,
        rejection_reason: record.rejection_reason,
        executed_at: record.executed_at.map(|t| t.to_rfc3339()),
        actual_profit_lamports: record.actual_profit_lamports,
        actual_gas_cost_lamports: record.actual_gas_cost_lamports,
        trade_id: trade.map(|t| t.id),
    }))
}

#[derive(Debug, Serialize)]
pub struct EdgeDetailResponse {
    #[serde(flatten)]
    pub edge: EdgeResponse,
    pub route_data: serde_json::Value,
    pub rejection_reason: Option<String>,
    pub executed_at: Option<String>,
    pub actual_profit_lamports: Option<i64>,
    pub actual_gas_cost_lamports: Option<i64>,
    pub trade_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct ApproveEdgeRequest {
    pub max_slippage_bps: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct ApproveEdgeResponse {
    pub edge_id: Uuid,
    pub status: String,
    pub message: String,
}

pub async fn approve_edge(
    State(state): State<AppState>,
    Path(edge_id): Path<Uuid>,
    Json(_request): Json<ApproveEdgeRequest>,
) -> AppResult<Json<ApproveEdgeResponse>> {
    let record = state
        .edge_repo
        .get_by_id(edge_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Edge {} not found", edge_id)))?;

    if record.status != "detected" && record.status != "pending_approval" {
        return Err(AppError::BadRequest(format!(
            "Edge {} cannot be approved in status: {}",
            edge_id, record.status
        )));
    }

    state
        .edge_repo
        .update(
            edge_id,
            crate::database::repositories::edges::UpdateEdgeRecord {
                status: Some(crate::models::EdgeStatus::PendingApproval),
                rejection_reason: None,
                executed_at: None,
                actual_profit_lamports: None,
                actual_gas_cost_lamports: None,
                simulation_tx_hash: None,
                max_gas_cost_lamports: None,
                simulated_profit_guaranteed: None,
            },
        )
        .await?;

    Ok(Json(ApproveEdgeResponse {
        edge_id,
        status: "approved".to_string(),
        message: "Edge approved and queued for execution".to_string(),
    }))
}

#[derive(Debug, Deserialize)]
pub struct RejectEdgeRequest {
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct RejectEdgeResponse {
    pub edge_id: Uuid,
    pub status: String,
    pub message: String,
}

pub async fn reject_edge(
    State(state): State<AppState>,
    Path(edge_id): Path<Uuid>,
    Json(request): Json<RejectEdgeRequest>,
) -> AppResult<Json<RejectEdgeResponse>> {
    let record = state
        .edge_repo
        .get_by_id(edge_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Edge {} not found", edge_id)))?;

    if record.status == "executed" || record.status == "failed" {
        return Err(AppError::BadRequest(format!(
            "Edge {} cannot be rejected in status: {}",
            edge_id, record.status
        )));
    }

    state
        .edge_repo
        .update(
            edge_id,
            crate::database::repositories::edges::UpdateEdgeRecord {
                status: Some(crate::models::EdgeStatus::Rejected),
                rejection_reason: Some(request.reason.clone()),
                executed_at: None,
                actual_profit_lamports: None,
                actual_gas_cost_lamports: None,
                simulation_tx_hash: None,
                max_gas_cost_lamports: None,
                simulated_profit_guaranteed: None,
            },
        )
        .await?;

    Ok(Json(RejectEdgeResponse {
        edge_id,
        status: "rejected".to_string(),
        message: format!("Edge rejected: {}", request.reason),
    }))
}

#[derive(Debug, Deserialize)]
pub struct ExecuteEdgeRequest {
    pub transaction_base64: String,
    pub max_slippage_bps: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct ExecuteEdgeResponse {
    pub edge_id: Uuid,
    pub success: bool,
    pub tx_signature: Option<String>,
    pub bundle_id: Option<String>,
    pub profit_lamports: Option<i64>,
    pub gas_cost_lamports: Option<u64>,
    pub execution_time_ms: u64,
    pub error: Option<String>,
}

pub async fn execute_edge(
    State(state): State<AppState>,
    Path(edge_id): Path<Uuid>,
    Json(request): Json<ExecuteEdgeRequest>,
) -> AppResult<Json<ExecuteEdgeResponse>> {
    let edge_record = state
        .edge_repo
        .get_by_id(edge_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Edge {} not found", edge_id)))?;

    if edge_record.status != "detected"
        && edge_record.status != "pending_approval"
        && edge_record.status != "approved"
    {
        return Err(AppError::BadRequest(format!(
            "Edge {} cannot be executed in status: {}",
            edge_id, edge_record.status
        )));
    }

    let strategy_id = edge_record
        .strategy_id
        .ok_or_else(|| AppError::BadRequest("Edge has no associated strategy".to_string()))?;

    let strategy_record = state
        .strategy_repo
        .get_by_id(strategy_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Strategy {} not found", strategy_id)))?;

    let edge = record_to_edge(&edge_record)?;
    let strategy = record_to_strategy(&strategy_record)?;

    let result = state
        .executor
        .execute_edge(&edge, &strategy, &request.transaction_base64)
        .await?;

    if result.success {
        state
            .trade_repo
            .create(crate::database::repositories::trades::CreateTradeRecord {
                edge_id,
                strategy_id,
                tx_signature: result.tx_signature.clone(),
                bundle_id: result.bundle_id.clone(),
                entry_price: None,
                exit_price: None,
                profit_lamports: result.profit_lamports,
                gas_cost_lamports: result.gas_cost_lamports.map(|g| g as i64),
                slippage_bps: request.max_slippage_bps,
            })
            .await?;

        state
            .edge_repo
            .update(
                edge_id,
                crate::database::repositories::edges::UpdateEdgeRecord {
                    status: Some(crate::models::EdgeStatus::Executed),
                    rejection_reason: None,
                    executed_at: Some(chrono::Utc::now()),
                    actual_profit_lamports: result.profit_lamports,
                    actual_gas_cost_lamports: result.gas_cost_lamports.map(|g| g as i64),
                    simulation_tx_hash: None,
                    max_gas_cost_lamports: None,
                    simulated_profit_guaranteed: None,
                },
            )
            .await?;
    } else {
        state
            .edge_repo
            .update(
                edge_id,
                crate::database::repositories::edges::UpdateEdgeRecord {
                    status: Some(crate::models::EdgeStatus::Failed),
                    rejection_reason: result.error.clone(),
                    executed_at: Some(chrono::Utc::now()),
                    actual_profit_lamports: None,
                    actual_gas_cost_lamports: result.gas_cost_lamports.map(|g| g as i64),
                    simulation_tx_hash: None,
                    max_gas_cost_lamports: None,
                    simulated_profit_guaranteed: None,
                },
            )
            .await?;
    }

    Ok(Json(ExecuteEdgeResponse {
        edge_id,
        success: result.success,
        tx_signature: result.tx_signature,
        bundle_id: result.bundle_id,
        profit_lamports: result.profit_lamports,
        gas_cost_lamports: result.gas_cost_lamports,
        execution_time_ms: result.execution_time_ms,
        error: result.error,
    }))
}

#[derive(Debug, Deserialize)]
pub struct SimulateEdgeRequest {
    pub transaction_base64: String,
}

#[derive(Debug, Serialize)]
pub struct SimulateEdgeResponse {
    pub edge_id: Uuid,
    pub success: bool,
    pub simulated_profit_lamports: Option<i64>,
    pub simulated_gas_lamports: u64,
    pub atomicity: String,
    pub profit_guaranteed: bool,
    pub error: Option<String>,
}

pub async fn simulate_edge(
    State(state): State<AppState>,
    Path(edge_id): Path<Uuid>,
    Json(request): Json<SimulateEdgeRequest>,
) -> AppResult<Json<SimulateEdgeResponse>> {
    let result = state
        .simulator
        .simulate_transaction(edge_id, &request.transaction_base64)
        .await?;

    if result.success {
        state
            .edge_repo
            .update(
                edge_id,
                crate::database::repositories::edges::UpdateEdgeRecord {
                    status: None,
                    rejection_reason: None,
                    executed_at: None,
                    actual_profit_lamports: None,
                    actual_gas_cost_lamports: None,
                    simulation_tx_hash: Some(format!("sim_{}", edge_id)),
                    max_gas_cost_lamports: Some(result.simulated_gas_lamports as i64),
                    simulated_profit_guaranteed: Some(result.profit_guaranteed),
                },
            )
            .await?;
    }

    let atomicity_str = match result.atomicity {
        crate::events::AtomicityLevel::FullyAtomic => "fully_atomic",
        crate::events::AtomicityLevel::PartiallyAtomic => "partially_atomic",
        crate::events::AtomicityLevel::NonAtomic => "non_atomic",
    };

    Ok(Json(SimulateEdgeResponse {
        edge_id,
        success: result.success,
        simulated_profit_lamports: result.simulated_profit_lamports,
        simulated_gas_lamports: result.simulated_gas_lamports,
        atomicity: atomicity_str.to_string(),
        profit_guaranteed: result.profit_guaranteed,
        error: result.error,
    }))
}

#[derive(Debug, Deserialize)]
pub struct ListAtomicEdgesQuery {
    pub min_profit_lamports: Option<i64>,
    pub limit: Option<i64>,
}

pub async fn list_atomic_edges(
    State(state): State<AppState>,
    Query(query): Query<ListAtomicEdgesQuery>,
) -> AppResult<Json<ListEdgesResponse>> {
    let min_profit = query.min_profit_lamports.unwrap_or(0);
    let limit = query.limit.unwrap_or(20);

    let records = state
        .edge_repo
        .list_atomic_opportunities(min_profit, limit)
        .await?;

    let edges: Vec<EdgeResponse> = records
        .iter()
        .map(|r| EdgeResponse {
            id: r.id,
            strategy_id: r.strategy_id,
            edge_type: r.edge_type.clone(),
            execution_mode: r.execution_mode.clone(),
            atomicity: r.atomicity.clone(),
            simulated_profit_guaranteed: r.simulated_profit_guaranteed,
            estimated_profit_lamports: r.estimated_profit_lamports,
            risk_score: r.risk_score,
            status: r.status.clone(),
            created_at: r.created_at.to_rfc3339(),
            expires_at: r.expires_at.map(|t| t.to_rfc3339()),
        })
        .collect();

    let total = edges.len();

    Ok(Json(ListEdgesResponse { edges, total }))
}

#[derive(Debug, Deserialize)]
pub struct ExecuteEdgeAutoRequest {
    pub slippage_bps: Option<u16>,
}

#[derive(Debug, Serialize)]
pub struct ExecuteEdgeAutoResponse {
    pub edge_id: Uuid,
    pub success: bool,
    pub tx_signature: Option<String>,
    pub bundle_id: Option<String>,
    pub profit_lamports: Option<i64>,
    pub gas_cost_lamports: Option<u64>,
    pub execution_time_ms: u64,
    pub error: Option<String>,
    pub route_info: Option<serde_json::Value>,
}

pub async fn execute_edge_auto(
    State(state): State<AppState>,
    Path(edge_id): Path<Uuid>,
    Json(request): Json<ExecuteEdgeAutoRequest>,
) -> AppResult<Json<ExecuteEdgeAutoResponse>> {
    let edge_record = state
        .edge_repo
        .get_by_id(edge_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Edge {} not found", edge_id)))?;

    if edge_record.status != "detected"
        && edge_record.status != "pending_approval"
        && edge_record.status != "approved"
    {
        return Err(AppError::BadRequest(format!(
            "Edge {} cannot be executed in status: {}",
            edge_id, edge_record.status
        )));
    }

    let strategy_id = edge_record
        .strategy_id
        .ok_or_else(|| AppError::BadRequest("Edge has no associated strategy".to_string()))?;

    let strategy_record = state
        .strategy_repo
        .get_by_id(strategy_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Strategy {} not found", strategy_id)))?;

    let edge = record_to_edge(&edge_record)?;
    let strategy = record_to_strategy(&strategy_record)?;

    let slippage_bps = request.slippage_bps.unwrap_or(100); // Default 1% slippage

    let result = state
        .executor
        .execute_edge_auto(
            &edge,
            &strategy,
            &state.tx_builder,
            &state.turnkey_signer,
            slippage_bps,
        )
        .await?;

    if result.success {
        state
            .trade_repo
            .create(crate::database::repositories::trades::CreateTradeRecord {
                edge_id,
                strategy_id,
                tx_signature: result.tx_signature.clone(),
                bundle_id: result.bundle_id.clone(),
                entry_price: None,
                exit_price: None,
                profit_lamports: result.profit_lamports,
                gas_cost_lamports: result.gas_cost_lamports.map(|g| g as i64),
                slippage_bps: Some(slippage_bps as i32),
            })
            .await?;

        state
            .edge_repo
            .update(
                edge_id,
                crate::database::repositories::edges::UpdateEdgeRecord {
                    status: Some(crate::models::EdgeStatus::Executed),
                    rejection_reason: None,
                    executed_at: Some(chrono::Utc::now()),
                    actual_profit_lamports: result.profit_lamports,
                    actual_gas_cost_lamports: result.gas_cost_lamports.map(|g| g as i64),
                    simulation_tx_hash: None,
                    max_gas_cost_lamports: None,
                    simulated_profit_guaranteed: None,
                },
            )
            .await?;
    } else {
        state
            .edge_repo
            .update(
                edge_id,
                crate::database::repositories::edges::UpdateEdgeRecord {
                    status: Some(crate::models::EdgeStatus::Failed),
                    rejection_reason: result.error.clone(),
                    executed_at: Some(chrono::Utc::now()),
                    actual_profit_lamports: None,
                    actual_gas_cost_lamports: result.gas_cost_lamports.map(|g| g as i64),
                    simulation_tx_hash: None,
                    max_gas_cost_lamports: None,
                    simulated_profit_guaranteed: None,
                },
            )
            .await?;
    }

    Ok(Json(ExecuteEdgeAutoResponse {
        edge_id,
        success: result.success,
        tx_signature: result.tx_signature,
        bundle_id: result.bundle_id,
        profit_lamports: result.profit_lamports,
        gas_cost_lamports: result.gas_cost_lamports,
        execution_time_ms: result.execution_time_ms,
        error: result.error,
        route_info: None,
    }))
}

fn record_to_edge(
    record: &crate::database::repositories::edges::EdgeRecord,
) -> AppResult<crate::models::Edge> {
    let atomicity = match record.atomicity.as_str() {
        "fully_atomic" => crate::events::AtomicityLevel::FullyAtomic,
        "partially_atomic" => crate::events::AtomicityLevel::PartiallyAtomic,
        _ => crate::events::AtomicityLevel::NonAtomic,
    };

    let status = match record.status.as_str() {
        "detected" => crate::models::EdgeStatus::Detected,
        "pending_approval" => crate::models::EdgeStatus::PendingApproval,
        "executing" => crate::models::EdgeStatus::Executing,
        "executed" => crate::models::EdgeStatus::Executed,
        "expired" => crate::models::EdgeStatus::Expired,
        "failed" => crate::models::EdgeStatus::Failed,
        "rejected" => crate::models::EdgeStatus::Rejected,
        _ => crate::models::EdgeStatus::Detected,
    };

    Ok(crate::models::Edge {
        id: record.id,
        strategy_id: record.strategy_id,
        edge_type: record.edge_type.clone(),
        execution_mode: record.execution_mode.clone(),
        atomicity,
        simulated_profit_guaranteed: record.simulated_profit_guaranteed,
        estimated_profit_lamports: record.estimated_profit_lamports,
        risk_score: record.risk_score,
        route_data: record.route_data.clone(),
        signal_data: Some(record.route_data.clone()),
        status,
        token_mint: None,
        created_at: record.created_at,
        expires_at: record.expires_at,
    })
}

fn record_to_strategy(
    record: &crate::database::repositories::strategies::StrategyRecord,
) -> AppResult<crate::models::Strategy> {
    let risk_params: crate::models::RiskParams =
        serde_json::from_value(record.risk_params.clone())
            .map_err(|e| AppError::Serialization(e.to_string()))?;

    Ok(crate::models::Strategy {
        id: record.id,
        wallet_address: record.wallet_address.clone(),
        name: record.name.clone(),
        strategy_type: record.strategy_type.clone(),
        venue_types: record.venue_types.clone(),
        execution_mode: record.execution_mode.clone(),
        risk_params,
        is_active: record.is_active,
        created_at: record.created_at,
        updated_at: record.updated_at,
    })
}
