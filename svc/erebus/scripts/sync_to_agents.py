#!/usr/bin/env python3
"""
Sync processor for Erebus to Agents database
This script processes the user_sync_queue and syncs data to the agents database
"""

import asyncio
import asyncpg
import json
import os
from datetime import datetime

# Database URLs
EREBUS_URL = os.getenv('EREBUS_DATABASE_URL', 'postgresql://postgres:REDACTED_DB_PASS@localhost:5440/erebus')
AGENTS_URL = os.getenv('AGENTS_DATABASE_URL', 'postgresql://postgres:REDACTED_DB_PASS@localhost:5441/agents')

async def process_sync_queue():
    """Process pending sync queue entries"""
    print("🔄 Starting sync processor...")
    
    # Connect to Erebus database
    erebus_conn = await asyncpg.connect(EREBUS_URL)
    print("✅ Connected to Erebus database")
    
    # Connect to Agents database
    agents_conn = await asyncpg.connect(AGENTS_URL)
    print("✅ Connected to Agents database")
    
    try:
        # Get unprocessed sync queue entries
        queue_entries = await erebus_conn.fetch("""
            SELECT id, user_id, operation, user_data, created_at
            FROM user_sync_queue 
            WHERE processed = false 
            ORDER BY created_at
        """)
        
        print(f"📋 Found {len(queue_entries)} unprocessed sync entries")
        
        for entry in queue_entries:
            queue_id = entry['id']
            user_id = entry['user_id']
            operation = entry['operation']
            user_data = entry['user_data']
            
            print(f"🔄 Processing {operation} for user {user_id}")
            
            try:
                if operation == 'INSERT':
                    await sync_user_insert(agents_conn, user_data)
                elif operation == 'UPDATE':
                    await sync_user_update(agents_conn, user_data)
                elif operation == 'DELETE':
                    await sync_user_delete(agents_conn, user_id)
                
                # Mark as processed
                await erebus_conn.execute("""
                    UPDATE user_sync_queue 
                    SET processed = true, processed_at = NOW() 
                    WHERE id = $1
                """, queue_id)
                
                print(f"✅ Processed {operation} for user {user_id}")
                
            except Exception as e:
                print(f"❌ Failed to process {operation} for user {user_id}: {e}")
                continue
        
        print("🎉 Sync processing completed!")
        
    finally:
        await erebus_conn.close()
        await agents_conn.close()

async def sync_user_insert(agents_conn, user_data):
    """Sync user insert to agents database"""
    await agents_conn.execute("""
        INSERT INTO user_references (
            id, source_identifier, chain, user_type, source_type, wallet_type, 
            email, metadata, preferences, additional_metadata, is_active
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        ON CONFLICT (source_identifier, chain) DO UPDATE SET
            user_type = EXCLUDED.user_type,
            source_type = EXCLUDED.source_type,
            wallet_type = EXCLUDED.wallet_type,
            email = EXCLUDED.email,
            metadata = EXCLUDED.metadata,
            preferences = EXCLUDED.preferences,
            additional_metadata = EXCLUDED.additional_metadata,
            is_active = EXCLUDED.is_active
    """, 
        user_data['id'],
        user_data['source_identifier'],
        user_data['chain'],
        user_data.get('user_type', 'external'),
        user_data['source_type'],
        user_data.get('wallet_type'),
        user_data.get('email'),
        user_data.get('metadata', {}),
        user_data.get('preferences', {}),
        user_data.get('additional_metadata', {}),
        user_data.get('is_active', True)
    )

async def sync_user_update(agents_conn, user_data):
    """Sync user update to agents database"""
    await agents_conn.execute("""
        UPDATE user_references SET
            source_identifier = $2,
            chain = $3,
            user_type = $4,
            source_type = $5,
            wallet_type = $6,
            email = $7,
            metadata = $8,
            preferences = $9,
            additional_metadata = $10,
            is_active = $11
        WHERE id = $1
    """,
        user_data['id'],
        user_data['source_identifier'],
        user_data['chain'],
        user_data.get('user_type', 'external'),
        user_data['source_type'],
        user_data.get('wallet_type'),
        user_data.get('email'),
        user_data.get('metadata', {}),
        user_data.get('preferences', {}),
        user_data.get('additional_metadata', {}),
        user_data.get('is_active', True)
    )

async def sync_user_delete(agents_conn, user_id):
    """Sync user delete to agents database"""
    await agents_conn.execute("""
        DELETE FROM user_references WHERE id = $1
    """, user_id)

if __name__ == "__main__":
    asyncio.run(process_sync_queue())
