-- Migration: Make user_references table source-type agnostic
-- This migration renames 'chain' to 'network' and enhances source_type structure
-- to support multiple source types beyond web3 wallets (agents, APIs, email auth, etc.)

-- Step 1: Rename 'chain' column to 'network' for source-agnostic terminology
ALTER TABLE user_references RENAME COLUMN chain TO network;

-- Step 2: Update unique constraint to use new column name
DROP INDEX IF EXISTS user_references_source_chain_unique;
CREATE UNIQUE INDEX user_references_source_network_unique ON user_references(source_identifier, network)
    WHERE source_identifier IS NOT NULL;

-- Step 3: Update network index
DROP INDEX IF EXISTS idx_user_references_chain;
CREATE INDEX idx_user_references_network ON user_references(network);

-- Step 4: Standardize source_type JSONB structure for different source types
-- Current web3_wallet entries will be preserved and standardized
UPDATE user_references
SET source_type = jsonb_build_object(
    'type', 'web3_wallet',
    'provider', COALESCE(wallet_type, 'unknown'),
    'network', network,
    'metadata', COALESCE(source_type->'metadata', '{}'::jsonb)
)
WHERE source_type->>'type' = 'web3_wallet' OR wallet_type IS NOT NULL;

-- Step 5: Remove wallet_type column as it's now consolidated into source_type
-- First, ensure any remaining wallet_type data is preserved in source_type
UPDATE user_references
SET source_type = jsonb_build_object(
    'type', 'web3_wallet',
    'provider', wallet_type,
    'network', network,
    'metadata', '{}'::jsonb
)
WHERE wallet_type IS NOT NULL AND (source_type IS NULL OR source_type->>'type' IS NULL);

-- Now safe to drop wallet_type
ALTER TABLE user_references DROP COLUMN IF EXISTS wallet_type;
DROP INDEX IF EXISTS idx_user_references_wallet_type;

-- Step 6: Update source_type indexes for efficient querying
DROP INDEX IF EXISTS idx_user_references_source_type;
DROP INDEX IF EXISTS idx_user_references_source_type_gin;
DROP INDEX IF EXISTS idx_user_references_source_type_type;

-- Create optimized indexes for source type queries
CREATE INDEX idx_user_references_source_type_gin ON user_references USING GIN (source_type);
CREATE INDEX idx_user_references_source_type_type ON user_references ((source_type->>'type'));
CREATE INDEX idx_user_references_source_type_provider ON user_references ((source_type->>'provider'));

-- Step 7: Add constraints to ensure source_type has required structure
ALTER TABLE user_references ADD CONSTRAINT check_source_type_structure
    CHECK (source_type ? 'type' AND source_type->>'type' IS NOT NULL);

-- Step 8: Update column comments for documentation
COMMENT ON COLUMN user_references.source_identifier IS 'Generic identifier for user source: wallet address, email, API key, agent ID, etc.';
COMMENT ON COLUMN user_references.network IS 'Network/context for the source: ethereum, solana, email, api, system, etc.';
COMMENT ON COLUMN user_references.source_type IS 'Structured source type metadata: {"type": "web3_wallet|api_key|email_auth|system_agent|oauth", "provider": "...", "metadata": {...}}';
COMMENT ON COLUMN user_references.user_type IS 'High-level user category: external, system, agent, api';

-- Step 9: Create helper function to validate source_type structure
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

-- Step 10: Add validation trigger for source_type
CREATE OR REPLACE FUNCTION trigger_validate_source_type() RETURNS TRIGGER AS $$
BEGIN
    IF NOT validate_source_type(NEW.source_type) THEN
        RAISE EXCEPTION 'Invalid source_type structure: %', NEW.source_type;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_validate_source_type_insert_update
    BEFORE INSERT OR UPDATE ON user_references
    FOR EACH ROW EXECUTE FUNCTION trigger_validate_source_type();