#!/bin/bash
echo "Testing auto_start mechanism..."

TASK_ID=$(curl -s -X POST http://localhost:9003/tasks \
  -H "Content-Type: application/json" \
  -H "x-wallet-address: debug-wallet-$(date +%s)" \
  -H "x-wallet-chain: solana" \
  -d '{"name":"Auto Start Test","description":"Testing auto_start","task_type":"user_assigned","priority":"high","auto_start":true}' \
  | jq -r '.data.id')

echo "Created task: $TASK_ID"
echo "Waiting 5 seconds for auto-processing..."
sleep 5

echo "Checking task status..."
curl -s http://localhost:9003/tasks/$TASK_ID | jq '{id: .data.id, status: .data.status.state, actioned_at: .data.actioned_at, action_result: .data.action_result}'
