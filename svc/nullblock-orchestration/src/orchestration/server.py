"""
FastAPI server for Nullblock Orchestration
"""

import logging
from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
from typing import Dict, Any, Optional
import uvicorn

from .workflow.engine import WorkflowOrchestrator

logger = logging.getLogger(__name__)

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





