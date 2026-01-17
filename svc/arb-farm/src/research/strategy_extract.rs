use reqwest::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::error::{AppError, AppResult};
use super::url_ingest::IngestResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrategyConfidence {
    High,
    Medium,
    Low,
}

impl StrategyConfidence {
    pub fn from_score(score: f64) -> Self {
        if score >= 0.75 {
            StrategyConfidence::High
        } else if score >= 0.5 {
            StrategyConfidence::Medium
        } else {
            StrategyConfidence::Low
        }
    }

    pub fn to_score(&self) -> f64 {
        match self {
            StrategyConfidence::High => 0.85,
            StrategyConfidence::Medium => 0.65,
            StrategyConfidence::Low => 0.35,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedStrategy {
    pub id: Uuid,
    pub source_id: Uuid,
    pub source_url: String,
    pub name: String,
    pub description: String,
    pub strategy_type: StrategyType,
    pub entry_conditions: Vec<Condition>,
    pub exit_conditions: Vec<Condition>,
    pub risk_params: RiskParams,
    pub tokens_mentioned: Vec<String>,
    pub confidence: StrategyConfidence,
    pub confidence_score: f64,
    pub raw_extraction: String,
    pub extracted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrategyType {
    DexArbitrage,
    BondingCurve,
    Momentum,
    MeanReversion,
    Breakout,
    Scalping,
    Swing,
    CopyTrade,
    Liquidation,
    Unknown,
}

impl StrategyType {
    pub fn from_str(s: &str) -> Self {
        let lower = s.to_lowercase();
        if lower.contains("arb") || lower.contains("arbitrage") {
            StrategyType::DexArbitrage
        } else if lower.contains("curve") || lower.contains("pump") || lower.contains("bonding") {
            StrategyType::BondingCurve
        } else if lower.contains("momentum") || lower.contains("trend") {
            StrategyType::Momentum
        } else if lower.contains("mean") || lower.contains("revert") {
            StrategyType::MeanReversion
        } else if lower.contains("breakout") || lower.contains("break") {
            StrategyType::Breakout
        } else if lower.contains("scalp") {
            StrategyType::Scalping
        } else if lower.contains("swing") {
            StrategyType::Swing
        } else if lower.contains("copy") || lower.contains("follow") {
            StrategyType::CopyTrade
        } else if lower.contains("liquidat") {
            StrategyType::Liquidation
        } else {
            StrategyType::Unknown
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub description: String,
    pub condition_type: ConditionType,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionType {
    PriceAbove,
    PriceBelow,
    VolumeSpike,
    TimeWindow,
    PercentageGain,
    PercentageLoss,
    CurveProgress,
    HolderConcentration,
    MarketCapThreshold,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskParams {
    pub max_position_sol: Option<f64>,
    pub stop_loss_percent: Option<f64>,
    pub take_profit_percent: Option<f64>,
    pub max_slippage_bps: Option<u16>,
    pub time_limit_minutes: Option<u32>,
}

impl Default for RiskParams {
    fn default() -> Self {
        Self {
            max_position_sol: Some(1.0),
            stop_loss_percent: Some(10.0),
            take_profit_percent: Some(50.0),
            max_slippage_bps: Some(100),
            time_limit_minutes: Some(60),
        }
    }
}

pub struct StrategyExtractor {
    client: Client,
    openrouter_url: String,
    openrouter_key: Option<String>,
    model: String,
}

impl StrategyExtractor {
    pub fn new(openrouter_url: String, openrouter_key: Option<String>) -> Self {
        Self {
            client: Client::new(),
            openrouter_url,
            openrouter_key,
            model: "anthropic/claude-3-haiku".to_string(),
        }
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }

    pub async fn extract(&self, ingest_result: &IngestResult) -> AppResult<Option<ExtractedStrategy>> {
        let prompt = self.build_extraction_prompt(ingest_result);

        let response = self.call_llm(&prompt).await?;

        self.parse_llm_response(&response, ingest_result)
    }

    fn build_extraction_prompt(&self, ingest: &IngestResult) -> String {
        format!(
            r#"Analyze this content for trading strategies. Extract any actionable trading strategy.

Source URL: {}
Content Type: {:?}
Author: {}

Content:
{}

Tokens Mentioned: {:?}
Numbers Found: {:?}

If a trading strategy is found, respond with JSON in this exact format:
{{
  "found": true,
  "name": "Strategy Name",
  "description": "Brief description",
  "strategy_type": "one of: dex_arb, bonding_curve, momentum, mean_reversion, breakout, scalping, swing, copy_trade, liquidation, unknown",
  "entry_conditions": [
    {{"description": "condition description", "type": "price_above|price_below|volume_spike|time_window|percentage_gain|percentage_loss|curve_progress|holder_concentration|market_cap|custom", "params": {{}}}}
  ],
  "exit_conditions": [
    {{"description": "condition description", "type": "same types as entry", "params": {{}}}}
  ],
  "risk_params": {{
    "max_position_sol": 1.0,
    "stop_loss_percent": 10,
    "take_profit_percent": 50,
    "max_slippage_bps": 100
  }},
  "confidence": 0.75
}}

If NO trading strategy is found, respond with:
{{"found": false, "reason": "why no strategy was found"}}

Only extract strategies that have clear, actionable entry/exit conditions. Ignore vague statements like "buy low sell high"."#,
            ingest.url,
            ingest.content_type,
            ingest.author.as_deref().unwrap_or("Unknown"),
            ingest.cleaned_content,
            ingest.extracted_tokens,
            ingest.extracted_numbers,
        )
    }

    async fn call_llm(&self, prompt: &str) -> AppResult<String> {
        let api_key = self.openrouter_key.as_ref()
            .ok_or_else(|| AppError::Configuration("OpenRouter API key not configured".to_string()))?;

        let request_body = serde_json::json!({
            "model": self.model,
            "messages": [
                {
                    "role": "system",
                    "content": "You are a trading strategy analyst. Extract actionable trading strategies from content. Be precise and only extract strategies with clear entry/exit conditions. Respond only with valid JSON."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0.3,
            "max_tokens": 2000
        });

        let response = self.client
            .post(format!("{}/chat/completions", self.openrouter_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        let response_json: serde_json::Value = response.json().await?;

        let content = response_json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(content)
    }

    fn parse_llm_response(&self, response: &str, ingest: &IngestResult) -> AppResult<Option<ExtractedStrategy>> {
        let json_str = self.extract_json_from_response(response);

        let parsed: LlmExtractionResponse = match serde_json::from_str(&json_str) {
            Ok(p) => p,
            Err(_) => return Ok(None),
        };

        if !parsed.found {
            return Ok(None);
        }

        let strategy_type = StrategyType::from_str(&parsed.strategy_type.unwrap_or_default());

        let entry_conditions = parsed.entry_conditions
            .unwrap_or_default()
            .into_iter()
            .map(|c| Condition {
                description: c.description,
                condition_type: self.parse_condition_type(&c.condition_type),
                parameters: c.params.unwrap_or(serde_json::json!({})),
            })
            .collect();

        let exit_conditions = parsed.exit_conditions
            .unwrap_or_default()
            .into_iter()
            .map(|c| Condition {
                description: c.description,
                condition_type: self.parse_condition_type(&c.condition_type),
                parameters: c.params.unwrap_or(serde_json::json!({})),
            })
            .collect();

        let risk_params = if let Some(rp) = parsed.risk_params {
            RiskParams {
                max_position_sol: rp.max_position_sol,
                stop_loss_percent: rp.stop_loss_percent,
                take_profit_percent: rp.take_profit_percent,
                max_slippage_bps: rp.max_slippage_bps,
                time_limit_minutes: None,
            }
        } else {
            RiskParams::default()
        };

        let confidence_score = parsed.confidence.unwrap_or(0.5);

        Ok(Some(ExtractedStrategy {
            id: Uuid::new_v4(),
            source_id: ingest.id,
            source_url: ingest.url.clone(),
            name: parsed.name.unwrap_or_else(|| "Unnamed Strategy".to_string()),
            description: parsed.description.unwrap_or_default(),
            strategy_type,
            entry_conditions,
            exit_conditions,
            risk_params,
            tokens_mentioned: ingest.extracted_tokens.clone(),
            confidence: StrategyConfidence::from_score(confidence_score),
            confidence_score,
            raw_extraction: response.to_string(),
            extracted_at: Utc::now(),
        }))
    }

    fn extract_json_from_response(&self, response: &str) -> String {
        if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                return response[start..=end].to_string();
            }
        }
        response.to_string()
    }

    fn parse_condition_type(&self, s: &str) -> ConditionType {
        match s.to_lowercase().as_str() {
            "price_above" => ConditionType::PriceAbove,
            "price_below" => ConditionType::PriceBelow,
            "volume_spike" => ConditionType::VolumeSpike,
            "time_window" => ConditionType::TimeWindow,
            "percentage_gain" => ConditionType::PercentageGain,
            "percentage_loss" => ConditionType::PercentageLoss,
            "curve_progress" => ConditionType::CurveProgress,
            "holder_concentration" => ConditionType::HolderConcentration,
            "market_cap" | "market_cap_threshold" => ConditionType::MarketCapThreshold,
            _ => ConditionType::Custom,
        }
    }

    pub fn extract_without_llm(&self, ingest: &IngestResult) -> Option<ExtractedStrategy> {
        let content_lower = ingest.cleaned_content.to_lowercase();

        let strategy_type = if content_lower.contains("pump.fun") || content_lower.contains("bonding curve") {
            Some(StrategyType::BondingCurve)
        } else if content_lower.contains("arbitrage") || content_lower.contains("arb ") {
            Some(StrategyType::DexArbitrage)
        } else {
            None
        };

        let strategy_type = strategy_type?;

        let mut entry_conditions = Vec::new();
        let mut exit_conditions = Vec::new();

        for num in &ingest.extracted_numbers {
            match num.context.as_str() {
                "percentage" => {
                    if content_lower.contains("buy") || content_lower.contains("entry") {
                        entry_conditions.push(Condition {
                            description: format!("At {}%", num.value),
                            condition_type: ConditionType::CurveProgress,
                            parameters: serde_json::json!({"threshold": num.value}),
                        });
                    }
                    if content_lower.contains("sell") || content_lower.contains("exit") {
                        exit_conditions.push(Condition {
                            description: format!("At {}%", num.value),
                            condition_type: ConditionType::CurveProgress,
                            parameters: serde_json::json!({"threshold": num.value}),
                        });
                    }
                }
                "multiplier" => {
                    exit_conditions.push(Condition {
                        description: format!("{}x profit target", num.value),
                        condition_type: ConditionType::PercentageGain,
                        parameters: serde_json::json!({"multiplier": num.value}),
                    });
                }
                _ => {}
            }
        }

        if entry_conditions.is_empty() && exit_conditions.is_empty() {
            return None;
        }

        Some(ExtractedStrategy {
            id: Uuid::new_v4(),
            source_id: ingest.id,
            source_url: ingest.url.clone(),
            name: format!("{:?} Strategy", strategy_type),
            description: "Auto-extracted strategy from content".to_string(),
            strategy_type,
            entry_conditions,
            exit_conditions,
            risk_params: RiskParams::default(),
            tokens_mentioned: ingest.extracted_tokens.clone(),
            confidence: StrategyConfidence::Low,
            confidence_score: 0.4,
            raw_extraction: String::new(),
            extracted_at: Utc::now(),
        })
    }
}

#[derive(Debug, Deserialize)]
struct LlmExtractionResponse {
    found: bool,
    name: Option<String>,
    description: Option<String>,
    strategy_type: Option<String>,
    entry_conditions: Option<Vec<LlmCondition>>,
    exit_conditions: Option<Vec<LlmCondition>>,
    risk_params: Option<LlmRiskParams>,
    confidence: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct LlmCondition {
    description: String,
    #[serde(rename = "type")]
    condition_type: String,
    params: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct LlmRiskParams {
    max_position_sol: Option<f64>,
    stop_loss_percent: Option<f64>,
    take_profit_percent: Option<f64>,
    max_slippage_bps: Option<u16>,
}

pub struct TextStrategyExtractor {
    client: Client,
    openrouter_url: String,
    openrouter_key: Option<String>,
    model: String,
}

impl TextStrategyExtractor {
    pub fn new(openrouter_url: String, openrouter_key: Option<String>) -> Self {
        Self {
            client: Client::new(),
            openrouter_url,
            openrouter_key,
            model: "anthropic/claude-3-haiku".to_string(),
        }
    }

    pub async fn extract_from_text(&self, description: &str, context: Option<&str>) -> AppResult<Option<ExtractedStrategy>> {
        let prompt = self.build_text_extraction_prompt(description, context);
        let response = self.call_llm(&prompt).await?;
        self.parse_llm_response(&response, description)
    }

    fn build_text_extraction_prompt(&self, description: &str, context: Option<&str>) -> String {
        let context_section = context
            .map(|c| format!("\nAdditional Context:\n{}", c))
            .unwrap_or_default();

        format!(
            r#"Extract a trading strategy from this natural language description.

User's Strategy Description:
{}
{}

Create a structured trading strategy based on this description. Identify:
- Strategy type (dex_arb, bonding_curve, momentum, mean_reversion, breakout, scalping, swing, copy_trade, liquidation)
- Entry conditions (when to buy)
- Exit conditions (when to sell)
- Risk parameters (position size, stop loss, take profit, etc.)

Respond with JSON in this exact format:
{{
  "found": true,
  "name": "Strategy Name based on description",
  "description": "Clear description of the strategy",
  "strategy_type": "one of: dex_arb, bonding_curve, momentum, mean_reversion, breakout, scalping, swing, copy_trade, liquidation",
  "entry_conditions": [
    {{"description": "condition description", "type": "price_above|price_below|volume_spike|time_window|percentage_gain|percentage_loss|curve_progress|holder_concentration|market_cap|custom", "params": {{}}}}
  ],
  "exit_conditions": [
    {{"description": "condition description", "type": "same types as entry", "params": {{}}}}
  ],
  "risk_params": {{
    "max_position_sol": 1.0,
    "stop_loss_percent": 10,
    "take_profit_percent": 50,
    "max_slippage_bps": 100,
    "time_limit_minutes": 60
  }},
  "venue_types": ["dex_amm", "bonding_curve"],
  "execution_mode": "agent_directed",
  "confidence": 0.75
}}

If the description is too vague or doesn't describe a clear trading strategy, respond with:
{{"found": false, "reason": "explanation"}}

Be helpful - try to infer reasonable defaults from the description. If the user mentions "small positions" use 0.5 SOL, if they mention "aggressive" use higher take profits, etc."#,
            description,
            context_section,
        )
    }

    async fn call_llm(&self, prompt: &str) -> AppResult<String> {
        let api_key = self.openrouter_key.as_ref()
            .ok_or_else(|| AppError::Configuration("OpenRouter API key not configured".to_string()))?;

        let request_body = serde_json::json!({
            "model": self.model,
            "messages": [
                {
                    "role": "system",
                    "content": "You are a helpful trading strategy assistant. Convert natural language descriptions into structured trading strategies. Be practical and infer reasonable defaults. Respond only with valid JSON."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0.3,
            "max_tokens": 2000
        });

        let response = self.client
            .post(format!("{}/chat/completions", self.openrouter_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        let response_json: serde_json::Value = response.json().await?;

        let content = response_json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(content)
    }

    fn parse_llm_response(&self, response: &str, original_description: &str) -> AppResult<Option<ExtractedStrategy>> {
        let json_str = self.extract_json_from_response(response);

        let parsed: TextExtractionResponse = match serde_json::from_str(&json_str) {
            Ok(p) => p,
            Err(_) => return Ok(None),
        };

        if !parsed.found {
            return Ok(None);
        }

        let strategy_type = StrategyType::from_str(&parsed.strategy_type.unwrap_or_default());

        let entry_conditions = parsed.entry_conditions
            .unwrap_or_default()
            .into_iter()
            .map(|c| Condition {
                description: c.description,
                condition_type: self.parse_condition_type(&c.condition_type),
                parameters: c.params.unwrap_or(serde_json::json!({})),
            })
            .collect();

        let exit_conditions = parsed.exit_conditions
            .unwrap_or_default()
            .into_iter()
            .map(|c| Condition {
                description: c.description,
                condition_type: self.parse_condition_type(&c.condition_type),
                parameters: c.params.unwrap_or(serde_json::json!({})),
            })
            .collect();

        let risk_params = if let Some(rp) = parsed.risk_params {
            RiskParams {
                max_position_sol: rp.max_position_sol,
                stop_loss_percent: rp.stop_loss_percent,
                take_profit_percent: rp.take_profit_percent,
                max_slippage_bps: rp.max_slippage_bps,
                time_limit_minutes: rp.time_limit_minutes,
            }
        } else {
            RiskParams::default()
        };

        let confidence_score = parsed.confidence.unwrap_or(0.5);

        Ok(Some(ExtractedStrategy {
            id: Uuid::new_v4(),
            source_id: Uuid::new_v4(),
            source_url: "user_input".to_string(),
            name: parsed.name.unwrap_or_else(|| "Custom Strategy".to_string()),
            description: parsed.description.unwrap_or_else(|| original_description.to_string()),
            strategy_type,
            entry_conditions,
            exit_conditions,
            risk_params,
            tokens_mentioned: Vec::new(),
            confidence: StrategyConfidence::from_score(confidence_score),
            confidence_score,
            raw_extraction: response.to_string(),
            extracted_at: Utc::now(),
        }))
    }

    fn extract_json_from_response(&self, response: &str) -> String {
        if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                return response[start..=end].to_string();
            }
        }
        response.to_string()
    }

    fn parse_condition_type(&self, s: &str) -> ConditionType {
        match s.to_lowercase().as_str() {
            "price_above" => ConditionType::PriceAbove,
            "price_below" => ConditionType::PriceBelow,
            "volume_spike" => ConditionType::VolumeSpike,
            "time_window" => ConditionType::TimeWindow,
            "percentage_gain" => ConditionType::PercentageGain,
            "percentage_loss" => ConditionType::PercentageLoss,
            "curve_progress" => ConditionType::CurveProgress,
            "holder_concentration" => ConditionType::HolderConcentration,
            "market_cap" | "market_cap_threshold" => ConditionType::MarketCapThreshold,
            _ => ConditionType::Custom,
        }
    }
}

#[derive(Debug, Deserialize)]
struct TextExtractionResponse {
    found: bool,
    name: Option<String>,
    description: Option<String>,
    strategy_type: Option<String>,
    entry_conditions: Option<Vec<LlmCondition>>,
    exit_conditions: Option<Vec<LlmCondition>>,
    risk_params: Option<TextLlmRiskParams>,
    #[allow(dead_code)]
    venue_types: Option<Vec<String>>,
    #[allow(dead_code)]
    execution_mode: Option<String>,
    confidence: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct TextLlmRiskParams {
    max_position_sol: Option<f64>,
    stop_loss_percent: Option<f64>,
    take_profit_percent: Option<f64>,
    max_slippage_bps: Option<u16>,
    time_limit_minutes: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_type_parsing() {
        assert!(matches!(StrategyType::from_str("dex arbitrage"), StrategyType::DexArbitrage));
        assert!(matches!(StrategyType::from_str("pump.fun curve"), StrategyType::BondingCurve));
        assert!(matches!(StrategyType::from_str("momentum trading"), StrategyType::Momentum));
    }

    #[test]
    fn test_confidence_from_score() {
        assert!(matches!(StrategyConfidence::from_score(0.9), StrategyConfidence::High));
        assert!(matches!(StrategyConfidence::from_score(0.6), StrategyConfidence::Medium));
        assert!(matches!(StrategyConfidence::from_score(0.3), StrategyConfidence::Low));
    }
}
