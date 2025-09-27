#!/usr/bin/env python3
"""
Simple sync service for Erebus to Agents database
This service monitors the user_sync_queue and processes sync operations
"""

import time
import subprocess
import sys
from datetime import datetime

def run_sql_command(container, database, sql):
    """Run SQL command in a Docker container"""
    cmd = f"docker exec {container} psql -U postgres -d {database} -c \"{sql}\""
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
    return result.returncode == 0, result.stdout, result.stderr

def check_sync_queue():
    """Check for unprocessed sync queue entries"""
    sql = "SELECT COUNT(*) FROM user_sync_queue WHERE processed = false"
    success, stdout, stderr = run_sql_command("nullblock-postgres-erebus", "erebus", sql)
    
    if success:
        count = int(stdout.strip().split('\n')[2].strip())  # Parse the count
        return count
    return 0

def process_sync_queue():
    """Process unprocessed sync queue entries"""
    print(f"🔄 [{datetime.now()}] Checking sync queue...")
    
    # Get unprocessed entries
    sql = """
    SELECT id, user_id, operation, user_data 
    FROM user_sync_queue 
    WHERE processed = false 
    ORDER BY created_at
    """
    
    success, stdout, stderr = run_sql_command("nullblock-postgres-erebus", "erebus", sql)
    
    if not success:
        print(f"❌ Failed to query sync queue: {stderr}")
        return
    
    lines = stdout.strip().split('\n')
    if len(lines) < 4:  # No data rows
        return
    
    # Process each entry
    for i in range(3, len(lines) - 1):  # Skip header lines
        line = lines[i].strip()
        if not line or '|' not in line:
            continue
            
        parts = [p.strip() for p in line.split('|')]
        if len(parts) < 4:
            continue
            
        queue_id = parts[0]
        user_id = parts[1]
        operation = parts[2]
        
        print(f"🔄 Processing {operation} for user {user_id}")
        
        # For now, we'll do a simple sync by copying all active users
        sync_all_users()
        
        # Mark as processed
        mark_processed_sql = f"UPDATE user_sync_queue SET processed = true, processed_at = NOW() WHERE id = '{queue_id}'"
        run_sql_command("nullblock-postgres-erebus", "erebus", mark_processed_sql)
        
        print(f"✅ Processed {operation} for user {user_id}")

def sync_all_users():
    """Sync all active users from Erebus to Agents"""
    print("🔄 Syncing all users from Erebus to Agents...")
    
    # Clear agents database
    run_sql_command("nullblock-postgres-agents", "agents", "DELETE FROM user_references")
    
    # Get users from Erebus and insert into Agents
    # This is a simplified approach - in production you'd want more sophisticated sync
    sync_sql = """
    INSERT INTO user_references (id, source_identifier, chain, user_type, source_type, wallet_type, email, metadata, preferences, additional_metadata, is_active)
    SELECT id, source_identifier, chain, user_type, source_type, wallet_type, email, metadata, preferences, additional_metadata, is_active
    FROM dblink('host=localhost port=5440 dbname=erebus user=postgres password=REDACTED_DB_PASS',
        'SELECT id, source_identifier, chain, user_type, source_type, wallet_type, email, metadata, preferences, additional_metadata, is_active FROM user_references WHERE is_active = true'
    ) AS erebus_user (
        id uuid, source_identifier text, chain text, user_type text, source_type jsonb, 
        wallet_type text, email text, metadata jsonb, preferences jsonb, additional_metadata jsonb, is_active boolean
    )
    """
    
    success, stdout, stderr = run_sql_command("nullblock-postgres-agents", "agents", sync_sql)
    
    if success:
        print("✅ Users synced successfully")
    else:
        print(f"❌ Failed to sync users: {stderr}")

def main():
    """Main sync service loop"""
    print("🚀 Starting Erebus to Agents sync service...")
    print("📋 Monitoring user_sync_queue for changes...")
    
    while True:
        try:
            unprocessed_count = check_sync_queue()
            
            if unprocessed_count > 0:
                print(f"📋 Found {unprocessed_count} unprocessed sync entries")
                process_sync_queue()
            else:
                print(f"✅ [{datetime.now()}] No pending sync operations")
            
            # Wait 30 seconds before checking again
            time.sleep(30)
            
        except KeyboardInterrupt:
            print("\n🛑 Sync service stopped by user")
            break
        except Exception as e:
            print(f"❌ Error in sync service: {e}")
            time.sleep(10)  # Wait before retrying

if __name__ == "__main__":
    main()

