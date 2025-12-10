# Hecate Agent API - Curl Test Commands

## Base Configuration
```bash
# Set the base URL (adjust port if needed)
BASE_URL="http://localhost:9002"
```

## 1. Health Check
```bash
curl -X GET "${BASE_URL}/health" \
  -H "Content-Type: application/json"
```

## 2. Model Status
```bash
curl -X GET "${BASE_URL}/model-status" \
  -H "Content-Type: application/json"
```

## 3. Simple Chat
```bash
curl -X POST "${BASE_URL}/chat" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "Hello Hecate! What can you do?",
    "user_context": {
      "wallet_address": "0x1234567890abcdef",
      "wallet_type": "metamask"
    }
  }'
```

### Fixed Version (Single Line)
```bash
curl -X POST "${BASE_URL}/chat" -H "Content-Type: application/json" -d '{"message": "Hello Hecate!", "user_context": {"wallet_address": "0x1234567890abcdef"}}'
```

## 4. Trading Query
```bash
curl -X POST "${BASE_URL}/chat" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "I want to analyze trading opportunities",
    "user_context": {
      "wallet_address": "0xabcdef1234567890",
      "wallet_type": "phantom"
    }
  }'
```

## 5. Analysis Query
```bash
curl -X POST "${BASE_URL}/chat" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "Can you analyze market trends?",
    "user_context": {
      "wallet_address": "0x9876543210fedcba",
      "wallet_type": "metamask"
    }
  }'
```

## 6. Social Sentiment Query
```bash
curl -X POST "${BASE_URL}/chat" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "What is the social sentiment around crypto?",
    "user_context": {
      "wallet_address": "0xfedcba0987654321",
      "wallet_type": "phantom"
    }
  }'
```

## 7. Get Conversation History
```bash
curl -X GET "${BASE_URL}/history" \
  -H "Content-Type: application/json"
```

## 8. Change Personality to Technical Expert
```bash
curl -X POST "${BASE_URL}/personality" \
  -H "Content-Type: application/json" \
  -d '{
    "personality": "technical_expert"
  }'
```

## 9. Test Technical Query
```bash
curl -X POST "${BASE_URL}/chat" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "Explain blockchain consensus mechanisms",
    "user_context": {
      "wallet_address": "0x1234567890abcdef",
      "wallet_type": "metamask"
    }
  }'
```

## 10. Change to Concise Personality
```bash
curl -X POST "${BASE_URL}/personality" \
  -H "Content-Type: application/json" \
  -d '{
    "personality": "concise_assistant"
  }'
```

## 11. Test Concise Response
```bash
curl -X POST "${BASE_URL}/chat" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "Give me a quick summary of DeFi",
    "user_context": {
      "wallet_address": "0xabcdef1234567890",
      "wallet_type": "phantom"
    }
  }'
```

## 12. Clear Conversation
```bash
curl -X POST "${BASE_URL}/clear" \
  -H "Content-Type: application/json"
```

## 13. Test Error Handling (Invalid JSON)
```bash
curl -X POST "${BASE_URL}/chat" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "Test message",
    "user_context": {
      "wallet_address": "0x1234567890abcdef"
    }'
```

## 14. Test Long Message
```bash
curl -X POST "${BASE_URL}/chat" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "This is a very long message to test how Hecate handles extended inputs. I want to see if the system can process longer conversations and maintain context properly. This should help us understand the capabilities of the conversation management system and how it handles various input lengths.",
    "user_context": {
      "wallet_address": "0x1234567890abcdef",
      "wallet_type": "metamask"
    }
  }'
```

## 15. Test with Detailed User Context
```bash
curl -X POST "${BASE_URL}/chat" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "What trading strategies would you recommend?",
    "user_context": {
      "wallet_address": "0x1234567890abcdef",
      "wallet_type": "metamask",
      "session_time": "15 minutes",
      "preferences": {
        "risk_tolerance": "moderate",
        "investment_horizon": "short_term",
        "preferred_chains": ["ethereum", "polygon"]
      }
    }
  }'
```

## Quick Test Script
You can also run the complete test script:
```bash
chmod +x test_hecate_curls.sh
./test_hecate_curls.sh
```

## Expected Responses

### Health Check Response
```json
{
  "status": "healthy",
  "timestamp": "2024-01-01T12:00:00",
  "agent": {
    "agent_running": true,
    "personality": "helpful_cyberpunk",
    "conversation_length": 5,
    "model_status": {
      "status": "running",
      "current_model": "gpt-3.5-turbo",
      "health": {...},
      "stats": {...}
    }
  }
}
```

### Chat Response
```json
{
  "content": "Hello! I am Hecate, your advanced AI assistant...",
  "model_used": "gpt-3.5-turbo",
  "latency_ms": 1250.5,
  "confidence_score": 0.85,
  "metadata": {
    "personality": "helpful_cyberpunk",
    "cost_estimate": 0.002,
    "token_usage": {...},
    "finish_reason": "stop",
    "conversation_length": 6
  },
  "processing_time_s": 1.25
}
```

## Notes
- The server runs on port 9002 by default (configurable via HECATE_PORT env var)
- All endpoints return JSON responses
- The agent maintains conversation context across requests
- Personality changes affect response style and system prompts
- Error responses include detailed error information
