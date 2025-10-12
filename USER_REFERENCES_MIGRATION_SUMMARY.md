# User References Migration Summary

## Overview
This document summarizes the comprehensive fix for the user_references table schema to make it truly source-agnostic, supporting multiple authentication types beyond just Web3 wallets.

## Problems Identified

### 1. **Missing Base Table**
- The existing migrations tried to modify a `user_references` table that didn't exist
- No foundational migration to create the table structure

### 2. **Schema Inconsistencies**
- Multiple migration files with conflicting approaches:
  - Some expected `wallet_address` → `source_identifier`
  - Some expected `chain` → `network` 
  - Some expected `source_type` as VARCHAR, others as JSONB
  - References to non-existent `additional_metadata` column

### 3. **Rust Code Mismatch**
- Rust code expected schema that didn't match migration files
- SQL queries referenced non-existent columns

## Solutions Implemented

### 1. **New Foundational Migration** (`000_create_user_references_table.sql`)
Created a comprehensive source-agnostic schema from the start:

```sql
CREATE TABLE user_references (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_identifier VARCHAR NOT NULL,  -- wallet address, email, API key, agent ID, etc.
    network VARCHAR NOT NULL,            -- ethereum, solana, email, api, system, oauth, etc.
    user_type VARCHAR NOT NULL DEFAULT 'external',  -- external, system, agent, api
    source_type JSONB NOT NULL DEFAULT '{"type": "web3_wallet", "provider": "unknown", "metadata": {}}'::jsonb,
    email VARCHAR,                       -- Email address if available
    metadata JSONB DEFAULT '{}'::jsonb,   -- General user metadata
    preferences JSONB DEFAULT '{}'::jsonb, -- User preferences
    is_active BOOLEAN DEFAULT true,     -- Soft delete flag
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### 2. **Source-Agnostic Design**
The new schema supports all authentication types:

#### **Web3 Wallets**
```json
{
  "type": "web3_wallet",
  "provider": "metamask",
  "network": "ethereum",
  "metadata": {"wallet_version": "10.0.0"}
}
```

#### **API Keys**
```json
{
  "type": "api_key", 
  "name": "trading_bot",
  "scope": ["read", "write"],
  "metadata": {"permissions": ["trade", "withdraw"]}
}
```

#### **Email Authentication**
```json
{
  "type": "email_auth",
  "verified": true,
  "provider": "gmail", 
  "metadata": {"last_login": "2025-01-20T10:00:00Z"}
}
```

#### **System Agents**
```json
{
  "type": "system_agent",
  "agent_type": "task_runner",
  "capabilities": ["execute_tasks", "monitor_status"],
  "metadata": {"version": "1.0.0"}
}
```

#### **OAuth**
```json
{
  "type": "oauth",
  "provider": "google",
  "user_id": "12345",
  "metadata": {"access_token_expires": "2025-02-20T10:00:00Z"}
}
```

### 3. **Database Validation**
- Added `validate_source_type()` function to ensure proper JSONB structure
- Added trigger to validate source_type on insert/update
- Added trigger to automatically update `updated_at` timestamp

### 4. **Agents Database Sync Cache** (`000_create_user_references_sync_cache.sql`)
Created a read-only sync cache in the agents database with additional sync tracking fields:
- `synced_at` - When record was synced from Erebus
- `erebus_created_at` - Original creation timestamp from Erebus  
- `erebus_updated_at` - Original update timestamp from Erebus

### 5. **Rust Code Updates**
- Fixed SQL queries to match new schema (removed `additional_metadata` references)
- Updated all database operations to use correct column names
- Maintained backward compatibility with existing SourceType enum

## Key Features

### **Source Agnostic**
- Single table supports all authentication types
- Flexible JSONB structure for source-specific metadata
- Type-safe validation with database triggers

### **Performance Optimized**
- Comprehensive indexing strategy:
  - B-tree indexes for common queries
  - GIN indexes for JSONB fields
  - Unique constraints for data integrity

### **Future Extensible**
- Easy to add new source types without schema changes
- JSONB structure allows for evolving metadata requirements
- Validation function can be extended for new types

### **Data Integrity**
- Database-level validation ensures proper source_type structure
- Unique constraints prevent duplicate users
- Soft delete support with `is_active` flag

## Migration Files Created

1. **`svc/erebus/migrations/000_create_user_references_table.sql`** - Main Erebus table
2. **`svc/nullblock-agents/migrations/000_create_user_references_sync_cache.sql`** - Agents sync cache
3. **`test_user_references_migration.sql`** - Test script with examples

## Testing

The migration includes comprehensive test cases for all source types:
- Web3 wallet users (MetaMask, Phantom, etc.)
- API key users (trading bots, automation scripts)
- Email authenticated users (Gmail, Outlook, etc.)
- System agent users (task runners, monitors)
- OAuth users (Google, GitHub, Discord, etc.)

## Next Steps

1. **Apply Migrations**: Run the new migration files to create the tables
2. **Test Integration**: Verify the Rust code works with the new schema
3. **Update API Endpoints**: Ensure all user management endpoints use the new schema
4. **Sync Setup**: Configure Kafka events to sync between Erebus and Agents databases

## Benefits

- **Unified User Management**: Single table for all user types
- **Type Safety**: Database-level validation ensures data integrity
- **Performance**: Optimized indexes for common query patterns
- **Extensibility**: Easy to add new authentication methods
- **Backward Compatibility**: Existing Web3 wallet flows continue to work
- **Future-Proof**: Supports any authentication method without schema changes















