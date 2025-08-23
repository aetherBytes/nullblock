"""
LLM Model Definitions and Configurations

Defines available LLM models, their capabilities, and configuration parameters.
"""

from enum import Enum
from typing import Dict, List, Optional
from dataclasses import dataclass

class ModelProvider(Enum):
    """Available LLM providers"""
    OPENAI = "openai"
    ANTHROPIC = "anthropic"
    HUGGINGFACE = "huggingface"
    LOCAL = "local"
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
    
    # LM Studio Models (Local)
    "gemma-3-270m-it-mlx": ModelConfig(
        name="gemma-3-270m-it-mlx",
        provider=ModelProvider.LOCAL,
        tier=ModelTier.LOCAL,
        capabilities=[
            ModelCapability.CONVERSATION,
            ModelCapability.REASONING,
            ModelCapability.CODE
        ],
        metrics=ModelMetrics(
            avg_latency_ms=800,
            tokens_per_second=30,
            cost_per_1k_tokens=0.0,  # No cost for local
            context_window=4096,
            quality_score=0.75,  # Higher quality score than Ollama models
            reliability_score=0.95  # Higher reliability score
        ),
        api_endpoint="http://localhost:1234/v1/chat/completions",
        description="Local Gemma3 270M model via LM Studio - optimized for development"
    ),
    
    "openai/gpt-oss-20b": ModelConfig(
        name="openai/gpt-oss-20b",
        provider=ModelProvider.LOCAL,
        tier=ModelTier.STANDARD,
        capabilities=[
            ModelCapability.CONVERSATION,
            ModelCapability.REASONING,
            ModelCapability.CODE,
            ModelCapability.CREATIVE,
            ModelCapability.DATA_ANALYSIS
        ],
        metrics=ModelMetrics(
            avg_latency_ms=1200,
            tokens_per_second=20,
            cost_per_1k_tokens=0.0,
            context_window=8192,
            quality_score=0.88,
            reliability_score=0.92
        ),
        api_endpoint="http://localhost:1234/v1/chat/completions",
        description="GPT-OSS 20B model via LM Studio - open-source GPT alternative"
    ),
    
    "qwen/qwen3-4b-thinking-2507": ModelConfig(
        name="qwen/qwen3-4b-thinking-2507",
        provider=ModelProvider.LOCAL,
        tier=ModelTier.STANDARD,
        capabilities=[
            ModelCapability.REASONING,
            ModelCapability.MATH,
            ModelCapability.CODE,
            ModelCapability.DATA_ANALYSIS,
            ModelCapability.LONG_CONTEXT
        ],
        metrics=ModelMetrics(
            avg_latency_ms=1000,
            tokens_per_second=25,
            cost_per_1k_tokens=0.0,
            context_window=32768,
            quality_score=0.85,
            reliability_score=0.92
        ),
        api_endpoint="http://localhost:1234/v1/chat/completions",
        description="Qwen3 4B thinking model via LM Studio - advanced reasoning capabilities"
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

def get_quality_models(min_quality: float = 0.9) -> Dict[str, ModelConfig]:
    """Get models with quality score above threshold"""
    return {
        name: config for name, config in AVAILABLE_MODELS.items()
        if config.metrics.quality_score >= min_quality and config.enabled
    }