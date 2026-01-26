use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::events::{ArbEvent, AtomicityLevel};
use crate::models::{Edge, EdgeStatus, Strategy};
use crate::wallet::turnkey::{SignRequest, TurnkeySigner};

use super::jito::{BundleConfig, BundleState, JitoClient};
use super::risk::{RiskManager, RiskCheck, ViolationSeverity};
use super::simulation::{SimulationConfig, SimulationResult, TransactionSimulator};
use super::transaction_builder::{TransactionBuilder, BuildResult};

pub struct ExecutorAgent {
    id: Uuid,
    config: ExecutorConfig,
    jito_client: JitoClient,
    simulator: TransactionSimulator,
    risk_manager: RiskManager,
    event_tx: broadcast::Sender<ArbEvent>,
    pending_executions: Arc<RwLock<HashMap<Uuid, PendingExecution>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorConfig {
    pub simulation: SimulationConfig,
    pub bundle: BundleConfig,
    pub auto_execute_atomic: bool,
    pub require_simulation: bool,
    pub max_concurrent_executions: u32,
    pub execution_timeout_secs: u64,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            simulation: SimulationConfig::default(),
            bundle: BundleConfig::default(),
            auto_execute_atomic: true,
            require_simulation: true,
            max_concurrent_executions: 5,
            execution_timeout_secs: 60,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingExecution {
    pub edge_id: Uuid,
    pub strategy_id: Uuid,
    pub status: ExecutionStatus,
    pub simulation_result: Option<SimulationResult>,
    pub risk_check: Option<RiskCheck>,
    pub bundle_id: Option<String>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    Pending,
    Simulating,
    RiskCheck,
    AwaitingApproval,
    Submitting,
    Confirming,
    Completed,
    Failed,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub edge_id: Uuid,
    pub strategy_id: Uuid,
    pub success: bool,
    pub tx_signature: Option<String>,
    pub bundle_id: Option<String>,
    pub profit_lamports: Option<i64>,
    pub gas_cost_lamports: Option<u64>,
    pub execution_time_ms: u64,
    pub error: Option<String>,
    pub landed_slot: Option<u64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    Autonomous,
    Hybrid,
    AgentDirected,
}

impl ExecutorAgent {
    pub fn new(
        jito_block_engine_url: String,
        rpc_url: String,
        config: ExecutorConfig,
        event_tx: broadcast::Sender<ArbEvent>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            jito_client: JitoClient::new(jito_block_engine_url, None),
            simulator: TransactionSimulator::new(rpc_url),
            risk_manager: RiskManager::new(Default::default()),
            config,
            event_tx,
            pending_executions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_db_pool(mut self, pool: PgPool) -> Self {
        self.risk_manager = RiskManager::new(Default::default()).with_db_pool(pool);
        self
    }

    pub async fn load_risk_stats(&self) -> AppResult<()> {
        self.risk_manager.load_daily_stats_from_db().await
    }

    pub async fn execute_edge(
        &self,
        edge: &Edge,
        strategy: &Strategy,
        transaction_base64: &str,
    ) -> AppResult<ExecutionResult> {
        let start = std::time::Instant::now();
        let edge_id = edge.id;
        let strategy_id = strategy.id;

        self.create_pending_execution(edge_id, strategy_id).await;
        self.emit_edge_event(edge_id, EdgeStatus::Executing).await;

        let result = self
            .execute_internal(edge, strategy, transaction_base64)
            .await;

        let execution_time_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(mut exec_result) => {
                exec_result.execution_time_ms = execution_time_ms;
                self.complete_execution(edge_id, &exec_result).await;

                if exec_result.success {
                    self.emit_edge_event(edge_id, EdgeStatus::Executed).await;
                    if let Some(profit) = exec_result.profit_lamports {
                        self.risk_manager.record_trade_result(profit).await;
                    }
                } else {
                    self.emit_edge_event(edge_id, EdgeStatus::Failed).await;
                }

                Ok(exec_result)
            }
            Err(e) => {
                let exec_result = ExecutionResult {
                    edge_id,
                    strategy_id,
                    success: false,
                    tx_signature: None,
                    bundle_id: None,
                    profit_lamports: None,
                    gas_cost_lamports: None,
                    execution_time_ms,
                    error: Some(e.to_string()),
                    landed_slot: None,
                };

                self.complete_execution(edge_id, &exec_result).await;
                self.emit_edge_event(edge_id, EdgeStatus::Failed).await;

                Err(e)
            }
        }
    }

    async fn execute_internal(
        &self,
        edge: &Edge,
        strategy: &Strategy,
        transaction_base64: &str,
    ) -> AppResult<ExecutionResult> {
        self.update_execution_status(edge.id, ExecutionStatus::Simulating)
            .await;
        let simulation = if self.config.require_simulation {
            let sim_result = self
                .simulator
                .simulate_transaction(edge.id, transaction_base64)
                .await?;

            if !sim_result.success {
                return Ok(ExecutionResult {
                    edge_id: edge.id,
                    strategy_id: strategy.id,
                    success: false,
                    tx_signature: None,
                    bundle_id: None,
                    profit_lamports: None,
                    gas_cost_lamports: Some(sim_result.simulated_gas_lamports),
                    execution_time_ms: 0,
                    error: sim_result.error,
                    landed_slot: None,
                });
            }

            self.store_simulation_result(edge.id, &sim_result).await;
            Some(sim_result)
        } else {
            None
        };

        self.update_execution_status(edge.id, ExecutionStatus::RiskCheck)
            .await;
        let risk_check = self
            .risk_manager
            .check_edge(edge, &strategy.risk_params)
            .await;

        self.store_risk_check(edge.id, &risk_check).await;

        if !risk_check.passed {
            let blocking_violations: Vec<_> = risk_check
                .violations
                .iter()
                .filter(|v| v.severity == ViolationSeverity::Block)
                .map(|v| v.message.clone())
                .collect();

            return Ok(ExecutionResult {
                edge_id: edge.id,
                strategy_id: strategy.id,
                success: false,
                tx_signature: None,
                bundle_id: None,
                profit_lamports: None,
                gas_cost_lamports: None,
                execution_time_ms: 0,
                error: Some(format!("Risk check failed: {}", blocking_violations.join("; "))),
                landed_slot: None,
            });
        }

        let execution_mode = self.determine_execution_mode(edge, strategy, &risk_check);

        match execution_mode {
            ExecutionMode::AgentDirected => {
                self.update_execution_status(edge.id, ExecutionStatus::AwaitingApproval)
                    .await;
                self.emit_approval_needed_event(edge, strategy).await;

                Ok(ExecutionResult {
                    edge_id: edge.id,
                    strategy_id: strategy.id,
                    success: false,
                    tx_signature: None,
                    bundle_id: None,
                    profit_lamports: None,
                    gas_cost_lamports: None,
                    execution_time_ms: 0,
                    error: Some("Awaiting agent approval".to_string()),
                    landed_slot: None,
                })
            }

            ExecutionMode::Autonomous | ExecutionMode::Hybrid => {
                self.submit_and_confirm(edge, strategy, transaction_base64, simulation)
                    .await
            }
        }
    }

    async fn submit_and_confirm(
        &self,
        edge: &Edge,
        strategy: &Strategy,
        transaction_base64: &str,
        simulation: Option<SimulationResult>,
    ) -> AppResult<ExecutionResult> {
        self.update_execution_status(edge.id, ExecutionStatus::Submitting)
            .await;

        let estimated_profit = simulation
            .as_ref()
            .and_then(|s| s.simulated_profit_lamports)
            .unwrap_or(edge.estimated_profit_lamports.unwrap_or(0));

        let tip = self.config.bundle.calculate_tip(estimated_profit);

        let tx_base58 = base64_to_base58(transaction_base64)?;

        let bundle_result = self
            .jito_client
            .send_bundle(vec![tx_base58], tip)
            .await?;

        let bundle_id = bundle_result.id.to_string();
        self.store_bundle_id(edge.id, &bundle_id).await;

        self.update_execution_status(edge.id, ExecutionStatus::Confirming)
            .await;

        let status = self
            .jito_client
            .wait_for_bundle(&bundle_id, self.config.execution_timeout_secs)
            .await?;

        match status.status {
            BundleState::Landed => {
                self.risk_manager
                    .open_position(edge.id, edge.token_mint.clone(), 0.0)
                    .await;

                Ok(ExecutionResult {
                    edge_id: edge.id,
                    strategy_id: strategy.id,
                    success: true,
                    tx_signature: Some(bundle_id.clone()),
                    bundle_id: Some(bundle_id),
                    profit_lamports: simulation.as_ref().and_then(|s| s.simulated_profit_lamports),
                    gas_cost_lamports: simulation.as_ref().map(|s| s.simulated_gas_lamports),
                    execution_time_ms: 0,
                    error: None,
                    landed_slot: status.landed_slot,
                })
            }

            BundleState::Failed | BundleState::Dropped => Ok(ExecutionResult {
                edge_id: edge.id,
                strategy_id: strategy.id,
                success: false,
                tx_signature: None,
                bundle_id: Some(bundle_id.clone()),
                profit_lamports: None,
                gas_cost_lamports: simulation.as_ref().map(|s| s.simulated_gas_lamports),
                execution_time_ms: 0,
                error: Some(format!("Bundle {}: {:?}", bundle_id, status.status)),
                landed_slot: None,
            }),

            BundleState::Pending => Ok(ExecutionResult {
                edge_id: edge.id,
                strategy_id: strategy.id,
                success: false,
                tx_signature: None,
                bundle_id: Some(bundle_id.clone()),
                profit_lamports: None,
                gas_cost_lamports: None,
                execution_time_ms: 0,
                error: Some("Bundle timed out in pending state".to_string()),
                landed_slot: None,
            }),
        }
    }

    fn determine_execution_mode(
        &self,
        edge: &Edge,
        strategy: &Strategy,
        risk_check: &RiskCheck,
    ) -> ExecutionMode {
        if edge.atomicity == AtomicityLevel::FullyAtomic
            && edge.simulated_profit_guaranteed
            && self.config.auto_execute_atomic
        {
            return ExecutionMode::Autonomous;
        }

        match strategy.execution_mode.as_str() {
            "autonomous" => {
                if risk_check.risk_score <= strategy.risk_params.max_risk_score / 2 {
                    ExecutionMode::Autonomous
                } else {
                    ExecutionMode::Hybrid
                }
            }
            "hybrid" => {
                let profit_bps = edge
                    .estimated_profit_lamports
                    .map(|p| (p / 10000) as i32)
                    .unwrap_or(0);

                if profit_bps > strategy.risk_params.min_profit_bps as i32 * 2
                    && risk_check.risk_score <= strategy.risk_params.max_risk_score
                {
                    ExecutionMode::Autonomous
                } else {
                    ExecutionMode::AgentDirected
                }
            }
            _ => ExecutionMode::AgentDirected,
        }
    }

    async fn create_pending_execution(&self, edge_id: Uuid, strategy_id: Uuid) {
        let mut pending = self.pending_executions.write().await;
        pending.insert(
            edge_id,
            PendingExecution {
                edge_id,
                strategy_id,
                status: ExecutionStatus::Pending,
                simulation_result: None,
                risk_check: None,
                bundle_id: None,
                started_at: chrono::Utc::now(),
                completed_at: None,
            },
        );
    }

    async fn update_execution_status(&self, edge_id: Uuid, status: ExecutionStatus) {
        let mut pending = self.pending_executions.write().await;
        if let Some(exec) = pending.get_mut(&edge_id) {
            exec.status = status;
        }
    }

    async fn store_simulation_result(&self, edge_id: Uuid, result: &SimulationResult) {
        let mut pending = self.pending_executions.write().await;
        if let Some(exec) = pending.get_mut(&edge_id) {
            exec.simulation_result = Some(result.clone());
        }
    }

    async fn store_risk_check(&self, edge_id: Uuid, check: &RiskCheck) {
        let mut pending = self.pending_executions.write().await;
        if let Some(exec) = pending.get_mut(&edge_id) {
            exec.risk_check = Some(check.clone());
        }
    }

    async fn store_bundle_id(&self, edge_id: Uuid, bundle_id: &str) {
        let mut pending = self.pending_executions.write().await;
        if let Some(exec) = pending.get_mut(&edge_id) {
            exec.bundle_id = Some(bundle_id.to_string());
        }
    }

    async fn complete_execution(&self, edge_id: Uuid, _result: &ExecutionResult) {
        let mut pending = self.pending_executions.write().await;
        if let Some(exec) = pending.get_mut(&edge_id) {
            exec.status = ExecutionStatus::Completed;
            exec.completed_at = Some(chrono::Utc::now());
        }
    }

    async fn emit_edge_event(&self, edge_id: Uuid, status: EdgeStatus) {
        let event_type = match status {
            EdgeStatus::Executing => "edge.executing",
            EdgeStatus::Executed => "edge.executed",
            EdgeStatus::Failed => "edge.failed",
            _ => return,
        };

        let topic = format!("arb.edge.{}", status);

        let event = ArbEvent::new(
            event_type,
            crate::events::EventSource::Agent(crate::events::AgentType::Executor),
            topic,
            serde_json::json!({
                "edge_id": edge_id,
                "status": status.to_string(),
            }),
        );

        if let Err(e) = self.event_tx.send(event) {
            tracing::warn!("Event broadcast failed (channel full/closed): {}", e);
        }
    }

    async fn emit_approval_needed_event(&self, edge: &Edge, strategy: &Strategy) {
        let event = ArbEvent::new(
            "edge.pending_approval",
            crate::events::EventSource::Agent(crate::events::AgentType::Executor),
            "arb.edge.pending_approval",
            serde_json::json!({
                "edge_id": edge.id,
                "strategy_id": strategy.id,
                "edge_type": edge.edge_type,
                "estimated_profit_lamports": edge.estimated_profit_lamports,
                "risk_score": edge.risk_score,
                "requires_consensus": true,
            }),
        );

        if let Err(e) = self.event_tx.send(event) {
            tracing::warn!("Event broadcast failed (channel full/closed): {}", e);
        }
    }

    pub async fn approve_edge(&self, edge_id: Uuid) -> AppResult<()> {
        let mut pending = self.pending_executions.write().await;
        if let Some(exec) = pending.get_mut(&edge_id) {
            if exec.status == ExecutionStatus::AwaitingApproval {
                exec.status = ExecutionStatus::Pending;
                return Ok(());
            }
        }
        Err(AppError::NotFound(format!(
            "Edge {} not awaiting approval",
            edge_id
        )))
    }

    pub async fn reject_edge(&self, edge_id: Uuid, reason: &str) -> AppResult<()> {
        let mut pending = self.pending_executions.write().await;
        if let Some(exec) = pending.get_mut(&edge_id) {
            exec.status = ExecutionStatus::Rejected;
            exec.completed_at = Some(chrono::Utc::now());

            let event = ArbEvent::new(
                "edge.rejected",
                crate::events::EventSource::Agent(crate::events::AgentType::Executor),
                "arb.edge.rejected",
                serde_json::json!({
                    "edge_id": edge_id,
                    "reason": reason,
                }),
            );

            if let Err(e) = self.event_tx.send(event) {
            tracing::warn!("Event broadcast failed (channel full/closed): {}", e);
        }
            return Ok(());
        }
        Err(AppError::NotFound(format!("Edge {} not found", edge_id)))
    }

    pub async fn get_pending_executions(&self) -> Vec<PendingExecution> {
        let pending = self.pending_executions.read().await;
        pending.values().cloned().collect()
    }

    pub async fn get_execution_status(&self, edge_id: Uuid) -> Option<PendingExecution> {
        let pending = self.pending_executions.read().await;
        pending.get(&edge_id).cloned()
    }

    pub async fn get_risk_stats(&self) -> super::risk::DailyRiskStats {
        self.risk_manager.get_stats().await
    }

    pub async fn execute_edge_auto(
        &self,
        edge: &Edge,
        strategy: &Strategy,
        tx_builder: &TransactionBuilder,
        signer: &TurnkeySigner,
        slippage_bps: u16,
    ) -> AppResult<ExecutionResult> {
        let start = std::time::Instant::now();
        let edge_id = edge.id;
        let strategy_id = strategy.id;

        self.create_pending_execution(edge_id, strategy_id).await;
        self.emit_edge_event(edge_id, EdgeStatus::Executing).await;

        // Step 1: Get wallet status and address
        let wallet_status = signer.get_status().await;
        let user_wallet = match &wallet_status.wallet_address {
            Some(addr) => addr.clone(),
            None => {
                return Ok(ExecutionResult {
                    edge_id,
                    strategy_id,
                    success: false,
                    tx_signature: None,
                    bundle_id: None,
                    profit_lamports: None,
                    gas_cost_lamports: None,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    error: Some("No wallet configured".to_string()),
                    landed_slot: None,
                });
            }
        };

        // Step 2: Build the transaction
        self.update_execution_status(edge_id, ExecutionStatus::Simulating).await;
        let build_result = match tx_builder.build_swap(edge, &user_wallet, slippage_bps).await {
            Ok(result) => result,
            Err(e) => {
                return Ok(ExecutionResult {
                    edge_id,
                    strategy_id,
                    success: false,
                    tx_signature: None,
                    bundle_id: None,
                    profit_lamports: None,
                    gas_cost_lamports: None,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    error: Some(format!("Failed to build transaction: {}", e)),
                    landed_slot: None,
                });
            }
        };

        // Step 3: Risk check
        self.update_execution_status(edge_id, ExecutionStatus::RiskCheck).await;
        let risk_check = self.risk_manager.check_edge(edge, &strategy.risk_params).await;
        self.store_risk_check(edge_id, &risk_check).await;

        if !risk_check.passed {
            let blocking_violations: Vec<_> = risk_check
                .violations
                .iter()
                .filter(|v| v.severity == ViolationSeverity::Block)
                .map(|v| v.message.clone())
                .collect();

            return Ok(ExecutionResult {
                edge_id,
                strategy_id,
                success: false,
                tx_signature: None,
                bundle_id: None,
                profit_lamports: None,
                gas_cost_lamports: None,
                execution_time_ms: start.elapsed().as_millis() as u64,
                error: Some(format!("Risk check failed: {}", blocking_violations.join("; "))),
                landed_slot: None,
            });
        }

        // Step 4: Sign the transaction via Turnkey
        let sign_request = SignRequest {
            transaction_base64: build_result.transaction_base64.clone(),
            estimated_amount_lamports: build_result.route_info.in_amount,
            estimated_profit_lamports: edge.estimated_profit_lamports,
            edge_id: Some(edge_id),
            description: format!(
                "Swap {} -> {} for edge {}",
                build_result.route_info.input_mint,
                build_result.route_info.output_mint,
                edge_id
            ),
        };

        let sign_result = match signer.sign_transaction(sign_request).await {
            Ok(result) => result,
            Err(e) => {
                return Ok(ExecutionResult {
                    edge_id,
                    strategy_id,
                    success: false,
                    tx_signature: None,
                    bundle_id: None,
                    profit_lamports: None,
                    gas_cost_lamports: None,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    error: Some(format!("Signing failed: {}", e)),
                    landed_slot: None,
                });
            }
        };

        if !sign_result.success {
            return Ok(ExecutionResult {
                edge_id,
                strategy_id,
                success: false,
                tx_signature: None,
                bundle_id: None,
                profit_lamports: None,
                gas_cost_lamports: None,
                execution_time_ms: start.elapsed().as_millis() as u64,
                error: sign_result.error.or_else(|| {
                    sign_result.policy_violation.map(|v| v.message)
                }),
                landed_slot: None,
            });
        }

        let signed_tx = match sign_result.signed_transaction_base64 {
            Some(tx) => tx,
            None => {
                return Ok(ExecutionResult {
                    edge_id,
                    strategy_id,
                    success: false,
                    tx_signature: None,
                    bundle_id: None,
                    profit_lamports: None,
                    gas_cost_lamports: None,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    error: Some("No signed transaction returned".to_string()),
                    landed_slot: None,
                });
            }
        };

        // Step 5: Submit to Jito
        self.update_execution_status(edge_id, ExecutionStatus::Submitting).await;

        let estimated_profit = edge.estimated_profit_lamports.unwrap_or(0);
        let tip = self.config.bundle.calculate_tip(estimated_profit);

        let tx_base58 = base64_to_base58(&signed_tx)?;

        let bundle_result = match self.jito_client.send_bundle(vec![tx_base58], tip).await {
            Ok(result) => result,
            Err(e) => {
                return Ok(ExecutionResult {
                    edge_id,
                    strategy_id,
                    success: false,
                    tx_signature: None,
                    bundle_id: None,
                    profit_lamports: None,
                    gas_cost_lamports: Some(build_result.priority_fee_lamports),
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    error: Some(format!("Jito bundle submission failed: {}", e)),
                    landed_slot: None,
                });
            }
        };

        let bundle_id = bundle_result.id.to_string();
        self.store_bundle_id(edge_id, &bundle_id).await;

        // Step 6: Wait for confirmation
        self.update_execution_status(edge_id, ExecutionStatus::Confirming).await;

        let status = match self.jito_client.wait_for_bundle(&bundle_id, self.config.execution_timeout_secs).await {
            Ok(status) => status,
            Err(e) => {
                return Ok(ExecutionResult {
                    edge_id,
                    strategy_id,
                    success: false,
                    tx_signature: None,
                    bundle_id: Some(bundle_id),
                    profit_lamports: None,
                    gas_cost_lamports: Some(build_result.priority_fee_lamports),
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    error: Some(format!("Bundle confirmation failed: {}", e)),
                    landed_slot: None,
                });
            }
        };

        let execution_time_ms = start.elapsed().as_millis() as u64;

        match status.status {
            BundleState::Landed => {
                self.risk_manager
                    .open_position(edge_id, edge.token_mint.clone(), build_result.route_info.in_amount as f64 / 1e9)
                    .await;

                if let Some(profit) = edge.estimated_profit_lamports {
                    self.risk_manager.record_trade_result(profit).await;
                }

                self.emit_edge_event(edge_id, EdgeStatus::Executed).await;

                Ok(ExecutionResult {
                    edge_id,
                    strategy_id,
                    success: true,
                    tx_signature: sign_result.signature,
                    bundle_id: Some(bundle_id),
                    profit_lamports: edge.estimated_profit_lamports,
                    gas_cost_lamports: Some(build_result.priority_fee_lamports),
                    execution_time_ms,
                    error: None,
                    landed_slot: status.landed_slot,
                })
            }

            BundleState::Failed | BundleState::Dropped => {
                self.emit_edge_event(edge_id, EdgeStatus::Failed).await;

                Ok(ExecutionResult {
                    edge_id,
                    strategy_id,
                    success: false,
                    tx_signature: None,
                    bundle_id: Some(bundle_id.clone()),
                    profit_lamports: None,
                    gas_cost_lamports: Some(build_result.priority_fee_lamports),
                    execution_time_ms,
                    error: Some(format!("Bundle {}: {:?}", bundle_id, status.status)),
                    landed_slot: None,
                })
            }

            BundleState::Pending => {
                self.emit_edge_event(edge_id, EdgeStatus::Failed).await;

                Ok(ExecutionResult {
                    edge_id,
                    strategy_id,
                    success: false,
                    tx_signature: None,
                    bundle_id: Some(bundle_id),
                    profit_lamports: None,
                    gas_cost_lamports: None,
                    execution_time_ms,
                    error: Some("Bundle timed out in pending state".to_string()),
                    landed_slot: None,
                })
            }
        }
    }
}

fn base64_to_base58(base64_str: &str) -> AppResult<String> {
    use base64::{engine::general_purpose::STANDARD, Engine};

    let bytes = STANDARD
        .decode(base64_str)
        .map_err(|e| AppError::Execution(format!("Invalid base64: {}", e)))?;

    Ok(bs58::encode(bytes).into_string())
}
