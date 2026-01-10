# Agent Architecture

Overview of the NullBlock agent system.

## Philosophy

Agents in NullBlock are autonomous AI entities that can:
- Process natural language requests
- Execute tasks across multiple LLM providers
- Persist context via Engrams
- Communicate via A2A protocol

## Agent Types

| Agent | Purpose | Location |
|-------|---------|----------|
| **HECATE** | Primary orchestrator, exploration companion | `/svc/nullblock-agents/src/agents/hecate.rs` |
| **Siren** | Marketing content, social media | `/svc/nullblock-agents/src/agents/siren_marketing.rs` |

## Chat State

**CRITICAL**: Chat is ephemeral - no persistence across sessions.

- Messages stored in React state only
- Clears on page refresh
- Clears on wallet disconnect
- Different users never see each other's history

### Context Persistence

For persistent memory, use **Engrams**:
- User-specific context linked to wallet
- Cross-session knowledge retention
- Versioned, forkable memory units

## LLM Provider System

### Factory Pattern

The `LLMFactory` manages all provider connections:

- **OpenRouter** (primary aggregator)
- **OpenAI** (direct)
- **Anthropic** (direct)
- **Groq** (direct)
- **HuggingFace** (direct)

### Model Selection

```
Score = base + quality + reliability + cost_opt + tier - frequency
```

### Fallback Chain

1. Try configured default model
2. On failure, fetch live free models
3. Sort by context window
4. Try each until success

## API Key Management

Keys stored in Erebus database, not `.env.dev`:

```bash
# Seed keys
cd svc/erebus && cargo run --bin seed_agent_keys
```

## Task Integration

Tasks follow A2A Protocol v0.3.0:

```
submitted → working → completed
```

See [Protocols Service](../services/protocols.md) for full A2A details.

## Related

- [HECATE Agent](./hecate.md)
- [Siren Marketing](./siren.md)
- [LLM Factory](./llm-factory.md)
- [Engrams Service](../services/engrams.md)
