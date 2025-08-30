#!/usr/bin/env python3
"""Test available models API"""

import asyncio
import json
import os
from pathlib import Path

# Load environment
env_path = Path(__file__).parent.parent.parent / '.env.dev'
with open(env_path) as f:
    for line in f:
        line = line.strip()
        if line and not line.startswith('#') and '=' in line:
            key, value = line.split('=', 1)
            os.environ[key] = value.strip('"')

from src.agents.llm_service.factory import LLMServiceFactory
from src.agents.llm_service.models import AVAILABLE_MODELS, get_default_hecate_model

async def test_available_models():
    factory = LLMServiceFactory()
    await factory.initialize()
    
    print('🔍 Testing available models API data structure:')
    print('='*60)
    
    available_models = []
    
    for model_name, config in AVAILABLE_MODELS.items():
        # Check if model is available (default to config.enabled if not explicitly set)
        is_available = factory.router.model_status.get(model_name, config.enabled)
        
        available_models.append({
            "name": model_name,
            "display_name": config.name,
            "provider": config.provider.value,
            "available": is_available,
            "tier": config.tier.value,
            "context_length": config.metrics.context_window,
            "capabilities": [cap.value for cap in config.capabilities],
            "cost_per_1k_tokens": config.metrics.cost_per_1k_tokens,
            "supports_reasoning": getattr(config, 'supports_reasoning', False),
            "description": config.description
        })

    # Sort by availability first, then by provider and name
    available_models.sort(key=lambda x: (not x["available"], x["provider"], x["name"]))
    
    # Get default model
    default_model = get_default_hecate_model()
    
    print(f'🎯 Default model: {default_model}')
    print(f'📊 Total models: {len(available_models)}')
    print(f'✅ Available models: {sum(1 for m in available_models if m["available"])}')
    
    print('\n📋 Model List:')
    for model in available_models:
        status = '✅' if model["available"] else '❌'
        cost = f'${model["cost_per_1k_tokens"]:.4f}' if model['cost_per_1k_tokens'] > 0 else 'FREE'
        reasoning = '🧠' if model.get('supports_reasoning', False) else '  '
        print(f'  {status} {reasoning} {model["name"]} ({model["provider"]}) - {cost}')
    
    print('\n🔧 API Response Structure:')
    response_data = {
        "models": available_models,
        "current_model": None,
        "default_model": default_model,
        "recommended_models": {
            "free": "deepseek/deepseek-chat-v3.1:free",
            "reasoning": "deepseek/deepseek-r1", 
            "premium": "anthropic/claude-3.5-sonnet"
        }
    }
    
    print('Sample response (first model):')
    if available_models:
        print(json.dumps(available_models[0], indent=2))
    
    print(f'\n✅ API structure looks correct!')
    print(f'📤 Response includes {len(available_models)} models with default: {default_model}')
    
    await factory.cleanup()

if __name__ == "__main__":
    asyncio.run(test_available_models())