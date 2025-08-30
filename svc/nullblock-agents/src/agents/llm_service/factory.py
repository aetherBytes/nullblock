"""
LLM Service Factory

Main factory class that provides unified LLM access across all Nullblock agents.
Handles model selection, request routing, and response processing.
"""

import asyncio
import aiohttp
import logging
import os
import json
from typing import Dict, List, Any, Optional, AsyncGenerator
from dataclasses import dataclass
from datetime import datetime

from .models import ModelConfig, ModelProvider, ModelTier, ModelCapability, ModelMetrics, AVAILABLE_MODELS, get_model_config, get_default_hecate_model
from .router import ModelRouter, TaskRequirements, RoutingDecision, OptimizationGoal, Priority

logger = logging.getLogger(__name__)

@dataclass
class ReasoningConfig:
    """Configuration for reasoning/thinking tokens"""
    enabled: bool = False
    effort: Optional[str] = None  # "high", "medium", "low" for OpenAI-style
    max_tokens: Optional[int] = None  # Specific token limit for Anthropic-style
    exclude: bool = False  # Exclude reasoning tokens from response

@dataclass
class LLMRequest:
    """Request to LLM service"""
    prompt: str
    system_prompt: Optional[str] = None
    messages: Optional[List[Dict[str, str]]] = None  # Full conversation history as [{"role": "user/assistant", "content": "..."}]
    max_tokens: Optional[int] = None
    temperature: Optional[float] = None
    top_p: Optional[float] = None
    stop_sequences: Optional[List[str]] = None
    tools: Optional[List[Dict]] = None
    model_override: Optional[str] = None  # Force specific model
    concise: bool = False  # Generate shorter, more concise responses
    max_chars: Optional[int] = None  # Maximum characters in response (default 100 when enabled)
    reasoning: Optional[ReasoningConfig] = None  # Reasoning token configuration
    
@dataclass
class LLMResponse:
    """Response from LLM service"""
    content: str
    model_used: str
    usage: Dict[str, int]  # token usage stats
    latency_ms: float
    cost_estimate: float
    finish_reason: str
    tool_calls: Optional[List[Dict]] = None
    metadata: Dict[str, Any] = None
    reasoning: Optional[str] = None  # Reasoning/thinking tokens if available
    reasoning_details: Optional[List[Dict]] = None  # Detailed reasoning blocks

class LLMServiceFactory:
    """
    Unified LLM service factory for all Nullblock agents
    
    Provides:
    - Intelligent model selection based on task requirements
    - Unified API across different providers
    - Cost optimization and monitoring
    - Automatic fallbacks and error handling
    - Response caching and optimization
    """
    
    def __init__(self, enable_caching: bool = True, cache_ttl: int = 300):
        self.router = ModelRouter()
        self.sessions: Dict[str, aiohttp.ClientSession] = {}
        
        # Response caching
        self.enable_caching = enable_caching
        self.cache_ttl = cache_ttl
        self.response_cache: Dict[str, Dict] = {}
        
        # Statistics tracking
        self.request_stats: Dict[str, int] = {}
        self.cost_tracking: Dict[str, float] = {}
        
        logger.info("LLMServiceFactory initialized")
    
    async def initialize(self):
        """Initialize HTTP sessions for all providers"""
        try:
            # Check for available API keys and warn about missing ones
            available_providers = []
            missing_providers = []
            
            # Check API key availability
            api_keys = {
                ModelProvider.OPENAI: os.getenv('OPENAI_API_KEY'),
                ModelProvider.ANTHROPIC: os.getenv('ANTHROPIC_API_KEY'),
                ModelProvider.GROQ: os.getenv('GROQ_API_KEY'),
                ModelProvider.HUGGINGFACE: os.getenv('HUGGINGFACE_API_KEY'),
                ModelProvider.OPENROUTER: os.getenv('OPENROUTER_API_KEY')
            }
            
            for provider, key in api_keys.items():
                if key:
                    available_providers.append(provider.value)
                else:
                    missing_providers.append(provider.value)
            
            # Always available (local providers)
            available_providers.extend([ModelProvider.OLLAMA.value])
            
            # Log provider status
            if available_providers:
                logger.info(f"Available LLM providers: {', '.join(available_providers)}")
            if missing_providers:
                logger.warning(f"Missing API keys for providers: {', '.join(missing_providers)}")
            
            # Log OpenRouter as cloud model aggregator
            if os.getenv('OPENROUTER_API_KEY'):
                logger.info("OpenRouter is configured as the cloud model aggregator")
            
            # Create sessions for each provider (even without keys for potential local use)
            self.sessions[ModelProvider.OPENAI.value] = aiohttp.ClientSession(
                headers={"Authorization": f"Bearer {os.getenv('OPENAI_API_KEY', '')}"}
            )
            
            self.sessions[ModelProvider.ANTHROPIC.value] = aiohttp.ClientSession(
                headers={
                    "x-api-key": os.getenv('ANTHROPIC_API_KEY', ''),
                    "anthropic-version": "2023-06-01"
                }
            )
            
            self.sessions[ModelProvider.GROQ.value] = aiohttp.ClientSession(
                headers={"Authorization": f"Bearer {os.getenv('GROQ_API_KEY', '')}"}
            )
            
            self.sessions[ModelProvider.HUGGINGFACE.value] = aiohttp.ClientSession(
                headers={"Authorization": f"Bearer {os.getenv('HUGGINGFACE_API_KEY', '')}"}
            )
            
            self.sessions[ModelProvider.OPENROUTER.value] = aiohttp.ClientSession(
                headers={
                    "Authorization": f"Bearer {os.getenv('OPENROUTER_API_KEY', '')}",
                    "HTTP-Referer": "https://nullblock.ai",
                    "X-Title": "NullBlock Agent Platform"
                }
            )
            
            # Local providers don't need auth headers
            self.sessions[ModelProvider.OLLAMA.value] = aiohttp.ClientSession()
            
            # Update router with provider availability
            self._update_model_availability(api_keys)
            
            # Test local model connectivity and update availability
            await self._test_local_models()
            
            logger.info("HTTP sessions initialized for all providers")
            
        except Exception as e:
            logger.error(f"Failed to initialize sessions: {e}")
            raise
    
    def _update_model_availability(self, api_keys: Dict):
        """Update model availability based on API key presence"""
        for model_name, config in AVAILABLE_MODELS.items():
            # Disable models that require API keys when keys are missing
            if config.provider in [ModelProvider.OPENAI, ModelProvider.ANTHROPIC, 
                                 ModelProvider.GROQ, ModelProvider.HUGGINGFACE, ModelProvider.OPENROUTER]:
                has_key = bool(api_keys.get(config.provider))
                if not has_key:
                    self.router.update_model_status(model_name, False)
                    logger.debug(f"Disabled model {model_name} - missing API key for {config.provider.value}")
    
    async def _test_local_models(self):
        """Test connectivity to local model providers and update availability"""
        # Test Ollama (local model server)
        try:
            async with self.sessions[ModelProvider.OLLAMA.value].get("http://localhost:11434/api/tags", timeout=3) as resp:
                if resp.status == 200:
                    logger.info("Ollama is available (local model server)")
                    # Enable Ollama models
                    for model_name, config in AVAILABLE_MODELS.items():
                        if config.provider == ModelProvider.OLLAMA:
                            self.router.update_model_status(model_name, True)
                else:
                    logger.warning(f"Ollama returned HTTP {resp.status}")
                    # Disable Ollama models
                    for model_name, config in AVAILABLE_MODELS.items():
                        if config.provider == ModelProvider.OLLAMA:
                            self.router.update_model_status(model_name, False)
        except Exception as e:
            logger.warning(f"Ollama not accessible: {e}")
            # Disable Ollama models
            for model_name, config in AVAILABLE_MODELS.items():
                if config.provider == ModelProvider.OLLAMA:
                    self.router.update_model_status(model_name, False)
    
    def get_available_providers(self) -> List[str]:
        """Get list of providers with valid API keys or local availability"""
        available = []
        
        # Check API key providers
        if os.getenv('OPENAI_API_KEY'):
            available.append(ModelProvider.OPENAI.value)
        if os.getenv('ANTHROPIC_API_KEY'):
            available.append(ModelProvider.ANTHROPIC.value)
        if os.getenv('GROQ_API_KEY'):
            available.append(ModelProvider.GROQ.value)
        if os.getenv('HUGGINGFACE_API_KEY'):
            available.append(ModelProvider.HUGGINGFACE.value)
        if os.getenv('OPENROUTER_API_KEY'):
            available.append(ModelProvider.OPENROUTER.value)
        
        # Local providers are always available (if services are running)
        available.extend([ModelProvider.OLLAMA.value])
        
        return available
    
    def check_prerequisites(self) -> Dict[str, Any]:
        """Check if minimum prerequisites are met for LLM operations"""
        available_providers = self.get_available_providers()
        
        # Filter out local providers for API key check
        api_providers = [p for p in available_providers 
                        if p not in [ModelProvider.OLLAMA.value]]
        
        status = {
            "has_api_keys": len(api_providers) > 0,
            "available_providers": available_providers,
            "api_providers": api_providers,
            "local_providers": [ModelProvider.OLLAMA.value],
            "total_available": len(available_providers),
            "can_operate": len(available_providers) > 0
        }
        
        return status
    
    async def check_model_availability(self) -> bool:
        """Check if any models are actually available and working"""
        try:
            # Try to get a routing decision with basic requirements
            requirements = TaskRequirements(
                required_capabilities=[ModelCapability.REASONING],
                optimization_goal=OptimizationGoal.BALANCED,
                priority=Priority.LOW,
                task_type="health_check"
            )
            
            routing_decision = await self.router.route_request(requirements)
            return bool(routing_decision.selected_model)
            
        except Exception as e:
            logger.debug(f"Model availability check failed: {e}")
            return False
    
    async def cleanup(self):
        """Clean up HTTP sessions"""
        for session in self.sessions.values():
            await session.close()
        
        logger.info("HTTP sessions cleaned up")
    
    async def generate(self, request: LLMRequest, requirements: TaskRequirements = None) -> LLMResponse:
        """
        Generate response using optimal model selection
        
        Args:
            request: LLM request parameters
            requirements: Task requirements for model selection (optional)
            
        Returns:
            LLMResponse with generated content and metadata
        """
        start_time = asyncio.get_event_loop().time()
        
        # Use default requirements if none provided
        if requirements is None:
            requirements = TaskRequirements(
                required_capabilities=[ModelCapability.REASONING],
                optimization_goal=OptimizationGoal.BALANCED,
                priority=Priority.MEDIUM,
                task_type="general"
            )
        
        # Adjust for concise mode
        if request.concise:
            request = self._adjust_request_for_concise_mode(request)
        
        # Auto-adjust for local models when API keys are missing
        requirements = self._adjust_requirements_for_availability(requirements)
        
        try:
            # Check cache first
            if self.enable_caching:
                cached_response = self._get_cached_response(request, requirements)
                if cached_response:
                    logger.debug("Using cached response")
                    return cached_response
            
            # Route request to optimal model
            routing_decision = await self.router.route_request(requirements)
            
            # Check if any models are available
            if not routing_decision.selected_model:
                raise ConnectionError("No LLM models available - check API keys and network connectivity")
            
            # Override model if specified in request
            if request.model_override:
                if request.model_override in AVAILABLE_MODELS:
                    routing_decision.selected_model = request.model_override
                    routing_decision.model_config = AVAILABLE_MODELS[request.model_override]
                else:
                    logger.warning(f"Model override {request.model_override} not found, using routed model")
            
            logger.info(f"Routing decision: {routing_decision.selected_model} (confidence: {routing_decision.confidence:.2f})")
            
            # Generate response
            response = await self._generate_with_model(request, routing_decision)
            
            # Calculate metrics
            end_time = asyncio.get_event_loop().time()
            response.latency_ms = (end_time - start_time) * 1000
            
            # Cache successful response
            if self.enable_caching and response.finish_reason == "stop":
                self._cache_response(request, requirements, response)
            
            # Update statistics
            self._update_stats(routing_decision.selected_model, response)
            
            return response
            
        except Exception as e:
            logger.error(f"Generation failed: {e}")
            
            # Try fallback if available
            try:
                if hasattr(routing_decision, 'fallback_models') and routing_decision.fallback_models:
                    fallback_model = routing_decision.fallback_models[0]
                    logger.info(f"Trying fallback model: {fallback_model}")
                    
                    fallback_config = get_model_config(fallback_model)
                    if fallback_config:
                        fallback_decision = RoutingDecision(
                            selected_model=fallback_model,
                            model_config=fallback_config,
                            confidence=0.5,
                            reasoning=["Fallback due to primary failure"],
                            alternatives=[],
                            estimated_cost=0.0,
                            estimated_latency_ms=fallback_config.metrics.avg_latency_ms,
                            fallback_models=[]
                        )
                        
                        return await self._generate_with_model(request, fallback_decision)
            except Exception as fallback_error:
                logger.error(f"Fallback also failed: {fallback_error}")
            
            raise
    
    def _adjust_request_for_concise_mode(self, request: LLMRequest) -> LLMRequest:
        """
        Adjust request for concise mode by modifying prompts and token limits
        """
        from copy import deepcopy
        adjusted_request = deepcopy(request)
        
        # Handle max_chars constraint
        max_chars = adjusted_request.max_chars or 100  # Default to 100 chars for concise mode
        char_instruction = f"Keep your response to {max_chars} characters or less."
        
        # Add concise instruction to system prompt
        concise_instruction = f"Be concise and direct. Provide short, focused responses without unnecessary elaboration. {char_instruction}"
        
        if adjusted_request.system_prompt:
            adjusted_request.system_prompt = f"{adjusted_request.system_prompt}\n\n{concise_instruction}"
        else:
            adjusted_request.system_prompt = concise_instruction
        
        # Set max_chars if not already set
        if adjusted_request.max_chars is None:
            adjusted_request.max_chars = 100
        
        # Reduce max_tokens for shorter responses (estimate ~4 chars per token)
        estimated_tokens_for_chars = max_chars // 4
        if adjusted_request.max_tokens:
            # Use the more restrictive limit
            adjusted_request.max_tokens = min(
                adjusted_request.max_tokens // 2, 
                estimated_tokens_for_chars,
                200
            )
        else:
            # Set token limit based on character limit
            adjusted_request.max_tokens = min(estimated_tokens_for_chars, 150)
        
        # Slightly lower temperature for more focused responses
        if adjusted_request.temperature is None:
            adjusted_request.temperature = 0.5
        else:
            adjusted_request.temperature = min(adjusted_request.temperature, 0.7)
        
        logger.debug(f"Adjusted request for concise mode: max_tokens={adjusted_request.max_tokens}, max_chars={adjusted_request.max_chars}")
        return adjusted_request

    def _adjust_requirements_for_availability(self, requirements: TaskRequirements) -> TaskRequirements:
        """
        Adjust task requirements based on available models and API keys
        
        When API keys are missing, prioritize local models and adjust optimization goals.
        """
        # Check API key availability
        api_providers_available = []
        if os.getenv('OPENAI_API_KEY'):
            api_providers_available.append('openai')
        if os.getenv('ANTHROPIC_API_KEY'):
            api_providers_available.append('anthropic')
        if os.getenv('GROQ_API_KEY'):
            api_providers_available.append('groq')
        if os.getenv('HUGGINGFACE_API_KEY'):
            api_providers_available.append('huggingface')
        
        # If no API keys are available, adjust for free models
        if not api_providers_available:
            logger.info("No API keys available, adjusting requirements for free models")
            
            # Create a copy to avoid modifying the original
            from copy import deepcopy
            adjusted_requirements = deepcopy(requirements)
            
            # Allow local models as fallback
            adjusted_requirements.allow_local_models = True
            
            # Try OpenRouter first (for free models like DeepSeek), then Ollama as fallback
            adjusted_requirements.preferred_providers = ['openrouter', 'ollama']
            
            # Prefer free models like DeepSeek when no API keys are available
            adjusted_requirements.preferred_providers = ['openrouter']
            
            # Adjust optimization goal to favor free models
            if requirements.optimization_goal == OptimizationGoal.COST:
                # Cost optimization already favors free models
                pass
            elif requirements.optimization_goal == OptimizationGoal.QUALITY:
                # For quality, balance with cost to get free models
                adjusted_requirements.optimization_goal = OptimizationGoal.BALANCED
                logger.info("Adjusted optimization from QUALITY to BALANCED to favor free models")
            elif requirements.optimization_goal == OptimizationGoal.SPEED:
                # For speed, still prefer free models when possible
                adjusted_requirements.optimization_goal = OptimizationGoal.COST
                logger.info("Adjusted optimization from SPEED to COST to favor free models")
            else:
                # For balanced/reliability, favor free models
                adjusted_requirements.optimization_goal = OptimizationGoal.COST
                logger.info("Adjusted optimization to COST to favor free models")
            
            # Relax quality requirements slightly for local models
            if requirements.min_quality_score and requirements.min_quality_score > 0.7:
                adjusted_requirements.min_quality_score = 0.65
                logger.info(f"Relaxed min_quality_score to {adjusted_requirements.min_quality_score} for local models")
            
            return adjusted_requirements
        
        # If some API keys are available, allow local models as fallback
        elif len(api_providers_available) < 2:
            logger.info(f"Limited API providers available ({api_providers_available}), enabling local fallback")
            from copy import deepcopy
            adjusted_requirements = deepcopy(requirements)
            adjusted_requirements.allow_local_models = True
            # Ollama is the local model server
            adjusted_requirements.preferred_providers = ["ollama"]
            return adjusted_requirements
        
        # All API providers available, use original requirements
        return requirements
    
    def _get_default_local_model(self) -> Optional[str]:
        """Get the best available local model"""
        # Ollama is the local model server
        local_models = [
            "llama2",                      # Ollama models
            "codellama"                    # Ollama models
        ]
        
        for model_name in local_models:
            if model_name in AVAILABLE_MODELS:
                config = AVAILABLE_MODELS[model_name]
                if config.enabled and self.router.model_status.get(model_name, True):
                    return model_name
        
        return None
    
    def get_default_hecate_model(self) -> Optional[str]:
        """Get the default model for Hecate Agent (prioritizes DeepSeek free model)"""
        default_model = get_default_hecate_model()
        
        # Check if the default model is available and enabled
        if default_model in AVAILABLE_MODELS:
            config = AVAILABLE_MODELS[default_model]
            if config.enabled and self.router.model_status.get(default_model, True):
                return default_model
        
        return None
    
    async def test_local_connectivity(self) -> Dict[str, bool]:
        """Test connectivity to local model providers"""
        results = {}
        
        # Test Ollama
        try:
            session = self.sessions.get(ModelProvider.OLLAMA.value)
            if session:
                async with session.get("http://localhost:11434/api/tags", timeout=3) as resp:
                    results["ollama"] = resp.status == 200
            else:
                results["ollama"] = False
        except Exception:
            results["ollama"] = False
        
        return results

    async def stream_generate(self, request: LLMRequest, requirements: TaskRequirements) -> AsyncGenerator[str, None]:
        """
        Stream response generation for real-time applications
        """
        # Adjust requirements for availability
        requirements = self._adjust_requirements_for_availability(requirements)
        
        # Route to optimal model
        routing_decision = await self.router.route_request(requirements)
        
        # Override model if specified
        if request.model_override and request.model_override in AVAILABLE_MODELS:
            routing_decision.selected_model = request.model_override
            routing_decision.model_config = AVAILABLE_MODELS[request.model_override]
        
        logger.info(f"Streaming with model: {routing_decision.selected_model}")
        
        async for chunk in self._stream_with_model(request, routing_decision):
            yield chunk
    
    async def _generate_with_model(self, request: LLMRequest, routing_decision: RoutingDecision) -> LLMResponse:
        """Generate response with specific model"""
        config = routing_decision.model_config
        provider = config.provider
        
        if provider == ModelProvider.OPENAI:
            return await self._generate_openai(request, config)
        elif provider == ModelProvider.ANTHROPIC:
            return await self._generate_anthropic(request, config)
        elif provider == ModelProvider.GROQ:
            return await self._generate_groq(request, config)
        elif provider == ModelProvider.OLLAMA:
            return await self._generate_ollama(request, config)
        elif provider == ModelProvider.HUGGINGFACE:
            return await self._generate_huggingface(request, config)
        elif provider == ModelProvider.OPENROUTER:
            return await self._generate_openrouter(request, config)
        else:
            raise ValueError(f"Unsupported provider: {provider}")
    
    async def _generate_openai(self, request: LLMRequest, config: ModelConfig) -> LLMResponse:
        """Generate response using OpenAI API"""
        session = self.sessions[ModelProvider.OPENAI.value]
        
        # Build request payload
        if request.messages:
            # Use provided conversation history
            messages = request.messages.copy()
            # Add system prompt if not already present
            if request.system_prompt and (not messages or messages[0]["role"] != "system"):
                messages.insert(0, {"role": "system", "content": request.system_prompt})
            # Add current prompt as latest user message if not already included
            if not messages or messages[-1]["content"] != request.prompt:
                messages.append({"role": "user", "content": request.prompt})
        else:
            # Fallback to old behavior
            messages = []
            if request.system_prompt:
                messages.append({"role": "system", "content": request.system_prompt})
            messages.append({"role": "user", "content": request.prompt})
        
        payload = {
            "model": config.name,
            "messages": messages,
            "max_tokens": request.max_tokens or config.max_tokens,
            "temperature": request.temperature or config.temperature
        }
        
        if request.tools:
            payload["tools"] = request.tools
            payload["tool_choice"] = "auto"
        
        if request.stop_sequences:
            payload["stop"] = request.stop_sequences
        
        async with session.post(config.api_endpoint, json=payload) as resp:
            if resp.status != 200:
                error_text = await resp.text()
                raise Exception(f"OpenAI API error {resp.status}: {error_text}")
            
            data = await resp.json()
            
            choice = data["choices"][0]
            usage = data.get("usage", {})
            
            # Calculate cost estimate
            input_tokens = usage.get("prompt_tokens", 0)
            output_tokens = usage.get("completion_tokens", 0)
            total_tokens = input_tokens + output_tokens
            cost_estimate = total_tokens * config.metrics.cost_per_1k_tokens / 1000
            
            return LLMResponse(
                content=choice["message"]["content"] or "",
                model_used=config.name,
                usage=usage,
                latency_ms=0.0,  # Will be set by caller
                cost_estimate=cost_estimate,
                finish_reason=choice["finish_reason"],
                tool_calls=choice["message"].get("tool_calls"),
                metadata={"provider": "openai", "model_config": config.name}
            )
    
    async def _generate_anthropic(self, request: LLMRequest, config: ModelConfig) -> LLMResponse:
        """Generate response using Anthropic API"""
        session = self.sessions[ModelProvider.ANTHROPIC.value]
        
        # Build messages payload
        if request.messages:
            # Use provided conversation history, filtering out system messages (handled separately in Anthropic)
            messages = [msg for msg in request.messages if msg["role"] != "system"]
            # Add current prompt if not already included
            if not messages or messages[-1]["content"] != request.prompt:
                messages.append({"role": "user", "content": request.prompt})
        else:
            messages = [{"role": "user", "content": request.prompt}]
        
        payload = {
            "model": config.name,
            "max_tokens": request.max_tokens or config.max_tokens,
            "temperature": request.temperature or config.temperature,
            "messages": messages
        }
        
        if request.system_prompt:
            payload["system"] = request.system_prompt
        
        if request.stop_sequences:
            payload["stop_sequences"] = request.stop_sequences
        
        async with session.post(config.api_endpoint, json=payload) as resp:
            if resp.status != 200:
                error_text = await resp.text()
                raise Exception(f"Anthropic API error {resp.status}: {error_text}")
            
            data = await resp.json()
            
            content = data["content"][0]["text"] if data["content"] else ""
            usage = data.get("usage", {})
            
            # Calculate cost estimate
            input_tokens = usage.get("input_tokens", 0)
            output_tokens = usage.get("output_tokens", 0)
            total_tokens = input_tokens + output_tokens
            cost_estimate = total_tokens * config.metrics.cost_per_1k_tokens / 1000
            
            return LLMResponse(
                content=content,
                model_used=config.name,
                usage={"prompt_tokens": input_tokens, "completion_tokens": output_tokens, "total_tokens": total_tokens},
                latency_ms=0.0,
                cost_estimate=cost_estimate,
                finish_reason=data.get("stop_reason", "stop"),
                metadata={"provider": "anthropic", "model_config": config.name}
            )
    
    async def _generate_groq(self, request: LLMRequest, config: ModelConfig) -> LLMResponse:
        """Generate response using Groq API (OpenAI-compatible)"""
        # Groq uses OpenAI-compatible API
        return await self._generate_openai(request, config)
    
    async def _generate_ollama(self, request: LLMRequest, config: ModelConfig) -> LLMResponse:
        """Generate response using Ollama local API"""
        session = self.sessions[ModelProvider.OLLAMA.value]
        
        payload = {
            "model": config.name,
            "prompt": request.prompt,
            "stream": False,
            "options": {
                "temperature": request.temperature or config.temperature,
                "num_predict": request.max_tokens or config.max_tokens
            }
        }
        
        if request.system_prompt:
            payload["system"] = request.system_prompt
        
        async with session.post(config.api_endpoint, json=payload) as resp:
            if resp.status != 200:
                error_text = await resp.text()
                raise Exception(f"Ollama API error {resp.status}: {error_text}")
            
            data = await resp.json()
            
            # Ollama doesn't provide detailed usage stats
            estimated_tokens = len(data["response"].split()) * 1.3  # Rough estimate
            
            return LLMResponse(
                content=data["response"],
                model_used=config.name,
                usage={"total_tokens": int(estimated_tokens)},
                latency_ms=0.0,
                cost_estimate=0.0,  # Local models are free
                finish_reason="stop",
                metadata={"provider": "ollama", "model_config": config.name}
            )
    
    async def _generate_huggingface(self, request: LLMRequest, config: ModelConfig) -> LLMResponse:
        """Generate response using HuggingFace Inference API"""
        session = self.sessions[ModelProvider.HUGGINGFACE.value]
        
        payload = {
            "inputs": request.prompt,
            "parameters": {
                "max_new_tokens": request.max_tokens or config.max_tokens,
                "temperature": request.temperature or config.temperature
            }
        }
        
        async with session.post(config.api_endpoint, json=payload) as resp:
            if resp.status != 200:
                error_text = await resp.text()
                raise Exception(f"HuggingFace API error {resp.status}: {error_text}")
            
            data = await resp.json()
            
            # HuggingFace returns different formats
            if isinstance(data, list) and data:
                content = data[0].get("generated_text", "")
                # Remove the input prompt from response
                if content.startswith(request.prompt):
                    content = content[len(request.prompt):].strip()
            else:
                content = str(data)
            
            estimated_tokens = len(content.split()) * 1.3
            cost_estimate = estimated_tokens * config.metrics.cost_per_1k_tokens / 1000
            
            return LLMResponse(
                content=content,
                model_used=config.name,
                usage={"total_tokens": int(estimated_tokens)},
                latency_ms=0.0,
                cost_estimate=cost_estimate,
                finish_reason="stop",
                metadata={"provider": "huggingface", "model_config": config.name}
            )
    
    async def _generate_openrouter(self, request: LLMRequest, config: ModelConfig) -> LLMResponse:
        """Generate response using OpenRouter API (OpenAI-compatible)"""
        session = self.sessions[ModelProvider.OPENROUTER.value]
        
        # Build request payload (OpenAI format)
        if request.messages:
            # Use provided conversation history
            messages = request.messages.copy()
            # Add system prompt if not already present
            if request.system_prompt and (not messages or messages[0]["role"] != "system"):
                messages.insert(0, {"role": "system", "content": request.system_prompt})
            # Add current prompt as latest user message if not already included
            if not messages or messages[-1]["content"] != request.prompt:
                messages.append({"role": "user", "content": request.prompt})
        else:
            # Fallback to old behavior
            messages = []
            if request.system_prompt:
                messages.append({"role": "system", "content": request.system_prompt})
            messages.append({"role": "user", "content": request.prompt})
        
        payload = {
            "model": config.name,
            "messages": messages,
            "max_tokens": request.max_tokens or config.max_tokens,
            "temperature": request.temperature or config.temperature
        }
        
        if request.tools:
            payload["tools"] = request.tools
            payload["tool_choice"] = "auto"
        
        if request.stop_sequences:
            payload["stop"] = request.stop_sequences
        
        # Add reasoning configuration if supported and requested
        if request.reasoning and config.supports_reasoning:
            reasoning_payload = {}
            if request.reasoning.enabled:
                reasoning_payload["enabled"] = True
            
            # Only one of effort or max_tokens can be specified
            if request.reasoning.effort:
                reasoning_payload["effort"] = request.reasoning.effort
            elif request.reasoning.max_tokens:
                reasoning_payload["max_tokens"] = request.reasoning.max_tokens
            else:
                # Default to medium effort if reasoning is enabled but no specific config
                reasoning_payload["effort"] = "medium"
                
            if request.reasoning.exclude:
                reasoning_payload["exclude"] = request.reasoning.exclude
            
            if reasoning_payload:
                payload["reasoning"] = reasoning_payload
        
        async with session.post(config.api_endpoint, json=payload) as resp:
            if resp.status != 200:
                error_text = await resp.text()
                raise Exception(f"OpenRouter API error {resp.status}: {error_text}")
            
            data = await resp.json()
            
            choice = data["choices"][0]
            usage = data.get("usage", {})
            
            # Calculate cost estimate
            input_tokens = usage.get("prompt_tokens", 0)
            output_tokens = usage.get("completion_tokens", 0)
            total_tokens = input_tokens + output_tokens
            cost_estimate = total_tokens * config.metrics.cost_per_1k_tokens / 1000
            
            # Extract reasoning information if available
            reasoning = choice["message"].get("reasoning")
            reasoning_details = choice["message"].get("reasoning_details")
            
            return LLMResponse(
                content=choice["message"]["content"] or "",
                model_used=config.name,
                usage=usage,
                latency_ms=0.0,  # Will be set by caller
                cost_estimate=cost_estimate,
                finish_reason=choice["finish_reason"],
                tool_calls=choice["message"].get("tool_calls"),
                metadata={"provider": "openrouter", "model_config": config.name},
                reasoning=reasoning,
                reasoning_details=reasoning_details
            )
    
    async def _stream_with_model(self, request: LLMRequest, routing_decision: RoutingDecision) -> AsyncGenerator[str, None]:
        """Stream response with specific model"""
        # This is a simplified streaming implementation
        # In practice, you'd implement proper streaming for each provider
        
        response = await self._generate_with_model(request, routing_decision)
        
        # Simulate streaming by yielding chunks
        words = response.content.split()
        for i, word in enumerate(words):
            yield word + (" " if i < len(words) - 1 else "")
            await asyncio.sleep(0.05)  # Simulate streaming delay
    
    def _get_cache_key(self, request: LLMRequest, requirements: TaskRequirements) -> str:
        """Generate cache key for request"""
        key_data = {
            "prompt": request.prompt,
            "system_prompt": request.system_prompt,
            "optimization_goal": requirements.optimization_goal.value,
            "capabilities": [cap.value for cap in requirements.required_capabilities]
        }
        return str(hash(json.dumps(key_data, sort_keys=True)))
    
    def _get_cached_response(self, request: LLMRequest, requirements: TaskRequirements) -> Optional[LLMResponse]:
        """Get cached response if available and valid"""
        cache_key = self._get_cache_key(request, requirements)
        
        if cache_key in self.response_cache:
            cached_data = self.response_cache[cache_key]
            
            # Check if cache is still valid
            if (datetime.now().timestamp() - cached_data["timestamp"]) < self.cache_ttl:
                logger.debug("Cache hit")
                return cached_data["response"]
            else:
                # Remove expired cache entry
                del self.response_cache[cache_key]
        
        return None
    
    def _cache_response(self, request: LLMRequest, requirements: TaskRequirements, response: LLMResponse):
        """Cache response"""
        cache_key = self._get_cache_key(request, requirements)
        
        self.response_cache[cache_key] = {
            "response": response,
            "timestamp": datetime.now().timestamp()
        }
        
        # Limit cache size
        if len(self.response_cache) > 1000:
            # Remove oldest entries
            oldest_keys = sorted(
                self.response_cache.keys(),
                key=lambda k: self.response_cache[k]["timestamp"]
            )[:100]
            
            for key in oldest_keys:
                del self.response_cache[key]
    
    def _update_stats(self, model_name: str, response: LLMResponse):
        """Update usage statistics"""
        if model_name not in self.request_stats:
            self.request_stats[model_name] = 0
        if model_name not in self.cost_tracking:
            self.cost_tracking[model_name] = 0.0
        
        self.request_stats[model_name] += 1
        self.cost_tracking[model_name] += response.cost_estimate
    
    def get_stats(self) -> Dict[str, Any]:
        """Get usage statistics"""
        return {
            "request_stats": self.request_stats,
            "cost_tracking": self.cost_tracking,
            "cache_stats": {
                "cache_size": len(self.response_cache),
                "cache_enabled": self.enable_caching
            },
            "router_stats": self.router.get_usage_stats()
        }
    
    async def quick_generate(self, prompt: str, task_type: str = "general", 
                           optimization_goal: str = "balanced", concise: bool = False,
                           max_chars: Optional[int] = None, reasoning: Optional[ReasoningConfig] = None) -> str:
        """Quick generation with minimal configuration"""
        request = LLMRequest(prompt=prompt, concise=concise, max_chars=max_chars, reasoning=reasoning)
        
        requirements = TaskRequirements(
            required_capabilities=[],
            optimization_goal=OptimizationGoal(optimization_goal),
            priority=Priority.MEDIUM,
            task_type=task_type
        )
        
        response = await self.generate(request, requirements)
        return response.content
    
    async def generate_with_reasoning(self, prompt: str, model_name: str = None, 
                                     effort: str = "medium", max_reasoning_tokens: int = None,
                                     exclude_reasoning: bool = False) -> LLMResponse:
        """Generate response with reasoning tokens"""
        reasoning_config = ReasoningConfig(
            enabled=True,
            effort=effort,  # Prioritize effort over max_tokens
            max_tokens=max_reasoning_tokens,
            exclude=exclude_reasoning
        )
        
        request = LLMRequest(
            prompt=prompt,
            reasoning=reasoning_config,
            model_override=model_name
        )
        
        requirements = TaskRequirements(
            required_capabilities=[ModelCapability.REASONING, ModelCapability.REASONING_TOKENS],
            optimization_goal=OptimizationGoal.QUALITY,
            priority=Priority.HIGH,
            task_type="reasoning"
        )
        
        return await self.generate(request, requirements)
    
    async def health_check(self) -> Dict[str, Any]:
        """Comprehensive health check for LLM services"""
        status = {
            "overall_status": "healthy",
            "api_providers": {},
            "local_providers": {},
            "models_available": 0,
            "default_model": None,
            "issues": []
        }
        
        try:
            # Check API providers
            api_keys = {
                "openai": bool(os.getenv('OPENAI_API_KEY')),
                "anthropic": bool(os.getenv('ANTHROPIC_API_KEY')),
                "groq": bool(os.getenv('GROQ_API_KEY')),
                "huggingface": bool(os.getenv('HUGGINGFACE_API_KEY')),
                "openrouter": bool(os.getenv('OPENROUTER_API_KEY'))
            }
            status["api_providers"] = api_keys
            
            # Check local providers
            local_connectivity = await self.test_local_connectivity()
            status["local_providers"] = local_connectivity
            
            # Count available models
            available_models = 0
            for model_name, config in AVAILABLE_MODELS.items():
                if config.enabled and self.router.model_status.get(model_name, True):
                    # Check if provider is available
                    if config.provider == ModelProvider.OLLAMA:
                        if local_connectivity.get("ollama", False):
                            available_models += 1
                    elif config.api_key_env and os.getenv(config.api_key_env):
                        available_models += 1
            
            status["models_available"] = available_models
            
            # Determine default model
            if any(api_keys.values()):
                default_hecate = self.get_default_hecate_model()
                status["default_model"] = default_hecate or "API models available"
            elif any(local_connectivity.values()):
                local_model = self._get_default_local_model()
                status["default_model"] = local_model or "local models available"
            else:
                status["default_model"] = None
                status["issues"].append("No models available")
            
            # Check for issues
            if available_models == 0:
                status["overall_status"] = "unhealthy"
                status["issues"].append("No working models available")
            elif not any(api_keys.values()) and not any(local_connectivity.values()):
                status["overall_status"] = "degraded"
                status["issues"].append("No API keys and no local models accessible")
            elif available_models < 2:
                status["overall_status"] = "degraded"
                status["issues"].append("Limited model options available")
            
            return status
            
        except Exception as e:
            status["overall_status"] = "error"
            status["issues"].append(f"Health check failed: {e}")
            return status