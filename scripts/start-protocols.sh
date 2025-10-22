#!/usr/bin/env bash
set -e

echo "🌐 Starting Protocol Server..."
cd svc/nullblock-protocols
mkdir -p logs
echo "🦀 Starting Rust protocol service with A2A and MCP support..."
export PORT=8001
export DATABASE_URL="postgresql://postgres:postgres_secure_pass@localhost:5441/agents"
echo ""
echo "🗄️  Waiting for Agents database to be ready..."
while ! docker exec nullblock-postgres-agents pg_isready -U postgres > /dev/null 2>&1; do
  echo "⏳ Waiting for Agents PostgreSQL container to be ready..."
  sleep 2
done
echo "✅ Agents database connection ready"
echo ""
echo "📝 Starting protocol server with database connection..."
echo "🔗 A2A JSON-RPC endpoint: http://localhost:8001/a2a/jsonrpc"
echo "🌐 A2A REST endpoints: http://localhost:8001/v1/*"
echo "📄 Agent Card: http://localhost:8001/v1/card"
echo "📨 Messages: POST /v1/messages, /v1/messages/stream"
echo "📋 Tasks: GET /v1/tasks, /v1/tasks/:id, POST /v1/tasks/:id/cancel"
echo ""
cargo run 2>&1 | tee logs/protocols-server.log
