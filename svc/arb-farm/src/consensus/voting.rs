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
}

impl Default for VotingEngine {
    fn default() -> Self {
        Self {
            min_agreement: 0.5,
            min_weighted_confidence: 0.6,
        }
    }
}

impl VotingEngine {
    pub fn new(min_agreement: f64, min_weighted_confidence: f64) -> Self {
        Self {
            min_agreement: min_agreement.clamp(0.0, 1.0),
            min_weighted_confidence: min_weighted_confidence.clamp(0.0, 1.0),
        }
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

        let approved = agreement_score >= self.min_agreement
            && weighted_confidence >= self.min_weighted_confidence;

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
