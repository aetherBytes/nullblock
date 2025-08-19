#!/usr/bin/env python3
"""
Temporary Erebus server replacement while fixing Rust compilation issues.
This provides the basic health endpoint to unblock testing.
"""

import json
import logging
from datetime import datetime
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
import uvicorn

# Setup logging to both file and console
import os
os.makedirs('logs', exist_ok=True)

# Configure logging
log_format = logging.Formatter(
    '%(asctime)s [EREBUS-TEMP] %(levelname)s %(message)s',
    datefmt='%Y-%m-%d %H:%M:%S'
)

logger = logging.getLogger(__name__)
logger.setLevel(logging.INFO)

# Console handler
console_handler = logging.StreamHandler()
console_handler.setFormatter(log_format)
logger.addHandler(console_handler)

# File handler
file_handler = logging.FileHandler('logs/erebus-temp.log')
file_handler.setFormatter(log_format)
logger.addHandler(file_handler)

app = FastAPI(title="Erebus (Temporary)", version="0.1.0")

# Enable CORS
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

@app.get("/")
async def root():
    return {
        "status": "running",
        "service": "erebus-temp",
        "version": "0.1.0",
        "message": "💡 Temporary Erebus - Basic health endpoint only"
    }

@app.get("/health")
async def health_check():
    logger.info("🏥 Health check requested")
    return {
        "status": "healthy",
        "service": "erebus-temp",
        "version": "0.1.0",
        "timestamp": datetime.now().isoformat(),
        "message": "🎯 Basic health endpoint (Rust version under repair)"
    }

if __name__ == "__main__":
    logger.info("🚀 Starting temporary Erebus server...")
    logger.info("📍 Version: 0.1.0 (Python temporary replacement)")
    logger.info(f"🕐 Timestamp: {datetime.now()}")
    logger.info("============================================================")
    logger.info("🌐 Server starting on http://0.0.0.0:3000")
    logger.info("🏥 Health check: http://localhost:3000/health")
    logger.info("💡 Temporary server while fixing Rust compilation")
    logger.info("✅ Server ready")
    
    uvicorn.run(app, host="0.0.0.0", port=3000, log_level="info")