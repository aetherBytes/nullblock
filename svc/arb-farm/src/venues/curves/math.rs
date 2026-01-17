use serde::{Deserialize, Serialize};

pub const PUMP_FUN_FEE_BPS: u16 = 100;
pub const MOONSHOT_FEE_BPS: u16 = 100;

pub const PUMP_FUN_PROGRAM_ID: &str = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";
pub const PUMP_FUN_GLOBAL_STATE: &str = "4wTV1YmiEkRvAtNtsSGPtUrqRYQMe5SKy2uB4Jjaxnjf";
pub const PUMP_FUN_FEE_RECIPIENT: &str = "CebN5WGQ4jvEPvsVU4EoHEpgzq1VV7AbicfhtW4xC9iM";
pub const PUMP_FUN_EVENT_AUTHORITY: &str = "Ce6TQqeHC9p8KetsN6JsjHK7UTZk7nasjjnr7XxXp9F1";
pub const PUMP_FUN_FEE_PROGRAM: &str = "pfeeUxB6jkeY1Hxd7CsFCAjcbHA9rWtchMGdZ6VojVZ";
pub const TOKEN_2022_PROGRAM_ID: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";

pub const MOONSHOT_PROGRAM_ID: &str = "MoonCVVNZFSYkqNXP6bxHLPL6QQJiMagDL3qcqUQTrG";

pub const PUMP_FUN_VIRTUAL_SOL_RESERVES: u64 = 30_000_000_000;
pub const PUMP_FUN_VIRTUAL_TOKEN_RESERVES: u64 = 1_073_000_000_000_000;
pub const PUMP_FUN_INITIAL_REAL_TOKEN_RESERVES: u64 = 793_100_000_000_000;
pub const PUMP_FUN_TOTAL_SUPPLY: u64 = 1_000_000_000_000_000;
pub const PUMP_FUN_GRADUATION_THRESHOLD_SOL: u64 = 85_000_000_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BondingCurveParams {
    pub virtual_sol_reserves: u64,
    pub virtual_token_reserves: u64,
    pub real_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub fee_bps: u16,
}

impl Default for BondingCurveParams {
    fn default() -> Self {
        Self::pump_fun_initial()
    }
}

impl BondingCurveParams {
    pub fn pump_fun_initial() -> Self {
        Self {
            virtual_sol_reserves: PUMP_FUN_VIRTUAL_SOL_RESERVES,
            virtual_token_reserves: PUMP_FUN_VIRTUAL_TOKEN_RESERVES,
            real_sol_reserves: 0,
            real_token_reserves: PUMP_FUN_INITIAL_REAL_TOKEN_RESERVES,
            fee_bps: PUMP_FUN_FEE_BPS,
        }
    }

    pub fn graduation_progress(&self) -> f64 {
        let threshold = PUMP_FUN_GRADUATION_THRESHOLD_SOL as f64;
        let current = self.real_sol_reserves as f64;
        (current / threshold).min(1.0) * 100.0
    }

    pub fn is_graduated(&self) -> bool {
        self.real_sol_reserves >= PUMP_FUN_GRADUATION_THRESHOLD_SOL
    }

    pub fn k(&self) -> u128 {
        (self.virtual_sol_reserves as u128) * (self.virtual_token_reserves as u128)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuyResult {
    pub tokens_out: u64,
    pub fee_lamports: u64,
    pub sol_spent: u64,
    pub price_per_token: f64,
    pub price_impact_percent: f64,
    pub new_virtual_sol: u64,
    pub new_virtual_token: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SellResult {
    pub sol_out: u64,
    pub fee_lamports: u64,
    pub tokens_sold: u64,
    pub price_per_token: f64,
    pub price_impact_percent: f64,
    pub new_virtual_sol: u64,
    pub new_virtual_token: u64,
}

pub trait BondingCurveMath {
    fn calculate_buy_amount(&self, sol_in_lamports: u64) -> BuyResult;
    fn calculate_sell_amount(&self, tokens_in: u64) -> SellResult;
    fn calculate_price_impact(&self, amount: u64, is_buy: bool) -> f64;
    fn get_current_price(&self) -> f64;
    fn get_market_cap_sol(&self) -> f64;
}

#[derive(Debug, Clone)]
pub struct PumpFunCurve {
    pub params: BondingCurveParams,
}

impl PumpFunCurve {
    pub fn new(params: BondingCurveParams) -> Self {
        Self { params }
    }

    pub fn with_current_state(
        virtual_sol: u64,
        virtual_token: u64,
        real_sol: u64,
        real_token: u64,
    ) -> Self {
        Self {
            params: BondingCurveParams {
                virtual_sol_reserves: virtual_sol,
                virtual_token_reserves: virtual_token,
                real_sol_reserves: real_sol,
                real_token_reserves: real_token,
                fee_bps: PUMP_FUN_FEE_BPS,
            },
        }
    }

    fn calculate_fee(&self, amount: u64) -> u64 {
        (amount as u128 * self.params.fee_bps as u128 / 10000) as u64
    }
}

impl BondingCurveMath for PumpFunCurve {
    fn calculate_buy_amount(&self, sol_in_lamports: u64) -> BuyResult {
        let fee = self.calculate_fee(sol_in_lamports);
        let sol_after_fee = sol_in_lamports - fee;

        let k = self.params.k();
        let new_virtual_sol = self.params.virtual_sol_reserves + sol_after_fee;
        let new_virtual_token = (k / new_virtual_sol as u128) as u64;

        let tokens_out = self.params.virtual_token_reserves.saturating_sub(new_virtual_token);
        let tokens_out = tokens_out.min(self.params.real_token_reserves);

        let price_before = self.get_current_price();
        let price_after = new_virtual_sol as f64 / new_virtual_token as f64;
        let price_impact = ((price_after - price_before) / price_before * 100.0).abs();

        BuyResult {
            tokens_out,
            fee_lamports: fee,
            sol_spent: sol_in_lamports,
            price_per_token: sol_after_fee as f64 / tokens_out as f64,
            price_impact_percent: price_impact,
            new_virtual_sol,
            new_virtual_token,
        }
    }

    fn calculate_sell_amount(&self, tokens_in: u64) -> SellResult {
        let tokens_to_sell = tokens_in.min(self.params.real_token_reserves);

        let k = self.params.k();
        let new_virtual_token = self.params.virtual_token_reserves + tokens_to_sell;
        let new_virtual_sol = (k / new_virtual_token as u128) as u64;

        let sol_out_before_fee = self.params.virtual_sol_reserves.saturating_sub(new_virtual_sol);
        let fee = self.calculate_fee(sol_out_before_fee);
        let sol_out = sol_out_before_fee - fee;

        let price_before = self.get_current_price();
        let price_after = new_virtual_sol as f64 / new_virtual_token as f64;
        let price_impact = ((price_before - price_after) / price_before * 100.0).abs();

        SellResult {
            sol_out,
            fee_lamports: fee,
            tokens_sold: tokens_to_sell,
            price_per_token: sol_out as f64 / tokens_to_sell as f64,
            price_impact_percent: price_impact,
            new_virtual_sol,
            new_virtual_token,
        }
    }

    fn calculate_price_impact(&self, amount: u64, is_buy: bool) -> f64 {
        if is_buy {
            self.calculate_buy_amount(amount).price_impact_percent
        } else {
            self.calculate_sell_amount(amount).price_impact_percent
        }
    }

    fn get_current_price(&self) -> f64 {
        self.params.virtual_sol_reserves as f64 / self.params.virtual_token_reserves as f64
    }

    fn get_market_cap_sol(&self) -> f64 {
        let price = self.get_current_price();
        price * PUMP_FUN_TOTAL_SUPPLY as f64
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MoonshotCurveType {
    Linear,
    Exponential,
    Sigmoid,
}

impl Default for MoonshotCurveType {
    fn default() -> Self {
        Self::Linear
    }
}

impl std::str::FromStr for MoonshotCurveType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "linear" => Ok(Self::Linear),
            "exponential" => Ok(Self::Exponential),
            "sigmoid" => Ok(Self::Sigmoid),
            _ => Err(format!("Unknown curve type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoonshotCurveParams {
    pub base_params: BondingCurveParams,
    pub curve_type: MoonshotCurveType,
    pub graduation_threshold_usd: f64,
    pub sol_price_usd: f64,
}

impl Default for MoonshotCurveParams {
    fn default() -> Self {
        Self {
            base_params: BondingCurveParams {
                virtual_sol_reserves: 30_000_000_000,
                virtual_token_reserves: 1_000_000_000_000_000,
                real_sol_reserves: 0,
                real_token_reserves: 800_000_000_000_000,
                fee_bps: MOONSHOT_FEE_BPS,
            },
            curve_type: MoonshotCurveType::Linear,
            graduation_threshold_usd: 500_000.0,
            sol_price_usd: 100.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MoonshotCurve {
    pub params: MoonshotCurveParams,
}

impl MoonshotCurve {
    pub fn new(params: MoonshotCurveParams) -> Self {
        Self { params }
    }

    fn curve_multiplier(&self, progress: f64) -> f64 {
        match self.params.curve_type {
            MoonshotCurveType::Linear => 1.0,
            MoonshotCurveType::Exponential => (1.0 + progress).powf(1.5),
            MoonshotCurveType::Sigmoid => {
                let x = (progress - 0.5) * 10.0;
                1.0 + 1.0 / (1.0 + (-x).exp())
            }
        }
    }

    fn calculate_fee(&self, amount: u64) -> u64 {
        (amount as u128 * self.params.base_params.fee_bps as u128 / 10000) as u64
    }

    pub fn graduation_progress(&self) -> f64 {
        let market_cap_sol = self.get_market_cap_sol();
        let market_cap_usd = market_cap_sol * self.params.sol_price_usd;
        (market_cap_usd / self.params.graduation_threshold_usd).min(1.0) * 100.0
    }
}

impl BondingCurveMath for MoonshotCurve {
    fn calculate_buy_amount(&self, sol_in_lamports: u64) -> BuyResult {
        let fee = self.calculate_fee(sol_in_lamports);
        let sol_after_fee = sol_in_lamports - fee;

        let progress = self.graduation_progress() / 100.0;
        let multiplier = self.curve_multiplier(progress);

        let k = self.params.base_params.k();
        let adjusted_sol = (sol_after_fee as f64 / multiplier) as u64;
        let new_virtual_sol = self.params.base_params.virtual_sol_reserves + adjusted_sol;
        let new_virtual_token = (k / new_virtual_sol as u128) as u64;

        let tokens_out = self
            .params
            .base_params
            .virtual_token_reserves
            .saturating_sub(new_virtual_token);
        let tokens_out = tokens_out.min(self.params.base_params.real_token_reserves);

        let price_before = self.get_current_price();
        let price_after = new_virtual_sol as f64 / new_virtual_token as f64 * multiplier;
        let price_impact = ((price_after - price_before) / price_before * 100.0).abs();

        BuyResult {
            tokens_out,
            fee_lamports: fee,
            sol_spent: sol_in_lamports,
            price_per_token: sol_after_fee as f64 / tokens_out.max(1) as f64,
            price_impact_percent: price_impact,
            new_virtual_sol,
            new_virtual_token,
        }
    }

    fn calculate_sell_amount(&self, tokens_in: u64) -> SellResult {
        let tokens_to_sell = tokens_in.min(self.params.base_params.real_token_reserves);

        let progress = self.graduation_progress() / 100.0;
        let multiplier = self.curve_multiplier(progress);

        let k = self.params.base_params.k();
        let new_virtual_token = self.params.base_params.virtual_token_reserves + tokens_to_sell;
        let new_virtual_sol = (k / new_virtual_token as u128) as u64;

        let sol_out_before_fee = self
            .params
            .base_params
            .virtual_sol_reserves
            .saturating_sub(new_virtual_sol);
        let adjusted_sol_out = (sol_out_before_fee as f64 * multiplier) as u64;
        let fee = self.calculate_fee(adjusted_sol_out);
        let sol_out = adjusted_sol_out - fee;

        let price_before = self.get_current_price();
        let price_after = new_virtual_sol as f64 / new_virtual_token as f64 * multiplier;
        let price_impact = ((price_before - price_after) / price_before * 100.0).abs();

        SellResult {
            sol_out,
            fee_lamports: fee,
            tokens_sold: tokens_to_sell,
            price_per_token: sol_out as f64 / tokens_to_sell.max(1) as f64,
            price_impact_percent: price_impact,
            new_virtual_sol,
            new_virtual_token,
        }
    }

    fn calculate_price_impact(&self, amount: u64, is_buy: bool) -> f64 {
        if is_buy {
            self.calculate_buy_amount(amount).price_impact_percent
        } else {
            self.calculate_sell_amount(amount).price_impact_percent
        }
    }

    fn get_current_price(&self) -> f64 {
        let base_price = self.params.base_params.virtual_sol_reserves as f64
            / self.params.base_params.virtual_token_reserves as f64;
        let progress = self.graduation_progress() / 100.0;
        base_price * self.curve_multiplier(progress)
    }

    fn get_market_cap_sol(&self) -> f64 {
        let price = self.get_current_price();
        price * 1_000_000_000_000_000.0
    }
}

pub fn calculate_min_tokens_out(tokens_out: u64, slippage_bps: u16) -> u64 {
    let slippage_factor = 10000u64 - slippage_bps as u64;
    (tokens_out as u128 * slippage_factor as u128 / 10000) as u64
}

pub fn calculate_min_sol_out(sol_out: u64, slippage_bps: u16) -> u64 {
    let slippage_factor = 10000u64 - slippage_bps as u64;
    (sol_out as u128 * slippage_factor as u128 / 10000) as u64
}

pub fn lamports_to_sol(lamports: u64) -> f64 {
    lamports as f64 / 1_000_000_000.0
}

pub fn sol_to_lamports(sol: f64) -> u64 {
    (sol * 1_000_000_000.0) as u64
}

pub fn tokens_to_ui(tokens: u64, decimals: u8) -> f64 {
    tokens as f64 / 10f64.powi(decimals as i32)
}

pub fn ui_to_tokens(amount: f64, decimals: u8) -> u64 {
    (amount * 10f64.powi(decimals as i32)) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pump_fun_buy_calculation() {
        let curve = PumpFunCurve::new(BondingCurveParams::pump_fun_initial());

        let result = curve.calculate_buy_amount(sol_to_lamports(0.1));

        assert!(result.tokens_out > 0);
        assert!(result.fee_lamports > 0);
        assert!(result.price_impact_percent < 10.0);
    }

    #[test]
    fn test_pump_fun_sell_calculation() {
        let curve = PumpFunCurve::new(BondingCurveParams::pump_fun_initial());

        let buy_result = curve.calculate_buy_amount(sol_to_lamports(1.0));
        let tokens = buy_result.tokens_out;

        let mut sell_params = BondingCurveParams::pump_fun_initial();
        sell_params.virtual_sol_reserves = buy_result.new_virtual_sol;
        sell_params.virtual_token_reserves = buy_result.new_virtual_token;
        sell_params.real_token_reserves -= tokens;

        let sell_curve = PumpFunCurve::new(sell_params);
        let sell_result = sell_curve.calculate_sell_amount(tokens);

        assert!(sell_result.sol_out > 0);
        assert!(sell_result.sol_out < sol_to_lamports(1.0));
    }

    #[test]
    fn test_slippage_calculation() {
        let tokens_out = 1_000_000_000_000u64;
        let min_tokens = calculate_min_tokens_out(tokens_out, 100);

        assert_eq!(min_tokens, 990_000_000_000);
    }

    #[test]
    fn test_graduation_progress() {
        let mut params = BondingCurveParams::pump_fun_initial();
        params.real_sol_reserves = PUMP_FUN_GRADUATION_THRESHOLD_SOL / 2;

        assert!((params.graduation_progress() - 50.0).abs() < 0.1);
        assert!(!params.is_graduated());

        params.real_sol_reserves = PUMP_FUN_GRADUATION_THRESHOLD_SOL;
        assert!(params.is_graduated());
    }

    #[test]
    fn test_moonshot_curve_types() {
        let linear_params = MoonshotCurveParams {
            curve_type: MoonshotCurveType::Linear,
            ..Default::default()
        };
        let exp_params = MoonshotCurveParams {
            curve_type: MoonshotCurveType::Exponential,
            ..Default::default()
        };

        let linear_curve = MoonshotCurve::new(linear_params);
        let exp_curve = MoonshotCurve::new(exp_params);

        let linear_result = linear_curve.calculate_buy_amount(sol_to_lamports(1.0));
        let exp_result = exp_curve.calculate_buy_amount(sol_to_lamports(1.0));

        assert!(exp_result.tokens_out != linear_result.tokens_out);
    }
}
