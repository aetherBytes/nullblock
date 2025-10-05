#!/bin/bash

BASE_URL="${PROTOCOLS_SERVICE_URL:-http://localhost:8001}"
MCP_ENDPOINT="$BASE_URL/mcp/jsonrpc"

echo "🧪 Testing MCP Protocol Implementation"
echo "📍 Endpoint: $MCP_ENDPOINT"
echo ""

echo "1️⃣  Testing initialize..."
curl -X POST "$MCP_ENDPOINT" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
      "protocolVersion": "2025-06-18",
      "capabilities": {},
      "clientInfo": {
        "name": "test-client",
        "version": "1.0.0"
      }
    }
  }' | jq '.'

echo ""
echo "2️⃣  Testing tools/list..."
curl -X POST "$MCP_ENDPOINT" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/list"
  }' | jq '.'

echo ""
echo "3️⃣  Testing resources/list..."
curl -X POST "$MCP_ENDPOINT" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 3,
    "method": "resources/list"
  }' | jq '.'

echo ""
echo "4️⃣  Testing prompts/list..."
curl -X POST "$MCP_ENDPOINT" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 4,
    "method": "prompts/list"
  }' | jq '.'

echo ""
echo "5️⃣  Testing ping..."
curl -X POST "$MCP_ENDPOINT" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 5,
    "method": "ping"
  }' | jq '.'

echo ""
echo "✅ MCP tests complete!"

