import logging
import logging.config
from pythonjsonlogger import jsonlogger
from typing import Any, Dict, Optional

# Define the logging configuration
logging_config = {
    "version": 1,
    "disable_existing_loggers": False,
    "formatters": {
        "json": {
            "()": jsonlogger.JsonFormatter,  # type: ignore
            "format": "%(asctime)s %(levelname)s %(name)s %(message)s %(context)s",
        },
    },
    "handlers": {
        "console": {
            "class": "logging.StreamHandler",
            "level": "DEBUG",
            "formatter": "json",
            "stream": "ext://sys.stdout",
        },
        "file": {
            "class": "logging.FileHandler",
            "level": "DEBUG",
            "formatter": "json",
            "filename": "logs/helios.log",
            "mode": "a",
        },
    },
    "loggers": {
        "": {
            "level": "DEBUG",
            "handlers": ["console", "file"],
            "propagate": False,
        },
    },
}

# Configure logging
logging.config.dictConfig(logging_config)


# Custom logging function to handle context
def log_with_context(
    logger: logging.Logger,
    level: str,
    message: str,
    context: Optional[Dict[str, Any]] = None,
) -> None:
    extra = {"context": context} if context else {}
    if level.lower() == "debug":
        logger.debug(message, extra=extra)
    elif level.lower() == "info":
        logger.info(message, extra=extra)
    elif level.lower() == "warning":
        logger.warning(message, extra=extra)
    elif level.lower() == "error":
        logger.error(message, extra=extra)
    elif level.lower() == "critical":
        logger.critical(message, extra=extra)
    else:
        raise ValueError("Invalid log level specified")
