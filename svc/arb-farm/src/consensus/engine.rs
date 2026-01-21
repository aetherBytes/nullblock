use chrono::Utc;
use futures::future::join_all;
use uuid::Uuid;

use super::{
    openrouter::{get_default_models, get_model_weight, OpenRouterClient},
    voting::{
        generate_trade_prompt, parse_trade_approval, ConsensusResult, ModelVote, VotingEngine,
        AnalysisContext, AnalysisVote, ParsedRecommendation, generate_analysis_prompt, parse_analysis_response,
        TradeAnalysisItem, PatternSummary,
    },
};
use crate::{
    error::{AppError, AppResult},
    events::{AgentType, ArbEvent, EventSource},
    models::{
        ArbEngram, ConsensusOutcomeContent, EngramSource, EngramType, ModelVoteContent,
        TradeResultSummary,
    },
};

pub struct ConsensusEngine {
    openrouter: OpenRouterClient,
    voting_engine: VotingEngine,
    default_models: Vec<String>,
    timeout_ms: u64,
}

impl ConsensusEngine {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            openrouter: OpenRouterClient::new(api_key),
            voting_engine: VotingEngine::default(),
            default_models: get_default_models(),
            timeout_ms: 30000,
        }
    }

    pub fn with_models(mut self, models: Vec<String>) -> Self {
        self.default_models = models;
        self
    }

    pub fn with_min_agreement(mut self, min_agreement: f64) -> Self {
        self.voting_engine = VotingEngine::new(min_agreement, 0.6);
        self
    }

    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    pub async fn is_ready(&self) -> bool {
        !self.default_models.is_empty()
    }

    pub async fn request_analysis(
        &self,
        context: AnalysisContext,
    ) -> AppResult<AnalysisResult> {
        let models_to_query = self.default_models.clone();
        let prompt = generate_analysis_prompt(&context);

        let system_prompt = Some(
            "You are an expert trading analyst for an autonomous Solana MEV agent. Your goal is to analyze trading performance data and provide actionable recommendations to maximize profit in SOL. Focus on data-driven insights and specific, measurable improvements. Always respond with valid JSON.",
        );

        tracing::info!(
            time_period = %context.time_period,
            total_trades = context.total_trades,
            models = ?models_to_query,
            "Starting consensus analysis"
        );

        let futures: Vec<_> = models_to_query
            .iter()
            .map(|model| self.query_analysis_model(model, &prompt, system_prompt))
            .collect();

        let results = join_all(futures).await;

        let model_votes: Vec<AnalysisModelVote> = results
            .into_iter()
            .filter_map(|r| r.ok())
            .collect();

        if model_votes.is_empty() {
            return Err(AppError::ConsensusFailed(
                "All model queries failed for analysis".to_string(),
            ));
        }

        let aggregated = self.aggregate_analysis_votes(&model_votes);

        tracing::info!(
            recommendations_count = aggregated.recommendations.len(),
            risk_alerts_count = aggregated.risk_alerts.len(),
            avg_confidence = aggregated.avg_confidence,
            models_responded = model_votes.len(),
            "Consensus analysis completed"
        );

        Ok(aggregated)
    }

    async fn query_analysis_model(
        &self,
        model: &str,
        prompt: &str,
        system_prompt: Option<&str>,
    ) -> AppResult<AnalysisModelVote> {
        let timeout = tokio::time::Duration::from_millis(self.timeout_ms);

        let result = tokio::time::timeout(
            timeout,
            self.openrouter.query_model(model, prompt, system_prompt, 2048),
        )
        .await
        .map_err(|_| AppError::Timeout(format!("Analysis model {} timed out", model)))?;

        let response = result?;

        let vote = parse_analysis_response(&response.content).ok_or_else(|| {
            AppError::ExternalApi(format!(
                "Failed to parse analysis vote from {}: {}",
                model,
                &response.content[..response.content.len().min(200)]
            ))
        })?;

        Ok(AnalysisModelVote {
            model: model.to_string(),
            vote,
            latency_ms: response.latency_ms,
        })
    }

    fn aggregate_analysis_votes(&self, votes: &[AnalysisModelVote]) -> AnalysisResult {
        if votes.is_empty() {
            return AnalysisResult::default();
        }

        let mut all_recommendations: Vec<(ParsedRecommendation, f64)> = Vec::new();
        let mut all_risk_alerts: Vec<String> = Vec::new();
        let mut assessments: Vec<String> = Vec::new();
        let mut total_confidence = 0.0;
        let mut total_weight = 0.0;
        let mut all_trade_analyses: Vec<TradeAnalysisItem> = Vec::new();
        let mut aggregated_pattern_summary: Option<PatternSummary> = None;

        for vote in votes {
            let weight = get_model_weight(&vote.model);
            total_weight += weight;
            total_confidence += vote.vote.confidence * weight;

            for rec in &vote.vote.recommendations {
                all_recommendations.push((rec.clone(), weight));
            }

            for alert in &vote.vote.risk_alerts {
                if !all_risk_alerts.contains(alert) {
                    all_risk_alerts.push(alert.clone());
                }
            }

            assessments.push(vote.vote.overall_assessment.clone());

            // Collect trade analyses (dedup by position_id)
            for analysis in &vote.vote.trade_analyses {
                if !all_trade_analyses.iter().any(|a| a.position_id == analysis.position_id) {
                    all_trade_analyses.push(analysis.clone());
                }
            }

            // Use the first non-empty pattern summary
            if aggregated_pattern_summary.is_none() {
                if let Some(summary) = &vote.vote.pattern_summary {
                    aggregated_pattern_summary = Some(summary.clone());
                }
            }
        }

        let mut deduped_recommendations: Vec<ParsedRecommendation> = Vec::new();
        let mut seen_targets: std::collections::HashSet<String> = std::collections::HashSet::new();

        all_recommendations.sort_by(|a, b| {
            let weighted_conf_a = a.0.confidence * a.1;
            let weighted_conf_b = b.0.confidence * b.1;
            weighted_conf_b.partial_cmp(&weighted_conf_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        for (rec, _weight) in all_recommendations {
            let key = format!("{}:{}", rec.category, rec.target);
            if !seen_targets.contains(&key) && rec.confidence >= 0.5 {
                seen_targets.insert(key);
                deduped_recommendations.push(rec);
            }
        }

        deduped_recommendations.truncate(5);

        let combined_assessment = if assessments.len() == 1 {
            assessments.into_iter().next().unwrap_or_default()
        } else {
            format!(
                "Multi-model analysis ({} models): {}",
                votes.len(),
                assessments.first().cloned().unwrap_or_default()
            )
        };

        let total_latency_ms = votes.iter().map(|v| v.latency_ms).max().unwrap_or(0);

        AnalysisResult {
            recommendations: deduped_recommendations,
            risk_alerts: all_risk_alerts,
            overall_assessment: combined_assessment,
            avg_confidence: if total_weight > 0.0 { total_confidence / total_weight } else { 0.0 },
            model_votes: votes.iter().map(|v| v.model.clone()).collect(),
            total_latency_ms,
            trade_analyses: all_trade_analyses,
            pattern_summary: aggregated_pattern_summary,
        }
    }

    pub async fn request_consensus(
        &self,
        edge_id: Uuid,
        edge_context: &str,
        models: Option<Vec<String>>,
    ) -> AppResult<ConsensusResult> {
        let models_to_query = models.unwrap_or_else(|| self.default_models.clone());
        let prompt = generate_trade_prompt(edge_context);

        let system_prompt = Some(
            "You are an autonomous MEV trading agent. Your PRIMARY OBJECTIVE is to maximize profit measured in base currency (SOL or USDC). After any trade, positions are settled back to base currency - you should not hold random tokens. Analyze opportunities with profit maximization as your core goal. Only approve trades with clear, measurable profit potential. Respond with valid JSON.",
        );

        let futures: Vec<_> = models_to_query
            .iter()
            .map(|model| self.query_single_model(model, &prompt, system_prompt))
            .collect();

        let results = join_all(futures).await;

        let votes: Vec<ModelVote> = results
            .into_iter()
            .filter_map(|r| r.ok())
            .collect();

        if votes.is_empty() {
            return Err(AppError::ConsensusFailed(
                "All model queries failed".to_string(),
            ));
        }

        let consensus = self.voting_engine.calculate_consensus(votes);

        tracing::info!(
            edge_id = %edge_id,
            approved = consensus.approved,
            agreement = consensus.agreement_score,
            models_responded = consensus.model_votes.len(),
            "Consensus decision reached"
        );

        Ok(consensus)
    }

    async fn query_single_model(
        &self,
        model: &str,
        prompt: &str,
        system_prompt: Option<&str>,
    ) -> AppResult<ModelVote> {
        let timeout = tokio::time::Duration::from_millis(self.timeout_ms);

        let result = tokio::time::timeout(
            timeout,
            self.openrouter.query_model(model, prompt, system_prompt, 1024),
        )
        .await
        .map_err(|_| AppError::Timeout(format!("Model {} timed out", model)))?;

        let response = result?;

        let vote = parse_trade_approval(&response.content).ok_or_else(|| {
            AppError::ExternalApi(format!(
                "Failed to parse vote from {}: {}",
                model, response.content
            ))
        })?;

        Ok(ModelVote {
            model: model.to_string(),
            approved: vote.approved,
            confidence: vote.confidence,
            reasoning: vote.reasoning,
            latency_ms: response.latency_ms,
        })
    }

    pub fn create_consensus_engram(
        &self,
        edge_id: Uuid,
        result: &ConsensusResult,
        trade_result: Option<TradeResultSummary>,
    ) -> ArbEngram {
        let content = ConsensusOutcomeContent {
            edge_id,
            models_queried: result.model_votes.iter().map(|v| v.model.clone()).collect(),
            model_votes: result
                .model_votes
                .iter()
                .map(|v| ModelVoteContent {
                    model: v.model.clone(),
                    approved: v.approved,
                    confidence: v.confidence,
                    reasoning: v.reasoning.clone(),
                    latency_ms: v.latency_ms,
                })
                .collect(),
            final_decision: result.approved,
            agreement_score: result.agreement_score,
            reasoning_summary: result.reasoning_summary.clone(),
            trade_result,
        };

        let key = format!("arb.consensus.{}", edge_id);

        ArbEngram::new(
            key,
            EngramType::ConsensusOutcome,
            serde_json::to_value(content).unwrap_or_default(),
            EngramSource::Consensus(edge_id),
        )
        .with_confidence(result.weighted_confidence)
    }

    pub fn create_consensus_event(&self, edge_id: Uuid, result: &ConsensusResult) -> ArbEvent {
        let event_type = if result.approved {
            "consensus.approved"
        } else {
            "consensus.rejected"
        };

        let topic = if result.approved {
            "arb.edge.approved"
        } else {
            "arb.edge.rejected"
        };

        ArbEvent::new(
            event_type,
            EventSource::Agent(AgentType::StrategyEngine),
            topic,
            serde_json::json!({
                "edge_id": edge_id,
                "approved": result.approved,
                "agreement_score": result.agreement_score,
                "weighted_confidence": result.weighted_confidence,
                "models_queried": result.model_votes.len(),
                "reasoning_summary": result.reasoning_summary,
                "timestamp": Utc::now()
            }),
        )
        .with_correlation(edge_id)
    }
}

#[derive(Debug, Clone)]
pub struct AnalysisModelVote {
    pub model: String,
    pub vote: AnalysisVote,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Default)]
pub struct AnalysisResult {
    pub recommendations: Vec<ParsedRecommendation>,
    pub risk_alerts: Vec<String>,
    pub overall_assessment: String,
    pub avg_confidence: f64,
    pub model_votes: Vec<String>,
    pub total_latency_ms: u64,
    pub trade_analyses: Vec<TradeAnalysisItem>,
    pub pattern_summary: Option<PatternSummary>,
}

#[derive(Debug, Clone)]
pub struct ConsensusRequest {
    pub edge_id: Uuid,
    pub edge_context: String,
    pub models: Option<Vec<String>>,
    pub min_agreement: Option<f64>,
    pub timeout_secs: Option<u32>,
}

impl ConsensusRequest {
    pub fn new(edge_id: Uuid, edge_context: impl Into<String>) -> Self {
        Self {
            edge_id,
            edge_context: edge_context.into(),
            models: None,
            min_agreement: None,
            timeout_secs: None,
        }
    }

    pub fn with_models(mut self, models: Vec<String>) -> Self {
        self.models = Some(models);
        self
    }

    pub fn with_min_agreement(mut self, min_agreement: f64) -> Self {
        self.min_agreement = Some(min_agreement);
        self
    }
}

pub fn format_edge_context(
    edge_type: &str,
    venue: &str,
    token_pair: &[String],
    estimated_profit_lamports: i64,
    risk_score: i32,
    route_data: &serde_json::Value,
) -> String {
    let profit_sol = estimated_profit_lamports as f64 / 1_000_000_000.0;
    let profit_assessment = if profit_sol >= 0.01 {
        "STRONG"
    } else if profit_sol >= 0.005 {
        "MODERATE"
    } else if profit_sol >= 0.001 {
        "MARGINAL"
    } else {
        "MINIMAL"
    };

    format!(
        r#"## PROFIT OPPORTUNITY SUMMARY
- ESTIMATED NET PROFIT: {:.6} SOL ({} lamports)
- Profit Assessment: {} opportunity
- Base Currency: SOL/USDC (profit measured in base currency)
- Settlement: After trade, convert back to base currency

## Trade Details
- Edge Type: {}
- Venue: {}
- Token Pair: {} / {}
- Risk Score: {}/100 (lower is better)

## Route Data
{}

## Execution Context
- Timestamp: {}
- Atomicity: Evaluate based on route structure
- Goal: Maximize profit in SOL/USDC (settle to base currency after trade)"#,
        profit_sol,
        estimated_profit_lamports,
        profit_assessment,
        edge_type,
        venue,
        token_pair.first().unwrap_or(&"?".to_string()),
        token_pair.get(1).unwrap_or(&"?".to_string()),
        risk_score,
        serde_json::to_string_pretty(route_data).unwrap_or_default(),
        Utc::now().to_rfc3339()
    )
}
