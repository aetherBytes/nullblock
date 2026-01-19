# MCP Servers

Model Context Protocol servers integrated with NullBlock development.

## Overview

MCP servers extend Claude Code's capabilities. Automatically available when running Claude Code in the NullBlock project.

## NullBlock MCP Server

**Purpose**: NullBlock's own MCP server exposing agent tools, engrams, and resources.

**Protocol Version**: 2025-11-25

**Endpoint**: `http://localhost:3000/mcp/jsonrpc` (via Erebus proxy)

### Available Tools (9)

| Tool | Description |
|------|-------------|
| `send_agent_message` | Send message to a NullBlock agent |
| `create_task` | Create a new task |
| `get_task_status` | Get task status by ID |
| `list_engrams` | List engrams for a wallet |
| `get_engram` | Get engram by ID |
| `create_engram` | Create a new engram |
| `update_engram` | Update an existing engram |
| `delete_engram` | Delete an engram |
| `search_engrams` | Search engrams by query |

### Available Resources (2)

| URI | Description |
|-----|-------------|
| `agent://hecate` | HECATE vessel AI agent |
| `agent://siren` | Siren marketing agent |

### Available Prompts (2)

| Prompt | Arguments | Description |
|--------|-----------|-------------|
| `agent_chat` | `agent` (req), `context` (opt) | Chat with a NullBlock agent |
| `task_template` | `type` (req) | Create task from template |

### Example: Initialize Session

```bash
curl -X POST http://localhost:3000/mcp/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
      "protocolVersion": "2025-11-25",
      "capabilities": {},
      "clientInfo": { "name": "my-client", "version": "1.0.0" }
    }
  }'
```

### Example: Call Tool

```bash
curl -X POST http://localhost:3000/mcp/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/call",
    "params": {
      "name": "list_engrams",
      "arguments": { "wallet_address": "0x..." }
    }
  }'
```

## ArbFarm MCP Server

**Purpose**: Solana MEV agent swarm - trading, consensus, engram learning tools.

**Protocol Version**: 2025-11-25

**Endpoint**: `http://localhost:9007/mcp/jsonrpc` (direct) or `http://localhost:3000/api/arb/mcp/jsonrpc` (via Erebus)

### Available Tools (97)

| Category | Count | Examples |
|----------|-------|----------|
| Scanner | 3 | `scanner_status`, `scanner_signals`, `scanner_add_venue` |
| Edges | 5 | `edge_list`, `edge_details`, `edge_approve`, `edge_reject` |
| Strategies | 4 | `strategy_list`, `strategy_create`, `strategy_toggle`, `strategy_kill` |
| Curves | 15 | `curve_tokens`, `curve_quote`, `curve_buy`, `curve_sell` |
| Positions | 8 | `position_list`, `position_open`, `position_close`, `position_exit_config` |
| Consensus | 9 | `consensus_request`, `consensus_models_list`, `consensus_config_get` |
| Learning | 6 | `engram_get_arbfarm_learning`, `engram_get_trade_history`, `engram_get_errors` |
| Approvals | 5 | `approval_list`, `approval_approve`, `approval_reject` |
| Threat | 8 | `threat_check_token`, `threat_check_wallet`, `threat_report` |
| KOL | 10 | `kol_list`, `kol_add`, `kol_discovery_status`, `kol_copy_enable` |
| Wallet | 6 | `wallet_status`, `wallet_balance`, `wallet_capital` |
| Swarm | 8 | `swarm_status`, `swarm_health`, `swarm_pause`, `swarm_resume` |
| Other | 10 | `settings_*`, `helius_*`, `research_*` |

### Tool Annotations (MCP 2025-11-25)

All 97 tools include annotations:
- `readOnlyHint`: Safe read operations (list, get, status)
- `destructiveHint`: Dangerous operations (kill, reject, block)
- `idempotentHint`: Safe to retry (toggle, update, config)

### A2A Discovery Tags

Learning tools tagged with `arbFarm.learning` for agent discovery:
- `engram_get_arbfarm_learning`
- `engram_acknowledge_recommendation`
- `engram_get_trade_history`
- `engram_get_errors`
- `engram_request_analysis`
- `engram_get_by_ids`

### Example: List Tools with Annotations

```bash
curl -X POST http://localhost:9007/mcp/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/list",
    "params": {}
  }' | jq '.result.tools[0]'
```

### Example: Get Trade History

```bash
curl -X POST http://localhost:9007/mcp/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/call",
    "params": {
      "name": "engram_get_trade_history",
      "arguments": { "limit": 10 }
    }
  }'
```

## Chrome DevTools MCP

**Purpose**: Browser automation, debugging, testing for Hecate frontend.

### Installation

```bash
claude mcp add chrome-devtools npx chrome-devtools-mcp@latest
```

### Configuration

| Setting | Value |
|---------|-------|
| Debug Port | 9222 |
| User Data Dir | `/tmp/chrome-nullblock-dev` |
| Default URL | `http://localhost:5173` |

### Capabilities

| Category | Tools |
|----------|-------|
| **Navigation** | `navigate_page`, `new_page`, `close_page`, `list_pages` |
| **Inspection** | `take_snapshot`, `take_screenshot`, `hover` |
| **Interaction** | `click`, `fill`, `fill_form`, `press_key`, `drag` |
| **Network** | `list_network_requests`, `get_network_request` |
| **Console** | `list_console_messages`, `evaluate_script` |
| **Performance** | `performance_start_trace`, `performance_analyze_insight` |

### Use Cases

**Testing UI**:
```
"Take a snapshot and check if login button is visible"
"Click wallet connect and verify modal appears"
```

**Debugging Network**:
```
"List network requests to /api/agents"
"Show response from last failed request"
```

**Performance**:
```
"Start trace, reload page, analyze LCP insight"
```

### Manual Chrome Launch

```bash
/Applications/Google\ Chrome.app/Contents/MacOS/Google\ Chrome \
  --remote-debugging-port=9222 \
  --user-data-dir=/tmp/chrome-nullblock-dev \
  --no-first-run \
  "http://localhost:5173"
```

## PixelLab MCP

**Purpose**: AI-powered pixel art generation.

### Capabilities

- Character creation (4 or 8 directions)
- Character animations
- Isometric tiles
- Top-down and sidescroller tilesets
- Map objects

### Use Cases

```
"Create a wizard character with 8 directional views"
"Generate walking animation for the wizard"
"Create isometric grass tile"
```

## Adding New MCP Servers

### Install

```bash
claude mcp add <server-name> <command>
```

### Configuration Location

```
~/.claude/claude_desktop_config.json
```

### Auto-Start Integration

Edit `scripts/nullblock-dev-mac.yml` to add startup commands.

## Troubleshooting

### Chrome Not Connecting

```bash
# Verify Chrome running
lsof -i :9222

# Kill existing
pkill -f "Chrome.*remote-debugging-port=9222"
```

### MCP Not Available

```bash
# List installed
claude mcp list

# Test standalone
npx chrome-devtools-mcp@latest
```

## Related

- [Tmuxinator Setup](../infra/tmuxinator.md)
- [Quick Start](../quickstart.md)
