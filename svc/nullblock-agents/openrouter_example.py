#!/usr/bin/env python3
"""
OpenRouter Integration Example

This script demonstrates how to use the updated LLM service with OpenRouter.
Set your OPENROUTER_API_KEY environment variable before running.

Usage:
    export OPENROUTER_API_KEY="your_api_key_here"
    python openrouter_example.py
"""

import asyncio
import os
from src.agents.llm_service.factory import LLMServiceFactory, LLMRequest
from src.agents.llm_service.router import TaskRequirements, OptimizationGoal, Priority
from src.agents.llm_service.models import ModelCapability, get_default_hecate_model

async def main():
    # Check if OpenRouter API key is set
    if not os.getenv('OPENROUTER_API_KEY'):
        print("❌ Please set your OPENROUTER_API_KEY environment variable")
        print("   export OPENROUTER_API_KEY='your_api_key_here'")
        return
    
    print("🚀 Testing OpenRouter Integration")
    print("=" * 50)
    
    # Initialize the LLM service factory
    factory = LLMServiceFactory()
    await factory.initialize()
    
    # Check available providers
    providers = factory.get_available_providers()
    print(f"Available providers: {providers}")
    
    # Run health check
    health = await factory.health_check()
    print(f"Health status: {health['overall_status']}")
    print(f"Available models: {health['models_available']}")
    
    if health['overall_status'] == 'unhealthy':
        print("❌ Service is unhealthy, cannot proceed with examples")
        await factory.cleanup()
        return
    
    print("\n" + "=" * 50)
    print("🧪 Running Example Requests")
    print("=" * 50)
    
    # Show default Hecate model
    default_model = factory.get_default_hecate_model()
    print(f"\n🎯 Default Hecate model: {default_model}")
    
    # Example 1: Default Hecate model (DeepSeek free)
    print("\n1. Default Hecate model (DeepSeek free):")
    try:
        request = LLMRequest(
            prompt="Write a haiku about artificial intelligence",
            model_override="deepseek/deepseek-chat-v3.1:free"
        )
        
        requirements = TaskRequirements(
            required_capabilities=[ModelCapability.CREATIVE],
            optimization_goal=OptimizationGoal.COST,  # Cost optimization will prefer free models
            priority=Priority.MEDIUM
        )
        
        response = await factory.generate(request, requirements)
        print(f"Model used: {response.model_used}")
        print(f"Response: {response.content}")
        print(f"Cost estimate: ${response.cost_estimate:.4f}")
        print(f"Latency: {response.latency_ms:.0f}ms")
        
    except Exception as e:
        print(f"Error: {e}")
    
    # Example 2: Code generation with Claude
    print("\n2. Code generation with Claude 3.5 Sonnet:")
    try:
        request = LLMRequest(
            prompt="Write a Python function to calculate fibonacci numbers using memoization",
            system_prompt="You are a helpful coding assistant. Write clean, efficient Python code.",
            model_override="anthropic/claude-3.5-sonnet"
        )
        
        requirements = TaskRequirements(
            required_capabilities=[ModelCapability.CODE],
            optimization_goal=OptimizationGoal.QUALITY,
            priority=Priority.HIGH
        )
        
        response = await factory.generate(request, requirements)
        print(f"Model used: {response.model_used}")
        print(f"Response: {response.content}")
        print(f"Cost estimate: ${response.cost_estimate:.4f}")
        
    except Exception as e:
        print(f"Error: {e}")
    
    # Example 3: Fast response with Llama
    print("\n3. Fast response with Llama 3.1:")
    try:
        request = LLMRequest(
            prompt="Explain quantum computing in one sentence",
            model_override="meta-llama/llama-3.1-8b-instruct"
        )
        
        requirements = TaskRequirements(
            required_capabilities=[ModelCapability.CONVERSATION],
            optimization_goal=OptimizationGoal.SPEED,
            priority=Priority.LOW
        )
        
        response = await factory.generate(request, requirements)
        print(f"Model used: {response.model_used}")
        print(f"Response: {response.content}")
        print(f"Cost estimate: ${response.cost_estimate:.4f}")
        print(f"Latency: {response.latency_ms:.0f}ms")
        
    except Exception as e:
        print(f"Error: {e}")
    
    # Example 4: Automatic model selection
    print("\n4. Automatic model selection (balanced optimization):")
    try:
        request = LLMRequest(
            prompt="What are the main differences between REST and GraphQL APIs?",
            concise=True,
            max_chars=200
        )
        
        requirements = TaskRequirements(
            required_capabilities=[ModelCapability.REASONING, ModelCapability.CONVERSATION],
            optimization_goal=OptimizationGoal.BALANCED,
            priority=Priority.MEDIUM
        )
        
        response = await factory.generate(request, requirements)
        print(f"Model used: {response.model_used}")
        print(f"Response: {response.content}")
        print(f"Cost estimate: ${response.cost_estimate:.4f}")
        
    except Exception as e:
        print(f"Error: {e}")
    
    # Show usage statistics
    print("\n" + "=" * 50)
    print("📊 Usage Statistics")
    print("=" * 50)
    stats = factory.get_stats()
    print(f"Request stats: {stats['request_stats']}")
    print(f"Cost tracking: {stats['cost_tracking']}")
    
    # Cleanup
    await factory.cleanup()
    print("\n✅ OpenRouter integration test completed!")

if __name__ == "__main__":
    asyncio.run(main())