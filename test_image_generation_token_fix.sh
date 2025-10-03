#!/bin/bash
# Test script for image generation token optimization
# This tests that images can be generated multiple times without token exhaustion

set -e

echo "🎨 Testing Image Generation Token Fix"
echo "======================================"
echo ""

API_URL="http://localhost:9001/hecate"
LOG_FILE="logs/image-generation-test.log"

# Create logs directory if it doesn't exist
mkdir -p logs

# Test 1: Send first image generation request
echo "Test 1: First image generation request"
echo "Request: 'Create a simple cyberpunk logo for NullBlock'"
echo ""

RESPONSE1=$(curl -s -X POST "$API_URL/chat" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "Create a simple cyberpunk logo for NullBlock",
    "user_context": {
      "wallet_address": "0x1234567890123456789012345678901234567890"
    }
  }')

# Check if response contains image data
if echo "$RESPONSE1" | grep -q "data:image"; then
  echo "✅ First request successful - image generated"
  MODEL=$(echo "$RESPONSE1" | jq -r '.model_used // "unknown"')
  TOKENS=$(echo "$RESPONSE1" | jq -r '.metadata.token_usage.total_tokens // 0')
  echo "   Model: $MODEL"
  echo "   Tokens used: $TOKENS"
else
  echo "❌ First request failed - no image in response"
  echo "$RESPONSE1" | jq '.'
  exit 1
fi

echo ""
sleep 2

# Test 2: Send second image generation request (should not be exhausted)
echo "Test 2: Second image generation request"
echo "Request: 'Now make it more futuristic with neon effects'"
echo ""

RESPONSE2=$(curl -s -X POST "$API_URL/chat" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "Now make it more futuristic with neon effects",
    "user_context": {
      "wallet_address": "0x1234567890123456789012345678901234567890"
    }
  }')

# Check if response contains image data
if echo "$RESPONSE2" | grep -q "data:image"; then
  echo "✅ Second request successful - image generated"
  MODEL=$(echo "$RESPONSE2" | jq -r '.model_used // "unknown"')
  TOKENS=$(echo "$RESPONSE2" | jq -r '.metadata.token_usage.total_tokens // 0')
  echo "   Model: $MODEL"
  echo "   Tokens used: $TOKENS"
else
  echo "❌ Second request failed - no image in response"
  echo "$RESPONSE2" | jq '.'
  exit 1
fi

echo ""
sleep 2

# Test 3: Third request to confirm sustained functionality
echo "Test 3: Third image generation request"
echo "Request: 'Create another version with gold and silver colors'"
echo ""

RESPONSE3=$(curl -s -X POST "$API_URL/chat" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "Create another version with gold and silver colors",
    "user_context": {
      "wallet_address": "0x1234567890123456789012345678901234567890"
    }
  }')

# Check if response contains image data
if echo "$RESPONSE3" | grep -q "data:image"; then
  echo "✅ Third request successful - image generated"
  MODEL=$(echo "$RESPONSE3" | jq -r '.model_used // "unknown"')
  TOKENS=$(echo "$RESPONSE3" | jq -r '.metadata.token_usage.total_tokens // 0')
  echo "   Model: $MODEL"
  echo "   Tokens used: $TOKENS"
else
  echo "❌ Third request failed - no image in response"
  echo "$RESPONSE3" | jq '.'
  exit 1
fi

echo ""
echo "======================================"
echo "✅ All tests passed!"
echo ""
echo "Summary:"
echo "- Multiple image generation requests completed successfully"
echo "- No token exhaustion after 3 requests"
echo "- Context is being properly managed"
echo ""
echo "Check logs/agents-db.log for detailed token usage information"
echo "Look for: 🖼️ Stripped base64 image data messages"

# Save full responses to log file
{
  echo "Test run: $(date)"
  echo ""
  echo "Response 1:"
  echo "$RESPONSE1" | jq '.'
  echo ""
  echo "Response 2:"
  echo "$RESPONSE2" | jq '.'
  echo ""
  echo "Response 3:"
  echo "$RESPONSE3" | jq '.'
} > "$LOG_FILE"

echo ""
echo "Full responses saved to: $LOG_FILE"

