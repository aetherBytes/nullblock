"""
Nullblock Integration Demo

Demonstrates the complete Information Gathering Agent pipeline with:
- Nullblock MCP server for data source access
- LLM Service Factory for intelligent model selection
- Pattern detection and analysis
- Multi-agent coordination via orchestration

Prerequisites:
1. Start Nullblock MCP server: cd svc/nullblock-mcp && MCP_SERVER_HOST=0.0.0.0 MCP_SERVER_PORT=8001 python -m mcp.server
2. Ensure API keys are set in environment (OPENAI_API_KEY, etc.)
3. Install dependencies: pip install -e svc/nullblock-agents
"""

import asyncio
import logging
import sys
import os
from datetime import datetime
from typing import Dict, Any, Optional
from dataclasses import dataclass

# Add the agent packages to the path
sys.path.insert(0, 'svc/nullblock-agents/src')

from agents.information_gathering.main import InformationGatheringAgent, DataRequest
from agents.llm_service.factory import LLMServiceFactory, LLMRequest
from agents.llm_service.router import TaskRequirements, OptimizationGoal, Priority
from agents.llm_service.models import ModelCapability

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)

logger = logging.getLogger(__name__)

class ServiceHealthError(Exception):
    """Raised when a critical service is unhealthy"""
    pass

class DemoError(Exception):
    """Raised when demo encounters a critical error"""
    pass

@dataclass
class ServiceStatus:
    """Standardized service status response"""
    service_name: str
    healthy: bool
    error_message: Optional[str] = None
    details: Optional[Dict[str, Any]] = None

class NullblockIntegrationDemo:
    """Complete integration demo for Nullblock agent infrastructure"""
    
    def __init__(self):
        self.info_agent = None
        self.llm_factory = None
        
    async def check_prerequisites(self) -> Dict[str, ServiceStatus]:
        """Check all prerequisites and return status"""
        print("🔍 Checking Prerequisites...")
        print("-" * 40)
        
        status = {}
        
        # Check MCP Server
        print("1️⃣ Checking MCP Server...")
        try:
            import aiohttp
            async with aiohttp.ClientSession() as session:
                async with session.get("http://localhost:8001/health", timeout=5) as resp:
                    if resp.status == 200:
                        health_data = await resp.json()
                        status["mcp_server"] = ServiceStatus(
                            service_name="MCP Server",
                            healthy=True,
                            details=health_data
                        )
                        print("   ✅ MCP Server is running")
                    else:
                        status["mcp_server"] = ServiceStatus(
                            service_name="MCP Server",
                            healthy=False,
                            error_message=f"HTTP {resp.status}"
                        )
                        print(f"   ❌ MCP Server returned HTTP {resp.status}")
        except Exception as e:
            status["mcp_server"] = ServiceStatus(
                service_name="MCP Server",
                healthy=False,
                error_message=str(e)
            )
            print(f"   ❌ MCP Server is not accessible: {e}")
        
        # Check API Keys
        print("\n2️⃣ Checking API Keys...")
        api_keys = {
            "OPENAI_API_KEY": os.getenv('OPENAI_API_KEY'),
            "ANTHROPIC_API_KEY": os.getenv('ANTHROPIC_API_KEY'),
            "GROQ_API_KEY": os.getenv('GROQ_API_KEY'),
            "HUGGINGFACE_API_KEY": os.getenv('HUGGINGFACE_API_KEY')
        }
        
        available_keys = [name for name, key in api_keys.items() if key]
        if available_keys:
            status["api_keys"] = ServiceStatus(
                service_name="API Keys",
                healthy=True,
                details={"available": available_keys}
            )
            print(f"   ✅ Available API keys: {', '.join(available_keys)}")
        else:
            status["api_keys"] = ServiceStatus(
                service_name="API Keys",
                healthy=False,
                error_message="No API keys found"
            )
            print("   ⚠️  No API keys found (will use local models only)")
        
        # Check Network Connectivity
        print("\n3️⃣ Checking Network Connectivity...")
        try:
            import aiohttp
            async with aiohttp.ClientSession() as session:
                async with session.get("https://httpbin.org/get", timeout=5) as resp:
                    if resp.status == 200:
                        status["network"] = ServiceStatus(
                            service_name="Network",
                            healthy=True
                        )
                        print("   ✅ Network connectivity is available")
                    else:
                        status["network"] = ServiceStatus(
                            service_name="Network",
                            healthy=False,
                            error_message=f"HTTP {resp.status}"
                        )
                        print(f"   ❌ Network connectivity issues: HTTP {resp.status}")
        except Exception as e:
            status["network"] = ServiceStatus(
                service_name="Network",
                healthy=False,
                error_message=str(e)
            )
            print(f"   ❌ Network connectivity failed: {e}")
        
        print("\n" + "=" * 40)
        return status
    
    async def initialize(self):
        """Initialize all components with proper error handling"""
        print("🚀 Initializing Nullblock Integration Demo")
        print("=" * 60)
        
        # Check prerequisites first
        status = await self.check_prerequisites()
        
        # Fail fast if MCP server is not available
        if not status.get("mcp_server", ServiceStatus("", False)).healthy:
            raise ServiceHealthError(
                f"MCP Server is not available: {status['mcp_server'].error_message}. "
                "Please start the MCP server with: cd svc/nullblock-mcp && MCP_SERVER_HOST=0.0.0.0 MCP_SERVER_PORT=8001 python -m mcp.server"
            )
        
        # Initialize Information Gathering Agent
        print("📊 Initializing Information Gathering Agent...")
        try:
            self.info_agent = InformationGatheringAgent(mcp_server_url="http://localhost:8001")
            await self.info_agent.mcp_client.connect()
            
            # Verify MCP connection and data sources
            health = await self.info_agent.mcp_client.health_check()
            if health.get('status') != 'healthy':
                raise ServiceHealthError(f"MCP server health check failed: {health}")
            
            sources = await self.info_agent.mcp_client.get_available_sources()
            if not sources:
                raise ServiceHealthError("No data sources available from MCP server")
            
            print("✅ Information Gathering Agent connected to MCP server")
            print(f"   📋 Available Data Sources: {len(sources)} types")
            for source_type, source_names in sources.items():
                print(f"      {source_type}: {', '.join(source_names)}")
                
        except Exception as e:
            raise DemoError(f"Failed to initialize Information Gathering Agent: {e}")
        
        # Initialize LLM Service Factory
        print("\n🤖 Initializing LLM Service Factory...")
        try:
            self.llm_factory = LLMServiceFactory()
            await self.llm_factory.initialize()
            
            # Check if any models are available
            models_available = await self.llm_factory.check_model_availability()
            if not models_available:
                raise ServiceHealthError(
                    "No LLM models are available. Please check your API keys and network connectivity."
                )
            
            # Test LLM connectivity with a simple request
            test_request = LLMRequest(
                prompt="Hello",
                max_tokens=10
            )
            
            # Try to get a response (this will test connectivity)
            try:
                await self.llm_factory.generate(test_request)
                print("✅ LLM Service Factory initialized and tested")
            except Exception as e:
                if "Cannot connect to host localhost:11434" in str(e):
                    # Ollama connection failed, but LM Studio might be available
                    print("   ⚠️  Ollama not available, trying LM Studio...")
                    # Test LM Studio specifically
                    try:
                        import aiohttp
                        async with aiohttp.ClientSession() as session:
                            async with session.get("http://localhost:1234/v1/models", timeout=3) as resp:
                                if resp.status == 200:
                                    print("   ✅ LM Studio is available")
                                    # Force use of LM Studio models
                                    test_request.model_override = "gemma-3-270m-it-mlx"
                                    await self.llm_factory.generate(test_request)
                                    print("✅ LLM Service Factory initialized with LM Studio")
                                else:
                                    raise ServiceHealthError(
                                        f"LM Studio returned HTTP {resp.status}. "
                                        "Please ensure LM Studio is running and serving on localhost:1234."
                                    )
                    except Exception as lm_e:
                        raise ServiceHealthError(
                            f"LM Studio is not accessible: {lm_e}. "
                            "Please start LM Studio and load a model (serves on localhost:1234)."
                        )
                elif "Could not contact DNS servers" in str(e) or "Connect call failed" in str(e):
                    raise ServiceHealthError(
                        f"LLM services are not accessible: {e}. "
                        "Please check your network connection and API keys."
                    )
                else:
                    raise DemoError(f"LLM connectivity test failed: {e}")
                    
        except Exception as e:
            if isinstance(e, ServiceHealthError):
                raise
            raise DemoError(f"Failed to initialize LLM Service Factory: {e}")
        
        print("\n🎯 Demo ready! All components initialized successfully.\n")
    
    async def demo_market_intelligence(self):
        """Demo: Market intelligence analysis with LLM insights"""
        print("📈 DEMO 1: Market Intelligence Analysis")
        print("-" * 40)
        
        # Step 1: Gather market data
        print("1️⃣ Gathering market data from CoinGecko API...")
        
        symbols = ["bitcoin", "ethereum", "solana"]
        market_data = {}
        errors = []
        
        print(f"   🌐 API Endpoint: https://api.coingecko.com/api/v3/simple/price")
        print(f"   📡 Fetching real-time data for: {', '.join(symbols)}")
        
        for symbol in symbols:
            try:
                print(f"   🔄 Requesting {symbol} data from CoinGecko...")
                data = await self.info_agent.get_real_time_data(
                    source_type="price_oracle",
                    source_name="coingecko",
                    parameters={"symbols": [symbol], "vs_currency": "usd"}
                )
                
                # Check if we got meaningful data
                if not data or (isinstance(data, dict) and not data.get('data')):
                    raise ValueError(f"No data returned for {symbol}")
                
                # Show actual API response
                print(f"   📊 CoinGecko API Response for {symbol}:")
                print(f"      📅 Timestamp: {data.get('timestamp', 'N/A')}")
                print(f"      🎯 Source: {data.get('source', 'N/A')}")
                print(f"      ✅ Reliability: {data.get('reliability_score', 'N/A')}")
                
                # Extract and display price data
                if 'data' in data:
                    # Handle different data formats
                    if isinstance(data['data'], list) and data['data']:
                        # DataPoint format from MCP
                        data_point = data['data'][0]
                        if hasattr(data_point, 'value'):
                            price = data_point.value
                            print(f"      💰 Price: ${price:,.2f}")
                            
                            # Check for metadata
                            if hasattr(data_point, 'metadata') and data_point.metadata:
                                change_24h = data_point.metadata.get('change_24h', 0)
                                volume_24h = data_point.metadata.get('volume_24h', 0)
                                emoji = "🟢" if change_24h >= 0 else "🔴"
                                print(f"      {emoji} 24h Change: {change_24h:+.2f}%")
                                if volume_24h > 0:
                                    print(f"      📊 24h Volume: ${volume_24h:,.0f}")
                    
                    elif isinstance(data['data'], dict) and symbol in data['data']:
                        # Direct API response format
                        price_info = data['data'][symbol]
                        if isinstance(price_info, dict):
                            if 'price' in price_info:
                                print(f"      💰 Price: ${price_info['price']:,.2f}")
                            if 'change_24h' in price_info:
                                change = price_info['change_24h']
                                emoji = "🟢" if change >= 0 else "🔴"
                                print(f"      {emoji} 24h Change: {change:+.2f}%")
                        else:
                            print(f"      💰 Price: ${price_info:,.2f}")
                    
                    else:
                        print(f"      💰 Raw API Data: {data['data']}")
                else:
                    print(f"      ⚠️  No price data found in response")
                    
                market_data[symbol] = data
                print(f"   ✅ {symbol}: Real market data successfully retrieved")
                
            except Exception as e:
                # Fail immediately if any data source fails
                raise DemoError(f"Failed to get data for {symbol}: {e}")
        
        # Step 2: Analyze patterns
        print("\n2️⃣ Analyzing market patterns...")
        
        try:
            analysis_result = await self.info_agent.analyze_market_trends(
                symbols, timeframe="24h"
            )
            
            print(f"   📊 Analysis completed with {analysis_result.confidence_score:.1%} confidence")
            print(f"   🔍 Found {len(analysis_result.patterns_detected)} patterns")
            print(f"   ⚠️  Detected {len(analysis_result.anomalies)} anomalies")
            
        except Exception as e:
            raise DemoError(f"Pattern analysis failed: {e}")
        
        # Step 3: Generate intelligent insights with LLM
        print("\n3️⃣ Generating intelligent insights...")
        
        if analysis_result:
            # Prepare context for LLM
            context = f"""
Market Analysis Results for {', '.join(symbols)}:

Insights:
{chr(10).join(f'• {insight}' for insight in analysis_result.insights)}

Patterns Detected:
{chr(10).join(f'• {pattern}' for pattern in analysis_result.patterns_detected)}

Anomalies:
{chr(10).join(f'• {anomaly}' for anomaly in analysis_result.anomalies)}

Recommendations:
{chr(10).join(f'• {rec}' for rec in analysis_result.recommendations)}
"""
            
            # Request LLM analysis
            llm_request = LLMRequest(
                prompt=f"Based on this market analysis data, provide a concise summary of the current market conditions and 3 specific actionable insights for traders:\n\n{context}",
                system_prompt="You are a professional cryptocurrency market analyst. Provide clear, actionable insights based on data.",
                max_tokens=500
            )
            
            # Use LLM factory with requirements for data analysis
            requirements = TaskRequirements(
                required_capabilities=[ModelCapability.DATA_ANALYSIS, ModelCapability.REASONING],
                optimization_goal=OptimizationGoal.QUALITY,
                priority=Priority.HIGH,
                task_type="market_analysis",
                max_latency_ms=5000
            )
            
            try:
                print(f"   📤 LLM Input Prompt:")
                print(f"   {llm_request.prompt[:150]}...")
                
                # Log the full prompt for debugging
                logger.info(f"LLM Request - Prompt: {llm_request.prompt}")
                logger.info(f"LLM Request - System: {llm_request.system_prompt}")
                
                llm_response = await self.llm_factory.generate(llm_request, requirements)
                
                # Log the full response for debugging
                logger.info(f"LLM Response - Content: {llm_response.content}")
                logger.info(f"LLM Response - Model: {llm_response.model_used}")
                logger.info(f"LLM Response - Latency: {llm_response.latency_ms}ms")
                
                # Validate response is not empty
                if not llm_response.content or len(llm_response.content.strip()) == 0:
                    raise DemoError("LLM returned empty response - model connectivity issue")
                
                print(f"   🤖 LLM Analysis ({llm_response.model_used}):")
                print(f"   💰 Cost: ${llm_response.cost_estimate:.4f}")
                print(f"   ⏱️  Latency: {llm_response.latency_ms:.0f}ms")
                print(f"   📏 Response Length: {len(llm_response.content)} characters")
                print(f"\n   📝 Market Intelligence Report:")
                print("   " + "\n   ".join(llm_response.content.split("\n")))
                print(f"   🎯 Response validation: PASSED (non-empty)")
                
            except Exception as e:
                if "empty response" in str(e):
                    print(f"   ❌ CRITICAL: {e}")
                raise DemoError(f"LLM analysis failed: {e}")
        
        print("\n✅ Market Intelligence Demo completed!\n")
    
    async def demo_automated_research(self):
        """Demo: Automated research pipeline"""
        print("🔬 DEMO 2: Automated Research Pipeline")
        print("-" * 40)
        
        # Step 1: Define research question
        research_question = "What are the current trends in DeFi liquidity and yield farming opportunities?"
        print(f"❓ Research Question: {research_question}")
        
        # Step 2: Gather DeFi data
        print("\n1️⃣ Gathering DeFi protocol data...")
        
        protocols = ["uniswap"]  # Limited to available sources
        defi_data = {}
        errors = []
        
        for protocol in protocols:
            try:
                data = await self.info_agent.get_real_time_data(
                    source_type="defi_protocol",
                    source_name=protocol,
                    parameters={"metrics": ["tvl", "volume"], "timeframe": "7d"}
                )
                
                # Check if we got meaningful data
                if not data or (isinstance(data, dict) and not data.get('data')):
                    raise ValueError(f"No data returned for {protocol}")
                    
                defi_data[protocol] = data
                print(f"   ✅ {protocol}: Data retrieved successfully")
                
            except Exception as e:
                # Fail immediately if any data source fails
                raise DemoError(f"Failed to get data for {protocol}: {e}")
        
        # Step 3: Analyze opportunities
        print("\n2️⃣ Analyzing DeFi opportunities...")
        
        try:
            opportunities = await self.info_agent.detect_defi_opportunities(protocols)
            
            print(f"   📊 Opportunity analysis completed")
            print(f"   💡 Found {len(opportunities.insights)} insights")
            print(f"   🎯 Generated {len(opportunities.recommendations)} recommendations")
            
            # Step 4: Generate research report
            print("\n3️⃣ Generating comprehensive research report...")
            
            research_context = f"""
DeFi Research Analysis:

Question: {research_question}

Protocol Data: {defi_data}

Key Insights:
{chr(10).join(f'• {insight}' for insight in opportunities.insights)}

Detected Patterns:
{chr(10).join(f'• {pattern}' for pattern in opportunities.patterns_detected)}

Recommendations:
{chr(10).join(f'• {rec}' for rec in opportunities.recommendations)}
"""
            
            # Use premium model for research
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
            
            try:
                print(f"   📤 LLM Input Prompt:")
                print(f"   {llm_request.prompt[:150]}...")
                
                report = await self.llm_factory.generate(llm_request, requirements)
                
                # Validate response is not empty
                if not report.content or len(report.content.strip()) == 0:
                    raise DemoError("LLM returned empty response - model connectivity issue")
                
                print(f"   📋 Research Report Generated ({report.model_used}):")
                print(f"   💰 Cost: ${report.cost_estimate:.4f}")
                print(f"   ⏱️  Latency: {report.latency_ms:.0f}ms")
                print(f"   📏 Response Length: {len(report.content)} characters")
                print(f"\n   📄 DeFi Research Report:")
                print("   " + "\n   ".join(report.content.split("\n")))
                print(f"   🎯 Response validation: PASSED (non-empty)")
                
            except Exception as e:
                if "empty response" in str(e):
                    print(f"   ❌ CRITICAL: {e}")
                    raise DemoError(f"Research report generation failed: {e}")
                raise DemoError(f"Report generation failed: {e}")
                
        except Exception as e:
            raise DemoError(f"DeFi analysis failed: {e}")
        
        print("\n✅ Automated Research Demo completed!\n")
    
    async def demo_multi_model_comparison(self):
        """Demo: Multi-model comparison for different tasks"""
        print("⚖️  DEMO 3: Multi-Model Comparison")
        print("-" * 40)
        
        test_prompt = "Explain the key differences between Proof of Work and Proof of Stake consensus mechanisms in 2-3 sentences."
        
        # Test different optimization goals
        test_scenarios = [
            ("Speed Optimized", OptimizationGoal.SPEED, Priority.LOW),
            ("Quality Optimized", OptimizationGoal.QUALITY, Priority.HIGH),
            ("Cost Optimized", OptimizationGoal.COST, Priority.LOW),
            ("Balanced", OptimizationGoal.BALANCED, Priority.MEDIUM)
        ]
        
        print(f"📝 Test Prompt: {test_prompt}\n")
        
        successful_tests = 0
        total_tests = len(test_scenarios)
        
        for scenario_name, goal, priority in test_scenarios:
            print(f"🎯 Testing: {scenario_name}")
            
            llm_request = LLMRequest(
                prompt=test_prompt,
                max_tokens=150
            )
            
            requirements = TaskRequirements(
                required_capabilities=[ModelCapability.REASONING],
                optimization_goal=goal,
                priority=priority,
                task_type="explanation"
            )
            
            try:
                response = await self.llm_factory.generate(llm_request, requirements)
                
                print(f"   🤖 Model: {response.model_used}")
                print(f"   💰 Cost: ${response.cost_estimate:.4f}")
                print(f"   ⏱️  Latency: {response.latency_ms:.0f}ms")
                print(f"   📝 Response: {response.content[:100]}{'...' if len(response.content) > 100 else ''}")
                print()
                successful_tests += 1
                
            except Exception as e:
                print(f"   ❌ Failed: {e}\n")
        
        if successful_tests == 0:
            raise DemoError("All LLM model tests failed - no working models available")
        elif successful_tests < total_tests:
            print(f"⚠️  {successful_tests}/{total_tests} model tests succeeded")
        
        print("✅ Multi-Model Comparison completed!\n")
    
    async def demo_system_stats(self):
        """Demo: System statistics and monitoring"""
        print("📊 DEMO 4: System Statistics")
        print("-" * 40)
        
        # LLM Factory stats
        llm_stats = self.llm_factory.get_stats()
        print("🤖 LLM Service Factory Statistics:")
        print(f"   Request Stats: {llm_stats['request_stats']}")
        print(f"   Cost Tracking: ${sum(llm_stats['cost_tracking'].values()):.4f} total")
        print(f"   Cache Size: {llm_stats['cache_stats']['cache_size']} entries")
        
        # MCP connection stats
        mcp_status = self.info_agent.mcp_client.get_connection_status()
        print(f"\n📡 MCP Connection Statistics:")
        for key, value in mcp_status.items():
            print(f"   {key}: {value}")
        
        # Agent stats
        print(f"\n🔍 Information Gathering Agent Statistics:")
        print(f"   Active Requests: {len(self.info_agent.active_requests)}")
        print(f"   Cached Results: {len(self.info_agent.analysis_cache)}")
        print(f"   Running: {self.info_agent.running}")
        
        print("\n✅ System Statistics retrieved!\n")
    
    async def cleanup(self):
        """Clean up resources"""
        print("🧹 Cleaning up resources...")
        
        if self.info_agent:
            await self.info_agent.mcp_client.disconnect()
            print("   ✅ Information Gathering Agent disconnected")
        
        if self.llm_factory:
            await self.llm_factory.cleanup()
            print("   ✅ LLM Service Factory cleaned up")
        
        print("✅ Cleanup completed!\n")

async def main():
    """Run the complete integration demo"""
    print("🎬 NULLBLOCK INTEGRATION DEMO")
    print("=" * 60)
    print("This demo showcases the complete Nullblock agent infrastructure:")
    print("• Information Gathering Agent with MCP data sources")
    print("• LLM Service Factory with intelligent model routing")
    print("• Pattern detection and data analysis")
    print("• Multi-agent coordination and orchestration")
    print("=" * 60)
    
    demo = NullblockIntegrationDemo()
    
    try:
        # Initialize
        await demo.initialize()
        
        # Run demos
        await demo.demo_market_intelligence()
        await demo.demo_automated_research()
        await demo.demo_multi_model_comparison()
        await demo.demo_system_stats()
        
        print("🎉 ALL DEMOS COMPLETED SUCCESSFULLY!")
        print("=" * 60)
        print("The Nullblock agent infrastructure is fully operational and ready for production use.")
        
    except ServiceHealthError as e:
        print(f"\n❌ SERVICE HEALTH ERROR: {e}")
        print("=" * 60)
        print("Please ensure all required services are running:")
        print("1. MCP Server: cd svc/nullblock-mcp && MCP_SERVER_HOST=0.0.0.0 MCP_SERVER_PORT=8001 python -m mcp.server")
        print("2. LM Studio: Start LM Studio and load a model (serves on localhost:1234)")
        print("3. Network connectivity and API keys")
        sys.exit(1)
    except DemoError as e:
        print(f"\n❌ DEMO ERROR: {e}")
        print("=" * 60)
        print("The demo encountered a critical error and cannot continue.")
        sys.exit(1)
    except KeyboardInterrupt:
        print("\n🛑 Demo interrupted by user")
    except Exception as e:
        logger.error(f"Demo failed: {e}")
        print(f"\n❌ UNEXPECTED ERROR: {e}")
        print("=" * 60)
        print("An unexpected error occurred. Please check the logs for details.")
        sys.exit(1)
    finally:
        await demo.cleanup()

if __name__ == "__main__":
    print("📋 Prerequisites Check:")
    print("1. MCP Server: Start with 'cd svc/nullblock-mcp && MCP_SERVER_HOST=0.0.0.0 MCP_SERVER_PORT=8001 python -m mcp.server'")
    print("2. LM Studio: Start LM Studio and load a model (serves on localhost:1234)")
    print("3. API Keys: Ensure OPENAI_API_KEY, ANTHROPIC_API_KEY are set (optional)")
    print("4. Dependencies: Run 'pip install -e svc/nullblock-agents'")
    print("\nPress Enter to continue with demo...")
    input()  # Wait for user input
    
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\n🛑 Demo interrupted")
    except Exception as e:
        print(f"❌ Demo startup failed: {e}")
        sys.exit(1)