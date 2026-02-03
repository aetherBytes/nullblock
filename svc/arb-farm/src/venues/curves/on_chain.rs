use serde::{Deserialize, Serialize};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use std::sync::Arc;

use crate::error::{AppError, AppResult};
use crate::helius::HeliusClient;

use super::math::{
    BondingCurveParams, MoonshotCurveParams, MoonshotCurveType, MOONSHOT_FEE_BPS, PUMP_FUN_FEE_BPS,
    PUMP_FUN_GLOBAL_STATE, PUMP_FUN_PROGRAM_ID, TOKEN_2022_PROGRAM_ID,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnChainCurveState {
    pub mint: String,
    pub bonding_curve_address: String,
    pub associated_bonding_curve: String,
    pub virtual_sol_reserves: u64,
    pub virtual_token_reserves: u64,
    pub real_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub token_total_supply: u64,
    pub is_complete: bool,
    pub creator: String,
    pub created_slot: u64,
    #[serde(default)]
    pub is_mayhem_mode: bool,
}

impl OnChainCurveState {
    pub fn to_params(&self) -> BondingCurveParams {
        BondingCurveParams {
            virtual_sol_reserves: self.virtual_sol_reserves,
            virtual_token_reserves: self.virtual_token_reserves,
            real_sol_reserves: self.real_sol_reserves,
            real_token_reserves: self.real_token_reserves,
            fee_bps: PUMP_FUN_FEE_BPS,
        }
    }

    pub fn graduation_progress(&self) -> f64 {
        self.to_params().graduation_progress()
    }

    pub fn current_price_sol(&self) -> f64 {
        if self.virtual_token_reserves == 0 {
            return 0.0;
        }
        self.virtual_sol_reserves as f64 / self.virtual_token_reserves as f64
    }

    pub fn market_cap_sol(&self) -> f64 {
        self.current_price_sol() * (self.token_total_supply as f64 / 1e6)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoonshotOnChainState {
    pub mint: String,
    pub curve_address: String,
    pub virtual_sol_reserves: u64,
    pub virtual_token_reserves: u64,
    pub real_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub curve_type: MoonshotCurveType,
    pub graduation_threshold_usd: f64,
    pub is_graduated: bool,
    pub creator: String,
    pub dex_pool: Option<String>,
}

impl MoonshotOnChainState {
    pub fn to_params(&self, sol_price_usd: f64) -> MoonshotCurveParams {
        MoonshotCurveParams {
            base_params: BondingCurveParams {
                virtual_sol_reserves: self.virtual_sol_reserves,
                virtual_token_reserves: self.virtual_token_reserves,
                real_sol_reserves: self.real_sol_reserves,
                real_token_reserves: self.real_token_reserves,
                fee_bps: MOONSHOT_FEE_BPS,
            },
            curve_type: self.curve_type,
            graduation_threshold_usd: self.graduation_threshold_usd,
            sol_price_usd,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaydiumPoolInfo {
    pub pool_address: String,
    pub base_mint: String,
    pub quote_mint: String,
    pub base_reserve: u64,
    pub quote_reserve: u64,
    pub lp_mint: String,
    pub open_time: u64,
}

pub struct OnChainFetcher {
    rpc_client: Arc<RpcClient>,
    helius_client: Option<Arc<HeliusClient>>,
}

impl OnChainFetcher {
    pub fn new(rpc_url: &str) -> Self {
        Self {
            rpc_client: Arc::new(RpcClient::new(rpc_url.to_string())),
            helius_client: None,
        }
    }

    #[cfg(test)]
    pub fn new_mock() -> Self {
        Self {
            rpc_client: Arc::new(RpcClient::new("http://localhost:8899".to_string())),
            helius_client: None,
        }
    }

    pub fn with_helius(mut self, helius: Arc<HeliusClient>) -> Self {
        self.helius_client = Some(helius);
        self
    }

    /// Detect which token program a mint uses by checking the mint account's owner
    /// Returns true if Token-2022, false if standard SPL Token
    pub async fn is_token_2022(&self, mint: &str) -> AppResult<bool> {
        let mint_pubkey = Pubkey::from_str(mint)
            .map_err(|e| AppError::Validation(format!("Invalid mint address: {}", e)))?;

        let account = self
            .rpc_client
            .get_account(&mint_pubkey)
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to fetch mint account: {}", e)))?;

        let token_2022_program = Pubkey::from_str(TOKEN_2022_PROGRAM_ID)
            .map_err(|e| AppError::Internal(format!("Invalid token-2022 program: {}", e)))?;

        Ok(account.owner == token_2022_program)
    }

    pub async fn get_bonding_curve_state(&self, mint: &str) -> AppResult<OnChainCurveState> {
        self.get_pump_fun_bonding_curve(mint).await
    }

    pub async fn get_pump_fun_bonding_curve(&self, mint: &str) -> AppResult<OnChainCurveState> {
        let mint_pubkey = Pubkey::from_str(mint)
            .map_err(|e| AppError::Validation(format!("Invalid mint address: {}", e)))?;

        let program_id = Pubkey::from_str(PUMP_FUN_PROGRAM_ID)
            .map_err(|e| AppError::Internal(format!("Invalid program ID: {}", e)))?;

        let (bonding_curve_pda, _bump) =
            Pubkey::find_program_address(&[b"bonding-curve", mint_pubkey.as_ref()], &program_id);

        let token_2022_program = Pubkey::from_str(TOKEN_2022_PROGRAM_ID)
            .map_err(|e| AppError::Internal(format!("Invalid token-2022 program: {}", e)))?;
        let (associated_bonding_curve, _bump2) = Pubkey::find_program_address(
            &[
                bonding_curve_pda.as_ref(),
                token_2022_program.as_ref(),
                mint_pubkey.as_ref(),
            ],
            &spl_associated_token_account::ID,
        );

        let account_data = self
            .rpc_client
            .get_account_data(&bonding_curve_pda)
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to fetch bonding curve: {}", e)))?;

        if account_data.len() < 89 {
            return Err(AppError::Internal(format!(
                "Bonding curve data too short: {} bytes",
                account_data.len()
            )));
        }

        let virtual_token_reserves =
            u64::from_le_bytes(account_data[8..16].try_into().expect("validated len >= 89"));
        let virtual_sol_reserves = u64::from_le_bytes(
            account_data[16..24]
                .try_into()
                .expect("validated len >= 89"),
        );
        let real_token_reserves = u64::from_le_bytes(
            account_data[24..32]
                .try_into()
                .expect("validated len >= 89"),
        );
        let real_sol_reserves = u64::from_le_bytes(
            account_data[32..40]
                .try_into()
                .expect("validated len >= 89"),
        );
        let token_total_supply = u64::from_le_bytes(
            account_data[40..48]
                .try_into()
                .expect("validated len >= 89"),
        );
        let is_complete = account_data[48] != 0;

        // Creator pubkey is at bytes 49-80 (32 bytes), need at least 81 bytes
        let creator = if account_data.len() >= 81 {
            match Pubkey::try_from(&account_data[49..81]) {
                Ok(p) if p != Pubkey::default() => p.to_string(),
                _ => {
                    tracing::warn!(
                        mint = %mint,
                        data_len = account_data.len(),
                        "Bonding curve has invalid or zero creator address"
                    );
                    String::new()
                }
            }
        } else {
            tracing::warn!(
                mint = %mint,
                data_len = account_data.len(),
                "Bonding curve data too short to contain creator address"
            );
            String::new()
        };

        // Mayhem mode flag is at byte 81 (after creator pubkey)
        let is_mayhem_mode = if account_data.len() >= 82 {
            account_data[81] != 0
        } else {
            false
        };

        if is_mayhem_mode {
            tracing::info!(
                mint = %mint,
                "🎲 Token has mayhem mode enabled - requires special fee recipient"
            );
        }

        Ok(OnChainCurveState {
            mint: mint.to_string(),
            bonding_curve_address: bonding_curve_pda.to_string(),
            associated_bonding_curve: associated_bonding_curve.to_string(),
            virtual_sol_reserves,
            virtual_token_reserves,
            real_sol_reserves,
            real_token_reserves,
            token_total_supply,
            is_complete,
            creator,
            created_slot: 0,
            is_mayhem_mode,
        })
    }

    pub async fn find_raydium_pool(&self, mint: &str) -> AppResult<Option<RaydiumPoolInfo>> {
        let mint_pubkey = Pubkey::from_str(mint)
            .map_err(|e| AppError::Validation(format!("Invalid mint address: {}", e)))?;

        let sol_mint = spl_token::native_mint::ID;

        let raydium_amm_program = Pubkey::from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8")
            .map_err(|e| AppError::Internal(format!("Invalid Raydium program ID: {}", e)))?;

        let (pool_pda, _) = Pubkey::find_program_address(
            &[
                raydium_amm_program.as_ref(),
                mint_pubkey.as_ref(),
                sol_mint.as_ref(),
            ],
            &raydium_amm_program,
        );

        match self.rpc_client.get_account_data(&pool_pda).await {
            Ok(data) => {
                if data.len() < 200 {
                    return Ok(None);
                }

                Ok(Some(RaydiumPoolInfo {
                    pool_address: pool_pda.to_string(),
                    base_mint: mint.to_string(),
                    quote_mint: sol_mint.to_string(),
                    base_reserve: u64::from_le_bytes(data[104..112].try_into().unwrap_or([0; 8])),
                    quote_reserve: u64::from_le_bytes(data[112..120].try_into().unwrap_or([0; 8])),
                    lp_mint: String::new(),
                    open_time: u64::from_le_bytes(data[120..128].try_into().unwrap_or([0; 8])),
                }))
            }
            Err(_) => Ok(None),
        }
    }

    pub async fn is_token_graduated(&self, mint: &str) -> AppResult<GraduationStatus> {
        let curve_state = self.get_pump_fun_bonding_curve(mint).await?;

        if curve_state.is_complete {
            let raydium_pool = self.find_raydium_pool(mint).await?;

            return Ok(GraduationStatus::Graduated {
                graduation_slot: 0,
                raydium_pool: raydium_pool.map(|p| p.pool_address),
            });
        }

        let progress = curve_state.graduation_progress();

        if progress >= 95.0 {
            Ok(GraduationStatus::NearGraduation { progress })
        } else {
            Ok(GraduationStatus::PreGraduation { progress })
        }
    }

    pub async fn get_token_balance(&self, owner: &str, mint: &str) -> AppResult<u64> {
        let owner_pubkey = Pubkey::from_str(owner)
            .map_err(|e| AppError::Validation(format!("Invalid owner address: {}", e)))?;
        let mint_pubkey = Pubkey::from_str(mint)
            .map_err(|e| AppError::Validation(format!("Invalid mint address: {}", e)))?;

        // Try standard SPL Token ATA first
        let spl_ata =
            spl_associated_token_account::get_associated_token_address(&owner_pubkey, &mint_pubkey);

        if let Ok(balance) = self.rpc_client.get_token_account_balance(&spl_ata).await {
            if let Ok(amount) = balance.amount.parse::<u64>() {
                if amount > 0 {
                    return Ok(amount);
                }
            }
        }

        // Try Token-2022 ATA (used by pump.fun)
        let token_2022_program = Pubkey::from_str(TOKEN_2022_PROGRAM_ID)
            .map_err(|e| AppError::Internal(format!("Invalid token-2022 program: {}", e)))?;
        let token_2022_ata =
            spl_associated_token_account::get_associated_token_address_with_program_id(
                &owner_pubkey,
                &mint_pubkey,
                &token_2022_program,
            );

        match self
            .rpc_client
            .get_token_account_balance(&token_2022_ata)
            .await
        {
            Ok(balance) => {
                let amount = balance
                    .amount
                    .parse::<u64>()
                    .map_err(|e| AppError::Internal(format!("Failed to parse balance: {}", e)))?;
                Ok(amount)
            }
            Err(_) => Ok(0),
        }
    }

    pub async fn get_sol_balance(&self, address: &str) -> AppResult<u64> {
        let pubkey = Pubkey::from_str(address)
            .map_err(|e| AppError::Validation(format!("Invalid address: {}", e)))?;

        let balance = self
            .rpc_client
            .get_balance(&pubkey)
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to fetch balance: {}", e)))?;

        Ok(balance)
    }

    pub async fn get_pump_fun_global_state(&self) -> AppResult<PumpFunGlobalState> {
        let global_pubkey = Pubkey::from_str(PUMP_FUN_GLOBAL_STATE)
            .map_err(|e| AppError::Internal(format!("Invalid global state address: {}", e)))?;

        let account_data = self
            .rpc_client
            .get_account_data(&global_pubkey)
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to fetch global state: {}", e)))?;

        if account_data.len() < 40 {
            return Err(AppError::Internal(
                "Global state data too short".to_string(),
            ));
        }

        Ok(PumpFunGlobalState {
            initialized: account_data[8] != 0,
            fee_basis_points: u64::from_le_bytes(
                account_data[16..24]
                    .try_into()
                    .expect("validated len >= 40"),
            ),
            initial_virtual_token_reserves: u64::from_le_bytes(
                account_data[24..32]
                    .try_into()
                    .expect("validated len >= 40"),
            ),
            initial_virtual_sol_reserves: u64::from_le_bytes(
                account_data[32..40]
                    .try_into()
                    .expect("validated len >= 40"),
            ),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraduationStatus {
    PreGraduation {
        progress: f64,
    },
    NearGraduation {
        progress: f64,
    },
    Graduating,
    Graduated {
        graduation_slot: u64,
        raydium_pool: Option<String>,
    },
    Failed {
        reason: String,
    },
}

impl GraduationStatus {
    pub fn is_graduated(&self) -> bool {
        matches!(self, GraduationStatus::Graduated { .. })
    }

    pub fn progress(&self) -> f64 {
        match self {
            GraduationStatus::PreGraduation { progress } => *progress,
            GraduationStatus::NearGraduation { progress } => *progress,
            GraduationStatus::Graduating => 99.0,
            GraduationStatus::Graduated { .. } => 100.0,
            GraduationStatus::Failed { .. } => 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PumpFunGlobalState {
    pub initialized: bool,
    pub fee_basis_points: u64,
    pub initial_virtual_token_reserves: u64,
    pub initial_virtual_sol_reserves: u64,
}

pub fn derive_pump_fun_bonding_curve(mint: &str) -> AppResult<(String, String)> {
    let mint_pubkey = Pubkey::from_str(mint)
        .map_err(|e| AppError::Validation(format!("Invalid mint address: {}", e)))?;

    let program_id = Pubkey::from_str(PUMP_FUN_PROGRAM_ID)
        .map_err(|e| AppError::Internal(format!("Invalid program ID: {}", e)))?;

    let (bonding_curve_pda, _bump) =
        Pubkey::find_program_address(&[b"bonding-curve", mint_pubkey.as_ref()], &program_id);

    let token_2022_program = Pubkey::from_str(TOKEN_2022_PROGRAM_ID)
        .map_err(|e| AppError::Internal(format!("Invalid token-2022 program: {}", e)))?;
    let (associated_bonding_curve, _bump2) = Pubkey::find_program_address(
        &[
            bonding_curve_pda.as_ref(),
            token_2022_program.as_ref(),
            mint_pubkey.as_ref(),
        ],
        &spl_associated_token_account::ID,
    );

    Ok((
        bonding_curve_pda.to_string(),
        associated_bonding_curve.to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_bonding_curve_address() {
        let mint = "6p6xgHyF7AeE6TZkSmFsko111111111111111111111";

        let result = derive_pump_fun_bonding_curve(mint);

        assert!(result.is_ok());
        let (bonding_curve, associated) = result.unwrap();
        assert!(!bonding_curve.is_empty());
        assert!(!associated.is_empty());
    }

    #[test]
    fn test_graduation_status() {
        let pre = GraduationStatus::PreGraduation { progress: 50.0 };
        assert!(!pre.is_graduated());
        assert_eq!(pre.progress(), 50.0);

        let graduated = GraduationStatus::Graduated {
            graduation_slot: 12345,
            raydium_pool: Some("PoolAddress".to_string()),
        };
        assert!(graduated.is_graduated());
        assert_eq!(graduated.progress(), 100.0);
    }
}
