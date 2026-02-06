# Tmuxinator Integration for Content Service

## Location
The tmuxinator configuration is located at:
```
~/.config/tmuxinator/nullblock-dev.yml
```

## Content Service Window

The `nullblock-content` window has been added with 3 panes:

### Pane 1: Service Startup
```bash
~/nullblock/scripts/start-content.sh
```
- Loads .env.dev
- Waits for database readiness
- Runs migrations automatically
- Starts service with cargo run --release
- Logs to logs/content.log

### Pane 2: Health Monitor
```bash
~/nullblock/scripts/monitor-content-health.sh
```
- 5-second refresh loop
- Health endpoint check (http://localhost:8002/health)
- Pending queue count display
- Service status indicator

### Pane 3: Log Tail
```bash
~/nullblock/scripts/tail-content-logs.sh
```
- Live log streaming from logs/content.log
- Auto-waits for log file creation
- Shows all service output

## Window Configuration

```yaml
- nullblock-content:
    layout: tiled
    pre_window: |
      if [ -f .env.dev ]; then
        set -a
        source .env.dev
        set +a
      fi
    panes:
      - ~/nullblock/scripts/start-content.sh
      - ~/nullblock/scripts/monitor-content-health.sh
      - ~/nullblock/scripts/tail-content-logs.sh
```

## Port Cleanup

Port 8002 has been added to the orphaned process cleanup in `on_project_start`:
```yaml
for port in 3000 3001 8001 8002 9003 9004 9007; do
  pid=$(lsof -ti :$port 2>/dev/null)
  if [ -n "$pid" ]; then
    echo "   Killing process on port $port (PID: $pid)"
    kill $pid 2>/dev/null || true
  fi
done
```

## Startup Banner

Content service has been added to the service ports display:
```
📡 Service Ports:
   🌐 Erebus (unified router): http://localhost:3000
   🌐 Protocol Server (A2A/MCP): http://localhost:8001
   📝 Content Service: http://localhost:8002
   🎯 Hecate Agent (Rust): http://localhost:9003
   🧠 Engram Service: http://localhost:9004
   ⚡ ArbFarm (Solana MEV): http://localhost:9007
   🎨 Frontend: http://localhost:5173
   📚 Internal Docs: http://localhost:3001
```

## Usage

### Start Full Environment
```bash
just dev-mac  # or just dev-linux
```

This will:
1. Start all infrastructure (including content DB)
2. Run all migrations
3. Launch tmuxinator with content service window
4. Display 3 panes for content service

### Navigate to Content Window
In tmux, use:
- `Ctrl+b w` - Show window list
- Select `nullblock-content`

Or use keyboard shortcuts:
- `Ctrl+b 0-9` - Switch to window by number

### Switch Between Panes
- `Ctrl+b o` - Cycle through panes
- `Ctrl+b ←→↑↓` - Navigate panes with arrows
- `Ctrl+b q` - Show pane numbers

### Stop Service
- In service pane: `Ctrl+C`
- Or kill entire session: `tmux kill-session -t nullblock-dev`

## Troubleshooting

### Service Not Starting
**Issue:** Content pane shows errors
**Check:**
```bash
# Database ready?
docker exec nullblock-postgres-content pg_isready -U postgres

# Migrations run?
~/nullblock/scripts/migrate-content.sh

# Correct DATABASE_URL?
grep DATABASE_URL svc/nullblock-content/.env.dev
```

### No Logs Appearing
**Issue:** Log tail pane is empty
**Check:**
```bash
# Log file exists?
ls -la svc/nullblock-content/logs/content.log

# Service actually running?
lsof -i :8002
```

### Health Monitor Shows DOWN
**Issue:** Health monitor shows red "❌ Service: DOWN"
**Check:**
```bash
# Try manual curl
curl http://localhost:8002/health

# Check logs for errors
tail -50 svc/nullblock-content/logs/content.log
```

## Manual Setup (If Not Using Tmuxinator)

If you're not using tmuxinator, you can manually start the service with monitoring:

### Terminal 1: Service
```bash
cd ~/nullblock/svc/nullblock-content
cargo run --release
```

### Terminal 2: Health Monitor
```bash
~/nullblock/scripts/monitor-content-health.sh
```

### Terminal 3: Logs
```bash
tail -f ~/nullblock/svc/nullblock-content/logs/content.log
```

---

**Note:** The tmuxinator configuration is user-specific and lives in your home directory. Changes to `~/.config/tmuxinator/nullblock-dev.yml` are not tracked in git.
