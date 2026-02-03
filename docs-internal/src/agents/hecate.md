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
- **Function calling** (provider-agnostic: OpenRouter, Anthropic, Groq, OpenAI)
- **New user detection** — auto-detects empty profiles, greets warmly, learns about the visitor conversationally
- **Auto-remembering** — proactively saves important context via `hecate_remember` tool
- **Chat session persistence** — saves sessions to engrams every 10 messages and before clear
- **Per-user API keys** — users with their own provider keys bypass free-tier limits
- **Conversational model switching** — users say "use Opus" and Hecate finds the best match
- Intent analysis and task delegation
- Real-time chat with streaming support
- Task lifecycle management
- Image generation via DALL-E 3
- Pin/unpin engrams and cleanup old sessions

## Function Calling

Hecate uses provider-agnostic function calling. Tool definitions are in OpenAI format; each provider translates as needed (Anthropic uses `input_schema`/`tool_use`). Tools are only sent when a wallet is connected and the request is not an image generation.

### Available Function-Calling Tools

| Tool | Description |
|------|-------------|
| `user_profile_update` | Save/update profile fields (display_name, bio, interests, experience) |
| `hecate_remember` | Proactively save important context (preferences, facts, decisions) |
| `hecate_cleanup` | Compact old conversation sessions (respects pinned protection) |
| `hecate_set_model` | Switch LLM model conversationally ("use Opus", "switch to Claude") |
| `hecate_pin_engram` | Pin an engram to protect it from cleanup/deletion |
| `hecate_unpin_engram` | Remove pin protection from an engram |

### Tool Execution Loop

1. LLM response includes `tool_calls` -> parse each call
2. Auto-inject `wallet_address` into arguments (not exposed to LLM)
3. Execute via `mcp::handlers::execute_tool` (or special-case for `hecate_set_model`)
4. Collect results, re-prompt LLM with results in system prompt (tools=None to prevent loops)
5. Return final response to user

## New User Detection

`PersonaLoadResult` enum detects whether a wallet has meaningful profile data:

- **`Existing(String)`** — has real persona data -> inject as USER PERSONA CONTEXT
- **`NewUser`** — only bare defaults (null fields, title="Visitor") -> inject NEW_USER_SYSTEM_PROMPT

The new-user prompt instructs Hecate to learn about the visitor naturally through conversation and save findings via function calling tools.

## Per-User API Keys

Users can store their own API keys (Anthropic, OpenAI, OpenRouter, Groq) via the Erebus API. When a user has keys:

1. Keys are fetched via `get_user_decrypted_keys()` at chat time
2. Injected into `user_context` as `user_api_key_{provider}`
3. `generate_with_key()` creates a temporary provider instance with the user's key
4. Request executes with user's key, then the temporary provider is discarded

## API Endpoints

```bash
# Chat (now with function calling)
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

# Clear conversation (now accepts optional wallet_address to save session before wipe)
POST /hecate/clear
{
  "wallet_address": "0x..."
}

# MCP tools
GET /hecate/tools
POST /mcp/jsonrpc  # Execute any MCP tool via JSON-RPC
```

### User API Key Management (via Erebus)

```bash
GET  /api/users/:user_id/api-keys          # List keys (masked)
POST /api/users/:user_id/api-keys          # Add key
DELETE /api/users/:user_id/api-keys/:key_id # Delete key
```

## Engram Integration

HECATE auto-saves conversation context and proactively remembers important information:

| Engram Type | Content |
|-------------|---------|
| `persona` | User profile (display_name, bio, title, experience, interests) |
| `knowledge` | Facts and context Hecate remembers via `hecate_remember` |
| `preference` | UI/interaction settings |
| `conversation` | Chat session snapshots (saved every 10 messages and before clear) |

All scoped to wallet address. Pinned engrams are protected from deletion and cleanup.

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
