#!/usr/bin/env python3
"""Simple reasoning test"""

import asyncio
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

async def test_reasoning():
    factory = LLMServiceFactory()
    await factory.initialize()
    
    print('🧠 Testing Fixed Reasoning')
    
    try:
        response = await factory.generate_with_reasoning(
            prompt="What is 15 * 23? Please show your calculation.",
            model_name="deepseek/deepseek-r1",
            effort="high"
        )
        print(f"Model: {response.model_used}")
        print(f"Response: {response.content}")
        print(f"Has reasoning: {'Yes' if response.reasoning else 'No'}")
        print(f"Cost: ${response.cost_estimate:.4f}")
        
        if response.reasoning:
            print(f"Reasoning length: {len(response.reasoning)} characters")
            print(f"Reasoning preview: {response.reasoning[:300]}...")
        
    except Exception as e:
        print(f"Error: {e}")
    
    await factory.cleanup()

asyncio.run(test_reasoning())