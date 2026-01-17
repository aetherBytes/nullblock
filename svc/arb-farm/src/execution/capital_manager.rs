use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use tracing::{info, warn, debug};

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
}

impl CapitalManager {
    pub fn new() -> Self {
        Self {
            total_balance_lamports: RwLock::new(0),
            strategy_allocations: RwLock::new(HashMap::new()),
            reservations: RwLock::new(HashMap::new()),
            global_reserved_lamports: RwLock::new(0),
        }
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

    pub async fn register_strategy(
        &self,
        strategy_id: Uuid,
        max_percent: f64,
        max_positions: u32,
    ) {
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

        // Record reservation
        {
            let mut reservations = self.reservations.write().await;
            reservations.insert(
                position_id,
                CapitalReservation {
                    strategy_id,
                    position_id,
                    amount_lamports,
                    created_at: chrono::Utc::now(),
                },
            );
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
        // Get and remove reservation
        let reservation = {
            let mut reservations = self.reservations.write().await;
            reservations.remove(&position_id)
        }?;

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
        let total = *self.total_balance_lamports.read().await;
        let reserved = *self.global_reserved_lamports.read().await;
        let reservations = self.reservations.read().await;

        GlobalCapitalUsage {
            total_balance_lamports: total,
            global_reserved_lamports: reserved,
            available_lamports: total.saturating_sub(reserved),
            active_reservations: reservations.len(),
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
    MaxPositionsReached { current: u32, max: u32 },
    AllocationExceeded {
        requested: u64,
        available: u64,
        max_percent: f64,
    },
    InsufficientGlobalCapital { requested: u64, available: u64 },
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
