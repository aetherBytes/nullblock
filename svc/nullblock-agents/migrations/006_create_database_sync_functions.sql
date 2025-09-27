-- Migration: Create database sync functions and triggers
-- This migration creates functions to sync user data between Erebus and Agents databases
-- and automatically populate source_identifier and source_metadata when tasks are created

-- Create function to sync user from Erebus to Agents database
CREATE OR REPLACE FUNCTION sync_user_from_erebus(
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
BEGIN
    -- Insert or update user_references table
    INSERT INTO user_references (
        id, source_identifier, chain, user_type, source_type, wallet_type, email, 
        metadata, preferences, additional_metadata,
        synced_at, is_active, erebus_created_at, erebus_updated_at
    ) VALUES (
        p_user_id, p_source_identifier, p_chain, p_source_type, p_source_type, p_wallet_type, p_email,
        p_metadata, p_preferences, p_additional_metadata,
        NOW(), true, NOW(), NOW()
    )
    ON CONFLICT (source_identifier, chain) 
    DO UPDATE SET
        user_type = EXCLUDED.user_type,
        source_type = EXCLUDED.source_type,
        wallet_type = EXCLUDED.wallet_type,
        email = EXCLUDED.email,
        metadata = EXCLUDED.metadata,
        preferences = EXCLUDED.preferences,
        additional_metadata = EXCLUDED.additional_metadata,
        synced_at = NOW(),
        is_active = true,
        erebus_updated_at = NOW();
END;
$$ LANGUAGE plpgsql;

-- Create function to populate source fields when task is created
CREATE OR REPLACE FUNCTION populate_task_source_fields() RETURNS TRIGGER AS $$
DECLARE
    user_record RECORD;
BEGIN
    -- If user_id is provided, get user details and populate source fields
    IF NEW.user_id IS NOT NULL THEN
        SELECT source_identifier, chain, user_type, source_type, wallet_type, 
               metadata, additional_metadata
        INTO user_record
        FROM user_references 
        WHERE id = NEW.user_id AND is_active = true;
        
        IF FOUND THEN
            -- Populate source_identifier with the user's source identifier
            NEW.source_identifier := user_record.source_identifier;
            
            -- Populate source_metadata with enhanced user details
            NEW.source_metadata := jsonb_build_object(
                'type', user_record.source_type,
                'chain', user_record.chain,
                'wallet_type', user_record.wallet_type,
                'source_identifier', user_record.source_identifier,
                'user_id', NEW.user_id,
                'user_metadata', user_record.metadata,
                'additional_metadata', user_record.additional_metadata
            );
        END IF;
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger to automatically populate source fields
CREATE TRIGGER trigger_populate_task_source_fields
    BEFORE INSERT ON task_executions
    FOR EACH ROW
    EXECUTE FUNCTION populate_task_source_fields();

-- Create function to get source display information
CREATE OR REPLACE FUNCTION get_source_display_info(p_source_metadata JSONB) 
RETURNS TABLE(display_type VARCHAR, display_value VARCHAR) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        COALESCE(p_source_metadata->>'type', 'unknown')::VARCHAR as display_type,
        CASE 
            WHEN p_source_metadata->>'type' = 'web3_wallet' THEN 
                COALESCE(p_source_metadata->>'wallet_address', 'Unknown Wallet')
            WHEN p_source_metadata->>'type' = 'agent' THEN 
                COALESCE(p_source_metadata->>'agent_id', 'Unknown Agent')
            WHEN p_source_metadata->>'type' = 'api' THEN 
                COALESCE(p_source_metadata->>'api_key_name', 'API Call')
            WHEN p_source_metadata->>'type' = 'system' THEN 
                'System Generated'
            ELSE 
                COALESCE(p_source_metadata->>'identifier', 'Unknown Source')
        END::VARCHAR as display_value;
END;
$$ LANGUAGE plpgsql;

-- Create function to sync task execution back to Erebus (if needed)
CREATE OR REPLACE FUNCTION sync_task_to_erebus(
    p_task_id UUID,
    p_status VARCHAR,
    p_progress INTEGER,
    p_result JSONB DEFAULT NULL
) RETURNS VOID AS $$
BEGIN
    -- This would be implemented based on Erebus API requirements
    -- For now, we'll just log the sync attempt
    RAISE NOTICE 'Syncing task % to Erebus with status %', p_task_id, p_status;
    
    -- TODO: Implement actual sync to Erebus database/API
    -- This could involve:
    -- 1. HTTP API calls to Erebus
    -- 2. Kafka message publishing
    -- 3. Direct database connection to Erebus
END;
$$ LANGUAGE plpgsql;

-- Add comments for documentation
COMMENT ON FUNCTION sync_user_from_erebus IS 'Syncs user data from Erebus to Agents database';
COMMENT ON FUNCTION populate_task_source_fields IS 'Automatically populates source_identifier and source_metadata when tasks are created';
COMMENT ON FUNCTION get_source_display_info IS 'Returns formatted display information for task sources';
COMMENT ON FUNCTION sync_task_to_erebus IS 'Syncs task execution status back to Erebus database';
