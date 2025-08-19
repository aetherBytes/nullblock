"""
Hecate Agent HTTP Server

Simple FastAPI server wrapper for the Hecate agent to enable frontend integration.
"""

import asyncio
import logging
from datetime import datetime
from typing import Dict, List, Any, Optional
from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
import uvicorn

from .main import HecateAgent, ConversationMessage

logger = logging.getLogger(__name__)

# Pydantic models for API
class ChatRequest(BaseModel):
    message: str
    user_context: Optional[Dict[str, Any]] = None

class PersonalityRequest(BaseModel):
    personality: str

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
    
    @app.on_event("startup")
    async def startup_event():
        """Initialize agent on startup"""
        global agent
        try:
            agent = HecateAgent()
            await agent.start()
            logger.info("Hecate Agent started successfully")
        except Exception as e:
            logger.error(f"Failed to start Hecate Agent: {e}")
            raise
    
    @app.on_event("shutdown")
    async def shutdown_event():
        """Cleanup agent on shutdown"""
        global agent
        if agent:
            await agent.stop()
            logger.info("Hecate Agent stopped")
    
    @app.get("/health")
    async def health_check():
        """Health check endpoint"""
        if not agent or not agent.running:
            raise HTTPException(status_code=503, detail="Agent not running")
        
        return {
            "status": "healthy",
            "agent_running": agent.running,
            "personality": agent.personality,
            "conversation_length": len(agent.conversation_history)
        }
    
    @app.post("/chat")
    async def chat_endpoint(request: ChatRequest):
        """Send message to Hecate agent"""
        if not agent:
            raise HTTPException(status_code=503, detail="Agent not initialized")
        
        try:
            response = await agent.chat(request.message, request.user_context)
            return {
                "content": response.content,
                "model_used": response.model_used,
                "latency_ms": response.latency_ms,
                "confidence_score": response.confidence_score,
                "metadata": response.metadata
            }
        except Exception as e:
            logger.error(f"Chat request failed: {e}")
            raise HTTPException(status_code=500, detail=str(e))
    
    @app.get("/model-status")
    async def model_status():
        """Get current model status"""
        if not agent:
            raise HTTPException(status_code=503, detail="Agent not initialized")
        
        try:
            status = await agent.get_model_status()
            return status
        except Exception as e:
            logger.error(f"Model status request failed: {e}")
            raise HTTPException(status_code=500, detail=str(e))
    
    @app.post("/personality")
    async def set_personality(request: PersonalityRequest):
        """Set agent personality"""
        if not agent:
            raise HTTPException(status_code=503, detail="Agent not initialized")
        
        try:
            agent.set_personality(request.personality)
            return {"success": True, "personality": request.personality}
        except Exception as e:
            logger.error(f"Set personality failed: {e}")
            raise HTTPException(status_code=500, detail=str(e))
    
    @app.post("/clear")
    async def clear_conversation():
        """Clear conversation history"""
        if not agent:
            raise HTTPException(status_code=503, detail="Agent not initialized")
        
        try:
            agent.clear_conversation()
            return {"success": True, "message": "Conversation cleared"}
        except Exception as e:
            logger.error(f"Clear conversation failed: {e}")
            raise HTTPException(status_code=500, detail=str(e))
    
    @app.get("/history")
    async def get_history():
        """Get conversation history"""
        if not agent:
            raise HTTPException(status_code=503, detail="Agent not initialized")
        
        try:
            history = agent.get_conversation_history()
            return [
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
        except Exception as e:
            logger.error(f"Get history failed: {e}")
            raise HTTPException(status_code=500, detail=str(e))
    
    return app

def run_server(host: str = "0.0.0.0", port: int = 8001):
    """Run the Hecate agent server"""
    app = create_app()
    
    logging.basicConfig(
        level=logging.INFO,
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
    )
    
    logger.info(f"Starting Hecate Agent server on {host}:{port}")
    
    uvicorn.run(
        app,
        host=host,
        port=port,
        log_level="info",
        access_log=True
    )

if __name__ == "__main__":
    run_server()