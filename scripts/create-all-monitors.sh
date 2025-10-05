#!/bin/bash

# Create all remaining monitoring scripts for Mac tmuxinator

# Agents DB monitoring
cat > monitor-agents-db.sh << 'EOF'
#!/bin/bash
echo "🗄️  Agents Database Monitoring"
echo "PostgreSQL monitoring for tasks, agents, user_references tables..."
echo ""
mkdir -p ~/nullblock/logs
while true; do
  echo "$(date '+%Y-%m-%d %H:%M:%S') [DB] Agents database status..." | tee -a ~/nullblock/logs/agents-db.log
  if docker exec nullblock-postgres-agents psql -U postgres -d agents -c "SELECT COUNT(*) as task_count FROM tasks;" 2>/dev/null | grep -q "task_count"; then
    task_count=$(docker exec nullblock-postgres-agents psql -U postgres -d agents -t -c "SELECT COUNT(*) FROM tasks;" 2>/dev/null | xargs)
    user_ref_count=$(docker exec nullblock-postgres-agents psql -U postgres -d agents -t -c "SELECT COUNT(*) FROM user_references;" 2>/dev/null | xargs)
    agent_count=$(docker exec nullblock-postgres-agents psql -U postgres -d agents -t -c "SELECT COUNT(*) FROM agents;" 2>/dev/null | xargs)
    echo "📊 Total: Tasks=$task_count | Users=$user_ref_count | Agents=$agent_count" | tee -a ~/nullblock/logs/agents-db.log
    
    created_tasks=$(docker exec nullblock-postgres-agents psql -U postgres -d agents -t -c "SELECT COUNT(*) FROM tasks WHERE status='created';" 2>/dev/null | xargs)
    running_tasks=$(docker exec nullblock-postgres-agents psql -U postgres -d agents -t -c "SELECT COUNT(*) FROM tasks WHERE status='running';" 2>/dev/null | xargs)
    completed_tasks=$(docker exec nullblock-postgres-agents psql -U postgres -d agents -t -c "SELECT COUNT(*) FROM tasks WHERE status='completed';" 2>/dev/null | xargs)
    failed_tasks=$(docker exec nullblock-postgres-agents psql -U postgres -d agents -t -c "SELECT COUNT(*) FROM tasks WHERE status='failed';" 2>/dev/null | xargs)
    paused_tasks=$(docker exec nullblock-postgres-agents psql -U postgres -d agents -t -c "SELECT COUNT(*) FROM tasks WHERE status='paused';" 2>/dev/null | xargs)
    cancelled_tasks=$(docker exec nullblock-postgres-agents psql -U postgres -d agents -t -c "SELECT COUNT(*) FROM tasks WHERE status='cancelled';" 2>/dev/null | xargs)
    echo "📋 Status: Created=$created_tasks | Running=$running_tasks | Completed=$completed_tasks | Failed=$failed_tasks | Paused=$paused_tasks | Cancelled=$cancelled_tasks" | tee -a ~/nullblock/logs/agents-db.log
    
    actioned_tasks=$(docker exec nullblock-postgres-agents psql -U postgres -d agents -t -c "SELECT COUNT(*) FROM tasks WHERE actioned_at IS NOT NULL;" 2>/dev/null | xargs)
    echo "⚡ Actions: $actioned_tasks tasks have been processed by agents" | tee -a ~/nullblock/logs/agents-db.log
    
    recent_tasks=$(docker exec nullblock-postgres-agents psql -U postgres -d agents -t -c "SELECT COUNT(*) FROM tasks WHERE created_at >= NOW() - INTERVAL '1 hour';" 2>/dev/null | xargs)
    echo "⏰ Recent activity: $recent_tasks tasks created in last hour" | tee -a ~/nullblock/logs/agents-db.log
  else
    echo "❌ Database not accessible or tables not created" | tee -a ~/nullblock/logs/agents-db.log
  fi
  sleep 45
done
EOF

# Hecate health monitoring
cat > monitor-hecate-health.sh << 'EOF'
#!/bin/bash
echo "🎯 Hecate Agent Health & Task Monitoring"
echo "Real-time monitoring of agent health and task management..."
echo ""
mkdir -p ~/nullblock/logs
while true; do
  echo "$(date '+%Y-%m-%d %H:%M:%S') [AGENT] Hecate health check..." | tee -a ~/nullblock/logs/hecate-health.log
  if curl -s --max-time 5 "http://localhost:9003/hecate/health" > /dev/null 2>&1; then
    echo "✅ Hecate agent healthy" | tee -a ~/nullblock/logs/hecate-health.log
    curl -s "http://localhost:9003/hecate/model-status" | jq -r '"🧠 Model: " + (.current_model // "unknown") + " | Status: " + (.status // "unknown")' 2>/dev/null | tee -a ~/nullblock/logs/hecate-health.log
    task_count=$(curl -s "http://localhost:9003/tasks" | jq '.total // 0' 2>/dev/null || echo "0")
    echo "📋 Current tasks: $task_count" | tee -a ~/nullblock/logs/hecate-health.log
    if curl -s --max-time 3 "http://localhost:9003/health" | jq -r '.components.database.status' 2>/dev/null | grep -q "healthy"; then
      echo "🗄️  Database migrations: ✅ Complete" | tee -a ~/nullblock/logs/hecate-health.log
    else
      echo "🗄️  Database migrations: ⚠️  Check required" | tee -a ~/nullblock/logs/hecate-health.log
    fi
  else
    echo "❌ Hecate agent not responding" | tee -a ~/nullblock/logs/hecate-health.log
  fi
  sleep 30
done
EOF

# Protocol DB monitoring
cat > monitor-protocol-db.sh << 'EOF'
#!/bin/bash
echo "🗄️  Protocol Service Database Integration Monitoring"
echo "Monitoring Agents database connection for protocol service integration..."
echo ""
mkdir -p ~/nullblock/svc/nullblock-protocols/logs
while true; do
  echo "$(date '+%Y-%m-%d %H:%M:%S') [AGENTS-DB] Database status..." | tee -a ~/nullblock/svc/nullblock-protocols/logs/agents-db.log
  if docker exec nullblock-postgres-agents pg_isready -U postgres > /dev/null 2>&1; then
    echo "✅ PostgreSQL connection healthy" | tee -a ~/nullblock/svc/nullblock-protocols/logs/agents-db.log
  else
    echo "❌ PostgreSQL connection failed" | tee -a ~/nullblock/svc/nullblock-protocols/logs/agents-db.log
  fi
  sleep 45
done
EOF

# Protocol health monitoring
cat > monitor-protocol-health.sh << 'EOF'
#!/bin/bash
echo "🔗 Protocol Health Monitoring..."
echo "A2A and MCP protocol health monitoring..."
echo ""
echo "📊 Protocol endpoint monitoring..."
while true; do
  if curl -s --max-time 3 "http://localhost:8001/v1/card" > /dev/null 2>&1; then
    echo "$(date '+%H:%M:%S') ✅ A2A Agent Card endpoint healthy"
  else
    echo "$(date '+%H:%M:%S') ❌ A2A Agent Card endpoint not responding"
  fi
  if curl -s --max-time 3 "http://localhost:8001/health" > /dev/null 2>&1; then
    echo "$(date '+%H:%M:%S') ✅ Protocol health endpoint healthy"
  else
    echo "$(date '+%H:%M:%S') ❌ Protocol health endpoint not responding"
  fi
  sleep 60
done
EOF

# Protocol logs
cat > tail-protocol-logs.sh << 'EOF'
#!/bin/bash
echo "🌐 Protocol Server Logs & Health"
echo "Monitoring protocol server logs and A2A/MCP operations..."
echo ""
cd ~/nullblock/svc/nullblock-protocols
mkdir -p logs
tail -f logs/protocols-server.log 2>/dev/null || echo "⚠️ Waiting for protocol server logs..."
EOF

# Task system monitoring
cat > monitor-task-system.sh << 'EOF'
#!/bin/bash
echo "📋 Task Management System"
echo "The orchestration service has been integrated into the Hecate Agent (Rust)"
echo "Task management is now handled directly by the agents service on port 9003"
echo ""
echo "🔄 Monitoring Task System Performance..."
while true; do
  echo "$(date '+%Y-%m-%d %H:%M:%S') [TASK-SYS] Task system metrics..."
  if curl -s --max-time 3 "http://localhost:3000/api/agents/tasks" > /dev/null 2>&1; then
    echo "✅ Task API via Erebus - Healthy"
  else
    echo "❌ Task API via Erebus - Failed"
  fi
  if curl -s --max-time 3 "http://localhost:9003/health" > /dev/null 2>&1; then
    echo "✅ Hecate Agent Direct API - Healthy"
  else
    echo "❌ Hecate Agent Direct API - Failed"
  fi
  sleep 60
done
EOF

chmod +x *.sh
echo "✅ Created all monitoring scripts"
