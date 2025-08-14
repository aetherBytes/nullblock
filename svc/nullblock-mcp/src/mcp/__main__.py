#!/usr/bin/env python3
"""
Main entry point for Nullblock MCP Server
"""

import os
import sys
from pathlib import Path

# Add the src directory to the Python path
src_path = Path(__file__).parent.parent.parent
sys.path.insert(0, str(src_path))

from mcp.server import create_server

def main():
    """Main entry point"""
    server = create_server()
    server.run(host="0.0.0.0", port=8000, debug=False)

if __name__ == "__main__":
    main() 