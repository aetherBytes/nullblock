use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Deserializer, Serialize};
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
const DEFAULT_PRIORITY_FEE_MICRO_LAMPORTS: u64 = 10_000_000; // 10x increase for reliable exits (~0.002 SOL per tx)
const SOL_MINT: &str = "So11111111111111111111111111111111111111112";
const RAYDIUM_API_URL: &str = "https://transaction-v1.raydium.io";
const HIGH_SLIPPAGE_WARNING_BPS: u16 = 1500; // 15% - warn above this
const EXTREME_SLIPPAGE_WARNING_BPS: u16 = 3000; // 30% - warn strongly above this

fn deserialize_string_or_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::{self, Visitor};

    struct StringOrF64Visitor;

    impl<'de> Visitor<'de> for StringOrF64Visitor {
        type Value = Option<f64>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string or f64 representing a number")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_any(StringOrF64InnerVisitor).map(Some)
        }

        fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(value))
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(value as f64))
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(value as f64))
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            value.parse::<f64>().map(Some).map_err(de::Error::custom)
        }
    }

    struct StringOrF64InnerVisitor;

    impl<'de> Visitor<'de> for StringOrF64InnerVisitor {
        type Value = f64;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string or f64 representing a number")
        }

        fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value)
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value as f64)
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value as f64)
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            value.parse::<f64>().map_err(de::Error::custom)
        }
    }

    deserializer.deserialize_option(StringOrF64Visitor)
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostGraduationSellResult {
    pub transaction_base64: String,
    pub expected_sol_out: u64,
    pub price_impact_percent: f64,
    pub route_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostGraduationBuyResult {
    pub transaction_base64: String,
    pub expected_tokens_out: u64,
    pub price_impact_percent: f64,
    pub route_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JupiterQuoteResponse {
    pub input_mint: String,
    pub output_mint: String,
    pub in_amount: String,
    pub out_amount: String,
    pub other_amount_threshold: String,
    pub swap_mode: String,
    pub slippage_bps: u16,
    #[serde(default, deserialize_with = "deserialize_string_or_f64")]
    pub price_impact_pct: Option<f64>,
    pub route_plan: Vec<JupiterRoutePlan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JupiterRoutePlan {
    pub swap_info: JupiterSwapInfo,
    pub percent: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JupiterSwapInfo {
    pub amm_key: String,
    pub label: Option<String>,
    pub input_mint: String,
    pub output_mint: String,
    pub in_amount: String,
    pub out_amount: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JupiterSwapResponse {
    pub swap_transaction: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JupiterErrorResponse {
    pub error: String,
    #[serde(default)]
    pub error_code: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RaydiumApiResponse {
    #[serde(default)]
    pub success: Option<bool>,
    #[serde(default)]
    pub msg: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RaydiumQuoteResponse {
    pub input_mint: String,
    pub output_mint: String,
    pub in_amount: String,
    pub out_amount: String,
    #[serde(default, deserialize_with = "deserialize_string_or_f64")]
    pub price_impact_pct: Option<f64>,
    #[serde(default)]
    pub route_plan: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RaydiumSwapRequest {
    pub swap_response: RaydiumQuoteResponse,
    pub wallet: String,
    pub tx_version: String,
    pub wrap_sol: bool,
    pub unwrap_sol: bool,
    pub compute_unit_price_micro_lamports: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RaydiumSwapResponse {
    #[serde(alias = "data")]
    pub transaction: Option<String>,
    #[serde(default)]
    pub success: Option<bool>,
    #[serde(default)]
    pub msg: Option<String>,
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

    pub async fn get_wallet_balance(&self, wallet_address: &str) -> AppResult<u64> {
        let client = reqwest::Client::new();
        let response = client
            .post(&self.rpc_url)
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "getBalance",
                "params": [wallet_address]
            }))
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch balance: {}", e)))?;

        #[derive(Deserialize)]
        struct BalanceResponse {
            result: BalanceResult,
        }
        #[derive(Deserialize)]
        struct BalanceResult {
            value: u64,
        }

        let data: BalanceResponse = response
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse balance response: {}", e)))?;

        Ok(data.result.value)
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
        // Validate non-zero SOL amount to prevent wasting network fees
        if params.sol_amount_lamports == 0 {
            return Err(AppError::Validation(
                "Cannot buy with zero SOL amount".to_string(),
            ));
        }

        // Minimum 0.001 SOL to prevent dust transactions
        const MIN_SOL_LAMPORTS: u64 = 1_000_000; // 0.001 SOL
        if params.sol_amount_lamports < MIN_SOL_LAMPORTS {
            return Err(AppError::Validation(
                format!(
                    "SOL amount {} lamports below minimum {} (0.001 SOL)",
                    params.sol_amount_lamports, MIN_SOL_LAMPORTS
                ),
            ));
        }

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

        // Warn about high slippage settings
        if params.slippage_bps >= EXTREME_SLIPPAGE_WARNING_BPS {
            tracing::warn!(
                "⚠️ EXTREME SLIPPAGE on curve sell: {}bps ({}%) for {} - consider reducing",
                params.slippage_bps,
                params.slippage_bps as f64 / 100.0,
                &params.mint[..12.min(params.mint.len())]
            );
        } else if params.slippage_bps >= HIGH_SLIPPAGE_WARNING_BPS {
            tracing::warn!(
                "⚠️ High slippage on curve sell: {}bps ({}%) for {}",
                params.slippage_bps,
                params.slippage_bps as f64 / 100.0,
                &params.mint[..12.min(params.mint.len())]
            );
        }

        // Detect which token program this mint uses (Token-2022 or standard SPL Token)
        let is_token_2022 = self.on_chain_fetcher.is_token_2022(&params.mint).await?;
        let token_program = if is_token_2022 {
            Pubkey::from_str(TOKEN_2022_PROGRAM_ID)
                .map_err(|e| AppError::Internal(format!("Invalid token-2022 program: {}", e)))?
        } else {
            spl_token::id()
        };
        tracing::info!(
            mint = &params.mint[..12.min(params.mint.len())],
            is_token_2022 = is_token_2022,
            "Detected token program for sell"
        );

        let instructions = self.create_pump_fun_sell_instructions(
            &params.mint,
            &params.user_wallet,
            &curve_state,
            params.token_amount,
            min_sol,
            token_program,
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

        // Validate creator address - required for pump.fun trades
        if curve_state.creator.is_empty() || curve_state.creator == Pubkey::default().to_string() {
            return Err(AppError::Internal(
                "Cannot trade: bonding curve has no valid creator address. The curve data may be malformed.".to_string()
            ));
        }

        let creator_pubkey = Pubkey::from_str(&curve_state.creator)
            .map_err(|e| AppError::Internal(format!("Invalid creator address '{}': {}", curve_state.creator, e)))?;

        tracing::debug!(
            mint = %mint,
            creator = %curve_state.creator,
            "Building pump.fun buy with creator"
        );

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
        data.extend(borsh::to_vec(&args)
            .map_err(|e| AppError::Internal(format!("Failed to serialize buy args: {}", e)))?);

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
        token_program: Pubkey,
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
        let mint_pubkey = Pubkey::from_str(mint)
            .map_err(|e| AppError::Validation(format!("Invalid mint: {}", e)))?;
        let user_pubkey = Pubkey::from_str(user_wallet)
            .map_err(|e| AppError::Validation(format!("Invalid user wallet: {}", e)))?;
        let bonding_curve = Pubkey::from_str(&curve_state.bonding_curve_address)
            .map_err(|e| AppError::Internal(format!("Invalid bonding curve: {}", e)))?;
        let associated_bonding_curve = Pubkey::from_str(&curve_state.associated_bonding_curve)
            .map_err(|e| AppError::Internal(format!("Invalid associated bonding curve: {}", e)))?;

        // Validate creator address - if empty or invalid, we can't build the sell instruction
        if curve_state.creator.is_empty() || curve_state.creator == Pubkey::default().to_string() {
            return Err(AppError::Internal(
                "Cannot sell: bonding curve has no valid creator address. The curve data may be malformed.".to_string()
            ));
        }

        let creator_pubkey = Pubkey::from_str(&curve_state.creator)
            .map_err(|e| AppError::Internal(format!("Invalid creator address '{}': {}", curve_state.creator, e)))?;

        // Log the creator for debugging
        tracing::debug!(
            mint = %mint,
            creator = %curve_state.creator,
            "Building pump.fun sell with creator"
        );

        let user_token_account = get_associated_token_address_with_program_id(&user_pubkey, &mint_pubkey, &token_program);

        let (creator_vault, _) = Pubkey::find_program_address(
            &[b"creator-vault", creator_pubkey.as_ref()],
            &program_id,
        );

        tracing::debug!(
            creator_vault = %creator_vault,
            "Derived creator_vault PDA"
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
        data.extend(borsh::to_vec(&args)
            .map_err(|e| AppError::Internal(format!("Failed to serialize sell args: {}", e)))?);

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
                AccountMeta::new_readonly(token_program, false),
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

    pub async fn build_post_graduation_sell(
        &self,
        params: &CurveSellParams,
        jupiter_api_url: &str,
    ) -> AppResult<PostGraduationSellResult> {
        // Warn about high slippage settings
        if params.slippage_bps >= EXTREME_SLIPPAGE_WARNING_BPS {
            tracing::warn!(
                "⚠️ EXTREME SLIPPAGE on Jupiter sell: {}bps ({}%) for {} - consider reducing",
                params.slippage_bps,
                params.slippage_bps as f64 / 100.0,
                &params.mint[..12.min(params.mint.len())]
            );
        } else if params.slippage_bps >= HIGH_SLIPPAGE_WARNING_BPS {
            tracing::warn!(
                "⚠️ High slippage on Jupiter sell: {}bps ({}%) for {}",
                params.slippage_bps,
                params.slippage_bps as f64 / 100.0,
                &params.mint[..12.min(params.mint.len())]
            );
        }

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| AppError::Internal(format!("Failed to create HTTP client: {}", e)))?;

        let quote_url = format!(
            "{}/quote?inputMint={}&outputMint={}&amount={}&slippageBps={}&onlyDirectRoutes=false",
            jupiter_api_url,
            params.mint,
            SOL_MINT,
            params.token_amount,
            params.slippage_bps
        );

        let quote_response = client
            .get(&quote_url)
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Jupiter quote request failed: {}", e)))?;

        if !quote_response.status().is_success() {
            let error_text = quote_response.text().await.unwrap_or_default();
            return Err(AppError::ExternalApi(format!(
                "Jupiter quote error for graduated token: {}",
                error_text
            )));
        }

        let quote: JupiterQuoteResponse = quote_response
            .json()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to parse Jupiter quote: {}", e)))?;

        let swap_url = format!("{}/swap", jupiter_api_url);
        let swap_request = serde_json::json!({
            "userPublicKey": params.user_wallet,
            "quoteResponse": quote,
            "wrapAndUnwrapSol": true,
            "useSharedAccounts": false,
            "computeUnitPriceMicroLamports": self.priority_fee_micro_lamports,
            "priorityLevelWithMaxLamports": {
                "maxLamports": 5_000_000u64
            },
            "asLegacyTransaction": false,
            "dynamicComputeUnitLimit": true,
            "skipUserAccountsRpcCalls": false
        });

        let swap_response = client
            .post(&swap_url)
            .json(&swap_request)
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Jupiter swap request failed: {}", e)))?;

        if !swap_response.status().is_success() {
            let error_text = swap_response.text().await.unwrap_or_default();
            return Err(AppError::ExternalApi(format!(
                "Jupiter swap error for graduated token: {}",
                error_text
            )));
        }

        let swap_result: JupiterSwapResponse = swap_response
            .json()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to parse Jupiter swap: {}", e)))?;

        // CRITICAL: Fail if parsing fails to prevent zero-output trades
        let expected_sol_out: u64 = quote.out_amount.parse().map_err(|e| {
            AppError::ExternalApi(format!(
                "Failed to parse Jupiter quote out_amount '{}': {}",
                quote.out_amount, e
            ))
        })?;

        // Validate we're getting a reasonable output amount
        if expected_sol_out == 0 {
            return Err(AppError::ExternalApi(
                "Jupiter quote returned zero SOL output - quote invalid".to_string()
            ));
        }

        let price_impact: f64 = quote.price_impact_pct.unwrap_or(0.0);

        // Enforce slippage limit - reject if actual price impact exceeds requested slippage
        let max_allowed_impact = params.slippage_bps as f64 / 100.0; // Convert bps to percent
        if price_impact.abs() > max_allowed_impact {
            return Err(AppError::ExternalApi(format!(
                "Jupiter sell quote price impact {:.2}% exceeds max allowed {:.2}% ({}bps) - rejecting trade",
                price_impact,
                max_allowed_impact,
                params.slippage_bps
            )));
        }

        Ok(PostGraduationSellResult {
            transaction_base64: swap_result.swap_transaction,
            expected_sol_out,
            price_impact_percent: price_impact,
            route_label: quote.route_plan.first()
                .and_then(|r| r.swap_info.label.clone())
                .unwrap_or_else(|| "Jupiter".to_string()),
        })
    }

    pub async fn build_raydium_sell(
        &self,
        params: &CurveSellParams,
    ) -> AppResult<PostGraduationSellResult> {
        // Warn about high slippage settings
        if params.slippage_bps >= EXTREME_SLIPPAGE_WARNING_BPS {
            tracing::warn!(
                "⚠️ EXTREME SLIPPAGE on Raydium sell: {}bps ({}%) for {} - consider reducing",
                params.slippage_bps,
                params.slippage_bps as f64 / 100.0,
                &params.mint[..12.min(params.mint.len())]
            );
        } else if params.slippage_bps >= HIGH_SLIPPAGE_WARNING_BPS {
            tracing::warn!(
                "⚠️ High slippage on Raydium sell: {}bps ({}%) for {}",
                params.slippage_bps,
                params.slippage_bps as f64 / 100.0,
                &params.mint[..12.min(params.mint.len())]
            );
        }

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|e| AppError::Internal(format!("Failed to create HTTP client: {}", e)))?;

        let quote_url = format!(
            "{}/compute/swap-base-in?inputMint={}&outputMint={}&amount={}&slippageBps={}&txVersion=V0",
            RAYDIUM_API_URL,
            params.mint,
            SOL_MINT,
            params.token_amount,
            params.slippage_bps
        );

        tracing::debug!("Raydium quote URL: {}", quote_url);

        let quote_response = client
            .get(&quote_url)
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Raydium quote request failed: {}", e)))?;

        if !quote_response.status().is_success() {
            let error_text = quote_response.text().await.unwrap_or_default();
            return Err(AppError::ExternalApi(format!(
                "Raydium quote error: {}",
                error_text
            )));
        }

        let quote_text = quote_response.text().await
            .map_err(|e| AppError::ExternalApi(format!("Failed to read Raydium quote: {}", e)))?;

        // First check if Raydium returned an error (success: false)
        if let Ok(api_response) = serde_json::from_str::<RaydiumApiResponse>(&quote_text) {
            if api_response.success == Some(false) {
                let error_msg = api_response.msg.unwrap_or_else(|| "Unknown error".to_string());
                return Err(AppError::ExternalApi(format!(
                    "Raydium quote failed: {}",
                    error_msg
                )));
            }
        }

        let quote: RaydiumQuoteResponse = serde_json::from_str(&quote_text)
            .map_err(|e| AppError::ExternalApi(format!(
                "Failed to parse Raydium quote (response: {}): {}",
                &quote_text[..200.min(quote_text.len())],
                e
            )))?;

        let swap_url = format!("{}/transaction/swap-base-in", RAYDIUM_API_URL);
        let swap_request = RaydiumSwapRequest {
            swap_response: quote.clone(),
            wallet: params.user_wallet.clone(),
            tx_version: "V0".to_string(),
            wrap_sol: true,
            unwrap_sol: true,
            compute_unit_price_micro_lamports: self.priority_fee_micro_lamports,
        };

        let swap_response = client
            .post(&swap_url)
            .json(&swap_request)
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Raydium swap request failed: {}", e)))?;

        if !swap_response.status().is_success() {
            let error_text = swap_response.text().await.unwrap_or_default();
            return Err(AppError::ExternalApi(format!(
                "Raydium swap error: {}",
                error_text
            )));
        }

        let swap_result: RaydiumSwapResponse = swap_response
            .json()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to parse Raydium swap: {}", e)))?;

        let transaction = swap_result.transaction
            .or_else(|| swap_result.msg.clone())
            .ok_or_else(|| AppError::ExternalApi(
                "Raydium returned no transaction data".to_string()
            ))?;

        // CRITICAL: Fail if parsing fails to prevent zero-output trades
        let expected_sol_out: u64 = quote.out_amount.parse().map_err(|e| {
            AppError::ExternalApi(format!(
                "Failed to parse Raydium quote out_amount '{}': {}",
                quote.out_amount, e
            ))
        })?;

        // Validate we're getting a reasonable output amount
        if expected_sol_out == 0 {
            return Err(AppError::ExternalApi(
                "Raydium quote returned zero SOL output - quote invalid".to_string()
            ));
        }

        let price_impact: f64 = quote.price_impact_pct.unwrap_or(0.0);

        // Enforce slippage limit - reject if actual price impact exceeds requested slippage
        let max_allowed_impact = params.slippage_bps as f64 / 100.0; // Convert bps to percent
        if price_impact.abs() > max_allowed_impact {
            return Err(AppError::ExternalApi(format!(
                "Raydium sell quote price impact {:.2}% exceeds max allowed {:.2}% ({}bps) - rejecting trade",
                price_impact,
                max_allowed_impact,
                params.slippage_bps
            )));
        }

        Ok(PostGraduationSellResult {
            transaction_base64: transaction,
            expected_sol_out,
            price_impact_percent: price_impact,
            route_label: "Raydium".to_string(),
        })
    }

    /// Build a Jupiter swap transaction for buying a token after graduation (SOL -> Token)
    pub async fn build_post_graduation_buy(
        &self,
        mint: &str,
        sol_amount_lamports: u64,
        slippage_bps: u16,
        user_wallet: &str,
        jupiter_api_url: &str,
    ) -> AppResult<PostGraduationBuyResult> {
        // Validate non-zero SOL amount to prevent wasting network fees
        if sol_amount_lamports == 0 {
            return Err(AppError::Validation(
                "Cannot buy with zero SOL amount".to_string(),
            ));
        }

        // Minimum 0.001 SOL to prevent dust transactions
        const MIN_SOL_LAMPORTS: u64 = 1_000_000; // 0.001 SOL
        if sol_amount_lamports < MIN_SOL_LAMPORTS {
            return Err(AppError::Validation(
                format!(
                    "SOL amount {} lamports below minimum {} (0.001 SOL)",
                    sol_amount_lamports, MIN_SOL_LAMPORTS
                ),
            ));
        }

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| AppError::Internal(format!("Failed to create HTTP client: {}", e)))?;

        // Quote: SOL -> Token
        let quote_url = format!(
            "{}/quote?inputMint={}&outputMint={}&amount={}&slippageBps={}&onlyDirectRoutes=false",
            jupiter_api_url,
            SOL_MINT,
            mint,
            sol_amount_lamports,
            slippage_bps
        );

        let quote_response = client
            .get(&quote_url)
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Jupiter quote request failed: {}", e)))?;

        if !quote_response.status().is_success() {
            let error_text = quote_response.text().await.unwrap_or_default();
            return Err(AppError::ExternalApi(format!(
                "Jupiter quote error for post-graduation buy: {}",
                error_text
            )));
        }

        // Get the response text first so we can try parsing as error or success
        let response_text = quote_response.text().await
            .map_err(|e| AppError::ExternalApi(format!("Failed to read Jupiter quote response: {}", e)))?;

        // First try to parse as an error response (Jupiter returns 200 with error JSON for some errors)
        if let Ok(error_response) = serde_json::from_str::<JupiterErrorResponse>(&response_text) {
            let error_code = error_response.error_code.unwrap_or_else(|| "UNKNOWN".to_string());
            return Err(AppError::ExternalApi(format!(
                "Jupiter error ({}): {}",
                error_code,
                error_response.error
            )));
        }

        // Parse as successful quote response
        let quote: JupiterQuoteResponse = serde_json::from_str(&response_text)
            .map_err(|e| AppError::ExternalApi(format!(
                "Failed to parse Jupiter quote (response: {}): {}",
                &response_text[..200.min(response_text.len())],
                e
            )))?;

        let swap_url = format!("{}/swap", jupiter_api_url);
        let swap_request = serde_json::json!({
            "userPublicKey": user_wallet,
            "quoteResponse": quote,
            "wrapAndUnwrapSol": true,
            "useSharedAccounts": false,
            "computeUnitPriceMicroLamports": self.priority_fee_micro_lamports,
            "priorityLevelWithMaxLamports": {
                "maxLamports": 5_000_000u64
            },
            "asLegacyTransaction": false,
            "dynamicComputeUnitLimit": true,
            "skipUserAccountsRpcCalls": false
        });

        let swap_response = client
            .post(&swap_url)
            .json(&swap_request)
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Jupiter swap request failed: {}", e)))?;

        if !swap_response.status().is_success() {
            let error_text = swap_response.text().await.unwrap_or_default();
            return Err(AppError::ExternalApi(format!(
                "Jupiter swap error for post-graduation buy: {}",
                error_text
            )));
        }

        let swap_result: JupiterSwapResponse = swap_response
            .json()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to parse Jupiter swap: {}", e)))?;

        // CRITICAL: Fail if parsing fails to prevent zero-output trades
        let expected_tokens_out: u64 = quote.out_amount.parse().map_err(|e| {
            AppError::ExternalApi(format!(
                "Failed to parse Jupiter quote out_amount '{}': {}",
                quote.out_amount, e
            ))
        })?;

        // Validate we're getting a reasonable output amount
        if expected_tokens_out == 0 {
            return Err(AppError::ExternalApi(
                "Jupiter quote returned zero token output - quote invalid".to_string()
            ));
        }

        let price_impact: f64 = quote.price_impact_pct.unwrap_or(0.0);

        Ok(PostGraduationBuyResult {
            transaction_base64: swap_result.swap_transaction,
            expected_tokens_out,
            price_impact_percent: price_impact,
            route_label: quote.route_plan.first()
                .and_then(|r| r.swap_info.label.clone())
                .unwrap_or_else(|| "Jupiter".to_string()),
        })
    }

    /// Get actual on-chain token balance for a wallet
    pub async fn get_actual_token_balance(&self, owner: &str, mint: &str) -> AppResult<u64> {
        self.on_chain_fetcher
            .get_token_balance(owner, mint)
            .await
    }

    /// Check if a mint uses Token-2022 or standard SPL Token
    pub async fn is_token_2022(&self, mint: &str) -> AppResult<bool> {
        self.on_chain_fetcher.is_token_2022(mint).await
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
