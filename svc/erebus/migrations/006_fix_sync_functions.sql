-- Migration: Fix sync functions for JSONB source_type
-- This migration updates the sync functions to work with JSONB source_type

-- Drop the problematic sync function
DROP FUNCTION IF EXISTS sync_user_to_agents(UUID, VARCHAR, VARCHAR, VARCHAR, VARCHAR, VARCHAR, JSONB, JSONB, JSONB);

-- Create a simpler sync function that just logs the change
CREATE OR REPLACE FUNCTION sync_user_to_agents() RETURNS TRIGGER AS $$
BEGIN
    -- Log the user change for application-level sync processing
    RAISE NOTICE 'User % changed: % -> %', 
        COALESCE(NEW.id, OLD.id),
        TG_OP,
        COALESCE(NEW.source_type, OLD.source_type);
    
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

-- Update the trigger to use the simpler function
DROP TRIGGER IF EXISTS trigger_sync_user_to_agents ON user_references;
CREATE TRIGGER trigger_sync_user_to_agents
    AFTER INSERT OR UPDATE ON user_references
    FOR EACH ROW
    EXECUTE FUNCTION sync_user_to_agents();

-- Create a function to get source display information
CREATE OR REPLACE FUNCTION get_source_display_info(
    p_source_identifier VARCHAR,
    p_source_type JSONB
) RETURNS VARCHAR AS $$
DECLARE
    source_type_str VARCHAR;
    provider_str VARCHAR;
    display_value VARCHAR;
BEGIN
    source_type_str := COALESCE(p_source_type->>'type', 'unknown');
    provider_str := COALESCE(p_source_type->>'provider', 'unknown');
    
    CASE source_type_str
        WHEN 'web3_wallet' THEN
            display_value := COALESCE(p_source_identifier, 'Unknown Wallet');
            IF provider_str != 'unknown' THEN
                display_value := display_value || ' (' || provider_str || ')';
            END IF;
        WHEN 'agent' THEN
            display_value := COALESCE(p_source_identifier, 'Unknown Agent');
            IF provider_str != 'unknown' THEN
                display_value := display_value || ' (' || provider_str || ')';
            END IF;
        WHEN 'api' THEN
            display_value := COALESCE(p_source_identifier, 'Unknown API');
            IF provider_str != 'unknown' THEN
                display_value := display_value || ' (' || provider_str || ')';
            END IF;
        ELSE
            display_value := COALESCE(p_source_identifier, 'Unknown Source');
    END CASE;

    RETURN COALESCE(source_type_str || ': ' || display_value, 'Unknown Source');
END;
$$ LANGUAGE plpgsql;

-- Add comments
COMMENT ON FUNCTION sync_user_to_agents IS 'Logs user changes for application-level sync processing';
COMMENT ON FUNCTION get_source_display_info IS 'Generates human-readable source information from JSONB source_type';



