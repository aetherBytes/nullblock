use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
    transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address_with_program_id;
use std::str::FromStr;
use std::sync::Arc;

use crate::error::{AppError, AppResult};
use crate::venues::curves::math::{
    calculate_min_sol_out, calculate_min_tokens_out, BondingCurveMath, BuyResult, PumpFunCurve,
    SellResult, PUMP_FUN_EVENT_AUTHORITY, PUMP_FUN_FEE_PROGRAM, PUMP_FUN_FEE_RECIPIENT,
    PUMP_FUN_GLOBAL_STATE, PUMP_FUN_PROGRAM_ID, TOKEN_2022_PROGRAM_ID,
};
use crate::venues::curves::on_chain::{derive_pump_fun_bonding_curve, OnChainCurveState, OnChainFetcher};

const DEFAULT_COMPUTE_UNITS: u32 = 200_000;
const DEFAULT_PRIORITY_FEE_MICRO_LAMPORTS: u64 = 1_000_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurveBuildResult {
    pub transaction_base64: String,
    pub expected_tokens_out: Option<u64>,
    pub expected_sol_out: Option<u64>,
    pub min_tokens_out: Option<u64>,
    pub min_sol_out: Option<u64>,
    pub price_impact_percent: f64,
    pub fee_lamports: u64,
    pub compute_units: u32,
    pub priority_fee_lamports: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurveBuyParams {
    pub mint: String,
    pub sol_amount_lamports: u64,
    pub slippage_bps: u16,
    pub user_wallet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurveSellParams {
    pub mint: String,
    pub token_amount: u64,
    pub slippage_bps: u16,
    pub user_wallet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatedTrade {
    pub is_buy: bool,
    pub input_amount: u64,
    pub output_amount: u64,
    pub min_output: u64,
    pub price_impact_percent: f64,
    pub fee_lamports: u64,
    pub effective_price: f64,
}

#[derive(BorshSerialize, BorshDeserialize)]
struct PumpFunBuyArgs {
    amount: u64,
    max_sol_cost: u64,
}

#[derive(BorshSerialize, BorshDeserialize)]
struct PumpFunSellArgs {
    amount: u64,
    min_sol_output: u64,
}

pub struct CurveTransactionBuilder {
    rpc_url: String,
    on_chain_fetcher: Arc<OnChainFetcher>,
    compute_units: u32,
    priority_fee_micro_lamports: u64,
}

impl CurveTransactionBuilder {
    pub fn new(rpc_url: &str) -> Self {
        Self {
            rpc_url: rpc_url.to_string(),
            on_chain_fetcher: Arc::new(OnChainFetcher::new(rpc_url)),
            compute_units: DEFAULT_COMPUTE_UNITS,
            priority_fee_micro_lamports: DEFAULT_PRIORITY_FEE_MICRO_LAMPORTS,
        }
    }

    async fn get_recent_blockhash(&self) -> AppResult<solana_sdk::hash::Hash> {
        let client = reqwest::Client::new();
        let response = client
            .post(&self.rpc_url)
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "getLatestBlockhash",
                "params": [{"commitment": "confirmed"}]
            }))
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch blockhash: {}", e)))?;

        #[derive(Deserialize)]
        struct BlockhashResponse {
            result: BlockhashResult,
        }
        #[derive(Deserialize)]
        struct BlockhashResult {
            value: BlockhashValue,
        }
        #[derive(Deserialize)]
        struct BlockhashValue {
            blockhash: String,
        }

        let data: BlockhashResponse = response
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse blockhash response: {}", e)))?;

        solana_sdk::hash::Hash::from_str(&data.result.value.blockhash)
            .map_err(|e| AppError::Internal(format!("Invalid blockhash: {}", e)))
    }

    pub fn with_compute_units(mut self, units: u32) -> Self {
        self.compute_units = units;
        self
    }

    pub fn with_priority_fee(mut self, micro_lamports: u64) -> Self {
        self.priority_fee_micro_lamports = micro_lamports;
        self
    }

    pub fn with_on_chain_fetcher(mut self, fetcher: Arc<OnChainFetcher>) -> Self {
        self.on_chain_fetcher = fetcher;
        self
    }

    pub async fn simulate_buy(&self, params: &CurveBuyParams) -> AppResult<SimulatedTrade> {
        let curve_state = self
            .on_chain_fetcher
            .get_pump_fun_bonding_curve(&params.mint)
            .await?;

        if curve_state.is_complete {
            return Err(AppError::Validation(
                "Token has graduated. Use DEX for trading.".to_string(),
            ));
        }

        let curve = PumpFunCurve::with_current_state(
            curve_state.virtual_sol_reserves,
            curve_state.virtual_token_reserves,
            curve_state.real_sol_reserves,
            curve_state.real_token_reserves,
        );

        let result: BuyResult = curve.calculate_buy_amount(params.sol_amount_lamports);
        let min_tokens = calculate_min_tokens_out(result.tokens_out, params.slippage_bps);

        Ok(SimulatedTrade {
            is_buy: true,
            input_amount: params.sol_amount_lamports,
            output_amount: result.tokens_out,
            min_output: min_tokens,
            price_impact_percent: result.price_impact_percent,
            fee_lamports: result.fee_lamports,
            effective_price: result.price_per_token,
        })
    }

    pub async fn simulate_sell(&self, params: &CurveSellParams) -> AppResult<SimulatedTrade> {
        let curve_state = self
            .on_chain_fetcher
            .get_pump_fun_bonding_curve(&params.mint)
            .await?;

        if curve_state.is_complete {
            return Err(AppError::Validation(
                "Token has graduated. Use DEX for trading.".to_string(),
            ));
        }

        let curve = PumpFunCurve::with_current_state(
            curve_state.virtual_sol_reserves,
            curve_state.virtual_token_reserves,
            curve_state.real_sol_reserves,
            curve_state.real_token_reserves,
        );

        let result: SellResult = curve.calculate_sell_amount(params.token_amount);
        let min_sol = calculate_min_sol_out(result.sol_out, params.slippage_bps);

        Ok(SimulatedTrade {
            is_buy: false,
            input_amount: params.token_amount,
            output_amount: result.sol_out,
            min_output: min_sol,
            price_impact_percent: result.price_impact_percent,
            fee_lamports: result.fee_lamports,
            effective_price: result.price_per_token,
        })
    }

    pub async fn build_pump_fun_buy(
        &self,
        params: &CurveBuyParams,
    ) -> AppResult<CurveBuildResult> {
        let curve_state = self
            .on_chain_fetcher
            .get_pump_fun_bonding_curve(&params.mint)
            .await?;

        if curve_state.is_complete {
            return Err(AppError::Validation(
                "Token has graduated. Use DEX for trading.".to_string(),
            ));
        }

        let simulation = self.simulate_buy(params).await?;
        let min_tokens = simulation.min_output;

        let instructions = self.create_pump_fun_buy_instructions(
            &params.mint,
            &params.user_wallet,
            &curve_state,
            params.sol_amount_lamports,
            min_tokens,
        )?;

        let user_pubkey = Pubkey::from_str(&params.user_wallet)
            .map_err(|e| AppError::Validation(format!("Invalid user wallet: {}", e)))?;

        let recent_blockhash = self.get_recent_blockhash().await?;
        let message = solana_sdk::message::Message::new_with_blockhash(
            &instructions,
            Some(&user_pubkey),
            &recent_blockhash,
        );
        let tx = Transaction::new_unsigned(message);

        let tx_bytes = bincode::serialize(&tx)
            .map_err(|e| AppError::Internal(format!("Failed to serialize transaction: {}", e)))?;
        let tx_base64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &tx_bytes);

        let priority_fee_lamports =
            (self.compute_units as u64 * self.priority_fee_micro_lamports) / 1_000_000;

        Ok(CurveBuildResult {
            transaction_base64: tx_base64,
            expected_tokens_out: Some(simulation.output_amount),
            expected_sol_out: None,
            min_tokens_out: Some(min_tokens),
            min_sol_out: None,
            price_impact_percent: simulation.price_impact_percent,
            fee_lamports: simulation.fee_lamports,
            compute_units: self.compute_units,
            priority_fee_lamports,
        })
    }

    pub async fn build_pump_fun_sell(
        &self,
        params: &CurveSellParams,
    ) -> AppResult<CurveBuildResult> {
        let curve_state = self
            .on_chain_fetcher
            .get_pump_fun_bonding_curve(&params.mint)
            .await?;

        if curve_state.is_complete {
            return Err(AppError::Validation(
                "Token has graduated. Use DEX for trading.".to_string(),
            ));
        }

        let simulation = self.simulate_sell(params).await?;
        let min_sol = simulation.min_output;

        let instructions = self.create_pump_fun_sell_instructions(
            &params.mint,
            &params.user_wallet,
            &curve_state,
            params.token_amount,
            min_sol,
        )?;

        let user_pubkey = Pubkey::from_str(&params.user_wallet)
            .map_err(|e| AppError::Validation(format!("Invalid user wallet: {}", e)))?;

        let recent_blockhash = self.get_recent_blockhash().await?;
        let message = solana_sdk::message::Message::new_with_blockhash(
            &instructions,
            Some(&user_pubkey),
            &recent_blockhash,
        );
        let tx = Transaction::new_unsigned(message);

        let tx_bytes = bincode::serialize(&tx)
            .map_err(|e| AppError::Internal(format!("Failed to serialize transaction: {}", e)))?;
        let tx_base64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &tx_bytes);

        let priority_fee_lamports =
            (self.compute_units as u64 * self.priority_fee_micro_lamports) / 1_000_000;

        Ok(CurveBuildResult {
            transaction_base64: tx_base64,
            expected_tokens_out: None,
            expected_sol_out: Some(simulation.output_amount),
            min_tokens_out: None,
            min_sol_out: Some(min_sol),
            price_impact_percent: simulation.price_impact_percent,
            fee_lamports: simulation.fee_lamports,
            compute_units: self.compute_units,
            priority_fee_lamports,
        })
    }

    fn create_pump_fun_buy_instructions(
        &self,
        mint: &str,
        user_wallet: &str,
        curve_state: &OnChainCurveState,
        sol_amount: u64,
        min_tokens_out: u64,
    ) -> AppResult<Vec<Instruction>> {
        let mut instructions = Vec::new();

        instructions.push(ComputeBudgetInstruction::set_compute_unit_limit(
            self.compute_units,
        ));

        instructions.push(ComputeBudgetInstruction::set_compute_unit_price(
            self.priority_fee_micro_lamports,
        ));

        let program_id = Pubkey::from_str(PUMP_FUN_PROGRAM_ID)
            .map_err(|e| AppError::Internal(format!("Invalid program ID: {}", e)))?;
        let global_state = Pubkey::from_str(PUMP_FUN_GLOBAL_STATE)
            .map_err(|e| AppError::Internal(format!("Invalid global state: {}", e)))?;
        let fee_recipient = Pubkey::from_str(PUMP_FUN_FEE_RECIPIENT)
            .map_err(|e| AppError::Internal(format!("Invalid fee recipient: {}", e)))?;
        let event_authority = Pubkey::from_str(PUMP_FUN_EVENT_AUTHORITY)
            .map_err(|e| AppError::Internal(format!("Invalid event authority: {}", e)))?;
        let fee_program = Pubkey::from_str(PUMP_FUN_FEE_PROGRAM)
            .map_err(|e| AppError::Internal(format!("Invalid fee program: {}", e)))?;
        let token_2022_program = Pubkey::from_str(TOKEN_2022_PROGRAM_ID)
            .map_err(|e| AppError::Internal(format!("Invalid token-2022 program: {}", e)))?;
        let mint_pubkey = Pubkey::from_str(mint)
            .map_err(|e| AppError::Validation(format!("Invalid mint: {}", e)))?;
        let user_pubkey = Pubkey::from_str(user_wallet)
            .map_err(|e| AppError::Validation(format!("Invalid user wallet: {}", e)))?;
        let bonding_curve = Pubkey::from_str(&curve_state.bonding_curve_address)
            .map_err(|e| AppError::Internal(format!("Invalid bonding curve: {}", e)))?;
        let associated_bonding_curve = Pubkey::from_str(&curve_state.associated_bonding_curve)
            .map_err(|e| AppError::Internal(format!("Invalid associated bonding curve: {}", e)))?;
        let creator_pubkey = Pubkey::from_str(&curve_state.creator)
            .map_err(|e| AppError::Internal(format!("Invalid creator address: {}", e)))?;

        let user_token_account = get_associated_token_address_with_program_id(&user_pubkey, &mint_pubkey, &token_2022_program);

        let (creator_vault, _) = Pubkey::find_program_address(
            &[b"creator-vault", creator_pubkey.as_ref()],
            &program_id,
        );

        let (global_volume_accumulator, _) = Pubkey::find_program_address(
            &[b"global_volume_accumulator"],
            &program_id,
        );

        let (user_volume_accumulator, _) = Pubkey::find_program_address(
            &[b"user_volume_accumulator", user_pubkey.as_ref()],
            &program_id,
        );

        let (fee_config, _) = Pubkey::find_program_address(
            &[b"fee_config", program_id.as_ref()],
            &fee_program,
        );

        let create_ata_ix = spl_associated_token_account::instruction::create_associated_token_account_idempotent(
            &user_pubkey,
            &user_pubkey,
            &mint_pubkey,
            &token_2022_program,
        );
        instructions.push(create_ata_ix);

        let buy_discriminator: [u8; 8] = [102, 6, 61, 18, 1, 218, 235, 234];

        let mut data = buy_discriminator.to_vec();
        let args = PumpFunBuyArgs {
            amount: min_tokens_out,
            max_sol_cost: sol_amount,
        };
        data.extend(borsh::to_vec(&args).unwrap());

        let buy_ix = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new_readonly(global_state, false),
                AccountMeta::new(fee_recipient, false),
                AccountMeta::new_readonly(mint_pubkey, false),
                AccountMeta::new(bonding_curve, false),
                AccountMeta::new(associated_bonding_curve, false),
                AccountMeta::new(user_token_account, false),
                AccountMeta::new(user_pubkey, true),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(token_2022_program, false),
                AccountMeta::new(creator_vault, false),
                AccountMeta::new_readonly(event_authority, false),
                AccountMeta::new_readonly(program_id, false),
                AccountMeta::new(global_volume_accumulator, false),
                AccountMeta::new(user_volume_accumulator, false),
                AccountMeta::new_readonly(fee_config, false),
                AccountMeta::new_readonly(fee_program, false),
            ],
            data,
        };

        instructions.push(buy_ix);

        Ok(instructions)
    }

    fn create_pump_fun_sell_instructions(
        &self,
        mint: &str,
        user_wallet: &str,
        curve_state: &OnChainCurveState,
        token_amount: u64,
        min_sol_out: u64,
    ) -> AppResult<Vec<Instruction>> {
        let mut instructions = Vec::new();

        instructions.push(ComputeBudgetInstruction::set_compute_unit_limit(
            self.compute_units,
        ));

        instructions.push(ComputeBudgetInstruction::set_compute_unit_price(
            self.priority_fee_micro_lamports,
        ));

        let program_id = Pubkey::from_str(PUMP_FUN_PROGRAM_ID)
            .map_err(|e| AppError::Internal(format!("Invalid program ID: {}", e)))?;
        let global_state = Pubkey::from_str(PUMP_FUN_GLOBAL_STATE)
            .map_err(|e| AppError::Internal(format!("Invalid global state: {}", e)))?;
        let fee_recipient = Pubkey::from_str(PUMP_FUN_FEE_RECIPIENT)
            .map_err(|e| AppError::Internal(format!("Invalid fee recipient: {}", e)))?;
        let event_authority = Pubkey::from_str(PUMP_FUN_EVENT_AUTHORITY)
            .map_err(|e| AppError::Internal(format!("Invalid event authority: {}", e)))?;
        let fee_program = Pubkey::from_str(PUMP_FUN_FEE_PROGRAM)
            .map_err(|e| AppError::Internal(format!("Invalid fee program: {}", e)))?;
        let token_2022_program = Pubkey::from_str(TOKEN_2022_PROGRAM_ID)
            .map_err(|e| AppError::Internal(format!("Invalid token-2022 program: {}", e)))?;
        let mint_pubkey = Pubkey::from_str(mint)
            .map_err(|e| AppError::Validation(format!("Invalid mint: {}", e)))?;
        let user_pubkey = Pubkey::from_str(user_wallet)
            .map_err(|e| AppError::Validation(format!("Invalid user wallet: {}", e)))?;
        let bonding_curve = Pubkey::from_str(&curve_state.bonding_curve_address)
            .map_err(|e| AppError::Internal(format!("Invalid bonding curve: {}", e)))?;
        let associated_bonding_curve = Pubkey::from_str(&curve_state.associated_bonding_curve)
            .map_err(|e| AppError::Internal(format!("Invalid associated bonding curve: {}", e)))?;
        let creator_pubkey = Pubkey::from_str(&curve_state.creator)
            .map_err(|e| AppError::Internal(format!("Invalid creator address: {}", e)))?;

        let user_token_account = get_associated_token_address_with_program_id(&user_pubkey, &mint_pubkey, &token_2022_program);

        let (creator_vault, _) = Pubkey::find_program_address(
            &[b"creator-vault", creator_pubkey.as_ref()],
            &program_id,
        );

        let (fee_config, _) = Pubkey::find_program_address(
            &[b"fee_config", program_id.as_ref()],
            &fee_program,
        );

        let sell_discriminator: [u8; 8] = [51, 230, 133, 164, 1, 127, 131, 173];

        let mut data = sell_discriminator.to_vec();
        let args = PumpFunSellArgs {
            amount: token_amount,
            min_sol_output: min_sol_out,
        };
        data.extend(borsh::to_vec(&args).unwrap());

        let sell_ix = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new_readonly(global_state, false),
                AccountMeta::new(fee_recipient, false),
                AccountMeta::new_readonly(mint_pubkey, false),
                AccountMeta::new(bonding_curve, false),
                AccountMeta::new(associated_bonding_curve, false),
                AccountMeta::new(user_token_account, false),
                AccountMeta::new(user_pubkey, true),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new(creator_vault, false),
                AccountMeta::new_readonly(token_2022_program, false),
                AccountMeta::new_readonly(event_authority, false),
                AccountMeta::new_readonly(program_id, false),
                AccountMeta::new_readonly(fee_config, false),
                AccountMeta::new_readonly(fee_program, false),
            ],
            data,
        };

        instructions.push(sell_ix);

        Ok(instructions)
    }

    pub async fn get_curve_state(&self, mint: &str) -> AppResult<OnChainCurveState> {
        self.on_chain_fetcher
            .get_pump_fun_bonding_curve(mint)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::venues::curves::math::sol_to_lamports;

    #[test]
    fn test_curve_buy_params() {
        let params = CurveBuyParams {
            mint: "TokenMint123".to_string(),
            sol_amount_lamports: sol_to_lamports(0.1),
            slippage_bps: 100,
            user_wallet: "UserWallet123".to_string(),
        };

        assert_eq!(params.sol_amount_lamports, 100_000_000);
        assert_eq!(params.slippage_bps, 100);
    }

    #[test]
    fn test_curve_sell_params() {
        let params = CurveSellParams {
            mint: "TokenMint123".to_string(),
            token_amount: 1_000_000_000_000,
            slippage_bps: 150,
            user_wallet: "UserWallet123".to_string(),
        };

        assert_eq!(params.token_amount, 1_000_000_000_000);
        assert_eq!(params.slippage_bps, 150);
    }
}
