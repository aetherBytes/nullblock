#!/bin/bash

# Hecate Agent Setup Diagnostic Script
# This script helps diagnose LLM connectivity issues

echo "🔍 Hecate Agent Setup Diagnostic"
echo "================================="

# Set base URL if not already set
HECATE_AGENT_BASE_URL="${HECATE_AGENT_BASE_URL:-http://localhost:9002}"

echo -e "\n📋 Environment Variables Check:"
echo "OPENAI_API_KEY: ${OPENAI_API_KEY:+✅ SET}${OPENAI_API_KEY:-❌ NOT SET}"
echo "ANTHROPIC_API_KEY: ${ANTHROPIC_API_KEY:+✅ SET}${ANTHROPIC_API_KEY:-❌ NOT SET}"
echo "GROQ_API_KEY: ${GROQ_API_KEY:+✅ SET}${GROQ_API_KEY:-❌ NOT SET}"
echo "HUGGINGFACE_API_KEY: ${HUGGINGFACE_API_KEY:+✅ SET}${HUGGINGFACE_API_KEY:-❌ NOT SET}"

echo -e "\n🏠 Local Services Check:"
echo "LM Studio (port 1234):"
if curl -s http://localhost:1234/v1/models > /dev/null 2>&1; then
    echo "✅ Running"
    # Try to get model list
    models=$(curl -s http://localhost:1234/v1/models 2>/dev/null | jq -r '.data[].id' 2>/dev/null || echo "No models found")
    echo "   Models: $models"
else
    echo "❌ Not running"
fi

echo "Ollama (port 11434):"
if curl -s http://localhost:11434/api/tags > /dev/null 2>&1; then
    echo "✅ Running"
    # Try to get model list
    models=$(curl -s http://localhost:11434/api/tags 2>/dev/null | jq -r '.models[].name' 2>/dev/null || echo "No models found")
    echo "   Models: $models"
else
    echo "❌ Not running"
fi

echo -e "\n🤖 Hecate Agent Health Check:"
if curl -s "${HECATE_AGENT_BASE_URL}/health" > /dev/null 2>&1; then
    echo "✅ Hecate agent is running"
    
    # Get detailed health info
    health_response=$(curl -s "${HECATE_AGENT_BASE_URL}/health")
    
    # Extract key information
    overall_status=$(echo "$health_response" | jq -r '.agent.model_status.health.overall_status' 2>/dev/null || echo "unknown")
    models_available=$(echo "$health_response" | jq -r '.agent.model_status.health.models_available' 2>/dev/null || echo "unknown")
    
    echo "   Overall Status: $overall_status"
    echo "   Models Available: $models_available"
    
    # Show provider status
    echo "   API Providers:"
    echo "$health_response" | jq -r '.agent.model_status.health.api_providers | to_entries[] | "     \(.key): \(if .value then "✅" else "❌" end)"' 2>/dev/null || echo "     Unable to parse provider status"
    
    echo "   Local Providers:"
    echo "$health_response" | jq -r '.agent.model_status.health.local_providers | to_entries[] | "     \(.key): \(if .value then "✅" else "❌" end)"' 2>/dev/null || echo "     Unable to parse local provider status"
    
else
    echo "❌ Hecate agent is not running or not accessible"
    echo "   URL: $HECATE_AGENT_BASE_URL"
fi

echo -e "\n💡 Recommendations:"

# Check if any API keys are set
if [[ -z "$OPENAI_API_KEY" && -z "$ANTHROPIC_API_KEY" && -z "$GROQ_API_KEY" && -z "$HUGGINGFACE_API_KEY" ]]; then
    echo "❌ No API keys found. You need to set at least one:"
    echo "   export OPENAI_API_KEY=\"your-key-here\""
    echo "   export ANTHROPIC_API_KEY=\"your-key-here\""
    echo "   export GROQ_API_KEY=\"your-key-here\""
    echo "   export HUGGINGFACE_API_KEY=\"your-key-here\""
else
    echo "✅ API keys are configured"
fi

# Check if local services are running
if ! curl -s http://localhost:1234/v1/models > /dev/null 2>&1 && ! curl -s http://localhost:11434/api/tags > /dev/null 2>&1; then
    echo "❌ No local LLM services running. Consider:"
    echo "   - Starting LM Studio and loading a model"
    echo "   - Installing and starting Ollama: ollama serve"
fi

echo -e "\n🚀 Quick Fix Commands:"

if [[ -z "$OPENAI_API_KEY" ]]; then
    echo "# Set OpenAI API key (most common):"
    echo "export OPENAI_API_KEY=\"sk-your-key-here\""
    echo ""
fi

echo "# Test Hecate after setting up models:"
echo "curl -X POST \"${HECATE_AGENT_BASE_URL}/chat\" \\"
echo "  -H \"Content-Type: application/json\" \\"
echo "  -d '{\"message\": \"Hello!\", \"user_context\": {\"wallet_address\": \"0x1234567890abcdef\"}}'"

echo -e "\n✅ Diagnostic complete!"
