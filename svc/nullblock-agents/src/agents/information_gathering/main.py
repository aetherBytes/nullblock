"""
Information Gathering Agent

Core agent specialized in analyzing prepared and modeled data from various data sources.
Accesses data via the Nullblock MCP server for standardized data source interactions.
"""

import asyncio
import logging
from typing import Dict, List, Any, Optional
from dataclasses import dataclass
from datetime import datetime

from .data_analyzer import DataAnalyzer
from .pattern_detector import PatternDetector
from .mcp_client import MCPClient

logger = logging.getLogger(__name__)

@dataclass
class DataRequest:
    """Represents a data gathering request"""
    source_type: str  # 'price_oracle', 'defi_protocol', 'social_sentiment', 'onchain_analytics'
    source_name: str  # e.g., 'chainlink', 'uniswap', 'twitter', 'etherscan'
    parameters: Dict[str, Any]
    analysis_type: str  # 'trend', 'pattern', 'anomaly', 'correlation'
    priority: int = 1  # 1-5, higher = more urgent
    context: Optional[Dict[str, Any]] = None

@dataclass
class AnalysisResult:
    """Represents the result of data analysis"""
    request_id: str
    source_type: str
    source_name: str
    timestamp: datetime
    data: Dict[str, Any]
    insights: List[str]
    confidence_score: float  # 0.0-1.0
    patterns_detected: List[Dict[str, Any]]
    anomalies: List[Dict[str, Any]]
    recommendations: List[str]

class InformationGatheringAgent:
    """
    Core Information Gathering Agent
    
    Specializes in:
    - Multi-source data aggregation
    - Pattern recognition and trend analysis
    - Anomaly detection across data streams
    - Contextual analysis based on user goals
    """
    
    def __init__(self, mcp_server_url: str = "http://localhost:8000"):
        self.mcp_client = MCPClient(mcp_server_url)
        self.data_analyzer = DataAnalyzer()
        self.pattern_detector = PatternDetector()
        self.active_requests: Dict[str, DataRequest] = {}
        self.analysis_cache: Dict[str, AnalysisResult] = {}
        self.running = False
        
        logger.info(f"Information Gathering Agent initialized with MCP server: {mcp_server_url}")
    
    async def start(self):
        """Start the information gathering agent"""
        self.running = True
        logger.info("Information Gathering Agent started")
        
        # Initialize MCP connection
        await self.mcp_client.connect()
        
        # Start background processing tasks
        await asyncio.gather(
            self._data_processing_loop(),
            self._pattern_monitoring_loop(),
            self._cache_cleanup_loop()
        )
    
    async def stop(self):
        """Stop the information gathering agent"""
        self.running = False
        await self.mcp_client.disconnect()
        logger.info("Information Gathering Agent stopped")
    
    async def request_data_analysis(self, request: DataRequest) -> str:
        """
        Submit a data analysis request
        
        Args:
            request: DataRequest object specifying what data to gather and analyze
            
        Returns:
            request_id: Unique identifier for tracking the request
        """
        request_id = f"{request.source_type}_{request.source_name}_{datetime.now().timestamp()}"
        self.active_requests[request_id] = request
        
        logger.info(f"Data analysis request submitted: {request_id}")
        logger.debug(f"Request details: {request}")
        
        return request_id
    
    async def get_analysis_result(self, request_id: str) -> Optional[AnalysisResult]:
        """
        Get the result of a data analysis request
        
        Args:
            request_id: The request identifier
            
        Returns:
            AnalysisResult if available, None otherwise
        """
        return self.analysis_cache.get(request_id)
    
    async def get_real_time_data(self, source_type: str, source_name: str, parameters: Dict[str, Any]) -> Dict[str, Any]:
        """
        Get real-time data directly without full analysis
        
        Args:
            source_type: Type of data source
            source_name: Specific source name
            parameters: Query parameters
            
        Returns:
            Raw data from the source
        """
        return await self.mcp_client.get_data(source_type, source_name, parameters)
    
    async def analyze_market_trends(self, symbols: List[str], timeframe: str = "24h") -> AnalysisResult:
        """
        Convenience method for market trend analysis
        
        Args:
            symbols: List of token/asset symbols to analyze
            timeframe: Analysis timeframe (1h, 24h, 7d, etc.)
            
        Returns:
            AnalysisResult with market trend insights
        """
        request = DataRequest(
            source_type="price_oracle",
            source_name="multiple",
            parameters={"symbols": symbols, "timeframe": timeframe},
            analysis_type="trend",
            context={"analysis_goal": "market_trends"}
        )
        
        request_id = await self.request_data_analysis(request)
        
        # Wait for analysis to complete (with timeout)
        for _ in range(30):  # 30 second timeout
            result = await self.get_analysis_result(request_id)
            if result:
                return result
            await asyncio.sleep(1)
        
        raise TimeoutError(f"Analysis request {request_id} timed out")
    
    async def detect_defi_opportunities(self, protocols: List[str]) -> AnalysisResult:
        """
        Convenience method for DeFi opportunity detection
        
        Args:
            protocols: List of DeFi protocols to analyze
            
        Returns:
            AnalysisResult with DeFi opportunity insights
        """
        request = DataRequest(
            source_type="defi_protocol",
            source_name="multiple", 
            parameters={"protocols": protocols},
            analysis_type="pattern",
            context={"analysis_goal": "defi_opportunities"}
        )
        
        request_id = await self.request_data_analysis(request)
        
        # Wait for analysis to complete
        for _ in range(30):
            result = await self.get_analysis_result(request_id)
            if result:
                return result
            await asyncio.sleep(1)
            
        raise TimeoutError(f"Analysis request {request_id} timed out")
    
    async def _data_processing_loop(self):
        """Background loop for processing data requests"""
        while self.running:
            try:
                # Process pending requests
                for request_id, request in list(self.active_requests.items()):
                    if request_id not in self.analysis_cache:
                        await self._process_request(request_id, request)
                
                await asyncio.sleep(1)  # Process every second
                
            except Exception as e:
                logger.error(f"Error in data processing loop: {e}")
                await asyncio.sleep(5)  # Back off on error
    
    async def _pattern_monitoring_loop(self):
        """Background loop for continuous pattern monitoring"""
        while self.running:
            try:
                # Update pattern detection on recent data
                await self.pattern_detector.update_patterns()
                await asyncio.sleep(10)  # Update patterns every 10 seconds
                
            except Exception as e:
                logger.error(f"Error in pattern monitoring loop: {e}")
                await asyncio.sleep(30)
    
    async def _cache_cleanup_loop(self):
        """Background loop for cleaning up old cache entries"""
        while self.running:
            try:
                current_time = datetime.now()
                
                # Remove cache entries older than 1 hour
                expired_keys = []
                for request_id, result in self.analysis_cache.items():
                    if (current_time - result.timestamp).total_seconds() > 3600:
                        expired_keys.append(request_id)
                
                for key in expired_keys:
                    del self.analysis_cache[key]
                    if key in self.active_requests:
                        del self.active_requests[key]
                
                logger.debug(f"Cleaned up {len(expired_keys)} expired cache entries")
                await asyncio.sleep(300)  # Cleanup every 5 minutes
                
            except Exception as e:
                logger.error(f"Error in cache cleanup loop: {e}")
                await asyncio.sleep(600)
    
    async def _process_request(self, request_id: str, request: DataRequest):
        """Process a single data analysis request"""
        try:
            logger.info(f"Processing request: {request_id}")
            
            # Get data from MCP server
            raw_data = await self.mcp_client.get_data(
                request.source_type,
                request.source_name, 
                request.parameters
            )
            
            # Analyze the data
            analysis = await self.data_analyzer.analyze(
                raw_data,
                request.analysis_type,
                request.context
            )
            
            # Detect patterns
            patterns = await self.pattern_detector.detect_patterns(
                raw_data,
                request.analysis_type
            )
            
            # Create analysis result
            result = AnalysisResult(
                request_id=request_id,
                source_type=request.source_type,
                source_name=request.source_name,
                timestamp=datetime.now(),
                data=raw_data,
                insights=analysis.get("insights", []),
                confidence_score=analysis.get("confidence", 0.0),
                patterns_detected=patterns.get("patterns", []),
                anomalies=patterns.get("anomalies", []),
                recommendations=analysis.get("recommendations", [])
            )
            
            # Cache the result
            self.analysis_cache[request_id] = result
            
            # Remove from active requests
            if request_id in self.active_requests:
                del self.active_requests[request_id]
            
            logger.info(f"Completed processing request: {request_id}")
            
        except Exception as e:
            logger.error(f"Error processing request {request_id}: {e}")
            # Keep in active requests for retry
            
async def main():
    """Main entry point for running the information gathering agent"""
    logging.basicConfig(
        level=logging.INFO,
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
    )
    
    agent = InformationGatheringAgent()
    
    try:
        await agent.start()
    except KeyboardInterrupt:
        logger.info("Received interrupt signal")
    finally:
        await agent.stop()

if __name__ == "__main__":
    asyncio.run(main())