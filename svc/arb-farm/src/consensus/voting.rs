use serde::{Deserialize, Serialize};

use super::openrouter::get_model_weight;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelVote {
    pub model: String,
    pub approved: bool,
    pub confidence: f64,
    pub reasoning: String,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusResult {
    pub approved: bool,
    pub agreement_score: f64,
    pub weighted_confidence: f64,
    pub model_votes: Vec<ModelVote>,
    pub reasoning_summary: String,
    pub total_latency_ms: u64,
}

pub struct VotingEngine {
    min_agreement: f64,
    min_weighted_confidence: f64,
    min_quorum: usize,
    expected_models: usize,
}

impl Default for VotingEngine {
    fn default() -> Self {
        Self {
            min_agreement: 0.5,
            min_weighted_confidence: 0.6,
            min_quorum: 1, // Minimum 1 model for any consensus (graceful degradation)
            expected_models: 3, // Expect 3 models for full confidence
        }
    }
}

impl VotingEngine {
    pub fn new(min_agreement: f64, min_weighted_confidence: f64) -> Self {
        Self {
            min_agreement: min_agreement.clamp(0.0, 1.0),
            min_weighted_confidence: min_weighted_confidence.clamp(0.0, 1.0),
            min_quorum: 1,
            expected_models: 3,
        }
    }

    pub fn with_quorum(mut self, min_quorum: usize, expected_models: usize) -> Self {
        self.min_quorum = min_quorum.max(1);
        self.expected_models = expected_models.max(min_quorum);
        self
    }

    pub fn calculate_consensus(&self, votes: Vec<ModelVote>) -> ConsensusResult {
        if votes.is_empty() {
            return ConsensusResult {
                approved: false,
                agreement_score: 0.0,
                weighted_confidence: 0.0,
                model_votes: vec![],
                reasoning_summary: "No votes received".to_string(),
                total_latency_ms: 0,
            };
        }

        let total_weight: f64 = votes.iter().map(|v| get_model_weight(&v.model)).sum();

        let weighted_approve: f64 = votes
            .iter()
            .filter(|v| v.approved)
            .map(|v| get_model_weight(&v.model))
            .sum();

        let agreement_score = weighted_approve / total_weight;

        let weighted_confidence: f64 = votes
            .iter()
            .map(|v| v.confidence * get_model_weight(&v.model))
            .sum::<f64>()
            / total_weight;

        // Apply quorum penalty: reduce confidence if fewer than expected models responded
        let quorum_ratio = (votes.len() as f64) / (self.expected_models as f64);
        let adjusted_confidence = weighted_confidence * quorum_ratio.min(1.0);

        // Check if we meet minimum quorum (graceful degradation)
        let meets_quorum = votes.len() >= self.min_quorum;
        if !meets_quorum {
            tracing::warn!(
                "Consensus quorum not met: {} votes < {} required",
                votes.len(),
                self.min_quorum
            );
        }

        // Log degraded consensus warning
        if votes.len() < self.expected_models {
            tracing::warn!(
                "⚠️ Degraded consensus: only {}/{} models responded (confidence adjusted from {:.2} to {:.2})",
                votes.len(),
                self.expected_models,
                weighted_confidence,
                adjusted_confidence
            );
        }

        let approved = meets_quorum
            && agreement_score >= self.min_agreement
            && adjusted_confidence >= self.min_weighted_confidence;

        let total_latency_ms = votes.iter().map(|v| v.latency_ms).max().unwrap_or(0);

        let reasoning_summary = self.summarize_reasoning(&votes, approved);

        ConsensusResult {
            approved,
            agreement_score,
            weighted_confidence,
            model_votes: votes,
            reasoning_summary,
            total_latency_ms,
        }
    }

    fn summarize_reasoning(&self, votes: &[ModelVote], approved: bool) -> String {
        let approve_count = votes.iter().filter(|v| v.approved).count();
        let reject_count = votes.len() - approve_count;

        let decision = if approved { "APPROVED" } else { "REJECTED" };

        let key_reasons: Vec<String> = votes
            .iter()
            .take(3)
            .map(|v| {
                let status = if v.approved { "✓" } else { "✗" };
                format!(
                    "{} {} ({:.0}%): {}",
                    status,
                    short_model_name(&v.model),
                    v.confidence * 100.0,
                    truncate_reasoning(&v.reasoning, 100)
                )
            })
            .collect();

        format!(
            "{}: {}/{} models approved. {}",
            decision,
            approve_count,
            votes.len(),
            key_reasons.join(" | ")
        )
    }
}

fn short_model_name(model: &str) -> &str {
    model.split('/').last().unwrap_or(model)
}

fn truncate_reasoning(reasoning: &str, max_len: usize) -> String {
    if reasoning.len() <= max_len {
        reasoning.to_string()
    } else {
        format!("{}...", &reasoning[..max_len - 3])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeApprovalVote {
    pub approved: bool,
    pub confidence: f64,
    pub reasoning: String,
    pub risk_assessment: String,
    pub recommended_position_size: Option<f64>,
}

pub fn parse_trade_approval(content: &str) -> Option<TradeApprovalVote> {
    let json_start = content.find('{');
    let json_end = content.rfind('}');

    let json_str = match (json_start, json_end) {
        (Some(start), Some(end)) => &content[start..=end],
        _ => return None,
    };

    serde_json::from_str(json_str).ok()
}

pub fn generate_trade_prompt(edge_context: &str) -> String {
    format!(
        r#"You are an autonomous MEV trading agent on Solana. Your PRIMARY OBJECTIVE is to MAXIMIZE PROFIT measured in base currency (SOL or USDC).

## PROFIT MOTIVATION
- Your goal is to generate NET POSITIVE RETURNS in SOL or USDC
- After any trade completes, positions are settled back to base currency
- You should NOT hold random tokens - final position must be in SOL/USDC
- Success = growing the base currency balance over time
- Performance is measured by overall portfolio growth in SOL/USDC terms

## Trade Lifecycle
1. Entry: Use SOL/USDC to acquire opportunity
2. Execution: Complete the arbitrage or trade
3. Settlement: Convert any acquired tokens BACK to SOL/USDC
4. Profit: Net gain/loss measured in base currency

## Opportunity Analysis
{}

Respond with a JSON object in this exact format:
{{
    "approved": true/false,
    "confidence": 0.0-1.0,
    "reasoning": "Your detailed reasoning - explain the expected profit in SOL/USDC",
    "risk_assessment": "low/medium/high with explanation",
    "recommended_position_size": 0.0-1.0 or null
}}

## Evaluation Criteria (in priority order):
1. NET PROFIT in SOL/USDC - What is the expected return after settlement to base currency?
2. Settlement risk - Can we reliably convert back to base currency?
3. Risk-adjusted returns - Is the profit worth the risk of loss?
4. Execution probability - Can this trade be executed successfully?
5. Slippage and fees - Do transaction costs eat into profit?
6. Security - Is there rug/scam risk that could result in total loss?

IMPORTANT: Only approve trades with CLEAR, QUANTIFIABLE profit potential in base currency. When in doubt, reject."#,
        edge_context
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisContext {
    pub total_trades: u32,
    pub winning_trades: u32,
    pub win_rate: f64,
    pub total_pnl_sol: f64,
    pub today_pnl_sol: f64,
    pub week_pnl_sol: f64,
    pub avg_hold_minutes: f64,
    pub best_trade: Option<TradeHighlightContext>,
    pub worst_trade: Option<TradeHighlightContext>,
    pub take_profit_count: u32,
    pub stop_loss_count: u32,
    pub recent_errors: Vec<ErrorSummary>,
    pub time_period: String,
    #[serde(default)]
    pub recent_trades: Vec<DetailedTradeContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeHighlightContext {
    pub symbol: String,
    pub pnl_sol: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorSummary {
    pub error_type: String,
    pub count: u32,
    pub last_message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedTradeContext {
    pub position_id: uuid::Uuid,
    pub token_symbol: String,
    pub venue: String,
    pub entry_sol: f64,
    pub exit_sol: f64,
    pub pnl_sol: f64,
    pub pnl_percent: f64,
    pub hold_minutes: f64,
    pub exit_reason: String,
    pub stop_loss_pct: Option<f64>,
    pub take_profit_pct: Option<f64>,
    pub entry_time: chrono::DateTime<chrono::Utc>,
    pub exit_time: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisVote {
    pub recommendations: Vec<ParsedRecommendation>,
    pub risk_alerts: Vec<String>,
    pub overall_assessment: String,
    pub confidence: f64,
    #[serde(default)]
    pub trade_analyses: Vec<TradeAnalysisItem>,
    #[serde(default)]
    pub pattern_summary: Option<PatternSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeAnalysisItem {
    pub position_id: String,
    pub root_cause: String,
    #[serde(default)]
    pub config_issue: Option<String>,
    #[serde(default)]
    pub pattern: Option<String>,
    #[serde(default)]
    pub suggested_fix: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PatternSummary {
    #[serde(default)]
    pub losing_patterns: Vec<String>,
    #[serde(default)]
    pub winning_patterns: Vec<String>,
    #[serde(default)]
    pub config_recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedRecommendation {
    pub category: String,
    pub title: String,
    pub description: String,
    pub action_type: String,
    pub target: String,
    pub current_value: Option<serde_json::Value>,
    pub suggested_value: serde_json::Value,
    pub reasoning: String,
    pub confidence: f64,
}

pub fn generate_analysis_prompt(context: &AnalysisContext) -> String {
    let best_trade_str = context
        .best_trade
        .as_ref()
        .map(|t| format!("{} (+{:.4} SOL)", t.symbol, t.pnl_sol))
        .unwrap_or_else(|| "None".to_string());
    let worst_trade_str = context
        .worst_trade
        .as_ref()
        .map(|t| format!("{} ({:.4} SOL)", t.symbol, t.pnl_sol))
        .unwrap_or_else(|| "None".to_string());

    let errors_summary = if context.recent_errors.is_empty() {
        "No recent errors".to_string()
    } else {
        context
            .recent_errors
            .iter()
            .map(|e| format!("  - {} (x{}): {}", e.error_type, e.count, e.last_message))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let trades_table = if context.recent_trades.is_empty() {
        "No recent trades to analyze.".to_string()
    } else {
        let mut table = String::from(
            "| # | Token | Venue | Entry | Exit | PnL | Hold | Exit Reason | SL% | TP% |\n",
        );
        table.push_str(
            "|---|-------|-------|-------|------|-----|------|-------------|-----|-----|\n",
        );
        for (i, trade) in context.recent_trades.iter().enumerate() {
            let sl_str = trade
                .stop_loss_pct
                .map(|v| format!("{:.1}", v))
                .unwrap_or_else(|| "-".to_string());
            let tp_str = trade
                .take_profit_pct
                .map(|v| format!("{:.1}", v))
                .unwrap_or_else(|| "-".to_string());
            let pnl_sign = if trade.pnl_sol >= 0.0 { "+" } else { "" };
            table.push_str(&format!(
                "| {} | {} | {} | {:.4} | {:.4} | {}{:.4} ({:.1}%) | {:.1}m | {} | {} | {} |\n",
                i + 1,
                trade.token_symbol,
                trade.venue,
                trade.entry_sol,
                trade.exit_sol,
                pnl_sign,
                trade.pnl_sol,
                trade.pnl_percent,
                trade.hold_minutes,
                trade.exit_reason,
                sl_str,
                tp_str
            ));
        }
        table
    };

    let losing_trades_note = if !context.recent_trades.is_empty() {
        let losing_count = context
            .recent_trades
            .iter()
            .filter(|t| t.pnl_sol < 0.0)
            .count();
        format!("\n**Losing Trades to Analyze: {}**", losing_count)
    } else {
        String::new()
    };

    format!(
        r#"You are an AI trading analyst for an autonomous MEV agent on Solana. Analyze the trading performance and provide actionable recommendations to improve profitability.

## Performance Summary ({})

### Trading Stats
- Total Trades: {}
- Winning Trades: {} ({:.1}% win rate)
- Total PnL: {:.4} SOL
- Today's PnL: {:.4} SOL
- Week's PnL: {:.4} SOL
- Avg Hold Time: {:.1} minutes

### Exit Stats
- Take Profits: {}
- Stop Losses: {}

### Notable Trades
- Best Trade: {}
- Worst Trade: {}

### Recent Errors
{}

## Recent Trades (Analyze Each)
{}
{}

## Your Task

1. **Per-Trade Analysis**: For each LOSING trade above, identify the root cause and config correlation.
2. **Pattern Discovery**: Group similar issues across trades to find systematic problems.
3. **Actionable Recommendations**: Provide specific config changes with evidence from the trades.

Respond with a JSON object in this exact format:
{{
    "trade_analyses": [
        {{
            "position_id": "uuid from table",
            "root_cause": "specific issue (e.g., 'SL triggered during normal volatility')",
            "config_issue": "SL at 5% triggered, but token recovered to +20% - suggest 8% SL",
            "pattern": "pump.fun early entry SL trigger",
            "suggested_fix": "Increase stop_loss_percent from 5% to 8% for pump.fun entries"
        }}
    ],
    "pattern_summary": {{
        "losing_patterns": ["pattern1: X trades affected", "pattern2: Y trades affected"],
        "winning_patterns": ["pattern1: why these worked"],
        "config_recommendations": ["specific config change with evidence"]
    }},
    "recommendations": [
        {{
            "category": "strategy|risk|timing|venue|position",
            "title": "Brief title (max 50 chars)",
            "description": "Detailed explanation with trade evidence",
            "action_type": "config_change|strategy_toggle|risk_adjustment|venue_disable|avoid_token",
            "target": "The config/strategy/venue to modify",
            "current_value": null,
            "suggested_value": "The recommended value",
            "reasoning": "Why this change will improve performance - cite specific trades",
            "confidence": 0.0-1.0
        }}
    ],
    "risk_alerts": ["Any immediate risk concerns"],
    "overall_assessment": "Summary of trading performance and key insights",
    "confidence": 0.0-1.0
}}

Requirements:
- Analyze EACH losing trade and provide specific root cause
- Group similar issues into patterns
- Provide 2-5 specific recommendations with TRADE EVIDENCE
- Each recommendation must cite which trades support the change
- Prioritize recommendations by potential profit impact"#,
        context.time_period,
        context.total_trades,
        context.winning_trades,
        context.win_rate * 100.0,
        context.total_pnl_sol,
        context.today_pnl_sol,
        context.week_pnl_sol,
        context.avg_hold_minutes,
        context.take_profit_count,
        context.stop_loss_count,
        best_trade_str,
        worst_trade_str,
        errors_summary,
        trades_table,
        losing_trades_note
    )
}

pub fn parse_analysis_response(content: &str) -> Option<AnalysisVote> {
    let json_start = content.find('{');
    let json_end = content.rfind('}');

    let json_str = match (json_start, json_end) {
        (Some(start), Some(end)) if end > start => &content[start..=end],
        _ => return None,
    };

    serde_json::from_str(json_str).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consensus_calculation() {
        let engine = VotingEngine::default();

        let votes = vec![
            ModelVote {
                model: "anthropic/claude-3.5-sonnet".to_string(),
                approved: true,
                confidence: 0.85,
                reasoning: "Strong opportunity".to_string(),
                latency_ms: 1200,
            },
            ModelVote {
                model: "openai/gpt-4-turbo".to_string(),
                approved: true,
                confidence: 0.75,
                reasoning: "Looks profitable".to_string(),
                latency_ms: 800,
            },
            ModelVote {
                model: "meta-llama/llama-3.1-70b-instruct".to_string(),
                approved: false,
                confidence: 0.60,
                reasoning: "Too risky".to_string(),
                latency_ms: 600,
            },
        ];

        let result = engine.calculate_consensus(votes);

        assert!(result.approved);
        assert!(result.agreement_score > 0.5);
    }
}
