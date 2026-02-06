# MCP Tools Reference

Complete reference for all MCP tools in the NullBlock ecosystem. Tools are organized by service and filtered by allow lists for security.

## Tool Allow Lists

Tools are exposed differently depending on the context:

| Context | Description | Tool Count |
|---------|-------------|------------|
| **Agent** | Tools available to Hecate/Moros agents | 6 |
| **Crossroads** | Tools exposed to marketplace/external | 0 (pending verification) |
| **Internal** | All tools (dev/admin only) | 21 |

### Agent Allow List (Currently Enabled)

These 6 engram CRUD tools are available to Hecate and Moros agents:

```
engram_create, engram_get, engram_search, engram_update, engram_delete, engram_list_by_type
```

### Crossroads Allow List (Pending)

No tools currently exposed to Crossroads marketplace. Tools will be added after security verification.

---

## Engram Tools (6)

Core memory primitives. Available to agents.

### engram_create

Create a new engram (memory unit) for a wallet.

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `wallet_address` | string | Yes | Wallet address that owns this engram |
| `engram_type` | enum | Yes | `persona`, `preference`, `strategy`, `knowledge`, `compliance` |
| `key` | string | Yes | Unique key (e.g., `user.profile.base`) |
| `content` | string | Yes | JSON string content |
| `tags` | string[] | No | Tags for categorization |
| `is_public` | boolean | No | Whether publicly visible |

**Annotation**: Write operation

**Example**:
```json
{
  "name": "engram_create",
  "arguments": {
    "wallet_address": "0x1234...",
    "engram_type": "preference",
    "key": "trading.risk_tolerance",
    "content": "{\"level\": \"medium\", \"max_position\": 0.1}",
    "tags": ["trading", "risk"]
  }
}
```

---

### engram_get

Get a specific engram by wallet address and key.

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `wallet_address` | string | Yes | Wallet address that owns the engram |
| `key` | string | Yes | Engram key to look up |

**Annotation**: Read-only, idempotent

**Example**:
```json
{
  "name": "engram_get",
  "arguments": {
    "wallet_address": "0x1234...",
    "key": "trading.risk_tolerance"
  }
}
```

---

### engram_search

Search engrams with filters by wallet, type, tags, and query.

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `wallet_address` | string | No | Filter by wallet address |
| `engram_type` | enum | No | Filter by type |
| `tags` | string[] | No | Filter by tags |
| `limit` | integer | No | Max results (default 20) |
| `offset` | integer | No | Pagination offset |

**Annotation**: Read-only, idempotent

**Example**:
```json
{
  "name": "engram_search",
  "arguments": {
    "wallet_address": "0x1234...",
    "engram_type": "preference",
    "tags": ["trading"],
    "limit": 10
  }
}
```

---

### engram_update

Update an existing engram's content and/or tags by its ID.

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `id` | string | Yes | Engram ID to update |
| `content` | string | Yes | New JSON string content |
| `tags` | string[] | No | New tags (replaces existing) |

**Annotation**: Write operation

**Example**:
```json
{
  "name": "engram_update",
  "arguments": {
    "id": "eng_abc123",
    "content": "{\"level\": \"high\", \"max_position\": 0.2}",
    "tags": ["trading", "risk", "updated"]
  }
}
```

---

### engram_delete

Delete an engram by its ID. This action is permanent.

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `id` | string | Yes | Engram ID to delete |

**Annotation**: Destructive operation

**Example**:
```json
{
  "name": "engram_delete",
  "arguments": {
    "id": "eng_abc123"
  }
}
```

---

### engram_list_by_type

List all engrams for a wallet filtered by engram type.

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `wallet_address` | string | Yes | Wallet address to list engrams for |
| `engram_type` | enum | Yes | Type filter |
| `limit` | integer | No | Max results (default 50) |
| `offset` | integer | No | Pagination offset |

**Annotation**: Read-only, idempotent

**Example**:
```json
{
  "name": "engram_list_by_type",
  "arguments": {
    "wallet_address": "0x1234...",
    "engram_type": "strategy",
    "limit": 20
  }
}
```

---

## Profile Tools (2)

User profile management. **Currently disabled for agents.**

### user_profile_get

Get user profile engrams (persona type with `user.profile.*` keys) for a wallet.

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `wallet_address` | string | Yes | Wallet address to get profile for |

**Annotation**: Read-only, idempotent
**Tags**: `hecate.engrams`, `hecate.profile`

---

### user_profile_update

Update a user profile engram field. Maps field name to engram key `user.profile.<field>`.

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `wallet_address` | string | Yes | Wallet address to update |
| `field` | string | Yes | Profile field name (e.g., `base`) |
| `content` | string | Yes | New JSON string content |

**Annotation**: Idempotent write
**Tags**: `hecate.engrams`, `hecate.profile`

---

## Hecate Tools (9)

Session and memory management for Hecate agent. **Currently disabled for agents.**

### hecate_new_session

Create a new chat session. Clears current conversation and starts fresh.

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `wallet_address` | string | Yes | Wallet address of the user |

**Annotation**: Write operation
**Tags**: `hecate.session`

---

### hecate_list_sessions

List all chat sessions for a wallet address. Returns session summaries sorted by most recently updated.

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `wallet_address` | string | Yes | Wallet address of the user |
| `limit` | integer | No | Max sessions to return (default 20) |

**Annotation**: Read-only, idempotent
**Tags**: `hecate.session`

---

### hecate_resume_session

Resume an existing chat session. Loads the session's message history.

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `wallet_address` | string | Yes | Wallet address of the user |
| `session_id` | string | Yes | ID of the session to resume |

**Annotation**: Write operation
**Tags**: `hecate.session`

---

### hecate_delete_session

Delete a chat session. Cannot delete pinned sessions.

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `wallet_address` | string | Yes | Wallet address of the user |
| `session_id` | string | Yes | ID of the session to delete |

**Annotation**: Destructive operation
**Tags**: `hecate.session`

---

### hecate_remember

Proactively save important context about a visitor. Use when they share preferences, facts, decisions, or anything worth remembering.

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `wallet_address` | string | Yes | Wallet address of the visitor |
| `key` | string | Yes | Dot-path key (e.g., `visitor.preference.chains`) |
| `content` | string | Yes | Information to remember |
| `engram_type` | enum | No | `persona`, `preference`, `knowledge` |

**Annotation**: Idempotent write
**Tags**: `hecate.memory`

---

### hecate_cleanup

Compact old conversation sessions. Keeps 5 most recent and all pinned sessions.

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `wallet_address` | string | Yes | Wallet address to clean up |

**Annotation**: Destructive operation
**Tags**: `hecate.management`

---

### hecate_pin_engram

Pin an engram to protect it from cleanup/deletion.

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `id` | string | Yes | Engram ID to pin |

**Annotation**: Idempotent write
**Tags**: `hecate.management`

---

### hecate_unpin_engram

Remove pin protection from an engram, allowing deletion.

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `id` | string | Yes | Engram ID to unpin |

**Annotation**: Write operation
**Tags**: `hecate.management`

---

### hecate_set_model

Switch the AI model. Search by name or keyword.

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `query` | string | Yes | Model name or keyword (e.g., `opus`, `claude-sonnet`, `gpt-4o`) |

**Annotation**: Write operation
**Tags**: `hecate.management`

---

## Moros Tools (5)

Session and memory management for Moros agent. **Currently disabled for agents.**

### moros_remember

Proactively save important context about a visitor.

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `wallet_address` | string | Yes | Wallet address of the visitor |
| `key` | string | Yes | Dot-path key |
| `content` | string | Yes | Information to remember |
| `engram_type` | enum | No | `persona`, `preference`, `knowledge` |

**Annotation**: Idempotent write
**Tags**: `moros.memory`

---

### moros_cleanup

Compact old conversation sessions. Keeps 5 most recent and all pinned.

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `wallet_address` | string | Yes | Wallet address to clean up |

**Annotation**: Destructive operation
**Tags**: `moros.management`

---

### moros_pin_engram

Pin an engram to protect it from cleanup/deletion.

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `id` | string | Yes | Engram ID to pin |

**Annotation**: Idempotent write
**Tags**: `moros.management`

---

### moros_unpin_engram

Remove pin protection from an engram.

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `id` | string | Yes | Engram ID to unpin |

**Annotation**: Write operation
**Tags**: `moros.management`

---

### moros_set_model

Switch the AI model for Moros.

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `query` | string | Yes | Model name or keyword |

**Annotation**: Write operation
**Tags**: `moros.management`

---

## Tool Annotations

All tools include MCP 2025-11-25 annotations:

| Annotation | Description |
|------------|-------------|
| `readOnlyHint: true` | Safe read operation, no side effects |
| `destructiveHint: true` | Dangerous operation, may delete data |
| `idempotentHint: true` | Safe to retry, same result on repeat |

### Annotation Categories

| Category | readOnly | destructive | idempotent | Example Tools |
|----------|----------|-------------|------------|---------------|
| **Read-only** | true | false | true | `engram_get`, `engram_search` |
| **Write** | false | false | false | `engram_create`, `hecate_new_session` |
| **Idempotent Write** | false | false | true | `hecate_remember`, `user_profile_update` |
| **Destructive** | false | true | false | `engram_delete`, `hecate_cleanup` |

---

## Allow List Configuration

Allow lists are defined in `svc/nullblock-agents/src/config/tool_allowlist.rs`:

```rust
pub const AGENT_ALLOWED_TOOLS: &[&str] = &[
    "engram_create",
    "engram_get",
    "engram_search",
    "engram_update",
    "engram_delete",
    "engram_list_by_type",
];

pub const CROSSROADS_ALLOWED_TOOLS: &[&str] = &[
    // Pending verification
];
```

To add a tool to an allow list, add the tool name to the appropriate constant and rebuild the service.

---

## MCP Endpoints

| Endpoint | Description |
|----------|-------------|
| `POST /mcp/jsonrpc` | JSON-RPC 2.0 endpoint (nullblock-agents) |
| `POST http://localhost:9003/mcp/jsonrpc` | Direct agent service |
| `POST http://localhost:3000/mcp/jsonrpc` | Via Erebus proxy |

### List Available Tools

```bash
curl -X POST http://localhost:9003/mcp/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/list",
    "params": {}
  }'
```

### Call a Tool

```bash
curl -X POST http://localhost:9003/mcp/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/call",
    "params": {
      "name": "engram_search",
      "arguments": {
        "wallet_address": "0x1234...",
        "engram_type": "preference"
      }
    }
  }'
```

---

## Summary Table

| Tool | Category | Agent | Crossroads | Annotation |
|------|----------|:-----:|:----------:|------------|
| `engram_create` | Engram | ✅ | ❌ | Write |
| `engram_get` | Engram | ✅ | ❌ | Read-only |
| `engram_search` | Engram | ✅ | ❌ | Read-only |
| `engram_update` | Engram | ✅ | ❌ | Write |
| `engram_delete` | Engram | ✅ | ❌ | Destructive |
| `engram_list_by_type` | Engram | ✅ | ❌ | Read-only |
| `user_profile_get` | Profile | ❌ | ❌ | Read-only |
| `user_profile_update` | Profile | ❌ | ❌ | Idempotent |
| `hecate_new_session` | Hecate | ❌ | ❌ | Write |
| `hecate_list_sessions` | Hecate | ❌ | ❌ | Read-only |
| `hecate_resume_session` | Hecate | ❌ | ❌ | Write |
| `hecate_delete_session` | Hecate | ❌ | ❌ | Destructive |
| `hecate_remember` | Hecate | ❌ | ❌ | Idempotent |
| `hecate_cleanup` | Hecate | ❌ | ❌ | Destructive |
| `hecate_pin_engram` | Hecate | ❌ | ❌ | Idempotent |
| `hecate_unpin_engram` | Hecate | ❌ | ❌ | Write |
| `hecate_set_model` | Hecate | ❌ | ❌ | Write |
| `moros_remember` | Moros | ❌ | ❌ | Idempotent |
| `moros_cleanup` | Moros | ❌ | ❌ | Destructive |
| `moros_pin_engram` | Moros | ❌ | ❌ | Idempotent |
| `moros_unpin_engram` | Moros | ❌ | ❌ | Write |
| `moros_set_model` | Moros | ❌ | ❌ | Write |

---

## Related

- [MCP Servers](./mcp-servers.md) - External MCP server integrations
- [Engrams Service](../services/engrams.md) - Memory layer architecture
- [Agent Architecture](../agents/overview.md) - Agent system design
