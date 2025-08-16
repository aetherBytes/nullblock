"""
Data Source Tools for MCP Server

Provides standardized interfaces for accessing various data sources including
price oracles, DeFi protocols, social sentiment feeds, and on-chain analytics.
"""

import asyncio
import aiohttp
import logging
from typing import Dict, List, Any, Optional, Union
from dataclasses import dataclass, asdict
from datetime import datetime, timedelta
import json
import time
from abc import ABC, abstractmethod

logger = logging.getLogger(__name__)

@dataclass
class DataSourceConfig:
    """Configuration for a data source"""
    name: str
    base_url: str
    api_key: Optional[str] = None
    rate_limit: float = 1.0  # requests per second
    timeout: int = 30
    headers: Dict[str, str] = None
    
    def __post_init__(self):
        if self.headers is None:
            self.headers = {}

@dataclass
class DataPoint:
    """Standardized data point"""
    timestamp: datetime
    value: float
    metadata: Dict[str, Any] = None
    
    def __post_init__(self):
        if self.metadata is None:
            self.metadata = {}

@dataclass
class DataSourceResponse:
    """Standardized response from data sources"""
    success: bool
    data: Union[List[DataPoint], Dict[str, Any]]
    source: str
    timestamp: datetime
    error: Optional[str] = None
    rate_limited: bool = False
    cached: bool = False

class BaseDataSource(ABC):
    """Abstract base class for data sources"""
    
    def __init__(self, config: DataSourceConfig):
        self.config = config
        self.session: Optional[aiohttp.ClientSession] = None
        self.last_request_time = 0.0
        
    async def initialize(self):
        """Initialize the data source"""
        self.session = aiohttp.ClientSession(
            timeout=aiohttp.ClientTimeout(total=self.config.timeout),
            headers=self.config.headers
        )
        
    async def cleanup(self):
        """Clean up resources"""
        if self.session:
            await self.session.close()
            
    async def _rate_limit(self):
        """Apply rate limiting"""
        current_time = time.time()
        time_since_last = current_time - self.last_request_time
        min_interval = 1.0 / self.config.rate_limit
        
        if time_since_last < min_interval:
            sleep_time = min_interval - time_since_last
            await asyncio.sleep(sleep_time)
            
        self.last_request_time = time.time()
    
    @abstractmethod
    async def get_data(self, parameters: Dict[str, Any]) -> DataSourceResponse:
        """Get data from the source"""
        pass

class PriceOracleSource(BaseDataSource):
    """Base class for price oracle data sources"""
    
    async def get_price_data(self, symbols: List[str], vs_currency: str = "usd", 
                           timeframe: str = "24h") -> DataSourceResponse:
        """Get price data for symbols"""
        parameters = {
            "symbols": symbols,
            "vs_currency": vs_currency,
            "timeframe": timeframe
        }
        return await self.get_data(parameters)

class CoingeckoSource(PriceOracleSource):
    """CoinGecko price oracle implementation"""
    
    def __init__(self):
        config = DataSourceConfig(
            name="coingecko",
            base_url="https://api.coingecko.com/api/v3",
            rate_limit=0.5,  # Free tier limit
            headers={"accept": "application/json"}
        )
        super().__init__(config)
    
    async def get_data(self, parameters: Dict[str, Any]) -> DataSourceResponse:
        """Get data from CoinGecko API"""
        try:
            await self._rate_limit()
            
            symbols = parameters.get("symbols", [])
            vs_currency = parameters.get("vs_currency", "usd")
            timeframe = parameters.get("timeframe", "24h")
            
            if not symbols:
                return DataSourceResponse(
                    success=False,
                    data={},
                    source="coingecko",
                    timestamp=datetime.now(),
                    error="No symbols provided"
                )
            
            # Convert symbols to CoinGecko IDs (simplified)
            symbol_ids = ",".join([symbol.lower() for symbol in symbols])
            
            endpoint = f"{self.config.base_url}/simple/price"
            params = {
                "ids": symbol_ids,
                "vs_currencies": vs_currency,
                "include_24hr_change": "true",
                "include_24hr_vol": "true",
                "include_last_updated_at": "true"
            }
            
            async with self.session.get(endpoint, params=params) as resp:
                if resp.status == 200:
                    data = await resp.json()
                    
                    # Convert to standardized format
                    standardized_data = []
                    for symbol_id, price_data in data.items():
                        standardized_data.append(DataPoint(
                            timestamp=datetime.fromtimestamp(price_data.get("last_updated_at", time.time())),
                            value=price_data.get(vs_currency, 0.0),
                            metadata={
                                "symbol": symbol_id,
                                "vs_currency": vs_currency,
                                "change_24h": price_data.get(f"{vs_currency}_24h_change", 0.0),
                                "volume_24h": price_data.get(f"{vs_currency}_24h_vol", 0.0)
                            }
                        ))
                    
                    return DataSourceResponse(
                        success=True,
                        data=standardized_data,
                        source="coingecko",
                        timestamp=datetime.now()
                    )
                else:
                    error_text = await resp.text()
                    return DataSourceResponse(
                        success=False,
                        data={},
                        source="coingecko",
                        timestamp=datetime.now(),
                        error=f"HTTP {resp.status}: {error_text}",
                        rate_limited=resp.status == 429
                    )
                    
        except Exception as e:
            logger.error(f"Error getting CoinGecko data: {e}")
            return DataSourceResponse(
                success=False,
                data={},
                source="coingecko",
                timestamp=datetime.now(),
                error=str(e)
            )

class ChainlinkSource(PriceOracleSource):
    """Chainlink price feed implementation"""
    
    def __init__(self):
        config = DataSourceConfig(
            name="chainlink",
            base_url="https://api.chain.link/v1",
            rate_limit=5.0,  # Higher rate limit
            headers={"accept": "application/json"}
        )
        super().__init__(config)
        
    async def get_data(self, parameters: Dict[str, Any]) -> DataSourceResponse:
        """Get data from Chainlink price feeds"""
        try:
            await self._rate_limit()
            
            symbols = parameters.get("symbols", [])
            
            if not symbols:
                return DataSourceResponse(
                    success=False,
                    data={},
                    source="chainlink",
                    timestamp=datetime.now(),
                    error="No symbols provided"
                )
            
            # For demo purposes, return mock Chainlink data
            # In production, this would connect to actual Chainlink nodes
            standardized_data = []
            for symbol in symbols:
                # Mock price data
                mock_price = hash(symbol) % 10000 / 100.0  # Generate consistent mock price
                standardized_data.append(DataPoint(
                    timestamp=datetime.now(),
                    value=mock_price,
                    metadata={
                        "symbol": symbol,
                        "feed_address": f"0x{hash(symbol):040x}",  # Mock address
                        "decimals": 8,
                        "description": f"{symbol}/USD Price Feed"
                    }
                ))
            
            return DataSourceResponse(
                success=True,
                data=standardized_data,
                source="chainlink",
                timestamp=datetime.now()
            )
            
        except Exception as e:
            logger.error(f"Error getting Chainlink data: {e}")
            return DataSourceResponse(
                success=False,
                data={},
                source="chainlink",
                timestamp=datetime.now(),
                error=str(e)
            )

class UniswapSource(BaseDataSource):
    """Uniswap DeFi protocol data source"""
    
    def __init__(self):
        config = DataSourceConfig(
            name="uniswap",
            base_url="https://api.thegraph.com/subgraphs/name/uniswap/uniswap-v3",
            rate_limit=2.0,
            headers={"Content-Type": "application/json"}
        )
        super().__init__(config)
    
    async def get_data(self, parameters: Dict[str, Any]) -> DataSourceResponse:
        """Get Uniswap protocol data"""
        try:
            await self._rate_limit()
            
            metrics = parameters.get("metrics", ["tvl", "volume"])
            timeframe = parameters.get("timeframe", "24h")
            
            # GraphQL query for Uniswap data
            query = """
            {
              uniswapDayDatas(first: 7, orderBy: date, orderDirection: desc) {
                date
                volumeUSD
                tvlUSD
                txCount
              }
            }
            """
            
            async with self.session.post(
                self.config.base_url,
                json={"query": query}
            ) as resp:
                if resp.status == 200:
                    data = await resp.json()
                    
                    if "errors" in data:
                        return DataSourceResponse(
                            success=False,
                            data={},
                            source="uniswap",
                            timestamp=datetime.now(),
                            error=f"GraphQL errors: {data['errors']}"
                        )
                    
                    # Convert to standardized format
                    day_data = data.get("data", {}).get("uniswapDayDatas", [])
                    standardized_data = []
                    
                    for day in day_data:
                        timestamp = datetime.fromtimestamp(int(day["date"]))
                        
                        if "tvl" in metrics:
                            standardized_data.append(DataPoint(
                                timestamp=timestamp,
                                value=float(day["tvlUSD"]),
                                metadata={
                                    "metric": "tvl",
                                    "protocol": "uniswap",
                                    "currency": "USD"
                                }
                            ))
                        
                        if "volume" in metrics:
                            standardized_data.append(DataPoint(
                                timestamp=timestamp,
                                value=float(day["volumeUSD"]),
                                metadata={
                                    "metric": "volume",
                                    "protocol": "uniswap",
                                    "currency": "USD",
                                    "tx_count": int(day["txCount"])
                                }
                            ))
                    
                    return DataSourceResponse(
                        success=True,
                        data=standardized_data,
                        source="uniswap",
                        timestamp=datetime.now()
                    )
                else:
                    error_text = await resp.text()
                    return DataSourceResponse(
                        success=False,
                        data={},
                        source="uniswap",
                        timestamp=datetime.now(),
                        error=f"HTTP {resp.status}: {error_text}"
                    )
                    
        except Exception as e:
            logger.error(f"Error getting Uniswap data: {e}")
            return DataSourceResponse(
                success=False,
                data={},
                source="uniswap",
                timestamp=datetime.now(),
                error=str(e)
            )

class TwitterSentimentSource(BaseDataSource):
    """Twitter sentiment data source"""
    
    def __init__(self, api_key: Optional[str] = None):
        config = DataSourceConfig(
            name="twitter_sentiment",
            base_url="https://api.twitter.com/2",
            api_key=api_key,
            rate_limit=0.1,  # Conservative rate limit
            headers={
                "Authorization": f"Bearer {api_key}" if api_key else "",
                "Content-Type": "application/json"
            }
        )
        super().__init__(config)
    
    async def get_data(self, parameters: Dict[str, Any]) -> DataSourceResponse:
        """Get Twitter sentiment data"""
        try:
            await self._rate_limit()
            
            keywords = parameters.get("keywords", [])
            timeframe = parameters.get("timeframe", "24h")
            
            if not keywords or not self.config.api_key:
                # Return mock sentiment data for demo
                standardized_data = []
                for keyword in keywords:
                    # Generate mock sentiment score
                    sentiment_score = (hash(keyword) % 200 - 100) / 100.0  # -1.0 to 1.0
                    
                    standardized_data.append(DataPoint(
                        timestamp=datetime.now(),
                        value=sentiment_score,
                        metadata={
                            "keyword": keyword,
                            "metric": "sentiment_score",
                            "source": "twitter",
                            "sample_size": hash(keyword) % 1000 + 100,  # Mock sample size
                            "confidence": 0.7 + (abs(sentiment_score) * 0.3)
                        }
                    ))
                
                return DataSourceResponse(
                    success=True,
                    data=standardized_data,
                    source="twitter_sentiment",
                    timestamp=datetime.now()
                )
            
            # Real Twitter API implementation would go here
            # For now, return mock data
            return DataSourceResponse(
                success=False,
                data={},
                source="twitter_sentiment",
                timestamp=datetime.now(),
                error="Twitter API integration not implemented"
            )
            
        except Exception as e:
            logger.error(f"Error getting Twitter sentiment data: {e}")
            return DataSourceResponse(
                success=False,
                data={},
                source="twitter_sentiment",
                timestamp=datetime.now(),
                error=str(e)
            )

class EtherscanSource(BaseDataSource):
    """Etherscan on-chain analytics data source"""
    
    def __init__(self, api_key: Optional[str] = None):
        config = DataSourceConfig(
            name="etherscan",
            base_url="https://api.etherscan.io/api",
            api_key=api_key,
            rate_limit=0.2,  # Free tier limit
            headers={"accept": "application/json"}
        )
        super().__init__(config)
    
    async def get_data(self, parameters: Dict[str, Any]) -> DataSourceResponse:
        """Get Etherscan on-chain data"""
        try:
            await self._rate_limit()
            
            address = parameters.get("address")
            metrics = parameters.get("metrics", ["balance"])
            
            if not address:
                return DataSourceResponse(
                    success=False,
                    data={},
                    source="etherscan",
                    timestamp=datetime.now(),
                    error="No address provided"
                )
            
            standardized_data = []
            
            if "balance" in metrics:
                # Get ETH balance
                params = {
                    "module": "account",
                    "action": "balance",
                    "address": address,
                    "tag": "latest",
                    "apikey": self.config.api_key or "demo"
                }
                
                async with self.session.get(self.config.base_url, params=params) as resp:
                    if resp.status == 200:
                        data = await resp.json()
                        
                        if data.get("status") == "1":
                            balance_wei = int(data.get("result", 0))
                            balance_eth = balance_wei / 10**18
                            
                            standardized_data.append(DataPoint(
                                timestamp=datetime.now(),
                                value=balance_eth,
                                metadata={
                                    "address": address,
                                    "metric": "eth_balance",
                                    "currency": "ETH",
                                    "block": "latest"
                                }
                            ))
            
            if "transactions" in metrics:
                # Get transaction count
                params = {
                    "module": "proxy",
                    "action": "eth_getTransactionCount",
                    "address": address,
                    "tag": "latest",
                    "apikey": self.config.api_key or "demo"
                }
                
                async with self.session.get(self.config.base_url, params=params) as resp:
                    if resp.status == 200:
                        data = await resp.json()
                        
                        if "result" in data:
                            tx_count = int(data["result"], 16)  # Hex to decimal
                            
                            standardized_data.append(DataPoint(
                                timestamp=datetime.now(),
                                value=float(tx_count),
                                metadata={
                                    "address": address,
                                    "metric": "transaction_count",
                                    "block": "latest"
                                }
                            ))
            
            return DataSourceResponse(
                success=True,
                data=standardized_data,
                source="etherscan",
                timestamp=datetime.now()
            )
            
        except Exception as e:
            logger.error(f"Error getting Etherscan data: {e}")
            return DataSourceResponse(
                success=False,
                data={},
                source="etherscan",
                timestamp=datetime.now(),
                error=str(e)
            )

class DataSourceManager:
    """Manages all data sources and provides unified interface"""
    
    def __init__(self):
        self.sources: Dict[str, BaseDataSource] = {}
        self.response_cache: Dict[str, DataSourceResponse] = {}
        self.cache_ttl = 60  # 1 minute default cache
        
        logger.info("DataSourceManager initialized")
    
    async def initialize(self):
        """Initialize all data sources"""
        try:
            # Initialize price oracle sources
            self.sources["coingecko"] = CoingeckoSource()
            self.sources["chainlink"] = ChainlinkSource()
            
            # Initialize DeFi protocol sources
            self.sources["uniswap"] = UniswapSource()
            
            # Initialize social sentiment sources
            self.sources["twitter_sentiment"] = TwitterSentimentSource()
            
            # Initialize on-chain analytics sources
            self.sources["etherscan"] = EtherscanSource()
            
            # Initialize all sources
            for source in self.sources.values():
                await source.initialize()
            
            logger.info(f"Initialized {len(self.sources)} data sources")
            
        except Exception as e:
            logger.error(f"Error initializing data sources: {e}")
            raise
    
    async def cleanup(self):
        """Clean up all data sources"""
        for source in self.sources.values():
            await source.cleanup()
        
        logger.info("All data sources cleaned up")
    
    async def get_data(self, source_type: str, source_name: str, 
                      parameters: Dict[str, Any]) -> DataSourceResponse:
        """Get data from specified source"""
        # Check cache first
        cache_key = f"{source_type}/{source_name}:{hash(json.dumps(parameters, sort_keys=True))}"
        cached_response = self._get_cached_response(cache_key)
        
        if cached_response:
            logger.debug(f"Using cached response for {source_type}/{source_name}")
            cached_response.cached = True
            return cached_response
        
        # Get data from source
        source_key = source_name
        if source_type == "social_sentiment":
            source_key = f"{source_name}_sentiment"
        
        if source_key not in self.sources:
            return DataSourceResponse(
                success=False,
                data={},
                source=source_name,
                timestamp=datetime.now(),
                error=f"Unknown data source: {source_key}"
            )
        
        try:
            response = await self.sources[source_key].get_data(parameters)
            
            # Cache successful responses
            if response.success:
                self._cache_response(cache_key, response)
            
            return response
            
        except Exception as e:
            logger.error(f"Error getting data from {source_key}: {e}")
            return DataSourceResponse(
                success=False,
                data={},
                source=source_name,
                timestamp=datetime.now(),
                error=str(e)
            )
    
    def get_available_sources(self) -> Dict[str, List[str]]:
        """Get list of available data sources by type"""
        return {
            "price_oracle": ["coingecko", "chainlink"],
            "defi_protocol": ["uniswap"],
            "social_sentiment": ["twitter"],
            "onchain_analytics": ["etherscan"]
        }
    
    def _get_cached_response(self, cache_key: str) -> Optional[DataSourceResponse]:
        """Get cached response if still valid"""
        if cache_key not in self.response_cache:
            return None
        
        cached_response = self.response_cache[cache_key]
        cache_age = (datetime.now() - cached_response.timestamp).total_seconds()
        
        if cache_age <= self.cache_ttl:
            return cached_response
        else:
            # Remove expired cache entry
            del self.response_cache[cache_key]
            return None
    
    def _cache_response(self, cache_key: str, response: DataSourceResponse):
        """Cache response"""
        self.response_cache[cache_key] = response
        
        # Limit cache size
        if len(self.response_cache) > 1000:
            # Remove oldest entries
            oldest_keys = sorted(
                self.response_cache.keys(),
                key=lambda k: self.response_cache[k].timestamp
            )[:100]  # Remove 100 oldest
            
            for key in oldest_keys:
                del self.response_cache[key]
    
    async def get_historical_data(self, source_type: str, source_name: str, 
                                 parameters: Dict[str, Any], 
                                 start_time: datetime, end_time: datetime) -> List[DataPoint]:
        """Get historical data over time range"""
        # This would implement historical data fetching
        # For now, return empty list
        logger.warning(f"Historical data not implemented for {source_type}/{source_name}")
        return []
    
    def get_source_status(self) -> Dict[str, Dict[str, Any]]:
        """Get status of all data sources"""
        status = {}
        
        for name, source in self.sources.items():
            status[name] = {
                "name": source.config.name,
                "base_url": source.config.base_url,
                "rate_limit": source.config.rate_limit,
                "last_request_time": source.last_request_time,
                "initialized": source.session is not None
            }
        
        return status