use tracing::{debug, info, warn};

use crate::helius::HeliusClient;

const MAX_RETRIES: u32 = 3;
const RETRY_DELAY_MS: u64 = 2000;

#[derive(Debug, Clone)]
pub struct TxSettlement {
    pub sol_delta_lamports: i64,
    pub gas_lamports: u64,
    pub source: &'static str,
}

impl TxSettlement {
    pub fn sol_delta_sol(&self) -> f64 {
        self.sol_delta_lamports as f64 / 1_000_000_000.0
    }

    fn estimated_fallback() -> Self {
        Self {
            sol_delta_lamports: 0,
            gas_lamports: 0,
            source: "estimated",
        }
    }

    fn unknown_fallback(_entry_amount_sol: f64) -> Self {
        Self {
            sol_delta_lamports: 0,
            gas_lamports: 0,
            source: "unknown",
        }
    }
}

pub async fn resolve_settlement(
    helius: &HeliusClient,
    signature: &str,
    wallet_pubkey: &str,
) -> TxSettlement {
    if signature.starts_with("INFERRED_") {
        debug!(
            "Skipping settlement for inferred signature: {}",
            &signature[..30.min(signature.len())]
        );
        return TxSettlement::estimated_fallback();
    }

    for attempt in 0..MAX_RETRIES {
        if attempt > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(RETRY_DELAY_MS)).await;
        }

        match helius.get_transaction(signature).await {
            Ok(Some(tx_response)) => {
                let meta = match tx_response.meta {
                    Some(m) => m,
                    None => {
                        warn!(
                            "Transaction {} has no meta, using estimated PnL",
                            &signature[..16]
                        );
                        return TxSettlement::estimated_fallback();
                    }
                };

                let account_keys = match tx_response.transaction {
                    Some(ref inner) => &inner.message.account_keys,
                    None => {
                        warn!(
                            "Transaction {} has no inner transaction, using estimated PnL",
                            &signature[..16]
                        );
                        return TxSettlement::estimated_fallback();
                    }
                };

                let wallet_index = account_keys.iter().position(|key| key == wallet_pubkey);

                match wallet_index {
                    Some(idx)
                        if idx < meta.pre_balances.len() && idx < meta.post_balances.len() =>
                    {
                        let pre = meta.pre_balances[idx] as i64;
                        let post = meta.post_balances[idx] as i64;
                        let sol_delta = post - pre;

                        info!(
                            "[onchain] sig={} | wallet_idx={} | pre={} | post={} | delta={} lamports ({:.6} SOL) | fee={}",
                            &signature[..16],
                            idx,
                            meta.pre_balances[idx],
                            meta.post_balances[idx],
                            sol_delta,
                            sol_delta as f64 / 1e9,
                            meta.fee
                        );

                        return TxSettlement {
                            sol_delta_lamports: sol_delta,
                            gas_lamports: meta.fee,
                            source: "onchain",
                        };
                    }
                    Some(idx) => {
                        warn!(
                            "Wallet index {} out of bounds (pre_balances={}, post_balances={})",
                            idx,
                            meta.pre_balances.len(),
                            meta.post_balances.len()
                        );
                        return TxSettlement::estimated_fallback();
                    }
                    None => {
                        warn!(
                            "Wallet {} not found in transaction {} account keys ({} keys)",
                            &wallet_pubkey[..12],
                            &signature[..16],
                            account_keys.len()
                        );
                        return TxSettlement::estimated_fallback();
                    }
                }
            }
            Ok(None) => {
                debug!(
                    "Transaction {} not indexed yet (attempt {}/{})",
                    &signature[..16],
                    attempt + 1,
                    MAX_RETRIES
                );
            }
            Err(e) => {
                warn!(
                    "Failed to fetch transaction {} (attempt {}/{}): {}",
                    &signature[..16],
                    attempt + 1,
                    MAX_RETRIES,
                    e
                );
            }
        }
    }

    warn!(
        "[estimated] Could not resolve on-chain settlement for {} after {} attempts",
        &signature[..16],
        MAX_RETRIES
    );
    TxSettlement::estimated_fallback()
}

pub async fn resolve_inferred_settlement(
    helius: &HeliusClient,
    wallet_pubkey: &str,
    entry_amount_sol: f64,
    token_mint: Option<&str>,
) -> TxSettlement {
    match helius.get_signatures_for_address(wallet_pubkey, 30).await {
        Ok(signatures) => {
            let recent_sigs: Vec<_> = signatures.into_iter().filter(|s| s.err.is_none()).collect();

            if recent_sigs.is_empty() {
                warn!("[inferred] No recent successful transactions found for wallet");
                return TxSettlement::unknown_fallback(entry_amount_sol);
            }

            let mut total_positive_delta: i64 = 0;
            let mut total_gas: u64 = 0;
            let mut found_sell = false;

            for sig_info in recent_sigs.iter().take(15) {
                match helius.get_transaction(&sig_info.signature).await {
                    Ok(Some(tx)) => {
                        let meta = match tx.meta {
                            Some(m) => m,
                            None => continue,
                        };
                        let keys = match tx.transaction {
                            Some(ref inner) => &inner.message.account_keys,
                            None => continue,
                        };

                        if let Some(mint) = token_mint {
                            if !keys.iter().any(|k| k == mint) {
                                debug!(
                                    "[inferred] TX {} does not involve token {} - skipping",
                                    &sig_info.signature[..16],
                                    &mint[..8]
                                );
                                continue;
                            }
                        }

                        let idx = match keys.iter().position(|k| k == wallet_pubkey) {
                            Some(i)
                                if i < meta.pre_balances.len() && i < meta.post_balances.len() =>
                            {
                                i
                            }
                            _ => continue,
                        };

                        let delta = meta.post_balances[idx] as i64 - meta.pre_balances[idx] as i64;

                        if delta > 0 {
                            info!(
                                "[inferred-onchain] Found sell TX {} for {} | delta={} lamports ({:.6} SOL) | fee={}",
                                &sig_info.signature[..16],
                                token_mint.map(|m| &m[..8]).unwrap_or("unknown"),
                                delta, delta as f64 / 1e9, meta.fee
                            );
                            total_positive_delta += delta;
                            total_gas += meta.fee;
                            found_sell = true;
                            break;
                        }
                    }
                    Ok(None) => continue,
                    Err(e) => {
                        debug!(
                            "[inferred] Failed to fetch TX {}: {}",
                            &sig_info.signature[..16],
                            e
                        );
                        continue;
                    }
                }
            }

            if found_sell {
                let entry_lamports = (entry_amount_sol * 1e9) as i64;
                let realized_delta = total_positive_delta - entry_lamports;

                info!(
                    "[inferred-onchain] Resolved: received={:.6} SOL | entry={:.6} SOL | realized_pnl={:.6} SOL",
                    total_positive_delta as f64 / 1e9,
                    entry_amount_sol,
                    realized_delta as f64 / 1e9,
                );

                TxSettlement {
                    sol_delta_lamports: total_positive_delta,
                    gas_lamports: total_gas,
                    source: "inferred-onchain",
                }
            } else {
                warn!("[inferred] No positive-delta TX found for {} in recent history - marking as unknown",
                    token_mint.map(|m| &m[..8]).unwrap_or("unknown"));
                TxSettlement::unknown_fallback(entry_amount_sol)
            }
        }
        Err(e) => {
            warn!("[inferred] Failed to search wallet transactions: {}", e);
            TxSettlement::unknown_fallback(entry_amount_sol)
        }
    }
}
