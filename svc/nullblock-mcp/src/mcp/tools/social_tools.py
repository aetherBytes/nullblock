"""
MCP tools for social media data collection and monitoring
"""

import logging
import asyncio
import aiohttp
import json
from typing import Dict, List, Optional, Any, Union
from datetime import datetime, timedelta
from pydantic import BaseModel, Field

logger = logging.getLogger(__name__)


class SocialMediaConfig(BaseModel):
    """Configuration for social media monitoring"""
    twitter_bearer_token: Optional[str] = None
    dextools_api_key: Optional[str] = None
    update_interval: int = 60
    max_signals_per_token: int = 100
    sentiment_threshold: float = 0.1
    confidence_threshold: float = 0.3


class SocialSignalResult(BaseModel):
    """Result format for social media signals"""
    token_symbol: str
    token_address: Optional[str] = None
    sentiment_score: float = Field(..., ge=-1.0, le=1.0, description="Sentiment from -1 (bearish) to 1 (bullish)")
    confidence_score: float = Field(..., ge=0.0, le=1.0, description="Confidence in the signal")
    signal_count: int = Field(..., description="Number of signals aggregated")
    sources: List[str] = Field(..., description="Sources of the signals")
    last_updated: datetime = Field(default_factory=datetime.now)
    trending_score: float = Field(default=0.0, description="Trending momentum score")


class TokenTrendResult(BaseModel):
    """Result format for trending token analysis"""
    symbol: str
    address: Optional[str] = None
    price_change_24h: Optional[float] = None
    volume_24h: Optional[float] = None
    market_cap: Optional[float] = None
    social_score: float = Field(..., ge=0.0, le=1.0)
    sentiment: float = Field(..., ge=-1.0, le=1.0)
    trend_score: float = Field(..., ge=0.0, le=1.0)
    risk_level: str = Field(..., description="LOW, MEDIUM, HIGH")


class SocialMediaTools:
    """MCP tools for social media monitoring and sentiment analysis"""
    
    def __init__(self, config: SocialMediaConfig):
        self.config = config
        self.logger = logging.getLogger(__name__)
        self.session: Optional[aiohttp.ClientSession] = None
        
        # Cache for signals
        self.signal_cache: Dict[str, List[Dict]] = {}
        self.last_update: Optional[datetime] = None
    
    async def _ensure_session(self):
        """Ensure HTTP session is active"""
        if not self.session:
            self.session = aiohttp.ClientSession()
    
    async def get_twitter_sentiment(
        self, 
        query: str, 
        limit: int = 50,
        hours_back: int = 24
    ) -> Dict[str, Any]:
        """
        Get sentiment analysis from Twitter/X for a given query
        
        Args:
            query: Search query (e.g., "$BONK", "solana meme")
            limit: Maximum number of tweets to analyze
            hours_back: How many hours back to search
            
        Returns:
            Dict with sentiment analysis results
        """
        try:
            await self._ensure_session()
            
            # Mock implementation for MVP - replace with real Twitter API
            signals = []
            
            # Simulate Twitter API response
            mock_tweets = [
                {
                    "text": f"Just bought some {query}! This is going to moon! 🚀",
                    "author": "crypto_bull",
                    "created_at": datetime.now() - timedelta(minutes=10),
                    "metrics": {"likes": 25, "retweets": 8, "replies": 3},
                    "sentiment": 0.8
                },
                {
                    "text": f"{query} chart looking bearish. Time to sell.",
                    "author": "chart_master", 
                    "created_at": datetime.now() - timedelta(minutes=30),
                    "metrics": {"likes": 12, "retweets": 3, "replies": 8},
                    "sentiment": -0.6
                },
                {
                    "text": f"Neutral on {query} right now. Waiting for breakout.",
                    "author": "patient_trader",
                    "created_at": datetime.now() - timedelta(hours=2),
                    "metrics": {"likes": 5, "retweets": 1, "replies": 2},
                    "sentiment": 0.1
                }
            ]
            
            total_sentiment = 0.0
            total_weight = 0.0
            
            for tweet in mock_tweets:
                # Weight by engagement
                engagement = tweet["metrics"]["likes"] + tweet["metrics"]["retweets"]
                weight = max(1.0, engagement / 10.0)
                
                total_sentiment += tweet["sentiment"] * weight
                total_weight += weight
                
                signals.append({
                    "content": tweet["text"],
                    "author": tweet["author"],
                    "sentiment": tweet["sentiment"],
                    "engagement": engagement,
                    "timestamp": tweet["created_at"].isoformat()
                })
            
            avg_sentiment = total_sentiment / total_weight if total_weight > 0 else 0.0
            confidence = min(1.0, len(signals) / 10.0)  # More signals = higher confidence
            
            return {
                "query": query,
                "sentiment_score": round(avg_sentiment, 3),
                "confidence": round(confidence, 3),
                "signal_count": len(signals),
                "signals": signals[:10],  # Return top 10
                "source": "twitter"
            }
            
        except Exception as e:
            self.logger.error(f"Failed to get Twitter sentiment for {query}: {e}")
            return {
                "query": query,
                "sentiment_score": 0.0,
                "confidence": 0.0,
                "signal_count": 0,
                "signals": [],
                "error": str(e)
            }
    
    async def get_gmgn_trends(self, limit: int = 20) -> List[TokenTrendResult]:
        """
        Get trending tokens from GMGN.ai
        
        Args:
            limit: Maximum number of trending tokens to return
            
        Returns:
            List of trending token data
        """
        try:
            await self._ensure_session()
            
            # Mock GMGN API response
            mock_tokens = [
                {
                    "symbol": "BONK",
                    "address": "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",
                    "price_change_24h": 15.2,
                    "volume_24h": 2500000,
                    "market_cap": 180000000,
                    "social_mentions": 450,
                    "sentiment_score": 0.75
                },
                {
                    "symbol": "WIF",
                    "address": "EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm", 
                    "price_change_24h": -8.5,
                    "volume_24h": 1800000,
                    "market_cap": 650000000,
                    "social_mentions": 320,
                    "sentiment_score": -0.35
                },
                {
                    "symbol": "POPCAT",
                    "address": "7GCihgDB8fe6KNjn2MYtkzZcRjQy3t9GHdC8uHYmW2hr",
                    "price_change_24h": 25.8,
                    "volume_24h": 950000, 
                    "market_cap": 45000000,
                    "social_mentions": 680,
                    "sentiment_score": 0.85
                }
            ]
            
            results = []
            for token in mock_tokens:
                # Calculate risk level
                risk_level = "LOW"
                if token["market_cap"] < 10000000:  # Under $10M mcap
                    risk_level = "HIGH"
                elif token["market_cap"] < 100000000:  # Under $100M mcap
                    risk_level = "MEDIUM"
                
                # Calculate trend score based on price change and social activity
                price_factor = min(1.0, abs(token["price_change_24h"]) / 50.0)
                social_factor = min(1.0, token["social_mentions"] / 1000.0)
                trend_score = (price_factor + social_factor) / 2
                
                result = TokenTrendResult(
                    symbol=token["symbol"],
                    address=token["address"],
                    price_change_24h=token["price_change_24h"],
                    volume_24h=token["volume_24h"],
                    market_cap=token["market_cap"],
                    social_score=social_factor,
                    sentiment=token["sentiment_score"],
                    trend_score=trend_score,
                    risk_level=risk_level
                )
                
                results.append(result)
            
            # Sort by trend score
            results.sort(key=lambda x: x.trend_score, reverse=True)
            
            return results[:limit]
            
        except Exception as e:
            self.logger.error(f"Failed to get GMGN trends: {e}")
            return []
    
    async def get_dextools_social_score(
        self, 
        token_address: str, 
        chain: str = "solana"
    ) -> Dict[str, Any]:
        """
        Get social score and metrics from DEXTools
        
        Args:
            token_address: Token contract address
            chain: Blockchain (default: solana)
            
        Returns:
            Dict with social metrics
        """
        try:
            await self._ensure_session()
            
            # Mock DEXTools API response
            mock_data = {
                "address": token_address,
                "chain": chain,
                "social_score": 78,
                "telegram_members": 12500,
                "twitter_followers": 8900,
                "holders": 15420,
                "price_change_1h": 2.1,
                "price_change_24h": 15.2,
                "price_change_7d": -5.8,
                "volume_24h": 2100000,
                "liquidity": 850000,
                "market_cap": 45000000
            }
            
            # Calculate overall sentiment from metrics
            sentiment = 0.0
            if mock_data["price_change_24h"] > 10:
                sentiment += 0.4
            elif mock_data["price_change_24h"] > 0:
                sentiment += 0.2
            else:
                sentiment -= 0.3
            
            if mock_data["social_score"] > 70:
                sentiment += 0.3
            elif mock_data["social_score"] > 50:
                sentiment += 0.1
            
            sentiment = max(-1.0, min(1.0, sentiment))
            
            return {
                "token_address": token_address,
                "chain": chain,
                "social_score": mock_data["social_score"],
                "sentiment_score": round(sentiment, 3),
                "confidence": 0.8,
                "metrics": mock_data,
                "source": "dextools"
            }
            
        except Exception as e:
            self.logger.error(f"Failed to get DEXTools data for {token_address}: {e}")
            return {
                "token_address": token_address,
                "sentiment_score": 0.0,
                "confidence": 0.0,
                "error": str(e)
            }
    
    async def analyze_token_sentiment(
        self, 
        symbol: str, 
        address: Optional[str] = None
    ) -> SocialSignalResult:
        """
        Comprehensive sentiment analysis for a token across all sources
        
        Args:
            symbol: Token symbol (e.g., "BONK")
            address: Optional token contract address
            
        Returns:
            Aggregated sentiment analysis result
        """
        try:
            # Collect signals from all sources
            tasks = [
                self.get_twitter_sentiment(f"${symbol}"),
                self.get_gmgn_trends(limit=50),
            ]
            
            if address:
                tasks.append(self.get_dextools_social_score(address))
            
            results = await asyncio.gather(*tasks, return_exceptions=True)
            
            # Aggregate results
            sentiments = []
            confidences = []
            sources = []
            signal_count = 0
            
            # Process Twitter results
            if isinstance(results[0], dict) and "sentiment_score" in results[0]:
                twitter_data = results[0]
                sentiments.append(twitter_data["sentiment_score"])
                confidences.append(twitter_data["confidence"])
                sources.append("twitter")
                signal_count += twitter_data["signal_count"]
            
            # Process GMGN results
            if isinstance(results[1], list):
                gmgn_tokens = results[1]
                for token in gmgn_tokens:
                    if token.symbol == symbol:
                        sentiments.append(token.sentiment)
                        confidences.append(token.social_score)
                        sources.append("gmgn")
                        signal_count += 1
                        break
            
            # Process DEXTools results
            if len(results) > 2 and isinstance(results[2], dict):
                dextools_data = results[2]
                if "sentiment_score" in dextools_data:
                    sentiments.append(dextools_data["sentiment_score"])
                    confidences.append(dextools_data["confidence"])
                    sources.append("dextools")
                    signal_count += 1
            
            # Calculate weighted averages
            if sentiments:
                weighted_sentiment = sum(s * c for s, c in zip(sentiments, confidences))
                total_confidence = sum(confidences)
                
                avg_sentiment = weighted_sentiment / total_confidence if total_confidence > 0 else 0.0
                avg_confidence = total_confidence / len(confidences) if confidences else 0.0
                
                # Calculate trending score
                trending_score = min(1.0, (abs(avg_sentiment) * avg_confidence + len(sources) * 0.2))
            else:
                avg_sentiment = 0.0
                avg_confidence = 0.0
                trending_score = 0.0
            
            return SocialSignalResult(
                token_symbol=symbol,
                token_address=address,
                sentiment_score=round(avg_sentiment, 3),
                confidence_score=round(avg_confidence, 3),
                signal_count=signal_count,
                sources=sources,
                trending_score=round(trending_score, 3)
            )
            
        except Exception as e:
            self.logger.error(f"Failed to analyze sentiment for {symbol}: {e}")
            return SocialSignalResult(
                token_symbol=symbol,
                token_address=address,
                sentiment_score=0.0,
                confidence_score=0.0,
                signal_count=0,
                sources=[]
            )
    
    async def get_trending_tokens(
        self, 
        limit: int = 10,
        min_confidence: float = 0.3
    ) -> List[SocialSignalResult]:
        """
        Get currently trending tokens with sentiment analysis
        
        Args:
            limit: Maximum number of tokens to return
            min_confidence: Minimum confidence threshold
            
        Returns:
            List of trending tokens with sentiment data
        """
        try:
            # Get trending tokens from GMGN
            gmgn_tokens = await self.get_gmgn_trends(limit=50)
            
            # Analyze sentiment for each
            tasks = []
            for token in gmgn_tokens:
                task = self.analyze_token_sentiment(token.symbol, token.address)
                tasks.append(task)
            
            results = await asyncio.gather(*tasks, return_exceptions=True)
            
            # Filter and sort results
            trending = []
            for result in results:
                if (isinstance(result, SocialSignalResult) and 
                    result.confidence_score >= min_confidence):
                    trending.append(result)
            
            # Sort by trending score
            trending.sort(key=lambda x: x.trending_score, reverse=True)
            
            return trending[:limit]
            
        except Exception as e:
            self.logger.error(f"Failed to get trending tokens: {e}")
            return []
    
    async def monitor_token_mentions(
        self, 
        symbol: str, 
        duration_minutes: int = 60
    ) -> Dict[str, Any]:
        """
        Monitor mentions of a specific token over time
        
        Args:
            symbol: Token symbol to monitor
            duration_minutes: How long to monitor (in minutes)
            
        Returns:
            Dict with monitoring results
        """
        try:
            start_time = datetime.now()
            end_time = start_time + timedelta(minutes=duration_minutes)
            
            mentions = []
            sentiment_history = []
            
            # Mock monitoring data
            while datetime.now() < end_time:
                # Simulate getting new mentions
                current_sentiment = await self.analyze_token_sentiment(symbol)
                
                mentions.append({
                    "timestamp": datetime.now().isoformat(),
                    "sentiment": current_sentiment.sentiment_score,
                    "confidence": current_sentiment.confidence_score,
                    "signal_count": current_sentiment.signal_count
                })
                
                sentiment_history.append(current_sentiment.sentiment_score)
                
                # Wait 5 minutes between checks
                await asyncio.sleep(300)
            
            # Calculate summary statistics
            avg_sentiment = sum(sentiment_history) / len(sentiment_history) if sentiment_history else 0.0
            sentiment_volatility = max(sentiment_history) - min(sentiment_history) if sentiment_history else 0.0
            
            return {
                "symbol": symbol,
                "monitoring_duration_minutes": duration_minutes,
                "total_mentions": len(mentions),
                "average_sentiment": round(avg_sentiment, 3),
                "sentiment_volatility": round(sentiment_volatility, 3),
                "mentions": mentions,
                "summary": {
                    "bullish_periods": len([s for s in sentiment_history if s > 0.2]),
                    "bearish_periods": len([s for s in sentiment_history if s < -0.2]),
                    "neutral_periods": len([s for s in sentiment_history if -0.2 <= s <= 0.2])
                }
            }
            
        except Exception as e:
            self.logger.error(f"Failed to monitor {symbol}: {e}")
            return {
                "symbol": symbol,
                "error": str(e)
            }
    
    async def cleanup(self):
        """Clean up resources"""
        if self.session:
            await self.session.close()
            self.session = None