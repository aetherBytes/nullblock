use serde::{Deserialize, Serialize};

pub const ALLOWED_PROGRAMS: &[&str] = &[
    "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4",  // Jupiter v6 Aggregator
    "JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB",  // Jupiter v4
    "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8", // Raydium AMM v4
    "CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK", // Raydium CPMM
    "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P",  // pump.fun
    "MoonCVVNZFSYkqNXP6bxHLPL6QQJiMagDL3qcqUQTrG",  // moonshot
    "11111111111111111111111111111111",              // System Program
    "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",  // Token Program
    "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL", // Associated Token Program
    "ComputeBudget111111111111111111111111111111",   // Compute Budget
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbFarmPolicy {
    pub max_transaction_amount_lamports: u64,
    pub daily_volume_limit_lamports: u64,
    pub max_transactions_per_day: u32,
    pub require_simulation: bool,
    pub allowed_programs: Vec<String>,
    pub blocked_tokens: Vec<String>,
    pub min_profit_threshold_lamports: u64,
}

impl Default for ArbFarmPolicy {
    fn default() -> Self {
        Self {
            max_transaction_amount_lamports: 5_000_000_000, // 5 SOL
            daily_volume_limit_lamports: 25_000_000_000,    // 25 SOL
            max_transactions_per_day: 100,
            require_simulation: true,
            allowed_programs: ALLOWED_PROGRAMS.iter().map(|s| s.to_string()).collect(),
            blocked_tokens: Vec::new(),
            min_profit_threshold_lamports: 1_000_000, // 0.001 SOL (1000 lamports min profit)
        }
    }
}

impl ArbFarmPolicy {
    pub fn dev_testing() -> Self {
        Self {
            max_transaction_amount_lamports: 5_000_000_000, // 5 SOL
            daily_volume_limit_lamports: 25_000_000_000,    // 25 SOL
            max_transactions_per_day: 200,
            require_simulation: true,
            allowed_programs: ALLOWED_PROGRAMS.iter().map(|s| s.to_string()).collect(),
            blocked_tokens: Vec::new(),
            min_profit_threshold_lamports: 500_000, // 0.0005 SOL
        }
    }

    pub fn conservative() -> Self {
        Self {
            max_transaction_amount_lamports: 1_000_000_000, // 1 SOL
            daily_volume_limit_lamports: 5_000_000_000,     // 5 SOL
            max_transactions_per_day: 50,
            require_simulation: true,
            allowed_programs: ALLOWED_PROGRAMS.iter().map(|s| s.to_string()).collect(),
            blocked_tokens: Vec::new(),
            min_profit_threshold_lamports: 5_000_000, // 0.005 SOL
        }
    }

    pub fn is_program_allowed(&self, program_id: &str) -> bool {
        self.allowed_programs.iter().any(|p| p == program_id)
    }

    pub fn is_token_blocked(&self, mint: &str) -> bool {
        self.blocked_tokens.iter().any(|t| t == mint)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyViolation {
    pub violation_type: PolicyViolationType,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyViolationType {
    AmountExceedsLimit,
    DailyVolumeExceeded,
    TransactionCountExceeded,
    ProgramNotAllowed,
    TokenBlocked,
    SimulationRequired,
    ProfitBelowThreshold,
}

impl PolicyViolation {
    pub fn amount_exceeded(amount: u64, limit: u64) -> Self {
        Self {
            violation_type: PolicyViolationType::AmountExceedsLimit,
            message: format!(
                "Transaction amount {} lamports exceeds limit of {} lamports",
                amount, limit
            ),
            details: Some(serde_json::json!({
                "amount": amount,
                "limit": limit,
                "amount_sol": amount as f64 / 1_000_000_000.0,
                "limit_sol": limit as f64 / 1_000_000_000.0,
            })),
        }
    }

    pub fn daily_volume_exceeded(current: u64, limit: u64) -> Self {
        Self {
            violation_type: PolicyViolationType::DailyVolumeExceeded,
            message: format!(
                "Daily volume {} lamports would exceed limit of {} lamports",
                current, limit
            ),
            details: Some(serde_json::json!({
                "current_volume": current,
                "limit": limit,
            })),
        }
    }

    pub fn program_not_allowed(program_id: &str) -> Self {
        Self {
            violation_type: PolicyViolationType::ProgramNotAllowed,
            message: format!("Program {} is not in the allowed list", program_id),
            details: Some(serde_json::json!({
                "program_id": program_id,
            })),
        }
    }

    pub fn token_blocked(mint: &str) -> Self {
        Self {
            violation_type: PolicyViolationType::TokenBlocked,
            message: format!("Token {} is blocked", mint),
            details: Some(serde_json::json!({
                "mint": mint,
            })),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyUsage {
    pub date: chrono::NaiveDate,
    pub total_volume_lamports: u64,
    pub transaction_count: u32,
}

impl DailyUsage {
    pub fn new() -> Self {
        Self {
            date: chrono::Utc::now().date_naive(),
            total_volume_lamports: 0,
            transaction_count: 0,
        }
    }

    pub fn is_today(&self) -> bool {
        self.date == chrono::Utc::now().date_naive()
    }

    pub fn reset_if_new_day(&mut self) {
        if !self.is_today() {
            self.date = chrono::Utc::now().date_naive();
            self.total_volume_lamports = 0;
            self.transaction_count = 0;
        }
    }

    pub fn can_execute(&self, amount: u64, policy: &ArbFarmPolicy) -> Result<(), PolicyViolation> {
        if amount > policy.max_transaction_amount_lamports {
            return Err(PolicyViolation::amount_exceeded(
                amount,
                policy.max_transaction_amount_lamports,
            ));
        }

        let new_volume = self.total_volume_lamports + amount;
        if new_volume > policy.daily_volume_limit_lamports {
            return Err(PolicyViolation::daily_volume_exceeded(
                new_volume,
                policy.daily_volume_limit_lamports,
            ));
        }

        if self.transaction_count >= policy.max_transactions_per_day {
            return Err(PolicyViolation {
                violation_type: PolicyViolationType::TransactionCountExceeded,
                message: format!(
                    "Transaction count {} would exceed daily limit of {}",
                    self.transaction_count + 1,
                    policy.max_transactions_per_day
                ),
                details: None,
            });
        }

        Ok(())
    }

    pub fn record_transaction(&mut self, amount: u64) {
        self.reset_if_new_day();
        self.total_volume_lamports += amount;
        self.transaction_count += 1;
    }
}

impl Default for DailyUsage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_defaults() {
        let policy = ArbFarmPolicy::default();
        assert_eq!(policy.max_transaction_amount_lamports, 5_000_000_000);
        assert!(policy.require_simulation);
    }

    #[test]
    fn test_program_allowed() {
        let policy = ArbFarmPolicy::default();
        assert!(policy.is_program_allowed("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"));
        assert!(!policy.is_program_allowed("SomeRandomProgram111111111111111111111111111"));
    }

    #[test]
    fn test_daily_usage() {
        let policy = ArbFarmPolicy::default();
        let mut usage = DailyUsage::new();

        // Should allow first transaction
        assert!(usage.can_execute(1_000_000_000, &policy).is_ok());

        // Record it
        usage.record_transaction(1_000_000_000);

        // Should still allow more
        assert!(usage.can_execute(1_000_000_000, &policy).is_ok());

        // But not over limit
        assert!(usage.can_execute(6_000_000_000, &policy).is_err());
    }
}
