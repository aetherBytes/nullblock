use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CapitalReservation {
    pub strategy_id: Uuid,
    pub position_id: Uuid,
    pub amount_lamports: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct StrategyAllocation {
    pub max_percent: f64,
    pub reserved_lamports: u64,
    pub active_positions: u32,
    pub max_positions: u32,
}

#[derive(Debug)]
pub struct CapitalManager {
    total_balance_lamports: RwLock<u64>,
    strategy_allocations: RwLock<HashMap<Uuid, StrategyAllocation>>,
    reservations: RwLock<HashMap<Uuid, CapitalReservation>>,
    global_reserved_lamports: RwLock<u64>,
    db_pool: Option<PgPool>,
}

impl CapitalManager {
    pub fn new() -> Self {
        Self {
            total_balance_lamports: RwLock::new(0),
            strategy_allocations: RwLock::new(HashMap::new()),
            reservations: RwLock::new(HashMap::new()),
            global_reserved_lamports: RwLock::new(0),
            db_pool: None,
        }
    }

    pub fn with_db_pool(mut self, pool: PgPool) -> Self {
        self.db_pool = Some(pool);
        self
    }

    pub async fn load_reservations_from_db(&self) -> Result<usize, String> {
        let Some(pool) = &self.db_pool else {
            return Ok(0);
        };

        let rows: Vec<(Uuid, Uuid, i64, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
            "SELECT position_id, strategy_id, amount_lamports, created_at FROM capital_reservations"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Failed to load capital reservations: {}", e))?;

        let mut reservations = self.reservations.write().await;
        let mut global = self.global_reserved_lamports.write().await;
        let mut allocations = self.strategy_allocations.write().await;

        for (position_id, strategy_id, amount_lamports, created_at) in rows {
            let amount = amount_lamports as u64;

            reservations.insert(
                position_id,
                CapitalReservation {
                    strategy_id,
                    position_id,
                    amount_lamports: amount,
                    created_at,
                },
            );

            *global += amount;

            if let Some(alloc) = allocations.get_mut(&strategy_id) {
                alloc.reserved_lamports += amount;
                alloc.active_positions += 1;
            }
        }

        let count = reservations.len();
        if count > 0 {
            info!(
                "📊 Loaded {} capital reservations from DB ({} SOL reserved)",
                count,
                *global as f64 / 1_000_000_000.0
            );
        }

        Ok(count)
    }

    async fn persist_reservation(&self, reservation: &CapitalReservation) {
        let Some(pool) = &self.db_pool else {
            return;
        };

        if let Err(e) = sqlx::query(
            r#"
            INSERT INTO capital_reservations (position_id, strategy_id, amount_lamports, created_at)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (position_id) DO UPDATE SET
                strategy_id = EXCLUDED.strategy_id,
                amount_lamports = EXCLUDED.amount_lamports
            "#,
        )
        .bind(reservation.position_id)
        .bind(reservation.strategy_id)
        .bind(reservation.amount_lamports as i64)
        .bind(reservation.created_at)
        .execute(pool)
        .await
        {
            warn!("Failed to persist capital reservation: {}", e);
        }
    }

    async fn delete_reservation(&self, position_id: Uuid) -> Result<(), String> {
        let Some(pool) = &self.db_pool else {
            return Ok(()); // No DB configured, nothing to delete
        };

        sqlx::query("DELETE FROM capital_reservations WHERE position_id = $1")
            .bind(position_id)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to delete capital reservation: {}", e))?;

        Ok(())
    }

    pub async fn update_balance(&self, balance_lamports: u64) {
        let mut total = self.total_balance_lamports.write().await;
        *total = balance_lamports;
        info!(
            "💰 Capital manager updated balance: {} SOL",
            balance_lamports as f64 / 1_000_000_000.0
        );
    }

    pub async fn get_balance(&self) -> u64 {
        *self.total_balance_lamports.read().await
    }

    pub async fn register_strategy(&self, strategy_id: Uuid, max_percent: f64, max_positions: u32) {
        let mut allocations = self.strategy_allocations.write().await;
        allocations.insert(
            strategy_id,
            StrategyAllocation {
                max_percent,
                reserved_lamports: 0,
                active_positions: 0,
                max_positions,
            },
        );
        debug!(
            "Registered strategy {} with {}% max allocation, {} max positions",
            strategy_id, max_percent, max_positions
        );
    }

    pub async fn can_allocate(
        &self,
        strategy_id: Uuid,
        amount_lamports: u64,
    ) -> Result<(), CapitalError> {
        let total_balance = *self.total_balance_lamports.read().await;
        if total_balance == 0 {
            return Err(CapitalError::NoBalance);
        }

        let allocations = self.strategy_allocations.read().await;
        let allocation = allocations
            .get(&strategy_id)
            .ok_or(CapitalError::StrategyNotRegistered)?;

        // Check if strategy has room for more positions
        if allocation.active_positions >= allocation.max_positions {
            return Err(CapitalError::MaxPositionsReached {
                current: allocation.active_positions,
                max: allocation.max_positions,
            });
        }

        // Calculate max capital this strategy can use
        let max_capital = (total_balance as f64 * allocation.max_percent / 100.0) as u64;
        let new_reserved = allocation.reserved_lamports + amount_lamports;

        if new_reserved > max_capital {
            return Err(CapitalError::AllocationExceeded {
                requested: amount_lamports,
                available: max_capital.saturating_sub(allocation.reserved_lamports),
                max_percent: allocation.max_percent,
            });
        }

        // Check global capital availability (total - reserved)
        let global_reserved = *self.global_reserved_lamports.read().await;
        let available_global = total_balance.saturating_sub(global_reserved);

        if amount_lamports > available_global {
            return Err(CapitalError::InsufficientGlobalCapital {
                requested: amount_lamports,
                available: available_global,
            });
        }

        Ok(())
    }

    pub async fn reserve_capital(
        &self,
        strategy_id: Uuid,
        position_id: Uuid,
        amount_lamports: u64,
    ) -> Result<(), CapitalError> {
        // First verify we can allocate
        self.can_allocate(strategy_id, amount_lamports).await?;

        let reservation = CapitalReservation {
            strategy_id,
            position_id,
            amount_lamports,
            created_at: chrono::Utc::now(),
        };

        // Persist to DB FIRST before in-memory update (crash-safe)
        self.persist_reservation(&reservation).await;

        // Update strategy allocation
        {
            let mut allocations = self.strategy_allocations.write().await;
            if let Some(allocation) = allocations.get_mut(&strategy_id) {
                allocation.reserved_lamports += amount_lamports;
                allocation.active_positions += 1;
            }
        }

        // Update global reserved
        {
            let mut global = self.global_reserved_lamports.write().await;
            *global += amount_lamports;
        }

        // Record reservation in-memory
        {
            let mut reservations = self.reservations.write().await;
            reservations.insert(position_id, reservation);
        }

        info!(
            "💼 Reserved {} SOL for strategy {} position {}",
            amount_lamports as f64 / 1_000_000_000.0,
            strategy_id,
            position_id
        );

        Ok(())
    }

    pub async fn release_capital(&self, position_id: Uuid) -> Option<u64> {
        // Get reservation (read only first, don't remove yet)
        let reservation = {
            let reservations = self.reservations.read().await;
            reservations.get(&position_id).cloned()
        }?;

        // Delete from DB FIRST - if this fails, keep in-memory state unchanged
        if let Err(e) = self.delete_reservation(position_id).await {
            warn!(
                "❌ Failed to delete capital reservation from DB for position {}: {} - keeping in-memory reservation",
                position_id, e
            );
            return None;
        }

        // DB delete succeeded - now safe to update in-memory state
        {
            let mut reservations = self.reservations.write().await;
            reservations.remove(&position_id);
        }

        // Update strategy allocation
        {
            let mut allocations = self.strategy_allocations.write().await;
            if let Some(allocation) = allocations.get_mut(&reservation.strategy_id) {
                allocation.reserved_lamports = allocation
                    .reserved_lamports
                    .saturating_sub(reservation.amount_lamports);
                allocation.active_positions = allocation.active_positions.saturating_sub(1);
            }
        }

        // Update global reserved
        {
            let mut global = self.global_reserved_lamports.write().await;
            *global = global.saturating_sub(reservation.amount_lamports);
        }

        info!(
            "💸 Released {} SOL from strategy {} position {}",
            reservation.amount_lamports as f64 / 1_000_000_000.0,
            reservation.strategy_id,
            position_id
        );

        Some(reservation.amount_lamports)
    }

    pub async fn release_partial_capital(
        &self,
        position_id: Uuid,
        exit_percent: f64,
    ) -> Option<u64> {
        if exit_percent <= 0.0 || exit_percent > 100.0 {
            warn!(
                "Invalid exit percent {} for partial capital release on position {}",
                exit_percent, position_id
            );
            return None;
        }

        // Get current reservation
        let reservation = {
            let reservations = self.reservations.read().await;
            reservations.get(&position_id).cloned()
        }?;

        // Calculate amount to release (proportional to exit percent), rounded to nearest lamport
        let release_lamports =
            ((reservation.amount_lamports as f64) * (exit_percent / 100.0)).round() as u64;
        let release_lamports = release_lamports.min(reservation.amount_lamports); // Cap at reserved amount
        let new_reserved = reservation.amount_lamports.saturating_sub(release_lamports);

        // Update DB first
        if let Some(pool) = &self.db_pool {
            if let Err(e) = sqlx::query(
                "UPDATE capital_reservations SET amount_lamports = $1 WHERE position_id = $2",
            )
            .bind(new_reserved as i64)
            .bind(position_id)
            .execute(pool)
            .await
            {
                warn!(
                    "Failed to update partial capital release in DB for position {}: {}",
                    position_id, e
                );
                return None;
            }
        }

        // Update in-memory reservation
        {
            let mut reservations = self.reservations.write().await;
            if let Some(res) = reservations.get_mut(&position_id) {
                res.amount_lamports = new_reserved;
            }
        }

        // Update strategy allocation
        {
            let mut allocations = self.strategy_allocations.write().await;
            if let Some(allocation) = allocations.get_mut(&reservation.strategy_id) {
                allocation.reserved_lamports = allocation
                    .reserved_lamports
                    .saturating_sub(release_lamports);
            }
        }

        // Update global reserved
        {
            let mut global = self.global_reserved_lamports.write().await;
            *global = global.saturating_sub(release_lamports);
        }

        info!(
            "💸 Partial release: {} SOL ({:.1}%) from strategy {} position {} | Remaining: {} SOL",
            release_lamports as f64 / 1_000_000_000.0,
            exit_percent,
            reservation.strategy_id,
            position_id,
            new_reserved as f64 / 1_000_000_000.0
        );

        Some(release_lamports)
    }

    pub async fn get_strategy_usage(&self, strategy_id: Uuid) -> Option<StrategyUsage> {
        let total_balance = *self.total_balance_lamports.read().await;
        let allocations = self.strategy_allocations.read().await;

        allocations.get(&strategy_id).map(|allocation| {
            let max_capital = (total_balance as f64 * allocation.max_percent / 100.0) as u64;
            StrategyUsage {
                strategy_id,
                max_allocation_percent: allocation.max_percent,
                max_allocation_lamports: max_capital,
                current_reserved_lamports: allocation.reserved_lamports,
                available_lamports: max_capital.saturating_sub(allocation.reserved_lamports),
                active_positions: allocation.active_positions,
                max_positions: allocation.max_positions,
            }
        })
    }

    pub async fn get_global_usage(&self) -> GlobalCapitalUsage {
        // Acquire all locks at once for consistent snapshot
        // Order: balance -> reserved -> reservations (prevent deadlock by consistent ordering)
        let total = *self.total_balance_lamports.read().await;
        let reserved = *self.global_reserved_lamports.read().await;
        let active_count = self.reservations.read().await.len();

        GlobalCapitalUsage {
            total_balance_lamports: total,
            global_reserved_lamports: reserved,
            available_lamports: total.saturating_sub(reserved),
            active_reservations: active_count,
            utilization_percent: if total > 0 {
                (reserved as f64 / total as f64) * 100.0
            } else {
                0.0
            },
        }
    }

    pub async fn update_total_balance(&self, balance_lamports: u64) {
        self.update_balance(balance_lamports).await;
    }

    pub async fn get_all_strategy_usage(&self) -> Vec<StrategyUsage> {
        // Acquire locks in consistent order: balance first, then allocations
        // Note: Brief race possible between reads, but acceptable for reporting endpoints
        let total_balance = *self.total_balance_lamports.read().await;
        let allocations = self.strategy_allocations.read().await;

        allocations
            .iter()
            .map(|(strategy_id, allocation)| {
                let max_capital = (total_balance as f64 * allocation.max_percent / 100.0) as u64;
                StrategyUsage {
                    strategy_id: *strategy_id,
                    max_allocation_percent: allocation.max_percent,
                    max_allocation_lamports: max_capital,
                    current_reserved_lamports: allocation.reserved_lamports,
                    available_lamports: max_capital.saturating_sub(allocation.reserved_lamports),
                    active_positions: allocation.active_positions,
                    max_positions: allocation.max_positions,
                }
            })
            .collect()
    }

    pub async fn get_active_reservations(&self) -> Vec<CapitalReservation> {
        let reservations = self.reservations.read().await;
        reservations.values().cloned().collect()
    }

    pub async fn register_strategies_equal(
        &self,
        strategy_ids: Vec<Uuid>,
        max_positions_each: u32,
    ) {
        if strategy_ids.is_empty() {
            return;
        }

        let equal_percent = 100.0 / strategy_ids.len() as f64;

        info!(
            "📊 Registering {} strategies with equal allocation: {:.2}% each",
            strategy_ids.len(),
            equal_percent
        );

        for strategy_id in strategy_ids {
            self.register_strategy(strategy_id, equal_percent, max_positions_each)
                .await;
        }
    }

    pub async fn rebalance_equal(&self) {
        let allocations = self.strategy_allocations.read().await;
        let count = allocations.len();
        drop(allocations);

        if count == 0 {
            return;
        }

        let equal_percent = 100.0 / count as f64;

        let mut allocations = self.strategy_allocations.write().await;
        for allocation in allocations.values_mut() {
            allocation.max_percent = equal_percent;
        }

        info!(
            "📊 Rebalanced {} strategies to {:.2}% each",
            count, equal_percent
        );
    }

    pub async fn get_strategy_count(&self) -> usize {
        self.strategy_allocations.read().await.len()
    }

    pub async fn start_balance_refresh(
        self: Arc<Self>,
        rpc_url: String,
        wallet_address: String,
        refresh_interval_secs: u64,
    ) {
        info!(
            "💰 Starting periodic balance refresh (every {}s)",
            refresh_interval_secs
        );

        let client = match reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to create HTTP client for balance refresh: {}", e);
                return;
            }
        };

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(refresh_interval_secs)).await;

                match Self::fetch_sol_balance(&client, &rpc_url, &wallet_address).await {
                    Ok(balance_lamports) => {
                        let current = self.get_balance().await;
                        if balance_lamports != current {
                            debug!(
                                "💰 Balance changed: {} -> {} SOL",
                                current as f64 / 1_000_000_000.0,
                                balance_lamports as f64 / 1_000_000_000.0
                            );
                        }
                        self.update_balance(balance_lamports).await;
                    }
                    Err(e) => {
                        debug!("Failed to refresh balance: {}", e);
                    }
                }
            }
        });
    }

    async fn fetch_sol_balance(
        client: &reqwest::Client,
        rpc_url: &str,
        wallet_address: &str,
    ) -> Result<u64, String> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getBalance",
            "params": [wallet_address]
        });

        // Per-request timeout (10s) to prevent hanging on slow RPC
        let response = client
            .post(rpc_url)
            .header("Content-Type", "application/json")
            .json(&request)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| format!("RPC request failed (timeout 10s): {}", e))?;

        if !response.status().is_success() {
            return Err(format!("RPC returned error: {}", response.status()));
        }

        #[derive(serde::Deserialize)]
        struct RpcResponse {
            result: Option<BalanceResult>,
        }
        #[derive(serde::Deserialize)]
        struct BalanceResult {
            value: u64,
        }

        let rpc_response: RpcResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        rpc_response
            .result
            .map(|r| r.value)
            .ok_or_else(|| "Missing result in response".to_string())
    }
}

impl Default for CapitalManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct StrategyUsage {
    pub strategy_id: Uuid,
    pub max_allocation_percent: f64,
    pub max_allocation_lamports: u64,
    pub current_reserved_lamports: u64,
    pub available_lamports: u64,
    pub active_positions: u32,
    pub max_positions: u32,
}

#[derive(Debug, Clone)]
pub struct GlobalCapitalUsage {
    pub total_balance_lamports: u64,
    pub global_reserved_lamports: u64,
    pub available_lamports: u64,
    pub active_reservations: usize,
    pub utilization_percent: f64,
}

#[derive(Debug, Clone)]
pub enum CapitalError {
    NoBalance,
    StrategyNotRegistered,
    MaxPositionsReached {
        current: u32,
        max: u32,
    },
    AllocationExceeded {
        requested: u64,
        available: u64,
        max_percent: f64,
    },
    InsufficientGlobalCapital {
        requested: u64,
        available: u64,
    },
}

impl std::fmt::Display for CapitalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CapitalError::NoBalance => write!(f, "No balance available"),
            CapitalError::StrategyNotRegistered => write!(f, "Strategy not registered"),
            CapitalError::MaxPositionsReached { current, max } => {
                write!(f, "Max positions reached: {}/{}", current, max)
            }
            CapitalError::AllocationExceeded {
                requested,
                available,
                max_percent,
            } => write!(
                f,
                "Allocation exceeded: requested {} SOL, available {} SOL ({}% max)",
                *requested as f64 / 1_000_000_000.0,
                *available as f64 / 1_000_000_000.0,
                max_percent
            ),
            CapitalError::InsufficientGlobalCapital {
                requested,
                available,
            } => write!(
                f,
                "Insufficient global capital: requested {} SOL, available {} SOL",
                *requested as f64 / 1_000_000_000.0,
                *available as f64 / 1_000_000_000.0
            ),
        }
    }
}

impl std::error::Error for CapitalError {}
