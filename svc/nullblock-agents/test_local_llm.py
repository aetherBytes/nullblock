#!/usr/bin/env python3
"""
Test script for local LLM integration with Gemma3 270M via LM Studio

This script demonstrates the updated LLM Service Factory's ability to:
1. Detect missing API keys and fall back to local models
2. Use Gemma3 270M via LM Studio as default when available
3. Provide helpful error messages for setup issues

Usage:
    python test_local_llm.py
"""

import asyncio
import sys
import os
import logging

# Add the agent packages to the path
sys.path.insert(0, 'src')

from agents.llm_service.factory import LLMServiceFactory, LLMRequest
from agents.llm_service.router import TaskRequirements, OptimizationGoal, Priority
from agents.llm_service.models import ModelCapability

# Setup logging
logging.basicConfig(level=logging.INFO, format='%(levelname)s: %(message)s')
logger = logging.getLogger(__name__)

async def test_local_llm():
    """Test local LLM functionality"""
    print("🤖 Testing Local LLM Integration with Gemma3 270M")
    print("=" * 60)
    
    # Initialize factory
    factory = LLMServiceFactory()
    await factory.initialize()
    
    try:
        # Health check
        print("\n📋 LLM Service Health Check:")
        health = await factory.health_check()
        print(f"Overall Status: {health['overall_status']}")
        print(f"Models Available: {health['models_available']}")
        print(f"Default Model: {health['default_model']}")
        
        if health['local_providers']:
            print("Local Providers:")
            for provider, status in health['local_providers'].items():
                print(f"  {provider}: {'✅' if status else '❌'}")
        
        if health['api_providers']:
            api_available = sum(health['api_providers'].values())
            print(f"API Keys Available: {api_available}/4")
        
        if health['issues']:
            print("Issues:")
            for issue in health['issues']:
                print(f"  ⚠️  {issue}")
        
        # Test local connectivity
        print("\n🔌 Testing Local Model Connectivity:")
        local_status = await factory.test_local_connectivity()
        for service, connected in local_status.items():
            status_icon = "✅" if connected else "❌"
            print(f"  {service}: {status_icon}")
        
        # Test generation with different scenarios
        test_prompt = "Hello! Please introduce yourself briefly."
        
        print(f"\n💬 Testing Generation with prompt: '{test_prompt}'")
        print("-" * 40)
        
        # Test 1: Default behavior (should prefer local if no API keys)
        print("\n1️⃣ Default Generation (auto-routing):")
        try:
            response = await factory.quick_generate(test_prompt, "test", "balanced")
            print(f"✅ Response: {response[:100]}{'...' if len(response) > 100 else ''}")
        except Exception as e:
            print(f"❌ Error: {e}")
        
        # Test 2: Force local model
        print("\n2️⃣ Forced Local Model (Gemma3):")
        try:
            request = LLMRequest(prompt=test_prompt, model_override="gemma-3-270m-it-mlx")
            requirements = TaskRequirements(
                required_capabilities=[ModelCapability.CONVERSATION],
                optimization_goal=OptimizationGoal.COST,  # Favors local
                priority=Priority.MEDIUM,
                task_type="test"
            )
            response = await factory.generate(request, requirements)
            print(f"✅ Model Used: {response.model_used}")
            print(f"✅ Response: {response.content[:100]}{'...' if len(response.content) > 100 else ''}")
            print(f"💰 Cost: ${response.cost_estimate:.4f}")
            print(f"⏱️ Latency: {response.latency_ms:.0f}ms")
        except Exception as e:
            print(f"❌ Error: {e}")
            if "Cannot connect" in str(e):
                print("💡 Tip: Make sure LM Studio is running with Gemma3 270M loaded")
        
        # Test 3: Speed optimization (should prefer fastest available)
        print("\n3️⃣ Speed-Optimized Generation:")
        try:
            requirements = TaskRequirements(
                required_capabilities=[ModelCapability.CONVERSATION],
                optimization_goal=OptimizationGoal.SPEED,
                priority=Priority.HIGH,
                task_type="test"
            )
            response = await factory.generate(LLMRequest(prompt=test_prompt), requirements)
            print(f"✅ Model Used: {response.model_used}")
            print(f"⏱️ Latency: {response.latency_ms:.0f}ms")
        except Exception as e:
            print(f"❌ Error: {e}")
        
        # Show statistics
        print("\n📊 Usage Statistics:")
        stats = factory.get_stats()
        if stats['request_stats']:
            for model, count in stats['request_stats'].items():
                cost = stats['cost_tracking'].get(model, 0)
                print(f"  {model}: {count} requests, ${cost:.4f} cost")
        else:
            print("  No requests completed")
        
    except Exception as e:
        logger.error(f"Test failed: {e}")
        return False
    
    finally:
        await factory.cleanup()
    
    print("\n✅ Local LLM test completed!")
    return True

async def main():
    """Main test function"""
    print("🧪 Nullblock Local LLM Test Suite")
    print("This test demonstrates local model integration with fallback behavior")
    print("\nPrerequisites:")
    print("1. LM Studio running on localhost:1234")
    print("2. Gemma3 270M model loaded in LM Studio")
    print("3. No API keys required (will test fallback behavior)")
    
    # Check environment
    api_keys_present = any([
        os.getenv('OPENAI_API_KEY'),
        os.getenv('ANTHROPIC_API_KEY'),
        os.getenv('GROQ_API_KEY'),
        os.getenv('HUGGINGFACE_API_KEY')
    ])
    
    if api_keys_present:
        print("\n⚠️  Note: API keys detected. Local model fallback behavior may not be fully demonstrated.")
    else:
        print("\n✅ No API keys detected. Perfect for testing local model fallback!")
    
    input("\nPress Enter to continue...")
    
    success = await test_local_llm()
    
    if success:
        print("\n🎉 All tests completed successfully!")
        print("\nNext steps:")
        print("1. The LLM Service Factory now automatically falls back to local models")
        print("2. Gemma3 270M is prioritized when LM Studio is available")
        print("3. Agents can use local models for cost-effective development")
    else:
        print("\n❌ Tests failed. Please check LM Studio setup and try again.")
        sys.exit(1)

if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\n🛑 Test interrupted by user")
    except Exception as e:
        print(f"❌ Test failed: {e}")
        sys.exit(1)