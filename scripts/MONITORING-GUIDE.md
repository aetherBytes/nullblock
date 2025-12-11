# 📊 NullBlock Monitoring & Debug Guide

## Quick Status Check

Run this anytime to see complete system status:
```bash
~/nullblock/scripts/service-status.sh
```

## Monitoring Scripts

### 🎯 Main Health Dashboard
```bash
~/nullblock/scripts/monitor-health.sh
```
Shows comprehensive system health with color-coded status:
- ✅ GREEN: Service healthy and responding
- ⚠️ YELLOW: Service partially working or degraded
- ❌ RED: Service down or not responding

**Checks:**
- Core services (Erebus, Agents, Protocols, Frontend)
- Infrastructure (PostgreSQL, Redis, Kafka)
- API connectivity

### 🧪 API Endpoint Testing
```bash
~/nullblock/scripts/monitor-api.sh
```
Continuously tests API endpoints with detailed diagnostics:
- Health endpoints for all services
- Task management API (Erebus → Agents)
- Protocols service endpoints
- Logs results to `logs/api-tests.log`

### 📋 Task System Monitor
```bash
~/nullblock/scripts/monitor-task-system.sh
```
Focused monitoring for task management system:
- Checks Erebus router availability
- Tests Agents service health
- Verifies task API routing
- Shows task state breakdown (working, completed, failed)
- Database connectivity status
- Logs to `logs/task-system.log`

### 📋 Enhanced Task Metrics
```bash
~/nullblock/scripts/monitor-tasks.sh
```
Detailed task system diagnostics:
- Service availability checks
- Task API response validation
- Agent model status
- Clear error messages with troubleshooting hints
- Logs to `logs/task-metrics.log`

### 🏥 Health Monitor Dashboard
```bash
~/nullblock/scripts/monitor-health-dashboard.sh
```
Live-refreshing health dashboard (10-second intervals):
- Service health with version info
- Database connectivity tests
- Color-coded visual feedback
- Never clears screen, continuous monitoring

## Common Issues & Solutions

### ❌ "Task system not available via Erebus"

**Cause:** Erebus router (port 3000) is not running or not responding

**Fix:**
```bash
lsof -ti:3000
cd ~/nullblock/svc/erebus && cargo run
tail -f ~/nullblock/svc/erebus/logs/erebus.log
```

### ❌ "Agents service not running (port 9003)"

**Cause:** Agents service is down or failed to start

**Fix:**
```bash
lsof -ti:9003
cd ~/nullblock/svc/nullblock-agents && cargo run
tail -f ~/nullblock/svc/nullblock-agents/logs/*.log
```

### ❌ "Database connection failed"

**Cause:** PostgreSQL containers not running

**Fix:**
```bash
docker ps | grep postgres
just start
```

### ⚠️ "Task API - Unexpected response"

**Cause:** API contract mismatch or database schema issue

**Debug:**
```bash
curl -s "http://localhost:3000/api/agents/tasks" | jq .
psql -h localhost -p 5441 -U postgres -d agents -c "SELECT COUNT(*) FROM tasks;"
```

## Development Environment

### Start Everything (Recommended)
```bash
tmuxinator start nullblock-dev
```

This launches all services in organized tmux windows:
- **monitoring**: Health dashboard, task monitor, API tests
- **erebus**: Erebus router + database monitors
- **nullblock-agents**: Agents service + Kafka monitor
- **nullblock-protocols**: Protocols service
- **frontend**: Hecate React app

### Manual Service Start
```bash
just start
cd svc/erebus && cargo run
cd svc/nullblock-agents && cargo run
cd svc/nullblock-protocols && cargo run
cd svc/hecate && npm run develop
```

## Log Locations

- **Erebus**: `~/nullblock/svc/erebus/logs/erebus.log`
- **Agents**: `~/nullblock/svc/nullblock-agents/logs/*.log`
- **Protocols**: `~/nullblock/svc/nullblock-protocols/logs/*.log`
- **Monitoring**: `~/nullblock/logs/*.log`

## Port Reference

| Service | Port | Health Check |
|---------|------|--------------|
| Erebus Router | 3000 | http://localhost:3000/health |
| Hecate Agent API | 9003 | http://localhost:9003/health |
| Protocols Service | 8001 | http://localhost:8001/health |
| Frontend | 5173 | http://localhost:5173 |
| Erebus DB | 5440 | psql -h localhost -p 5440 -U postgres -d erebus |
| Agents DB | 5441 | psql -h localhost -p 5441 -U postgres -d agents |
| Redis | 6379 | redis-cli ping |
| Kafka | 9092 | docker exec nullblock-kafka kafka-topics --list |

## Troubleshooting Workflow

1. **Check overall health:**
   ```bash
   ~/nullblock/scripts/service-status.sh
   ```

2. **Identify failing service** (look for ❌ RED indicators)

3. **Check service logs:**
   ```bash
   tail -f ~/nullblock/svc/<service-name>/logs/*.log
   ```

4. **Test specific API endpoint:**
   ```bash
   curl -s "http://localhost:<port>/health" | jq .
   ```

5. **Restart service if needed:**
   ```bash
   cd ~/nullblock/svc/<service-name> && cargo run
   ```

6. **Monitor recovery:**
   Use the relevant monitoring script to watch the service come back online

## Color Legend

- 🟢 **GREEN (✅)**: Healthy - service is running and responding correctly
- 🟡 **YELLOW (⚠️)**: Warning - service is up but degraded or returning errors
- 🔴 **RED (❌)**: Critical - service is down or not responding
- 🔵 **CYAN**: Informational headers and test descriptions

## Tips

- All monitoring scripts auto-refresh, press **Ctrl+C** to exit
- Scripts create log files in `~/nullblock/logs/` for historical analysis
- Use tmux/tmuxinator for side-by-side monitoring
- Run `just term` to cleanly stop all Docker infrastructure
