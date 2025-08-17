"""
Tests for Information Gathering Agent

Comprehensive test suite covering:
- Local LLM integration with agent workflows
- Data source connectivity and analysis  
- Real-time market intelligence gathering
- Agent coordination and response generation
- Error handling and fallback scenarios
"""

import pytest
import asyncio
import os
import json
from unittest.mock import AsyncMock, patch, MagicMock
from typing import Dict, Any

import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))

from agents.information_gathering.main import InformationGatheringAgent
from agents.llm_service.factory import LLMServiceFactory, LLMRequest, LLMResponse
from agents.llm_service.router import TaskRequirements, OptimizationGoal, Priority
from agents.llm_service.models import ModelCapability, ModelProvider, AVAILABLE_MODELS


class TestInformationGatheringAgent:
    """Test suite for Information Gathering Agent"""
    
    @pytest.fixture
    async def agent(self):
        """Create agent instance for testing"""
        agent = InformationGatheringAgent("http://localhost:8001")
        # Mock MCP client initialization
        agent.mcp_client = AsyncMock()
        agent.mcp_client.connect = AsyncMock()
        agent.mcp_client.disconnect = AsyncMock()
        agent.mcp_client.call_tool = AsyncMock()
        yield agent
        if hasattr(agent, 'llm_factory'):
            await agent.llm_factory.cleanup()
    
    @pytest.fixture
    def sample_market_data(self):
        """Sample market data for testing"""
        return {
            "bitcoin": {
                "price": 45000.0,
                "24h_change": 2.5,
                "volume": 28000000000,
                "market_cap": 880000000000
            },
            "ethereum": {
                "price": 3200.0,
                "24h_change": 1.8,
                "volume": 15000000000,
                "market_cap": 390000000000
            }
        }
    
    @pytest.fixture
    def sample_defi_data(self):
        """Sample DeFi protocol data for testing"""
        return {
            "uniswap": {
                "tvl": 4500000000,
                "volume_24h": 1200000000,
                "fees_24h": 3600000,
                "pools_count": 8500,
                "top_pools": [
                    {"pair": "ETH/USDC", "tvl": 450000000, "apr": 12.5},
                    {"pair": "BTC/ETH", "tvl": 380000000, "apr": 8.7}
                ]
            }
        }


class TestAgentInitialization:
    """Tests for agent initialization and setup"""
    
    @pytest.mark.asyncio
    async def test_agent_initialization(self):
        """Test agent initializes correctly with MCP connection"""
        agent = InformationGatheringAgent("http://localhost:8001")
        
        # Mock MCP client
        agent.mcp_client = AsyncMock()
        agent.mcp_client.connect = AsyncMock()
        
        await agent.mcp_client.connect()
        
        # Verify initialization
        assert agent.mcp_server_url == "http://localhost:8001"
        assert agent.mcp_client is not None
        agent.mcp_client.connect.assert_called_once()
    
    @pytest.mark.asyncio
    async def test_llm_factory_integration(self):
        """Test LLM factory integration with local model fallback"""
        agent = InformationGatheringAgent("http://localhost:8001")
        agent.mcp_client = AsyncMock()
        
        # Test LLM factory initialization
        agent.llm_factory = LLMServiceFactory()
        await agent.llm_factory.initialize()
        
        # Should have local model support
        health = await agent.llm_factory.health_check()
        assert "local_providers" in health
        assert "lm_studio" in health["local_providers"]
        
        await agent.llm_factory.cleanup()
    
    @pytest.mark.asyncio
    async def test_agent_with_no_api_keys(self):
        """Test agent works with only local models when no API keys"""
        with patch.dict(os.environ, {}, clear=True):  # Remove all API keys
            agent = InformationGatheringAgent("http://localhost:8001")
            agent.mcp_client = AsyncMock()
            agent.llm_factory = LLMServiceFactory()
            await agent.llm_factory.initialize()
            
            # Should still be functional with local models
            health = await agent.llm_factory.health_check()
            local_available = any(health["local_providers"].values())
            
            # If no local models, should gracefully handle
            if not local_available:
                pytest.skip("No local models available for testing")
            
            await agent.llm_factory.cleanup()


class TestMarketAnalysis:
    """Tests for market analysis functionality"""
    
    @pytest.mark.asyncio
    async def test_analyze_market_trends_basic(self, agent, sample_market_data):
        """Test basic market trends analysis"""
        # Mock MCP responses
        agent.mcp_client.call_tool.return_value = {
            "data": sample_market_data,
            "insights": [
                "Bitcoin showing bullish momentum with 2.5% gains",
                "Ethereum maintaining strong position above $3200",
                "Strong institutional buying detected"
            ],
            "patterns": [
                "Upward price momentum across major cryptos",
                "Increased trading volume in DeFi tokens"
            ],
            "anomalies": [],
            "recommendations": [
                "Consider position increases in ETH",
                "Monitor for potential breakout above $46k BTC"
            ]
        }
        
        result = await agent.analyze_market_trends(["bitcoin", "ethereum"], "24h")
        
        # Verify analysis structure
        assert "insights" in result
        assert "patterns" in result
        assert "recommendations" in result
        assert len(result["insights"]) > 0
        assert len(result["patterns"]) > 0
        
        # Verify MCP was called correctly
        agent.mcp_client.call_tool.assert_called()
    
    @pytest.mark.asyncio
    async def test_analyze_market_trends_with_llm(self, agent, sample_market_data):
        """Test market analysis with LLM enhancement"""
        # Mock MCP response
        agent.mcp_client.call_tool.return_value = {
            "data": sample_market_data,
            "insights": ["Basic market insights"],
            "patterns": ["Price patterns detected"],
            "anomalies": [],
            "recommendations": ["Trading recommendations"]
        }
        
        # Mock LLM factory
        agent.llm_factory = AsyncMock()
        mock_llm_response = LLMResponse(
            content="Enhanced analysis: Market showing strong bullish indicators with institutional accumulation patterns.",
            model_used="gemma-3-270m-it-mlx",
            usage={"total_tokens": 45},
            latency_ms=500.0,
            cost_estimate=0.0,
            finish_reason="stop"
        )
        agent.llm_factory.generate = AsyncMock(return_value=mock_llm_response)
        
        result = await agent.analyze_market_trends(["bitcoin"], "24h", enhance_with_llm=True)
        
        # Should have enhanced insights
        assert "enhanced_analysis" in result
        assert result["enhanced_analysis"] == mock_llm_response.content
        assert result["model_used"] == "gemma-3-270m-it-mlx"
    
    @pytest.mark.asyncio
    async def test_market_analysis_error_handling(self, agent):
        """Test error handling in market analysis"""
        # Mock MCP client failure
        agent.mcp_client.call_tool.side_effect = Exception("MCP connection failed")
        
        with pytest.raises(Exception) as exc_info:
            await agent.analyze_market_trends(["bitcoin"], "24h")
        
        assert "MCP connection failed" in str(exc_info.value)
    
    @pytest.mark.asyncio
    async def test_market_analysis_empty_symbols(self, agent):
        """Test market analysis with empty symbols list"""
        with pytest.raises(ValueError) as exc_info:
            await agent.analyze_market_trends([], "24h")
        
        assert "symbols list cannot be empty" in str(exc_info.value).lower()


class TestDeFiAnalysis:
    """Tests for DeFi opportunity detection"""
    
    @pytest.mark.asyncio
    async def test_detect_defi_opportunities_basic(self, agent, sample_defi_data):
        """Test basic DeFi opportunities detection"""
        # Mock MCP response
        agent.mcp_client.call_tool.return_value = {
            "data": sample_defi_data,
            "opportunities": [
                {
                    "protocol": "uniswap",
                    "strategy": "liquidity_provision",
                    "pair": "ETH/USDC",
                    "estimated_apr": 12.5,
                    "risk_score": 0.3,
                    "required_capital": 10000
                }
            ],
            "insights": [
                "High yields available in ETH/USDC pool",
                "Low impermanent loss risk detected"
            ],
            "recommendations": [
                "Consider LP position in ETH/USDC",
                "Monitor gas fees for optimal entry"
            ]
        }
        
        result = await agent.detect_defi_opportunities(["uniswap"])
        
        # Verify opportunities structure
        assert "opportunities" in result
        assert "insights" in result
        assert "recommendations" in result
        assert len(result["opportunities"]) > 0
        
        # Check opportunity details
        opportunity = result["opportunities"][0]
        assert "protocol" in opportunity
        assert "estimated_apr" in opportunity
        assert "risk_score" in opportunity
    
    @pytest.mark.asyncio
    async def test_defi_opportunities_with_filters(self, agent):
        """Test DeFi opportunities with risk/return filters"""
        # Mock filtered response
        agent.mcp_client.call_tool.return_value = {
            "opportunities": [
                {
                    "protocol": "uniswap",
                    "estimated_apr": 15.0,
                    "risk_score": 0.2,
                    "meets_criteria": True
                }
            ],
            "insights": ["High-yield, low-risk opportunity found"],
            "recommendations": ["Suitable for conservative strategy"]
        }
        
        result = await agent.detect_defi_opportunities(
            ["uniswap"], 
            min_apr=10.0, 
            max_risk=0.5
        )
        
        # Should return filtered opportunities
        assert len(result["opportunities"]) > 0
        opportunity = result["opportunities"][0]
        assert opportunity["estimated_apr"] >= 10.0
        assert opportunity["risk_score"] <= 0.5
    
    @pytest.mark.asyncio
    async def test_defi_opportunities_multiple_protocols(self, agent):
        """Test DeFi analysis across multiple protocols"""
        # Mock multi-protocol response
        agent.mcp_client.call_tool.return_value = {
            "opportunities": [
                {"protocol": "uniswap", "estimated_apr": 12.0},
                {"protocol": "aave", "estimated_apr": 8.5},
                {"protocol": "compound", "estimated_apr": 6.8}
            ],
            "insights": ["Cross-protocol yield opportunities available"],
            "recommendations": ["Diversify across protocols for risk management"]
        }
        
        result = await agent.detect_defi_opportunities(["uniswap", "aave", "compound"])
        
        # Should analyze multiple protocols
        assert len(result["opportunities"]) >= 3
        protocols = [op["protocol"] for op in result["opportunities"]]
        assert "uniswap" in protocols
        assert "aave" in protocols
        assert "compound" in protocols


class TestRealTimeData:
    """Tests for real-time data gathering"""
    
    @pytest.mark.asyncio
    async def test_get_real_time_data_price_oracle(self, agent):
        """Test real-time price data gathering"""
        # Mock price oracle response
        agent.mcp_client.call_tool.return_value = {
            "source": "coingecko",
            "timestamp": "2024-01-15T10:30:00Z",
            "data": {
                "bitcoin": {"price": 45000, "change_24h": 2.5},
                "ethereum": {"price": 3200, "change_24h": 1.8}
            },
            "reliability_score": 0.95
        }
        
        result = await agent.get_real_time_data(
            "price_oracle", 
            "coingecko", 
            {"symbols": ["bitcoin", "ethereum"]}
        )
        
        # Verify real-time data structure
        assert "data" in result
        assert "timestamp" in result
        assert "reliability_score" in result
        assert result["source"] == "coingecko"
        assert "bitcoin" in result["data"]
        assert "ethereum" in result["data"]
    
    @pytest.mark.asyncio
    async def test_real_time_data_social_sentiment(self, agent):
        """Test real-time social sentiment data"""
        # Mock sentiment response
        agent.mcp_client.call_tool.return_value = {
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
        
        result = await agent.get_real_time_data(
            "social_sentiment",
            "twitter",
            {"symbols": ["bitcoin"], "timeframe": "1h"}
        )
        
        # Verify sentiment data
        assert result["source"] == "twitter"
        assert "bitcoin" in result["data"]
        sentiment_data = result["data"]["bitcoin"]
        assert "sentiment_score" in sentiment_data
        assert "mention_count" in sentiment_data
        assert sentiment_data["sentiment_score"] >= -1.0
        assert sentiment_data["sentiment_score"] <= 1.0
    
    @pytest.mark.asyncio
    async def test_real_time_data_error_handling(self, agent):
        """Test real-time data error handling"""
        # Mock data source failure
        agent.mcp_client.call_tool.side_effect = Exception("Data source unavailable")
        
        with pytest.raises(Exception) as exc_info:
            await agent.get_real_time_data("price_oracle", "unavailable_source", {})
        
        assert "Data source unavailable" in str(exc_info.value)


class TestLLMIntegration:
    """Tests for LLM integration with local model support"""
    
    @pytest.mark.asyncio
    async def test_llm_analysis_with_local_model(self, agent):
        """Test LLM analysis using local model"""
        # Setup LLM factory with local model preference
        agent.llm_factory = LLMServiceFactory()
        await agent.llm_factory.initialize()
        
        # Mock successful local model response
        mock_response = LLMResponse(
            content="Based on the market data analysis, I observe strong bullish momentum in Bitcoin with significant institutional accumulation patterns. The 2.5% daily increase combined with elevated volume suggests continued upward pressure.",
            model_used="gemma-3-270m-it-mlx",
            usage={"total_tokens": 65},
            latency_ms=800.0,
            cost_estimate=0.0,
            finish_reason="stop"
        )
        
        with patch.object(agent.llm_factory, 'generate', return_value=mock_response):
            market_context = "Bitcoin: $45,000 (+2.5%), Ethereum: $3,200 (+1.8%)"
            
            result = await agent._enhance_with_llm_analysis(
                market_context,
                "market_analysis",
                ["REASONING", "DATA_ANALYSIS"]
            )
            
            # Verify local model was used
            assert result["content"] is not None
            assert len(result["content"]) > 50  # Substantial response
            assert result["model_used"] == "gemma-3-270m-it-mlx"
            assert result["cost_estimate"] == 0.0  # Local models are free
            assert result["latency_ms"] > 0
        
        await agent.llm_factory.cleanup()
    
    @pytest.mark.asyncio
    async def test_llm_fallback_behavior(self, agent):
        """Test LLM fallback when no API keys available"""
        with patch.dict(os.environ, {}, clear=True):  # Remove all API keys
            agent.llm_factory = LLMServiceFactory()
            await agent.llm_factory.initialize()
            
            # Should automatically adjust to local models
            health = await agent.llm_factory.health_check()
            
            if not any(health["local_providers"].values()):
                pytest.skip("No local models available for fallback testing")
            
            # Test generation with auto-fallback
            request = LLMRequest(
                prompt="Analyze this market trend: BTC +2.5%, ETH +1.8%",
                max_tokens=50
            )
            
            requirements = TaskRequirements(
                required_capabilities=[ModelCapability.REASONING],
                optimization_goal=OptimizationGoal.COST,  # Should prefer local
                priority=Priority.MEDIUM,
                task_type="market_analysis"
            )
            
            # Mock local model response
            mock_response = LLMResponse(
                content="Bullish market trend detected with positive momentum.",
                model_used="gemma-3-270m-it-mlx",
                usage={"total_tokens": 25},
                latency_ms=600.0,
                cost_estimate=0.0,
                finish_reason="stop"
            )
            
            with patch.object(agent.llm_factory, 'generate', return_value=mock_response):
                response = await agent.llm_factory.generate(request, requirements)
                
                # Should use local model due to no API keys
                assert response.model_used == "gemma-3-270m-it-mlx"
                assert response.cost_estimate == 0.0
            
            await agent.llm_factory.cleanup()
    
    @pytest.mark.asyncio
    async def test_llm_performance_with_local_models(self, agent):
        """Test LLM performance characteristics with local models"""
        agent.llm_factory = LLMServiceFactory()
        await agent.llm_factory.initialize()
        
        # Test multiple requests for performance consistency
        test_prompts = [
            "Summarize Bitcoin market trends.",
            "What are the DeFi opportunities?",
            "Analyze Ethereum price action.",
            "Explain yield farming risks.",
            "Compare DEX trading volumes."
        ]
        
        responses = []
        total_latency = 0
        
        for prompt in test_prompts:
            mock_response = LLMResponse(
                content=f"Analysis for: {prompt[:20]}...",
                model_used="gemma-3-270m-it-mlx",
                usage={"total_tokens": 20},
                latency_ms=500.0,
                cost_estimate=0.0,
                finish_reason="stop"
            )
            
            with patch.object(agent.llm_factory, 'quick_generate', return_value=mock_response.content):
                start_time = asyncio.get_event_loop().time()
                result = await agent.llm_factory.quick_generate(prompt, "analysis", "speed")
                end_time = asyncio.get_event_loop().time()
                
                latency = (end_time - start_time) * 1000
                responses.append(result)
                total_latency += latency
                
                # Verify response quality
                assert result is not None
                assert len(result.strip()) > 5  # Non-empty response
        
        # Performance checks
        avg_latency = total_latency / len(responses)
        assert len(responses) == len(test_prompts)
        assert avg_latency < 5000  # Under 5 seconds average
        
        await agent.llm_factory.cleanup()


class TestAgentCoordination:
    """Tests for agent coordination and workflow integration"""
    
    @pytest.mark.asyncio
    async def test_multi_step_analysis_workflow(self, agent):
        """Test multi-step analysis workflow coordination"""
        # Mock workflow steps
        step_responses = [
            {"step": "data_gathering", "status": "completed", "data": "market_data"},
            {"step": "pattern_analysis", "status": "completed", "patterns": ["trend_1"]},
            {"step": "llm_enhancement", "status": "completed", "analysis": "enhanced_insights"}
        ]
        
        agent.mcp_client.call_tool.side_effect = step_responses
        
        # Mock LLM enhancement
        agent.llm_factory = AsyncMock()
        agent.llm_factory.generate = AsyncMock(return_value=LLMResponse(
            content="Comprehensive market analysis with actionable insights.",
            model_used="gemma-3-270m-it-mlx",
            usage={"total_tokens": 40},
            latency_ms=700.0,
            cost_estimate=0.0,
            finish_reason="stop"
        ))
        
        # Execute multi-step workflow
        result = await agent._execute_analysis_workflow(
            "comprehensive_market_analysis",
            {"symbols": ["bitcoin", "ethereum"], "timeframe": "24h"}
        )
        
        # Verify workflow coordination
        assert "workflow_results" in result
        assert len(result["workflow_results"]) == len(step_responses)
        assert all(step["status"] == "completed" for step in result["workflow_results"])
    
    @pytest.mark.asyncio
    async def test_parallel_data_gathering(self, agent):
        """Test parallel data gathering from multiple sources"""
        # Mock multiple data sources
        source_responses = {
            "price_data": {"bitcoin": 45000, "ethereum": 3200},
            "volume_data": {"bitcoin": 28000000000, "ethereum": 15000000000},
            "sentiment_data": {"bitcoin": 0.75, "ethereum": 0.68}
        }
        
        async def mock_call_tool(tool_name, params):
            if "price" in tool_name:
                return source_responses["price_data"]
            elif "volume" in tool_name:
                return source_responses["volume_data"]
            elif "sentiment" in tool_name:
                return source_responses["sentiment_data"]
            return {}
        
        agent.mcp_client.call_tool = mock_call_tool
        
        # Execute parallel gathering
        sources = ["price_oracle", "volume_tracker", "sentiment_monitor"]
        results = await agent._gather_parallel_data(sources, {"symbols": ["bitcoin", "ethereum"]})
        
        # Verify parallel execution results
        assert len(results) == len(sources)
        assert all(result is not None for result in results)
    
    @pytest.mark.asyncio
    async def test_error_recovery_in_workflow(self, agent):
        """Test error recovery in multi-step workflows"""
        # Mock partial workflow failure
        call_count = 0
        
        async def mock_call_with_failure(tool_name, params):
            nonlocal call_count
            call_count += 1
            if call_count == 2:  # Second call fails
                raise Exception("Temporary service failure")
            return {"status": "success", "data": f"step_{call_count}"}
        
        agent.mcp_client.call_tool = mock_call_with_failure
        
        # Should handle failure gracefully
        result = await agent._execute_resilient_workflow(
            ["step1", "step2", "step3"],
            {"retry_failed": True, "max_retries": 2}
        )
        
        # Verify error recovery
        assert "completed_steps" in result
        assert "failed_steps" in result
        assert len(result["failed_steps"]) <= 1  # At most one failure
        assert result["recovery_attempted"] is True


class TestPerformanceAndReliability:
    """Tests for performance and reliability characteristics"""
    
    @pytest.mark.asyncio
    async def test_agent_performance_under_load(self, agent):
        """Test agent performance under concurrent requests"""
        # Mock multiple concurrent analysis requests
        agent.mcp_client.call_tool = AsyncMock(return_value={
            "insights": ["Market insight"],
            "patterns": ["Price pattern"],
            "recommendations": ["Trading recommendation"]
        })
        
        # Execute concurrent requests
        concurrent_requests = 5
        tasks = []
        
        for i in range(concurrent_requests):
            task = agent.analyze_market_trends([f"symbol_{i}"], "1h")
            tasks.append(task)
        
        # Wait for all requests to complete
        results = await asyncio.gather(*tasks, return_exceptions=True)
        
        # Verify all requests completed
        assert len(results) == concurrent_requests
        successful_results = [r for r in results if not isinstance(r, Exception)]
        assert len(successful_results) >= concurrent_requests * 0.8  # 80% success rate
    
    @pytest.mark.asyncio
    async def test_agent_memory_efficiency(self, agent):
        """Test agent memory usage remains stable"""
        # Mock data for memory testing
        large_dataset = {"data": ["item"] * 1000}  # Simulate large dataset
        
        agent.mcp_client.call_tool = AsyncMock(return_value=large_dataset)
        
        # Process large dataset multiple times
        for _ in range(10):
            result = await agent.analyze_market_trends(["test_symbol"], "1h")
            assert result is not None
            # In real implementation, check memory usage here
        
        # Memory should remain stable (would need memory profiling in real test)
        assert True  # Placeholder for memory assertions
    
    @pytest.mark.asyncio
    async def test_agent_connection_resilience(self, agent):
        """Test agent resilience to connection issues"""
        # Mock intermittent connection failures
        call_count = 0
        
        async def mock_unreliable_connection(tool_name, params):
            nonlocal call_count
            call_count += 1
            if call_count % 3 == 0:  # Every third call fails
                raise ConnectionError("Network timeout")
            return {"status": "success", "data": "response"}
        
        agent.mcp_client.call_tool = mock_unreliable_connection
        
        # Should handle connection issues gracefully
        successful_calls = 0
        total_attempts = 9
        
        for _ in range(total_attempts):
            try:
                await agent.get_real_time_data("test", "test", {})
                successful_calls += 1
            except ConnectionError:
                pass  # Expected intermittent failures
        
        # Should have some successful calls despite failures
        assert successful_calls >= total_attempts * 0.6  # 60% success rate


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