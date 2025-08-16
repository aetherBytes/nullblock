"""
Model Router

Intelligent routing system that selects the best LLM model based on task requirements,
performance constraints, and cost considerations.
"""

import logging
from typing import Dict, List, Optional, Tuple
from dataclasses import dataclass
from enum import Enum

from .models import ModelConfig, ModelTier, ModelCapability, AVAILABLE_MODELS

logger = logging.getLogger(__name__)

class Priority(Enum):
    """Task priority levels"""
    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"
    CRITICAL = "critical"

class OptimizationGoal(Enum):
    """Optimization goals for model selection"""
    QUALITY = "quality"         # Best possible quality
    SPEED = "speed"            # Fastest response
    COST = "cost"              # Lowest cost
    BALANCED = "balanced"      # Balance of quality/speed/cost
    RELIABILITY = "reliability" # Most reliable model

@dataclass
class TaskRequirements:
    """Requirements and constraints for a task"""
    # Required capabilities
    required_capabilities: List[ModelCapability]
    
    # Performance constraints
    max_latency_ms: Optional[int] = None
    max_cost_per_request: Optional[float] = None
    min_quality_score: Optional[float] = None
    min_reliability_score: Optional[float] = None
    
    # Task characteristics
    priority: Priority = Priority.MEDIUM
    optimization_goal: OptimizationGoal = OptimizationGoal.BALANCED
    context_length: int = 2000
    expected_output_length: int = 500
    
    # Preferences
    preferred_providers: Optional[List[str]] = None
    excluded_models: Optional[List[str]] = None
    allow_local_models: bool = True
    
    # Context
    task_type: str = "general"
    user_id: Optional[str] = None
    session_id: Optional[str] = None

@dataclass
class RoutingDecision:
    """Result of model routing decision"""
    selected_model: str
    model_config: ModelConfig
    confidence: float           # 0-1, confidence in this choice
    reasoning: List[str]        # Human-readable reasoning
    alternatives: List[str]     # Alternative models considered
    estimated_cost: float       # Estimated cost for this request
    estimated_latency_ms: float # Estimated latency
    fallback_models: List[str]  # Fallback options if primary fails

class ModelRouter:
    """
    Intelligent model selection router
    """
    
    def __init__(self, enable_fallbacks: bool = True, enable_learning: bool = True):
        self.enable_fallbacks = enable_fallbacks
        self.enable_learning = enable_learning
        self.usage_history: Dict[str, List[Dict]] = {}  # Track usage for learning
        self.model_status: Dict[str, bool] = {}  # Track model availability
        
        # Initialize all models as available
        for model_name in AVAILABLE_MODELS.keys():
            self.model_status[model_name] = True
        
        logger.info("ModelRouter initialized")
    
    async def route_request(self, requirements: TaskRequirements) -> RoutingDecision:
        """
        Route a request to the best available model
        
        Args:
            requirements: Task requirements and constraints
            
        Returns:
            RoutingDecision with selected model and reasoning
        """
        try:
            # Step 1: Filter models by requirements
            candidate_models = self._filter_by_requirements(requirements)
            
            if not candidate_models:
                raise ValueError("No models meet the specified requirements")
            
            # Step 2: Score and rank models
            scored_models = self._score_models(candidate_models, requirements)
            
            # Step 3: Select best model
            selected_model_name = max(scored_models.keys(), key=lambda k: scored_models[k]['score'])
            selected_config = AVAILABLE_MODELS[selected_model_name]
            
            # Step 4: Generate reasoning
            reasoning = self._generate_reasoning(selected_model_name, requirements, scored_models)
            
            # Step 5: Select alternatives and fallbacks
            alternatives = self._select_alternatives(scored_models, selected_model_name, top_n=3)
            fallbacks = self._select_fallbacks(requirements, exclude=[selected_model_name])
            
            # Step 6: Estimate cost and latency
            estimated_cost = self._estimate_cost(selected_config, requirements)
            estimated_latency = self._estimate_latency(selected_config, requirements)
            
            decision = RoutingDecision(
                selected_model=selected_model_name,
                model_config=selected_config,
                confidence=scored_models[selected_model_name]['confidence'],
                reasoning=reasoning,
                alternatives=alternatives,
                estimated_cost=estimated_cost,
                estimated_latency_ms=estimated_latency,
                fallback_models=fallbacks
            )
            
            # Log decision for learning
            if self.enable_learning:
                self._log_decision(decision, requirements)
            
            return decision
            
        except Exception as e:
            logger.error(f"Error in model routing: {e}")
            # Return emergency fallback
            return self._emergency_fallback(requirements)
    
    def _filter_by_requirements(self, requirements: TaskRequirements) -> Dict[str, ModelConfig]:
        """Filter models based on hard requirements"""
        candidate_models = {}
        
        for model_name, config in AVAILABLE_MODELS.items():
            # Skip if model is disabled or unavailable
            if not config.enabled or not self.model_status.get(model_name, True):
                continue
            
            # Skip if model is explicitly excluded
            if requirements.excluded_models and model_name in requirements.excluded_models:
                continue
            
            # Skip local models if not allowed
            if not requirements.allow_local_models and config.tier == ModelTier.LOCAL:
                continue
            
            # Check provider preferences
            if (requirements.preferred_providers and 
                config.provider.value not in requirements.preferred_providers):
                continue
            
            # Check required capabilities
            if not all(cap in config.capabilities for cap in requirements.required_capabilities):
                continue
            
            # Check hard constraints
            if (requirements.max_latency_ms and 
                config.metrics.avg_latency_ms > requirements.max_latency_ms):
                continue
            
            if (requirements.min_quality_score and 
                config.metrics.quality_score < requirements.min_quality_score):
                continue
            
            if (requirements.min_reliability_score and 
                config.metrics.reliability_score < requirements.min_reliability_score):
                continue
            
            # Check context window
            if config.metrics.context_window < requirements.context_length:
                continue
            
            candidate_models[model_name] = config
        
        return candidate_models
    
    def _score_models(self, models: Dict[str, ModelConfig], requirements: TaskRequirements) -> Dict[str, Dict]:
        """Score models based on requirements and optimization goals"""
        scored_models = {}
        
        for model_name, config in models.items():
            score_components = {}
            
            # Quality score (0-1)
            score_components['quality'] = config.metrics.quality_score
            
            # Speed score (inverse of latency, normalized)
            max_latency = max(m.metrics.avg_latency_ms for m in models.values())
            speed_score = 1.0 - (config.metrics.avg_latency_ms / max_latency)
            score_components['speed'] = speed_score
            
            # Cost score (inverse of cost, normalized)
            max_cost = max(m.metrics.cost_per_1k_tokens for m in models.values())
            if max_cost > 0:
                cost_score = 1.0 - (config.metrics.cost_per_1k_tokens / max_cost)
            else:
                cost_score = 1.0  # Free models get perfect cost score
            score_components['cost'] = cost_score
            
            # Reliability score (0-1)
            score_components['reliability'] = config.metrics.reliability_score
            
            # Capability match score
            required_caps = set(requirements.required_capabilities)
            model_caps = set(config.capabilities)
            capability_score = len(required_caps.intersection(model_caps)) / len(required_caps)
            score_components['capability'] = capability_score
            
            # Priority adjustment
            priority_multiplier = {
                Priority.LOW: 0.8,
                Priority.MEDIUM: 1.0,
                Priority.HIGH: 1.2,
                Priority.CRITICAL: 1.5
            }[requirements.priority]
            
            # Weighted final score based on optimization goal
            if requirements.optimization_goal == OptimizationGoal.QUALITY:
                final_score = (
                    score_components['quality'] * 0.6 +
                    score_components['reliability'] * 0.2 +
                    score_components['capability'] * 0.2
                )
            elif requirements.optimization_goal == OptimizationGoal.SPEED:
                final_score = (
                    score_components['speed'] * 0.6 +
                    score_components['capability'] * 0.3 +
                    score_components['reliability'] * 0.1
                )
            elif requirements.optimization_goal == OptimizationGoal.COST:
                final_score = (
                    score_components['cost'] * 0.6 +
                    score_components['capability'] * 0.3 +
                    score_components['reliability'] * 0.1
                )
            elif requirements.optimization_goal == OptimizationGoal.RELIABILITY:
                final_score = (
                    score_components['reliability'] * 0.6 +
                    score_components['quality'] * 0.2 +
                    score_components['capability'] * 0.2
                )
            else:  # BALANCED
                final_score = (
                    score_components['quality'] * 0.3 +
                    score_components['speed'] * 0.25 +
                    score_components['cost'] * 0.25 +
                    score_components['reliability'] * 0.2
                )
            
            final_score *= priority_multiplier
            
            # Calculate confidence based on score distribution
            confidence = min(0.95, final_score * 0.8 + 0.2)
            
            scored_models[model_name] = {
                'score': final_score,
                'confidence': confidence,
                'components': score_components
            }
        
        return scored_models
    
    def _generate_reasoning(self, selected_model: str, requirements: TaskRequirements, 
                          scored_models: Dict[str, Dict]) -> List[str]:
        """Generate human-readable reasoning for the selection"""
        reasoning = []
        
        config = AVAILABLE_MODELS[selected_model]
        score_info = scored_models[selected_model]
        
        # Primary selection reason
        if requirements.optimization_goal == OptimizationGoal.QUALITY:
            reasoning.append(f"Selected {selected_model} for highest quality (score: {config.metrics.quality_score:.2f})")
        elif requirements.optimization_goal == OptimizationGoal.SPEED:
            reasoning.append(f"Selected {selected_model} for speed ({config.metrics.avg_latency_ms}ms latency)")
        elif requirements.optimization_goal == OptimizationGoal.COST:
            reasoning.append(f"Selected {selected_model} for cost efficiency (${config.metrics.cost_per_1k_tokens:.4f}/1K tokens)")
        else:
            reasoning.append(f"Selected {selected_model} for balanced performance (score: {score_info['score']:.2f})")
        
        # Capability matching
        matched_caps = len(set(requirements.required_capabilities).intersection(set(config.capabilities)))
        reasoning.append(f"Matches {matched_caps}/{len(requirements.required_capabilities)} required capabilities")
        
        # Performance characteristics
        if config.metrics.avg_latency_ms < 1000:
            reasoning.append("Fast response time")
        if config.metrics.cost_per_1k_tokens < 0.001:
            reasoning.append("Low cost")
        if config.metrics.quality_score > 0.9:
            reasoning.append("High quality model")
        if config.tier == ModelTier.LOCAL:
            reasoning.append("Local model for privacy")
        
        return reasoning
    
    def _select_alternatives(self, scored_models: Dict[str, Dict], 
                           selected_model: str, top_n: int = 3) -> List[str]:
        """Select alternative models"""
        # Sort by score, excluding selected model
        alternatives = sorted(
            [(name, info['score']) for name, info in scored_models.items() if name != selected_model],
            key=lambda x: x[1],
            reverse=True
        )
        
        return [name for name, _ in alternatives[:top_n]]
    
    def _select_fallbacks(self, requirements: TaskRequirements, 
                         exclude: List[str] = None) -> List[str]:
        """Select fallback models in case of failures"""
        if not self.enable_fallbacks:
            return []
        
        exclude = exclude or []
        fallbacks = []
        
        # Always include a fast, reliable model as fallback
        fast_models = [
            name for name, config in AVAILABLE_MODELS.items()
            if (config.metrics.avg_latency_ms < 1000 and 
                config.metrics.reliability_score > 0.9 and
                name not in exclude and
                config.enabled)
        ]
        
        if fast_models:
            fallbacks.append(fast_models[0])
        
        # Include a local model if available and allowed
        if requirements.allow_local_models:
            local_models = [
                name for name, config in AVAILABLE_MODELS.items()
                if (config.tier == ModelTier.LOCAL and
                    name not in exclude and
                    config.enabled)
            ]
            if local_models:
                fallbacks.append(local_models[0])
        
        return fallbacks[:2]  # Limit to 2 fallbacks
    
    def _estimate_cost(self, config: ModelConfig, requirements: TaskRequirements) -> float:
        """Estimate cost for the request"""
        # Estimate total tokens (input + output)
        estimated_tokens = requirements.context_length + requirements.expected_output_length
        cost_per_token = config.metrics.cost_per_1k_tokens / 1000
        return estimated_tokens * cost_per_token
    
    def _estimate_latency(self, config: ModelConfig, requirements: TaskRequirements) -> float:
        """Estimate latency for the request"""
        base_latency = config.metrics.avg_latency_ms
        
        # Adjust for output length
        tokens_per_second = config.metrics.tokens_per_second
        if tokens_per_second > 0:
            generation_time = (requirements.expected_output_length / tokens_per_second) * 1000
            return base_latency + generation_time
        
        return base_latency
    
    def _log_decision(self, decision: RoutingDecision, requirements: TaskRequirements):
        """Log decision for learning and analytics"""
        if decision.selected_model not in self.usage_history:
            self.usage_history[decision.selected_model] = []
        
        log_entry = {
            'timestamp': logger.handlers[0].formatter.formatTime if logger.handlers else None,
            'task_type': requirements.task_type,
            'optimization_goal': requirements.optimization_goal.value,
            'estimated_cost': decision.estimated_cost,
            'estimated_latency': decision.estimated_latency_ms,
            'confidence': decision.confidence
        }
        
        self.usage_history[decision.selected_model].append(log_entry)
        
        # Keep only last 100 entries per model
        if len(self.usage_history[decision.selected_model]) > 100:
            self.usage_history[decision.selected_model] = self.usage_history[decision.selected_model][-100:]
    
    def _emergency_fallback(self, requirements: TaskRequirements) -> RoutingDecision:
        """Emergency fallback when routing fails"""
        # Try to find any working model
        for model_name, config in AVAILABLE_MODELS.items():
            if config.enabled and self.model_status.get(model_name, True):
                return RoutingDecision(
                    selected_model=model_name,
                    model_config=config,
                    confidence=0.1,
                    reasoning=["Emergency fallback - routing failed"],
                    alternatives=[],
                    estimated_cost=0.0,
                    estimated_latency_ms=config.metrics.avg_latency_ms,
                    fallback_models=[]
                )
        
        # If no models available, raise error
        raise RuntimeError("No LLM models available")
    
    def update_model_status(self, model_name: str, available: bool):
        """Update availability status of a model"""
        self.model_status[model_name] = available
        logger.info(f"Model {model_name} status updated to: {'available' if available else 'unavailable'}")
    
    def get_usage_stats(self) -> Dict[str, Dict]:
        """Get usage statistics for all models"""
        stats = {}
        
        for model_name, history in self.usage_history.items():
            if history:
                total_requests = len(history)
                avg_confidence = sum(entry['confidence'] for entry in history) / total_requests
                total_cost = sum(entry['estimated_cost'] for entry in history)
                
                stats[model_name] = {
                    'total_requests': total_requests,
                    'avg_confidence': avg_confidence,
                    'total_estimated_cost': total_cost,
                    'last_used': history[-1]['timestamp'] if history else None
                }
        
        return stats