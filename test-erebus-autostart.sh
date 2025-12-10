#!/bin/bash
echo "=== Testing auto_start VIA EREBUS (simulating frontend) ==="

TASK_ID=$(curl -s -X POST http://localhost:3000/api/agents/tasks \
  -H "Content-Type: application/json" \
  -H "x-wallet-address: erebus-test-$(date +%s)" \
  -H "x-wallet-chain: solana" \
  -d '{
    "name": "Erebus Auto Start Test",
    "description": "What is 5+5?",
    "task_type": "user_assigned",
    "priority": "high",
    "auto_start": true
  }' | jq -r '.data.id')

echo "✅ Created task via Erebus: $TASK_ID"
echo "⏳ Waiting 2 seconds..."
sleep 2

echo "📊 Checking if actioned_at is set..."
curl -s http://localhost:3000/api/agents/tasks/$TASK_ID | jq '{id: .data.id, status: .data.status.state, actioned_at: .data.actioned_at}'

echo ""
echo "⏳ Waiting 15 more seconds for completion..."
sleep 15

echo "📊 Final status:"
curl -s http://localhost:3000/api/agents/tasks/$TASK_ID | jq '{id: .data.id, status: .data.status.state, action_duration: .data.action_duration}'
