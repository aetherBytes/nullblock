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
from ..llm_service.factory import LLMServiceFactory, LLMRequest
from ..llm_service.router import TaskRequirements, OptimizationGoal, Priority
from ..llm_service.models import ModelCapability

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
        self.mcp_server_url = mcp_server_url
        self.mcp_client = MCPClient(mcp_server_url)
        self.data_analyzer = DataAnalyzer()
        self.pattern_detector = PatternDetector()
        self.llm_factory: Optional[LLMServiceFactory] = None
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
        
        # Initialize LLM factory for enhanced analysis
        self.llm_factory = LLMServiceFactory()
        await self.llm_factory.initialize()
        
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
        if self.llm_factory:
            await self.llm_factory.cleanup()
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
    
    async def analyze_market_trends(self, symbols: List[str], timeframe: str = "24h", enhance_with_llm: bool = False) -> Dict[str, Any]:
        """
        Convenience method for market trend analysis
        
        Args:
            symbols: List of token/asset symbols to analyze
            timeframe: Analysis timeframe (1h, 24h, 7d, etc.)
            enhance_with_llm: Whether to enhance analysis with LLM insights
            
        Returns:
            Dict with market trend insights and analysis
        """
        if not symbols:
            raise ValueError("Symbols list cannot be empty")
        
        # Use MCP client to get market data
        result = await self.mcp_client.call_tool("analyze_market_trends", {
            "symbols": symbols,
            "timeframe": timeframe
        })
        
        # Create AnalysisResult object with proper structure
        analysis_result = AnalysisResult(
            request_id=f"market_trends_{datetime.now().timestamp()}",
            source_type="price_oracle",
            source_name="coingecko",
            timestamp=datetime.now(),
            data=result,
            insights=result.get("insights", []),
            confidence_score=result.get("confidence_score", 0.85),
            patterns_detected=result.get("patterns", []),
            anomalies=result.get("anomalies", []),
            recommendations=result.get("recommendations", [])
        )
        
        # Enhance with LLM if requested
        if enhance_with_llm and self.llm_factory:
            try:
                llm_enhancement = await self._enhance_with_llm_analysis(
                    context=f"Market analysis for {', '.join(symbols)} over {timeframe}",
                    analysis_type="market_analysis",
                    capabilities=["REASONING", "DATA_ANALYSIS"]
                )
                analysis_result["enhanced_analysis"] = llm_enhancement["content"]
                analysis_result["model_used"] = llm_enhancement["model_used"]
            except Exception as e:
                logger.warning(f"LLM enhancement failed: {e}")
        
        return analysis_result
    
    async def detect_defi_opportunities(self, protocols: List[str], min_apr: float = 0.0, max_risk: float = 1.0) -> Dict[str, Any]:
        """
        Convenience method for DeFi opportunity detection
        
        Args:
            protocols: List of DeFi protocols to analyze
            min_apr: Minimum APR threshold
            max_risk: Maximum risk score threshold
            
        Returns:
            Dict with DeFi opportunity insights
        """
        # Use MCP client to get DeFi data
        result = await self.mcp_client.call_tool("detect_defi_opportunities", {
            "protocols": protocols,
            "min_apr": min_apr,
            "max_risk": max_risk
        })
        
        # Create AnalysisResult object with proper structure for DeFi opportunities
        opportunities_result = AnalysisResult(
            request_id=f"defi_opportunities_{datetime.now().timestamp()}",
            source_type="defi_protocol",
            source_name="uniswap",
            timestamp=datetime.now(),
            data=result,
            insights=result.get("insights", []),
            confidence_score=0.80,  # Default confidence for DeFi analysis
            patterns_detected=result.get("patterns", []),
            anomalies=[],  # DeFi opportunities don't typically have anomalies
            recommendations=result.get("recommendations", [])
        )
        
        return opportunities_result
    
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
    
    async def _enhance_with_llm_analysis(self, context: str, analysis_type: str, capabilities: List[str]) -> Dict[str, Any]:
        """
        Enhance analysis results using LLM
        
        Args:
            context: Analysis context and data
            analysis_type: Type of analysis being performed
            capabilities: Required LLM capabilities
            
        Returns:
            Dict with LLM enhancement results
        """
        if not self.llm_factory:
            raise ValueError("LLM factory not initialized")
        
        # Map string capabilities to ModelCapability enum
        capability_mapping = {
            "REASONING": ModelCapability.REASONING,
            "DATA_ANALYSIS": ModelCapability.DATA_ANALYSIS,
            "CONVERSATION": ModelCapability.CONVERSATION,
            "CREATIVE": ModelCapability.CREATIVE,
            "CODE": ModelCapability.CODE
        }
        
        mapped_capabilities = [capability_mapping.get(cap, ModelCapability.REASONING) for cap in capabilities]
        
        # Create LLM request
        request = LLMRequest(
            prompt=f"Analyze and provide insights for this {analysis_type}: {context}",
            system_prompt=f"You are a professional {analysis_type} expert. Provide clear, actionable insights.",
            max_tokens=300
        )
        
        # Create requirements
        requirements = TaskRequirements(
            required_capabilities=mapped_capabilities,
            optimization_goal=OptimizationGoal.BALANCED,
            priority=Priority.MEDIUM,
            task_type=analysis_type
        )
        
        # Generate enhanced analysis
        response = await self.llm_factory.generate(request, requirements)
        
        return {
            "content": response.content,
            "model_used": response.model_used,
            "latency_ms": response.latency_ms,
            "cost_estimate": response.cost_estimate
        }
    
    async def _execute_analysis_workflow(self, workflow_type: str, parameters: Dict[str, Any]) -> Dict[str, Any]:
        """
        Execute a multi-step analysis workflow
        
        Args:
            workflow_type: Type of workflow to execute
            parameters: Workflow parameters
            
        Returns:
            Dict with workflow results
        """
        workflow_steps = []
        
        if workflow_type == "comprehensive_market_analysis":
            workflow_steps = [
                {"tool": "get_market_data", "params": parameters},
                {"tool": "analyze_patterns", "params": parameters},
                {"tool": "enhance_with_llm", "params": parameters}
            ]
        
        results = []
        for step in workflow_steps:
            try:
                result = await self.mcp_client.call_tool(step["tool"], step["params"])
                results.append({
                    "step": step["tool"],
                    "status": "completed",
                    "data": result
                })
            except Exception as e:
                results.append({
                    "step": step["tool"],
                    "status": "failed",
                    "error": str(e)
                })
        
        return {
            "workflow_type": workflow_type,
            "workflow_results": results,
            "completed_steps": len([r for r in results if r["status"] == "completed"]),
            "total_steps": len(workflow_steps)
        }
    
    async def _gather_parallel_data(self, sources: List[str], parameters: Dict[str, Any]) -> List[Dict[str, Any]]:
        """
        Gather data from multiple sources in parallel
        
        Args:
            sources: List of data source names
            parameters: Common parameters for all sources
            
        Returns:
            List of results from each source
        """
        tasks = []
        for source in sources:
            task = self.mcp_client.call_tool(source, parameters)
            tasks.append(task)
        
        results = await asyncio.gather(*tasks, return_exceptions=True)
        
        # Convert exceptions to error dictionaries
        processed_results = []
        for i, result in enumerate(results):
            if isinstance(result, Exception):
                processed_results.append({
                    "source": sources[i],
                    "status": "error",
                    "error": str(result)
                })
            else:
                processed_results.append({
                    "source": sources[i],
                    "status": "success",
                    "data": result
                })
        
        return processed_results
    
    async def _execute_resilient_workflow(self, steps: List[str], options: Dict[str, Any]) -> Dict[str, Any]:
        """
        Execute workflow with error recovery
        
        Args:
            steps: List of workflow steps
            options: Execution options including retry settings
            
        Returns:
            Dict with workflow execution results
        """
        completed_steps = []
        failed_steps = []
        max_retries = options.get("max_retries", 1)
        
        for step in steps:
            success = False
            for attempt in range(max_retries + 1):
                try:
                    result = await self.mcp_client.call_tool(step, {})
                    completed_steps.append({
                        "step": step,
                        "result": result,
                        "attempt": attempt + 1
                    })
                    success = True
                    break
                except Exception as e:
                    if attempt == max_retries:
                        failed_steps.append({
                            "step": step,
                            "error": str(e),
                            "attempts": attempt + 1
                        })
                    else:
                        await asyncio.sleep(1)  # Brief delay before retry
            
        return {
            "completed_steps": completed_steps,
            "failed_steps": failed_steps,
            "recovery_attempted": len(failed_steps) > 0 and max_retries > 0,
            "success_rate": len(completed_steps) / len(steps) if steps else 1.0
        }
            
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