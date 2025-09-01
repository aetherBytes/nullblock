"""
LLM Model Definitions and Configurations

Defines available LLM models, their capabilities, and configuration parameters.
"""

import requests
import asyncio
from enum import Enum
from typing import Dict, List, Optional, Any
from dataclasses import dataclass

class ModelProvider(Enum):
    """Available LLM providers"""
    OPENAI = "openai"
    ANTHROPIC = "anthropic"
    HUGGINGFACE = "huggingface"
    OPENROUTER = "openrouter"
    OLLAMA = "ollama"
    GROQ = "groq"

class ModelTier(Enum):
    """Model performance tiers"""
    PREMIUM = "premium"      # Highest quality, most expensive
    STANDARD = "standard"    # Balanced quality/cost
    FAST = "fast"           # Optimized for speed
    ECONOMICAL = "economical" # Lowest cost
    LOCAL = "local"         # Local deployment

class ModelCapability(Enum):
    """Model specialized capabilities"""
    REASONING = "reasoning"           # Complex logical reasoning
    CODE = "code"                    # Code generation and analysis
    MATH = "math"                    # Mathematical computations
    DATA_ANALYSIS = "data_analysis"  # Data analysis and statistics
    CREATIVE = "creative"            # Creative writing and generation
    CONVERSATION = "conversation"    # Natural conversation
    FUNCTION_CALLING = "function_calling" # Tool/function usage
    MULTIMODAL = "multimodal"       # Image/text processing
    LONG_CONTEXT = "long_context"   # Large context windows
    REASONING_TOKENS = "reasoning_tokens" # Supports reasoning/thinking tokens

@dataclass
class ModelMetrics:
    """Performance and cost metrics for a model"""
    avg_latency_ms: float           # Average response latency
    tokens_per_second: int          # Generation speed
    cost_per_1k_tokens: float      # Cost in USD per 1K tokens
    context_window: int            # Maximum context length
    quality_score: float           # Subjective quality rating (0-1)
    reliability_score: float       # Reliability rating (0-1)

@dataclass
class ModelConfig:
    """Configuration for an LLM model"""
    name: str
    provider: ModelProvider
    tier: ModelTier
    capabilities: List[ModelCapability]
    metrics: ModelMetrics
    api_endpoint: Optional[str] = None
    api_key_env: Optional[str] = None
    model_path: Optional[str] = None     # For local models
    max_tokens: int = 4000
    temperature: float = 0.7
    enabled: bool = True
    description: str = ""
    supports_reasoning: bool = False     # Whether model supports reasoning tokens
    reasoning_max_tokens: int = 2000     # Default reasoning token limit
    display_name: Optional[str] = None   # User-friendly display name
    icon: Optional[str] = None          # Model icon
    created: Optional[int] = None        # Creation timestamp from provider

# Predefined model configurations
AVAILABLE_MODELS = {
    # OpenAI Models
    "gpt-4": ModelConfig(
        name="gpt-4",
        provider=ModelProvider.OPENAI,
        tier=ModelTier.PREMIUM,
        capabilities=[
            ModelCapability.REASONING,
            ModelCapability.CODE,
            ModelCapability.MATH,
            ModelCapability.DATA_ANALYSIS,
            ModelCapability.CREATIVE,
            ModelCapability.FUNCTION_CALLING
        ],
        metrics=ModelMetrics(
            avg_latency_ms=3000,
            tokens_per_second=20,
            cost_per_1k_tokens=0.03,
            context_window=8192,
            quality_score=0.95,
            reliability_score=0.98
        ),
        api_endpoint="https://api.openai.com/v1/chat/completions",
        api_key_env="OPENAI_API_KEY",
        description="Premium OpenAI model for complex reasoning and analysis"
    ),
    
    "gpt-3.5-turbo": ModelConfig(
        name="gpt-3.5-turbo",
        provider=ModelProvider.OPENAI,
        tier=ModelTier.FAST,
        capabilities=[
            ModelCapability.CONVERSATION,
            ModelCapability.CODE,
            ModelCapability.DATA_ANALYSIS,
            ModelCapability.FUNCTION_CALLING
        ],
        metrics=ModelMetrics(
            avg_latency_ms=800,
            tokens_per_second=50,
            cost_per_1k_tokens=0.002,
            context_window=4096,
            quality_score=0.80,
            reliability_score=0.95
        ),
        api_endpoint="https://api.openai.com/v1/chat/completions",
        api_key_env="OPENAI_API_KEY",
        description="Fast and economical model for routine tasks"
    ),
    
    # Anthropic Models
    "claude-3-opus": ModelConfig(
        name="claude-3-opus",
        provider=ModelProvider.ANTHROPIC,
        tier=ModelTier.PREMIUM,
        capabilities=[
            ModelCapability.REASONING,
            ModelCapability.CODE,
            ModelCapability.MATH,
            ModelCapability.DATA_ANALYSIS,
            ModelCapability.CREATIVE,
            ModelCapability.LONG_CONTEXT
        ],
        metrics=ModelMetrics(
            avg_latency_ms=2500,
            tokens_per_second=25,
            cost_per_1k_tokens=0.015,
            context_window=200000,
            quality_score=0.96,
            reliability_score=0.97
        ),
        api_endpoint="https://api.anthropic.com/v1/messages",
        api_key_env="ANTHROPIC_API_KEY",
        description="Premium Anthropic model with excellent reasoning capabilities"
    ),
    
    "claude-3-haiku": ModelConfig(
        name="claude-3-haiku",
        provider=ModelProvider.ANTHROPIC,
        tier=ModelTier.FAST,
        capabilities=[
            ModelCapability.CONVERSATION,
            ModelCapability.CODE,
            ModelCapability.DATA_ANALYSIS
        ],
        metrics=ModelMetrics(
            avg_latency_ms=600,
            tokens_per_second=60,
            cost_per_1k_tokens=0.0008,
            context_window=200000,
            quality_score=0.75,
            reliability_score=0.93
        ),
        api_endpoint="https://api.anthropic.com/v1/messages",
        api_key_env="ANTHROPIC_API_KEY",
        description="Fast Anthropic model for quick responses"
    ),
    
    # Local Models (via Ollama)
    "llama2": ModelConfig(
        name="llama2",
        provider=ModelProvider.OLLAMA,
        tier=ModelTier.LOCAL,
        capabilities=[
            ModelCapability.CONVERSATION,
            ModelCapability.REASONING,
            ModelCapability.CODE
        ],
        metrics=ModelMetrics(
            avg_latency_ms=5000,
            tokens_per_second=10,
            cost_per_1k_tokens=0.0,  # No cost for local
            context_window=4096,
            quality_score=0.65,  # Lower than LM Studio models
            reliability_score=0.80  # Lower than LM Studio models
        ),
        api_endpoint="http://localhost:11434/api/chat",
        description="Local Llama2 model for privacy-focused tasks"
    ),
    
    "codellama": ModelConfig(
        name="codellama",
        provider=ModelProvider.OLLAMA,
        tier=ModelTier.LOCAL,
        capabilities=[
            ModelCapability.CODE,
            ModelCapability.REASONING
        ],
        metrics=ModelMetrics(
            avg_latency_ms=4000,
            tokens_per_second=12,
            cost_per_1k_tokens=0.0,
            context_window=4096,
            quality_score=0.70,  # Lower than LM Studio models
            reliability_score=0.85  # Lower than LM Studio models
        ),
        api_endpoint="http://localhost:11434/api/chat",
        description="Local CodeLlama model specialized for code generation"
    ),
    
    # Groq (Fast inference)
    "mixtral-8x7b-groq": ModelConfig(
        name="mixtral-8x7b-32768",
        provider=ModelProvider.GROQ,
        tier=ModelTier.FAST,
        capabilities=[
            ModelCapability.REASONING,
            ModelCapability.CODE,
            ModelCapability.MATH,
            ModelCapability.LONG_CONTEXT
        ],
        metrics=ModelMetrics(
            avg_latency_ms=400,
            tokens_per_second=100,
            cost_per_1k_tokens=0.0005,
            context_window=32768,
            quality_score=0.82,
            reliability_score=0.90
        ),
        api_endpoint="https://api.groq.com/openai/v1/chat/completions",
        api_key_env="GROQ_API_KEY",
        description="Ultra-fast Mixtral model via Groq inference"
    ),
    
    # OpenRouter Models
    "openai/gpt-4o": ModelConfig(
        name="openai/gpt-4o",
        provider=ModelProvider.OPENROUTER,
        tier=ModelTier.PREMIUM,
        capabilities=[
            ModelCapability.REASONING,
            ModelCapability.CODE,
            ModelCapability.MATH,
            ModelCapability.DATA_ANALYSIS,
            ModelCapability.CREATIVE,
            ModelCapability.MULTIMODAL,
            ModelCapability.FUNCTION_CALLING
        ],
        metrics=ModelMetrics(
            avg_latency_ms=2500,
            tokens_per_second=25,
            cost_per_1k_tokens=0.005,
            context_window=128000,
            quality_score=0.96,
            reliability_score=0.98
        ),
        api_endpoint="https://openrouter.ai/api/v1/chat/completions",
        api_key_env="OPENROUTER_API_KEY",
        description="GPT-4o via OpenRouter - latest OpenAI model with multimodal capabilities",
        display_name="GPT-4o",
        icon="🧠"
    ),
    
    "anthropic/claude-3.5-sonnet": ModelConfig(
        name="anthropic/claude-3.5-sonnet",
        provider=ModelProvider.OPENROUTER,
        tier=ModelTier.PREMIUM,
        capabilities=[
            ModelCapability.REASONING,
            ModelCapability.CODE,
            ModelCapability.MATH,
            ModelCapability.DATA_ANALYSIS,
            ModelCapability.CREATIVE,
            ModelCapability.LONG_CONTEXT
        ],
        metrics=ModelMetrics(
            avg_latency_ms=2000,
            tokens_per_second=30,
            cost_per_1k_tokens=0.003,
            context_window=200000,
            quality_score=0.97,
            reliability_score=0.96
        ),
        api_endpoint="https://openrouter.ai/api/v1/chat/completions",
        api_key_env="OPENROUTER_API_KEY",
        description="Claude 3.5 Sonnet via OpenRouter - excellent reasoning and coding capabilities",
        display_name="Claude 3.5 Sonnet",
        icon="🤖"
    ),
    
    "qwen/qwen-2.5-72b-instruct": ModelConfig(
        name="qwen/qwen-2.5-72b-instruct",
        provider=ModelProvider.OPENROUTER,
        tier=ModelTier.STANDARD,
        capabilities=[
            ModelCapability.REASONING,
            ModelCapability.MATH,
            ModelCapability.CODE,
            ModelCapability.DATA_ANALYSIS,
            ModelCapability.CONVERSATION,
            ModelCapability.LONG_CONTEXT
        ],
        metrics=ModelMetrics(
            avg_latency_ms=3000,
            tokens_per_second=20,
            cost_per_1k_tokens=0.0008,
            context_window=32768,
            quality_score=0.88,
            reliability_score=0.94
        ),
        api_endpoint="https://openrouter.ai/api/v1/chat/completions",
        api_key_env="OPENROUTER_API_KEY",
        description="Qwen 2.5 72B via OpenRouter - powerful reasoning model with multilingual support",
        display_name="Qwen 2.5 72B",
        icon="🐉"
    ),
    
    "meta-llama/llama-3.1-8b-instruct": ModelConfig(
        name="meta-llama/llama-3.1-8b-instruct",
        provider=ModelProvider.OPENROUTER,
        tier=ModelTier.FAST,
        capabilities=[
            ModelCapability.CONVERSATION,
            ModelCapability.REASONING,
            ModelCapability.CODE,
            ModelCapability.CREATIVE
        ],
        metrics=ModelMetrics(
            avg_latency_ms=1500,
            tokens_per_second=40,
            cost_per_1k_tokens=0.0002,
            context_window=128000,
            quality_score=0.80,
            reliability_score=0.92
        ),
        api_endpoint="https://openrouter.ai/api/v1/chat/completions",
        api_key_env="OPENROUTER_API_KEY",
        description="Llama 3.1 8B via OpenRouter - fast and efficient for general tasks",
        display_name="Llama 3.1 8B",
        icon="⚡"
    ),
    
    "deepseek/deepseek-chat-v3.1:free": ModelConfig(
        name="deepseek/deepseek-chat-v3.1:free",
        provider=ModelProvider.OPENROUTER,
        tier=ModelTier.ECONOMICAL,
        capabilities=[
            ModelCapability.CONVERSATION,
            ModelCapability.REASONING,
            ModelCapability.CODE,
            ModelCapability.MATH,
            ModelCapability.DATA_ANALYSIS
        ],
        metrics=ModelMetrics(
            avg_latency_ms=2000,
            tokens_per_second=30,
            cost_per_1k_tokens=0.0,  # Free model
            context_window=32000,
            quality_score=0.82,
            reliability_score=0.90
        ),
        api_endpoint="https://openrouter.ai/api/v1/chat/completions",
        api_key_env="OPENROUTER_API_KEY",
        description="DeepSeek Chat v3.1 Free - excellent free model for conversation and coding",
        display_name="DeepSeek Chat (Free)",
        icon="💬"
    ),
    
    "deepseek/deepseek-r1": ModelConfig(
        name="deepseek/deepseek-r1",
        provider=ModelProvider.OPENROUTER,
        tier=ModelTier.STANDARD,
        capabilities=[
            ModelCapability.REASONING,
            ModelCapability.REASONING_TOKENS,
            ModelCapability.CODE,
            ModelCapability.MATH,
            ModelCapability.DATA_ANALYSIS,
            ModelCapability.CONVERSATION
        ],
        metrics=ModelMetrics(
            avg_latency_ms=5000,  # Reasoning models are slower
            tokens_per_second=15,
            cost_per_1k_tokens=0.0014,  # Input: $0.14, Output: $2.8 per 1M tokens
            context_window=64000,
            quality_score=0.93,
            reliability_score=0.95
        ),
        api_endpoint="https://openrouter.ai/api/v1/chat/completions",
        api_key_env="OPENROUTER_API_KEY",
        description="DeepSeek-R1 - Advanced reasoning model with transparent thinking process",
        supports_reasoning=True,
        reasoning_max_tokens=8000,
        display_name="DeepSeek-R1 (Reasoning)",
        icon="💭"
    ),
    
    "openai/o3-mini": ModelConfig(
        name="openai/o3-mini",
        provider=ModelProvider.OPENROUTER,
        tier=ModelTier.PREMIUM,
        capabilities=[
            ModelCapability.REASONING,
            ModelCapability.REASONING_TOKENS,
            ModelCapability.MATH,
            ModelCapability.CODE,
            ModelCapability.DATA_ANALYSIS
        ],
        metrics=ModelMetrics(
            avg_latency_ms=8000,  # Reasoning models are slower
            tokens_per_second=10,
            cost_per_1k_tokens=0.003,  # Estimated cost
            context_window=128000,
            quality_score=0.96,
            reliability_score=0.98
        ),
        api_endpoint="https://openrouter.ai/api/v1/chat/completions",
        api_key_env="OPENROUTER_API_KEY",
        description="OpenAI o3-mini - Advanced reasoning model with high-effort thinking",
        supports_reasoning=True,
        reasoning_max_tokens=10000,
        display_name="O3-Mini (Reasoning)",
        icon="🔍"
    ),
    
    # HuggingFace Models
    "mistral-7b": ModelConfig(
        name="mistralai/Mistral-7B-Instruct-v0.1",
        provider=ModelProvider.HUGGINGFACE,
        tier=ModelTier.ECONOMICAL,
        capabilities=[
            ModelCapability.CONVERSATION,
            ModelCapability.CODE,
            ModelCapability.REASONING
        ],
        metrics=ModelMetrics(
            avg_latency_ms=2000,
            tokens_per_second=15,
            cost_per_1k_tokens=0.0002,
            context_window=4096,
            quality_score=0.72,
            reliability_score=0.87
        ),
        api_endpoint="https://api-inference.huggingface.co/models/mistralai/Mistral-7B-Instruct-v0.1",
        api_key_env="HUGGINGFACE_API_KEY",
        description="Economical Mistral model via HuggingFace"
    )
}

async def fetch_openrouter_models() -> List[Dict[str, Any]]:
    """Fetch available models from OpenRouter API"""
    try:
        url = "https://openrouter.ai/api/v1/models"
        response = requests.get(url, timeout=10)
        response.raise_for_status()
        data = response.json()
        return data.get("data", [])
    except Exception as e:
        print(f"Error fetching OpenRouter models: {e}")
        return []

def create_model_config_from_openrouter(model_data: Dict[str, Any]) -> Optional[ModelConfig]:
    """Create a ModelConfig from OpenRouter model data"""
    try:
        model_id = model_data.get("id", "")
        name = model_data.get("name", "")
        description = model_data.get("description", "")
        
        # Extract pricing info
        pricing = model_data.get("pricing", {})
        prompt_cost = float(pricing.get("prompt", "0"))
        completion_cost = float(pricing.get("completion", "0"))
        total_cost = prompt_cost + completion_cost
        
        # Determine tier based on cost
        if total_cost == 0:
            tier = ModelTier.ECONOMICAL
        elif total_cost < 0.001:
            tier = ModelTier.FAST
        elif total_cost < 0.01:
            tier = ModelTier.STANDARD
        else:
            tier = ModelTier.PREMIUM
        
        # Determine capabilities based on model info
        capabilities = [ModelCapability.CONVERSATION]
        if "code" in description.lower() or "programming" in description.lower():
            capabilities.append(ModelCapability.CODE)
        if "reasoning" in description.lower() or "thinking" in description.lower():
            capabilities.append(ModelCapability.REASONING)
        if "math" in description.lower() or "mathematical" in description.lower():
            capabilities.append(ModelCapability.MATH)
        
        # Create display name
        display_name = name.replace("/", " ").replace("-", " ").title()
        if "free" in name.lower():
            display_name += " (Free)"
        
        # Determine icon based on model characteristics
        icon = "🤖"  # default
        if "claude" in name.lower():
            icon = "🤖"
        elif "gpt" in name.lower():
            icon = "🧠"
        elif "deepseek" in name.lower():
            icon = "💬"
        elif "llama" in name.lower():
            icon = "🦙"
        elif "gemma" in name.lower():
            icon = "💎"
        elif "qwen" in name.lower():
            icon = "🐉"
        elif "mistral" in name.lower():
            icon = "🌪️"
        elif "free" in name.lower():
            icon = "🆓"
        
        return ModelConfig(
            name=model_id,
            provider=ModelProvider.OPENROUTER,
            tier=tier,
            capabilities=capabilities,
            metrics=ModelMetrics(
                avg_latency_ms=2000,  # Default estimate
                tokens_per_second=20,  # Default estimate
                cost_per_1k_tokens=total_cost,
                context_window=model_data.get("context_length", 32000),
                quality_score=0.8,  # Default estimate
                reliability_score=0.9  # Default estimate
            ),
            api_endpoint="https://openrouter.ai/api/v1/chat/completions",
            api_key_env="OPENROUTER_API_KEY",
            description=description,
            display_name=display_name,
            icon=icon,
            created=model_data.get("created")  # Use real OpenRouter creation timestamp
        )
    except Exception as e:
        print(f"Error creating model config for {model_data.get('id', 'unknown')}: {e}")
        return None

# Popular models to always include
POPULAR_MODELS = [
    "anthropic/claude-3.5-sonnet",
    "deepseek/deepseek-chat-v3.1:free",
    "deepseek/deepseek-r1",
    "openai/gpt-4o",
    "openai/o3-mini",
    "meta-llama/llama-3.1-8b-instruct",
    "qwen/qwen-2.5-72b-instruct",
    "google/gemma-2-27b-it",
    "mistralai/mistral-7b-instruct",
    "anthropic/claude-3-haiku",
    "openai/gpt-3.5-turbo",
    "meta-llama/llama-3.1-70b-instruct"
]

async def get_dynamic_models() -> Dict[str, ModelConfig]:
    """Get dynamic models from OpenRouter API"""
    try:
        openrouter_models = await fetch_openrouter_models()
        dynamic_models = {}
        
        for model_data in openrouter_models:
            model_config = create_model_config_from_openrouter(model_data)
            if model_config:
                dynamic_models[model_config.name] = model_config
        
        return dynamic_models
    except Exception as e:
        print(f"Error getting dynamic models: {e}")
        return {}

def get_models_by_provider(provider: ModelProvider) -> Dict[str, ModelConfig]:
    """Get all models from a specific provider"""
    return {
        name: config for name, config in AVAILABLE_MODELS.items()
        if config.provider == provider and config.enabled
    }

def get_models_by_tier(tier: ModelTier) -> Dict[str, ModelConfig]:
    """Get all models in a specific tier"""
    return {
        name: config for name, config in AVAILABLE_MODELS.items()
        if config.tier == tier and config.enabled
    }

def get_models_by_capability(capability: ModelCapability) -> Dict[str, ModelConfig]:
    """Get all models with a specific capability"""
    return {
        name: config for name, config in AVAILABLE_MODELS.items()
        if capability in config.capabilities and config.enabled
    }

def get_model_config(model_name: str) -> Optional[ModelConfig]:
    """Get configuration for a specific model"""
    return AVAILABLE_MODELS.get(model_name)

def list_available_models() -> List[str]:
    """Get list of all available model names"""
    return [name for name, config in AVAILABLE_MODELS.items() if config.enabled]

def get_fastest_models(max_latency_ms: int = 1000) -> Dict[str, ModelConfig]:
    """Get models with latency below threshold"""
    return {
        name: config for name, config in AVAILABLE_MODELS.items()
        if config.metrics.avg_latency_ms <= max_latency_ms and config.enabled
    }

def get_cheapest_models(max_cost: float = 0.001) -> Dict[str, ModelConfig]:
    """Get models with cost below threshold"""
    return {
        name: config for name, config in AVAILABLE_MODELS.items()
        if config.metrics.cost_per_1k_tokens <= max_cost and config.enabled
    }

def get_default_hecate_model() -> str:
    """Get the default model for Hecate Agent (prioritizes free/cheap models)"""
    # Always return DeepSeek Chat v3.1 Free as default
    return "deepseek/deepseek-chat-v3.1:free"
    cheap_models = get_cheapest_models(0.001)
    if cheap_models:
        return list(cheap_models.keys())[0]
    
    # Final fallback to any available model
    available = list_available_models()
    if available:
        return available[0]
    
    return "gpt-3.5-turbo"  # Ultimate fallback

def get_reasoning_models() -> Dict[str, ModelConfig]:
    """Get all models that support reasoning tokens"""
    return {
        name: config for name, config in AVAILABLE_MODELS.items()
        if config.supports_reasoning and config.enabled
    }

def get_default_reasoning_model() -> Optional[str]:
    """Get the default reasoning model (prioritizes free reasoning models)"""
    reasoning_models = get_reasoning_models()
    
    # Prioritize DeepSeek-R1 if available
    if "deepseek/deepseek-r1" in reasoning_models:
        return "deepseek/deepseek-r1"
    
    # Fallback to any reasoning model
    if reasoning_models:
        return list(reasoning_models.keys())[0]
    
    return None

def get_quality_models(min_quality: float = 0.9) -> Dict[str, ModelConfig]:
    """Get models with quality score above threshold"""
    return {
        name: config for name, config in AVAILABLE_MODELS.items()
        if config.metrics.quality_score >= min_quality and config.enabled
    }