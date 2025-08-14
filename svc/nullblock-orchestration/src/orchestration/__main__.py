#!/usr/bin/env python3
"""
Main entry point for Nullblock Orchestration Engine
"""

import os
import sys
from pathlib import Path

# Add the src directory to the Python path
src_path = Path(__file__).parent.parent.parent
sys.path.insert(0, str(src_path))

from orchestration.workflow.engine import WorkflowEngine

def main():
    """Main entry point"""
    engine = WorkflowEngine()
    engine.start()

if __name__ == "__main__":
    main() 