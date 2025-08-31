"""
Hecate Agent HTTP Server

Simple FastAPI server wrapper for the Hecate agent to enable frontend integration.
"""

import asyncio
import difflib
import logging
import subprocess
import time
import json
from datetime import datetime
from typing import Dict, List, Any, Optional
from fastapi import FastAPI, HTTPException, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse
from pydantic import BaseModel
import uvicorn
from pathlib import Path
import os

from .main import HecateAgent, ConversationMessage
from ..logging_config import setup_agent_logging, log_agent_startup, log_agent_shutdown
from ..config import get_hecate_config, config
from ..llm_service.models import get_dynamic_models, POPULAR_MODELS, ModelCapability, AVAILABLE_MODELS, get_default_hecate_model

# Load environment variables from .env.dev file
def load_env_file():
    """Load environment variables from .env.dev file"""
    # Look for .env.dev in the project root (2 levels up from agents directory)
    # Path: svc/nullblock-agents/src/agents/hecate/server.py -> project root
    env_path = Path(__file__).parent.parent.parent.parent.parent.parent / '.env.dev'
    
    if env_path.exists():
        print(f"📝 Loading environment from: {env_path}")
        with open(env_path) as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith('#') and '=' in line:
                    key, value = line.split('=', 1)
                    # Remove quotes if present
                    value = value.strip('"').strip("'")
                    os.environ[key] = value
        print("✅ Environment variables loaded successfully")
    else:
        print(f"⚠️  .env.dev file not found at: {env_path}")

# Load environment variables
load_env_file()

logger = setup_agent_logging("hecate-server", "INFO", enable_file_logging=True)

# Cache for models endpoint to reduce logging frequency
_models_cache = None
_models_cache_timestamp = 0
_models_log_timestamp = 0
MODELS_CACHE_TTL = 30  # Cache for 30 seconds
MODELS_LOG_INTERVAL = 300  # Log only every 5 minutes (300 seconds)

# Enhanced logging utility for the server
def log_request(request: Request, message: str, data: Any = None):
    """Log request details with timestamp and context"""
    timestamp = datetime.now().isoformat()
    client_ip = request.client.host if request.client else "unknown"
    user_agent = request.headers.get("user-agent", "unknown")
    
    log_data = {
        "timestamp": timestamp,
        "method": request.method,
        "url": str(request.url),
        "client_ip": client_ip,
        "user_agent": user_agent[:100],  # Truncate long user agents
        "message": message
    }
    
    if data:
        log_data["data"] = data
    
    logger.info(f"🌐 [{timestamp}] {message}", log_data)

def log_response(status_code: int, message: str, data: Any = None):
    """Log response details"""
    timestamp = datetime.now().isoformat()
    emoji = "✅" if status_code < 400 else "❌" if status_code >= 500 else "⚠️"
    
    log_data = {
        "timestamp": timestamp,
        "status_code": status_code,
        "message": message
    }
    
    if data:
        log_data["data"] = data
    
    logger.info(f"{emoji} [{timestamp}] {message}", log_data)

# Pydantic models for API
class ChatRequest(BaseModel):
    message: str
    user_context: Optional[Dict[str, Any]] = None

class PersonalityRequest(BaseModel):
    personality: str

class ModelSelectionRequest(BaseModel):
    model_name: str

class ChatMessageResponse(BaseModel):
    id: str
    timestamp: str
    role: str
    content: str
    model_used: Optional[str] = None
    metadata: Optional[Dict[str, Any]] = None

# Global agent instance
agent: Optional[HecateAgent] = None

def create_app() -> FastAPI:
    """Create and configure FastAPI application"""
    app = FastAPI(
        title="Hecate Agent API",
        description="HTTP API for Hecate conversational agent",
        version="1.0.0"
    )
    
    # Add CORS middleware
    app.add_middleware(
        CORSMiddleware,
        allow_origins=["http://localhost:5173", "http://localhost:3000"],  # Vite dev server and production
        allow_credentials=True,
        allow_methods=["*"],
        allow_headers=["*"],
    )
    
    # Add request logging middleware
    @app.middleware("http")
    async def log_requests(request: Request, call_next):
        start_time = time.time()
        
        # Log incoming request
        log_request(request, f"📥 {request.method} {request.url.path}")
        
        # Process request
        response = await call_next(request)
        
        # Calculate duration
        duration = time.time() - start_time
        
        # Log response
        log_response(
            response.status_code, 
            f"📤 {request.method} {request.url.path} ({duration:.2f}s)"
        )
        
        return response
    
    @app.on_event("startup")
    async def startup_event():
        """Initialize agent on startup"""
        global agent
        try:
            logger.info("🚀 Starting Hecate Agent HTTP API Server...")
            logger.info("🔗 CORS enabled for: http://localhost:5173, http://localhost:3000")
            logger.info("📋 API Documentation: http://localhost:8001/docs")
            logger.info("🔍 Health Check: http://localhost:8001/health")
            
            log_agent_startup(logger, "hecate-server", "1.0.0")
            
            logger.info("🤖 Initializing Hecate Agent...")
            agent = HecateAgent()
            await agent.start()
            
            logger.info("✅ Hecate Agent initialized successfully")
            logger.info("🎯 HTTP API ready on port 8001")
            logger.info("=" * 80)
            
        except Exception as e:
            logger.error(f"❌ Failed to start Hecate Agent: {e}")
            logger.error("🔍 Full error details:", exc_info=True)
            raise
    
    @app.on_event("shutdown")
    async def shutdown_event():
        """Cleanup agent on shutdown"""
        global agent
        if agent:
            logger.info("🛑 Shutting down Hecate HTTP API Server...")
            try:
                await agent.stop()
                logger.info("✅ Agent stopped successfully")
            except Exception as e:
                logger.error(f"❌ Error stopping agent: {e}")
            
            log_agent_shutdown(logger, "hecate-server")
    
    @app.get("/health")
    async def health_check(request: Request):
        """Health check endpoint"""
        log_request(request, "🏥 Health check requested")
        
        if not agent or not agent.running:
            log_response(503, "🏥 Health check failed - agent not running")
            raise HTTPException(status_code=503, detail="Agent not running")
        
        try:
            # Get agent status
            agent_status = {
                "agent_running": agent.running,
                "personality": agent.personality,
                "conversation_length": len(agent.conversation_history),
                "model_status": await agent.get_model_status() if hasattr(agent, 'get_model_status') else "unknown"
            }
            
            log_response(200, "🏥 Health check successful", agent_status)
            
            return {
                "status": "healthy",
                "timestamp": datetime.now().isoformat(),
                "agent": agent_status
            }
        except Exception as e:
            logger.error(f"❌ Health check failed: {e}")
            log_response(503, "🏥 Health check failed", {"error": str(e)})
            raise HTTPException(status_code=503, detail=str(e))
    
    @app.post("/chat")
    async def chat_endpoint(request: Request, chat_request: ChatRequest):
        """Send message to Hecate agent"""
        if not agent:
            log_request(request, "🚫 Chat request rejected - agent not initialized")
            log_response(503, "🚫 Chat request failed - agent not initialized")
            raise HTTPException(status_code=503, detail="Agent not initialized")
        
        try:
            start_time = time.time()
            
            log_request(request, "💬 Chat request received", {
                "message_length": len(chat_request.message),
                "has_user_context": bool(chat_request.user_context)
            })
            
            # Process chat request
            response = await agent.chat(chat_request.message, chat_request.user_context)
            
            # Calculate processing time
            processing_time = time.time() - start_time
            
            response_data = {
                "content": response.content,
                "model_used": response.model_used,
                "latency_ms": response.latency_ms,
                "confidence_score": response.confidence_score,
                "metadata": response.metadata,
                "processing_time_s": processing_time
            }
            
            log_response(200, "💬 Chat response sent successfully", {
                "response_length": len(response.content),
                "processing_time_s": processing_time,
                "model_used": response.model_used,
                "confidence_score": response.confidence_score
            })
            
            return response_data
            
        except Exception as e:
            logger.error(f"❌ Chat request failed: {e}")
            log_response(500, "💬 Chat request failed", {"error": str(e)})
            raise HTTPException(status_code=500, detail=str(e))
    
    @app.get("/model-status")
    async def model_status(request: Request):
        """Get current model status"""
        if not agent:
            log_request(request, "🚫 Model status request rejected - agent not initialized")
            log_response(503, "🚫 Model status request failed - agent not initialized")
            raise HTTPException(status_code=503, detail="Agent not initialized")
        
        try:
            log_request(request, "🔍 Model status requested")
            
            status = await agent.get_model_status()
            
            log_response(200, "🔍 Model status retrieved successfully", status)
            
            return status
        except Exception as e:
            logger.error(f"❌ Model status request failed: {e}")
            log_response(500, "🔍 Model status request failed", {"error": str(e)})
            raise HTTPException(status_code=500, detail=str(e))
    
    @app.get("/search-models")
    async def search_models(request: Request, q: str = "", category: str = "", limit: int = 20):
        """Search for models by name, description, or category (free, fast, premium, thinkers)"""
        if not agent:
            log_request(request, "🚫 Model search rejected - agent not initialized")
            log_response(503, "🚫 Model search failed - agent not initialized")
            raise HTTPException(status_code=503, detail="Agent not initialized")
        
        try:
            log_request(request, "🔍 Model search requested", {"query": q, "category": category, "limit": limit})
            
            # Get available models (same data as /available-models endpoint)
            available_models = await get_dynamic_models()
            
            # Filter to only available models based on API keys
            filtered_models = {}
            for model_name, config in available_models.items():
                if agent.is_model_available(model_name):
                    filtered_models[model_name] = config
            
            # Category mapping
            def get_model_category(config):
                tier = getattr(config, 'tier', None)
                cost = getattr(config.metrics, 'cost_per_1k_tokens', 0)
                supports_reasoning = getattr(config, 'supports_reasoning', False)
                
                categories = []
                if cost == 0:
                    categories.append("free")
                if tier and tier.value == "fast":
                    categories.append("fast")
                if cost > 0.00001:  # Higher cost models are premium
                    categories.append("premium") 
                if supports_reasoning:
                    categories.append("thinkers")
                    
                return categories
            
            # Apply category filter if specified
            if category:
                category_lower = category.lower()
                temp_filtered = {}
                for model_name, config in filtered_models.items():
                    model_categories = get_model_category(config)
                    if category_lower in model_categories:
                        temp_filtered[model_name] = config
                filtered_models = temp_filtered
            
            # Convert to list for easier processing
            model_list = []
            for model_name, config in filtered_models.items():
                display_name = getattr(config, 'display_name', config.name)
                model_categories = get_model_category(config)
                
                model_data = {
                    "name": model_name,
                    "display_name": display_name,
                    "icon": getattr(config, 'icon', '🤖'),
                    "provider": config.provider.value,
                    "available": True,
                    "tier": config.tier.value,
                    "context_length": config.metrics.context_window,
                    "capabilities": [cap.value for cap in config.capabilities],
                    "cost_per_1k_tokens": config.metrics.cost_per_1k_tokens,
                    "supports_reasoning": getattr(config, 'supports_reasoning', False),
                    "description": config.description,
                    "is_popular": getattr(config, 'is_popular', False),
                    "categories": model_categories
                }
                model_list.append(model_data)
            
            # If no query, return models sorted by popularity and name
            if not q.strip():
                search_results = sorted(model_list, key=lambda x: (-x["is_popular"], x["name"]))[:limit]
            else:
                # Use improved search logic
                query_lower = q.lower().strip()
                search_results = []
                
                for model in model_list:
                    # Calculate relevance score  
                    score = 0
                    model_name_lower = model["name"].lower()
                    display_name_lower = model["display_name"].lower()
                    description_lower = model["description"].lower()
                    
                    # Exact matches get highest score
                    if query_lower == model_name_lower or query_lower == display_name_lower:
                        score = 100
                    # Starts with match
                    elif model_name_lower.startswith(query_lower) or display_name_lower.startswith(query_lower):
                        score = 90
                    # Contains match - this should catch "claude" in "anthropic/claude-3-haiku"
                    elif query_lower in model_name_lower or query_lower in display_name_lower:
                        score = 80
                    # Description match
                    elif query_lower in description_lower:
                        score = 70
                    # Fuzzy matching for partial matches
                    else:
                        name_ratio = difflib.SequenceMatcher(None, query_lower, model_name_lower).ratio()
                        display_ratio = difflib.SequenceMatcher(None, query_lower, display_name_lower).ratio()
                        max_ratio = max(name_ratio, display_ratio)
                        
                        if max_ratio > 0.5:  # Lower threshold for broader matching
                            score = int(max_ratio * 60)
                        # Word-based matching in description
                        elif len(query_lower) > 2:
                            query_words = query_lower.split()
                            if any(word in description_lower for word in query_words if len(word) > 2):
                                score = 40
                    
                    # Boost popular models
                    if model.get("is_popular", False):
                        score += 5
                    
                    # Add to results if score is above threshold
                    if score > 0:
                        model_copy = model.copy()  # Make a copy to avoid modifying original
                        model_copy["_score"] = score
                        search_results.append(model_copy)
                
                # Sort by score (descending) then name
                search_results.sort(key=lambda x: (-x.get("_score", 0), x["name"]))
                
                # Remove score and limit results
                for result in search_results:
                    result.pop("_score", None)
                search_results = search_results[:limit]
            
            log_response(200, "🔍 Model search completed successfully", {
                "query": q,
                "results_count": len(search_results),
                "total_available": len(filtered_models)
            })
            
            return {
                "query": q,
                "results": search_results,
                "total_available": len(filtered_models)
            }
        except Exception as e:
            logger.error(f"❌ Model search failed: {e}")
            log_response(500, "🔍 Model search failed", {"error": str(e)})
            raise HTTPException(status_code=500, detail=str(e))

    @app.get("/available-models")
    async def get_available_models(request: Request):
        """Get list of available models with dynamic loading from OpenRouter (cached)"""
        global _models_cache, _models_cache_timestamp, _models_log_timestamp
        
        if not agent:
            log_request(request, "🚫 Available models rejected - agent not initialized")
            log_response(503, "🚫 Available models failed - agent not initialized")
            raise HTTPException(status_code=503, detail="Agent not initialized")
        
        try:
            current_time = time.time()
            
            # Check if we should log this request (throttle logging)
            should_log = current_time - _models_log_timestamp > MODELS_LOG_INTERVAL
            if should_log:
                log_request(request, "📋 Available models requested (throttled logging)")
                _models_log_timestamp = current_time
            
            # Check cache first
            if _models_cache and current_time - _models_cache_timestamp < MODELS_CACHE_TTL:
                if should_log:
                    log_response(200, "📋 Available models from cache", {
                        "cache_age_seconds": current_time - _models_cache_timestamp,
                        "models_count": len(_models_cache.get("models", []))
                    })
                return _models_cache
            
            available_models = []
            
            # Get static models
            
            for model_name, config in AVAILABLE_MODELS.items():
                # Check if model is truly available (API keys, etc.)
                is_truly_available = agent.is_model_available(model_name)
                
                # Only include models that are actually available
                if is_truly_available:
                    available_models.append({
                        "name": model_name,
                        "display_name": getattr(config, 'display_name', config.name),
                        "icon": getattr(config, 'icon', '🤖'),
                        "provider": config.provider.value,
                        "available": True,
                        "tier": config.tier.value,
                        "context_length": config.metrics.context_window,
                        "capabilities": [cap.value for cap in config.capabilities],
                        "cost_per_1k_tokens": config.metrics.cost_per_1k_tokens,
                        "supports_reasoning": getattr(config, 'supports_reasoning', False) or ModelCapability.REASONING in config.capabilities,
                        "description": config.description,
                        "is_popular": model_name in POPULAR_MODELS
                    })
            
            # Get dynamic models from OpenRouter (popular ones first)
            dynamic_models = await get_dynamic_models()
            
            for model_name, config in dynamic_models.items():
                # Check if model is truly available (API keys, etc.)
                is_truly_available = agent.is_model_available(model_name)
                
                # Only include models that are actually available
                if is_truly_available:
                    available_models.append({
                        "name": model_name,
                        "display_name": getattr(config, 'display_name', config.name),
                        "icon": getattr(config, 'icon', '🤖'),
                        "provider": config.provider.value,
                        "available": True,
                        "tier": config.tier.value,
                        "context_length": config.metrics.context_window,
                        "capabilities": [cap.value for cap in config.capabilities],
                        "cost_per_1k_tokens": config.metrics.cost_per_1k_tokens,
                        "supports_reasoning": getattr(config, 'supports_reasoning', False) or ModelCapability.REASONING in config.capabilities,
                        "description": config.description,
                        "is_popular": model_name in POPULAR_MODELS
                    })
            
            # Sort by popularity first, then availability, then provider and name
            available_models.sort(key=lambda x: (
                not x["is_popular"],
                not x["available"], 
                x["provider"], 
                x["name"]
            ))
            
            # Get default model
            default_model = get_default_hecate_model()
            
            # Create response
            response_data = {
                "models": available_models,
                "current_model": getattr(agent, 'preferred_model', None),
                "default_model": default_model,
                "recommended_models": {
                    "free": "deepseek/deepseek-chat-v3.1:free",
                    "reasoning": "deepseek/deepseek-r1",
                    "premium": "anthropic/claude-3.5-sonnet"
                },
                "total_models": len(available_models),
                "dynamic_models": len(dynamic_models)
            }
            
            # Cache the response
            _models_cache = response_data
            _models_cache_timestamp = current_time
            
            # Only log when throttling allows
            if should_log:
                log_response(200, "📋 Available models retrieved successfully", {
                    "models_count": len(available_models),
                    "available_count": sum(1 for m in available_models if m["available"]),
                    "dynamic_count": len(dynamic_models),
                    "default_model": default_model
                })
            
            return response_data
        except Exception as e:
            logger.error(f"❌ Get available models failed: {e}")
            log_response(500, "📋 Get available models failed", {"error": str(e)})
            raise HTTPException(status_code=500, detail=str(e))

    @app.post("/refresh-models")
    async def refresh_models(request: Request):
        """Refresh model availability status"""
        if not agent:
            log_request(request, "🚫 Model refresh rejected - agent not initialized")
            log_response(503, "🚫 Model refresh failed - agent not initialized")
            raise HTTPException(status_code=503, detail="Agent not initialized")
        
        try:
            log_request(request, "🔄 Model refresh requested")
            
            # Re-test local models and update availability
            if hasattr(agent, 'llm_factory') and agent.llm_factory:
                await agent.llm_factory._test_local_models()
                logger.info("✅ Model availability refreshed")
            
            # Get updated status
            status = await agent.get_model_status()
            
            log_response(200, "🔄 Model refresh completed successfully", status)
            
            return {
                "success": True,
                "message": "Model availability refreshed",
                "status": status
            }
        except Exception as e:
            logger.error(f"❌ Model refresh failed: {e}")
            log_response(500, "🔄 Model refresh failed", {"error": str(e)})
            raise HTTPException(status_code=500, detail=str(e))
    
    @app.post("/set-model")
    async def set_model(request: Request, model_request: ModelSelectionRequest):
        """Set preferred model for chat responses"""
        if not agent:
            log_request(request, "🚫 Model selection rejected - agent not initialized")
            log_response(503, "🚫 Model selection failed - agent not initialized")
            raise HTTPException(status_code=503, detail="Agent not initialized")
        
        try:
            log_request(request, "🎯 Model selection requested", {
                "model_name": model_request.model_name
            })
            
            old_model = agent.get_preferred_model()
            success = await agent.set_preferred_model(model_request.model_name)
            
            if success:
                log_response(200, "🎯 Model selection successful", {
                    "old_model": old_model,
                    "new_model": model_request.model_name
                })
                
                return {
                    "success": True,
                    "model": model_request.model_name,
                    "previous_model": old_model
                }
            else:
                # Get more specific error message about why model is not available
                error_detail = agent.get_model_availability_reason(model_request.model_name)
                log_response(400, f"🎯 Model selection failed - {error_detail}", {
                    "requested_model": model_request.model_name
                })
                raise HTTPException(status_code=400, detail=error_detail)
                
        except HTTPException:
            raise
        except Exception as e:
            logger.error(f"❌ Set model failed: {e}")
            log_response(500, "🎯 Model selection failed", {"error": str(e)})
            raise HTTPException(status_code=500, detail=str(e))

    @app.get("/model-info")
    async def get_model_info(request: Request, model_name: str = None):
        """Get detailed information about a specific model or current model"""
        try:
            if not agent:
                log_response(503, "❌ Model info failed", {"error": "Agent not initialized"})
                raise HTTPException(status_code=503, detail="Agent not initialized")

            log_request(request, f"📋 Model info requested for: {model_name or 'current'}")

            # Use current model if no specific model requested
            if not model_name:
                model_name = getattr(agent, 'preferred_model', None)
                if not model_name:
                    return {"error": "No model currently loaded", "model_name": None}

            # Get model from static models first
            
            model_config = None
            is_dynamic = False
            
            if model_name in AVAILABLE_MODELS:
                model_config = AVAILABLE_MODELS[model_name]
            else:
                # Check dynamic models
                dynamic_models = await get_dynamic_models()
                if model_name in dynamic_models:
                    model_config = dynamic_models[model_name]
                    is_dynamic = True

            if not model_config:
                log_response(404, "❌ Model not found", {"model": model_name})
                raise HTTPException(status_code=404, detail=f"Model {model_name} not found")

            # Check if model is currently available
            is_available = agent.llm_factory.router.model_status.get(model_name, getattr(model_config, 'enabled', True))
            
            # Get current model status if this is the active model
            model_status = None
            llm_stats = None
            if model_name == getattr(agent, 'preferred_model', None):
                try:
                    status_info = await agent.get_model_status()
                    model_status = status_info.get('status', 'unknown')
                    llm_stats = status_info.get('stats', {})
                except Exception as e:
                    logger.warning(f"Failed to get model status: {e}")

            # Build detailed model information
            model_info = {
                "name": model_name,
                "display_name": getattr(model_config, 'display_name', getattr(model_config, 'name', model_name)),
                "icon": getattr(model_config, 'icon', '🤖'),
                "provider": model_config.provider.value if hasattr(model_config, 'provider') else 'unknown',
                "description": getattr(model_config, 'description', 'No description available'),
                "tier": model_config.tier.value if hasattr(model_config, 'tier') else 'standard',
                
                # Availability and status
                "available": is_available,
                "is_current": model_name == getattr(agent, 'preferred_model', None),
                "is_dynamic": is_dynamic,
                "status": model_status,
                
                # Technical specifications
                "context_length": getattr(model_config.metrics, 'context_window', 0) if hasattr(model_config, 'metrics') else 0,
                "max_tokens": getattr(model_config.metrics, 'max_output_tokens', 0) if hasattr(model_config, 'metrics') else 0,
                "capabilities": [cap.value for cap in model_config.capabilities] if hasattr(model_config, 'capabilities') else [],
                "supports_reasoning": getattr(model_config, 'supports_reasoning', False) or ModelCapability.REASONING in model_config.capabilities,
                "supports_vision": 'vision' in [cap.value for cap in getattr(model_config, 'capabilities', [])],
                "supports_function_calling": 'function_calling' in [cap.value for cap in getattr(model_config, 'capabilities', [])],
                
                # Cost information
                "cost_per_1k_tokens": getattr(model_config.metrics, 'cost_per_1k_tokens', 0.0) if hasattr(model_config, 'metrics') else 0.0,
                "cost_per_1m_tokens": (getattr(model_config.metrics, 'cost_per_1k_tokens', 0.0) * 1000) if hasattr(model_config, 'metrics') else 0.0,
                
                # Performance metrics from LLM factory if available
                "performance_stats": llm_stats or {},
                
                # Usage information
                "conversation_length": len(agent.conversation_history) if hasattr(agent, 'conversation_history') else 0,
                "last_used": None,  # Could be enhanced to track actual usage
            }
            
            # Add estimated costs for conversation if this is current model
            if model_info["is_current"] and model_info["conversation_length"] > 0:
                estimated_tokens = model_info["conversation_length"] * 100  # Rough estimate
                estimated_cost = (estimated_tokens / 1000) * model_info["cost_per_1k_tokens"]
                model_info["estimated_session_cost"] = estimated_cost
            
            log_response(200, "📋 Model info retrieved", {
                "model": model_name,
                "is_current": model_info["is_current"],
                "available": model_info["available"]
            })
            
            return model_info
            
        except HTTPException:
            raise
        except Exception as e:
            logger.error(f"❌ Get model info failed: {e}")
            log_response(500, "📋 Get model info failed", {"error": str(e)})
            raise HTTPException(status_code=500, detail=str(e))

    @app.post("/reset-models")
    async def reset_models(request: Request):
        """Reset and refresh model availability"""
        try:
            log_request(request, "🔄 Model reset requested")
            
            if hasattr(agent, 'llm_factory') and agent.llm_factory:
                # Re-initialize the factory to refresh model status
                await agent.llm_factory.initialize()
                logger.info("✅ Model factory reinitialized")
                
                # Get updated model status
                health = await agent.llm_factory.health_check()
                
                log_response(200, "🔄 Model reset completed successfully", health)
                
                return {
                    "success": True,
                    "message": "Models reset and refreshed successfully",
                    "health": health
                }
            else:
                raise HTTPException(status_code=503, detail="LLM factory not available")
                
        except Exception as e:
            logger.error(f"❌ Model reset failed: {e}")
            log_response(500, "🔄 Model reset failed", {"error": str(e)})
            raise HTTPException(status_code=500, detail=str(e))

    @app.post("/personality")
    async def set_personality(request: Request, personality_request: PersonalityRequest):
        """Set agent personality"""
        if not agent:
            log_request(request, "🚫 Personality change rejected - agent not initialized")
            log_response(503, "🚫 Personality change failed - agent not initialized")
            raise HTTPException(status_code=503, detail="Agent not initialized")
        
        try:
            log_request(request, "🎭 Personality change requested", {
                "new_personality": personality_request.personality
            })
            
            old_personality = agent.personality
            agent.set_personality(personality_request.personality)
            
            log_response(200, "🎭 Personality changed successfully", {
                "old_personality": old_personality,
                "new_personality": personality_request.personality
            })
            
            return {
                "success": True, 
                "personality": personality_request.personality,
                "previous_personality": old_personality
            }
        except Exception as e:
            logger.error(f"❌ Set personality failed: {e}")
            log_response(500, "🎭 Personality change failed", {"error": str(e)})
            raise HTTPException(status_code=500, detail=str(e))
    
    @app.post("/clear")
    async def clear_conversation(request: Request):
        """Clear conversation history"""
        if not agent:
            log_request(request, "🚫 Clear conversation rejected - agent not initialized")
            log_response(503, "🚫 Clear conversation failed - agent not initialized")
            raise HTTPException(status_code=503, detail="Agent not initialized")
        
        try:
            conversation_length = len(agent.conversation_history)
            
            log_request(request, "🗑️ Clear conversation requested", {
                "conversation_length": conversation_length
            })
            
            agent.clear_conversation()
            
            log_response(200, "🗑️ Conversation cleared successfully", {
                "cleared_messages": conversation_length
            })
            
            return {
                "success": True, 
                "message": "Conversation cleared",
                "cleared_messages": conversation_length
            }
        except Exception as e:
            logger.error(f"❌ Clear conversation failed: {e}")
            log_response(500, "🗑️ Clear conversation failed", {"error": str(e)})
            raise HTTPException(status_code=500, detail=str(e))
    
    @app.get("/history")
    async def get_history(request: Request):
        """Get conversation history"""
        if not agent:
            log_request(request, "🚫 Get history rejected - agent not initialized")
            log_response(503, "🚫 Get history failed - agent not initialized")
            raise HTTPException(status_code=503, detail="Agent not initialized")
        
        try:
            log_request(request, "📜 Conversation history requested")
            
            history = agent.get_conversation_history()
            
            response_data = [
                ChatMessageResponse(
                    id=f"{msg.timestamp.isoformat()}-{msg.role}",
                    timestamp=msg.timestamp.isoformat(),
                    role=msg.role,
                    content=msg.content,
                    model_used=msg.model_used,
                    metadata=msg.metadata
                )
                for msg in history
            ]
            
            log_response(200, "📜 Conversation history retrieved successfully", {
                "history_length": len(response_data)
            })
            
            return response_data
        except Exception as e:
            logger.error(f"❌ Get history failed: {e}")
            log_response(500, "📜 Get history failed", {"error": str(e)})
            raise HTTPException(status_code=500, detail=str(e))
    
    return app

def run_server(host: str = "0.0.0.0", port: int = None):
    """Run the Hecate agent server"""
    # Use environment variable if port not specified
    if port is None:
        import os
        port = int(os.getenv('HECATE_PORT', '9002'))
    
    app = create_app()
    
    logger.info(f"🚀 Starting Hecate Agent HTTP server on {host}:{port}")
    logger.info(f"🌐 Server will be accessible at: http://{host}:{port}")
    logger.info(f"📋 Health check: http://{host}:{port}/health")
    logger.info(f"💬 Chat endpoint: http://{host}:{port}/chat")
    logger.info(f"📚 API Documentation: http://{host}:{port}/docs")
    logger.info(f"🔍 Model status: http://{host}:{port}/model-status")
    logger.info(f"📋 Available models: http://{host}:{port}/available-models")
    logger.info(f"🎯 Set model: http://{host}:{port}/set-model")
    logger.info(f"🔄 Reset models: http://{host}:{port}/reset-models")
    logger.info(f"📜 History: http://{host}:{port}/history")
    logger.info(f"🎭 Personality: http://{host}:{port}/personality")
    logger.info(f"🗑️ Clear: http://{host}:{port}/clear")
    
    uvicorn.run(
        app,
        host=host,
        port=port,
        log_level="info",
        access_log=True
    )

if __name__ == "__main__":
    run_server()