-- Migration: Update source_type to JSONB for rich metadata
-- This migration changes source_type from VARCHAR to JSONB to store detailed source information

-- First, let's see what data we currently have
-- We'll need to migrate existing data from VARCHAR to JSONB format

-- Add a temporary column for the new JSONB source_type
ALTER TABLE user_references ADD COLUMN source_type_new JSONB;

-- Migrate existing data from VARCHAR to JSONB format
-- Convert existing source_type values to JSONB with type and metadata
UPDATE user_references 
SET source_type_new = jsonb_build_object(
    'type', COALESCE(source_type, 'web3_wallet'),
    'provider', COALESCE(wallet_type, 'unknown'),
    'metadata', additional_metadata
)
WHERE source_type_new IS NULL;

-- Drop the old VARCHAR column
ALTER TABLE user_references DROP COLUMN source_type;

-- Rename the new column to source_type
ALTER TABLE user_references RENAME COLUMN source_type_new TO source_type;

-- Update the default value for new records
ALTER TABLE user_references ALTER COLUMN source_type SET DEFAULT '{"type": "web3_wallet", "provider": "unknown", "metadata": {}}'::jsonb;

-- Create new indexes for the JSONB field
CREATE INDEX idx_user_references_source_type_gin ON user_references USING GIN (source_type);
CREATE INDEX idx_user_references_source_type_type ON user_references USING BTREE ((source_type->>'type'));

-- Update comments
COMMENT ON COLUMN user_references.source_type IS 'JSONB field containing source type information: {"type": "web3_wallet", "provider": "phantom", "metadata": {...}}';

-- Update the sync functions to work with JSONB source_type
CREATE OR REPLACE FUNCTION trigger_queue_user_sync() RETURNS TRIGGER AS $$
DECLARE
    user_data JSONB;
    operation VARCHAR;
BEGIN
    -- Determine operation type
    IF TG_OP = 'DELETE' THEN
        operation := 'DELETE';
        user_data := to_jsonb(OLD);
    ELSE
        operation := TG_OP;
        user_data := to_jsonb(NEW);
    END IF;
    
    -- Queue the sync operation
    PERFORM queue_user_sync(
        COALESCE(NEW.id, OLD.id),
        operation,
        user_data
    );
    
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

-- Update the sync function to handle JSONB source_type
CREATE OR REPLACE FUNCTION sync_user_to_agents(
    p_user_id UUID,
    p_source_identifier VARCHAR,
    p_chain VARCHAR,
    p_source_type JSONB DEFAULT '{"type": "web3_wallet", "provider": "unknown", "metadata": {}}'::jsonb,
    p_wallet_type VARCHAR DEFAULT NULL,
    p_email VARCHAR DEFAULT NULL,
    p_metadata JSONB DEFAULT '{}'::jsonb,
    p_preferences JSONB DEFAULT '{}'::jsonb,
    p_additional_metadata JSONB DEFAULT '{}'::jsonb
) RETURNS VOID AS $$
DECLARE
    agents_conn_str TEXT := 'postgresql://postgres:postgres_secure_pass@localhost:5441/agents';
    agents_conn CONNECTION;
BEGIN
    -- Connect to agents database
    agents_conn := dblink_connect(agents_conn_str);
    
    -- Insert or update user_references in agents database (read-only cache)
    PERFORM dblink_exec(agents_conn, 
        'INSERT INTO user_references (
            id, source_identifier, chain, user_type, source_type, wallet_type, email, 
            metadata, preferences, additional_metadata, is_active
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        ON CONFLICT (source_identifier, chain) 
        DO UPDATE SET
            user_type = EXCLUDED.user_type,
            source_type = EXCLUDED.source_type,
            wallet_type = EXCLUDED.wallet_type,
            email = EXCLUDED.email,
            metadata = EXCLUDED.metadata,
            preferences = EXCLUDED.preferences,
            additional_metadata = EXCLUDED.additional_metadata,
            is_active = EXCLUDED.is_active',
        p_user_id, p_source_identifier, p_chain, 'external', p_source_type, 
        p_wallet_type, p_email, p_metadata, p_preferences, p_additional_metadata, true
    );
    
    -- Disconnect from agents database
    PERFORM dblink_disconnect(agents_conn);
    
EXCEPTION
    WHEN OTHERS THEN
        -- Log error and continue
        RAISE WARNING 'Failed to sync user to agents database: %', SQLERRM;
        IF agents_conn IS NOT NULL THEN
            PERFORM dblink_disconnect(agents_conn);
        END IF;
END;
$$ LANGUAGE plpgsql;



