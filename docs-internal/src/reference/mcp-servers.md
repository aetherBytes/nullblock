# MCP Servers

Model Context Protocol servers integrated with NullBlock development.

## Overview

MCP servers extend Claude Code's capabilities. Automatically available when running Claude Code in the NullBlock project.

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
