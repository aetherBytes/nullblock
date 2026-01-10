# Agents Service

**The AI brain** - Hosts HECATE and all specialized agents.

## Overview

| Property | Value |
|----------|-------|
| **Port** | 9003 |
| **Location** | `/svc/nullblock-agents/` |
| **Database** | PostgreSQL (port 5441) |
| **Role** | Agent orchestration, LLM routing, task processing |

## Agents

### HECATE (Primary)

**HECATE** = Harmonic Exploration Companion & Autonomous Threshold Entity

| Property | Value |
|----------|-------|
| **Identity** | Von Neumann-class vessel AI in MK1 hull |
| **Purpose** | Exploration companion for the agent mesh |
| **Voice** | Calm authority with dry wit |
| **Default Model** | `cognitivecomputations/dolphin3.0-mistral-24b:free` |
| **Timeout** | 5 minutes (for thinking models) |
| **Max Tokens** | 16384 |

### Siren Marketing

| Property | Value |
|----------|-------|
| **Purpose** | Content generation, social media |
| **Capabilities** | Twitter posts, project analysis, marketing themes |

## API Endpoints

```bash
# Chat
POST /hecate/chat              # Chat with HECATE
POST /siren/chat               # Chat with Siren

# Model Management
POST /hecate/set-model         # Change active model
GET  /hecate/available-models  # List available models

# Tasks
GET  /tasks                    # List all tasks
POST /tasks                    # Create task
GET  /tasks/:id                # Get task
POST /tasks/:id/process        # Process with agent
```

## LLM Factory

**Location**: `/svc/nullblock-agents/src/llm/factory.rs`

### Providers

- **OpenRouter** (primary cloud aggregator)
- **OpenAI** (direct)
- **Anthropic** (direct)
- **Groq** (direct)
- **HuggingFace** (direct)

### Model Selection

```
Score = base_score + quality_bonus + reliability_bonus
        + cost_optimization + tier_bonus - frequency_penalty
```

| Factor | Weight |
|--------|--------|
| Quality | 40 pts |
| Reliability | 30 pts |
| Cost | Variable |
| Tier | Bonus |

### Fallback Chain

1. Try default model
2. On failure, fetch live free models from OpenRouter
3. Sort by context window size
4. Try each until success or exhaustion

## API Key Management

Keys stored in Erebus database (`agent_api_keys` table), not in `.env.dev`.

```bash
# Seed keys
cd svc/erebus && cargo run --bin seed_agent_keys
```

**Fetch flow**:
```
Startup → GET /internal/agents/:agent/api-keys/:provider/decrypted → Cache in memory
```

## Task Processing

### A2A Protocol States

`submitted` → `working` → `completed`

Other states: `input-required`, `canceled`, `failed`, `rejected`, `auth-required`

### Task Schema

```json
{
  "id": "uuid",
  "context_id": "uuid",
  "kind": "task",
  "status": {
    "state": "working",
    "message": "Processing...",
    "timestamp": "2025-01-10T..."
  },
  "history": [...],
  "artifacts": [...]
}
```

## Configuration

```bash
DATABASE_URL=postgresql://postgres:REDACTED_DB_PASS@localhost:5441/agents
EREBUS_BASE_URL=http://localhost:3000
DEFAULT_LLM_MODEL=cognitivecomputations/dolphin3.0-mistral-24b:free
LLM_REQUEST_TIMEOUT_MS=300000
```

## Related

- [HECATE Agent](../agents/hecate.md)
- [Siren Marketing](../agents/siren.md)
- [LLM Factory](../agents/llm-factory.md)
