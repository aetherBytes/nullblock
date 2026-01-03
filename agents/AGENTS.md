# NullBlock Agents

This document contains rules, patterns, and notes for the NullBlock agent system.

## Agent System Architecture

### Hecate Agent (Primary Orchestrator)

**Location**: `/svc/nullblock-agents/src/agents/hecate.rs`

**Purpose**: Main conversational interface and task orchestration engine

**Capabilities**:
- Multi-model LLM coordination with automatic failover
- Intent analysis and task delegation
- Real-time chat with streaming support
- Task lifecycle management and tracking
- Image generation via DALL-E 3 integration

**Key Configuration**:
- Default model: `cognitivecomputations/dolphin3.0-mistral-24b:free` (can be overridden via `DEFAULT_LLM_MODEL` env var)
- Timeout: 5 minutes for thinking models
- Max tokens: 16384 (required for base64 image responses)
- Fallback chain: Automatically tries alternative free models if default fails

### Specialized Agents

**Siren Marketing Agent** (`/svc/nullblock-agents/src/agents/siren_marketing.rs`)
- Content generation for social media
- Twitter post creation
- Project progress analysis
- Marketing content themes

## LLM Provider System

### Factory Pattern

**Location**: `/svc/nullblock-agents/src/llm/factory.rs`

The `LLMFactory` manages all LLM provider connections and routing logic.

**Providers Supported**:
- OpenRouter (primary cloud aggregator)
- OpenAI (direct)
- Anthropic (direct)
- Groq (direct)
- HuggingFace (direct)

**Model Selection Strategies**:
- `Quality`: Prioritizes model quality (GPT-4, Claude)
- `Speed`: Fast models (Groq, small models)
- `Cost`: Free or cheapest models
- `Balanced`: Weighted combination

### Model Routing Algorithm

**Scoring System** (see `factory.rs:234-282`):

```
Total Score = base_score + quality_bonus + reliability_bonus + cost_optimization + tier_bonus + provider_bonus - frequency_penalty
```

**Factors**:
- **Quality** (40 points max): Based on model capabilities and performance
- **Reliability** (30 points): Historical success rate and error frequency
- **Cost Optimization**: Penalty for paid models, bonus for free
- **Tier Bonus**: Enterprise > Pro > Standard > Free
- **Provider Diversity**: Small bonus for less-used providers
- **Usage Frequency Penalty**: Prevents over-reliance on single model

### Startup Validation Flow

1. Load API keys from environment (`.env.dev`)
2. Validate API keys (detect placeholders like `your-openrouter-key-here`)
3. Initialize provider connections
4. Test default model with simple query: "What is 2+2? Respond with only the number."
5. On failure, fetch live free models from OpenRouter
6. Sort fallbacks by context window size (larger = better for complex tasks)
7. Cache validated models in memory

### Runtime Fallback Chain

When a model request fails:

1. Log the error with model details
2. Check if response is empty (0 completion tokens)
3. Skip to next fallback model in chain
4. Retry up to configured max fallbacks
5. Return descriptive error if all fallbacks exhausted

**Empty Response Detection** (`factory.rs:161-164`):
```rust
if r.content.trim().is_empty() {
    warn!("⚠️ Fallback model {} returned empty response, trying next fallback", fallback_model);
    continue;
}
```

## API Key Management

### Environment Configuration

**CRITICAL**: All services require `.env.dev` file access for API keys.

**Required Keys**:
- `OPENROUTER_API_KEY`: Primary LLM provider (REQUIRED for reliable access)
- `OPENAI_API_KEY`: Optional for direct OpenAI access
- `ANTHROPIC_API_KEY`: Optional for direct Anthropic access
- `GROQ_API_KEY`: Optional for Groq models

### Symlink Approach

Services running in subdirectories (e.g., `/svc/nullblock-agents/`) need local `.env.dev` access:

```bash
# Create symlinks from service directories to project root
ln -s ../../.env.dev /svc/nullblock-agents/.env.dev
ln -s ../../.env.dev /svc/erebus/.env.dev
```

**Why**: Services use `dotenv::from_filename(".env.dev")` which searches in current working directory.

### API Key Validation

**Location**: `/svc/nullblock-agents/src/llm/factory.rs:85-105`

The factory validates API keys at startup:

```rust
// Detect invalid/placeholder keys
if api_key.is_empty() || api_key == "your-openrouter-key-here" {
    error!("🔑 CRITICAL: OpenRouter API key is invalid or placeholder!");
    error!("   Set OPENROUTER_API_KEY in .env.dev to a valid key from https://openrouter.ai/");
    error!("   Without a valid OpenRouter key, you'll hit severe rate limits on free models.");
}
```

**What it checks**:
- Empty keys
- Placeholder values
- Key format (logs partial key for verification: `sk-or-v1-49a74f...7c7a`)

### Anonymous Access Detection

**Location**: `/svc/nullblock-agents/src/llm/providers.rs:598-610`

OpenRouter returns a `user_id` field in API responses. If the key isn't configured properly:

```rust
if let Some(user_id) = error_json.get("user_id").and_then(|u| u.as_str()) {
    if user_id.starts_with("user_") && status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        error!("🔑 CRITICAL: OpenRouter is using anonymous/free tier access!");
        error!("   Your API key is NOT being sent or loaded properly.");
        error!("   Check that OPENROUTER_API_KEY is set in .env.dev");
        error!("   Anonymous user detected: {}", user_id);
    }
}
```

**Anonymous user ID format**: `user_31zWQ2TWoTbgXeEG32fC65c8YFK`
**Authenticated user ID format**: Matches your OpenRouter account ID

## Task Integration

### User Reference Creation

**CRITICAL FIX** (`svc/erebus/src/resources/agents/routes.rs:38-42`):

When creating tasks that require user references, the `source_type` object MUST include a `network` field:

```rust
let default_source_type = serde_json::json!({
    "type": "web3_wallet",
    "provider": "unknown",
    "network": wallet_chain,  // ← REQUIRED
    "metadata": {}
});
```

**Previous bug**: Missing `network` field caused deserialization errors: `"Failed to deserialize the JSON body into the target type: source_type: missing field 'network'"`

### Task Processing Flow

1. Frontend creates task via Erebus → POST `/api/agents/tasks`
2. Erebus validates user reference (auto-creates if needed)
3. Task stored in Agents DB with A2A-compliant schema
4. Hecate agent assigned for processing
5. Agent processes task and updates `history` + `artifacts` arrays
6. Kafka event published to `task.lifecycle` topic
7. Frontend receives completion via polling or SSE (future)

### A2A Task Schema

Tasks conform to [A2A Protocol v0.3.0](https://a2a-protocol.org/latest/specification/):

**Key fields**:
- `context_id`: Groups related tasks
- `kind`: Always "task"
- `status`: Object with `{state, message?, timestamp?}`
- `history`: JSONB array of messages (user + agent responses)
- `artifacts`: JSONB array of task outputs (completion results, images, files)

**States**: submitted, working, input-required, completed, canceled, failed, rejected, auth-required, unknown

**Current issue**: A2A uses "working" state but Hecate expects "created"/"running" - reconciliation needed.

## Error Handling Rules

### Logging Levels

- `error!`: Critical failures (missing API keys, invalid config, request failures)
- `warn!`: Degraded functionality (empty responses, fallback triggered, rate limits)
- `info!`: Normal operations (model selected, request successful, startup complete)
- `debug!`: Detailed diagnostics (request payloads, response bodies)

### User-Facing Error Messages

**DO**:
- Provide clear, actionable error messages
- Include specific configuration steps
- Log partial API keys for verification
- Detect and explain anonymous access vs authenticated access
- Show which model failed and why

**DON'T**:
- Return empty responses silently
- Show generic "error occurred" messages
- Expose full API keys in logs
- Leave users guessing about configuration issues

### Example Error Patterns

**Empty Response** (`factory.rs:161`):
```rust
if r.content.trim().is_empty() {
    warn!("⚠️ Fallback model {} returned empty response, trying next fallback", fallback_model);
    continue;
}
```

**Missing API Key** (`factory.rs:89`):
```rust
error!("🔑 CRITICAL: OpenRouter API key is missing!");
error!("   Set OPENROUTER_API_KEY in .env.dev with a key from https://openrouter.ai/");
error!("   Free tier has very strict rate limits - you need your own key for reliable access.");
```

**Anonymous Access** (`providers.rs:603`):
```rust
return Err(AppError::LLMRequestFailed(format!(
    "OpenRouter API key not configured properly. Using anonymous access (user: {}) which has strict rate limits. Set OPENROUTER_API_KEY in .env.dev",
    user_id
)));
```

## Model-Specific Notes

### Default Model Selection

**Hard-coded default**: `cognitivecomputations/dolphin3.0-mistral-24b:free`

**Override via environment**:
```bash
DEFAULT_LLM_MODEL=x-ai/grok-4-fast:free
```

**Validation**: On startup, factory sends test query to default model. If it fails, fetches live free models from OpenRouter API and sorts by context window.

### Image Generation

**Provider**: OpenAI DALL-E 3 (requires `OPENAI_API_KEY`)

**Current Issues**:
1. `useChat.ts parseContentForImages()` removes image markdown from content string
2. `max_tokens` too low (4096) for base64 images - needs 16384+
3. No error handling for truncated/timeout responses

**Fix Required**:
- Keep images in markdown for MarkdownRenderer
- Increase token limit in image generation requests
- Add validation and timeout handling

### Model Timeout Configuration

```bash
LLM_REQUEST_TIMEOUT_MS=300000  # 5 minutes for thinking models
```

Used for models like DeepSeek R1, o1-preview that require extended processing time.

## Agent Communication Patterns

### Erebus Router Pattern

**GOLDEN RULE**: ALL frontend communication MUST route through Erebus (port 3000).

```
Frontend → Erebus → {
  Agent chat → Hecate (9003)
  A2A/MCP → Protocols (8001)
  Marketplace → Crossroads (internal)
}
```

**Agents service endpoints** (port 9003):
- `/hecate/chat` - Chat with Hecate agent
- `/hecate/set-model` - Change active model
- `/hecate/available-models` - List available models
- `/tasks/*` - Task CRUD operations
- `/siren/chat` - Chat with Siren marketing agent

### Inter-Service Communication

Services communicate via HTTP JSON APIs:

**Erebus → Agents**: `http://localhost:9003/hecate/chat`
**Protocols → Agents**: `http://localhost:9003/tasks/*`

**Container-to-Container** (future):
- Use container names: `http://nullblock-agents:9003`
- Not host networking: `http://localhost:9003`

## Testing & Debugging

### Model Availability Test

```bash
curl http://localhost:9003/hecate/available-models
```

Returns list of free models with context window sizes.

### Chat Test

```bash
curl -X POST http://localhost:3000/api/agents/chat \
  -H "Content-Type: application/json" \
  -d '{"message": "Hello", "wallet_address": "test", "wallet_chain": "ethereum"}'
```

Should return agent response with model metadata.

### Log Monitoring

```bash
# Hecate chat logs
tail -f svc/nullblock-agents/logs/chats/hecate-chat.log

# Service logs
tail -f svc/nullblock-agents/logs/nullblock-agents.log

# Erebus logs
tail -f svc/erebus/logs/erebus.log
```

### API Key Verification

Check startup logs for API key confirmation:

```
✅ OpenRouter is configured as the cloud model aggregator (key: sk-or-v1-49a74f...7c7a)
```

If you see `user_31zWQ2TWoTbgXeEG32fC65c8YFK` in error logs, API key is NOT loaded properly.

## Future Improvements

### Pending Tasks

1. **Task State Alignment**: Reconcile A2A "working" state with Hecate "running"/"created" states
2. **Auto-Processing**: Implement automatic task processing when `auto_start=true`
3. **SSE Streaming**: Real-time task updates via Server-Sent Events
4. **Image Generation Fix**: Resolve markdown parsing and token limit issues
5. **Model Persistence**: Remember user's model selection across sessions (Siren should follow Hecate selection)

### Architectural Goals

- **Agent-to-Agent (A2A) Protocol**: Full compliance with v0.3.0 spec
- **Kafka Event Streaming**: Bridge task lifecycle events to SSE for real-time frontend updates
- **Push Notifications**: Webhook system for task status updates
- **Enhanced Security**: API key rotation, rate limiting per client, request signing

---

**For detailed implementation notes, see**:
- `/svc/nullblock-agents/src/llm/factory.rs` - LLM factory and routing
- `/svc/nullblock-agents/src/llm/providers.rs` - OpenRouter provider implementation
- `/svc/nullblock-agents/src/agents/hecate.rs` - Hecate agent logic
- `/svc/nullblock-agents/src/handlers/tasks.rs` - Task management handlers
- `CLAUDE.md` - Project-wide architecture and conventions
