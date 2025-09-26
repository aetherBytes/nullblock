# Database Linkage System

This document describes the linkage between the Erebus database and the Agents database, including the new source tracking system for task executions.

## Overview

The system now provides a comprehensive audit trail for task creation and execution, linking user information from Erebus with task execution data in the Agents database.

## Database Schema Changes

### Table Rename
- `tasks` table renamed to `task_executions` for better semantic clarity
- All existing data is preserved during the migration

### New Source Tracking Fields

#### `source_identifier` (VARCHAR)
- Generic identifier for the source that created the task
- Examples:
  - Wallet address: `"0x1234...5678"`
  - Agent UUID: `"550e8400-e29b-41d4-a716-446655440000"`
  - API key name: `"webhook_trigger_123"`
  - System identifier: `"cron_job_daily"`

#### `source_metadata` (JSONB)
- Rich metadata about the source
- Structure:
```json
{
  "type": "web3_wallet",
  "chain": "solana",
  "wallet_type": "phantom",
  "user_id": "uuid-here",
  "session_id": "session-uuid",
  "ip_address": "192.168.1.1",
  "user_agent": "Mozilla/5.0...",
  "additional_context": {}
}
```

## Source Types

### Web3 Wallet
```json
{
  "type": "web3_wallet",
  "chain": "solana|ethereum|polygon",
  "wallet_type": "phantom|metamask|coinbase",
  "wallet_address": "0x1234...5678",
  "user_id": "uuid-here"
}
```

### Agent
```json
{
  "type": "agent",
  "agent_id": "agent-uuid",
  "agent_name": "Hecate",
  "agent_type": "hecate|arbitrage|social"
}
```

### API Call
```json
{
  "type": "api",
  "api_key_name": "webhook_trigger_123",
  "endpoint": "/api/tasks/create",
  "ip_address": "192.168.1.1",
  "user_agent": "Mozilla/5.0..."
}
```

### System Generated
```json
{
  "type": "system",
  "trigger": "cron_job_daily",
  "schedule": "0 0 * * *",
  "system_component": "scheduler"
}
```

## Database Sync Functions

### `sync_user_from_erebus()`
- Syncs user data from Erebus to Agents database
- Handles wallet addresses, chain information, and user metadata
- Automatically populates `user_references` table

### `populate_task_source_fields()`
- Trigger function that automatically populates source fields when tasks are created
- Links user information to task execution records
- Ensures audit trail completeness

### `get_source_display_info()`
- Returns formatted display information for task sources
- Handles different source types with appropriate formatting
- Used by frontend for consistent display

## Migration Process

1. **Backup existing data** before running migrations
2. **Run migrations in order**:
   ```bash
   psql -d nullblock_agents -f migrations/007_run_all_migrations.sql
   ```
3. **Verify schema** with the included verification queries
4. **Update application code** to use new field names

## Frontend Integration

### Task Cards
- Display source information using `source_metadata.type`
- Show formatted source identifier
- Maintain backward compatibility with existing data

### Task Details
- Rich source information display
- Audit trail visualization
- Source type indicators with appropriate icons

## API Changes

### Task Creation
- New optional fields: `source_identifier`, `source_metadata`
- Automatic population via database triggers
- Manual override capability for system-generated tasks

### Task Queries
- Filter by source type: `?source_type=web3_wallet`
- Filter by source identifier: `?source_identifier=0x1234...`
- Search within source metadata: `?source_metadata_search=phantom`

## Benefits

1. **Complete Audit Trail**: Every task execution is linked to its source
2. **Flexible Source Types**: Supports wallets, agents, APIs, and system sources
3. **Rich Metadata**: Detailed context about task creation
4. **Database Sync**: Automatic synchronization between Erebus and Agents databases
5. **Backward Compatibility**: Existing data is preserved and migrated
6. **Extensible**: Easy to add new source types and metadata fields

## Usage Examples

### Creating a Task with Source Information
```rust
let task = TaskEntity {
    // ... other fields
    source_identifier: Some("0x1234...5678".to_string()),
    source_metadata: serde_json::json!({
        "type": "web3_wallet",
        "chain": "solana",
        "wallet_type": "phantom",
        "user_id": user_id
    }),
};
```

### Querying Tasks by Source
```sql
-- Get all tasks created by web3 wallets
SELECT * FROM task_executions 
WHERE source_metadata->>'type' = 'web3_wallet';

-- Get tasks from specific wallet
SELECT * FROM task_executions 
WHERE source_identifier = '0x1234...5678';

-- Get tasks with specific metadata
SELECT * FROM task_executions 
WHERE source_metadata @> '{"chain": "solana"}';
```

## Troubleshooting

### Migration Issues
- Check database permissions
- Verify table existence before renaming
- Review foreign key constraints

### Sync Issues
- Monitor `user_references` table for sync status
- Check trigger function execution
- Verify Erebus database connectivity

### Frontend Issues
- Update TypeScript interfaces
- Handle missing source metadata gracefully
- Provide fallback display values
