-- Migration: Create sync functions from Erebus to Agents database
-- This migration creates functions to sync user data from Erebus (owner) to Agents (cache)

-- Create function to sync user from Erebus to Agents database
CREATE OR REPLACE FUNCTION sync_user_to_agents(
    p_user_id UUID,
    p_source_identifier VARCHAR,
    p_chain VARCHAR,
    p_source_type VARCHAR DEFAULT 'web3_wallet',
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
        p_user_id, p_source_identifier, p_chain, p_source_type, p_source_type, 
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

-- Create trigger to automatically sync user changes to agents database
CREATE OR REPLACE FUNCTION trigger_sync_user_to_agents() RETURNS TRIGGER AS $$
BEGIN
    -- Sync the user data to agents database
    PERFORM sync_user_to_agents(
        NEW.id,
        NEW.source_identifier,
        NEW.chain,
        NEW.source_type,
        NEW.wallet_type,
        NEW.email,
        NEW.metadata,
        NEW.preferences,
        NEW.additional_metadata
    );
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger on user_references table
DROP TRIGGER IF EXISTS trigger_sync_user_to_agents ON user_references;
CREATE TRIGGER trigger_sync_user_to_agents
    AFTER INSERT OR UPDATE ON user_references
    FOR EACH ROW
    EXECUTE FUNCTION trigger_sync_user_to_agents();

-- Add comments for documentation
COMMENT ON FUNCTION sync_user_to_agents IS 'Syncs user data from Erebus (owner) to Agents database (read-only cache)';
COMMENT ON FUNCTION trigger_sync_user_to_agents IS 'Automatically syncs user changes to agents database';



