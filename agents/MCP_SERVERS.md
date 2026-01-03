# MCP Servers

Guide for Model Context Protocol (MCP) servers integrated with NullBlock development.

## Overview

MCP servers extend Claude Code's capabilities by providing specialized tools for interacting with external systems. These are automatically available when running Claude Code in the NullBlock project.

## Installed MCP Servers

### Chrome DevTools MCP

**Purpose**: Browser automation, debugging, and testing for the Hecate frontend.

**Installation**:
```bash
claude mcp add chrome-devtools npx chrome-devtools-mcp@latest
```

**Auto-Start**: Integrated into `just dev-mac` - Chrome launches automatically with remote debugging enabled.

**Configuration**:
| Setting | Value |
|---------|-------|
| Debug Port | 9222 |
| User Data Dir | `/tmp/chrome-nullblock-dev` |
| Default URL | `http://localhost:5173` |

#### Capabilities

| Category | Tools |
|----------|-------|
| **Navigation** | `navigate_page`, `new_page`, `close_page`, `list_pages`, `select_page` |
| **Inspection** | `take_snapshot`, `take_screenshot`, `hover` |
| **Interaction** | `click`, `fill`, `fill_form`, `press_key`, `drag`, `upload_file` |
| **Network** | `list_network_requests`, `get_network_request` |
| **Console** | `list_console_messages`, `get_console_message`, `evaluate_script` |
| **Performance** | `performance_start_trace`, `performance_stop_trace`, `performance_analyze_insight` |
| **Emulation** | `emulate` (CPU throttling, network conditions, geolocation) |
| **Dialogs** | `handle_dialog`, `wait_for` |

#### Common Use Cases

**Testing UI Components**:
```
"Take a snapshot of the current page and check if the login button is visible"
"Click the wallet connect button and verify the modal appears"
```

**Debugging Network Issues**:
```
"List all network requests to the /api/agents endpoint"
"Show me the response from the last failed request"
```

**Performance Analysis**:
```
"Start a performance trace, reload the page, then analyze the LCP insight"
```

**Form Testing**:
```
"Fill the registration form with test data and submit"
```

#### Manual Chrome Launch

If you need to start Chrome manually (outside of `just dev-mac`):

```bash
/Applications/Google\ Chrome.app/Contents/MacOS/Google\ Chrome \
  --remote-debugging-port=9222 \
  --user-data-dir=/tmp/chrome-nullblock-dev \
  --no-first-run \
  --no-default-browser-check \
  "http://localhost:5173"
```

### PixelLab MCP

**Purpose**: AI-powered pixel art generation for game assets.

**Capabilities**:
- Character creation with directional views (4 or 8 directions)
- Character animations (walk, run, attack, etc.)
- Isometric tiles for game maps
- Top-down and sidescroller tilesets
- Map objects with transparent backgrounds

**Use Cases**:
```
"Create a wizard character with 8 directional views"
"Generate a walking animation for the wizard"
"Create an isometric grass tile"
```

## Adding New MCP Servers

### Installation

```bash
claude mcp add <server-name> <command>
```

### Integration with Dev Environment

To auto-start an MCP server's dependencies with `just dev-mac`, edit:
```
scripts/nullblock-dev-mac.yml
```

Add startup commands to the `on_project_start` section and cleanup to `on_project_stop`.

### Configuration Location

MCP server configurations are stored in:
```
~/.claude/claude_desktop_config.json
```

## Troubleshooting

### Chrome DevTools Not Connecting

1. Verify Chrome is running with debug port:
   ```bash
   lsof -i :9222
   ```

2. Kill any existing debug instances:
   ```bash
   pkill -f "Chrome.*remote-debugging-port=9222"
   ```

3. Restart with `just dev-mac` or launch manually.

### MCP Server Not Available

1. Check if server is installed:
   ```bash
   claude mcp list
   ```

2. Verify the server command works standalone:
   ```bash
   npx chrome-devtools-mcp@latest
   ```

## Related Documentation

- **Development setup**: See [CLAUDE.md](../CLAUDE.md) for full dev environment
- **Tmuxinator config**: `scripts/nullblock-dev-mac.yml`
- **Agent system**: See [AGENTS.md](./AGENTS.md)
