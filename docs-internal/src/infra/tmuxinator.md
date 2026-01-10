# Tmuxinator Setup

**One-command dev environment** - All services in organized tmux windows.

## Quick Start

```bash
cd ~/nullblock && just dev-mac    # macOS
cd ~/nullblock && just dev-linux  # Linux
```

## Attach to Session

```bash
tmux attach -t nullblock-dev
```

## Window Layout

| Key | Window | Contents |
|-----|--------|----------|
| `Ctrl+B, 0` | Monitoring | Health checks, logs |
| `Ctrl+B, 1` | Erebus | Port 3000 |
| `Ctrl+B, 2` | Agents | Port 9003 |
| `Ctrl+B, 3` | Protocols | Port 8001 |
| `Ctrl+B, 4` | Engrams | Port 9004 |
| `Ctrl+B, 5` | Tasks | Task management |
| `Ctrl+B, 6` | Frontend | Hecate (port 5173) |

## Configuration File

Location: `scripts/nullblock-dev-mac.yml`

### Structure

```yaml
name: nullblock-dev
root: ~/nullblock

on_project_start:
  - just start          # Docker infrastructure
  - start Chrome with debugging

windows:
  - monitoring:
      layout: main-horizontal
      panes:
        - watch health checks
        - tail logs

  - erebus:
      root: ~/nullblock/svc/erebus
      panes:
        - cargo run

  # ... more windows
```

## Customization

### Add New Window

Edit `scripts/nullblock-dev-mac.yml`:

```yaml
windows:
  # ... existing windows ...

  - my-service:
      root: ~/nullblock/svc/my-service
      panes:
        - cargo run
```

### Add to on_project_start

```yaml
on_project_start:
  - just start
  - echo "Starting my custom service..."
```

## Manual Service Start

If you need to start services individually:

```bash
# Terminal 1 - Erebus
cd ~/nullblock/svc/erebus && cargo run

# Terminal 2 - Agents
cd ~/nullblock/svc/nullblock-agents && cargo run

# Terminal 3 - Engrams
cd ~/nullblock/svc/nullblock-engrams && cargo run

# Terminal 4 - Protocols
cd ~/nullblock/svc/nullblock-protocols && cargo run

# Terminal 5 - Frontend
cd ~/nullblock/svc/hecate && npm run develop
```

## Troubleshooting

### Session Not Creating

```bash
# Kill existing session
tmux kill-session -t nullblock-dev

# Restart
tmuxinator start nullblock-dev
```

### Window Missing

Check `scripts/nullblock-dev-mac.yml` for syntax errors.

### Services Not Starting

Check individual service logs:

```bash
# In tmux, switch to window
Ctrl+B, <window-number>

# Check for errors in output
```

## Related

- [Quick Start](../quickstart.md)
- [Docker & Containers](./docker.md)
