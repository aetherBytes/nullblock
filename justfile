# Nullblock MVP Commands
# Run with: just <command>

# OS detection (used by dev command)
_os := if os() == "macos" { "macos" } else { "linux" }

# Start all infrastructure services (OS-aware)
start:
    #!/usr/bin/env bash
    if [[ "{{_os}}" == "macos" ]]; then
        echo "🍎 Starting services for macOS (with port mapping)..."
        just start-mac
    else
        echo "🐧 Starting services for Linux (with host networking)..."
        just start-linux
    fi

# Start all infrastructure services (macOS with port mapping)
start-mac:
    @echo "🚀 Starting all NullBlock infrastructure services (macOS)..."
    @echo "Using port mapping for Docker Desktop compatibility..."
    @echo ""
    @echo "📦 Creating Docker network for container communication..."
    @docker network create nullblock-network 2>/dev/null || true
    @echo "  ✅ Network ready"
    @echo ""
    @echo "📦 Creating persistent volumes..."
    @docker volume create nullblock-postgres-erebus-data 2>/dev/null || true
    @docker volume create nullblock-postgres-agents-data 2>/dev/null || true
    @docker volume create nullblock-redis-data 2>/dev/null || true
    @echo "  ✅ Volumes ready"
    @echo ""
    @docker rm -f nullblock-postgres-erebus nullblock-postgres-agents nullblock-redis nullblock-zookeeper nullblock-kafka 2>/dev/null || true
    @echo "📦 Starting PostgreSQL databases..."
    @docker run -d --name nullblock-postgres-erebus --network nullblock-network -p 5440:5432 -v nullblock-postgres-erebus-data:/var/lib/postgresql/data -e POSTGRES_DB=erebus -e POSTGRES_USER=postgres -e POSTGRES_PASSWORD=REDACTED_DB_PASS postgres:15-alpine postgres -c wal_level=logical -c max_replication_slots=4 -c max_wal_senders=4 -c max_logical_replication_workers=4
    @docker run -d --name nullblock-postgres-agents --network nullblock-network -p 5441:5432 -v nullblock-postgres-agents-data:/var/lib/postgresql/data -e POSTGRES_DB=agents -e POSTGRES_USER=postgres -e POSTGRES_PASSWORD=REDACTED_DB_PASS postgres:15-alpine postgres -c wal_level=logical -c max_replication_slots=4 -c max_wal_senders=4 -c max_logical_replication_workers=4
    @echo "📦 Starting Redis..."
    @docker run -d --name nullblock-redis --network nullblock-network -p 6379:6379 -v nullblock-redis-data:/data redis:7-alpine redis-server --appendonly yes
    @echo "📦 Starting Zookeeper..."
    @docker run -d --name nullblock-zookeeper --network nullblock-network -p 2181:2181 -e ZOOKEEPER_CLIENT_PORT=2181 -e ZOOKEEPER_TICK_TIME=2000 confluentinc/cp-zookeeper:7.4.0
    @echo ""
    @echo "⏳ Waiting for services to be ready..."
    @sleep 3
    @while ! docker exec nullblock-postgres-erebus pg_isready -U postgres > /dev/null 2>&1; do echo "  ⏳ Waiting for Erebus PostgreSQL..."; sleep 2; done
    @echo "  ✅ Erebus PostgreSQL ready"
    @while ! docker exec nullblock-postgres-agents pg_isready -U postgres > /dev/null 2>&1; do echo "  ⏳ Waiting for Agents PostgreSQL..."; sleep 2; done
    @echo "  ✅ Agents PostgreSQL ready"
    @while ! docker exec nullblock-redis redis-cli ping > /dev/null 2>&1; do echo "  ⏳ Waiting for Redis..."; sleep 2; done
    @echo "  ✅ Redis ready"
    @sleep 5
    @echo "  ✅ Zookeeper ready"
    @echo ""
    @echo "📦 Starting Kafka (requires Zookeeper)..."
    @docker run -d --name nullblock-kafka --network nullblock-network -p 9092:9092 -p 9093:9093 -e KAFKA_BROKER_ID=1 -e KAFKA_ZOOKEEPER_CONNECT=nullblock-zookeeper:2181 -e KAFKA_ADVERTISED_LISTENERS=PLAINTEXT://localhost:9092,PLAINTEXT_INTERNAL://nullblock-kafka:9093 -e KAFKA_LISTENER_SECURITY_PROTOCOL_MAP=PLAINTEXT:PLAINTEXT,PLAINTEXT_INTERNAL:PLAINTEXT -e KAFKA_INTER_BROKER_LISTENER_NAME=PLAINTEXT_INTERNAL -e KAFKA_OFFSETS_TOPIC_REPLICATION_FACTOR=1 -e KAFKA_AUTO_CREATE_TOPICS_ENABLE=true confluentinc/cp-kafka:7.4.0
    @echo "⏳ Waiting for Kafka to be ready..."
    @sleep 15
    @while ! docker exec nullblock-kafka kafka-broker-api-versions --bootstrap-server localhost:9092 > /dev/null 2>&1; do echo "  ⏳ Waiting for Kafka broker..."; sleep 3; done
    @echo "  ✅ Kafka ready"
    @echo ""
    @echo "🔄 Running database migrations..."
    @just migrate
    @echo ""
    @echo "✅ All infrastructure services are running!"
    @docker ps --filter "name=nullblock" --format "table {{"{{"}}.Names}}\t{{"{{"}}.Status}}"
    @echo ""
    @echo "🚀 Services ready. You can now start application servers:"
    @echo "   Erebus:    cd svc/erebus && cargo run"
    @echo "   Agents:    cd svc/nullblock-agents && cargo run --release"
    @echo "   Protocols: cd svc/nullblock-protocols && cargo run"
    @echo "   Frontend:  cd svc/hecate && npm run develop"

# Start all infrastructure services (Linux with bridge networking)
start-linux:
    @echo "🚀 Starting all NullBlock infrastructure services..."
    @echo "Using Docker bridge networking for container-to-container communication..."
    @echo ""
    @echo "📦 Creating Docker network for container communication..."
    @docker network create nullblock-network 2>/dev/null || true
    @echo "  ✅ Network ready"
    @echo ""
    @echo "📦 Creating persistent volumes..."
    @docker volume create nullblock-postgres-erebus-data 2>/dev/null || true
    @docker volume create nullblock-postgres-agents-data 2>/dev/null || true
    @docker volume create nullblock-redis-data 2>/dev/null || true
    @echo "  ✅ Volumes ready"
    @echo ""
    @docker rm -f nullblock-postgres-erebus nullblock-postgres-agents nullblock-redis nullblock-zookeeper nullblock-kafka 2>/dev/null || true
    @echo "📦 Starting PostgreSQL databases..."
    @docker run -d --name nullblock-postgres-erebus --network nullblock-network -p 5440:5432 -v nullblock-postgres-erebus-data:/var/lib/postgresql/data -e POSTGRES_DB=erebus -e POSTGRES_USER=postgres -e POSTGRES_PASSWORD=REDACTED_DB_PASS postgres:15-alpine postgres -c wal_level=logical -c max_replication_slots=4 -c max_wal_senders=4 -c max_logical_replication_workers=4
    @docker run -d --name nullblock-postgres-agents --network nullblock-network -p 5441:5432 -v nullblock-postgres-agents-data:/var/lib/postgresql/data -e POSTGRES_DB=agents -e POSTGRES_USER=postgres -e POSTGRES_PASSWORD=REDACTED_DB_PASS postgres:15-alpine postgres -c wal_level=logical -c max_replication_slots=4 -c max_wal_senders=4 -c max_logical_replication_workers=4
    @echo "📦 Starting Redis..."
    @docker run -d --name nullblock-redis --network nullblock-network -p 6379:6379 -v nullblock-redis-data:/data redis:7-alpine redis-server --appendonly yes
    @echo "📦 Starting Zookeeper..."
    @docker run -d --name nullblock-zookeeper --network nullblock-network -p 2181:2181 -e ZOOKEEPER_CLIENT_PORT=2181 -e ZOOKEEPER_TICK_TIME=2000 confluentinc/cp-zookeeper:7.4.0
    @echo ""
    @echo "⏳ Waiting for services to be ready..."
    @sleep 3
    @while ! docker exec nullblock-postgres-erebus pg_isready -U postgres > /dev/null 2>&1; do echo "  ⏳ Waiting for Erebus PostgreSQL..."; sleep 2; done
    @echo "  ✅ Erebus PostgreSQL ready"
    @while ! docker exec nullblock-postgres-agents pg_isready -U postgres > /dev/null 2>&1; do echo "  ⏳ Waiting for Agents PostgreSQL..."; sleep 2; done
    @echo "  ✅ Agents PostgreSQL ready"
    @while ! docker exec nullblock-redis redis-cli ping > /dev/null 2>&1; do echo "  ⏳ Waiting for Redis..."; sleep 2; done
    @echo "  ✅ Redis ready"
    @sleep 5
    @echo "  ✅ Zookeeper ready"
    @echo ""
    @echo "📦 Starting Kafka (requires Zookeeper)..."
    @docker run -d --name nullblock-kafka --network nullblock-network -p 9092:9092 -e KAFKA_BROKER_ID=1 -e KAFKA_ZOOKEEPER_CONNECT=nullblock-zookeeper:2181 -e KAFKA_ADVERTISED_LISTENERS=PLAINTEXT://localhost:9092,PLAINTEXT_INTERNAL://nullblock-kafka:9093 -e KAFKA_LISTENER_SECURITY_PROTOCOL_MAP=PLAINTEXT:PLAINTEXT,PLAINTEXT_INTERNAL:PLAINTEXT -e KAFKA_INTER_BROKER_LISTENER_NAME=PLAINTEXT_INTERNAL -e KAFKA_OFFSETS_TOPIC_REPLICATION_FACTOR=1 -e KAFKA_AUTO_CREATE_TOPICS_ENABLE=true confluentinc/cp-kafka:7.4.0
    @echo "⏳ Waiting for Kafka to be ready..."
    @sleep 15
    @while ! docker exec nullblock-kafka kafka-broker-api-versions --bootstrap-server localhost:9092 > /dev/null 2>&1; do echo "  ⏳ Waiting for Kafka broker..."; sleep 3; done
    @echo "  ✅ Kafka ready"
    @echo ""
    @echo "🔄 Running database migrations..."
    @just migrate
    @echo ""
    @echo "✅ All infrastructure services are running!"
    @docker ps --filter "name=nullblock" --format "table {{"{{"}}.Names}}\t{{"{{"}}.Status}}"
    @echo ""
    @echo "🚀 Services ready. You can now start application servers:"
    @echo "   Erebus:    cd svc/erebus && cargo run"
    @echo "   Agents:    cd svc/nullblock-agents && cargo run --release"
    @echo "   Protocols: cd svc/nullblock-protocols && cargo run"
    @echo "   Frontend:  cd svc/hecate && npm run develop"

# Terminate all services (Docker containers)
term:
    @echo "🛑 Terminating all NullBlock services..."
    @echo "Stopping Docker containers..."
    @docker stop nullblock-kafka nullblock-zookeeper nullblock-redis nullblock-postgres-agents nullblock-postgres-erebus 2>/dev/null || true
    @echo "Removing Docker containers..."
    @docker rm nullblock-kafka nullblock-zookeeper nullblock-redis nullblock-postgres-agents nullblock-postgres-erebus 2>/dev/null || true
    @echo "Removing Docker network..."
    @docker network rm nullblock-network 2>/dev/null || true
    @echo "✅ All services terminated and cleaned up"
    @docker ps --filter "name=nullblock" --format "table {{"{{"}}.Names}}\t{{"{{"}}.Status}}"

# Stop all Nullblock services (legacy - use 'term' instead)
stop:
    @echo "🛑 Stopping Nullblock services..."
    @just term

# Kill all development services by port (cross-platform)
kill-services:
    ./scripts/kill-services.sh

# Restart all services
restart:
    @echo "🔄 Restarting Nullblock services..."
    @just term
    @just start

# Build all services
build:
    @echo "🔨 Building Nullblock services..."
    ./scripts/start-nullblock.sh build

# Build Erebus Rust server
build-erebus:
    @echo "🦀 Building Erebus Rust server..."
    ./scripts/build-erebus

# Show service logs
logs service="":
    @echo "📋 Showing logs..."
    ./scripts/start-nullblock.sh logs {{service}}

# Check service status
status:
    @echo "📊 Checking service status..."
    ./scripts/start-nullblock.sh status

# Health check all services
health:
    @echo "🏥 Running health checks..."
    ./scripts/start-nullblock.sh health

# Comprehensive health check with curl
health-check:
    @mkdir -p logs
    @echo "$(date '+%Y-%m-%d %H:%M:%S') [HEALTH-CHECK] 🏥 Starting comprehensive health check" | tee -a logs/just-commands.log
    @echo "🏥 Comprehensive Health Check - All Services"
    @echo "=============================================="
    @echo ""
    @echo "🔍 MCP Server (Port 8001):"
    @curl -s --max-time 5 http://localhost:8001/health | jq '.' 2>/dev/null | tee -a logs/just-commands.log || (echo "❌ MCP Server not responding" | tee -a logs/just-commands.log)
    @echo ""
    @echo "🔍 Orchestration (Port 8002):"
    @curl -s --max-time 5 http://localhost:8002/health | jq '.' 2>/dev/null | tee -a logs/just-commands.log || (echo "❌ Orchestration not responding" | tee -a logs/just-commands.log)
    @echo ""
    @echo "🔍 Erebus Server (Port 3000):"
    @curl -s --max-time 5 http://localhost:3000/health | jq '.' 2>/dev/null | tee -a logs/just-commands.log || (echo "❌ Erebus Server not responding" | tee -a logs/just-commands.log)
    @echo ""
    @echo "🔍 General Agents (Port 9001):"
    @curl -s --max-time 5 http://localhost:9001/health | jq '.' 2>/dev/null | tee -a logs/just-commands.log || (echo "❌ General Agents not responding" | tee -a logs/just-commands.log)
    @echo ""
    @echo "🔍 Hecate Agent (Port 9003):"
    @curl -s --max-time 5 http://localhost:9003/health | jq '.' 2>/dev/null | tee -a logs/just-commands.log || (echo "❌ Hecate Agent not responding" | tee -a logs/just-commands.log)
    @echo ""
    @echo "🔍 Frontend (Port 5173):"
    @curl -s --max-time 5 http://localhost:5173 >/dev/null && (echo "✅ Frontend responding" | tee -a logs/just-commands.log) || (echo "❌ Frontend not responding" | tee -a logs/just-commands.log)
    @echo ""
    @echo "🔍 LM Studio (Port 1234):"
    @curl -s --max-time 5 http://localhost:1234/v1/models >/dev/null && (echo "✅ LM Studio responding" | tee -a logs/just-commands.log) || (echo "❌ LM Studio not responding" | tee -a logs/just-commands.log)
    @echo "$(date '+%Y-%m-%d %H:%M:%S') [HEALTH-CHECK] ✅ Health check completed" | tee -a logs/just-commands.log
    @echo ""

# Mock API tests for base endpoints
test-api:
    @mkdir -p logs
    @echo "$(date '+%Y-%m-%d %H:%M:%S') [API-TEST] 🧪 Starting API endpoint tests" | tee -a logs/just-commands.log
    @echo "🧪 API Endpoint Tests - Mock Data"
    @echo "=================================="
    @echo ""
    @echo "🔹 Testing MCP Server endpoints:"
    @curl -s --max-time 5 http://localhost:8001/mcp/data-sources | jq '.' 2>/dev/null | tee -a logs/just-commands.log || (echo "❌ MCP data sources not available" | tee -a logs/just-commands.log)
    @echo ""
    @echo "🔹 Testing Orchestration endpoints:"
    @curl -s --max-time 5 http://localhost:8002/orchestration/status | jq '.' 2>/dev/null | tee -a logs/just-commands.log || (echo "❌ Orchestration status not available" | tee -a logs/just-commands.log)
    @echo ""
    @echo "🔹 Testing Erebus wallet endpoints:"
    @curl -s --max-time 5 http://localhost:3000/api/wallets | jq '.' 2>/dev/null | tee -a logs/just-commands.log || (echo "❌ Erebus wallets not available" | tee -a logs/just-commands.log)
    @echo ""
    @echo "🔹 Testing Hecate Agent endpoints:"
    @curl -s --max-time 5 http://localhost:9003/hecate/model-status | jq '.' 2>/dev/null | tee -a logs/just-commands.log || (echo "❌ Hecate model status not available" | tee -a logs/just-commands.log)
    @echo ""
    @echo "🔹 Testing LLM Service endpoints:"
    @curl -s --max-time 5 http://localhost:1234/v1/models | jq '.' 2>/dev/null | tee -a logs/just-commands.log || (echo "❌ LM Studio models not available" | tee -a logs/just-commands.log)
    @echo "$(date '+%Y-%m-%d %H:%M:%S') [API-TEST] ✅ API tests completed" | tee -a logs/just-commands.log
    @echo ""

# Quick connectivity test
ping-services:
    @mkdir -p logs
    @echo "$(date '+%Y-%m-%d %H:%M:%S') [PING-TEST] 🏓 Starting connectivity ping" | tee -a logs/just-commands.log
    @echo "🏓 Quick Connectivity Test"
    @echo "=========================="
    @echo -n "MCP (8001): "; curl -s --max-time 2 http://localhost:8001/health >/dev/null && (echo "✅ UP" | tee -a logs/just-commands.log) || (echo "❌ DOWN" | tee -a logs/just-commands.log)
    @echo -n "Orchestration (8002): "; curl -s --max-time 2 http://localhost:8002/health >/dev/null && (echo "✅ UP" | tee -a logs/just-commands.log) || (echo "❌ DOWN" | tee -a logs/just-commands.log)
    @echo -n "Erebus (3000): "; curl -s --max-time 2 http://localhost:3000/health >/dev/null && (echo "✅ UP" | tee -a logs/just-commands.log) || (echo "❌ DOWN" | tee -a logs/just-commands.log)
    @echo -n "General Agents (9001): "; curl -s --max-time 2 http://localhost:9001/health >/dev/null && (echo "✅ UP" | tee -a logs/just-commands.log) || (echo "❌ DOWN" | tee -a logs/just-commands.log)
    @echo -n "Hecate Agent (9003): "; curl -s --max-time 2 http://localhost:9003/health >/dev/null && (echo "✅ UP" | tee -a logs/just-commands.log) || (echo "❌ DOWN" | tee -a logs/just-commands.log)
    @echo -n "Frontend (5173): "; curl -s --max-time 2 http://localhost:5173 >/dev/null && (echo "✅ UP" | tee -a logs/just-commands.log) || (echo "❌ DOWN" | tee -a logs/just-commands.log)
    @echo -n "LM Studio (1234): "; curl -s --max-time 2 http://localhost:1234/v1/models >/dev/null && (echo "✅ UP" | tee -a logs/just-commands.log) || (echo "❌ DOWN" | tee -a logs/just-commands.log)
    @echo "$(date '+%Y-%m-%d %H:%M:%S') [PING-TEST] ✅ Ping test completed" | tee -a logs/just-commands.log

# Test Hecate Agent chat endpoint with mock message
test-chat:
    @mkdir -p logs
    @echo "$(date '+%Y-%m-%d %H:%M:%S') [CHAT-TEST] 💬 Starting Hecate chat test" | tee -a logs/just-commands.log
    @echo "💬 Testing Hecate Agent Chat"
    @echo "============================="
    @echo "Sending test message..."
    @echo "$(date '+%Y-%m-%d %H:%M:%S') [CHAT-TEST] 📤 REQUEST:" | tee -a logs/just-commands.log
    @echo '{"message":"Hello, this is a test message from the health check system."}' | jq '.' | tee -a logs/just-commands.log
    @echo "$(date '+%Y-%m-%d %H:%M:%S') [CHAT-TEST] 📥 RESPONSE:" | tee -a logs/just-commands.log
    @curl -s --max-time 10 -X POST http://localhost:9003/chat \
        -H "Content-Type: application/json" \
        -d '{"message":"Hello, this is a test message from the health check system."}' | \
        jq '.' 2>/dev/null | tee -a logs/just-commands.log || (echo "❌ Chat endpoint not responding" | tee -a logs/just-commands.log)
    @echo "$(date '+%Y-%m-%d %H:%M:%S') [CHAT-TEST] ✅ Chat test completed" | tee -a logs/just-commands.log
    @echo ""

# Test MCP data sources with mock query
test-mcp-data:
    @mkdir -p logs
    @echo "$(date '+%Y-%m-%d %H:%M:%S') [MCP-DATA-TEST] 📊 Starting MCP data test" | tee -a logs/just-commands.log
    @echo "📊 Testing MCP Data Sources"
    @echo "==========================="
    @echo "Available data sources:"
    @curl -s --max-time 5 http://localhost:8001/mcp/data-sources | jq '.' 2>/dev/null | tee -a logs/just-commands.log || (echo "❌ No data sources available" | tee -a logs/just-commands.log)
    @echo ""
    @echo "Testing price oracle data:"
    @curl -s --max-time 5 "http://localhost:8001/mcp/data/price_oracle/coingecko?symbols=bitcoin" | jq '.' 2>/dev/null | tee -a logs/just-commands.log || (echo "❌ Price data not available" | tee -a logs/just-commands.log)
    @echo "$(date '+%Y-%m-%d %H:%M:%S') [MCP-DATA-TEST] ✅ MCP data test completed" | tee -a logs/just-commands.log
    @echo ""

# Clean up everything
cleanup:
    @echo "🧹 Cleaning up..."
    ./scripts/start-nullblock.sh cleanup

# Wipe database volumes and start fresh
wipe-db:
    @echo "⚠️  WARNING: This will DELETE ALL DATABASE DATA!"
    @echo "⚠️  This action cannot be undone."
    @echo ""
    @echo "Stopping all services..."
    @just term
    @echo ""
    @echo "🗑️  Removing database volumes..."
    @docker volume rm nullblock-postgres-erebus-data nullblock-postgres-agents-data 2>/dev/null || true
    @echo "✅ Database volumes removed"
    @echo ""
    @echo "💡 Run 'just start' to create fresh databases with migrations"

# Setup development environment
dev-setup:
    @echo "⚙️ Setting up development environment..."
    ./scripts/dev-setup.sh setup

# Install dependencies only
dev-install:
    @echo "📦 Installing dependencies..."
    ./scripts/dev-setup.sh install

# Create development environment file
dev-env:
    @echo "🔧 Creating development environment..."
    ./scripts/dev-setup.sh env

# Create development scripts
dev-scripts:
    @echo "📝 Creating development scripts..."
    ./scripts/dev-setup.sh scripts

# Start development services
dev-services:
    @echo "🚀 Starting development services..."
    ./scripts/dev-setup.sh services

# Start Erebus Rust server
start-erebus:
    @echo "🦀 Starting Erebus Rust server..."
    ./scripts/start-erebus

# Test the complete setup
test:
    @echo "🧪 Testing Nullblock setup..."
    @echo "Running comprehensive test suite..."
    @echo ""
    just health-check
    @echo ""
    just ping-services
    @echo ""
    just test-api
    @echo ""
    just test-chat
    @echo ""
    just test-mcp-data
    @echo ""
    @echo "✅ Complete test suite finished"

# Quick start (build + start)
quick:
    @echo "⚡ Quick start..."
    ./scripts/start-nullblock.sh build
    ./scripts/start-nullblock.sh start

# Launch development environment with tmuxinator (OS-aware)
dev:
    #!/usr/bin/env bash
    if [[ "{{_os}}" == "macos" ]]; then
        echo "🍎 Launching macOS development environment..."
        ./scripts/dev-mac
    else
        echo "🐧 Launching Linux development environment..."
        ./scripts/dev-tmux
    fi

# Launch development environment with tmuxinator (Linux/original)
dev-tmux:
    @echo "🖥️ Launching development environment with tmuxinator..."
    ./scripts/dev-tmux

# Launch development environment with tmuxinator (macOS)
dev-mac:
    @echo "🍎 Launching macOS development environment..."
    ./scripts/dev-mac

# Install tmuxinator and setup
dev-tmux-install:
    @echo "📦 Installing tmuxinator and setup..."
    ./scripts/dev-tmux install

# Setup tmuxinator configuration
dev-tmux-setup:
    @echo "⚙️ Setting up tmuxinator configuration..."
    ./scripts/dev-tmux setup

# Install tmuxinator and setup (macOS)
dev-mac-install:
    @echo "📦 Installing tmuxinator and setup (macOS)..."
    ./scripts/dev-mac install

# Setup tmuxinator configuration (macOS)
dev-mac-setup:
    @echo "⚙️ Setting up tmuxinator configuration (macOS)..."
    ./scripts/dev-mac setup

# Validate Mac development environment
dev-mac-validate:
    @echo "🔍 Validating macOS development environment..."
    ./scripts/test-mac-setup.sh

# Initialize databases (Docker)
init-db:
    @echo "🗄️ Initializing Docker PostgreSQL databases..."
    docker-compose up postgres-agents postgres-erebus postgres-mcp postgres-orchestration postgres-analytics -d
    @echo "⏳ Waiting for databases to be ready..."
    @sleep 10
    @echo "🔍 Checking database health..."
    docker-compose ps | grep postgres

# Show Docker database connections
docker-db-info:
    @echo "🗄️ Docker Database Connection Info..."
    @echo "PostgreSQL Agents:     postgresql://postgres:REDACTED_DB_PASS@localhost:5441/agents"
    @echo "PostgreSQL Erebus:     postgresql://postgres:REDACTED_DB_PASS@localhost:5440/erebus"
    @echo "PostgreSQL MCP:        postgresql://postgres:REDACTED_DB_PASS@localhost:5442/mcp"
    @echo "PostgreSQL Orchestration: postgresql://postgres:REDACTED_DB_PASS@localhost:5443/orchestration"
    @echo "PostgreSQL Analytics:  postgresql://postgres:REDACTED_DB_PASS@localhost:5444/analytics"
    @echo "Redis:                 redis://localhost:6379"
    @echo "Kafka:                 localhost:9092"
    @echo "IPFS API:              http://localhost:5001"
    @echo "IPFS Gateway:          http://localhost:8080"

# Run all database migrations and syncs
migrate:
    @echo "🔄 Running all database migrations..."
    @echo "=============================================="
    @echo ""
    @echo "📋 Step 1: Running Erebus database migrations..."
    @echo "Applying Erebus schema updates..."
    @./scripts/run-erebus-migrations.sh
    @echo "✅ Erebus migrations completed"
    @echo ""
    @echo "📋 Step 2: Running Agents database migrations..."
    @echo "Applying Agents schema updates..."
    @./scripts/run-agents-migrations.sh
    @echo "✅ Agents migrations completed"
    @echo ""
    @echo "📊 Final status check..."
    @echo "Erebus tables:"
    @docker exec nullblock-postgres-erebus psql -U postgres -d erebus -c "\dt" 2>/dev/null || echo "❌ Erebus database not accessible"
    @echo ""
    @echo "Agents tables:"
    @docker exec nullblock-postgres-agents psql -U postgres -d agents -c "\dt" 2>/dev/null || echo "❌ Agents database not accessible"
    @echo ""
    @echo "📡 Logical replication status:"
    @docker exec nullblock-postgres-erebus psql -U postgres -d erebus -c "SELECT pubname, puballtables FROM pg_publication WHERE pubname = 'erebus_user_sync';" 2>/dev/null || echo "⚠️  Publication not configured (run migration 002 on Erebus)"
    @docker exec nullblock-postgres-agents psql -U postgres -d agents -c "SELECT subname, subenabled FROM pg_subscription WHERE subname = 'agents_user_sync';" 2>/dev/null || echo "ℹ️  Subscription not configured (logical replication optional for development)"
    @echo ""
    @echo "🎉 All migrations completed successfully!"

# Show help
help:
    @echo "Nullblock MVP Commands:"
    @echo ""
    @echo "🚀 Quick Start Commands:"
    @echo "  just start     - Start all infrastructure services (auto-detects OS)"
    @echo "  just start-mac - Start services for macOS (port mapping)"
    @echo "  just start-linux - Start services for Linux (host networking)"
    @echo "  just term      - Terminate all services (stop and remove containers)"
    @echo "  just migrate   - Run all database migrations"
    @echo ""
    @echo "📦 Docker Commands:"
    @echo "  just stop      - Stop all services (alias for 'term')"
    @echo "  just kill-services - Kill all development services by port"
    @echo "  just restart   - Restart all services"
    @echo "  just build     - Build all services"
    @echo "  just logs      - Show all logs"
    @echo "  just logs <service> - Show specific service logs"
    @echo "  just status    - Check service status"
    @echo "  just health    - Run health checks"
    @echo "  just cleanup   - Clean up everything"
    @echo "  just wipe-db   - ⚠️  DELETE all database data (fresh start)"
    @echo "  just quick     - Quick start (build + start)"
    @echo ""
    @echo "🛠️  Development Commands:"
    @echo "  just dev-setup    - Complete development setup"
    @echo "  just dev-install  - Install dependencies only"
    @echo "  just dev-env      - Create environment file"
    @echo "  just dev-scripts  - Create development scripts"
    @echo "  just dev-services - Start development services"
    @echo "  just start-erebus - Start Erebus Rust server"
    @echo "  just build-erebus - Build Erebus Rust server"
    @echo "  just init-db      - Initialize PostgreSQL databases (local)"
    @echo "  just docker-db-info - Show Docker database connections"
    @echo ""
    @echo "Testing Commands:"
    @echo "  just test         - Test complete setup"
    @echo "  just health-check - Comprehensive health check with curl"
    @echo "  just ping-services - Quick connectivity test"
    @echo "  just test-api     - Test API endpoints with mock data"
    @echo "  just test-chat    - Test Hecate Agent chat endpoint"
    @echo "  just test-mcp-data - Test MCP data sources"
    @echo ""
    @echo "Development Environment:"
    @echo "  just dev             - Launch dev environment (auto-detects OS)"
    @echo "  just dev-tmux        - Launch Linux dev environment with tmuxinator"
    @echo "  just dev-mac         - Launch macOS dev environment with tmuxinator"
    @echo "  just dev-tmux-install - Install tmuxinator and setup (Linux)"
    @echo "  just dev-tmux-setup   - Setup tmuxinator configuration (Linux)"
    @echo "  just dev-mac-install  - Install tmuxinator and setup (macOS)"
    @echo "  just dev-mac-setup    - Setup tmuxinator configuration (macOS)"
    @echo "  just dev-mac-validate - Validate macOS development environment"
    @echo ""
    @echo "Service URLs:"
    @echo "  Frontend: http://localhost:5173"
    @echo "  MCP API: http://localhost:8001"
    @echo "  Orchestration API: http://localhost:8002"
    @echo "  Agents API: http://localhost:8003"
    @echo "  Erebus API: http://localhost:3000"
    @echo ""
    @echo "  IPFS Gateway: http://localhost:8080"
