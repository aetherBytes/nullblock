#!/bin/bash

echo "=== Testing Task Creation Flow ==="
echo

# Test 1: Direct to Agents service
echo "1️⃣ Testing direct to Agents service (port 9003):"
RESPONSE=$(curl -s -X POST http://localhost:9003/tasks \
  -H "Content-Type: application/json" \
  -H "x-wallet-address: test-wallet-123" \
  -H "x-wallet-chain: solana" \
  -d '{
    "name": "Debug Test 1",
    "description": "Testing direct to Agents",
    "task_type": "user_assigned",
    "priority": "medium",
    "auto_start": false
  }')
echo "$RESPONSE" | jq '.'
SUCCESS=$(echo "$RESPONSE" | jq -r '.success')
echo "✅ Success field: $SUCCESS"
echo

# Test 2: Via Erebus proxy
echo "2️⃣ Testing via Erebus proxy (port 3000):"
RESPONSE=$(curl -s -X POST http://localhost:3000/api/agents/tasks \
  -H "Content-Type: application/json" \
  -H "x-wallet-address: test-wallet-123" \
  -H "x-wallet-chain: solana" \
  -d '{
    "name": "Debug Test 2",
    "description": "Testing via Erebus",
    "task_type": "user_assigned",
    "priority": "medium",
    "auto_start": false
  }')
echo "$RESPONSE" | jq '.'
SUCCESS=$(echo "$RESPONSE" | jq -r '.success')
DATA=$(echo "$RESPONSE" | jq '.data')
echo "✅ Success field: $SUCCESS"
echo "📦 Data field exists: $(if [ "$DATA" != "null" ]; then echo "YES"; else echo "NO"; fi)"
echo

# Test 3: Check response structure
echo "3️⃣ Response structure check:"
echo "$RESPONSE" | jq 'keys'
