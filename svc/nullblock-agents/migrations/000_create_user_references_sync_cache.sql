-- Migration: Create user_references sync cache table in Agents database
-- This table is a read-only cache populated via Kafka events from Erebus
-- Used for task foreign key validation and user attribution

-- Create the user_references sync cache table
CREATE TABLE user_references (
    -- Primary identification (mirrors Erebus exactly)
    id UUID PRIMARY KEY,
    
    -- Source identification (agnostic to source type)
    source_identifier VARCHAR NOT NULL,
    network VARCHAR NOT NULL,
    
    -- User classification
    user_type VARCHAR NOT NULL DEFAULT 'external',
    
    -- Source type metadata (JSONB for flexibility)
    source_type JSONB NOT NULL DEFAULT '{"type": "web3_wallet", "provider": "unknown", "metadata": {}}'::jsonb,
    
    -- Optional user data
    email VARCHAR,
    metadata JSONB DEFAULT '{}'::jsonb,
    preferences JSONB DEFAULT '{}'::jsonb,
    
    -- System fields
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Sync tracking fields
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    erebus_created_at TIMESTAMPTZ,
    erebus_updated_at TIMESTAMPTZ
);

-- Create indexes for performance
CREATE INDEX idx_user_references_source_identifier ON user_references(source_identifier);
CREATE INDEX idx_user_references_network ON user_references(network);
CREATE INDEX idx_user_references_user_type ON user_references(user_type);
CREATE INDEX idx_user_references_email ON user_references(email);
CREATE INDEX idx_user_references_is_active ON user_references(is_active);
CREATE INDEX idx_user_references_synced_at ON user_references(synced_at);

-- JSONB indexes for source_type queries
CREATE INDEX idx_user_references_source_type_gin ON user_references USING GIN (source_type);
CREATE INDEX idx_user_references_source_type_type ON user_references ((source_type->>'type'));
CREATE INDEX idx_user_references_source_type_provider ON user_references ((source_type->>'provider'));

-- JSONB indexes for metadata and preferences
CREATE INDEX idx_user_references_metadata_gin ON user_references USING GIN (metadata);
CREATE INDEX idx_user_references_preferences_gin ON user_references USING GIN (preferences);

-- Unique constraints (same as Erebus)
CREATE UNIQUE INDEX user_references_source_network_unique ON user_references(source_identifier, network)
    WHERE source_identifier IS NOT NULL AND is_active = true;
CREATE UNIQUE INDEX user_references_email_unique ON user_references(email)
    WHERE email IS NOT NULL AND is_active = true;

-- Add column comments for documentation
COMMENT ON TABLE user_references IS 'Read-only sync cache of user_references from Erebus database. Populated via Kafka events.';
COMMENT ON COLUMN user_references.id IS 'Primary key UUID (mirrors Erebus user_references.id)';
COMMENT ON COLUMN user_references.source_identifier IS 'Generic identifier for user source: wallet address, email, API key, agent ID, OAuth user ID, etc.';
COMMENT ON COLUMN user_references.network IS 'Network/context for the source: ethereum, solana, email, api, system, oauth, etc.';
COMMENT ON COLUMN user_references.user_type IS 'High-level user category: external, system, agent, api';
COMMENT ON COLUMN user_references.source_type IS 'Structured source type metadata: {"type": "web3_wallet|api_key|email_auth|system_agent|oauth", "provider": "...", "metadata": {...}}';
COMMENT ON COLUMN user_references.email IS 'Email address if available for the user';
COMMENT ON COLUMN user_references.metadata IS 'General user metadata and profile information';
COMMENT ON COLUMN user_references.preferences IS 'User preferences and settings';
COMMENT ON COLUMN user_references.is_active IS 'Soft delete flag - false means user is deactivated';
COMMENT ON COLUMN user_references.synced_at IS 'Timestamp when this record was synced from Erebus';
COMMENT ON COLUMN user_references.erebus_created_at IS 'Original creation timestamp from Erebus';
COMMENT ON COLUMN user_references.erebus_updated_at IS 'Original update timestamp from Erebus';
