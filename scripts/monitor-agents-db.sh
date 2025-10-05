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
