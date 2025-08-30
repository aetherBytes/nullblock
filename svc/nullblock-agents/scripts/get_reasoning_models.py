#!/usr/bin/env python3
"""Script to get available reasoning models."""

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

from src.agents.llm_service.models import get_reasoning_models


def main():
    """Get available reasoning models."""
    models = get_reasoning_models()
    for name, config in models.items():
        print(f'   - {name}: {config.description}')


if __name__ == "__main__":
    main()