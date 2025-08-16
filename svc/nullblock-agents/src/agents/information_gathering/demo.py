"""
Demo script for Information Gathering Agent with MCP integration

This script demonstrates the complete pipeline from information gathering agent
to MCP server data source access, analysis, and pattern detection.
"""

import asyncio
import logging
from datetime import datetime
from typing import List

from .main import InformationGatheringAgent, DataRequest, AnalysisResult

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)

logger = logging.getLogger(__name__)

async def demo_price_analysis():
    """Demo price oracle data analysis"""
    print("\n🔍 DEMO: Price Oracle Analysis")
    print("=" * 50)
    
    # Initialize the information gathering agent
    agent = InformationGatheringAgent(mcp_server_url="http://localhost:8000")
    
    try:
        print("📡 Starting Information Gathering Agent...")
        # Start the agent (this will also start background loops)
        await asyncio.sleep(1)  # Give agent time to initialize
        
        # Initialize MCP connection manually for demo
        await agent.mcp_client.connect()
        print("✅ Connected to MCP server")
        
        # Test data source availability
        sources = await agent.mcp_client.get_available_sources()
        print(f"📋 Available data sources: {sources}")
        
        # Analyze market trends for popular tokens
        symbols = ["bitcoin", "ethereum", "solana"]
        print(f"\n📈 Analyzing market trends for: {', '.join(symbols)}")
        
        result = await agent.analyze_market_trends(symbols, timeframe="24h")
        
        print(f"\n📊 Analysis Results:")
        print(f"   Source: {result.source_name}")
        print(f"   Confidence: {result.confidence_score:.2%}")
        print(f"   Timestamp: {result.timestamp}")
        
        print(f"\n💡 Insights:")
        for insight in result.insights:
            print(f"   • {insight}")
        
        print(f"\n🔍 Patterns Detected:")
        for pattern in result.patterns_detected:
            print(f"   • {pattern}")
        
        print(f"\n⚠️  Anomalies:")
        for anomaly in result.anomalies:
            print(f"   • {anomaly}")
        
        print(f"\n💡 Recommendations:")
        for rec in result.recommendations:
            print(f"   • {rec}")
            
    except Exception as e:
        logger.error(f"Demo failed: {e}")
        print(f"❌ Demo failed: {e}")
    finally:
        await agent.mcp_client.disconnect()
        print("🔌 Disconnected from MCP server")

async def demo_defi_opportunities():
    """Demo DeFi opportunity detection"""
    print("\n🏦 DEMO: DeFi Opportunity Detection")
    print("=" * 50)
    
    agent = InformationGatheringAgent(mcp_server_url="http://localhost:8000")
    
    try:
        await agent.mcp_client.connect()
        print("✅ Connected to MCP server")
        
        # Analyze DeFi protocols
        protocols = ["uniswap", "aave", "compound"]
        print(f"\n💰 Analyzing DeFi opportunities in: {', '.join(protocols)}")
        
        result = await agent.detect_defi_opportunities(protocols)
        
        print(f"\n📊 DeFi Analysis Results:")
        print(f"   Source: {result.source_name}")
        print(f"   Confidence: {result.confidence_score:.2%}")
        
        print(f"\n💡 Opportunities:")
        for insight in result.insights:
            print(f"   • {insight}")
        
        print(f"\n🔍 Patterns:")
        for pattern in result.patterns_detected:
            print(f"   • {pattern}")
        
        print(f"\n💡 Recommendations:")
        for rec in result.recommendations:
            print(f"   • {rec}")
            
    except Exception as e:
        logger.error(f"DeFi demo failed: {e}")
        print(f"❌ DeFi demo failed: {e}")
    finally:
        await agent.mcp_client.disconnect()

async def demo_real_time_monitoring():
    """Demo real-time data monitoring"""
    print("\n⚡ DEMO: Real-time Data Monitoring")
    print("=" * 50)
    
    agent = InformationGatheringAgent(mcp_server_url="http://localhost:8000")
    
    try:
        await agent.mcp_client.connect()
        print("✅ Connected to MCP server")
        
        # Monitor real-time price data
        symbols = ["ethereum"]
        print(f"\n⏱️  Monitoring real-time data for: {', '.join(symbols)}")
        
        for i in range(3):  # Monitor for 3 iterations
            print(f"\n📡 Update {i+1}:")
            
            data = await agent.get_real_time_data(
                source_type="price_oracle",
                source_name="coingecko",
                parameters={"symbols": symbols, "vs_currency": "usd"}
            )
            
            print(f"   Data: {data}")
            
            # Wait before next update
            if i < 2:
                await asyncio.sleep(2)
        
        print("\n✅ Real-time monitoring completed")
        
    except Exception as e:
        logger.error(f"Real-time demo failed: {e}")
        print(f"❌ Real-time demo failed: {e}")
    finally:
        await agent.mcp_client.disconnect()

async def demo_custom_analysis():
    """Demo custom data analysis request"""
    print("\n🎯 DEMO: Custom Analysis Request")
    print("=" * 50)
    
    agent = InformationGatheringAgent(mcp_server_url="http://localhost:8000")
    
    try:
        await agent.mcp_client.connect()
        print("✅ Connected to MCP server")
        
        # Create custom analysis request
        request = DataRequest(
            source_type="price_oracle",
            source_name="coingecko",
            parameters={
                "symbols": ["bitcoin", "ethereum"],
                "timeframe": "7d"
            },
            analysis_type="correlation",
            context={"analysis_goal": "correlation_analysis"}
        )
        
        print(f"\n📝 Submitting custom analysis request:")
        print(f"   Source: {request.source_type}/{request.source_name}")
        print(f"   Analysis: {request.analysis_type}")
        print(f"   Parameters: {request.parameters}")
        
        request_id = await agent.request_data_analysis(request)
        print(f"   Request ID: {request_id}")
        
        # Wait for analysis to complete
        print(f"\n⏳ Waiting for analysis to complete...")
        
        for attempt in range(10):  # Wait up to 10 seconds
            result = await agent.get_analysis_result(request_id)
            if result:
                print(f"\n✅ Analysis completed!")
                print(f"   Confidence: {result.confidence_score:.2%}")
                print(f"   Insights: {len(result.insights)} found")
                print(f"   Patterns: {len(result.patterns_detected)} detected")
                break
            
            await asyncio.sleep(1)
        else:
            print(f"\n⏰ Analysis timed out")
        
    except Exception as e:
        logger.error(f"Custom analysis demo failed: {e}")
        print(f"❌ Custom analysis demo failed: {e}")
    finally:
        await agent.mcp_client.disconnect()

async def demo_agent_status():
    """Demo agent status and health monitoring"""
    print("\n🩺 DEMO: Agent Status & Health")
    print("=" * 50)
    
    agent = InformationGatheringAgent(mcp_server_url="http://localhost:8000")
    
    try:
        await agent.mcp_client.connect()
        print("✅ Connected to MCP server")
        
        # Check MCP server health
        health = await agent.mcp_client.health_check()
        print(f"\n💓 MCP Server Health:")
        for key, value in health.items():
            print(f"   {key}: {value}")
        
        # Check MCP connection status
        connection_status = agent.mcp_client.get_connection_status()
        print(f"\n🔗 MCP Connection Status:")
        for key, value in connection_status.items():
            print(f"   {key}: {value}")
        
        # Check agent statistics
        print(f"\n📊 Agent Statistics:")
        print(f"   Active requests: {len(agent.active_requests)}")
        print(f"   Cached results: {len(agent.analysis_cache)}")
        print(f"   Running: {agent.running}")
        
    except Exception as e:
        logger.error(f"Status demo failed: {e}")
        print(f"❌ Status demo failed: {e}")
    finally:
        await agent.mcp_client.disconnect()

async def run_all_demos():
    """Run all demo scenarios"""
    print("🚀 INFORMATION GATHERING AGENT DEMO")
    print("=" * 60)
    print("This demo showcases the Information Gathering Agent")
    print("connecting to the Nullblock MCP server for data analysis.")
    print("=" * 60)
    
    # Run demos sequentially
    demos = [
        demo_agent_status,
        demo_price_analysis,
        demo_defi_opportunities,
        demo_real_time_monitoring,
        demo_custom_analysis
    ]
    
    for i, demo in enumerate(demos, 1):
        print(f"\n🎬 Running Demo {i}/{len(demos)}")
        try:
            await demo()
        except Exception as e:
            logger.error(f"Demo {i} failed: {e}")
            print(f"❌ Demo {i} failed: {e}")
        
        if i < len(demos):
            print(f"\n⏸️  Waiting 2 seconds before next demo...")
            await asyncio.sleep(2)
    
    print(f"\n🎉 All demos completed!")
    print("=" * 60)

if __name__ == "__main__":
    print("Starting Information Gathering Agent Demo...")
    
    # Add instruction for starting MCP server
    print("\n📋 Prerequisites:")
    print("1. Start the Nullblock MCP server:")
    print("   cd svc/nullblock-mcp")
    print("   python -m mcp.server")
    print("2. Server should be running on http://localhost:8000")
    print("\nPress Enter to continue with demo...")
    # input()  # Uncomment to wait for user input
    
    try:
        asyncio.run(run_all_demos())
    except KeyboardInterrupt:
        print("\n🛑 Demo interrupted by user")
    except Exception as e:
        logger.error(f"Demo failed: {e}")
        print(f"❌ Demo failed: {e}")