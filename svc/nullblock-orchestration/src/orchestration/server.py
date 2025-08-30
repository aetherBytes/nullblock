"""
FastAPI server for Nullblock Orchestration
"""

import logging
import logging.handlers
import os
from pathlib import Path
from datetime import datetime
from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
from typing import Dict, Any, Optional
import uvicorn

from .workflow.engine import WorkflowOrchestrator

def setup_orchestration_logging():
    """Setup comprehensive logging for Orchestration service"""
    # Create logs directory
    log_dir = Path("logs")
    log_dir.mkdir(exist_ok=True)
    
    # Setup logger
    orchestration_logger = logging.getLogger("orchestration")
    orchestration_logger.setLevel(logging.INFO)
    
    # Clear existing handlers
    orchestration_logger.handlers.clear()
    
    # Console handler
    console_handler = logging.StreamHandler()
    console_formatter = logging.Formatter(
        '%(asctime)s [ORCHESTRATION] %(levelname)-8s %(message)s',
        datefmt='%H:%M:%S'
    )
    console_handler.setFormatter(console_formatter)
    orchestration_logger.addHandler(console_handler)
    
    # Main log file handler
    file_handler = logging.handlers.RotatingFileHandler(
        log_dir / "orchestration.log",
        maxBytes=10*1024*1024,  # 10MB
        backupCount=5,
        encoding='utf-8'
    )
    file_formatter = logging.Formatter(
        '%(asctime)s [ORCHESTRATION] %(levelname)-8s %(message)s',
        datefmt='%Y-%m-%d %H:%M:%S'
    )
    file_handler.setFormatter(file_formatter)
    orchestration_logger.addHandler(file_handler)
    
    # Error log file handler
    error_handler = logging.handlers.RotatingFileHandler(
        log_dir / "orchestration-errors.log",
        maxBytes=5*1024*1024,  # 5MB
        backupCount=3,
        encoding='utf-8'
    )
    error_handler.setLevel(logging.ERROR)
    error_handler.setFormatter(file_formatter)
    orchestration_logger.addHandler(error_handler)
    
    # Workflow log file handler
    workflow_handler = logging.handlers.RotatingFileHandler(
        log_dir / "orchestration-workflows.log",
        maxBytes=10*1024*1024,  # 10MB
        backupCount=5,
        encoding='utf-8'
    )
    workflow_formatter = logging.Formatter(
        '%(asctime)s [WORKFLOW] %(message)s',
        datefmt='%Y-%m-%d %H:%M:%S'
    )
    workflow_handler.setFormatter(workflow_formatter)
    
    # Create workflow logger
    workflow_logger = logging.getLogger("orchestration.workflows")
    workflow_logger.setLevel(logging.INFO)
    workflow_logger.addHandler(workflow_handler)
    workflow_logger.propagate = False
    
    orchestration_logger.propagate = False
    return orchestration_logger

logger = setup_orchestration_logging()

# Create FastAPI app
app = FastAPI(
    title="Nullblock Orchestration",
    description="Goal-driven workflow orchestration engine",
    version="0.1.0"
)

# Add CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Initialize workflow engine
workflow_engine = WorkflowOrchestrator()

# Health check model
class HealthResponse(BaseModel):
    status: str
    service: str
    version: str
    timestamp: str

# Workflow models
class CreateWorkflowRequest(BaseModel):
    name: str
    description: str
    goal_description: str
    target_metric: str
    target_value: float
    user_id: str

class WorkflowResponse(BaseModel):
    workflow_id: str
    name: str
    status: str
    message: str

@app.get("/health", response_model=HealthResponse)
async def health_check():
    """Health check endpoint"""
    import datetime
    return HealthResponse(
        status="healthy",
        service="nullblock-orchestration",
        version="0.1.0",
        timestamp=datetime.datetime.now().isoformat()
    )

@app.get("/")
async def root():
    """Root endpoint"""
    return {
        "service": "Nullblock Orchestration",
        "version": "0.1.0",
        "status": "running"
    }

@app.post("/workflows", response_model=WorkflowResponse)
async def create_workflow(request: CreateWorkflowRequest):
    """Create a new workflow"""
    try:
        # Create workflow using the engine
        workflow_id = workflow_engine.create_workflow(
            name=request.name,
            description=request.description,
            goal_description=request.goal_description,
            target_metric=request.target_metric,
            target_value=request.target_value,
            user_id=request.user_id
        )
        
        return WorkflowResponse(
            workflow_id=workflow_id,
            name=request.name,
            status="created",
            message="Workflow created successfully"
        )
    except Exception as e:
        logger.error(f"Error creating workflow: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/workflows/{workflow_id}")
async def get_workflow_status(workflow_id: str):
    """Get workflow status"""
    try:
        status = workflow_engine.get_workflow_status(workflow_id)
        if not status:
            raise HTTPException(status_code=404, detail="Workflow not found")
        return status
    except Exception as e:
        logger.error(f"Error getting workflow status: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/workflows/{workflow_id}/start")
async def start_workflow(workflow_id: str):
    """Start a workflow"""
    try:
        workflow_engine.start_workflow(workflow_id)
        return {"message": "Workflow started successfully"}
    except Exception as e:
        logger.error(f"Error starting workflow: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.on_event("startup")
async def startup_event():
    """Log startup information"""
    logger.info("=" * 60)
    logger.info("🚀 ORCHESTRATION SERVICE STARTING")
    logger.info(f"📍 Version: 0.1.0")
    logger.info(f"🕐 Timestamp: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    logger.info("=" * 60)
    logger.info("✅ Workflow engine initialized")
    logger.info("🎯 Orchestration service ready for connections")

@app.on_event("shutdown")
async def shutdown_event():
    """Log shutdown information"""
    logger.info("=" * 60)
    logger.info("🛑 ORCHESTRATION SERVICE SHUTTING DOWN")
    logger.info(f"🕐 Timestamp: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    logger.info("=" * 60)
    logger.info("👋 Orchestration service shutdown complete")

def run_server(host: str = "0.0.0.0", port: int = 8002, debug: bool = False):
    """Run the orchestration server"""
    uvicorn.run(
        app,
        host=host,
        port=port,
        reload=debug,
        log_level="info"
    )

if __name__ == "__main__":
    run_server(debug=True)











