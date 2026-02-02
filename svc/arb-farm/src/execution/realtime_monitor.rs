use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::error::AppResult;
use crate::events::{edge as edge_topics, ArbEvent};
use crate::helius::laserstream::{AccountUpdate, LaserStreamClient, LaserStreamStatus};
use crate::venues::curves::math::PUMP_FUN_PROGRAM_ID;

use super::position_manager::{OpenPosition, PositionManager};
use super::position_monitor::PositionMonitor;

pub struct RealtimePositionMonitor {
    laserstream: Arc<LaserStreamClient>,
    position_manager: Arc<PositionManager>,
    position_monitor: Arc<PositionMonitor>,
    event_tx: broadcast::Sender<ArbEvent>,
    subscribed_positions: Arc<RwLock<HashMap<String, PositionSubscription>>>,
}

struct PositionSubscription {
    position_id: Uuid,
    mint: String,
    bonding_curve_address: String,
}

impl RealtimePositionMonitor {
    pub fn new(
        laserstream: Arc<LaserStreamClient>,
        position_manager: Arc<PositionManager>,
        position_monitor: Arc<PositionMonitor>,
        event_tx: broadcast::Sender<ArbEvent>,
    ) -> Self {
        Self {
            laserstream,
            position_manager,
            position_monitor,
            event_tx,
            subscribed_positions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn start(&self) -> AppResult<()> {
        if !self.laserstream.is_configured() {
            warn!("⚠️ LaserStream not configured - real-time position monitoring disabled");
            return Ok(());
        }

        if self.laserstream.get_status().await != LaserStreamStatus::Connected {
            info!("📡 Connecting to LaserStream for real-time position monitoring...");
            self.laserstream.connect().await.map_err(|e| {
                crate::error::AppError::ExternalApi(format!("LaserStream connect failed: {}", e))
            })?;
        }

        let positions = self.position_manager.get_open_positions().await;
        if !positions.is_empty() {
            info!(
                "📊 Subscribing to {} existing positions for real-time monitoring",
                positions.len()
            );
            for position in positions {
                if let Err(e) = self.subscribe_position(&position).await {
                    warn!("Failed to subscribe to position {}: {}", position.id, e);
                }
            }
        }

        self.start_update_listener().await;
        self.start_event_listener().await;
        self.start_connection_monitor().await;

        info!("🔭 Real-time position monitor fully started");

        Ok(())
    }

    async fn start_connection_monitor(&self) {
        let laserstream = self.laserstream.clone();
        let subscribed_positions = self.subscribed_positions.clone();

        tokio::spawn(async move {
            const CHECK_INTERVAL_SECS: u64 = 30;
            const DISCONNECT_WARNING_THRESHOLD_SECS: u64 = 30;
            let mut consecutive_disconnected_checks = 0u32;

            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(CHECK_INTERVAL_SECS)).await;

                let status = laserstream.get_status().await;
                let sub_count = subscribed_positions.read().await.len();

                match status {
                    LaserStreamStatus::Connected => {
                        if consecutive_disconnected_checks > 0 {
                            info!("✅ LaserStream reconnected - real-time monitoring resumed ({} subscriptions)", sub_count);
                        }
                        consecutive_disconnected_checks = 0;
                    }
                    LaserStreamStatus::Disconnected | LaserStreamStatus::Reconnecting => {
                        consecutive_disconnected_checks += 1;
                        let disconnected_secs =
                            consecutive_disconnected_checks as u64 * CHECK_INTERVAL_SECS;

                        if disconnected_secs >= DISCONNECT_WARNING_THRESHOLD_SECS {
                            warn!(
                                "⚠️ LaserStream disconnected for {}s - falling back to polling-based price updates ({} positions affected)",
                                disconnected_secs,
                                sub_count
                            );
                        }
                    }
                    LaserStreamStatus::Connecting => {
                        debug!("LaserStream connecting...");
                    }
                }
            }
        });
    }

    pub async fn subscribe_position(&self, position: &OpenPosition) -> AppResult<()> {
        let bonding_curve_address = derive_bonding_curve_pda(&position.token_mint)?;

        {
            let subs = self.subscribed_positions.read().await;
            if subs.contains_key(&bonding_curve_address) {
                debug!(
                    "Already subscribed to bonding curve {} for position {}",
                    &bonding_curve_address[..12],
                    position.id
                );
                return Ok(());
            }
        }

        info!(
            "📡 Subscribing to bonding curve {} for {} ({})",
            &bonding_curve_address[..12],
            position
                .token_symbol
                .as_deref()
                .unwrap_or(&position.token_mint[..8]),
            position.id
        );

        self.laserstream
            .subscribe_accounts(vec![bonding_curve_address.clone()])
            .await
            .map_err(|e| crate::error::AppError::ExternalApi(format!("Subscribe failed: {}", e)))?;

        let mut subs = self.subscribed_positions.write().await;
        subs.insert(
            bonding_curve_address.clone(),
            PositionSubscription {
                position_id: position.id,
                mint: position.token_mint.clone(),
                bonding_curve_address,
            },
        );

        Ok(())
    }

    pub async fn unsubscribe_position(&self, position_id: Uuid) -> AppResult<()> {
        let address_to_remove = {
            let subs = self.subscribed_positions.read().await;
            subs.iter()
                .find(|(_, sub)| sub.position_id == position_id)
                .map(|(addr, _)| addr.clone())
        };

        if let Some(address) = address_to_remove {
            info!(
                "📡 Unsubscribing from bonding curve {} for position {}",
                &address[..12],
                position_id
            );

            self.laserstream
                .unsubscribe_accounts(vec![address.clone()])
                .await
                .map_err(|e| {
                    crate::error::AppError::ExternalApi(format!("Unsubscribe failed: {}", e))
                })?;

            let mut subs = self.subscribed_positions.write().await;
            subs.remove(&address);
        }

        Ok(())
    }

    async fn start_update_listener(&self) {
        let mut receiver = self.laserstream.subscribe_account_updates();
        let subscribed_positions = self.subscribed_positions.clone();
        let position_manager = self.position_manager.clone();
        let position_monitor = self.position_monitor.clone();

        tokio::spawn(async move {
            info!("🔭 Real-time price listener started - waiting for account updates");

            loop {
                match receiver.recv().await {
                    Ok(update) => {
                        let subscription = {
                            let subs = subscribed_positions.read().await;
                            subs.get(&update.pubkey).cloned()
                        };

                        if let Some(sub) = subscription {
                            if let Some(price) = decode_bonding_curve_price(&update) {
                                debug!(
                                    "📈 Real-time price update for {}: {:.12} SOL",
                                    &sub.mint[..8],
                                    price
                                );

                                let signals = position_manager.update_price(&sub.mint, price).await;

                                if !signals.is_empty() {
                                    info!(
                                        "🚨 {} exit signals triggered by real-time update for {}",
                                        signals.len(),
                                        &sub.mint[..8]
                                    );

                                    for signal in signals {
                                        info!(
                                            "🎯 Exit signal: {:?} for position {} ({}% exit)",
                                            signal.reason, signal.position_id, signal.exit_percent
                                        );
                                        if let Err(e) =
                                            position_monitor.trigger_exit_with_reason(&signal).await
                                        {
                                            error!(
                                                "Failed to execute exit for position {}: {}",
                                                signal.position_id, e
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        if e == tokio::sync::broadcast::error::RecvError::Closed {
                            warn!("Account update channel closed, stopping listener");
                            break;
                        }
                    }
                }
            }
        });
    }

    async fn start_event_listener(&self) {
        let mut event_rx = self.event_tx.subscribe();
        let subscribed_positions = self.subscribed_positions.clone();
        let laserstream = self.laserstream.clone();
        let position_manager = self.position_manager.clone();

        tokio::spawn(async move {
            info!("🔭 Position event listener started - auto-subscribing to new positions");

            loop {
                match event_rx.recv().await {
                    Ok(event) => {
                        if event.topic == edge_topics::EXECUTED {
                            if let Some(mint) = event
                                .payload
                                .get("mint")
                                .or_else(|| event.payload.get("token_mint"))
                                .and_then(|v| v.as_str())
                            {
                                if let Ok(bonding_curve_address) = derive_bonding_curve_pda(mint) {
                                    let already_subscribed = {
                                        let subs = subscribed_positions.read().await;
                                        subs.contains_key(&bonding_curve_address)
                                    };

                                    if !already_subscribed {
                                        info!(
                                            "📡 Auto-subscribing to bonding curve {} for newly opened position",
                                            &bonding_curve_address[..12]
                                        );

                                        if let Err(e) = laserstream
                                            .subscribe_accounts(vec![bonding_curve_address.clone()])
                                            .await
                                        {
                                            warn!(
                                                "Failed to auto-subscribe to {}: {}",
                                                &bonding_curve_address[..12],
                                                e
                                            );
                                        } else {
                                            if let Some(position) = position_manager
                                                .get_open_position_for_mint(mint)
                                                .await
                                            {
                                                let mut subs = subscribed_positions.write().await;
                                                subs.insert(
                                                    bonding_curve_address.clone(),
                                                    PositionSubscription {
                                                        position_id: position.id,
                                                        mint: mint.to_string(),
                                                        bonding_curve_address,
                                                    },
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        } else if event.event_type == "position.exit_completed" {
                            if let Some(mint) =
                                event.payload.get("token_mint").and_then(|v| v.as_str())
                            {
                                if let Ok(bonding_curve_address) = derive_bonding_curve_pda(mint) {
                                    let has_subscription = {
                                        let subs = subscribed_positions.read().await;
                                        subs.contains_key(&bonding_curve_address)
                                    };

                                    if has_subscription {
                                        let has_other_positions =
                                            position_manager.has_open_position_for_mint(mint).await;

                                        if !has_other_positions {
                                            info!(
                                                "📡 Auto-unsubscribing from bonding curve {} (position closed)",
                                                &bonding_curve_address[..12]
                                            );

                                            if let Err(e) = laserstream
                                                .unsubscribe_accounts(vec![
                                                    bonding_curve_address.clone()
                                                ])
                                                .await
                                            {
                                                warn!(
                                                    "Failed to auto-unsubscribe from {}: {}",
                                                    &bonding_curve_address[..12],
                                                    e
                                                );
                                            } else {
                                                let mut subs = subscribed_positions.write().await;
                                                subs.remove(&bonding_curve_address);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        warn!("Event channel closed, stopping event listener");
                        break;
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!("Event listener lagged by {} events", n);
                    }
                }
            }
        });
    }

    pub async fn get_subscribed_count(&self) -> usize {
        self.subscribed_positions.read().await.len()
    }
}

fn derive_bonding_curve_pda(mint: &str) -> AppResult<String> {
    let mint_pubkey = Pubkey::from_str(mint)
        .map_err(|e| crate::error::AppError::Validation(format!("Invalid mint address: {}", e)))?;

    let program_id = Pubkey::from_str(PUMP_FUN_PROGRAM_ID)
        .map_err(|e| crate::error::AppError::Internal(format!("Invalid program ID: {}", e)))?;

    let (bonding_curve_pda, _bump) =
        Pubkey::find_program_address(&[b"bonding-curve", mint_pubkey.as_ref()], &program_id);

    Ok(bonding_curve_pda.to_string())
}

fn decode_bonding_curve_price(update: &AccountUpdate) -> Option<f64> {
    use base64::{engine::general_purpose::STANDARD, Engine};

    let account_data = STANDARD.decode(&update.data).ok()?;

    if account_data.len() < 48 {
        return None;
    }

    let virtual_token_reserves = u64::from_le_bytes(account_data[8..16].try_into().ok()?);
    let virtual_sol_reserves = u64::from_le_bytes(account_data[16..24].try_into().ok()?);

    if virtual_token_reserves == 0 {
        return None;
    }

    Some(virtual_sol_reserves as f64 / virtual_token_reserves as f64)
}

impl Clone for PositionSubscription {
    fn clone(&self) -> Self {
        Self {
            position_id: self.position_id,
            mint: self.mint.clone(),
            bonding_curve_address: self.bonding_curve_address.clone(),
        }
    }
}
