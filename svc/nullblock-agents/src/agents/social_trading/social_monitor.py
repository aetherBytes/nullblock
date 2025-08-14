"""
Social media monitoring agent for X/Twitter, GMGN, and meme coin signals
"""

import logging
import asyncio
import aiohttp
import json
import time
from typing import Dict, List, Optional, Any, Tuple
from datetime import datetime, timedelta
from dataclasses import dataclass
from pydantic import BaseModel, Field
import re
from urllib.parse import urlparse

logger = logging.getLogger(__name__)


@dataclass
class SocialSignal:
    """Social media signal data"""
    source: str  # 'twitter', 'gmgn', 'dextools', etc.
    token_symbol: str
    token_address: Optional[str]
    sentiment_score: float  # -1.0 to 1.0
    engagement_score: float  # 0.0 to 1.0
    content: str
    author: str
    timestamp: datetime
    url: Optional[str] = None
    metrics: Dict[str, Any] = None


class TwitterMonitor:
    """Monitor Twitter/X for crypto signals"""
    
    def __init__(self, bearer_token: Optional[str] = None):
        self.bearer_token = bearer_token
        self.api_base = "https://api.twitter.com/2"
        self.session: Optional[aiohttp.ClientSession] = None
        self.logger = logging.getLogger(f"{__name__}.TwitterMonitor")
        
        # Keywords and accounts to monitor
        self.crypto_keywords = [
            "$SOL", "$BONK", "$WIF", "$PEPE", "$DOGE", "$SHIB",
            "solana", "meme coin", "memecoin", "pump.fun", "raydium"
        ]
        
        self.monitored_accounts = [
            "elonmusk", "VitalikButerin", "justinsuntron",
            "cz_binance", "SBF_FTX", "stablekwon"
        ]
    
    async def _ensure_session(self):
        """Ensure HTTP session is active"""
        if not self.session:
            headers = {}
            if self.bearer_token:
                headers["Authorization"] = f"Bearer {self.bearer_token}"
            self.session = aiohttp.ClientSession(headers=headers)
    
    async def search_tweets(self, query: str, limit: int = 50) -> List[SocialSignal]:
        """Search tweets for crypto-related content"""
        try:
            await self._ensure_session()
            
            # Mock implementation for MVP (replace with real Twitter API)
            signals = []
            
            # Generate mock tweets for testing
            mock_tweets = [
                {
                    "text": "Just bought some $BONK! This meme coin is going to the moon! 🚀",
                    "author": "crypto_trader_123",
                    "created_at": datetime.now() - timedelta(minutes=5),
                    "public_metrics": {"like_count": 25, "retweet_count": 8, "reply_count": 3}
                },
                {
                    "text": "$WIF looking bullish on the charts. Solana memes are heating up again",
                    "author": "meme_analyst",
                    "created_at": datetime.now() - timedelta(minutes=15),
                    "public_metrics": {"like_count": 45, "retweet_count": 12, "reply_count": 7}
                },
                {
                    "text": "Dump your bags! $PEPE is going to zero. Market manipulation everywhere.",
                    "author": "bear_market_bob",
                    "created_at": datetime.now() - timedelta(minutes=30),
                    "public_metrics": {"like_count": 8, "retweet_count": 2, "reply_count": 15}
                }
            ]
            
            for tweet in mock_tweets:
                # Extract token symbols
                token_symbols = re.findall(r'\$([A-Z]{3,10})', tweet["text"])
                
                for symbol in token_symbols:
                    # Calculate sentiment (mock implementation)
                    bullish_words = ["moon", "bullish", "pump", "buy", "rocket", "🚀"]
                    bearish_words = ["dump", "crash", "sell", "bear", "zero", "scam"]
                    
                    sentiment = 0.0
                    text_lower = tweet["text"].lower()
                    
                    for word in bullish_words:
                        if word in text_lower:
                            sentiment += 0.2
                    
                    for word in bearish_words:
                        if word in text_lower:
                            sentiment -= 0.3
                    
                    sentiment = max(-1.0, min(1.0, sentiment))
                    
                    # Calculate engagement score
                    metrics = tweet["public_metrics"]
                    total_engagement = metrics["like_count"] + metrics["retweet_count"] + metrics["reply_count"]
                    engagement_score = min(1.0, total_engagement / 100.0)  # Normalize to 0-1
                    
                    signal = SocialSignal(
                        source="twitter",
                        token_symbol=symbol,
                        token_address=None,
                        sentiment_score=sentiment,
                        engagement_score=engagement_score,
                        content=tweet["text"],
                        author=tweet["author"],
                        timestamp=tweet["created_at"],
                        url=f"https://twitter.com/{tweet['author']}/status/123456789",
                        metrics=metrics
                    )
                    
                    signals.append(signal)
            
            return signals
            
        except Exception as e:
            self.logger.error(f"Failed to search tweets: {e}")
            return []
    
    async def monitor_accounts(self) -> List[SocialSignal]:
        """Monitor specific Twitter accounts for crypto signals"""
        signals = []
        
        for account in self.monitored_accounts:
            try:
                # Mock implementation - would use Twitter API to get recent tweets
                # For now, return empty list
                pass
            except Exception as e:
                self.logger.error(f"Failed to monitor account {account}: {e}")
        
        return signals


class GMGNMonitor:
    """Monitor GMGN.ai for Solana token data"""
    
    def __init__(self):
        self.api_base = "https://gmgn.ai/api"
        self.session: Optional[aiohttp.ClientSession] = None
        self.logger = logging.getLogger(f"{__name__}.GMGNMonitor")
    
    async def _ensure_session(self):
        """Ensure HTTP session is active"""
        if not self.session:
            self.session = aiohttp.ClientSession()
    
    async def get_trending_tokens(self, limit: int = 20) -> List[SocialSignal]:
        """Get trending Solana tokens from GMGN"""
        try:
            await self._ensure_session()
            
            # Mock implementation for MVP
            trending_tokens = [
                {
                    "symbol": "BONK",
                    "address": "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",
                    "price_change_24h": 15.2,
                    "volume_24h": 2500000,
                    "market_cap": 180000000,
                    "trend_score": 0.85
                },
                {
                    "symbol": "WIF", 
                    "address": "EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm",
                    "price_change_24h": -8.5,
                    "volume_24h": 1800000,
                    "market_cap": 650000000,
                    "trend_score": 0.72
                },
                {
                    "symbol": "POPCAT",
                    "address": "7GCihgDB8fe6KNjn2MYtkzZcRjQy3t9GHdC8uHYmW2hr", 
                    "price_change_24h": 25.8,
                    "volume_24h": 950000,
                    "market_cap": 45000000,
                    "trend_score": 0.91
                }
            ]
            
            signals = []
            for token in trending_tokens:
                # Convert trend data to social signal
                sentiment = 0.0
                if token["price_change_24h"] > 10:
                    sentiment = 0.8
                elif token["price_change_24h"] > 0:
                    sentiment = 0.4
                elif token["price_change_24h"] > -10:
                    sentiment = -0.2
                else:
                    sentiment = -0.6
                
                signal = SocialSignal(
                    source="gmgn",
                    token_symbol=token["symbol"],
                    token_address=token["address"],
                    sentiment_score=sentiment,
                    engagement_score=token["trend_score"],
                    content=f"GMGN trending: {token['symbol']} {token['price_change_24h']:+.1f}% (24h)",
                    author="gmgn_api",
                    timestamp=datetime.now(),
                    url=f"https://gmgn.ai/sol/token/{token['address']}",
                    metrics={
                        "price_change_24h": token["price_change_24h"],
                        "volume_24h": token["volume_24h"],
                        "market_cap": token["market_cap"],
                        "trend_score": token["trend_score"]
                    }
                )
                
                signals.append(signal)
            
            return signals
            
        except Exception as e:
            self.logger.error(f"Failed to get trending tokens from GMGN: {e}")
            return []


class DexToolsMonitor:
    """Monitor DEXTools for Solana token data and social metrics"""
    
    def __init__(self, api_key: Optional[str] = None):
        self.api_key = api_key
        self.api_base = "https://public-api.dextools.io"
        self.session: Optional[aiohttp.ClientSession] = None
        self.logger = logging.getLogger(f"{__name__}.DexToolsMonitor")
    
    async def _ensure_session(self):
        """Ensure HTTP session is active"""
        if not self.session:
            headers = {}
            if self.api_key:
                headers["X-API-Key"] = self.api_key
            self.session = aiohttp.ClientSession(headers=headers)
    
    async def get_hot_pairs(self, chain: str = "solana") -> List[SocialSignal]:
        """Get hot trading pairs from DEXTools"""
        try:
            await self._ensure_session()
            
            # Mock implementation for MVP
            hot_pairs = [
                {
                    "token": {"symbol": "BONK", "address": "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263"},
                    "priceUsd": "0.000025",
                    "priceChange24h": 12.5,
                    "volume24h": 2100000,
                    "socialScore": 85,
                    "holders": 15420
                },
                {
                    "token": {"symbol": "MYRO", "address": "HhJpBhRRn4g56VsyLuT8DL5Bv31HkXqsrahTTUCZeZg4"},
                    "priceUsd": "0.125",
                    "priceChange24h": -5.2,
                    "volume24h": 850000,
                    "socialScore": 68,
                    "holders": 8750
                }
            ]
            
            signals = []
            for pair in hot_pairs:
                token = pair["token"]
                
                # Calculate sentiment based on price change and social score
                price_sentiment = 0.0
                if pair["priceChange24h"] > 5:
                    price_sentiment = 0.6
                elif pair["priceChange24h"] > 0:
                    price_sentiment = 0.2
                else:
                    price_sentiment = -0.3
                
                social_sentiment = (pair["socialScore"] - 50) / 50.0  # Normalize to -1 to 1
                
                overall_sentiment = (price_sentiment + social_sentiment) / 2
                overall_sentiment = max(-1.0, min(1.0, overall_sentiment))
                
                signal = SocialSignal(
                    source="dextools",
                    token_symbol=token["symbol"],
                    token_address=token["address"],
                    sentiment_score=overall_sentiment,
                    engagement_score=pair["socialScore"] / 100.0,
                    content=f"DEXTools hot pair: {token['symbol']} ${pair['priceUsd']} ({pair['priceChange24h']:+.1f}%)",
                    author="dextools_api",
                    timestamp=datetime.now(),
                    url=f"https://www.dextools.io/app/en/solana/pair-explorer/{token['address']}",
                    metrics={
                        "price_usd": pair["priceUsd"],
                        "price_change_24h": pair["priceChange24h"],
                        "volume_24h": pair["volume24h"],
                        "social_score": pair["socialScore"],
                        "holders": pair["holders"]
                    }
                )
                
                signals.append(signal)
            
            return signals
            
        except Exception as e:
            self.logger.error(f"Failed to get hot pairs from DEXTools: {e}")
            return []


class SocialMonitorAgent:
    """Main social monitoring agent that aggregates signals from multiple sources"""
    
    def __init__(self, config: Dict[str, Any] = None):
        self.config = config or {}
        self.logger = logging.getLogger(__name__)
        
        # Initialize monitors
        self.twitter_monitor = TwitterMonitor(
            bearer_token=self.config.get("twitter_bearer_token")
        )
        self.gmgn_monitor = GMGNMonitor()
        self.dextools_monitor = DexToolsMonitor(
            api_key=self.config.get("dextools_api_key")
        )
        
        # Signal cache
        self.signal_cache: Dict[str, List[SocialSignal]] = {}
        
        # Running state
        self.is_running = False
        self.monitoring_task: Optional[asyncio.Task] = None
        
        # Monitoring interval
        self.update_interval = self.config.get("update_interval", 60)  # 1 minute default
    
    async def start_monitoring(self):
        """Start social media monitoring"""
        if self.is_running:
            return
        
        self.is_running = True
        self.monitoring_task = asyncio.create_task(self._monitoring_loop())
        self.logger.info("Started social media monitoring")
    
    async def stop_monitoring(self):
        """Stop social media monitoring"""
        if not self.is_running:
            return
        
        self.is_running = False
        if self.monitoring_task:
            self.monitoring_task.cancel()
            try:
                await self.monitoring_task
            except asyncio.CancelledError:
                pass
        
        self.logger.info("Stopped social media monitoring")
    
    async def _monitoring_loop(self):
        """Main monitoring loop"""
        while self.is_running:
            try:
                await self._collect_all_signals()
                await asyncio.sleep(self.update_interval)
            except asyncio.CancelledError:
                break
            except Exception as e:
                self.logger.error(f"Error in monitoring loop: {e}")
                await asyncio.sleep(30)  # Wait 30s on error
    
    async def _collect_all_signals(self):
        """Collect signals from all sources"""
        tasks = [
            asyncio.create_task(self.twitter_monitor.search_tweets("crypto meme")),
            asyncio.create_task(self.gmgn_monitor.get_trending_tokens()),
            asyncio.create_task(self.dextools_monitor.get_hot_pairs())
        ]
        
        results = await asyncio.gather(*tasks, return_exceptions=True)
        
        # Aggregate signals
        all_signals = []
        for result in results:
            if isinstance(result, list):
                all_signals.extend(result)
            elif isinstance(result, Exception):
                self.logger.error(f"Error collecting signals: {result}")
        
        # Cache signals by token symbol
        self.signal_cache.clear()
        for signal in all_signals:
            symbol = signal.token_symbol
            if symbol not in self.signal_cache:
                self.signal_cache[symbol] = []
            self.signal_cache[symbol].append(signal)
        
        self.logger.info(f"Collected {len(all_signals)} signals for {len(self.signal_cache)} tokens")
    
    def get_token_signals(self, symbol: str) -> List[SocialSignal]:
        """Get all signals for a specific token"""
        return self.signal_cache.get(symbol, [])
    
    def get_aggregated_sentiment(self, symbol: str) -> Tuple[float, float]:
        """Get aggregated sentiment and confidence for a token"""
        signals = self.get_token_signals(symbol)
        
        if not signals:
            return 0.0, 0.0
        
        # Weight signals by engagement and recency
        total_weighted_sentiment = 0.0
        total_weight = 0.0
        
        now = datetime.now()
        
        for signal in signals:
            # Time decay factor (newer signals have more weight)
            age_hours = (now - signal.timestamp).total_seconds() / 3600
            time_weight = max(0.1, 1.0 - (age_hours / 24))  # Decay over 24 hours
            
            # Engagement weight
            engagement_weight = signal.engagement_score
            
            # Source weight (some sources may be more reliable)
            source_weights = {"twitter": 1.0, "gmgn": 1.2, "dextools": 1.1}
            source_weight = source_weights.get(signal.source, 1.0)
            
            # Combined weight
            weight = time_weight * engagement_weight * source_weight
            
            total_weighted_sentiment += signal.sentiment_score * weight
            total_weight += weight
        
        if total_weight == 0:
            return 0.0, 0.0
        
        avg_sentiment = total_weighted_sentiment / total_weight
        confidence = min(1.0, total_weight / len(signals))  # Confidence based on signal quality
        
        return avg_sentiment, confidence
    
    def get_trending_tokens(self, min_signals: int = 2, min_confidence: float = 0.3) -> List[Tuple[str, float, float]]:
        """Get trending tokens with their sentiment and confidence scores"""
        trending = []
        
        for symbol, signals in self.signal_cache.items():
            if len(signals) >= min_signals:
                sentiment, confidence = self.get_aggregated_sentiment(symbol)
                if confidence >= min_confidence:
                    trending.append((symbol, sentiment, confidence))
        
        # Sort by confidence * abs(sentiment) to get tokens with strong signals
        trending.sort(key=lambda x: x[2] * abs(x[1]), reverse=True)
        
        return trending
    
    def get_market_summary(self) -> Dict[str, Any]:
        """Get overall market sentiment summary"""
        all_signals = []
        for signals in self.signal_cache.values():
            all_signals.extend(signals)
        
        if not all_signals:
            return {
                "total_signals": 0,
                "unique_tokens": 0,
                "overall_sentiment": 0.0,
                "trending_tokens": []
            }
        
        # Calculate overall sentiment
        total_sentiment = sum(s.sentiment_score for s in all_signals)
        avg_sentiment = total_sentiment / len(all_signals)
        
        trending = self.get_trending_tokens()
        
        return {
            "total_signals": len(all_signals),
            "unique_tokens": len(self.signal_cache),
            "overall_sentiment": avg_sentiment,
            "trending_tokens": trending[:10],  # Top 10
            "last_update": datetime.now()
        }


if __name__ == "__main__":
    # Test the social monitor
    async def test_monitor():
        monitor = SocialMonitorAgent()
        await monitor.start_monitoring()
        
        # Let it run for a bit
        await asyncio.sleep(10)
        
        # Check results
        summary = monitor.get_market_summary()
        print(f"Market Summary: {summary}")
        
        await monitor.stop_monitoring()
    
    asyncio.run(test_monitor())