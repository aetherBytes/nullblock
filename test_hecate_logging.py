#!/usr/bin/env python3
"""
Test script to verify Hecate agent logging is working correctly
"""

import asyncio
import sys
import os
from pathlib import Path

# Add the agents path to allow imports
sys.path.append(os.path.join(os.path.dirname(__file__), 'svc', 'nullblock-agents', 'src'))

from agents.hecate.main import HecateAgent
from agents.logging_config import setup_agent_logging, log_agent_startup

async def test_hecate_logging():
    """Test Hecate agent logging functionality"""
    print("🧪 Testing Hecate Agent Logging")
    print("=" * 50)
    
    # Create logs directory
    os.makedirs("logs", exist_ok=True)
    
    # Setup test logger
    test_logger = setup_agent_logging("hecate-test", "INFO", enable_file_logging=True)
    log_agent_startup(test_logger, "hecate-test", "1.0.0")
    
    # Initialize agent
    agent = HecateAgent(personality="helpful_cyberpunk")
    
    try:
        # Start the agent
        print("1. Starting Hecate agent...")
        await agent.start()
        print("   ✅ Agent started successfully")
        
        # Test chat functionality with logging
        print("\n2. Testing chat with logging...")
        
        test_messages = [
            "Hello Hecate!",
            "Can you analyze market trends for Bitcoin?",
            "What's the weather like?",
        ]
        
        for i, message in enumerate(test_messages, 1):
            print(f"\n   Test {i}: {message}")
            response = await agent.chat(message)
            print(f"   Response: {response.content[:100]}{'...' if len(response.content) > 100 else ''}")
            print(f"   Model: {response.model_used} | Latency: {response.latency_ms:.0f}ms")
        
        # Check log files
        print("\n3. Checking log files...")
        log_files = [
            "logs/hecate-test.log",
            "logs/hecate.log",
            "logs/hecate-server.log"
        ]
        
        for log_file in log_files:
            if os.path.exists(log_file):
                print(f"   ✅ {log_file} exists")
                with open(log_file, 'r') as f:
                    lines = f.readlines()
                    print(f"      📊 {len(lines)} lines")
                    if lines:
                        print(f"      📝 Latest: {lines[-1].strip()}")
            else:
                print(f"   ❌ {log_file} not found")
        
        print("\n✅ Logging test completed!")
        
    except Exception as e:
        print(f"\n❌ Test failed: {e}")
        return False
    
    finally:
        # Stop the agent
        await agent.stop()
        print("\n🛑 Agent stopped")
    
    return True

if __name__ == "__main__":
    success = asyncio.run(test_hecate_logging())
    sys.exit(0 if success else 1)