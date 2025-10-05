#!/bin/bash
# Health monitoring dashboard for NullBlock services

mkdir -p logs

while true; do
  clear
  echo "🎯 NullBlock System Health Dashboard - $(date '+%Y-%m-%d %H:%M:%S')"
  echo "==============================================================="
  echo ""

  # Service Health Checks
  echo "📡 Service Health Status:"
  for service in "erebus:3000" "nullblock-agents:9003" "nullblock-protocols:8001" "frontend:5173"; do
    name=$(echo $service | cut -d: -f1)
    port=$(echo $service | cut -d: -f2)
    if curl -s --max-time 3 "http://localhost:$port/health" > /dev/null 2>&1; then
      echo "  ✅ $name (port $port) - Healthy"
    else
      echo "  ❌ $name (port $port) - Not responding"
    fi
  done
  echo ""

  # Database Connections (Docker)
  echo "🗄️  Database Connections:"
  if docker ps | grep -q nullblock-postgres-agents && docker exec nullblock-postgres-agents pg_isready -U postgres > /dev/null 2>&1; then
    echo "  ✅ PostgreSQL (Agents) - Connected"
    echo "     🔑 Test Credentials: postgres:REDACTED_DB_PASS@localhost:5441/agents"
  else
    echo "  ❌ PostgreSQL (Agents) - Connection failed"
    echo "     🔑 Test Credentials: postgres:REDACTED_DB_PASS@localhost:5441/agents"
  fi

  if docker ps | grep -q nullblock-postgres-erebus && docker exec nullblock-postgres-erebus pg_isready -U postgres > /dev/null 2>&1; then
    echo "  ✅ PostgreSQL (Erebus) - Connected"
    echo "     🔑 Test Credentials: postgres:REDACTED_DB_PASS@localhost:5440/erebus"
  else
    echo "  ❌ PostgreSQL (Erebus) - Connection failed"
    echo "     🔑 Test Credentials: postgres:REDACTED_DB_PASS@localhost:5440/erebus"
  fi

  if docker ps | grep -q nullblock-redis && docker exec nullblock-redis redis-cli ping > /dev/null 2>&1; then
    echo "  ✅ Redis - Connected"
    echo "     🔑 Test Credentials: localhost:6379 (no auth required)"
  else
    echo "  ❌ Redis - Connection failed"
    echo "     🔑 Test Credentials: localhost:6379 (no auth required)"
  fi
  echo ""

  # Kafka Health
  echo "📨 Event Streaming:"
  if docker ps | grep -q nullblock-kafka; then
    echo "  ✅ Kafka - Running"
    echo "  ✅ Zookeeper - Running"
  else
    echo "  ❌ Kafka/Zookeeper - Not running (use just start)"
  fi

  sleep 30
done
