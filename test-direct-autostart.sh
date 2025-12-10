#!/bin/bash
echo "=== Testing auto_start DIRECTLY to Agents service (bypass Erebus) ==="

TASK_ID=$(curl -s -X POST http://localhost:9003/tasks \
  -H "Content-Type: application/json" \
  -H "x-wallet-address: direct-test-$(date +%s)" \
  -H "x-wallet-chain: solana" \
  -d '{
    "name": "Direct Auto Start Test",
    "description": "Tell me what 2+2 equals",
    "task_type": "user_assigned",
    "priority": "high",
    "auto_start": true
  }' | jq -r '.data.id')

echo "✅ Created task: $TASK_ID"
echo "⏳ Waiting 2 seconds for background processing to start..."
sleep 2

echo "📊 Checking if actioned_at is set (means background task started)..."
curl -s http://localhost:9003/tasks/$TASK_ID | jq '{id, status: .data.status.state, actioned_at: .data.actioned_at}'

echo ""
echo "⏳ Waiting 15 more seconds for completion..."
sleep 15

echo "📊 Final status:"
curl -s http://localhost:9003/tasks/$TASK_ID | jq '{id, status: .data.status.state, action_duration: .data.action_duration, action_result: .data.action_result}'
