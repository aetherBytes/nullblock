use chrono::Utc;
use futures::future::join_all;
use uuid::Uuid;

use super::{
    openrouter::{get_default_models, OpenRouterClient},
    voting::{generate_trade_prompt, parse_trade_approval, ConsensusResult, ModelVote, VotingEngine},
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

    pub async fn request_consensus(
        &self,
        edge_id: Uuid,
        edge_context: &str,
        models: Option<Vec<String>>,
    ) -> AppResult<ConsensusResult> {
        let models_to_query = models.unwrap_or_else(|| self.default_models.clone());
        let prompt = generate_trade_prompt(edge_context);

        let system_prompt = Some(
            "You are an expert MEV trader. Analyze opportunities carefully and respond with valid JSON.",
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
    format!(
        r#"Edge Type: {}
Venue: {}
Token Pair: {} / {}
Estimated Profit: {} lamports ({:.4} SOL)
Risk Score: {}/100

Route Data:
{}

Additional Context:
- Current timestamp: {}
- Atomicity: Evaluate based on route structure"#,
        edge_type,
        venue,
        token_pair.first().unwrap_or(&"?".to_string()),
        token_pair.get(1).unwrap_or(&"?".to_string()),
        estimated_profit_lamports,
        estimated_profit_lamports as f64 / 1_000_000_000.0,
        risk_score,
        serde_json::to_string_pretty(route_data).unwrap_or_default(),
        Utc::now().to_rfc3339()
    )
}
