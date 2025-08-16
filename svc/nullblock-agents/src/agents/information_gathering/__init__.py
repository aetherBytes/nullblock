"""
Information Gathering Agent Package

This package contains the core information gathering agent and supporting components
for analyzing prepared and modeled data from various data sources via MCP.
"""

from .main import InformationGatheringAgent
from .data_analyzer import DataAnalyzer
from .pattern_detector import PatternDetector
from .mcp_client import MCPClient

__all__ = [
    "InformationGatheringAgent",
    "DataAnalyzer", 
    "PatternDetector",
    "MCPClient"
]