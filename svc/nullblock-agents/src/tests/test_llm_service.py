"""
Tests for LLM Service Factory

Comprehensive test suite covering:
- OpenRouter and cloud provider integration
- API key fallback behavior
- Model routing and selection
- Error handling and connectivity
- Performance and cost tracking
"""

import pytest
import asyncio
import os
import aiohttp
from unittest.mock import AsyncMock, patch, MagicMock
from typing import Dict, Any

import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))

from agents.llm_service.factory import LLMServiceFactory, LLMRequest, LLMResponse
from agents.llm_service.router import TaskRequirements, OptimizationGoal, Priority
from agents.llm_service.models import ModelCapability, ModelProvider, AVAILABLE_MODELS

class TestLLMServiceFactory:
    """Test suite for LLM Service Factory"""
    
    @pytest.fixture
    async def factory(self):
        """Create factory instance for testing"""
        factory = LLMServiceFactory(enable_caching=False)
        await factory.initialize()
        yield factory
        await factory.cleanup()
    
    @pytest.fixture
    def sample_request(self):
        """Sample LLM request for testing"""
        return LLMRequest(
            prompt="Hello, test prompt",
            system_prompt="You are a helpful assistant",
            max_tokens=100,
            temperature=0.7
        )
    
    @pytest.fixture
    def concise_request(self):
        """Sample concise LLM request for testing"""
        return LLMRequest(
            prompt="Explain machine learning",
            max_tokens=200,
            temperature=0.8,
            concise=True,
            max_chars=100
        )
    
    @pytest.fixture
    def basic_requirements(self):
        """Basic task requirements for testing"""
        return TaskRequirements(
            required_capabilities=[ModelCapability.CONVERSATION],
            optimization_goal=OptimizationGoal.BALANCED,
            priority=Priority.MEDIUM,
            task_type="test"
        )

class TestConciseModeFeatures:
    """Tests for concise mode and max_chars functionality"""
    
    @pytest.mark.asyncio
    async def test_concise_mode_adjustments(self):
        """Test that concise mode properly adjusts request parameters"""
        factory = LLMServiceFactory()
        await factory.initialize()
        
        # Test basic concise request
        request = LLMRequest(
            prompt="Explain quantum computing",
            max_tokens=500,
            temperature=0.9,
            concise=True
        )
        
        adjusted = factory._adjust_request_for_concise_mode(request)
        
        # Should have reduced max_tokens
        assert adjusted.max_tokens < 500
        assert adjusted.max_tokens <= 200  # Should cap at 200
        
        # Should have set default max_chars
        assert adjusted.max_chars == 100
        
        # Should have lowered temperature
        assert adjusted.temperature <= 0.7
        
        # Should have added concise instructions to system prompt
        assert "concise" in adjusted.system_prompt.lower()
        assert "100 characters or less" in adjusted.system_prompt
        
        await factory.cleanup()
    
    @pytest.mark.asyncio
    async def test_max_chars_custom_value(self):
        """Test custom max_chars value"""
        factory = LLMServiceFactory()
        await factory.initialize()
        
        request = LLMRequest(
            prompt="Describe AI",
            concise=True,
            max_chars=50  # Custom limit
        )
        
        adjusted = factory._adjust_request_for_concise_mode(request)
        
        # Should preserve custom max_chars
        assert adjusted.max_chars == 50
        
        # Should mention 50 characters in instructions
        assert "50 characters or less" in adjusted.system_prompt
        
        # Token limit should be adjusted for character limit (~4 chars per token)
        expected_max_tokens = 50 // 4
        assert adjusted.max_tokens <= expected_max_tokens + 5  # Small tolerance
        
        await factory.cleanup()
    
    @pytest.mark.asyncio
    async def test_quick_generate_with_concise_and_max_chars(self):
        """Test quick_generate with concise and max_chars parameters"""
        factory = LLMServiceFactory()
        await factory.initialize()
        
        # Mock a response
        mock_response = LLMResponse(
            content="AI is machine intelligence.",
            model_used="test-model",
            usage={"total_tokens": 10},
            latency_ms=100.0,
            cost_estimate=0.0,
            finish_reason="stop"
        )
        
        with patch.object(factory, 'generate', return_value=mock_response) as mock_generate:
            result = await factory.quick_generate(
                prompt="What is AI?",
                task_type="explanation", 
                concise=True,
                max_chars=100
            )
            
            # Verify the request was properly constructed
            call_args = mock_generate.call_args
            request = call_args[0][0]  # First argument (LLMRequest)
            
            assert request.concise == True
            assert request.max_chars == 100
            assert result == "AI is machine intelligence."
        
        await factory.cleanup()
    
    @pytest.mark.asyncio
    async def test_concise_mode_with_existing_system_prompt(self):
        """Test concise mode preserves existing system prompt"""
        factory = LLMServiceFactory()
        await factory.initialize()
        
        original_system = "You are a helpful coding assistant."
        request = LLMRequest(
            prompt="Explain Python",
            system_prompt=original_system,
            concise=True,
            max_chars=75
        )
        
        adjusted = factory._adjust_request_for_concise_mode(request)
        
        # Should preserve original system prompt
        assert original_system in adjusted.system_prompt
        
        # Should add concise instructions
        assert "concise" in adjusted.system_prompt.lower()
        assert "75 characters or less" in adjusted.system_prompt
        
        # Should be properly separated
        assert "\n\n" in adjusted.system_prompt
        
        await factory.cleanup()
    
    @pytest.mark.asyncio
    async def test_token_estimation_for_characters(self):
        """Test token estimation based on character limits"""
        factory = LLMServiceFactory()
        await factory.initialize()
        
        test_cases = [
            (20, 5),   # 20 chars -> ~5 tokens
            (40, 10),  # 40 chars -> ~10 tokens
            (100, 25), # 100 chars -> ~25 tokens
            (200, 50), # 200 chars -> ~50 tokens
        ]
        
        for max_chars, expected_tokens in test_cases:
            request = LLMRequest(
                prompt="Test prompt",
                concise=True,
                max_chars=max_chars
            )
            
            adjusted = factory._adjust_request_for_concise_mode(request)
            
            # Token limit should be approximately chars/4, capped at 150
            expected = min(expected_tokens, 150)
            assert abs(adjusted.max_tokens - expected) <= 2  # Small tolerance
        
        await factory.cleanup()

class TestHealthChecking:
    """Tests for comprehensive health checking"""
    
    @pytest.mark.asyncio
    async def test_health_check_comprehensive(self):
        """Test comprehensive health check functionality"""
        factory = LLMServiceFactory()
        await factory.initialize()
        
        health = await factory.health_check()
        
        # Check structure
        assert "overall_status" in health
        assert "api_providers" in health
        assert "api_providers" in health
        assert "models_available" in health
        assert "default_model" in health
        assert "issues" in health
        
        # Check API provider detection
        assert isinstance(health["api_providers"], dict)
        assert all(provider in health["api_providers"] for provider in 
                  ["openai", "anthropic", "groq", "huggingface"])
        
        # Check local provider detection
        assert isinstance(health["api_providers"], dict)
        assert "openrouter" in health["api_providers"]
        assert "ollama" in health["api_providers"]
        
        await factory.cleanup()
    
    @pytest.mark.asyncio
    async def test_health_check_no_services(self):
        """Test health check when no services are available"""
        with patch.dict(os.environ, {}, clear=True):
            factory = LLMServiceFactory()
            await factory.initialize()
            
            # Mock no local connectivity
            with patch.object(factory, 'test_local_connectivity', return_value={"openrouter": False, "ollama": False}):
                health = await factory.health_check()
                
                assert health["overall_status"] in ["unhealthy", "degraded"]
                assert len(health["issues"]) > 0
                assert health["models_available"] == 0
        
        await factory.cleanup()
    
    @pytest.mark.asyncio
    async def test_default_local_model_selection(self):
        """Test default local model selection logic"""
        factory = LLMServiceFactory()
        await factory.initialize()
        
        default_model = factory._get_default_local_model()
        
        # Should prefer Gemma3 if available
        if "gemma-3-270m-it-mlx" in AVAILABLE_MODELS:
            expected_order = [
                "gemma-3-270m-it-mlx",
                "lm-studio-default", 
                "llama2",
                "codellama"
            ]
            
            # First available model should be selected
            for model_name in expected_order:
                if model_name in AVAILABLE_MODELS and AVAILABLE_MODELS[model_name].enabled:
                    assert default_model == model_name
                    break
        
        await factory.cleanup()

class TestIntegrationScenarios:
    """Integration tests for real-world scenarios"""
    
    @pytest.mark.asyncio
    async def test_development_scenario_no_api_keys(self):
        """Test typical development scenario with no API keys"""
        with patch.dict(os.environ, {}, clear=True):
            factory = LLMServiceFactory()
            await factory.initialize()
            
            # Simulate cloud providers being available
            with patch.object(factory, 'test_local_connectivity', return_value={"openrouter": True, "ollama": False}):
                request = LLMRequest(prompt="Hello from development!")
                
                # Mock successful local response
                mock_response = LLMResponse(
                    content="Hello! I'm running locally via Gemma3.",
                    model_used="gemma-3-270m-it-mlx",
                    usage={"total_tokens": 20},
                    latency_ms=500.0,
                    cost_estimate=0.0,
                    finish_reason="stop"
                )
                
                with patch.object(factory, '_generate_openrouter', return_value=mock_response):
                    response = await factory.generate(request)
                    
                    assert response.model_used == "gemma-3-270m-it-mlx"
                    assert response.cost_estimate == 0.0
                    assert "locally" in response.content.lower()
        
        await factory.cleanup()

class TestPerformanceAndMonitoring:
    """Tests for performance tracking and monitoring"""
    
    @pytest.mark.asyncio
    async def test_statistics_tracking(self):
        """Test usage statistics tracking"""
        factory = LLMServiceFactory()
        await factory.initialize()
        
        # Simulate some usage
        mock_response = LLMResponse(
            content="Test response",
            model_used="gemma-3-270m-it-mlx",
            usage={"total_tokens": 10},
            latency_ms=200.0,
            cost_estimate=0.0,
            finish_reason="stop"
        )
        
        factory._update_stats("gemma-3-270m-it-mlx", mock_response)
        
        stats = factory.get_stats()
        
        assert "request_stats" in stats
        assert "cost_tracking" in stats
        assert "cache_stats" in stats
        assert "gemma-3-270m-it-mlx" in stats["request_stats"]
        assert stats["request_stats"]["gemma-3-270m-it-mlx"] == 1
        assert stats["cost_tracking"]["gemma-3-270m-it-mlx"] == 0.0
        
        await factory.cleanup()
    
    @pytest.mark.asyncio
    async def test_caching_behavior(self):
        """Test response caching with local models"""
        factory = LLMServiceFactory(enable_caching=True, cache_ttl=60)
        await factory.initialize()
        
        request = LLMRequest(prompt="Cacheable prompt")
        requirements = TaskRequirements(
            required_capabilities=[ModelCapability.CONVERSATION],
            optimization_goal=OptimizationGoal.BALANCED,
            priority=Priority.MEDIUM,
            task_type="test"
        )
        
        # Cache key should be generated
        cache_key = factory._get_cache_key(request, requirements)
        assert isinstance(cache_key, str)
        
        # Mock response for caching
        mock_response = LLMResponse(
            content="Cached response",
            model_used="gemma-3-270m-it-mlx",
            usage={"total_tokens": 10},
            latency_ms=100.0,
            cost_estimate=0.0,
            finish_reason="stop"
        )
        
        # Cache the response
        factory._cache_response(request, requirements, mock_response)
        
        # Should retrieve from cache
        cached = factory._get_cached_response(request, requirements)
        assert cached is not None
        assert cached.content == "Cached response"
        
        await factory.cleanup()

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