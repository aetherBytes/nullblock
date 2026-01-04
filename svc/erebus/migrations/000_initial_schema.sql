-- Erebus Initial Schema
-- Tables: user_references, user_api_keys, agent_api_keys, user_rate_limits
-- Includes logical replication setup for syncing user_references to Agents DB

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- =============================================================================
-- USER REFERENCES (source-agnostic user identity)
-- =============================================================================

CREATE TABLE IF NOT EXISTS user_references (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_identifier VARCHAR NOT NULL,
    network VARCHAR NOT NULL,
    user_type VARCHAR NOT NULL DEFAULT 'external',
    source_type JSONB NOT NULL DEFAULT '{"type": "web3_wallet", "provider": "unknown"}'::jsonb,
    email VARCHAR,
    metadata JSONB DEFAULT '{}'::jsonb,
    preferences JSONB DEFAULT '{}'::jsonb,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_user_references_source ON user_references(source_identifier, network);
CREATE INDEX IF NOT EXISTS idx_user_references_email ON user_references(email) WHERE email IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_user_references_active ON user_references(is_active) WHERE is_active = true;
CREATE UNIQUE INDEX IF NOT EXISTS user_references_source_network_unique
    ON user_references(source_identifier, network) WHERE is_active = true;

-- =============================================================================
-- USER API KEYS (encrypted LLM provider keys for users)
-- =============================================================================

CREATE TABLE IF NOT EXISTS user_api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES user_references(id) ON DELETE CASCADE,
    provider VARCHAR NOT NULL CHECK (provider IN ('openai', 'anthropic', 'groq', 'openrouter', 'huggingface', 'ollama')),
    encrypted_key BYTEA NOT NULL,
    encryption_iv BYTEA NOT NULL,
    encryption_tag BYTEA NOT NULL,
    key_prefix VARCHAR(20),
    key_suffix VARCHAR(20),
    key_name VARCHAR(255),
    last_used_at TIMESTAMPTZ,
    usage_count BIGINT DEFAULT 0,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT user_api_keys_user_provider_unique UNIQUE (user_id, provider)
);

CREATE INDEX IF NOT EXISTS idx_user_api_keys_user_id ON user_api_keys(user_id);
CREATE INDEX IF NOT EXISTS idx_user_api_keys_provider ON user_api_keys(provider);

-- =============================================================================
-- AGENT API KEYS (encrypted LLM provider keys for agents)
-- =============================================================================

CREATE TABLE IF NOT EXISTS agent_api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_name VARCHAR NOT NULL,
    provider VARCHAR NOT NULL CHECK (provider IN ('openai', 'anthropic', 'groq', 'openrouter', 'huggingface', 'ollama')),
    encrypted_key BYTEA NOT NULL,
    encryption_iv BYTEA NOT NULL,
    encryption_tag BYTEA NOT NULL,
    key_prefix VARCHAR(20),
    key_suffix VARCHAR(20),
    key_name VARCHAR(255),
    last_used_at TIMESTAMPTZ,
    usage_count BIGINT DEFAULT 0,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT agent_api_keys_agent_provider_unique UNIQUE (agent_name, provider)
);

CREATE INDEX IF NOT EXISTS idx_agent_api_keys_agent_name ON agent_api_keys(agent_name);
CREATE INDEX IF NOT EXISTS idx_agent_api_keys_provider ON agent_api_keys(provider);

-- =============================================================================
-- USER RATE LIMITS (daily API usage tracking)
-- =============================================================================

CREATE TABLE IF NOT EXISTS user_rate_limits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES user_references(id) ON DELETE CASCADE,
    agent_name VARCHAR NOT NULL,
    daily_count INTEGER DEFAULT 0,
    last_reset_date DATE DEFAULT CURRENT_DATE,
    daily_limit INTEGER DEFAULT 100,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT user_rate_limits_user_agent_unique UNIQUE (user_id, agent_name)
);

CREATE INDEX IF NOT EXISTS idx_user_rate_limits_user_agent ON user_rate_limits(user_id, agent_name);

-- =============================================================================
-- TRIGGER FUNCTIONS
-- =============================================================================

CREATE OR REPLACE FUNCTION trigger_update_updated_at() RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS update_user_references_updated_at ON user_references;
CREATE TRIGGER update_user_references_updated_at
    BEFORE UPDATE ON user_references FOR EACH ROW EXECUTE FUNCTION trigger_update_updated_at();

DROP TRIGGER IF EXISTS update_user_api_keys_updated_at ON user_api_keys;
CREATE TRIGGER update_user_api_keys_updated_at
    BEFORE UPDATE ON user_api_keys FOR EACH ROW EXECUTE FUNCTION trigger_update_updated_at();

DROP TRIGGER IF EXISTS update_agent_api_keys_updated_at ON agent_api_keys;
CREATE TRIGGER update_agent_api_keys_updated_at
    BEFORE UPDATE ON agent_api_keys FOR EACH ROW EXECUTE FUNCTION trigger_update_updated_at();

DROP TRIGGER IF EXISTS update_user_rate_limits_updated_at ON user_rate_limits;
CREATE TRIGGER update_user_rate_limits_updated_at
    BEFORE UPDATE ON user_rate_limits FOR EACH ROW EXECUTE FUNCTION trigger_update_updated_at();

-- =============================================================================
-- LOGICAL REPLICATION SETUP (for syncing user_references to Agents DB)
-- =============================================================================

DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'nullblock_replicator') THEN
        CREATE USER nullblock_replicator WITH REPLICATION PASSWORD 'nullblock_replication_secure_2024';
    END IF;
END $$;

GRANT CONNECT ON DATABASE erebus TO nullblock_replicator;
GRANT USAGE ON SCHEMA public TO nullblock_replicator;
GRANT SELECT ON user_references TO nullblock_replicator;

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_publication WHERE pubname = 'erebus_user_sync') THEN
        CREATE PUBLICATION erebus_user_sync FOR TABLE user_references;
    END IF;
END $$;
