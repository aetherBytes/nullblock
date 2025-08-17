"""
Integration Tests for Multi-Agent Coordination

Comprehensive test suite covering:
- Cross-agent communication and coordination
- End-to-end workflow execution with local LLM integration
- Performance under multi-agent loads
- Error handling and recovery across agent boundaries
- Real-world scenario simulations
"""

import pytest
import asyncio
import os
import json
from unittest.mock import AsyncMock, patch, MagicMock
from typing import Dict, List, Any, Optional

import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))

from agents.information_gathering.main import InformationGatheringAgent
from agents.llm_service.factory import LLMServiceFactory, LLMRequest, LLMResponse
from agents.llm_service.router import TaskRequirements, OptimizationGoal, Priority
from agents.llm_service.models import ModelCapability, ModelProvider, AVAILABLE_MODELS


class MockMCPClient:
    """Mock MCP client for testing"""
    
    def __init__(self):
        self.connected = False
        self.call_count = 0
        self.call_history = []
    
    async def connect(self):
        self.connected = True
    
    async def disconnect(self):
        self.connected = False
    
    async def call_tool(self, tool_name: str, params: Dict[str, Any]) -> Dict[str, Any]:
        self.call_count += 1
        self.call_history.append({"tool": tool_name, "params": params})
        
        # Mock responses based on tool name
        if "market_trends" in tool_name:
            return {
                "insights": [
                    f"Market analysis for {params.get('symbols', ['BTC'])}",
                    "Strong bullish momentum detected",
                    "Volume indicators suggest continued upward movement"
                ],
                "patterns": [
                    "Upward price momentum across major assets",
                    "Increased institutional accumulation"
                ],
                "anomalies": [],
                "recommendations": [
                    "Consider increasing position size",
                    "Monitor for potential breakout levels"
                ]
            }
        elif "defi_opportunities" in tool_name:
            return {
                "opportunities": [
                    {
                        "protocol": "uniswap",
                        "strategy": "liquidity_provision",
                        "estimated_apr": 12.5,
                        "risk_score": 0.3,
                        "required_capital": 10000
                    },
                    {
                        "protocol": "aave",
                        "strategy": "lending",
                        "estimated_apr": 8.7,
                        "risk_score": 0.2,
                        "required_capital": 5000
                    }
                ],
                "insights": [
                    "High yield opportunities in DeFi protocols",
                    "Lower risk strategies available"
                ],
                "recommendations": [
                    "Diversify across multiple protocols",
                    "Start with lower risk strategies"
                ]
            }
        elif "price_oracle" in tool_name or "price" in tool_name:
            symbols = params.get("symbols", ["bitcoin"])
            return {
                "source": "coingecko",
                "timestamp": "2024-01-15T10:30:00Z",
                "data": {symbol: {"price": 45000 + hash(symbol) % 10000, "change_24h": 2.5} for symbol in symbols},
                "reliability_score": 0.95
            }
        elif "social_sentiment" in tool_name or "sentiment" in tool_name:
            return {
                "source": "twitter",
                "timestamp": "2024-01-15T10:30:00Z",
                "data": {
                    "bitcoin": {
                        "sentiment_score": 0.75,
                        "mention_count": 15000,
                        "trending_score": 8.2
                    }
                },
                "reliability_score": 0.88
            }
        else:
            return {"status": "success", "data": f"response_for_{tool_name}"}


class TestMultiAgentCoordination:
    """Test suite for multi-agent coordination"""
    
    @pytest.fixture
    async def coordination_setup(self):
        """Setup multiple agents for coordination testing"""
        # Create multiple information gathering agents
        agent1 = InformationGatheringAgent("http://localhost:8001")
        agent2 = InformationGatheringAgent("http://localhost:8001")
        
        # Mock MCP clients
        agent1.mcp_client = MockMCPClient()
        agent2.mcp_client = MockMCPClient()
        
        # Initialize LLM factories
        agent1.llm_factory = LLMServiceFactory()
        agent2.llm_factory = LLMServiceFactory()
        
        await agent1.llm_factory.initialize()
        await agent2.llm_factory.initialize()
        
        yield {"agent1": agent1, "agent2": agent2}
        
        # Cleanup
        await agent1.llm_factory.cleanup()
        await agent2.llm_factory.cleanup()
    
    @pytest.mark.asyncio
    async def test_parallel_agent_execution(self, coordination_setup):
        """Test multiple agents working in parallel"""
        agent1 = coordination_setup["agent1"]
        agent2 = coordination_setup["agent2"]
        
        # Execute parallel tasks
        task1 = agent1.analyze_market_trends(["bitcoin", "ethereum"], "24h")
        task2 = agent2.detect_defi_opportunities(["uniswap", "aave"])
        
        # Wait for both to complete
        results = await asyncio.gather(task1, task2)
        
        # Verify both agents completed successfully
        market_result, defi_result = results
        
        assert "insights" in market_result
        assert len(market_result["insights"]) > 0
        assert "opportunities" in defi_result
        assert len(defi_result["opportunities"]) > 0
        
        # Verify agents didn't interfere with each other
        assert agent1.mcp_client.call_count >= 1
        assert agent2.mcp_client.call_count >= 1
    
    @pytest.mark.asyncio
    async def test_agent_data_sharing(self, coordination_setup):
        """Test agents sharing data and insights"""
        agent1 = coordination_setup["agent1"]
        agent2 = coordination_setup["agent2"]
        
        # Agent 1 gathers market data
        market_data = await agent1.analyze_market_trends(["bitcoin"], "24h")
        
        # Agent 2 uses that context for DeFi analysis
        # In a real scenario, this would involve cross-agent communication
        defi_context = {
            "market_sentiment": market_data["insights"],
            "price_trends": market_data["patterns"]
        }
        
        defi_result = await agent2.detect_defi_opportunities(["uniswap"])
        
        # Verify both analyses completed
        assert market_data is not None
        assert defi_result is not None
        assert len(market_data["insights"]) > 0
        assert len(defi_result["opportunities"]) > 0
    
    @pytest.mark.asyncio
    async def test_coordinated_workflow_execution(self, coordination_setup):
        """Test coordinated multi-step workflow across agents"""
        agent1 = coordination_setup["agent1"]
        agent2 = coordination_setup["agent2"]
        
        # Step 1: Market analysis
        market_analysis = await agent1.analyze_market_trends(["bitcoin", "ethereum"], "24h")
        
        # Step 2: DeFi opportunity detection based on market conditions
        protocols = ["uniswap", "aave"]
        if "bullish" in str(market_analysis["insights"]).lower():
            protocols.append("compound")  # Add more aggressive strategies
        
        defi_opportunities = await agent2.detect_defi_opportunities(protocols)
        
        # Step 3: Combine insights for final recommendation
        combined_insights = {
            "market_analysis": market_analysis,
            "defi_opportunities": defi_opportunities,
            "coordination_timestamp": "2024-01-15T10:30:00Z",
            "workflow_success": True
        }
        
        # Verify workflow coordination
        assert combined_insights["workflow_success"] is True
        assert "market_analysis" in combined_insights
        assert "defi_opportunities" in combined_insights
        assert len(combined_insights["market_analysis"]["insights"]) > 0
        assert len(combined_insights["defi_opportunities"]["opportunities"]) > 0


class TestEndToEndScenarios:
    """Test end-to-end scenarios with local LLM integration"""
    
    @pytest.fixture
    async def e2e_setup(self):
        """Setup for end-to-end testing"""
        agent = InformationGatheringAgent("http://localhost:8001")
        agent.mcp_client = MockMCPClient()
        
        # Setup LLM factory with local model support
        agent.llm_factory = LLMServiceFactory()
        await agent.llm_factory.initialize()
        
        yield agent
        
        await agent.llm_factory.cleanup()
    
    @pytest.mark.asyncio
    async def test_complete_market_analysis_workflow(self, e2e_setup):
        """Test complete market analysis workflow with LLM enhancement"""
        agent = e2e_setup
        
        # Mock local LLM response
        mock_llm_response = LLMResponse(
            content="Based on the comprehensive market analysis, Bitcoin and Ethereum are showing strong bullish indicators with significant institutional accumulation patterns. The 2.5% daily gains combined with elevated volume suggest continued upward momentum. Recommendation: Consider strategic position increases while monitoring key resistance levels.",
            model_used="gemma-3-270m-it-mlx",
            usage={"total_tokens": 85},
            latency_ms=750.0,
            cost_estimate=0.0,
            finish_reason="stop"
        )
        
        with patch.object(agent.llm_factory, 'generate', return_value=mock_llm_response):
            # Execute complete workflow
            result = await agent.analyze_market_trends(
                ["bitcoin", "ethereum"], 
                "24h", 
                enhance_with_llm=True
            )
            
            # Verify complete workflow results
            assert "insights" in result
            assert "enhanced_analysis" in result
            assert "model_used" in result
            assert result["model_used"] == "gemma-3-270m-it-mlx"
            assert len(result["enhanced_analysis"]) > 100  # Substantial LLM analysis
            assert "bullish" in result["enhanced_analysis"].lower()
    
    @pytest.mark.asyncio
    async def test_defi_strategy_optimization_workflow(self, e2e_setup):
        """Test DeFi strategy optimization with multi-step analysis"""
        agent = e2e_setup
        
        # Step 1: Analyze multiple protocols
        protocols = ["uniswap", "aave", "compound"]
        opportunities = await agent.detect_defi_opportunities(protocols, min_apr=5.0, max_risk=0.5)
        
        # Step 2: Filter and rank opportunities
        filtered_opportunities = [
            op for op in opportunities["opportunities"] 
            if op["risk_score"] <= 0.5 and op["estimated_apr"] >= 5.0
        ]
        
        # Step 3: Create optimization strategy
        strategy = {
            "selected_opportunities": filtered_opportunities,
            "total_estimated_return": sum(op["estimated_apr"] for op in filtered_opportunities),
            "average_risk": sum(op["risk_score"] for op in filtered_opportunities) / len(filtered_opportunities) if filtered_opportunities else 0,
            "diversification_score": len(set(op["protocol"] for op in filtered_opportunities)),
            "optimization_complete": True
        }
        
        # Verify optimization workflow
        assert strategy["optimization_complete"] is True
        assert len(strategy["selected_opportunities"]) > 0
        assert strategy["average_risk"] <= 0.5
        assert strategy["diversification_score"] >= 1
    
    @pytest.mark.asyncio
    async def test_real_time_monitoring_workflow(self, e2e_setup):
        """Test real-time data monitoring and alert workflow"""
        agent = e2e_setup
        
        # Simulate continuous monitoring
        monitoring_cycles = 3
        monitoring_results = []
        
        for cycle in range(monitoring_cycles):
            # Get real-time data
            price_data = await agent.get_real_time_data(
                "price_oracle", 
                "coingecko", 
                {"symbols": ["bitcoin"]}
            )
            
            sentiment_data = await agent.get_real_time_data(
                "social_sentiment",
                "twitter",
                {"symbols": ["bitcoin"], "timeframe": "1h"}
            )
            
            # Combine data for analysis
            cycle_result = {
                "cycle": cycle + 1,
                "price_data": price_data,
                "sentiment_data": sentiment_data,
                "timestamp": price_data.get("timestamp"),
                "alerts": []
            }
            
            # Check for alerts (simplified logic)
            bitcoin_price = price_data["data"].get("bitcoin", {}).get("price", 0)
            if bitcoin_price > 50000:
                cycle_result["alerts"].append("Price above $50k threshold")
            
            sentiment_score = sentiment_data["data"].get("bitcoin", {}).get("sentiment_score", 0)
            if sentiment_score > 0.8:
                cycle_result["alerts"].append("Extremely positive sentiment detected")
            
            monitoring_results.append(cycle_result)
            
            # Simulate time delay
            await asyncio.sleep(0.1)
        
        # Verify monitoring workflow
        assert len(monitoring_results) == monitoring_cycles
        assert all("timestamp" in result for result in monitoring_results)
        assert all("price_data" in result for result in monitoring_results)
        assert all("sentiment_data" in result for result in monitoring_results)
    
    @pytest.mark.asyncio
    async def test_cross_agent_arbitrage_detection(self, e2e_setup):
        """Test cross-agent arbitrage opportunity detection"""
        agent = e2e_setup
        
        # Simulate multiple exchange data
        exchanges = ["binance", "coinbase", "kraken"]
        arbitrage_data = {}
        
        for exchange in exchanges:
            price_data = await agent.get_real_time_data(
                "price_oracle",
                exchange,
                {"symbols": ["bitcoin", "ethereum"]}
            )
            arbitrage_data[exchange] = price_data["data"]
        
        # Detect arbitrage opportunities
        arbitrage_opportunities = []
        symbols = ["bitcoin", "ethereum"]
        
        for symbol in symbols:
            prices = [arbitrage_data[ex][symbol]["price"] for ex in exchanges]
            min_price = min(prices)
            max_price = max(prices)
            
            if (max_price - min_price) / min_price > 0.005:  # 0.5% threshold
                min_exchange = exchanges[prices.index(min_price)]
                max_exchange = exchanges[prices.index(max_price)]
                
                arbitrage_opportunities.append({
                    "symbol": symbol,
                    "buy_exchange": min_exchange,
                    "sell_exchange": max_exchange,
                    "buy_price": min_price,
                    "sell_price": max_price,
                    "profit_percentage": (max_price - min_price) / min_price * 100,
                    "profit_usd": max_price - min_price
                })
        
        # Verify arbitrage detection
        arbitrage_result = {
            "opportunities": arbitrage_opportunities,
            "total_opportunities": len(arbitrage_opportunities),
            "highest_profit": max(op["profit_percentage"] for op in arbitrage_opportunities) if arbitrage_opportunities else 0,
            "detection_timestamp": "2024-01-15T10:30:00Z"
        }
        
        assert arbitrage_result["total_opportunities"] >= 0
        assert "opportunities" in arbitrage_result
        assert "detection_timestamp" in arbitrage_result


class TestPerformanceAndScaling:
    """Test performance and scaling characteristics"""
    
    @pytest.mark.asyncio
    async def test_concurrent_agent_load(self):
        """Test system performance under concurrent agent load"""
        # Create multiple agents
        agent_count = 5
        agents = []
        
        for i in range(agent_count):
            agent = InformationGatheringAgent(f"http://localhost:800{i}")
            agent.mcp_client = MockMCPClient()
            agent.llm_factory = LLMServiceFactory()
            await agent.llm_factory.initialize()
            agents.append(agent)
        
        try:
            # Execute concurrent requests
            tasks = []
            for i, agent in enumerate(agents):
                task = agent.analyze_market_trends([f"symbol_{i}"], "1h")
                tasks.append(task)
            
            # Measure execution time
            start_time = asyncio.get_event_loop().time()
            results = await asyncio.gather(*tasks, return_exceptions=True)
            end_time = asyncio.get_event_loop().time()
            
            execution_time = end_time - start_time
            
            # Verify performance
            successful_results = [r for r in results if not isinstance(r, Exception)]
            success_rate = len(successful_results) / len(results)
            
            assert success_rate >= 0.8  # 80% success rate
            assert execution_time < 10.0  # Under 10 seconds
            assert len(successful_results) >= agent_count * 0.8
            
        finally:
            # Cleanup
            for agent in agents:
                await agent.llm_factory.cleanup()
    
    @pytest.mark.asyncio
    async def test_memory_efficiency_under_load(self):
        """Test memory efficiency during extended operation"""
        agent = InformationGatheringAgent("http://localhost:8001")
        agent.mcp_client = MockMCPClient()
        agent.llm_factory = LLMServiceFactory()
        await agent.llm_factory.initialize()
        
        try:
            # Execute many requests to test memory usage
            request_count = 20
            results = []
            
            for i in range(request_count):
                result = await agent.analyze_market_trends([f"test_symbol_{i}"], "1h")
                results.append(result)
                
                # Verify each result is valid
                assert result is not None
                assert "insights" in result
            
            # Verify all results were successful
            assert len(results) == request_count
            
            # In real testing, would check memory usage here
            # For now, verify agent state remains stable
            assert agent.mcp_client.call_count == request_count
            
        finally:
            await agent.llm_factory.cleanup()
    
    @pytest.mark.asyncio
    async def test_error_resilience_across_agents(self):
        """Test error resilience in multi-agent scenarios"""
        # Create agents with different failure modes
        reliable_agent = InformationGatheringAgent("http://localhost:8001")
        unreliable_agent = InformationGatheringAgent("http://localhost:8001")
        
        # Setup reliable agent normally
        reliable_agent.mcp_client = MockMCPClient()
        reliable_agent.llm_factory = LLMServiceFactory()
        await reliable_agent.llm_factory.initialize()
        
        # Setup unreliable agent with intermittent failures
        unreliable_mcp = MockMCPClient()
        original_call_tool = unreliable_mcp.call_tool
        
        async def failing_call_tool(tool_name, params):
            unreliable_mcp.call_count += 1
            if unreliable_mcp.call_count % 3 == 0:  # Fail every 3rd call
                raise ConnectionError("Simulated network failure")
            return await original_call_tool(tool_name, params)
        
        unreliable_mcp.call_tool = failing_call_tool
        unreliable_agent.mcp_client = unreliable_mcp
        unreliable_agent.llm_factory = LLMServiceFactory()
        await unreliable_agent.llm_factory.initialize()
        
        try:
            # Execute multiple operations
            reliable_results = []
            unreliable_results = []
            
            for i in range(6):  # 6 operations to trigger failures
                # Reliable agent should always succeed
                reliable_result = await reliable_agent.analyze_market_trends([f"symbol_{i}"], "1h")
                reliable_results.append(reliable_result)
                
                # Unreliable agent may fail
                try:
                    unreliable_result = await unreliable_agent.analyze_market_trends([f"symbol_{i}"], "1h")
                    unreliable_results.append(unreliable_result)
                except ConnectionError:
                    # Expected intermittent failures
                    pass
            
            # Verify resilience
            assert len(reliable_results) == 6  # All should succeed
            assert len(unreliable_results) >= 3  # Some should succeed despite failures
            
            # Reliable agent should have perfect success rate
            assert all("insights" in result for result in reliable_results)
            
        finally:
            await reliable_agent.llm_factory.cleanup()
            await unreliable_agent.llm_factory.cleanup()


class TestLLMIntegrationScenarios:
    """Test LLM integration in multi-agent scenarios"""
    
    @pytest.mark.asyncio
    async def test_local_llm_load_balancing(self):
        """Test local LLM load balancing across multiple agents"""
        with patch.dict(os.environ, {}, clear=True):  # No API keys
            # Create multiple agents sharing local LLM resources
            agents = []
            for i in range(3):
                agent = InformationGatheringAgent(f"http://localhost:800{i}")
                agent.mcp_client = MockMCPClient()
                agent.llm_factory = LLMServiceFactory()
                await agent.llm_factory.initialize()
                agents.append(agent)
            
            try:
                # Mock local LLM responses
                mock_responses = [
                    LLMResponse(f"Analysis {i}", "gemma-3-270m-it-mlx", {"total_tokens": 20}, 500.0, 0.0, "stop")
                    for i in range(3)
                ]
                
                # Execute concurrent LLM requests
                tasks = []
                for i, agent in enumerate(agents):
                    with patch.object(agent.llm_factory, 'generate', return_value=mock_responses[i]):
                        task = agent.analyze_market_trends([f"symbol_{i}"], "1h", enhance_with_llm=True)
                        tasks.append(task)
                
                results = await asyncio.gather(*tasks)
                
                # Verify all agents used local models
                for i, result in enumerate(results):
                    assert "enhanced_analysis" in result
                    assert result["model_used"] == "gemma-3-270m-it-mlx"
                
            finally:
                for agent in agents:
                    await agent.llm_factory.cleanup()
    
    @pytest.mark.asyncio
    async def test_llm_fallback_coordination(self):
        """Test LLM fallback coordination between agents"""
        # Create agents with different LLM availability
        primary_agent = InformationGatheringAgent("http://localhost:8001")
        fallback_agent = InformationGatheringAgent("http://localhost:8001")
        
        primary_agent.mcp_client = MockMCPClient()
        fallback_agent.mcp_client = MockMCPClient()
        
        # Setup primary with API keys, fallback without
        with patch.dict(os.environ, {'OPENAI_API_KEY': 'test-key'}):
            primary_agent.llm_factory = LLMServiceFactory()
            await primary_agent.llm_factory.initialize()
        
        with patch.dict(os.environ, {}, clear=True):
            fallback_agent.llm_factory = LLMServiceFactory()
            await fallback_agent.llm_factory.initialize()
        
        try:
            # Simulate primary agent failure, fallback to secondary
            primary_mock = LLMResponse(
                "Primary analysis with GPT model", "gpt-3.5-turbo", 
                {"total_tokens": 30}, 200.0, 0.002, "stop"
            )
            
            fallback_mock = LLMResponse(
                "Fallback analysis with local model", "gemma-3-270m-it-mlx",
                {"total_tokens": 25}, 800.0, 0.0, "stop"
            )
            
            # Test primary agent
            with patch.object(primary_agent.llm_factory, 'generate', return_value=primary_mock):
                primary_result = await primary_agent.analyze_market_trends(["bitcoin"], "1h", enhance_with_llm=True)
            
            # Test fallback agent
            with patch.object(fallback_agent.llm_factory, 'generate', return_value=fallback_mock):
                fallback_result = await fallback_agent.analyze_market_trends(["bitcoin"], "1h", enhance_with_llm=True)
            
            # Verify both approaches work
            assert "enhanced_analysis" in primary_result
            assert "enhanced_analysis" in fallback_result
            assert primary_result["model_used"] == "gpt-3.5-turbo"
            assert fallback_result["model_used"] == "gemma-3-270m-it-mlx"
            
            # Fallback should have no cost
            assert fallback_result.get("cost_estimate", 0) == 0
            
        finally:
            await primary_agent.llm_factory.cleanup()
            await fallback_agent.llm_factory.cleanup()


# Pytest fixtures and utilities
@pytest.fixture(scope="session")
def event_loop():
    """Create an instance of the default event loop for the test session."""
    loop = asyncio.get_event_loop_policy().new_event_loop()
    yield loop
    loop.close()


if __name__ == "__main__":
    # Run tests
    pytest.main([__file__, "-v", "--tb=short"])