#!/bin/bash

# Hecate Agent API Test Script
# Base URL - adjust port if needed
BASE_URL="http://localhost:9002"

echo "🧪 Testing Hecate Agent API endpoints..."
echo "=========================================="

# 1. Health Check
echo -e "\n1️⃣ Testing Health Check:"
curl -X GET "${BASE_URL}/health" \
  -H "Content-Type: application/json" \
  -w "\nHTTP Status: %{http_code}\nResponse Time: %{time_total}s\n"

# 2. Model Status
echo -e "\n2️⃣ Testing Model Status:"
curl -X GET "${BASE_URL}/model-status" \
  -H "Content-Type: application/json" \
  -w "\nHTTP Status: %{http_code}\nResponse Time: %{time_total}s\n"

# 3. Simple Chat Message
echo -e "\n3️⃣ Testing Simple Chat:"
curl -X POST "${BASE_URL}/chat" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "Hello Hecate! Can you tell me about yourself?",
    "user_context": {
      "wallet_address": "0x1234567890abcdef",
      "wallet_type": "metamask",
      "session_time": "5 minutes"
    }
  }' \
  -w "\nHTTP Status: %{http_code}\nResponse Time: %{time_total}s\n"

# 4. Trading Related Query
echo -e "\n4️⃣ Testing Trading Query:"
curl -X POST "${BASE_URL}/chat" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "I want to analyze some trading opportunities. What can you help me with?",
    "user_context": {
      "wallet_address": "0xabcdef1234567890",
      "wallet_type": "phantom",
      "session_time": "10 minutes"
    }
  }' \
  -w "\nHTTP Status: %{http_code}\nResponse Time: %{time_total}s\n"

# 5. Analysis Query
echo -e "\n5️⃣ Testing Analysis Query:"
curl -X POST "${BASE_URL}/chat" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "Can you analyze market trends and patterns for me?",
    "user_context": {
      "wallet_address": "0x9876543210fedcba",
      "wallet_type": "metamask",
      "session_time": "15 minutes"
    }
  }' \
  -w "\nHTTP Status: %{http_code}\nResponse Time: %{time_total}s\n"

# 6. Social Sentiment Query
echo -e "\n6️⃣ Testing Social Sentiment Query:"
curl -X POST "${BASE_URL}/chat" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "What is the current social sentiment around crypto markets?",
    "user_context": {
      "wallet_address": "0xfedcba0987654321",
      "wallet_type": "phantom",
      "session_time": "20 minutes"
    }
  }' \
  -w "\nHTTP Status: %{http_code}\nResponse Time: %{time_total}s\n"

# 7. Get Conversation History
echo -e "\n7️⃣ Testing Get History:"
curl -X GET "${BASE_URL}/history" \
  -H "Content-Type: application/json" \
  -w "\nHTTP Status: %{http_code}\nResponse Time: %{time_total}s\n"

# 8. Change Personality to Technical Expert
echo -e "\n8️⃣ Testing Personality Change:"
curl -X POST "${BASE_URL}/personality" \
  -H "Content-Type: application/json" \
  -d '{
    "personality": "technical_expert"
  }' \
  -w "\nHTTP Status: %{http_code}\nResponse Time: %{time_total}s\n"

# 9. Test Technical Query with New Personality
echo -e "\n9️⃣ Testing Technical Query with New Personality:"
curl -X POST "${BASE_URL}/chat" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "Explain the technical aspects of blockchain consensus mechanisms",
    "user_context": {
      "wallet_address": "0x1234567890abcdef",
      "wallet_type": "metamask",
      "session_time": "25 minutes"
    }
  }' \
  -w "\nHTTP Status: %{http_code}\nResponse Time: %{time_total}s\n"

# 10. Change to Concise Personality
echo -e "\n🔟 Testing Concise Personality:"
curl -X POST "${BASE_URL}/personality" \
  -H "Content-Type: application/json" \
  -d '{
    "personality": "concise_assistant"
  }' \
  -w "\nHTTP Status: %{http_code}\nResponse Time: %{time_total}s\n"

# 11. Test Concise Response
echo -e "\n1️⃣1️⃣ Testing Concise Response:"
curl -X POST "${BASE_URL}/chat" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "Give me a quick summary of DeFi",
    "user_context": {
      "wallet_address": "0xabcdef1234567890",
      "wallet_type": "phantom",
      "session_time": "30 minutes"
    }
  }' \
  -w "\nHTTP Status: %{http_code}\nResponse Time: %{time_total}s\n"

# 12. Clear Conversation
echo -e "\n1️⃣2️⃣ Testing Clear Conversation:"
curl -X POST "${BASE_URL}/clear" \
  -H "Content-Type: application/json" \
  -w "\nHTTP Status: %{http_code}\nResponse Time: %{time_total}s\n"

# 13. Verify History is Cleared
echo -e "\n1️⃣3️⃣ Verifying History is Cleared:"
curl -X GET "${BASE_URL}/history" \
  -H "Content-Type: application/json" \
  -w "\nHTTP Status: %{http_code}\nResponse Time: %{time_total}s\n"

# 14. Test Error Handling (Invalid JSON)
echo -e "\n1️⃣4️⃣ Testing Error Handling (Invalid JSON):"
curl -X POST "${BASE_URL}/chat" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "This is a test message",
    "user_context": {
      "wallet_address": "0x1234567890abcdef"
    ' \
  -w "\nHTTP Status: %{http_code}\nResponse Time: %{time_total}s\n"

# 15. Test Long Message
echo -e "\n1️⃣5️⃣ Testing Long Message:"
curl -X POST "${BASE_URL}/chat" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "This is a very long message to test how Hecate handles longer inputs. I want to see if the system can process extended conversations and maintain context properly. This should help us understand the capabilities of the conversation management system and how it handles various input lengths. I am particularly interested in seeing how the token management and context window handling works in practice.",
    "user_context": {
      "wallet_address": "0x1234567890abcdef",
      "wallet_type": "metamask",
      "session_time": "35 minutes"
    }
  }' \
  -w "\nHTTP Status: %{http_code}\nResponse Time: %{time_total}s\n"

echo -e "\n✅ All tests completed!"
echo "=========================================="
