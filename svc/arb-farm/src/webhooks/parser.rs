use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::helius::{EnhancedTransactionEvent, SwapEvent, TokenAmount};

pub struct TransactionParser;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedTransaction {
    pub signature: String,
    pub fee_payer: String,
    pub fee: u64,
    pub slot: u64,
    pub timestamp: i64,
    pub source: String,
    pub transaction_type: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedSwap {
    pub id: Uuid,
    pub signature: String,
    pub wallet_address: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub input_mint: String,
    pub input_amount: u64,
    pub input_decimals: u8,
    pub output_mint: String,
    pub output_amount: u64,
    pub output_decimals: u8,
    pub dex_source: String,
    pub fee_lamports: u64,
    pub is_native_input: bool,
    pub is_native_output: bool,
}

impl TransactionParser {
    pub fn parse_enhanced_transaction(event: &EnhancedTransactionEvent) -> EnhancedTransaction {
        EnhancedTransaction {
            signature: event.signature.clone(),
            fee_payer: event.fee_payer.clone(),
            fee: event.fee,
            slot: event.slot,
            timestamp: event.timestamp,
            source: event.source.clone(),
            transaction_type: event.transaction_type.clone(),
            description: event.description.clone(),
        }
    }

    pub fn parse_swap(event: &EnhancedTransactionEvent) -> Option<ParsedSwap> {
        let swap_event = event.events.swap.as_ref()?;

        let (input_mint, input_amount, input_decimals, is_native_input) =
            Self::extract_input(swap_event)?;
        let (output_mint, output_amount, output_decimals, is_native_output) =
            Self::extract_output(swap_event)?;

        let dex_source = Self::extract_dex_source(swap_event);

        let timestamp =
            chrono::DateTime::from_timestamp(event.timestamp, 0).unwrap_or_else(chrono::Utc::now);

        Some(ParsedSwap {
            id: Uuid::new_v4(),
            signature: event.signature.clone(),
            wallet_address: event.fee_payer.clone(),
            timestamp,
            input_mint,
            input_amount,
            input_decimals,
            output_mint,
            output_amount,
            output_decimals,
            dex_source,
            fee_lamports: event.fee,
            is_native_input,
            is_native_output,
        })
    }

    fn extract_input(swap: &SwapEvent) -> Option<(String, u64, u8, bool)> {
        // Check for native SOL input first
        if let Some(native_input) = &swap.native_input {
            let amount: u64 = native_input.amount.parse().unwrap_or(0);
            if amount > 0 {
                return Some((
                    "So11111111111111111111111111111111111111112".to_string(),
                    amount,
                    9,
                    true,
                ));
            }
        }

        // Check token inputs
        if let Some(token_input) = swap.token_inputs.first() {
            let amount: u64 = token_input
                .raw_token_amount
                .token_amount
                .parse()
                .unwrap_or(0);
            return Some((
                token_input.mint.clone(),
                amount,
                token_input.raw_token_amount.decimals,
                false,
            ));
        }

        None
    }

    fn extract_output(swap: &SwapEvent) -> Option<(String, u64, u8, bool)> {
        // Check for native SOL output first
        if let Some(native_output) = &swap.native_output {
            let amount: u64 = native_output.amount.parse().unwrap_or(0);
            if amount > 0 {
                return Some((
                    "So11111111111111111111111111111111111111112".to_string(),
                    amount,
                    9,
                    true,
                ));
            }
        }

        // Check token outputs
        if let Some(token_output) = swap.token_outputs.first() {
            let amount: u64 = token_output
                .raw_token_amount
                .token_amount
                .parse()
                .unwrap_or(0);
            return Some((
                token_output.mint.clone(),
                amount,
                token_output.raw_token_amount.decimals,
                false,
            ));
        }

        None
    }

    fn extract_dex_source(swap: &SwapEvent) -> String {
        // Try to get the DEX from inner swaps
        if let Some(inner_swap) = swap.inner_swaps.first() {
            return inner_swap.program_info.source.clone();
        }

        "Unknown".to_string()
    }

    pub fn is_swap_transaction(event: &EnhancedTransactionEvent) -> bool {
        event.transaction_type.to_uppercase() == "SWAP" || event.events.swap.is_some()
    }

    pub fn calculate_swap_value_sol(swap: &ParsedSwap) -> f64 {
        if swap.is_native_input {
            swap.input_amount as f64 / 1e9
        } else if swap.is_native_output {
            swap.output_amount as f64 / 1e9
        } else {
            0.0
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KOLTradeSignal {
    pub id: Uuid,
    pub kol_address: String,
    pub kol_name: Option<String>,
    pub swap: ParsedSwap,
    pub value_sol: f64,
    pub copy_recommended: bool,
    pub recommendation_reason: Option<String>,
    pub detected_at: chrono::DateTime<chrono::Utc>,
}

impl KOLTradeSignal {
    pub fn from_swap(swap: ParsedSwap, kol_name: Option<String>, trust_score: f64) -> Self {
        let value_sol = TransactionParser::calculate_swap_value_sol(&swap);
        let copy_recommended = trust_score >= 0.7 && value_sol >= 0.1;
        let recommendation_reason = if copy_recommended {
            Some(format!(
                "High trust KOL ({:.0}%) trading {} SOL",
                trust_score * 100.0,
                value_sol
            ))
        } else if trust_score < 0.7 {
            Some(format!("Trust score too low: {:.0}%", trust_score * 100.0))
        } else {
            Some(format!("Trade value too small: {} SOL", value_sol))
        };

        Self {
            id: Uuid::new_v4(),
            kol_address: swap.wallet_address.clone(),
            kol_name,
            swap,
            value_sol,
            copy_recommended,
            recommendation_reason,
            detected_at: chrono::Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swap_value_calculation() {
        let swap = ParsedSwap {
            id: Uuid::new_v4(),
            signature: "test".to_string(),
            wallet_address: "test".to_string(),
            timestamp: chrono::Utc::now(),
            input_mint: "So11111111111111111111111111111111111111112".to_string(),
            input_amount: 1_000_000_000, // 1 SOL
            input_decimals: 9,
            output_mint: "token".to_string(),
            output_amount: 100_000_000,
            output_decimals: 6,
            dex_source: "Jupiter".to_string(),
            fee_lamports: 5000,
            is_native_input: true,
            is_native_output: false,
        };

        let value = TransactionParser::calculate_swap_value_sol(&swap);
        assert_eq!(value, 1.0);
    }
}
