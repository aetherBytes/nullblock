#!/usr/bin/env python3
"""
Startup script for Hecate Agent Server with comprehensive logging setup
"""

import os
import sys
import asyncio
import time
import platform
import psutil
from pathlib import Path
from datetime import datetime

# Add the agents path to allow imports
current_dir = Path(__file__).parent
sys.path.insert(0, str(current_dir / "src"))

from src.agents.hecate.server import run_server
from src.agents.logging_config import setup_agent_logging, log_agent_startup

def get_system_info():
    """Get comprehensive system information for logging"""
    try:
        cpu_count = psutil.cpu_count()
        memory = psutil.virtual_memory()
        disk = psutil.disk_usage('/')
        
        return {
            'platform': platform.platform(),
            'python_version': platform.python_version(),
            'cpu_count': cpu_count,
            'memory_total_gb': round(memory.total / (1024**3), 2),
            'memory_available_gb': round(memory.available / (1024**3), 2),
            'disk_total_gb': round(disk.total / (1024**3), 2),
            'disk_free_gb': round(disk.free / (1024**3), 2)
        }
    except Exception as e:
        return {'error': f'Failed to get system info: {e}'}

def get_environment_info():
    """Get environment variables and configuration info"""
    env_vars = {
        'HECATE_HOST': os.getenv('HECATE_HOST', '0.0.0.0'),
        'HECATE_PORT': os.getenv('HECATE_PORT', '9002'),
        'AGENTS_HOST': os.getenv('AGENTS_HOST', '0.0.0.0'),
        'AGENTS_PORT': os.getenv('AGENTS_PORT', '9001'),
        'MCP_SERVER_HOST': os.getenv('MCP_SERVER_HOST', 'localhost'),
        'MCP_SERVER_PORT': os.getenv('MCP_SERVER_PORT', '8001'),
        'EREBUS_HOST': os.getenv('EREBUS_HOST', 'localhost'),
        'EREBUS_PORT': os.getenv('EREBUS_PORT', '3000'),
        'ORCHESTRATION_HOST': os.getenv('ORCHESTRATION_HOST', 'localhost'),
        'ORCHESTRATION_PORT': os.getenv('ORCHESTRATION_PORT', '8002'),
    }
    
    return env_vars

def print_startup_banner():
    """Print a beautiful startup banner"""
    banner = """
╔══════════════════════════════════════════════════════════════╗
║                    🎯 HECATE AGENT SERVER                    ║
║                                                              ║
║  🤖 AI-Powered Trading & Analysis Platform                  ║
║  🌐 HTTP API Server for Frontend Integration                ║
║  📊 Real-time Market Data & Portfolio Management            ║
║  🔗 Connected to Nullblock Ecosystem                        ║
╚══════════════════════════════════════════════════════════════╝
"""
    print(banner)

def main():
    """Main entry point with comprehensive logging setup"""
    
    # Print startup banner
    print_startup_banner()
    
    start_time = time.time()
    
    # Ensure we're in the right directory (nullblock-agents)
    if not os.path.exists("src/agents"):
        print("❌ Error: Please run from the nullblock-agents directory")
        print("📍 Current directory:", os.getcwd())
        print("📝 Expected to find: src/agents/")
        sys.exit(1)
    
    # Create logs directory
    os.makedirs("logs", exist_ok=True)
    
    # Setup logging with file output
    logger = setup_agent_logging("hecate-startup", "INFO", enable_file_logging=True)
    
    # Log comprehensive startup information
    logger.info("🚀 Starting Hecate Agent HTTP Server...")
    logger.info("📁 Working directory: " + os.getcwd())
    logger.info("📝 Log files will be written to: logs/")
    
    # Get the actual port from environment
    hecate_port = os.getenv('HECATE_PORT', '9002')
    logger.info(f"🔗 Server will run on: http://0.0.0.0:{hecate_port}")
    
    # Log system information
    system_info = get_system_info()
    logger.info("💻 System Information:", system_info)
    
    # Log environment configuration
    env_info = get_environment_info()
    logger.info("⚙️  Environment Configuration:", env_info)
    
    # Log startup timing
    startup_duration = time.time() - start_time
    logger.info(f"⏱️  Startup preparation completed in {startup_duration:.2f}s")
    
    # Log service dependencies
    logger.info("🔗 Service Dependencies:")
    logger.info("  • MCP Server: http://%s:%s", env_info['MCP_SERVER_HOST'], env_info['MCP_SERVER_PORT'])
    logger.info("  • Erebus Server: http://%s:%s", env_info['EREBUS_HOST'], env_info['EREBUS_PORT'])
    logger.info("  • Orchestration: http://%s:%s", env_info['ORCHESTRATION_HOST'], env_info['ORCHESTRATION_PORT'])
    logger.info("  • General Agents: http://%s:%s", env_info['AGENTS_HOST'], env_info['AGENTS_PORT'])
    
    # Log agent startup
    log_agent_startup(logger, "hecate-startup", "1.0.0")
    
    logger.info("🎯 Server initialization complete - Starting HTTP server...")
    logger.info("=" * 80)
    
    try:
        # Run the server
        server_start_time = time.time()
        logger.info("🌐 HTTP Server starting...")
        
        # Get port from environment variable
        hecate_port = int(os.getenv('HECATE_PORT', '9002'))
        run_server(host="0.0.0.0", port=hecate_port)
    except KeyboardInterrupt:
        logger.info("🛑 Server shutdown requested by user")
        logger.info("⏱️  Server ran for %.2f seconds", time.time() - server_start_time)
    except Exception as e:
        logger.error(f"❌ Server failed to start: {e}")
        logger.error("🔍 Full error details:", exc_info=True)
        sys.exit(1)

if __name__ == "__main__":
    main()