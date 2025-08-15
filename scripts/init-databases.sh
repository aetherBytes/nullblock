#!/bin/bash

# Nullblock Database Initialization Script
# This script creates all PostgreSQL databases needed for the Nullblock ecosystem

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Database configuration
DB_HOST="localhost"
DB_PORT="5432"
DB_USER="nullblock"
DB_PASSWORD="REDACTED_DB_PASS"
DB_SUPERUSER="postgres"

# Database names for each service
DATABASES=(
    "nullblock_mcp"
    "nullblock_orchestration" 
    "nullblock_agents"
    "nullblock_erebus"
    "nullblock_hecate"
    "nullblock_platform"
)

# Check if PostgreSQL is running
check_postgres() {
    print_status "Checking PostgreSQL connection..."
    
    if pg_isready -h $DB_HOST -p $DB_PORT -U $DB_SUPERUSER > /dev/null 2>&1; then
        print_success "PostgreSQL is running and accessible"
        return 0
    else
        print_error "PostgreSQL is not running or not accessible"
        print_status "Please start PostgreSQL first:"
        echo "  brew services start postgresql@15"
        echo "  OR"
        echo "  docker-compose up -d postgres"
        return 1
    fi
}

# Create superuser if it doesn't exist
create_superuser() {
    print_status "Creating superuser '$DB_USER'..."
    
    if psql -h $DB_HOST -p $DB_PORT -U $DB_SUPERUSER -d postgres -c "SELECT 1 FROM pg_roles WHERE rolname='$DB_USER'" | grep -q 1; then
        print_success "Superuser '$DB_USER' already exists"
    else
        psql -h $DB_HOST -p $DB_PORT -U $DB_SUPERUSER -d postgres -c "CREATE USER $DB_USER WITH PASSWORD '$DB_PASSWORD' SUPERUSER;" > /dev/null 2>&1
        print_success "Superuser '$DB_USER' created successfully"
    fi
}

# Create database
create_database() {
    local db_name=$1
    local service_name=$2
    
    print_status "Creating database '$db_name' for $service_name..."
    
    if psql -h $DB_HOST -p $DB_PORT -U $DB_SUPERUSER -d postgres -c "SELECT 1 FROM pg_database WHERE datname='$db_name'" | grep -q 1; then
        print_warning "Database '$db_name' already exists"
    else
        psql -h $DB_HOST -p $DB_PORT -U $DB_SUPERUSER -d postgres -c "CREATE DATABASE $db_name OWNER $DB_USER;" > /dev/null 2>&1
        print_success "Database '$db_name' created successfully"
    fi
}

# Create database schema
create_schema() {
    local db_name=$1
    local service_name=$2
    
    print_status "Creating schema for $service_name..."
    
    # Create basic tables based on service
    case $service_name in
        "nullblock_mcp")
            psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $db_name -c "
                CREATE TABLE IF NOT EXISTS wallet_sessions (
                    id SERIAL PRIMARY KEY,
                    wallet_address VARCHAR(42) NOT NULL,
                    session_id VARCHAR(255) UNIQUE NOT NULL,
                    provider VARCHAR(50) NOT NULL,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    expires_at TIMESTAMP NOT NULL,
                    is_active BOOLEAN DEFAULT TRUE
                );
                
                CREATE TABLE IF NOT EXISTS user_context (
                    id SERIAL PRIMARY KEY,
                    wallet_address VARCHAR(42) NOT NULL,
                    context_data JSONB,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                );
                
                CREATE TABLE IF NOT EXISTS trading_preferences (
                    id SERIAL PRIMARY KEY,
                    wallet_address VARCHAR(42) NOT NULL,
                    preferences JSONB,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                );
                
                CREATE INDEX idx_wallet_sessions_address ON wallet_sessions(wallet_address);
                CREATE INDEX idx_wallet_sessions_active ON wallet_sessions(is_active);
                CREATE INDEX idx_user_context_address ON user_context(wallet_address);
                CREATE INDEX idx_trading_preferences_address ON trading_preferences(wallet_address);
            " > /dev/null 2>&1
            ;;
            
        "nullblock_orchestration")
            psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $db_name -c "
                CREATE TABLE IF NOT EXISTS workflows (
                    id SERIAL PRIMARY KEY,
                    workflow_id VARCHAR(255) UNIQUE NOT NULL,
                    name VARCHAR(255) NOT NULL,
                    description TEXT,
                    goal_description TEXT,
                    target_metric VARCHAR(100),
                    target_value DECIMAL,
                    user_id VARCHAR(255) NOT NULL,
                    status VARCHAR(50) DEFAULT 'created',
                    schedule VARCHAR(100),
                    context_data JSONB,
                    metadata JSONB,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    started_at TIMESTAMP,
                    completed_at TIMESTAMP
                );
                
                CREATE TABLE IF NOT EXISTS tasks (
                    id SERIAL PRIMARY KEY,
                    task_id VARCHAR(255) UNIQUE NOT NULL,
                    workflow_id VARCHAR(255) NOT NULL,
                    name VARCHAR(255) NOT NULL,
                    description TEXT,
                    agent_type VARCHAR(100) NOT NULL,
                    parameters JSONB,
                    dependencies TEXT[],
                    status VARCHAR(50) DEFAULT 'pending',
                    priority INTEGER DEFAULT 2,
                    retry_count INTEGER DEFAULT 0,
                    max_retries INTEGER DEFAULT 3,
                    timeout_seconds INTEGER DEFAULT 300,
                    result JSONB,
                    error TEXT,
                    started_at TIMESTAMP,
                    completed_at TIMESTAMP,
                    FOREIGN KEY (workflow_id) REFERENCES workflows(workflow_id)
                );
                
                CREATE TABLE IF NOT EXISTS bittensor_tasks (
                    id SERIAL PRIMARY KEY,
                    task_id VARCHAR(255) UNIQUE NOT NULL,
                    description TEXT NOT NULL,
                    reward_amount DECIMAL,
                    status VARCHAR(50) DEFAULT 'pending',
                    submitted_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    completed_at TIMESTAMP
                );
                
                CREATE INDEX idx_workflows_user_id ON workflows(user_id);
                CREATE INDEX idx_workflows_status ON workflows(status);
                CREATE INDEX idx_tasks_workflow_id ON tasks(workflow_id);
                CREATE INDEX idx_tasks_status ON tasks(status);
                CREATE INDEX idx_bittensor_tasks_status ON bittensor_tasks(status);
            " > /dev/null 2>&1
            ;;
            
        "nullblock_agents")
            psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $db_name -c "
                CREATE TABLE IF NOT EXISTS arbitrage_opportunities (
                    id SERIAL PRIMARY KEY,
                    token_pair VARCHAR(50) NOT NULL,
                    buy_dex VARCHAR(100) NOT NULL,
                    sell_dex VARCHAR(100) NOT NULL,
                    buy_price DECIMAL NOT NULL,
                    sell_price DECIMAL NOT NULL,
                    profit_percentage DECIMAL NOT NULL,
                    profit_amount DECIMAL NOT NULL,
                    trade_amount DECIMAL NOT NULL,
                    gas_cost DECIMAL NOT NULL,
                    net_profit DECIMAL NOT NULL,
                    confidence DECIMAL NOT NULL,
                    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                );
                
                CREATE TABLE IF NOT EXISTS price_data (
                    id SERIAL PRIMARY KEY,
                    dex VARCHAR(100) NOT NULL,
                    token_a VARCHAR(50) NOT NULL,
                    token_b VARCHAR(50) NOT NULL,
                    price DECIMAL NOT NULL,
                    liquidity DECIMAL NOT NULL,
                    volume_24h DECIMAL NOT NULL,
                    gas_cost DECIMAL DEFAULT 0,
                    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                );
                
                CREATE TABLE IF NOT EXISTS executed_trades (
                    id SERIAL PRIMARY KEY,
                    opportunity_id INTEGER,
                    wallet_address VARCHAR(42) NOT NULL,
                    token_pair VARCHAR(50) NOT NULL,
                    trade_amount DECIMAL NOT NULL,
                    profit_amount DECIMAL NOT NULL,
                    gas_cost DECIMAL NOT NULL,
                    net_profit DECIMAL NOT NULL,
                    status VARCHAR(50) DEFAULT 'pending',
                    transaction_hash VARCHAR(66),
                    executed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (opportunity_id) REFERENCES arbitrage_opportunities(id)
                );
                
                CREATE INDEX idx_arbitrage_opportunities_timestamp ON arbitrage_opportunities(timestamp);
                CREATE INDEX idx_arbitrage_opportunities_profit ON arbitrage_opportunities(profit_percentage);
                CREATE INDEX idx_price_data_dex_pair ON price_data(dex, token_a, token_b);
                CREATE INDEX idx_price_data_timestamp ON price_data(timestamp);
                CREATE INDEX idx_executed_trades_wallet ON executed_trades(wallet_address);
                CREATE INDEX idx_executed_trades_status ON executed_trades(status);
            " > /dev/null 2>&1
            ;;
            
        "nullblock_erebus")
            psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $db_name -c "
                CREATE TABLE IF NOT EXISTS solana_wallets (
                    id SERIAL PRIMARY KEY,
                    wallet_address VARCHAR(44) NOT NULL UNIQUE,
                    public_key VARCHAR(44) NOT NULL,
                    balance DECIMAL DEFAULT 0,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                );
                
                CREATE TABLE IF NOT EXISTS solana_transactions (
                    id SERIAL PRIMARY KEY,
                    signature VARCHAR(88) NOT NULL UNIQUE,
                    wallet_address VARCHAR(44) NOT NULL,
                    transaction_type VARCHAR(50) NOT NULL,
                    amount DECIMAL,
                    fee DECIMAL,
                    status VARCHAR(50) DEFAULT 'pending',
                    block_time TIMESTAMP,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (wallet_address) REFERENCES solana_wallets(wallet_address)
                );
                
                CREATE TABLE IF NOT EXISTS memory_cards (
                    id SERIAL PRIMARY KEY,
                    wallet_address VARCHAR(44) NOT NULL,
                    card_id VARCHAR(255) NOT NULL,
                    card_data JSONB,
                    is_mutable BOOLEAN DEFAULT FALSE,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (wallet_address) REFERENCES solana_wallets(wallet_address)
                );
                
                CREATE TABLE IF NOT EXISTS missions (
                    id SERIAL PRIMARY KEY,
                    wallet_address VARCHAR(44) NOT NULL,
                    mission_id VARCHAR(255) NOT NULL,
                    mission_type VARCHAR(100) NOT NULL,
                    status VARCHAR(50) DEFAULT 'active',
                    progress JSONB,
                    rewards JSONB,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    completed_at TIMESTAMP,
                    FOREIGN KEY (wallet_address) REFERENCES solana_wallets(wallet_address)
                );
                
                CREATE INDEX idx_solana_wallets_address ON solana_wallets(wallet_address);
                CREATE INDEX idx_solana_transactions_signature ON solana_transactions(signature);
                CREATE INDEX idx_solana_transactions_wallet ON solana_transactions(wallet_address);
                CREATE INDEX idx_memory_cards_wallet ON memory_cards(wallet_address);
                CREATE INDEX idx_missions_wallet ON missions(wallet_address);
                CREATE INDEX idx_missions_status ON missions(status);
            " > /dev/null 2>&1
            ;;
            
        "nullblock_hecate")
            psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $db_name -c "
                CREATE TABLE IF NOT EXISTS user_sessions (
                    id SERIAL PRIMARY KEY,
                    session_id VARCHAR(255) UNIQUE NOT NULL,
                    wallet_address VARCHAR(42),
                    theme VARCHAR(20) DEFAULT 'light',
                    preferences JSONB,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    last_activity TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                );
                
                CREATE TABLE IF NOT EXISTS chat_messages (
                    id SERIAL PRIMARY KEY,
                    session_id VARCHAR(255) NOT NULL,
                    message_type VARCHAR(20) NOT NULL,
                    content TEXT NOT NULL,
                    metadata JSONB,
                    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (session_id) REFERENCES user_sessions(session_id)
                );
                
                CREATE TABLE IF NOT EXISTS system_events (
                    id SERIAL PRIMARY KEY,
                    event_type VARCHAR(100) NOT NULL,
                    event_data JSONB,
                    severity VARCHAR(20) DEFAULT 'info',
                    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                );
                
                CREATE INDEX idx_user_sessions_wallet ON user_sessions(wallet_address);
                CREATE INDEX idx_user_sessions_activity ON user_sessions(last_activity);
                CREATE INDEX idx_chat_messages_session ON chat_messages(session_id);
                CREATE INDEX idx_chat_messages_timestamp ON chat_messages(timestamp);
                CREATE INDEX idx_system_events_type ON system_events(event_type);
                CREATE INDEX idx_system_events_timestamp ON system_events(timestamp);
            " > /dev/null 2>&1
            ;;
            
        "nullblock_platform")
            psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $db_name -c "
                CREATE TABLE IF NOT EXISTS users (
                    id SERIAL PRIMARY KEY,
                    wallet_address VARCHAR(42) UNIQUE NOT NULL,
                    username VARCHAR(100),
                    email VARCHAR(255),
                    profile_data JSONB,
                    subscription_tier VARCHAR(50) DEFAULT 'free',
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                );
                
                CREATE TABLE IF NOT EXISTS workflows_marketplace (
                    id SERIAL PRIMARY KEY,
                    workflow_id VARCHAR(255) UNIQUE NOT NULL,
                    creator_address VARCHAR(42) NOT NULL,
                    name VARCHAR(255) NOT NULL,
                    description TEXT,
                    category VARCHAR(100),
                    price DECIMAL DEFAULT 0,
                    revenue_share DECIMAL DEFAULT 0.05,
                    downloads INTEGER DEFAULT 0,
                    rating DECIMAL DEFAULT 0,
                    is_public BOOLEAN DEFAULT TRUE,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (creator_address) REFERENCES users(wallet_address)
                );
                
                CREATE TABLE IF NOT EXISTS transactions (
                    id SERIAL PRIMARY KEY,
                    transaction_hash VARCHAR(66) UNIQUE NOT NULL,
                    from_address VARCHAR(42) NOT NULL,
                    to_address VARCHAR(42) NOT NULL,
                    amount DECIMAL NOT NULL,
                    transaction_type VARCHAR(50) NOT NULL,
                    status VARCHAR(50) DEFAULT 'pending',
                    gas_used INTEGER,
                    gas_price DECIMAL,
                    block_number INTEGER,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                );
                
                CREATE TABLE IF NOT EXISTS analytics (
                    id SERIAL PRIMARY KEY,
                    metric_name VARCHAR(100) NOT NULL,
                    metric_value DECIMAL NOT NULL,
                    metric_data JSONB,
                    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                );
                
                CREATE INDEX idx_users_wallet ON users(wallet_address);
                CREATE INDEX idx_workflows_marketplace_creator ON workflows_marketplace(creator_address);
                CREATE INDEX idx_workflows_marketplace_category ON workflows_marketplace(category);
                CREATE INDEX idx_transactions_hash ON transactions(transaction_hash);
                CREATE INDEX idx_transactions_from ON transactions(from_address);
                CREATE INDEX idx_transactions_type ON transactions(transaction_type);
                CREATE INDEX idx_analytics_metric ON analytics(metric_name);
                CREATE INDEX idx_analytics_timestamp ON analytics(timestamp);
            " > /dev/null 2>&1
            ;;
    esac
    
    print_success "Schema created for $service_name"
}

# Create connection strings file
create_connection_strings() {
    print_status "Creating connection strings file..."
    
    cat > scripts/database-connections.txt << EOF
# Nullblock Database Connection Strings
# Use these in your applications or DBeaver

# PostgreSQL Connection Details
Host: localhost
Port: 5432
Username: nullblock
Password: REDACTED_DB_PASS

# Individual Database Connections:

# MCP Server Database
Database: nullblock_mcp
Connection String: postgresql://nullblock:REDACTED_DB_PASS@localhost:5432/nullblock_mcp

# Orchestration Engine Database  
Database: nullblock_orchestration
Connection String: postgresql://nullblock:REDACTED_DB_PASS@localhost:5432/nullblock_orchestration

# Agents Service Database
Database: nullblock_agents
Connection String: postgresql://nullblock:REDACTED_DB_PASS@localhost:5432/nullblock_agents

# Erebus Rust Server Database
Database: nullblock_erebus
Connection String: postgresql://nullblock:REDACTED_DB_PASS@localhost:5432/nullblock_erebus

# Hecate Frontend Database
Database: nullblock_hecate
Connection String: postgresql://nullblock:REDACTED_DB_PASS@localhost:5432/nullblock_hecate

# Platform Marketplace Database
Database: nullblock_platform
Connection String: postgresql://nullblock:REDACTED_DB_PASS@localhost:5432/nullblock_platform

# DBeaver Connection Setup:
# 1. Create new PostgreSQL connection
# 2. Host: localhost
# 3. Port: 5432
# 4. Database: (select from above)
# 5. Username: nullblock
# 6. Password: REDACTED_DB_PASS
EOF

    print_success "Connection strings saved to scripts/database-connections.txt"
}

# Show database status
show_status() {
    print_status "Database Status:"
    echo ""
    
    for db in "${DATABASES[@]}"; do
        if psql -h $DB_HOST -p $DB_PORT -U $DB_SUPERUSER -d postgres -c "SELECT 1 FROM pg_database WHERE datname='$db'" | grep -q 1; then
            print_success "✅ $db - Ready"
        else
            print_error "❌ $db - Not found"
        fi
    done
    
    echo ""
    print_status "Connection Details:"
    echo "  Host: $DB_HOST"
    echo "  Port: $DB_PORT"
    echo "  Username: $DB_USER"
    echo "  Password: $DB_PASSWORD"
    echo ""
    print_status "DBeaver Setup:"
    echo "  1. Create new PostgreSQL connection"
    echo "  2. Use connection details above"
    echo "  3. Select database from the list"
    echo "  4. Test connection"
}

# Main function
main() {
    print_status "Initializing Nullblock Databases..."
    echo ""
    
    # Check PostgreSQL
    if ! check_postgres; then
        exit 1
    fi
    
    # Create superuser
    create_superuser
    
    # Create databases and schemas
    for db in "${DATABASES[@]}"; do
        service_name=$(echo $db | sed 's/nullblock_//')
        create_database $db $service_name
        create_schema $db $service_name
    done
    
    # Create connection strings file
    create_connection_strings
    
    echo ""
    print_success "All databases initialized successfully!"
    echo ""
    
    # Show status
    show_status
}

# Run main function
main "$@"




