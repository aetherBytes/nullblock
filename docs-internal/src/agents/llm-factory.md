# LLM Factory

**The brain router** - Manages all LLM provider connections and model selection.

## Location

`/svc/nullblock-agents/src/llm/factory.rs`

## Providers

| Provider | Type | Notes |
|----------|------|-------|
| **OpenRouter** | Aggregator | Primary, 100+ models |
| **OpenAI** | Direct | GPT-4, DALL-E |
| **Anthropic** | Direct | Claude models |
| **Groq** | Direct | Fast inference |
| **HuggingFace** | Direct | Open models |

## Model Selection Strategies

| Strategy | Use Case |
|----------|----------|
| `Quality` | Best output (GPT-4, Claude) |
| `Speed` | Fast responses (Groq) |
| `Cost` | Free/cheap models |
| `Balanced` | Weighted combination |

## Scoring Algorithm

```
Total = base_score + quality_bonus + reliability_bonus
        + cost_optimization + tier_bonus + provider_bonus
        - frequency_penalty
```

| Factor | Max Points |
|--------|------------|
| Quality | 40 |
| Reliability | 30 |
| Cost Opt | Variable |
| Tier Bonus | 10 |

## Startup Flow

1. Load API keys from Erebus database
2. Validate keys (detect placeholders)
3. Initialize provider connections
4. Test default model: "What is 2+2?"
5. On failure, fetch live free models
6. Sort by context window size
7. Cache validated models

## Fallback Chain

When a request fails:

1. Log error with model details
2. Check for empty response (0 tokens)
3. Skip to next fallback
4. Retry up to max fallbacks
5. Return descriptive error if exhausted

```rust
if r.content.trim().is_empty() {
    warn!("⚠️ Fallback model {} returned empty response", model);
    continue;
}
```

## API Key Storage

Keys in Erebus database (`agent_api_keys` table):

```sql
SELECT agent_name, provider, key_prefix, key_suffix, is_active
FROM agent_api_keys;
-- Example: hecate | openrouter | sk-or-v1-8 | e098 | t
```

**Seed keys**:
```bash
cd svc/erebus && cargo run --bin seed_agent_keys
```

## Anonymous Access Detection

OpenRouter returns `user_id` in responses:
- **Authenticated**: Your account ID
- **Anonymous**: `user_31zWQ2TWoTbgXeEG32fC65c8YFK`

If you see anonymous IDs, your API key isn't loading properly.

## Related

- [Agent Overview](./overview.md)
- [HECATE Agent](./hecate.md)
