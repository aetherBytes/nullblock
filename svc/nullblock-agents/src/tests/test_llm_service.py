"""
Tests for LLM Service Factory

Comprehensive test suite covering:
- Local model integration (Gemma3 270M via LM Studio)
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
    def basic_requirements(self):
        """Basic task requirements for testing"""
        return TaskRequirements(
            required_capabilities=[ModelCapability.CONVERSATION],
            optimization_goal=OptimizationGoal.BALANCED,
            priority=Priority.MEDIUM,
            task_type="test"
        )


class TestLocalModelIntegration:
    """Tests for local model integration and fallback"""
    
    @pytest.mark.asyncio
    async def test_gemma3_model_configuration(self):
        """Test Gemma3 270M model is properly configured"""
        # Check model exists in configuration
        assert "gemma-3-270m-it-mlx" in AVAILABLE_MODELS
        
        config = AVAILABLE_MODELS["gemma-3-270m-it-mlx"]
        assert config.provider == ModelProvider.LOCAL
        assert config.api_endpoint == "http://localhost:1234/v1/chat/completions"
        assert config.metrics.cost_per_1k_tokens == 0.0  # Local models are free
        assert ModelCapability.CONVERSATION in config.capabilities
    
    @pytest.mark.asyncio
    async def test_local_model_prioritization_no_api_keys(self):
        """Test local models are prioritized when no API keys are available"""
        with patch.dict(os.environ, {}, clear=True):
            factory = LLMServiceFactory()
            await factory.initialize()
            
            requirements = TaskRequirements(
                required_capabilities=[ModelCapability.CONVERSATION],
                optimization_goal=OptimizationGoal.BALANCED,
                priority=Priority.MEDIUM,
                task_type="test"
            )
            
            # Should adjust requirements to prefer local models
            adjusted = factory._adjust_requirements_for_availability(requirements)
            
            assert adjusted.allow_local_models == True
            assert 'local' in adjusted.preferred_providers
            assert 'ollama' in adjusted.preferred_providers
            
            await factory.cleanup()
    
    @pytest.mark.asyncio
    async def test_local_model_fallback_partial_api_keys(self):
        """Test local models as fallback when limited API keys available"""
        with patch.dict(os.environ, {'OPENAI_API_KEY': 'test-key'}, clear=True):
            factory = LLMServiceFactory()
            await factory.initialize()
            
            requirements = TaskRequirements(
                required_capabilities=[ModelCapability.CONVERSATION],
                optimization_goal=OptimizationGoal.BALANCED,
                priority=Priority.MEDIUM,
                task_type="test"
            )
            
            adjusted = factory._adjust_requirements_for_availability(requirements)
            
            # Should enable local fallback with limited API providers
            assert adjusted.allow_local_models == True
            
            await factory.cleanup()
    
    @pytest.mark.asyncio
    async def test_quality_adjustment_for_local_models(self):
        """Test quality requirements are adjusted appropriately for local models"""
        with patch.dict(os.environ, {}, clear=True):
            factory = LLMServiceFactory()
            await factory.initialize()
            
            requirements = TaskRequirements(
                required_capabilities=[ModelCapability.REASONING],
                optimization_goal=OptimizationGoal.QUALITY,
                min_quality_score=0.9,
                priority=Priority.HIGH,
                task_type="analysis"
            )
            
            adjusted = factory._adjust_requirements_for_availability(requirements)
            
            assert adjusted.min_quality_score == 0.65  # Relaxed for local
            assert adjusted.optimization_goal == OptimizationGoal.BALANCED  # Adjusted from QUALITY
            
            await factory.cleanup()


class TestLMStudioIntegration:
    """Tests specifically for LM Studio integration"""
    
    @pytest.mark.asyncio
    async def test_lm_studio_connectivity_check(self):
        """Test LM Studio connectivity testing"""
        factory = LLMServiceFactory()
        await factory.initialize()
        
        # Test connectivity check
        connectivity = await factory.test_local_connectivity()
        
        assert "lm_studio" in connectivity
        assert isinstance(connectivity["lm_studio"], bool)
        assert "ollama" in connectivity
        assert isinstance(connectivity["ollama"], bool)
        
        await factory.cleanup()
    
    @pytest.mark.asyncio
    async def test_lm_studio_request_format(self):
        """Test LM Studio request formatting"""
        factory = LLMServiceFactory()
        await factory.initialize()
        
        request = LLMRequest(
            prompt="Test prompt",
            system_prompt="Test system",
            max_tokens=50,
            temperature=0.8
        )
        
        config = AVAILABLE_MODELS["gemma-3-270m-it-mlx"]
        
        # Mock successful response
        mock_response = {
            "choices": [{
                "message": {"content": "Test response"},
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 5,
                "total_tokens": 15
            }
        }
        
        with patch.object(factory.sessions[ModelProvider.LOCAL.value], 'post') as mock_post:
            mock_post.return_value.__aenter__.return_value.status = 200
            mock_post.return_value.__aenter__.return_value.json = AsyncMock(return_value=mock_response)
            
            response = await factory._generate_lm_studio(request, config)
            
            assert response.content == "Test response"
            assert response.model_used == "gemma-3-270m-it-mlx"
            assert response.cost_estimate == 0.0  # Local models are free
            assert response.usage["total_tokens"] == 15
        
        await factory.cleanup()
    
    @pytest.mark.asyncio
    async def test_lm_studio_error_handling(self):
        """Test LM Studio error handling with helpful messages"""
        factory = LLMServiceFactory()
        await factory.initialize()
        
        request = LLMRequest(prompt="Test")
        config = AVAILABLE_MODELS["gemma-3-270m-it-mlx"]
        
        # Test 404 error (server not running)
        with patch.object(factory.sessions[ModelProvider.LOCAL.value], 'post') as mock_post:
            mock_post.return_value.__aenter__.return_value.status = 404
            mock_post.return_value.__aenter__.return_value.text = AsyncMock(return_value="Not Found")
            
            with pytest.raises(Exception) as exc_info:
                await factory._generate_lm_studio(request, config)
            
            assert "LM Studio server not running" in str(exc_info.value)
            assert "localhost:1234" in str(exc_info.value)
        
        # Test 422 error (model not loaded)
        with patch.object(factory.sessions[ModelProvider.LOCAL.value], 'post') as mock_post:
            mock_post.return_value.__aenter__.return_value.status = 422
            mock_post.return_value.__aenter__.return_value.text = AsyncMock(return_value="Unprocessable Entity")
            
            with pytest.raises(Exception) as exc_info:
                await factory._generate_lm_studio(request, config)
            
            assert "model not loaded" in str(exc_info.value)
        
        await factory.cleanup()
    
    @pytest.mark.asyncio
    async def test_lm_studio_timeout_handling(self):
        """Test LM Studio timeout handling"""
        factory = LLMServiceFactory()
        await factory.initialize()
        
        request = LLMRequest(prompt="Test")
        config = AVAILABLE_MODELS["gemma-3-270m-it-mlx"]
        
        with patch.object(factory.sessions[ModelProvider.LOCAL.value], 'post') as mock_post:
            mock_post.side_effect = asyncio.TimeoutError()
            
            with pytest.raises(Exception) as exc_info:
                await factory._generate_lm_studio(request, config)
            
            assert "timed out" in str(exc_info.value)
            assert "slow or server overloaded" in str(exc_info.value)
        
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
        assert "local_providers" in health
        assert "models_available" in health
        assert "default_model" in health
        assert "issues" in health
        
        # Check API provider detection
        assert isinstance(health["api_providers"], dict)
        assert all(provider in health["api_providers"] for provider in 
                  ["openai", "anthropic", "groq", "huggingface"])
        
        # Check local provider detection
        assert isinstance(health["local_providers"], dict)
        assert "lm_studio" in health["local_providers"]
        assert "ollama" in health["local_providers"]
        
        await factory.cleanup()
    
    @pytest.mark.asyncio
    async def test_health_check_no_services(self):
        """Test health check when no services are available"""
        with patch.dict(os.environ, {}, clear=True):
            factory = LLMServiceFactory()
            await factory.initialize()
            
            # Mock no local connectivity
            with patch.object(factory, 'test_local_connectivity', return_value={"lm_studio": False, "ollama": False}):
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
            
            # Simulate LM Studio being available
            with patch.object(factory, 'test_local_connectivity', return_value={"lm_studio": True, "ollama": False}):
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
                
                with patch.object(factory, '_generate_lm_studio', return_value=mock_response):
                    response = await factory.generate(request)
                    
                    assert response.model_used == "gemma-3-270m-it-mlx"
                    assert response.cost_estimate == 0.0
                    assert "locally" in response.content.lower()
        
        await factory.cleanup()


class TestRealModelInteraction:
    """Tests that require actual local model availability"""
    
    @pytest.mark.asyncio
    async def test_local_model_actual_response(self):
        """Test actual response from local model (requires LM Studio running)"""
        factory = LLMServiceFactory()
        await factory.initialize()
        
        # Check if local models are actually available
        connectivity = await factory.test_local_connectivity()
        
        if connectivity.get("lm_studio", False):
            print("✅ LM Studio detected, testing actual model interaction")
            
            # Test with a simple prompt
            request = LLMRequest(
                prompt="What is 2+2? Answer with just the number.",
                max_tokens=10,
                model_override="gemma-3-270m-it-mlx"
            )
            
            try:
                response = await factory.generate(request)
                
                # Verify we got a real response
                assert response.content is not None
                assert len(response.content.strip()) > 0
                assert response.model_used == "gemma-3-270m-it-mlx"
                assert response.cost_estimate == 0.0  # Local models are free
                assert response.latency_ms > 0  # Should have some latency
                
                print(f"✅ Model response: '{response.content.strip()}'")
                print(f"✅ Latency: {response.latency_ms:.0f}ms")
                
                # Simple validation that it's a reasonable response
                content = response.content.strip().lower()
                assert len(content) >= 1, "Response should not be empty"
                
                # For math question, should contain "4" or similar
                if "4" in content or "four" in content:
                    print("✅ Model correctly answered math question")
                
            except Exception as e:
                if "Cannot connect" in str(e) or "not running" in str(e):
                    pytest.skip("LM Studio not accessible - skipping real model test")
                else:
                    raise e
        else:
            pytest.skip("No local models available - skipping real interaction test")
        
        await factory.cleanup()
    
    @pytest.mark.asyncio 
    async def test_local_model_conversation(self):
        """Test conversational interaction with local model"""
        factory = LLMServiceFactory()
        await factory.initialize()
        
        connectivity = await factory.test_local_connectivity()
        
        if connectivity.get("lm_studio", False):
            print("✅ Testing conversational interaction with local model")
            
            # Test conversation
            request = LLMRequest(
                prompt="Hello! Please introduce yourself in one sentence.",
                system_prompt="You are a helpful AI assistant.",
                max_tokens=50,
                model_override="gemma-3-270m-it-mlx"
            )
            
            try:
                response = await factory.generate(request)
                
                # Verify response quality
                assert response.content is not None
                assert len(response.content.strip()) > 10  # Should be a substantial response
                assert response.model_used == "gemma-3-270m-it-mlx"
                
                print(f"✅ Conversation response: '{response.content.strip()[:100]}...'")
                
                # Should contain greeting or introduction
                content = response.content.lower()
                greeting_words = ["hello", "hi", "i am", "i'm", "assistant", "help"]
                has_greeting = any(word in content for word in greeting_words)
                
                if has_greeting:
                    print("✅ Model provided appropriate conversational response")
                
                # Response should not be just an error or empty
                assert not content.startswith("error")
                assert "failed" not in content
                
            except Exception as e:
                if "Cannot connect" in str(e) or "not running" in str(e):
                    pytest.skip("LM Studio not accessible - skipping conversation test")
                else:
                    raise e
        else:
            pytest.skip("No local models available - skipping conversation test")
        
        await factory.cleanup()
    
    @pytest.mark.asyncio
    async def test_local_model_performance(self):
        """Test local model performance characteristics"""
        factory = LLMServiceFactory()
        await factory.initialize()
        
        connectivity = await factory.test_local_connectivity()
        
        if connectivity.get("lm_studio", False):
            print("✅ Testing local model performance")
            
            # Test multiple short requests to check consistency
            requests = [
                "Count to 3.",
                "What color is the sky?",
                "Name one planet.",
                "What is 5+5?",
                "Say hello."
            ]
            
            responses = []
            total_latency = 0
            
            for i, prompt in enumerate(requests):
                request = LLMRequest(
                    prompt=prompt,
                    max_tokens=20,
                    model_override="gemma-3-270m-it-mlx"
                )
                
                try:
                    response = await factory.generate(request)
                    responses.append(response)
                    total_latency += response.latency_ms
                    
                    # Each response should be valid
                    assert response.content is not None
                    assert len(response.content.strip()) > 0
                    assert response.cost_estimate == 0.0
                    
                    print(f"✅ Request {i+1}: '{response.content.strip()[:30]}...' ({response.latency_ms:.0f}ms)")
                    
                except Exception as e:
                    if "Cannot connect" in str(e):
                        pytest.skip("LM Studio not accessible - skipping performance test")
                    else:
                        raise e
            
            # Performance checks
            if responses:
                avg_latency = total_latency / len(responses)
                print(f"✅ Average latency: {avg_latency:.0f}ms")
                
                # All responses should be successful
                assert len(responses) == len(requests)
                
                # Latency should be reasonable (under 30 seconds per request)
                assert avg_latency < 30000, f"Average latency too high: {avg_latency}ms"
                
                # All responses should be non-empty
                for response in responses:
                    assert len(response.content.strip()) > 0
        else:
            pytest.skip("No local models available - skipping performance test")
        
        await factory.cleanup()
    
    @pytest.mark.asyncio
    async def test_fallback_to_local_when_no_api_keys(self):
        """Test automatic fallback to local models when no API keys"""
        with patch.dict(os.environ, {}, clear=True):  # Remove all API keys
            factory = LLMServiceFactory()
            await factory.initialize()
            
            connectivity = await factory.test_local_connectivity()
            
            if connectivity.get("lm_studio", False):
                print("✅ Testing automatic fallback to local model")
                
                # Don't specify model override - should auto-select local
                request = LLMRequest(
                    prompt="Test automatic fallback. Respond with 'LOCAL'.",
                    max_tokens=20
                )
                
                requirements = TaskRequirements(
                    required_capabilities=[ModelCapability.CONVERSATION],
                    optimization_goal=OptimizationGoal.BALANCED,
                    priority=Priority.MEDIUM,
                    task_type="test"
                )
                
                try:
                    response = await factory.generate(request, requirements)
                    
                    # Should have automatically selected a local model
                    assert response.model_used in ["gemma-3-270m-it-mlx", "lm-studio-default"]
                    assert response.cost_estimate == 0.0  # Local models are free
                    assert len(response.content.strip()) > 0
                    
                    print(f"✅ Auto-selected model: {response.model_used}")
                    print(f"✅ Response: '{response.content.strip()}'")
                    
                except Exception as e:
                    if "Cannot connect" in str(e) or "No LLM models available" in str(e):
                        pytest.skip("No local models accessible - skipping fallback test")
                    else:
                        raise e
            else:
                pytest.skip("No local models available - skipping fallback test")
            
            await factory.cleanup()
    
    @pytest.mark.asyncio
    async def test_production_scenario_with_fallback(self):
        """Test production scenario with API keys and local fallback"""
        with patch.dict(os.environ, {'OPENAI_API_KEY': 'test-key'}):
            factory = LLMServiceFactory()
            await factory.initialize()
            
            # Primary API should be preferred, but local should be available as fallback
            requirements = TaskRequirements(
                required_capabilities=[ModelCapability.CONVERSATION],
                optimization_goal=OptimizationGoal.COST,  # This should prefer local
                priority=Priority.LOW,
                task_type="test"
            )
            
            adjusted = factory._adjust_requirements_for_availability(requirements)
            assert adjusted.allow_local_models == True  # Fallback enabled
        
        await factory.cleanup()
    
    @pytest.mark.asyncio
    async def test_quick_generate_local_fallback(self):
        """Test quick_generate method with local fallback"""
        with patch.dict(os.environ, {}, clear=True):
            factory = LLMServiceFactory()
            await factory.initialize()
            
            # Mock local model availability and response
            mock_response = LLMResponse(
                content="Quick local response",
                model_used="gemma-3-270m-it-mlx", 
                usage={"total_tokens": 15},
                latency_ms=300.0,
                cost_estimate=0.0,
                finish_reason="stop"
            )
            
            with patch.object(factory, 'generate', return_value=mock_response):
                result = await factory.quick_generate("Test prompt", "test", "balanced")
                
                assert result == "Quick local response"
        
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