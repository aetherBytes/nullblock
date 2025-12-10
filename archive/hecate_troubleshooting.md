# Hecate Agent Troubleshooting Guide

## Issue: "No LLM models available"

The Hecate agent is running but can't find any working LLM models. Here's how to fix this:

## 🔍 Diagnosis

From your health check, we can see:
- All API providers (OpenAI, Anthropic, Groq, HuggingFace) are failing
- LM Studio is detected but no models are available
- Ollama is not accessible

## 🛠️ Solutions

### Option 1: Set up API Keys (Recommended)

Set environment variables for at least one API provider:

```bash
# OpenAI (most common)
export OPENAI_API_KEY="your-openai-api-key-here"

# Or Anthropic
export ANTHROPIC_API_KEY="your-anthropic-api-key-here"

# Or Groq
export GROQ_API_KEY="your-groq-api-key-here"

# Or HuggingFace
export HUGGINGFACE_API_KEY="your-huggingface-api-key-here"
```

### Option 2: Start LM Studio (Local)

1. **Install LM Studio** from https://lmstudio.ai/
2. **Start LM Studio** and load a model
3. **Enable local server** in LM Studio settings:
   - Go to Settings → Local Server
   - Enable "Start local server"
   - Set port to 1234 (default)
   - Load a model (like Llama 2, Mistral, etc.)

### Option 3: Start Ollama (Alternative Local)

```bash
# Install Ollama
curl -fsSL https://ollama.ai/install.sh | sh

# Start Ollama service
ollama serve

# In another terminal, pull a model
ollama pull llama2
```

### Option 4: Quick Test with Environment Variables

Create a test script to verify your setup:

```bash
#!/bin/bash
# test_hecate_setup.sh

echo "🔍 Checking Hecate Agent Setup..."

# Check environment variables
echo "📋 Environment Variables:"
echo "OPENAI_API_KEY: ${OPENAI_API_KEY:+SET}"
echo "ANTHROPIC_API_KEY: ${ANTHROPIC_API_KEY:+SET}"
echo "GROQ_API_KEY: ${GROQ_API_KEY:+SET}"
echo "HUGGINGFACE_API_KEY: ${HUGGINGFACE_API_KEY:+SET}"

# Check local services
echo -e "\n🏠 Local Services:"
echo "LM Studio (port 1234):"
curl -s http://localhost:1234/v1/models > /dev/null && echo "✅ Running" || echo "❌ Not running"

echo "Ollama (port 11434):"
curl -s http://localhost:11434/api/tags > /dev/null && echo "✅ Running" || echo "❌ Not running"

# Test Hecate health
echo -e "\n🤖 Hecate Agent Health:"
curl -s "${HECATE_AGENT_BASE_URL}/health" | jq '.agent.model_status.health'
```

## 🚀 Quick Fix Commands

### For OpenAI (Most Common)
```bash
export OPENAI_API_KEY="sk-your-key-here"
# Restart Hecate agent
```

### For LM Studio
```bash
# Start LM Studio and load a model, then restart Hecate
```

### Test the Fix
```bash
# Test health
curl -X GET "${HECATE_AGENT_BASE_URL}/health" | jq '.agent.model_status.health'

# Test chat
curl -X POST "${HECATE_AGENT_BASE_URL}/chat" \
  -H "Content-Type: application/json" \
  -d '{"message": "Hello!", "user_context": {"wallet_address": "0x1234567890abcdef"}}'
```

## 📊 Expected Health Response (Working)

```json
{
  "status": "healthy",
  "agent": {
    "model_status": {
      "health": {
        "overall_status": "healthy",
        "api_providers": {
          "openai": true,
          "anthropic": false,
          "groq": false,
          "huggingface": false
        },
        "local_providers": {
          "lm_studio": true,
          "ollama": false
        },
        "models_available": 2,
        "default_model": "gpt-3.5-turbo"
      }
    }
  }
}
```

## 🔧 Advanced Configuration

### Custom Model Endpoints
If you're using custom endpoints, you can modify the configuration:

```bash
# For custom OpenAI endpoint
export OPENAI_API_BASE="https://your-custom-endpoint.com/v1"

# For custom LM Studio endpoint
export LM_STUDIO_URL="http://localhost:1234"
```

### Debug Mode
Enable debug logging to see detailed connection attempts:

```bash
export LOG_LEVEL="DEBUG"
# Restart Hecate agent
```

## 🆘 Still Having Issues?

1. **Check logs**: Look at the Hecate agent logs for detailed error messages
2. **Network connectivity**: Ensure the agent can reach the API endpoints
3. **API key validity**: Verify your API keys are valid and have sufficient credits
4. **Firewall/proxy**: Check if corporate firewalls are blocking connections

## 📝 Common Issues

- **"Connection refused"**: Local services not running
- **"401 Unauthorized"**: Invalid API key
- **"429 Too Many Requests"**: Rate limit exceeded
- **"503 Service Unavailable"**: API service down
