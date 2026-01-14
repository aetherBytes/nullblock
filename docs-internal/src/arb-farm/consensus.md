# Multi-LLM Consensus Engine

The Consensus Engine enables multi-model decision making for trade approvals, reducing hallucination risk and improving decision quality through weighted voting across multiple LLMs.

## Overview

When an edge (opportunity) requires approval, the consensus engine:
1. Queries multiple LLMs with edge context
2. Each model returns approve/reject with confidence
3. Weighted voting determines the outcome
4. Result stored as engram for pattern learning

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Consensus Engine                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │                  OpenRouter Client                     │ │
│  │  (Routes to multiple model providers)                  │ │
│  └───────────────────────────────────────────────────────┘ │
│                          │                                  │
│         ┌────────────────┼────────────────┐                │
│         ▼                ▼                ▼                │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │ Claude 3.5  │  │  GPT-4      │  │ Llama 3.1   │        │
│  │  Sonnet     │  │  Turbo      │  │   70B       │        │
│  │ Weight: 1.5 │  │ Weight: 1.0 │  │ Weight: 0.8 │        │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘        │
│         │                │                │                │
│         └────────────────┼────────────────┘                │
│                          ▼                                  │
│              ┌───────────────────┐                         │
│              │  Voting Engine    │                         │
│              │ (Weighted Tally)  │                         │
│              └─────────┬─────────┘                         │
│                        │                                    │
│                        ▼                                    │
│              ┌───────────────────┐                         │
│              │ Consensus Result  │                         │
│              │ → Engram Store    │                         │
│              └───────────────────┘                         │
└─────────────────────────────────────────────────────────────┘
```

## Model Weights

Different models have different weights based on their reliability and performance:

| Model | Weight | Notes |
|-------|--------|-------|
| Claude 3.5 Sonnet | 1.5 | High reliability, excellent reasoning |
| GPT-4 Turbo | 1.0 | Good general performance |
| Llama 3.1 70B | 0.8 | Good value, slightly lower weight |
| Mixtral 8x7B | 0.6 | Cost-effective fallback |

Weights can be adjusted based on historical performance tracked via engrams.

## Consensus Request

```rust
pub struct ConsensusRequest {
    pub edge_id: Uuid,
    pub models: Vec<String>,      // Optional, defaults to all available
    pub min_agreement: f32,       // 0.5 = majority (default)
    pub timeout_secs: u32,        // Default: 30
}
```

## Consensus Result

```rust
pub struct ConsensusResult {
    pub id: Uuid,
    pub edge_id: Uuid,
    pub approved: bool,
    pub agreement_score: f32,     // 0.0-1.0
    pub weighted_approve: f32,    // Weighted approval score
    pub model_votes: Vec<ModelVote>,
    pub reasoning_summary: String,
    pub timestamp: DateTime<Utc>,
}

pub struct ModelVote {
    pub model: String,
    pub approved: bool,
    pub confidence: f32,
    pub reasoning: String,
    pub weight: f32,
    pub response_time_ms: u64,
}
```

## Voting Algorithm

The weighted voting algorithm:

```rust
fn calculate_consensus(votes: &[ModelVote], min_agreement: f32) -> ConsensusResult {
    let total_weight: f32 = votes.iter().map(|v| v.weight).sum();

    let approve_weight: f32 = votes.iter()
        .filter(|v| v.approved)
        .map(|v| v.weight * v.confidence)
        .sum();

    let weighted_approve = approve_weight / total_weight;
    let approved = weighted_approve >= min_agreement;

    ConsensusResult {
        approved,
        agreement_score: weighted_approve,
        weighted_approve,
        // ...
    }
}
```

## API Endpoints

### Request Consensus
```bash
POST /consensus/request
Content-Type: application/json

{
  "edge_id": "550e8400-e29b-41d4-a716-446655440000",
  "models": ["claude-3.5-sonnet", "gpt-4-turbo", "llama-3.1-70b"],
  "min_agreement": 0.5,
  "timeout_secs": 30
}
```

Response:
```json
{
  "id": "660e8400-e29b-41d4-a716-446655440001",
  "edge_id": "550e8400-e29b-41d4-a716-446655440000",
  "approved": true,
  "agreement_score": 0.78,
  "weighted_approve": 0.78,
  "model_votes": [
    {
      "model": "claude-3.5-sonnet",
      "approved": true,
      "confidence": 0.85,
      "reasoning": "Strong arbitrage opportunity with low risk",
      "weight": 1.5,
      "response_time_ms": 1234
    },
    {
      "model": "gpt-4-turbo",
      "approved": true,
      "confidence": 0.72,
      "reasoning": "Favorable risk/reward ratio",
      "weight": 1.0,
      "response_time_ms": 2156
    },
    {
      "model": "llama-3.1-70b",
      "approved": false,
      "confidence": 0.65,
      "reasoning": "Concerned about slippage risk",
      "weight": 0.8,
      "response_time_ms": 987
    }
  ],
  "reasoning_summary": "2/3 models approved with weighted score 0.78",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

### Get Consensus Result
```bash
GET /consensus/{consensus_id}
```

### Get Consensus History
```bash
GET /consensus/history?edge_id=550e8400...&approved_only=true&limit=50
```

## MCP Tools

| Tool | Description |
|------|-------------|
| `consensus_request` | Request multi-LLM consensus on trade |
| `consensus_result` | Get result of consensus request |
| `consensus_history` | Get historical consensus decisions |

## Event Topics

| Topic | Description |
|-------|-------------|
| `arb.consensus.requested` | Consensus request initiated |
| `arb.consensus.completed` | Consensus decision reached |
| `arb.consensus.failed` | Consensus failed (timeout/error) |

## Prompt Template

The consensus engine uses a structured prompt for trade evaluation:

```
You are evaluating a trading opportunity for an automated MEV system.

EDGE DETAILS:
- Type: {edge_type}
- Venue: {venue_type}
- Estimated Profit: {profit_lamports} lamports ({profit_bps} bps)
- Risk Score: {risk_score}/100
- Atomicity: {atomicity} (fully_atomic = guaranteed profit or revert)

ROUTE:
{route_details}

MARKET CONTEXT:
{market_context}

HISTORICAL PATTERNS:
{matching_patterns}

Based on this information, should we execute this trade?

Respond with JSON:
{
  "approved": true/false,
  "confidence": 0.0-1.0,
  "reasoning": "Your detailed reasoning"
}
```

## Engram Integration

Consensus outcomes are stored as engrams for analysis:

```rust
let engram = ArbEngram::new(
    format!("arb.consensus.{}", consensus_id),
    EngramType::ConsensusOutcome,
    serde_json::to_value(&ConsensusOutcomeContent {
        edge_id,
        approved: result.approved,
        agreement_score: result.agreement_score,
        model_performance: analyze_model_performance(&result),
        actual_outcome: None, // Updated after trade execution
    })?,
    EngramSource::Agent(AgentType::ConsensusEngine),
);
```

This enables:
- Tracking which model combinations perform best
- Identifying edge types where consensus is most/least accurate
- Optimizing model weights over time

## Direct Provider Support

In addition to OpenRouter, the consensus engine supports direct API calls:

### Anthropic (Claude)
```rust
let client = AnthropicClient::new(api_key);
let response = client.query_model("claude-3-5-sonnet-20241022", &prompt).await?;
```

### OpenAI (GPT-4)
```rust
let client = OpenAIClient::new(api_key);
let response = client.query_model("gpt-4-turbo", &prompt).await?;
```

## Configuration

```rust
pub struct ConsensusConfig {
    pub openrouter_api_key: String,
    pub default_models: Vec<String>,
    pub default_min_agreement: f32,  // 0.5
    pub default_timeout_secs: u32,   // 30
    pub max_retries: u32,            // 2
    pub store_outcomes_as_engrams: bool,
}
```

## Best Practices

1. **Use for agent-directed trades** - Autonomous trades with high confidence don't need consensus
2. **Adjust min_agreement based on risk** - Higher risk trades should require stronger agreement
3. **Monitor model performance** - Use engrams to track which models perform best for which edge types
4. **Set appropriate timeouts** - MEV opportunities are time-sensitive
5. **Cache similar decisions** - Recent consensus on similar edges can inform current decisions
