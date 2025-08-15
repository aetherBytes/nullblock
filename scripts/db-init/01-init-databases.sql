-- Nullblock Database Initialization for Docker
-- This script creates all databases and schemas when PostgreSQL container starts

-- Create nullblock user if it doesn't exist
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'nullblock') THEN
        CREATE USER nullblock WITH PASSWORD 'nullblock_secure_pass' SUPERUSER;
    END IF;
END
$$;

-- Create databases for each service
CREATE DATABASE nullblock_mcp OWNER nullblock;
CREATE DATABASE nullblock_orchestration OWNER nullblock;
CREATE DATABASE nullblock_agents OWNER nullblock;
CREATE DATABASE nullblock_erebus OWNER nullblock;
CREATE DATABASE nullblock_hecate OWNER nullblock;
CREATE DATABASE nullblock_platform OWNER nullblock;

-- Connect to MCP database and create schema
\c nullblock_mcp;

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

-- Connect to Orchestration database and create schema
\c nullblock_orchestration;

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

-- Connect to Agents database and create schema
\c nullblock_agents;

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

-- Connect to Erebus database and create schema
\c nullblock_erebus;

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

-- Connect to Hecate database and create schema
\c nullblock_hecate;

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

-- Connect to Platform database and create schema
\c nullblock_platform;

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

-- Grant permissions to nullblock user
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO nullblock;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO nullblock;
GRANT ALL PRIVILEGES ON ALL FUNCTIONS IN SCHEMA public TO nullblock;

-- Return to postgres database
\c postgres;



