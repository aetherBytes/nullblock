"""
Standardized Logging Configuration for Nullblock Agents

Provides consistent logging format and configuration across all agents.
"""

import logging
import sys
from datetime import datetime
from typing import Optional

# ANSI color codes for console output
class Colors:
    RESET = '\033[0m'
    BOLD = '\033[1m'
    
    # Basic colors
    BLACK = '\033[30m'
    RED = '\033[31m'
    GREEN = '\033[32m'
    YELLOW = '\033[33m'
    BLUE = '\033[34m'
    MAGENTA = '\033[35m'
    CYAN = '\033[36m'
    WHITE = '\033[37m'
    
    # Bright colors
    BRIGHT_BLACK = '\033[90m'
    BRIGHT_RED = '\033[91m'
    BRIGHT_GREEN = '\033[92m'
    BRIGHT_YELLOW = '\033[93m'
    BRIGHT_BLUE = '\033[94m'
    BRIGHT_MAGENTA = '\033[95m'
    BRIGHT_CYAN = '\033[96m'
    BRIGHT_WHITE = '\033[97m'

class ColoredFormatter(logging.Formatter):
    """Custom formatter with colors and cyberpunk styling"""
    
    def __init__(self, agent_name: str):
        self.agent_name = agent_name
        super().__init__()
    
    def format(self, record):
        # Choose color based on log level
        level_colors = {
            'DEBUG': Colors.BRIGHT_BLACK,
            'INFO': Colors.BRIGHT_CYAN,
            'WARNING': Colors.BRIGHT_YELLOW,
            'ERROR': Colors.BRIGHT_RED,
            'CRITICAL': Colors.BRIGHT_MAGENTA + Colors.BOLD,
        }
        
        # Choose agent color based on agent name
        agent_colors = {
            'hecate': Colors.BRIGHT_BLUE,
            'information_gathering': Colors.BRIGHT_GREEN,
            'arbitrage': Colors.BRIGHT_YELLOW,
            'social_trading': Colors.BRIGHT_MAGENTA,
            'llm_service': Colors.BRIGHT_CYAN,
        }
        
        level_color = level_colors.get(record.levelname, Colors.WHITE)
        agent_color = agent_colors.get(self.agent_name.lower(), Colors.WHITE)
        
        # Format timestamp
        timestamp = datetime.fromtimestamp(record.created).strftime('%H:%M:%S.%f')[:-3]
        
        # Build the formatted message
        formatted = (
            f"{Colors.BRIGHT_BLACK}[{timestamp}]{Colors.RESET} "
            f"{agent_color}[{self.agent_name.upper()}]{Colors.RESET} "
            f"{level_color}{record.levelname:<8}{Colors.RESET} "
            f"{Colors.WHITE}{record.getMessage()}{Colors.RESET}"
        )
        
        # Add exception info if present
        if record.exc_info:
            formatted += f"\n{Colors.BRIGHT_RED}{self.formatException(record.exc_info)}{Colors.RESET}"
        
        return formatted

def setup_agent_logging(
    agent_name: str,
    log_level: str = "INFO",
    enable_file_logging: bool = False,
    log_file: Optional[str] = None
) -> logging.Logger:
    """
    Setup standardized logging for an agent
    
    Args:
        agent_name: Name of the agent (e.g., 'hecate', 'information_gathering')
        log_level: Logging level ('DEBUG', 'INFO', 'WARNING', 'ERROR', 'CRITICAL')
        enable_file_logging: Whether to also log to a file
        log_file: Optional path to log file (defaults to agent_name.log)
        
    Returns:
        Configured logger instance
    """
    logger = logging.getLogger(agent_name)
    logger.setLevel(getattr(logging, log_level.upper()))
    
    # Clear existing handlers
    logger.handlers.clear()
    
    # Console handler with colored output
    console_handler = logging.StreamHandler(sys.stdout)
    console_handler.setFormatter(ColoredFormatter(agent_name))
    logger.addHandler(console_handler)
    
    # File handler if requested
    if enable_file_logging:
        if not log_file:
            log_file = f"logs/{agent_name}.log"
        
        # Create logs directory if it doesn't exist
        import os
        log_dir = os.path.dirname(log_file)
        if log_dir:  # Only create directory if there's a directory component
            os.makedirs(log_dir, exist_ok=True)
        else:
            # If no directory specified, create logs directory in current working directory
            os.makedirs("logs", exist_ok=True)
            log_file = f"logs/{agent_name}.log"
        
        file_handler = logging.FileHandler(log_file)
        file_formatter = logging.Formatter(
            '%(asctime)s [%(name)s] %(levelname)-8s %(message)s',
            datefmt='%Y-%m-%d %H:%M:%S'
        )
        file_handler.setFormatter(file_formatter)
        logger.addHandler(file_handler)
    
    # Prevent duplicate logs
    logger.propagate = False
    
    return logger

def log_agent_startup(logger: logging.Logger, agent_name: str, version: str = "1.0.0"):
    """Log standardized agent startup information"""
    logger.info("=" * 60)
    logger.info(f"🤖 {agent_name.upper()} AGENT STARTING")
    logger.info(f"📍 Version: {version}")
    logger.info(f"🕐 Timestamp: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    logger.info(f"🔧 Log Level: {logger.level}")
    logger.info("=" * 60)

def log_agent_shutdown(logger: logging.Logger, agent_name: str):
    """Log standardized agent shutdown information"""
    logger.info("=" * 60)
    logger.info(f"🛑 {agent_name.upper()} AGENT SHUTTING DOWN")
    logger.info(f"🕐 Timestamp: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    logger.info("=" * 60)

def log_model_info(logger: logging.Logger, model_name: str, provider: str, cost: float = 0.0):
    """Log LLM model usage information"""
    logger.info(f"🧠 Model: {model_name} ({provider}) | Cost: ${cost:.4f}")

def log_request_start(logger: logging.Logger, request_type: str, details: str = ""):
    """Log request start with timing info"""
    logger.info(f"📥 {request_type} request started {details}")

def log_request_complete(logger: logging.Logger, request_type: str, duration_ms: float, success: bool = True):
    """Log request completion with timing"""
    status = "✅ SUCCESS" if success else "❌ FAILED"
    logger.info(f"📤 {request_type} request completed | {status} | {duration_ms:.0f}ms")

def log_agent_health(logger: logging.Logger, status: dict):
    """Log agent health status"""
    logger.info(f"💚 Health check: {status}")

# Example usage for quick setup
def get_logger(agent_name: str, debug: bool = False) -> logging.Logger:
    """Quick logger setup for agents"""
    level = "DEBUG" if debug else "INFO"
    return setup_agent_logging(agent_name, level)