use reqwest::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::events::AtomicityLevel;

pub struct TransactionSimulator {
    client: Client,
    rpc_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub edge_id: Uuid,
    pub success: bool,
    pub simulated_profit_lamports: Option<i64>,
    pub simulated_gas_lamports: u64,
    pub logs: Vec<String>,
    pub error: Option<String>,
    pub atomicity: AtomicityLevel,
    pub profit_guaranteed: bool,
    pub simulation_slot: u64,
}

#[derive(Debug, Serialize)]
struct SimulateRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct SimulateResponse {
    result: Option<SimulateResult>,
    error: Option<RpcError>,
}

#[derive(Debug, Deserialize)]
struct SimulateResult {
    context: SimulateContext,
    value: SimulateValue,
}

#[derive(Debug, Deserialize)]
struct SimulateContext {
    slot: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SimulateValue {
    err: Option<serde_json::Value>,
    logs: Option<Vec<String>>,
    accounts: Option<Vec<serde_json::Value>>,
    units_consumed: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct RpcError {
    code: i64,
    message: String,
}

impl TransactionSimulator {
    pub fn new(rpc_url: String) -> Self {
        Self {
            client: Client::new(),
            rpc_url,
        }
    }

    pub async fn simulate_transaction(
        &self,
        edge_id: Uuid,
        transaction_base64: &str,
    ) -> AppResult<SimulationResult> {
        let request = SimulateRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "simulateTransaction".to_string(),
            params: vec![
                serde_json::Value::String(transaction_base64.to_string()),
                serde_json::json!({
                    "encoding": "base64",
                    "commitment": "processed",
                    "replaceRecentBlockhash": true,
                }),
            ],
        };

        let response = self
            .client
            .post(&self.rpc_url)
            .json(&request)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Simulation request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::ExternalApi(format!(
                "RPC returned error status: {}",
                response.status()
            )));
        }

        let result: SimulateResponse = response
            .json()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to parse simulation response: {}", e)))?;

        if let Some(error) = result.error {
            return Ok(SimulationResult {
                edge_id,
                success: false,
                simulated_profit_lamports: None,
                simulated_gas_lamports: 0,
                logs: vec![],
                error: Some(format!("RPC error {}: {}", error.code, error.message)),
                atomicity: AtomicityLevel::NonAtomic,
                profit_guaranteed: false,
                simulation_slot: 0,
            });
        }

        if let Some(sim_result) = result.result {
            let success = sim_result.value.err.is_none();
            let logs = sim_result.value.logs.unwrap_or_default();
            let units_consumed = sim_result.value.units_consumed.unwrap_or(0);

            // Estimate gas cost (5000 lamports per compute unit is a rough estimate)
            let gas_lamports = (units_consumed as f64 * 0.000001 * 1e9) as u64;

            // Parse logs to extract profit information
            let (profit, atomicity, guaranteed) = self.analyze_simulation_logs(&logs);

            let error = if !success {
                Some(format!("Transaction simulation failed: {:?}", sim_result.value.err))
            } else {
                None
            };

            Ok(SimulationResult {
                edge_id,
                success,
                simulated_profit_lamports: profit,
                simulated_gas_lamports: gas_lamports,
                logs,
                error,
                atomicity,
                profit_guaranteed: guaranteed && success,
                simulation_slot: sim_result.context.slot,
            })
        } else {
            Ok(SimulationResult {
                edge_id,
                success: false,
                simulated_profit_lamports: None,
                simulated_gas_lamports: 0,
                logs: vec![],
                error: Some("Empty simulation result".to_string()),
                atomicity: AtomicityLevel::NonAtomic,
                profit_guaranteed: false,
                simulation_slot: 0,
            })
        }
    }

    fn analyze_simulation_logs(&self, logs: &[String]) -> (Option<i64>, AtomicityLevel, bool) {
        let mut atomicity = AtomicityLevel::NonAtomic;
        let mut profit_guaranteed = false;
        let mut profit: Option<i64> = None;

        for log in logs {
            // Check for flash loan patterns (indicates atomic execution)
            if log.contains("flash_loan") || log.contains("FlashLoan") {
                atomicity = AtomicityLevel::FullyAtomic;
                profit_guaranteed = true;
            }

            // Check for Jito bundle patterns
            if log.contains("jito") || log.contains("bundle") {
                atomicity = AtomicityLevel::FullyAtomic;
            }

            // Check for atomic swap patterns
            if log.contains("atomic") || log.contains("swap_exact") {
                if atomicity != AtomicityLevel::FullyAtomic {
                    atomicity = AtomicityLevel::PartiallyAtomic;
                }
            }

            // Try to extract profit from logs (format: "profit: 12345")
            if log.contains("profit:") {
                if let Some(profit_str) = log.split("profit:").nth(1) {
                    if let Ok(p) = profit_str.trim().split_whitespace().next().unwrap_or("0").parse::<i64>() {
                        profit = Some(p);
                    }
                }
            }
        }

        (profit, atomicity, profit_guaranteed)
    }

    pub async fn estimate_priority_fee(&self) -> AppResult<u64> {
        let request = SimulateRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "getRecentPrioritizationFees".to_string(),
            params: vec![],
        };

        let response = self
            .client
            .post(&self.rpc_url)
            .json(&request)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Priority fee request failed: {}", e)))?;

        if !response.status().is_success() {
            return Ok(5000); // Default priority fee
        }

        #[derive(Deserialize)]
        struct PriorityFeeResponse {
            result: Option<Vec<PriorityFeeEntry>>,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct PriorityFeeEntry {
            prioritization_fee: u64,
        }

        let result: PriorityFeeResponse = response
            .json()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to parse priority fee: {}", e)))?;

        if let Some(fees) = result.result {
            if !fees.is_empty() {
                // Use median of recent fees
                let mut fee_values: Vec<u64> = fees.iter().map(|f| f.prioritization_fee).collect();
                fee_values.sort();
                let median_idx = fee_values.len() / 2;
                return Ok(fee_values[median_idx]);
            }
        }

        Ok(5000) // Default
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub enabled: bool,
    pub max_simulation_time_ms: u64,
    pub require_profit_guarantee: bool,
    pub min_profit_after_gas_bps: i32,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_simulation_time_ms: 5000,
            require_profit_guarantee: false,
            min_profit_after_gas_bps: 10,
        }
    }
}
