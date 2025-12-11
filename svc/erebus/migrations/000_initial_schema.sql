-- Migration: Initial Schema - User References and API Key Management
-- This migration creates the foundational schema for Erebus including:
-- 1. User references table (source-agnostic user identity)
-- 2. User API keys table (encrypted LLM provider keys)
-- 3. Logical replication setup for user sync to Agents database

-- =============================================================================
-- PART 1: USER REFERENCES TABLE
-- =============================================================================

CREATE TABLE IF NOT EXISTS user_references (
    -- Primary identification
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

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
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for user_references
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

-- Add column comments
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

-- =============================================================================
-- PART 2: USER API KEYS TABLE
-- =============================================================================

CREATE TABLE IF NOT EXISTS user_api_keys (
    -- Primary identification
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- User association
    user_id UUID NOT NULL REFERENCES user_references(id) ON DELETE CASCADE,

    -- Provider identification
    provider VARCHAR NOT NULL CHECK (provider IN ('openai', 'anthropic', 'groq', 'openrouter', 'huggingface', 'ollama')),

    -- Encryption fields (AES-256-GCM)
    encrypted_key BYTEA NOT NULL,
    encryption_iv BYTEA NOT NULL,
    encryption_tag BYTEA NOT NULL,

    -- Display fields (for UI, never show full key after creation)
    key_prefix VARCHAR(20),
    key_suffix VARCHAR(20),
    key_name VARCHAR(255),

    -- Usage tracking
    last_used_at TIMESTAMPTZ,
    usage_count BIGINT DEFAULT 0,

    -- System fields
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Ensure one active key per user per provider
    CONSTRAINT user_api_keys_user_provider_unique UNIQUE (user_id, provider)
        DEFERRABLE INITIALLY DEFERRED
);

-- Create indexes for user_api_keys
CREATE INDEX IF NOT EXISTS idx_user_api_keys_user_id ON user_api_keys(user_id);
CREATE INDEX IF NOT EXISTS idx_user_api_keys_provider ON user_api_keys(provider);
CREATE INDEX IF NOT EXISTS idx_user_api_keys_is_active ON user_api_keys(is_active);
CREATE INDEX IF NOT EXISTS idx_user_api_keys_last_used_at ON user_api_keys(last_used_at);
CREATE INDEX IF NOT EXISTS idx_user_api_keys_created_at ON user_api_keys(created_at);

-- Add column comments
COMMENT ON TABLE user_api_keys IS 'Encrypted LLM provider API keys for registered users';
COMMENT ON COLUMN user_api_keys.id IS 'Primary key UUID for API key record';
COMMENT ON COLUMN user_api_keys.user_id IS 'Foreign key to user_references table';
COMMENT ON COLUMN user_api_keys.provider IS 'LLM provider: openai, anthropic, groq, openrouter, huggingface, ollama';
COMMENT ON COLUMN user_api_keys.encrypted_key IS 'AES-256-GCM encrypted API key (ciphertext)';
COMMENT ON COLUMN user_api_keys.encryption_iv IS 'Initialization vector for AES-GCM (12 bytes)';
COMMENT ON COLUMN user_api_keys.encryption_tag IS 'Authentication tag for AES-GCM (16 bytes)';
COMMENT ON COLUMN user_api_keys.key_prefix IS 'First few characters of key for display (e.g., "sk-proj-abc")';
COMMENT ON COLUMN user_api_keys.key_suffix IS 'Last few characters of key for display (e.g., "xyz")';
COMMENT ON COLUMN user_api_keys.key_name IS 'User-friendly name for the key (optional)';
COMMENT ON COLUMN user_api_keys.last_used_at IS 'Timestamp of last usage';
COMMENT ON COLUMN user_api_keys.usage_count IS 'Number of times key has been used';
COMMENT ON COLUMN user_api_keys.is_active IS 'Soft delete flag - false means key is revoked';
COMMENT ON COLUMN user_api_keys.created_at IS 'Timestamp when key was added';
COMMENT ON COLUMN user_api_keys.updated_at IS 'Timestamp when key was last updated';

-- =============================================================================
-- PART 3: TRIGGER FUNCTIONS
-- =============================================================================

-- Create function to validate source_type structure
CREATE OR REPLACE FUNCTION validate_source_type(source_type_data JSONB) RETURNS BOOLEAN AS $$
BEGIN
    IF NOT (source_type_data ? 'type' AND source_type_data->>'type' IS NOT NULL) THEN
        RETURN FALSE;
    END IF;

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

-- Create the validation trigger for user_references
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

-- Create the updated_at triggers
DROP TRIGGER IF EXISTS trigger_update_updated_at_user_references ON user_references;
CREATE TRIGGER trigger_update_updated_at_user_references
    BEFORE UPDATE ON user_references
    FOR EACH ROW EXECUTE FUNCTION trigger_update_updated_at();

DROP TRIGGER IF EXISTS trigger_update_updated_at_user_api_keys ON user_api_keys;
CREATE TRIGGER trigger_update_updated_at_user_api_keys
    BEFORE UPDATE ON user_api_keys
    FOR EACH ROW EXECUTE FUNCTION trigger_update_updated_at();

-- Create function to track API key usage
CREATE OR REPLACE FUNCTION increment_api_key_usage(key_id UUID) RETURNS void AS $$
BEGIN
    UPDATE user_api_keys
    SET
        usage_count = usage_count + 1,
        last_used_at = NOW()
    WHERE id = key_id AND is_active = true;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION increment_api_key_usage IS 'Increments usage count and updates last_used_at for an API key';

-- =============================================================================
-- PART 4: LOGICAL REPLICATION SETUP
-- =============================================================================

-- Create replication user with appropriate permissions
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'nullblock_replicator') THEN
        CREATE USER nullblock_replicator WITH REPLICATION PASSWORD 'nullblock_replication_secure_2024';
        RAISE NOTICE 'Created replication user: nullblock_replicator';
    ELSE
        RAISE NOTICE 'Replication user nullblock_replicator already exists';
    END IF;
END
$$;

-- Grant necessary permissions to replication user
GRANT CONNECT ON DATABASE erebus TO nullblock_replicator;
GRANT USAGE ON SCHEMA public TO nullblock_replicator;
GRANT SELECT ON user_references TO nullblock_replicator;

-- Create publication for user_references table
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_publication WHERE pubname = 'erebus_user_sync') THEN
        CREATE PUBLICATION erebus_user_sync FOR TABLE user_references;
        RAISE NOTICE 'Created publication: erebus_user_sync for table user_references';
    ELSE
        RAISE NOTICE 'Publication erebus_user_sync already exists';
    END IF;
END
$$;

COMMENT ON PUBLICATION erebus_user_sync IS 'Logical replication publication for syncing user_references to Agents database';

-- Create monitoring view for replication status
CREATE OR REPLACE VIEW replication_status AS
SELECT
    'erebus_user_sync' as publication_name,
    COUNT(*) as published_tables,
    string_agg(pt.schemaname || '.' || pt.tablename, ', ') as table_list
FROM pg_publication_tables pt
WHERE pt.pubname = 'erebus_user_sync';

-- =============================================================================
-- FINAL VERIFICATION
-- =============================================================================

DO $$
DECLARE
    user_table_count int;
    api_key_table_count int;
    pub_count int;
BEGIN
    SELECT COUNT(*) INTO user_table_count FROM information_schema.tables WHERE table_name = 'user_references';
    SELECT COUNT(*) INTO api_key_table_count FROM information_schema.tables WHERE table_name = 'user_api_keys';
    SELECT COUNT(*) INTO pub_count FROM pg_publication WHERE pubname = 'erebus_user_sync';

    RAISE NOTICE '====================================================';
    RAISE NOTICE 'EREBUS INITIAL SCHEMA SETUP COMPLETE';
    RAISE NOTICE '====================================================';
    RAISE NOTICE 'Tables created:';
    RAISE NOTICE '  - user_references: %', CASE WHEN user_table_count > 0 THEN '✅' ELSE '❌' END;
    RAISE NOTICE '  - user_api_keys: %', CASE WHEN api_key_table_count > 0 THEN '✅' ELSE '❌' END;
    RAISE NOTICE 'Publications created: %', pub_count;
    RAISE NOTICE '';
    RAISE NOTICE 'Next steps:';
    RAISE NOTICE '1. Ensure PostgreSQL config has wal_level = logical';
    RAISE NOTICE '2. Restart Erebus PostgreSQL container if config changed';
    RAISE NOTICE '3. Run subscription setup on Agents database (port 5441)';
    RAISE NOTICE '4. Generate encryption master key: ./scripts/generate-encryption-key.sh';
    RAISE NOTICE '5. Add ENCRYPTION_MASTER_KEY to .env.dev';
    RAISE NOTICE '';
    RAISE NOTICE 'Replication user: nullblock_replicator';
    RAISE NOTICE 'Publication: erebus_user_sync';
    RAISE NOTICE '====================================================';
END
$$;
