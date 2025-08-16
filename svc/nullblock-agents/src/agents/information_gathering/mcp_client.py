"""
MCP Client Component

Handles communication with the Nullblock MCP server for data source access.
Provides standardized interface for accessing various data sources and APIs.
"""

import aiohttp
import asyncio
import logging
from typing import Dict, List, Any, Optional
from dataclasses import dataclass
from datetime import datetime, timedelta
import json

logger = logging.getLogger(__name__)

@dataclass
class DataSourceConfig:
    """Configuration for a data source"""
    source_type: str
    source_name: str
    endpoint: str
    parameters: Dict[str, Any]
    cache_ttl: int = 60  # Cache time-to-live in seconds
    rate_limit: float = 1.0  # Requests per second
    
@dataclass
class MCPResponse:
    """Standardized MCP response format"""
    success: bool
    data: Dict[str, Any]
    timestamp: datetime
    source: str
    error: Optional[str] = None
    cached: bool = False

class MCPClient:
    """
    Client for communicating with Nullblock MCP server
    
    Handles:
    - Data source registration and management
    - Request routing and load balancing  
    - Response caching and optimization
    - Error handling and retry logic
    """
    
    def __init__(self, server_url: str = "http://localhost:8000"):
        self.server_url = server_url.rstrip('/')
        self.session: Optional[aiohttp.ClientSession] = None
        self.data_sources: Dict[str, DataSourceConfig] = {}
        self.response_cache: Dict[str, MCPResponse] = {}
        self.rate_limiters: Dict[str, float] = {}  # Last request time per source
        self.connected = False
        
        logger.info(f"MCPClient initialized for server: {server_url}")
    
    async def connect(self):
        """Establish connection to MCP server"""
        try:
            self.session = aiohttp.ClientSession(
                timeout=aiohttp.ClientTimeout(total=30),
                headers={
                    'Content-Type': 'application/json',
                    'User-Agent': 'Nullblock-InformationGatheringAgent/1.0'
                }
            )
            
            # Test connection and get available data sources
            await self._discover_data_sources()
            self.connected = True
            
            logger.info("Successfully connected to MCP server")
            
        except Exception as e:
            logger.error(f"Failed to connect to MCP server: {e}")
            raise
    
    async def disconnect(self):
        """Close connection to MCP server"""
        if self.session:
            await self.session.close()
            self.session = None
            
        self.connected = False
        logger.info("Disconnected from MCP server")
    
    async def get_data(self, source_type: str, source_name: str, parameters: Dict[str, Any]) -> Dict[str, Any]:
        """
        Get data from a specific source via MCP server
        
        Args:
            source_type: Type of data source ('price_oracle', 'defi_protocol', etc.)
            source_name: Specific source name ('chainlink', 'uniswap', etc.)
            parameters: Query parameters for the data source
            
        Returns:
            Raw data from the data source
        """
        if not self.connected:
            raise ConnectionError("Not connected to MCP server")
        
        # Check cache first
        cache_key = self._generate_cache_key(source_type, source_name, parameters)
        cached_response = self._get_cached_response(cache_key)
        if cached_response:
            logger.debug(f"Using cached response for {source_type}/{source_name}")
            return cached_response.data
        
        # Apply rate limiting
        await self._apply_rate_limit(source_name)
        
        try:
            # Route request based on source type
            if source_type == "price_oracle":
                response = await self._get_price_data(source_name, parameters)
            elif source_type == "defi_protocol":
                response = await self._get_defi_data(source_name, parameters)
            elif source_type == "social_sentiment":
                response = await self._get_social_data(source_name, parameters)
            elif source_type == "onchain_analytics":
                response = await self._get_onchain_data(source_name, parameters)
            else:
                response = await self._get_generic_data(source_type, source_name, parameters)
            
            # Check if response was successful
            if not response.success:
                raise ConnectionError(f"Data source request failed: {response.error}")
            
            # Cache successful response
            self._cache_response(cache_key, response)
            
            return response.data
            
        except Exception as e:
            logger.error(f"Error getting data from {source_type}/{source_name}: {e}")
            raise
    
    async def get_available_sources(self) -> Dict[str, List[str]]:
        """Get list of available data sources by type"""
        try:
            endpoint = f"{self.server_url}/mcp/data-sources"
            async with self.session.get(endpoint) as resp:
                if resp.status == 200:
                    data = await resp.json()
                    sources = data.get('sources', {})
                    if not sources:
                        raise ConnectionError("No data sources available from MCP server")
                    return sources
                else:
                    raise ConnectionError(f"Failed to get data sources: HTTP {resp.status}")
                    
        except Exception as e:
            logger.error(f"Error getting available sources: {e}")
            raise ConnectionError(f"Failed to get available data sources: {e}")
    
    async def health_check(self) -> Dict[str, Any]:
        """Check MCP server health and status"""
        try:
            endpoint = f"{self.server_url}/health"
            async with self.session.get(endpoint) as resp:
                if resp.status == 200:
                    health_data = await resp.json()
                    if health_data.get('status') != 'healthy':
                        raise ConnectionError(f"MCP server unhealthy: {health_data}")
                    return health_data
                else:
                    raise ConnectionError(f"MCP server health check failed: HTTP {resp.status}")
                    
        except Exception as e:
            logger.error(f"Health check failed: {e}")
            raise ConnectionError(f"MCP server health check failed: {e}")
    
    async def _discover_data_sources(self):
        """Discover available data sources from MCP server"""
        try:
            sources = await self.get_available_sources()
            
            # Register discovered sources
            for source_type, source_names in sources.items():
                for source_name in source_names:
                    config = DataSourceConfig(
                        source_type=source_type,
                        source_name=source_name,
                        endpoint=f"/mcp/data/{source_type}/{source_name}",
                        parameters={},
                        cache_ttl=self._get_cache_ttl_for_source(source_type),
                        rate_limit=self._get_rate_limit_for_source(source_name)
                    )
                    
                    source_key = f"{source_type}/{source_name}"
                    self.data_sources[source_key] = config
            
            logger.info(f"Discovered {len(self.data_sources)} data sources")
            
        except Exception as e:
            logger.warning(f"Could not discover data sources: {e}")
            # Continue with default sources
            self._setup_default_sources()
    
    def _setup_default_sources(self):
        """Setup default data sources if discovery fails"""
        default_sources = [
            DataSourceConfig("price_oracle", "chainlink", "/mcp/data/price_oracle/chainlink", {}),
            DataSourceConfig("price_oracle", "coingecko", "/mcp/data/price_oracle/coingecko", {}),
            DataSourceConfig("defi_protocol", "uniswap", "/mcp/data/defi_protocol/uniswap", {}),
            DataSourceConfig("defi_protocol", "aave", "/mcp/data/defi_protocol/aave", {}),
            DataSourceConfig("social_sentiment", "twitter", "/mcp/data/social_sentiment/twitter", {}),
            DataSourceConfig("onchain_analytics", "etherscan", "/mcp/data/onchain_analytics/etherscan", {})
        ]
        
        for config in default_sources:
            source_key = f"{config.source_type}/{config.source_name}"
            self.data_sources[source_key] = config
        
        logger.info(f"Setup {len(default_sources)} default data sources")
    
    async def _get_price_data(self, source_name: str, parameters: Dict[str, Any]) -> MCPResponse:
        """Get price data from price oracle sources"""
        endpoint = f"{self.server_url}/mcp/data/price_oracle/{source_name}"
        
        # Standardize price oracle parameters
        query_params = {
            'symbols': parameters.get('symbols', []),
            'timeframe': parameters.get('timeframe', '24h'),
            'vs_currency': parameters.get('vs_currency', 'usd')
        }
        
        return await self._make_request(endpoint, query_params, f"price_oracle/{source_name}")
    
    async def _get_defi_data(self, source_name: str, parameters: Dict[str, Any]) -> MCPResponse:
        """Get DeFi protocol data"""
        endpoint = f"{self.server_url}/mcp/data/defi_protocol/{source_name}"
        
        # Standardize DeFi parameters
        query_params = {
            'protocol': source_name,
            'metrics': parameters.get('metrics', ['tvl', 'volume', 'fees']),
            'timeframe': parameters.get('timeframe', '24h')
        }
        
        return await self._make_request(endpoint, query_params, f"defi_protocol/{source_name}")
    
    async def _get_social_data(self, source_name: str, parameters: Dict[str, Any]) -> MCPResponse:
        """Get social sentiment data"""
        endpoint = f"{self.server_url}/mcp/data/social_sentiment/{source_name}"
        
        # Standardize social sentiment parameters
        query_params = {
            'keywords': parameters.get('keywords', []),
            'sentiment_type': parameters.get('sentiment_type', 'overall'),
            'timeframe': parameters.get('timeframe', '24h')
        }
        
        return await self._make_request(endpoint, query_params, f"social_sentiment/{source_name}")
    
    async def _get_onchain_data(self, source_name: str, parameters: Dict[str, Any]) -> MCPResponse:
        """Get on-chain analytics data"""
        endpoint = f"{self.server_url}/mcp/data/onchain_analytics/{source_name}"
        
        # Standardize on-chain parameters
        query_params = {
            'address': parameters.get('address'),
            'contract': parameters.get('contract'),
            'metrics': parameters.get('metrics', ['balance', 'transactions']),
            'timeframe': parameters.get('timeframe', '24h')
        }
        
        return await self._make_request(endpoint, query_params, f"onchain_analytics/{source_name}")
    
    async def _get_generic_data(self, source_type: str, source_name: str, parameters: Dict[str, Any]) -> MCPResponse:
        """Get data from generic/unknown source types"""
        endpoint = f"{self.server_url}/mcp/data/{source_type}/{source_name}"
        
        return await self._make_request(endpoint, parameters, f"{source_type}/{source_name}")
    
    async def _make_request(self, endpoint: str, parameters: Dict[str, Any], source: str) -> MCPResponse:
        """Make HTTP request to MCP server"""
        try:
            # Use POST for complex queries, GET for simple ones
            if len(json.dumps(parameters)) > 200:  # Arbitrary threshold
                async with self.session.post(endpoint, json=parameters) as resp:
                    data = await resp.json() if resp.content_type == 'application/json' else {}
                    success = resp.status == 200
            else:
                async with self.session.get(endpoint, params=parameters) as resp:
                    data = await resp.json() if resp.content_type == 'application/json' else {}
                    success = resp.status == 200
            
            if not success:
                logger.error(f"Request failed for {source}: HTTP {resp.status}")
                return MCPResponse(
                    success=False,
                    data={},
                    timestamp=datetime.now(),
                    source=source,
                    error=f"HTTP {resp.status}: {data.get('detail', 'Unknown error')}"
                )
            
            return MCPResponse(
                success=True,
                data=data,
                timestamp=datetime.now(),
                source=source
            )
            
        except asyncio.TimeoutError:
            logger.error(f"Timeout requesting data from {source}")
            return MCPResponse(
                success=False,
                data={},
                timestamp=datetime.now(),
                source=source,
                error="Request timeout"
            )
            
        except Exception as e:
            logger.error(f"Error requesting data from {source}: {e}")
            return MCPResponse(
                success=False,
                data={},
                timestamp=datetime.now(),
                source=source,
                error=str(e)
            )
    
    async def _apply_rate_limit(self, source_name: str):
        """Apply rate limiting for data source requests"""
        source_key = f"rate_limit_{source_name}"
        current_time = asyncio.get_event_loop().time()
        
        if source_key in self.rate_limiters:
            last_request_time = self.rate_limiters[source_key]
            
            # Get rate limit for this source
            rate_limit = 1.0  # Default 1 request per second
            for source_config in self.data_sources.values():
                if source_config.source_name == source_name:
                    rate_limit = source_config.rate_limit
                    break
            
            time_since_last = current_time - last_request_time
            min_interval = 1.0 / rate_limit
            
            if time_since_last < min_interval:
                sleep_time = min_interval - time_since_last
                logger.debug(f"Rate limiting {source_name}: sleeping {sleep_time:.2f}s")
                await asyncio.sleep(sleep_time)
        
        self.rate_limiters[source_key] = asyncio.get_event_loop().time()
    
    def _generate_cache_key(self, source_type: str, source_name: str, parameters: Dict[str, Any]) -> str:
        """Generate cache key for request"""
        # Create deterministic key from parameters
        param_str = json.dumps(parameters, sort_keys=True)
        return f"{source_type}/{source_name}:{hash(param_str)}"
    
    def _get_cached_response(self, cache_key: str) -> Optional[MCPResponse]:
        """Get cached response if still valid"""
        if cache_key not in self.response_cache:
            return None
        
        cached_response = self.response_cache[cache_key]
        
        # Check if cache is still valid
        source_config = None
        for config in self.data_sources.values():
            if f"{config.source_type}/{config.source_name}" in cache_key:
                source_config = config
                break
        
        cache_ttl = source_config.cache_ttl if source_config else 60
        cache_age = (datetime.now() - cached_response.timestamp).total_seconds()
        
        if cache_age <= cache_ttl:
            cached_response.cached = True
            return cached_response
        else:
            # Remove expired cache entry
            del self.response_cache[cache_key]
            return None
    
    def _cache_response(self, cache_key: str, response: MCPResponse):
        """Cache successful response"""
        self.response_cache[cache_key] = response
        
        # Limit cache size (keep last 100 entries)
        if len(self.response_cache) > 100:
            # Remove oldest entries
            oldest_keys = sorted(
                self.response_cache.keys(),
                key=lambda k: self.response_cache[k].timestamp
            )[:20]  # Remove 20 oldest
            
            for key in oldest_keys:
                del self.response_cache[key]
    
    def _get_cache_ttl_for_source(self, source_type: str) -> int:
        """Get appropriate cache TTL for source type"""
        cache_ttls = {
            'price_oracle': 30,     # 30 seconds for price data
            'defi_protocol': 300,   # 5 minutes for DeFi data
            'social_sentiment': 600, # 10 minutes for social data
            'onchain_analytics': 120 # 2 minutes for on-chain data
        }
        
        return cache_ttls.get(source_type, 60)  # Default 1 minute
    
    def _get_rate_limit_for_source(self, source_name: str) -> float:
        """Get appropriate rate limit for source"""
        rate_limits = {
            'chainlink': 5.0,      # 5 requests per second
            'coingecko': 0.5,      # 0.5 requests per second (free tier)
            'etherscan': 0.2,      # 0.2 requests per second (free tier)
            'twitter': 0.1,        # 0.1 requests per second
            'uniswap': 2.0,        # 2 requests per second
            'aave': 2.0            # 2 requests per second
        }
        
        return rate_limits.get(source_name, 1.0)  # Default 1 request per second
    
    async def get_historical_data(self, source_type: str, source_name: str, parameters: Dict[str, Any], 
                                 start_time: datetime, end_time: datetime) -> List[Dict[str, Any]]:
        """Get historical data over a time range"""
        historical_data = []
        
        try:
            # Add time range to parameters
            time_params = parameters.copy()
            time_params.update({
                'start_time': start_time.isoformat(),
                'end_time': end_time.isoformat(),
                'historical': True
            })
            
            endpoint = f"{self.server_url}/mcp/data/{source_type}/{source_name}/historical"
            response = await self._make_request(endpoint, time_params, f"{source_type}/{source_name}")
            
            if response.success:
                # Expect historical data to be in 'data' field as list
                historical_data = response.data.get('data', [])
            
        except Exception as e:
            logger.error(f"Error getting historical data: {e}")
        
        return historical_data
    
    async def stream_data(self, source_type: str, source_name: str, parameters: Dict[str, Any],
                         callback, interval: float = 1.0):
        """Stream real-time data from source (WebSocket or polling)"""
        try:
            # Try WebSocket first
            ws_endpoint = f"{self.server_url.replace('http', 'ws')}/mcp/stream/{source_type}/{source_name}"
            
            try:
                async with self.session.ws_connect(ws_endpoint) as ws:
                    # Send subscription parameters
                    await ws.send_json(parameters)
                    
                    async for msg in ws:
                        if msg.type == aiohttp.WSMsgType.TEXT:
                            data = json.loads(msg.data)
                            await callback(data)
                        elif msg.type == aiohttp.WSMsgType.ERROR:
                            logger.error(f"WebSocket error: {ws.exception()}")
                            break
                            
            except Exception as e:
                logger.warning(f"WebSocket streaming failed, falling back to polling: {e}")
                
                # Fallback to polling
                while True:
                    try:
                        data = await self.get_data(source_type, source_name, parameters)
                        await callback(data)
                        await asyncio.sleep(interval)
                    except Exception as poll_error:
                        logger.error(f"Polling error: {poll_error}")
                        await asyncio.sleep(interval * 2)  # Back off on error
                        
        except Exception as e:
            logger.error(f"Error in data streaming: {e}")
    
    def get_connection_status(self) -> Dict[str, Any]:
        """Get current connection status and statistics"""
        return {
            'connected': self.connected,
            'server_url': self.server_url,
            'data_sources_count': len(self.data_sources),
            'cached_responses': len(self.response_cache),
            'rate_limiters_active': len(self.rate_limiters)
        }