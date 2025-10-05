-- Migration: Create user_references table with source-agnostic schema
-- This is the foundational migration that creates the user_references table
-- with the correct schema to support multiple source types from the start

-- Create the user_references table with source-agnostic design
CREATE TABLE IF NOT EXISTS user_references (
    -- Primary identification
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Source identification (agnostic to source type)
    source_identifier VARCHAR NOT NULL,  -- wallet address, email, API key, agent ID, etc.
    network VARCHAR NOT NULL,            -- ethereum, solana, email, api, system, oauth, etc.
    
    -- User classification
    user_type VARCHAR NOT NULL DEFAULT 'external',  -- external, system, agent, api
    
    -- Source type metadata (JSONB for flexibility)
    source_type JSONB NOT NULL DEFAULT '{"type": "web3_wallet", "provider": "unknown", "metadata": {}}'::jsonb,
    
    -- Optional user data
    email VARCHAR,                       -- Email address if available
    metadata JSONB DEFAULT '{}'::jsonb,   -- General user metadata
    preferences JSONB DEFAULT '{}'::jsonb, -- User preferences
    
    -- System fields
    is_active BOOLEAN DEFAULT true,     -- Soft delete flag
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_user_references_source_identifier ON user_references(source_identifier);
CREATE INDEX IF NOT EXISTS idx_user_references_network ON user_references(network);
CREATE INDEX IF NOT EXISTS idx_user_references_user_type ON user_references(user_type);
CREATE INDEX IF NOT EXISTS idx_user_references_email ON user_references(email);
CREATE INDEX IF NOT EXISTS idx_user_references_is_active ON user_references(is_active);
CREATE INDEX IF NOT EXISTS idx_user_references_created_at ON user_references(created_at);

-- JSONB indexes for source_type queries
CREATE INDEX IF NOT EXISTS idx_user_references_source_type_gin ON user_references USING GIN (source_type);
CREATE INDEX IF NOT EXISTS idx_user_references_source_type_type ON user_references ((source_type->>'type'));
CREATE INDEX IF NOT EXISTS idx_user_references_source_type_provider ON user_references ((source_type->>'provider'));

-- JSONB indexes for metadata and preferences
CREATE INDEX IF NOT EXISTS idx_user_references_metadata_gin ON user_references USING GIN (metadata);
CREATE INDEX IF NOT EXISTS idx_user_references_preferences_gin ON user_references USING GIN (preferences);

-- Unique constraints
CREATE UNIQUE INDEX IF NOT EXISTS user_references_source_network_unique ON user_references(source_identifier, network)
    WHERE source_identifier IS NOT NULL AND is_active = true;
CREATE UNIQUE INDEX IF NOT EXISTS user_references_email_unique ON user_references(email)
    WHERE email IS NOT NULL AND is_active = true;

-- Add column comments for documentation
COMMENT ON TABLE user_references IS 'Source-agnostic user references supporting Web3 wallets, API keys, email auth, system agents, and OAuth';
COMMENT ON COLUMN user_references.id IS 'Primary key UUID for user reference';
COMMENT ON COLUMN user_references.source_identifier IS 'Generic identifier for user source: wallet address, email, API key, agent ID, OAuth user ID, etc.';
COMMENT ON COLUMN user_references.network IS 'Network/context for the source: ethereum, solana, email, api, system, oauth, etc.';
COMMENT ON COLUMN user_references.user_type IS 'High-level user category: external, system, agent, api';
COMMENT ON COLUMN user_references.source_type IS 'Structured source type metadata: {"type": "web3_wallet|api_key|email_auth|system_agent|oauth", "provider": "...", "metadata": {...}}';
COMMENT ON COLUMN user_references.email IS 'Email address if available for the user';
COMMENT ON COLUMN user_references.metadata IS 'General user metadata and profile information';
COMMENT ON COLUMN user_references.preferences IS 'User preferences and settings';
COMMENT ON COLUMN user_references.is_active IS 'Soft delete flag - false means user is deactivated';
COMMENT ON COLUMN user_references.created_at IS 'Timestamp when user reference was created';
COMMENT ON COLUMN user_references.updated_at IS 'Timestamp when user reference was last updated';

-- Create function to validate source_type structure
CREATE OR REPLACE FUNCTION validate_source_type(source_type_data JSONB) RETURNS BOOLEAN AS $$
BEGIN
    -- Ensure required 'type' field exists
    IF NOT (source_type_data ? 'type' AND source_type_data->>'type' IS NOT NULL) THEN
        RETURN FALSE;
    END IF;

    -- Validate known source types have appropriate structure
    CASE source_type_data->>'type'
        WHEN 'web3_wallet' THEN
            RETURN source_type_data ? 'provider' AND source_type_data ? 'network';
        WHEN 'api_key' THEN
            RETURN source_type_data ? 'name';
        WHEN 'email_auth' THEN
            RETURN source_type_data ? 'provider';
        WHEN 'system_agent' THEN
            RETURN source_type_data ? 'agent_type';
        WHEN 'oauth' THEN
            RETURN source_type_data ? 'provider';
        ELSE
            -- Allow unknown types for future extensibility
            RETURN TRUE;
    END CASE;
END;
$$ LANGUAGE plpgsql;

-- Create trigger function to validate source_type on insert/update
CREATE OR REPLACE FUNCTION trigger_validate_source_type() RETURNS TRIGGER AS $$
BEGIN
    IF NOT validate_source_type(NEW.source_type) THEN
        RAISE EXCEPTION 'Invalid source_type structure: %. Required structure depends on type.', NEW.source_type;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create the validation trigger
DROP TRIGGER IF EXISTS trigger_validate_source_type_insert_update ON user_references;
CREATE TRIGGER trigger_validate_source_type_insert_update
    BEFORE INSERT OR UPDATE ON user_references
    FOR EACH ROW EXECUTE FUNCTION trigger_validate_source_type();

-- Create function to update updated_at timestamp
CREATE OR REPLACE FUNCTION trigger_update_updated_at() RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create the updated_at trigger
DROP TRIGGER IF EXISTS trigger_update_updated_at ON user_references;
CREATE TRIGGER trigger_update_updated_at
    BEFORE UPDATE ON user_references
    FOR EACH ROW EXECUTE FUNCTION trigger_update_updated_at();


