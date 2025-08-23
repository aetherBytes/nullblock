"""
Hecate Agent HTTP Server

Simple FastAPI server wrapper for the Hecate agent to enable frontend integration.
"""

import asyncio
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

from .main import HecateAgent, ConversationMessage
from ..logging_config import setup_agent_logging, log_agent_startup, log_agent_shutdown
from ..config import get_hecate_config, config

logger = setup_agent_logging("hecate-server", "INFO", enable_file_logging=True)

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
    
    @app.get("/available-models")
    async def get_available_models(request: Request):
        """Get list of available models for selection"""
        if not agent:
            log_request(request, "🚫 Available models request rejected - agent not initialized")
            log_response(503, "🚫 Available models request failed - agent not initialized")
            raise HTTPException(status_code=503, detail="Agent not initialized")
        
        try:
            log_request(request, "📋 Available models requested")
            
            available_models = []
            
            if hasattr(agent, 'llm_factory') and agent.llm_factory:
                # Get all models from the router
                from ..llm_service.models import AVAILABLE_MODELS
                
                for model_name, config in AVAILABLE_MODELS.items():
                    # Check if model is available
                    is_available = agent.llm_factory.router.model_status.get(model_name, False)
                    
                    available_models.append({
                        "name": model_name,
                        "display_name": config.name,
                        "provider": config.provider.value,
                        "available": is_available,
                        "tier": config.tier.value,
                        "context_length": config.metrics.context_window,
                        "capabilities": [cap.value for cap in config.capabilities]
                    })
            
            # Sort by availability first, then by provider and name
            available_models.sort(key=lambda x: (not x["available"], x["provider"], x["name"]))
            
            log_response(200, "📋 Available models retrieved successfully", {
                "models_count": len(available_models),
                "available_count": sum(1 for m in available_models if m["available"])
            })
            
            return {
                "models": available_models,
                "current_model": agent.current_model
            }
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
                log_response(400, "🎯 Model selection failed - model not available", {
                    "requested_model": model_request.model_name
                })
                raise HTTPException(status_code=400, detail=f"Model {model_request.model_name} is not available")
                
        except HTTPException:
            raise
        except Exception as e:
            logger.error(f"❌ Set model failed: {e}")
            log_response(500, "🎯 Model selection failed", {"error": str(e)})
            raise HTTPException(status_code=500, detail=str(e))

    @app.post("/unload-lm-studio-model")
    async def unload_lm_studio_model(request: Request, unload_request: dict):
        """Unload model from LM Studio using CLI"""
        try:
            log_request(request, "🔄 LM Studio model unload requested", {
                "current_model": unload_request.get("current_model")
            })
            
            # Use lms unload --all to unload all models for clean slate
            proc = await asyncio.create_subprocess_exec(
                "lms", "unload", "--all",
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE
            )
            
            stdout, stderr = await proc.communicate()
            
            if proc.returncode == 0:
                log_response(200, "🔄 LM Studio model unloaded successfully")
                return {
                    "success": True,
                    "message": "LM Studio model unloaded successfully",
                    "output": stdout.decode() if stdout else "No output"
                }
            else:
                error_msg = stderr.decode() if stderr else "Unknown error"
                log_response(400, "🔄 LM Studio model unload failed", {
                    "error": error_msg,
                    "return_code": proc.returncode
                })
                return {
                    "success": False,
                    "message": f"Failed to unload LM Studio model: {error_msg}",
                    "return_code": proc.returncode
                }
                
        except Exception as e:
            logger.error(f"❌ LM Studio model unload failed: {e}")
            log_response(500, "🔄 LM Studio model unload failed", {"error": str(e)})
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
    logger.info(f"🔄 Unload LM Studio model: http://{host}:{port}/unload-lm-studio-model")
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