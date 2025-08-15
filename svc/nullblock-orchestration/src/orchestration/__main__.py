#!/usr/bin/env python3
"""
Main entry point for Nullblock Orchestration Engine
"""

import os
import sys
import logging
from pathlib import Path

# Add the src directory to the Python path
src_path = Path(__file__).parent.parent.parent
sys.path.insert(0, str(src_path))

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    handlers=[
        logging.StreamHandler(sys.stdout),
        logging.FileHandler('orchestration.log')
    ]
)

from orchestration.server import run_server

def main():
    """Main entry point"""
    logger = logging.getLogger(__name__)
    logger.info("Starting Nullblock Orchestration Server...")
    
    # Get configuration from environment
    host = os.getenv("ORCHESTRATION_HOST", "0.0.0.0")
    port = int(os.getenv("ORCHESTRATION_PORT", "8002"))
    debug = os.getenv("ORCHESTRATION_DEBUG", "false").lower() == "true"
    
    logger.info(f"Starting server on {host}:{port}")
    run_server(host=host, port=port, debug=debug)

if __name__ == "__main__":
    main() 