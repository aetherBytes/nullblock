-- Migration: Create user_references table
-- READ-ONLY sync cache from Erebus user_references table
-- Populated via PostgreSQL logical replication
-- Schema matches Erebus exactly for seamless replication

CREATE TABLE IF NOT EXISTS user_references (
    id UUID PRIMARY KEY,
    source_identifier VARCHAR(255) NOT NULL,
    network VARCHAR(50) NOT NULL,
    user_type VARCHAR(50) DEFAULT 'external',
    email VARCHAR(255),

    -- User metadata and preferences
    metadata JSONB DEFAULT '{}'::jsonb,
    preferences JSONB DEFAULT '{}'::jsonb,
    additional_metadata JSONB DEFAULT '{}'::jsonb,

    -- Source type (replaces old user_type with structured data)
    source_type JSONB DEFAULT '{"type": "web3_wallet", "provider": "unknown", "metadata": {}}'::jsonb,

    -- Timestamps from Erebus
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    -- Sync tracking
    is_active BOOLEAN DEFAULT true
);

-- Indexes for performance and lookups
CREATE INDEX IF NOT EXISTS idx_user_references_source_identifier ON user_references(source_identifier);
CREATE INDEX IF NOT EXISTS idx_user_references_network ON user_references(network);
CREATE INDEX IF NOT EXISTS idx_user_references_user_type ON user_references(user_type);
CREATE INDEX IF NOT EXISTS idx_user_references_email ON user_references(email);
CREATE INDEX IF NOT EXISTS idx_user_references_is_active ON user_references(is_active);
CREATE INDEX IF NOT EXISTS idx_user_references_created_at ON user_references(created_at);
CREATE INDEX IF NOT EXISTS idx_user_references_source_type_gin ON user_references USING gin(source_type);
CREATE INDEX IF NOT EXISTS idx_user_references_source_type_type ON user_references((source_type->>'type'));
CREATE INDEX IF NOT EXISTS idx_user_references_source_type_provider ON user_references((source_type->>'provider'));

-- Unique constraints
CREATE UNIQUE INDEX IF NOT EXISTS user_references_source_network_unique ON user_references(source_identifier, network)
    WHERE source_identifier IS NOT NULL;
