#!/usr/bin/env python3
"""Test Hecate Agent API endpoints"""

import asyncio
import aiohttp
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

async def test_hecate_api():
    """Test Hecate Agent API endpoints"""
    
    print("🧪 Testing Hecate Agent API Endpoints")
    print("="*50)
    
    # NOTE: This assumes the Hecate agent server is running on port 9002
    # You can start it with: cd svc/nullblock-agents && python -m agents.hecate.server
    
    base_url = "http://localhost:9002"
    
    async with aiohttp.ClientSession() as session:
        
        # Test 1: Health Check
        print("\n1. 🏥 Health Check:")
        try:
            async with session.get(f"{base_url}/health") as resp:
                if resp.status == 200:
                    data = await resp.json()
                    print(f"   ✅ Status: {data['status']}")
                    print(f"   🤖 Agent running: {data['agent']['agent_running']}")
                else:
                    print(f"   ❌ Health check failed: HTTP {resp.status}")
        except Exception as e:
            print(f"   ❌ Health check error: {e}")
            print("   💡 Make sure Hecate server is running: python -m agents.hecate.server")
            return
        
        # Test 2: Available Models
        print("\n2. 📋 Available Models:")
        try:
            async with session.get(f"{base_url}/available-models") as resp:
                if resp.status == 200:
                    data = await resp.json()
                    print(f"   📊 Total models: {len(data['models'])}")
                    print(f"   🎯 Default model: {data['default_model']}")
                    print(f"   ✅ Available models:")
                    
                    available_count = 0
                    for model in data['models']:
                        if model['available']:
                            available_count += 1
                            cost = f"${model['cost_per_1k_tokens']:.4f}" if model['cost_per_1k_tokens'] > 0 else "FREE"
                            reasoning = "🧠" if model.get('supports_reasoning', False) else "  "
                            print(f"      {reasoning} {model['name']} ({model['provider']}) - {cost}")
                    
                    print(f"   📈 {available_count}/{len(data['models'])} models available")
                    print(f"   🆓 Free model: {data['recommended_models']['free']}")
                    print(f"   🧠 Reasoning model: {data['recommended_models']['reasoning']}")
                    
                else:
                    print(f"   ❌ Failed: HTTP {resp.status}")
                    
        except Exception as e:
            print(f"   ❌ Error: {e}")
        
        # Test 3: Model Status
        print("\n3. 🔍 Model Status:")
        try:
            async with session.get(f"{base_url}/model-status") as resp:
                if resp.status == 200:
                    data = await resp.json()
                    print(f"   ✅ Retrieved model status: {json.dumps(data, indent=4)}")
                else:
                    print(f"   ❌ Failed: HTTP {resp.status}")
        except Exception as e:
            print(f"   ❌ Error: {e}")
        
        # Test 4: Set Model (test with default)
        print("\n4. 🎯 Set Model (to default):")
        try:
            payload = {"model_name": "deepseek/deepseek-chat-v3.1:free"}
            async with session.post(f"{base_url}/set-model", json=payload) as resp:
                if resp.status == 200:
                    data = await resp.json()
                    print(f"   ✅ Model set successfully: {data['model']}")
                    print(f"   🔄 Previous model: {data.get('previous_model', 'None')}")
                else:
                    text = await resp.text()
                    print(f"   ❌ Failed: HTTP {resp.status} - {text}")
        except Exception as e:
            print(f"   ❌ Error: {e}")
        
        # Test 5: Chat (simple test)
        print("\n5. 💬 Chat Test:")
        try:
            payload = {"message": "Hello! What model are you using?"}
            async with session.post(f"{base_url}/chat", json=payload) as resp:
                if resp.status == 200:
                    data = await resp.json()
                    print(f"   ✅ Chat response received")
                    print(f"   🤖 Model used: {data['model_used']}")
                    print(f"   💬 Response: {data['content'][:100]}...")
                    print(f"   ⚡ Latency: {data['latency_ms']:.0f}ms")
                    print(f"   💰 Processing time: {data['processing_time_s']:.2f}s")
                else:
                    text = await resp.text()
                    print(f"   ❌ Failed: HTTP {resp.status} - {text}")
        except Exception as e:
            print(f"   ❌ Error: {e}")
        
        print("\n" + "="*50)
        print("✅ API testing completed!")
        print("💡 If any tests failed, make sure:")
        print("   1. Hecate server is running: python -m agents.hecate.server")
        print("   2. OpenRouter API key is set in .env.dev")
        print("   3. Server is accessible on localhost:9002")

if __name__ == "__main__":
    asyncio.run(test_hecate_api())