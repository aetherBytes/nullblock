#!/bin/bash

echo "🧪 Testing Erebus User Registration Endpoint"
echo "=============================================="

# Test user registration endpoint
echo "📤 Testing user registration with curl..."
curl -X POST http://localhost:3000/api/agents/users/register \
  -H "Content-Type: application/json" \
  -H "x-wallet-address: test-wallet-123" \
  -H "x-wallet-chain: solana" \
  -d '{
    "wallet_address": "test-wallet-123",
    "chain": "solana"
  }' \
  -v

echo ""
echo "🔍 Checking if Erebus is running..."
curl -s http://localhost:3000/health | jq . || echo "❌ Erebus not responding"

echo ""
echo "🔍 Checking if Agents service is running..."
curl -s http://localhost:9003/health | jq . || echo "❌ Agents service not responding"
