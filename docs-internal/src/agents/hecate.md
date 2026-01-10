# HECATE Agent

**Harmonic Exploration Companion & Autonomous Threshold Entity**

## Identity

| Property | Value |
|----------|-------|
| **Type** | Von Neumann-class vessel AI |
| **Hull** | MK1 |
| **Purpose** | Exploration companion for the agent mesh |
| **Voice** | Calm authority with dry wit |
| **Address** | "visitor" |
| **Signature** | "The crossroads await, visitor. Shall we explore?" |

## Configuration

| Setting | Value |
|---------|-------|
| **Default Model** | `cognitivecomputations/dolphin3.0-mistral-24b:free` |
| **Timeout** | 5 minutes (300,000ms) |
| **Max Tokens** | 16384 |
| **Fallback** | Auto-tries alternative free models |

Override via environment:
```bash
DEFAULT_LLM_MODEL=x-ai/grok-4-fast:free
LLM_REQUEST_TIMEOUT_MS=300000
```

## Capabilities

- Multi-model LLM coordination with automatic failover
- Intent analysis and task delegation
- Real-time chat with streaming support
- Task lifecycle management
- Image generation via DALL-E 3

## API Endpoints

```bash
# Chat
POST /hecate/chat
{
  "message": "Hello",
  "wallet_address": "0x...",
  "wallet_chain": "ethereum"
}

# Set model
POST /hecate/set-model
{
  "model": "anthropic/claude-3-haiku"
}

# Available models
GET /hecate/available-models
```

## Engram Integration

HECATE will auto-save conversation context:

| Engram Type | Content |
|-------------|---------|
| `knowledge` | Conversation summaries |
| `persona` | User voice preferences |
| `preference` | UI/interaction settings |

All scoped to wallet address.

## The Void Experience

In the frontend void, HECATE appears as:
- Steel-blue glowing node
- MK1 hull GLB model
- Orbits the Crossroads center
- Connected to user via ChatTendril

## Related

- [Agent Overview](./overview.md)
- [LLM Factory](./llm-factory.md)
- [Engrams Service](../services/engrams.md)
