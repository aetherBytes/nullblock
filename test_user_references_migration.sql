-- Test script to verify user_references migration works correctly
-- This script tests the new source-agnostic schema

-- Test 1: Create a Web3 wallet user
INSERT INTO user_references (
    source_identifier, 
    network, 
    user_type, 
    source_type,
    email
) VALUES (
    '0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6',
    'ethereum',
    'external',
    '{"type": "web3_wallet", "provider": "metamask", "network": "ethereum", "metadata": {"wallet_version": "10.0.0"}}'::jsonb,
    'user@example.com'
);

-- Test 2: Create an API key user
INSERT INTO user_references (
    source_identifier, 
    network, 
    user_type, 
    source_type
) VALUES (
    'api_key_12345',
    'api',
    'external',
    '{"type": "api_key", "name": "trading_bot", "scope": ["read", "write"], "metadata": {"permissions": ["trade", "withdraw"]}}'::jsonb
);

-- Test 3: Create an email authenticated user
INSERT INTO user_references (
    source_identifier, 
    network, 
    user_type, 
    source_type,
    email
) VALUES (
    'user@example.com',
    'email',
    'external',
    '{"type": "email_auth", "verified": true, "provider": "gmail", "metadata": {"last_login": "2025-01-20T10:00:00Z"}}'::jsonb,
    'user@example.com'
);

-- Test 4: Create a system agent user
INSERT INTO user_references (
    source_identifier, 
    network, 
    user_type, 
    source_type
) VALUES (
    'agent_task_runner_001',
    'system',
    'system',
    '{"type": "system_agent", "agent_type": "task_runner", "capabilities": ["execute_tasks", "monitor_status"], "metadata": {"version": "1.0.0"}}'::jsonb
);

-- Test 5: Create an OAuth user
INSERT INTO user_references (
    source_identifier, 
    network, 
    user_type, 
    source_type,
    email
) VALUES (
    'oauth_google_12345',
    'oauth',
    'external',
    '{"type": "oauth", "provider": "google", "user_id": "12345", "metadata": {"access_token_expires": "2025-02-20T10:00:00Z"}}'::jsonb,
    'oauth.user@gmail.com'
);

-- Verify all users were created
SELECT 
    id,
    source_identifier,
    network,
    user_type,
    source_type->>'type' as source_type_name,
    source_type->>'provider' as provider,
    email,
    is_active,
    created_at
FROM user_references 
ORDER BY created_at;

-- Test queries by source type
SELECT 
    source_type->>'type' as source_type_name,
    COUNT(*) as user_count
FROM user_references 
GROUP BY source_type->>'type'
ORDER BY user_count DESC;

-- Test JSONB queries
SELECT 
    source_identifier,
    source_type->>'provider' as provider,
    source_type->'metadata' as metadata
FROM user_references 
WHERE source_type->>'type' = 'web3_wallet';

-- Test network-based queries
SELECT 
    network,
    COUNT(*) as user_count
FROM user_references 
GROUP BY network
ORDER BY user_count DESC;

-- Test user type queries
SELECT 
    user_type,
    COUNT(*) as user_count
FROM user_references 
GROUP BY user_type
ORDER BY user_count DESC;



