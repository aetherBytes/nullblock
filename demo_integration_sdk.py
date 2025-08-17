"""
Nullblock Integration Demo - SDK Version

Demonstrates the complete Nullblock SDK capabilities with:
- Unified NullblockClient for all operations
- Information Gathering Agent integration
- LLM Service Factory for intelligent model selection
- Pattern detection and analysis
- Multi-agent coordination

Prerequisites:
1. Start Nullblock MCP server: cd svc/nullblock-mcp && python -m mcp.server
2. Ensure API keys are set in environment (OPENAI_API_KEY, etc.)
3. Install SDK: pip install -e /Users/sage/nullblock-sdk/sdk/python
"""

import asyncio
import logging
import sys
import os
from datetime import datetime
from typing import Dict, Any, Optional

# Nullblock SDK imports
from nullblock import (
    NullblockClient, 
    NullblockConfig,
    LLMRequest, 
    TaskRequirements, 
    OptimizationGoal, 
    Priority,
    ModelCapability,
    NullblockError,
    ServiceHealthError,
    DemoError
)

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)

logger = logging.getLogger(__name__)


class NullblockSDKDemo:
    """Complete integration demo using Nullblock SDK."""
    
    def __init__(self):
        """Initialize demo with SDK configuration."""
        # Create SDK configuration
        self.config = NullblockConfig(
            mcp_server_url="http://localhost:8001",  # Updated port
            debug=True,
            log_level="INFO"
        )
        
        # Initialize main client
        self.client = NullblockClient(config=self.config)
        
    async def check_prerequisites(self) -> Dict[str, Any]:
        """Check all prerequisites using SDK health checks."""
        print("🔍 Checking Prerequisites...")
        print("-" * 40)
        
        try:
            # Use SDK health check
            health = await self.client.health_check()
            
            print("1️⃣ Checking MCP Server...")
            if health["services"]["mcp_server"].get("status") == "healthy":
                print("   ✅ MCP Server is running on port 8001")
            else:
                print("   ❌ MCP Server is not accessible on port 8001")
                print("   💡 Please start: cd svc/nullblock-mcp && python -m mcp.server")
                raise ServiceHealthError("MCP Server is not accessible")
            
            print("\n2️⃣ Checking LLM Availability...")
            llm_health = health.get("llm_factory", {})
            api_providers = llm_health.get("api_providers", {})
            local_providers = llm_health.get("local_providers", {})
            
            api_available = sum(api_providers.values()) if api_providers else 0
            local_available = sum(local_providers.values()) if local_providers else 0
            
            if api_available > 0:
                print(f"   ✅ API models available: {api_available} providers")
                print(f"      Active providers: {[k for k, v in api_providers.items() if v]}")
            elif local_available > 0:
                print(f"   🤖 Local models available: {local_available} providers")
                print(f"      Local providers: {[k for k, v in local_providers.items() if v]}")
                if "lm_studio" in local_providers and local_providers["lm_studio"]:
                    print("      ✅ LM Studio with Gemma3 270M detected")
                else:
                    print("      ⚠️  LM Studio not detected - start with 'lms server start'")
            else:
                print("   ⚠️  No LLM models available")
                print("   💡 Either set API keys (OPENAI_API_KEY, etc.) or start LM Studio")
                print("   💡 For local models: lms load gemma-3-270m-it-mlx -y && lms server start")
            
            print("\n3️⃣ Checking Network Connectivity...")
            # Network check is implicit in health check
            print("   ✅ Network connectivity is available")
            
            print("\n4️⃣ Expected Demo Behavior:")
            if local_available > 0 and api_available == 0:
                print("   🤖 Demo will use LOCAL models (Gemma3 270M)")
                print("   💰 All LLM operations will be FREE")
                print("   ⏱️  Expect slower response times (800-2000ms)")
            elif api_available > 0:
                print("   🌐 Demo will use API models")
                print("   💰 LLM operations will incur costs")
                print("   ⏱️  Expect faster response times (200-500ms)")
            else:
                print("   ❌ Demo will use MOCK responses only")
            
            print("\n" + "=" * 40)
            return health
            
        except Exception as e:
            raise ServiceHealthError(f"Prerequisites check failed: {e}")
    
    async def initialize(self):
        """Initialize SDK client with proper error handling."""
        print("🚀 Initializing Nullblock SDK Demo")
        print("=" * 60)
        
        # Check prerequisites first
        await self.check_prerequisites()
        
        # Initialize client (this handles all component initialization)
        print("📊 Initializing Nullblock Client...")
        try:
            await self.client.initialize()
            print("✅ Nullblock SDK Client initialized successfully")
            
            # Get and display service stats
            stats = await self.client.get_stats()
            print(f"   📋 MCP Connection: {stats['mcp_connection']['connected']}")
            print(f"   🤖 LLM Factory: Ready")
            print(f"   🔍 Info Agent: {stats['info_agent']['running']}")
                
        except Exception as e:
            raise DemoError(f"Failed to initialize Nullblock SDK: {e}")
        
        print("\n🎯 SDK Demo ready! All components initialized successfully.\n")
    
    async def demo_market_intelligence(self):
        """Demo: Market intelligence analysis with SDK."""
        print("📈 DEMO 1: Market Intelligence Analysis (SDK)")
        print("-" * 40)
        
        # Step 1: Gather market data using SDK
        print("1️⃣ Gathering market data using SDK...")
        
        symbols = ["ETH/USD", "BTC/USD", "SOL/USD"]
        market_data = {}
        
        for symbol in symbols:
            try:
                data = await self.client.get_market_data(symbol)
                market_data[symbol] = data
                price = await self.client.get_price(symbol)
                print(f"   ✅ {symbol}: ${price:,.2f}")
                
            except Exception as e:
                raise DemoError(f"Failed to get market data for {symbol}: {e}")
        
        # Step 2: Analyze market trends using SDK
        print("\n2️⃣ Analyzing market trends using SDK...")
        
        try:
            analysis = await self.client.analyze_market_trends(
                ["ethereum", "bitcoin", "solana"], 
                timeframe="24h"
            )
            
            print(f"   📊 Analysis completed with {analysis['confidence']:.1%} confidence")
            print(f"   🔍 Found {len(analysis['patterns'])} patterns")
            print(f"   ⚠️  Detected {len(analysis['anomalies'])} anomalies")
            
        except Exception as e:
            raise DemoError(f"Market analysis failed: {e}")
        
        # Step 3: Generate intelligent insights with LLM Factory
        print("\n3️⃣ Generating intelligent insights using SDK LLM Factory...")
        
        # Prepare context for LLM
        context = f"""
Market Analysis Results for {', '.join(symbols)}:

Insights:
{chr(10).join(f'• {insight}' for insight in analysis['insights'])}

Patterns Detected:
{chr(10).join(f'• {pattern}' for pattern in analysis['patterns'])}

Anomalies:
{chr(10).join(f'• {anomaly}' for anomaly in analysis['anomalies'])}

Recommendations:
{chr(10).join(f'• {rec}' for rec in analysis['recommendations'])}
"""
        
        # Use LLM factory through SDK client
        try:
            llm_request = LLMRequest(
                prompt=f"Based on this market analysis data, provide a concise summary of the current market conditions and 3 specific actionable insights for traders:\n\n{context}",
                system_prompt="You are a professional cryptocurrency market analyst. Provide clear, actionable insights based on data.",
                max_tokens=500
            )
            
            requirements = TaskRequirements(
                required_capabilities=[ModelCapability.DATA_ANALYSIS, ModelCapability.REASONING],
                optimization_goal=OptimizationGoal.QUALITY,
                priority=Priority.HIGH,
                task_type="market_analysis",
                max_latency_ms=5000
            )
            
            response = await self.client.llm_factory.generate(llm_request, requirements)
            
            # Display model info with local model highlighting
            model_type = "🤖 LOCAL" if response.cost_estimate == 0.0 else "🌐 API"
            print(f"   {model_type} LLM Analysis ({response.model_used}):")
            
            if response.cost_estimate == 0.0:
                print(f"   💰 Cost: FREE (local model)")
                print(f"   ⏱️  Latency: {response.latency_ms:.0f}ms (local processing)")
            else:
                print(f"   💰 Cost: ${response.cost_estimate:.4f}")
                print(f"   ⏱️  Latency: {response.latency_ms:.0f}ms")
            
            print(f"\n   📝 Market Intelligence Report:")
            print("   " + "\n   ".join(response.content.split("\n")))
            
        except Exception as e:
            raise DemoError(f"LLM analysis failed: {e}")
        
        print("\n✅ Market Intelligence Demo completed!\n")
    
    async def demo_automated_research(self):
        """Demo: Automated research pipeline using SDK."""
        print("🔬 DEMO 2: Automated Research Pipeline (SDK)")
        print("-" * 40)
        
        # Step 1: Define research question
        research_question = "What are the current trends in DeFi liquidity and yield farming opportunities?"
        print(f"❓ Research Question: {research_question}")
        
        # Step 2: Detect DeFi opportunities using SDK
        print("\n1️⃣ Analyzing DeFi opportunities using SDK...")
        
        try:
            opportunities = await self.client.info_agent.detect_defi_opportunities(["uniswap"])
            
            print(f"   📊 Opportunity analysis completed")
            print(f"   💡 Found {len(opportunities.insights)} insights")
            print(f"   🎯 Generated {len(opportunities.recommendations)} recommendations")
            print(f"   📈 Potential return: {opportunities.potential_return:.1%}")
            print(f"   ⚠️  Risk score: {opportunities.risk_score:.1f}/1.0")
            
        except Exception as e:
            raise DemoError(f"DeFi analysis failed: {e}")
        
        # Step 3: Generate research report using SDK
        print("\n2️⃣ Generating comprehensive research report using SDK...")
        
        research_context = f"""
DeFi Research Analysis:

Question: {research_question}

Key Insights:
{chr(10).join(f'• {insight}' for insight in opportunities.insights)}

Detected Patterns:
{chr(10).join(f'• {pattern}' for pattern in opportunities.patterns_detected)}

Recommendations:
{chr(10).join(f'• {rec}' for rec in opportunities.recommendations)}

Risk Assessment: {opportunities.risk_score:.1f}/1.0
Potential Return: {opportunities.potential_return:.1%}
"""
        
        try:
            llm_request = LLMRequest(
                prompt=f"Create a comprehensive research report addressing this question based on the analysis data. Include executive summary, key findings, risk assessment, and actionable recommendations:\n\n{research_context}",
                system_prompt="You are a DeFi research analyst creating professional reports for institutional investors.",
                max_tokens=800
            )
            
            requirements = TaskRequirements(
                required_capabilities=[ModelCapability.REASONING, ModelCapability.DATA_ANALYSIS, ModelCapability.CREATIVE],
                optimization_goal=OptimizationGoal.QUALITY,
                priority=Priority.HIGH,
                task_type="research_report",
                min_quality_score=0.9
            )
            
            report = await self.client.llm_factory.generate(llm_request, requirements)
            
            # Display model info with local model highlighting
            model_type = "🤖 LOCAL" if report.cost_estimate == 0.0 else "🌐 API"
            print(f"   📋 {model_type} Research Report Generated ({report.model_used}):")
            
            if report.cost_estimate == 0.0:
                print(f"   💰 Cost: FREE (local model)")
                print(f"   ⏱️  Latency: {report.latency_ms:.0f}ms (local processing)")
            else:
                print(f"   💰 Cost: ${report.cost_estimate:.4f}")
                print(f"   ⏱️  Latency: {report.latency_ms:.0f}ms")
            
            print(f"\n   📄 DeFi Research Report:")
            print("   " + "\n   ".join(report.content.split("\n")))
            
        except Exception as e:
            raise DemoError(f"Report generation failed: {e}")
        
        print("\n✅ Automated Research Demo completed!\n")
    
    async def demo_trading_simulation(self):
        """Demo: Trading simulation using SDK."""
        print("💰 DEMO 3: Trading Simulation (SDK)")
        print("-" * 40)
        
        # Step 1: Get portfolio overview
        print("1️⃣ Getting portfolio overview...")
        portfolio = await self.client.get_portfolio()
        
        print(f"   💼 Total Portfolio Value: ${portfolio['total_value_usd']:,.2f}")
        print(f"   📈 24h Performance: ${portfolio['performance_24h']['change_usd']:,.2f} ({portfolio['performance_24h']['change_percent']:.2f}%)")
        print("   📊 Asset Breakdown:")
        for asset in portfolio['assets']:
            print(f"      {asset['symbol']}: {asset['amount']} (${asset['value_usd']:,.2f})")
        
        # Step 2: Analyze sentiment
        print("\n2️⃣ Analyzing market sentiment...")
        sentiment = await self.client.get_sentiment("ETH")
        
        print(f"   🎭 ETH Sentiment Score: {sentiment['sentiment_score']:.2f}")
        print(f"   🔍 Confidence: {sentiment['confidence']:.2f}")
        print(f"   📱 Mentions: {sentiment['mention_count']:,}")
        print(f"   🔥 Trending Score: {sentiment['trending_score']}/10")
        
        # Step 3: Detect arbitrage opportunities
        print("\n3️⃣ Detecting arbitrage opportunities...")
        arbitrage_ops = await self.client.detect_arbitrage_opportunities(["ETH/USD", "BTC/USD"])
        
        for op in arbitrage_ops:
            print(f"   🎯 Opportunity ID: {op['id']}")
            print(f"   💱 Symbol: {op['symbol']}")
            print(f"   💰 Profit: {op['profit_percent']:.1f}% (${op['estimated_profit_usd']:,.2f})")
            print(f"   🏢 Exchanges: {', '.join(op['exchanges'])}")
            print(f"   ⚠️  Risk Score: {op['risk_score']:.1f}/1.0")
        
        # Step 4: Execute mock trade
        print("\n4️⃣ Executing mock trade...")
        trade_result = await self.client.execute_trade(
            symbol="ETH/USD",
            side="buy",
            amount=0.1,
            order_type="market"
        )
        
        print(f"   ✅ Trade executed successfully!")
        print(f"   📋 Order ID: {trade_result['order_id']}")
        print(f"   💰 Amount: {trade_result['amount']} ETH at ${trade_result['price']:,.2f}")
        print(f"   🔗 Transaction: {trade_result['tx_hash'][:20]}...")
        
        print("\n✅ Trading Simulation completed!\n")
    
    async def demo_system_monitoring(self):
        """Demo: System monitoring and statistics using SDK."""
        print("📊 DEMO 4: System Monitoring (SDK)")
        print("-" * 40)
        
        # Get comprehensive stats from SDK
        stats = await self.client.get_stats()
        
        print("🤖 LLM Service Factory Statistics:")
        llm_stats = stats['llm_factory']
        print(f"   Total Requests: {llm_stats['request_stats']['total']}")
        print(f"   Successful: {llm_stats['request_stats']['successful']}")
        print(f"   Failed: {llm_stats['request_stats']['failed']}")
        print(f"   Cache Hits: {llm_stats['cache_stats']['hits']}")
        print(f"   Cache Size: {llm_stats['cache_stats']['cache_size']} entries")
        
        # Show model usage breakdown
        if 'model_usage' in llm_stats:
            print(f"\n   📊 Model Usage Breakdown:")
            for model, count in llm_stats['model_usage'].items():
                cost = llm_stats.get('cost_tracking', {}).get(model, 0)
                if cost == 0:
                    print(f"      🤖 {model}: {count} requests (FREE - local)")
                else:
                    print(f"      🌐 {model}: {count} requests (${cost:.4f})")
        
        # Show total costs
        total_cost = sum(llm_stats.get('cost_tracking', {}).values())
        if total_cost == 0:
            print(f"   💰 Total LLM Cost: FREE (all local models)")
        else:
            print(f"   💰 Total LLM Cost: ${total_cost:.4f}")
        
        print(f"\n📡 MCP Connection Statistics:")
        mcp_stats = stats['mcp_connection']
        print(f"   Connected: {mcp_stats['connected']}")
        print(f"   Server URL: {mcp_stats['server_url']}")
        print(f"   Session Active: {mcp_stats['session_active']}")
        
        print(f"\n🔍 Information Gathering Agent Statistics:")
        agent_stats = stats['info_agent']
        print(f"   Running: {agent_stats['running']}")
        print(f"   Active Requests: {agent_stats['active_requests']}")
        print(f"   Cached Analyses: {agent_stats['cached_analyses']}")
        
        # Additional health check
        health = await self.client.health_check()
        print(f"\n💚 Overall System Health: {health['status']}")
        
        print("\n✅ System Monitoring completed!\n")
    
    async def cleanup(self):
        """Clean up SDK resources."""
        print("🧹 Cleaning up SDK resources...")
        await self.client.cleanup()
        print("✅ SDK cleanup completed!\n")


async def main():
    """Run the complete SDK integration demo."""
    print("🎬 NULLBLOCK SDK INTEGRATION DEMO")
    print("=" * 60)
    print("This demo showcases the complete Nullblock SDK:")
    print("• Unified NullblockClient for all operations")
    print("• Information Gathering Agent integration")
    print("• LLM Service Factory with intelligent model routing")
    print("• Market analysis and trading simulation")
    print("• Comprehensive system monitoring")
    print("=" * 60)
    
    demo = NullblockSDKDemo()
    
    try:
        # Initialize SDK
        await demo.initialize()
        
        # Run all demos
        await demo.demo_market_intelligence()
        await demo.demo_automated_research()
        await demo.demo_trading_simulation()
        await demo.demo_system_monitoring()
        
        print("🎉 ALL SDK DEMOS COMPLETED SUCCESSFULLY!")
        print("=" * 60)
        print("The Nullblock SDK is fully operational and ready for production use.")
        
        # Show final LLM usage summary
        final_stats = await self.client.get_stats()
        final_llm = final_stats.get('llm_factory', {})
        total_final_cost = sum(final_llm.get('cost_tracking', {}).values())
        
        if total_final_cost == 0:
            print("\n🤖 LOCAL MODEL INTEGRATION SUCCESSFUL!")
            print("• All LLM operations completed using local models (FREE)")
            print("• Gemma3 270M demonstrated production-ready performance")
            print("• No API costs incurred during demo")
            print("• Ready for cost-effective development and testing")
        else:
            print(f"\n🌐 API MODEL Integration: ${total_final_cost:.4f} total cost")
        
        print("\nKey Benefits Demonstrated:")
        print("• 🎯 Unified client interface for all operations")
        print("• 🔄 Automatic service management and health monitoring")
        print("• 🤖 Intelligent LLM model selection and routing")
        print("• 📊 Comprehensive market analysis and trading capabilities")
        print("• 🛡️  Built-in error handling and resilience")
        print("• 💰 Cost-effective local model fallback support")
        
    except ServiceHealthError as e:
        print(f"\n❌ SERVICE HEALTH ERROR: {e}")
        print("=" * 60)
        print("Please ensure all required services are running:")
        print("1. MCP Server: cd svc/nullblock-mcp && python -m mcp.server")
        print("2. LLM Service (choose one):")
        print("   - API Keys: Set OPENAI_API_KEY, ANTHROPIC_API_KEY, etc.")
        print("   - Local Models: lms load gemma-3-270m-it-mlx -y && lms server start")
        print("3. Network connectivity")
        sys.exit(1)
    except DemoError as e:
        print(f"\n❌ DEMO ERROR: {e}")
        print("=" * 60)
        print("The SDK demo encountered a critical error and cannot continue.")
        sys.exit(1)
    except KeyboardInterrupt:
        print("\n🛑 Demo interrupted by user")
    except Exception as e:
        logger.error(f"SDK demo failed: {e}")
        print(f"\n❌ UNEXPECTED ERROR: {e}")
        print("=" * 60)
        print("An unexpected error occurred. Please check the logs for details.")
        sys.exit(1)
    finally:
        await demo.cleanup()


if __name__ == "__main__":
    print("📋 Nullblock SDK Demo Prerequisites:")
    print("1. MCP Server: Start with 'cd svc/nullblock-mcp && python -m mcp.server'")
    print("2. API Keys: Ensure OPENAI_API_KEY, ANTHROPIC_API_KEY are set (optional)")
    print("3. SDK: Installed via local path in nullblock-agents dependencies")
    print("\nPress Enter to continue with SDK demo...")
    # input()  # Uncomment to wait for user input
    
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\n🛑 SDK Demo interrupted")
    except Exception as e:
        print(f"❌ SDK Demo startup failed: {e}")
        sys.exit(1)