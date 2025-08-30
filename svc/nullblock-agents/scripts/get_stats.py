#!/usr/bin/env python3
"""Script to get LLM service stats for monitoring."""

import asyncio
import sys
import os
from dotenv import load_dotenv

# Add the project root to Python path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

# Load environment variables from .env.dev file in project root
project_root = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), '..', '..')
env_file = os.path.join(project_root, '.env.dev')
if os.path.exists(env_file):
    load_dotenv(env_file)

from src.agents.llm_service.factory import LLMServiceFactory


async def get_stats():
    """Get LLM service statistics."""
    factory = LLMServiceFactory()
    await factory.initialize()
    stats = factory.get_stats()
    print(f'Requests: {stats["request_stats"]}')
    print(f'Costs: {stats["cost_tracking"]}')
    await factory.cleanup()


if __name__ == "__main__":
    asyncio.run(get_stats())