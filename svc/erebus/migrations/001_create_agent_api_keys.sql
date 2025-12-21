-- Agent API Keys Table
-- Stores encrypted API keys for agents (HECATE, Siren, etc.)
-- Uses same AES-256-GCM encryption as user_api_keys

CREATE TABLE IF NOT EXISTS agent_api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_name VARCHAR NOT NULL,  -- 'hecate', 'siren', etc.
    provider VARCHAR NOT NULL CHECK (provider IN ('openai', 'anthropic', 'groq', 'openrouter', 'huggingface', 'ollama')),

    -- Encryption fields (AES-256-GCM)
    encrypted_key BYTEA NOT NULL,
    encryption_iv BYTEA NOT NULL,
    encryption_tag BYTEA NOT NULL,

    -- Display fields (never show full key)
    key_prefix VARCHAR(20),
    key_suffix VARCHAR(20),
    key_name VARCHAR(255),

    -- Usage tracking
    last_used_at TIMESTAMPTZ,
    usage_count BIGINT DEFAULT 0,

    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    -- One key per agent per provider
    CONSTRAINT agent_api_keys_agent_provider_unique UNIQUE (agent_name, provider)
);

-- Indexes for efficient lookups
CREATE INDEX IF NOT EXISTS idx_agent_api_keys_agent_name ON agent_api_keys(agent_name);
CREATE INDEX IF NOT EXISTS idx_agent_api_keys_provider ON agent_api_keys(provider);
CREATE INDEX IF NOT EXISTS idx_agent_api_keys_is_active ON agent_api_keys(is_active);
