#!/usr/bin/env python3
"""
Test script for Hecate Agent integration

This script tests the full integration of the Hecate agent with the frontend.
Run this to verify everything is working correctly.
"""

import asyncio
import sys
import os

# Add the agents path to allow imports
sys.path.append(os.path.join(os.path.dirname(__file__), 'svc', 'nullblock-agents', 'src'))

from agents.hecate.main import HecateAgent

async def test_hecate_agent():
    """Test the Hecate agent functionality"""
    print("🤖 Testing Hecate Agent Integration")
    print("=" * 50)
    
    # Initialize agent
    agent = HecateAgent(personality="helpful_cyberpunk")
    
    try:
        # Start the agent
        print("1. Starting Hecate agent...")
        await agent.start()
        print("   ✅ Agent started successfully")
        
        # Test model status
        print("\n2. Testing model status...")
        status = await agent.get_model_status()
        print(f"   Status: {status['status']}")
        print(f"   Current model: {status.get('current_model', 'None')}")
        
        # Test chat functionality
        print("\n3. Testing chat functionality...")
        
        # Simple conversation
        response1 = await agent.chat("Hello Hecate!")
        print(f"   User: Hello Hecate!")
        print(f"   Hecate ({response1.model_used}): {response1.content}")
        
        # Test orchestration keywords
        response2 = await agent.chat("Can you analyze the current market trends for Bitcoin?")
        print(f"\n   User: Can you analyze the current market trends for Bitcoin?")
        print(f"   Hecate ({response2.model_used}): {response2.content}")
        print(f"   Response type: {response2.metadata.get('response_type', 'normal')}")
        
        # Test conversation history
        print("\n4. Testing conversation history...")
        history = agent.get_conversation_history()
        print(f"   Conversation length: {len(history)} messages")
        
        print("\n✅ All tests passed!")
        print("\n🚀 Hecate Agent is ready for frontend integration!")
        print("\nNext steps:")
        print("1. Start the Hecate server: cd svc/nullblock-agents && python -m agents.hecate.server")
        print("2. Start the frontend: cd svc/hecate && npm run develop")
        print("3. Open chat interface and interact with Hecate")
        print("\n🎨 Visual Features:")
        print("- NullEye avatars show red 'idle' theme when agent is offline")
        print("- Chat header displays current model and connection status")
        print("- Graceful error handling with fallback messages")
        
    except Exception as e:
        print(f"\n❌ Test failed: {e}")
        return False
    
    finally:
        # Stop the agent
        await agent.stop()
        print("\n🛑 Agent stopped")
    
    return True

if __name__ == "__main__":
    success = asyncio.run(test_hecate_agent())
    sys.exit(0 if success else 1)