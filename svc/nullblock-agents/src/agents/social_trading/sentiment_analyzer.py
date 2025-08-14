"""
Advanced sentiment analyzer for social trading decisions
"""

import logging
import asyncio
import numpy as np
from typing import Dict, List, Optional, Any, Tuple
from datetime import datetime, timedelta
from dataclasses import dataclass
from pydantic import BaseModel, Field
import math

logger = logging.getLogger(__name__)


@dataclass
class TradingSignal:
    """Trading signal based on sentiment analysis"""
    token_symbol: str
    signal_type: str  # 'BUY', 'SELL', 'HOLD'
    strength: float  # 0.0 to 1.0
    confidence: float  # 0.0 to 1.0
    sentiment_score: float  # -1.0 to 1.0
    price_target: Optional[float] = None
    stop_loss: Optional[float] = None
    time_horizon: Optional[str] = None  # 'SHORT', 'MEDIUM', 'LONG'
    reasoning: List[str] = None


class SentimentMetrics(BaseModel):
    """Comprehensive sentiment metrics for a token"""
    token_symbol: str
    overall_sentiment: float = Field(..., ge=-1.0, le=1.0)
    sentiment_momentum: float = Field(..., ge=-1.0, le=1.0)
    social_volume: int = Field(..., ge=0)
    engagement_quality: float = Field(..., ge=0.0, le=1.0)
    influencer_sentiment: float = Field(..., ge=-1.0, le=1.0)
    retail_sentiment: float = Field(..., ge=-1.0, le=1.0)
    fear_greed_index: int = Field(..., ge=0, le=100)
    volatility_score: float = Field(..., ge=0.0, le=1.0)
    trend_strength: float = Field(..., ge=0.0, le=1.0)
    risk_score: float = Field(..., ge=0.0, le=1.0)


class MarketRegime(BaseModel):
    """Current market regime analysis"""
    regime_type: str  # 'BULL', 'BEAR', 'SIDEWAYS', 'VOLATILE'
    confidence: float = Field(..., ge=0.0, le=1.0)
    duration_days: int
    key_indicators: List[str]
    trading_strategy: str


class SentimentAnalyzer:
    """Advanced sentiment analyzer for trading decisions"""
    
    def __init__(self, config: Dict[str, Any] = None):
        self.config = config or {}
        self.logger = logging.getLogger(__name__)
        
        # Sentiment weights for different sources
        self.source_weights = {
            "twitter": 1.0,
            "gmgn": 1.2,
            "dextools": 1.1,
            "telegram": 0.9,
            "discord": 0.8,
            "reddit": 1.0
        }
        
        # Influencer accounts (higher weight)
        self.influencer_accounts = {
            "elonmusk": 2.0,
            "VitalikButerin": 1.8,
            "cz_binance": 1.5,
            "justinsuntron": 1.3,
            "stablekwon": 1.2
        }
        
        # Market regime indicators
        self.market_indicators = {
            "fear_greed_threshold": 25,  # Below = fear market
            "greed_threshold": 75,       # Above = greed market
            "volatility_threshold": 0.5,  # Above = volatile market
            "trend_threshold": 0.3       # Above = trending market
        }
    
    def analyze_sentiment_metrics(
        self, 
        social_signals: List[Dict[str, Any]]
    ) -> SentimentMetrics:
        """Analyze comprehensive sentiment metrics for a token"""
        try:
            if not social_signals:
                return self._create_neutral_metrics("UNKNOWN")
            
            # Extract token symbol (assuming all signals are for same token)
            token_symbol = social_signals[0].get("token_symbol", "UNKNOWN")
            
            # Calculate overall sentiment with weights
            weighted_sentiments = []
            total_weight = 0.0
            
            influencer_sentiments = []
            retail_sentiments = []
            
            for signal in social_signals:
                sentiment = signal.get("sentiment_score", 0.0)
                source = signal.get("source", "unknown")
                author = signal.get("author", "")
                engagement = signal.get("engagement_score", 0.5)
                
                # Base weight from source
                weight = self.source_weights.get(source, 1.0)
                
                # Influencer weight multiplier
                if author in self.influencer_accounts:
                    weight *= self.influencer_accounts[author]
                    influencer_sentiments.append(sentiment)
                else:
                    retail_sentiments.append(sentiment)
                
                # Engagement weight multiplier
                weight *= (0.5 + engagement * 0.5)  # 0.5x to 1.0x based on engagement
                
                weighted_sentiments.append(sentiment * weight)
                total_weight += weight
            
            # Calculate metrics
            overall_sentiment = sum(weighted_sentiments) / total_weight if total_weight > 0 else 0.0
            
            # Sentiment momentum (recent vs older signals)
            sentiment_momentum = self._calculate_sentiment_momentum(social_signals)
            
            # Social volume
            social_volume = len(social_signals)
            
            # Engagement quality
            engagement_scores = [s.get("engagement_score", 0.5) for s in social_signals]
            engagement_quality = sum(engagement_scores) / len(engagement_scores) if engagement_scores else 0.5
            
            # Influencer vs retail sentiment
            influencer_sentiment = sum(influencer_sentiments) / len(influencer_sentiments) if influencer_sentiments else 0.0
            retail_sentiment = sum(retail_sentiments) / len(retail_sentiments) if retail_sentiments else 0.0
            
            # Fear & Greed Index
            fear_greed_index = self._calculate_fear_greed_index(social_signals)
            
            # Volatility score
            sentiments = [s.get("sentiment_score", 0.0) for s in social_signals]
            volatility_score = self._calculate_volatility_score(sentiments)
            
            # Trend strength
            trend_strength = self._calculate_trend_strength(social_signals)
            
            # Risk score
            risk_score = self._calculate_risk_score(social_signals, overall_sentiment, volatility_score)
            
            return SentimentMetrics(
                token_symbol=token_symbol,
                overall_sentiment=round(overall_sentiment, 3),
                sentiment_momentum=round(sentiment_momentum, 3),
                social_volume=social_volume,
                engagement_quality=round(engagement_quality, 3),
                influencer_sentiment=round(influencer_sentiment, 3),
                retail_sentiment=round(retail_sentiment, 3),
                fear_greed_index=fear_greed_index,
                volatility_score=round(volatility_score, 3),
                trend_strength=round(trend_strength, 3),
                risk_score=round(risk_score, 3)
            )
            
        except Exception as e:
            self.logger.error(f"Failed to analyze sentiment metrics: {e}")
            return self._create_neutral_metrics("ERROR")
    
    def _calculate_sentiment_momentum(self, signals: List[Dict[str, Any]]) -> float:
        """Calculate sentiment momentum (trend direction)"""
        try:
            if len(signals) < 2:
                return 0.0
            
            # Sort signals by timestamp
            sorted_signals = sorted(
                signals, 
                key=lambda x: x.get("timestamp", datetime.now())
            )
            
            # Split into recent and older halves
            mid_point = len(sorted_signals) // 2
            older_signals = sorted_signals[:mid_point]
            recent_signals = sorted_signals[mid_point:]
            
            # Calculate average sentiment for each half
            older_avg = sum(s.get("sentiment_score", 0.0) for s in older_signals) / len(older_signals)
            recent_avg = sum(s.get("sentiment_score", 0.0) for s in recent_signals) / len(recent_signals)
            
            # Momentum is the difference
            momentum = recent_avg - older_avg
            
            return max(-1.0, min(1.0, momentum))
            
        except Exception:
            return 0.0
    
    def _calculate_fear_greed_index(self, signals: List[Dict[str, Any]]) -> int:
        """Calculate Fear & Greed Index (0-100)"""
        try:
            if not signals:
                return 50
            
            # Sentiment component (40% weight)
            sentiments = [s.get("sentiment_score", 0.0) for s in signals]
            avg_sentiment = sum(sentiments) / len(sentiments)
            sentiment_score = (avg_sentiment + 1) * 50  # Convert -1,1 to 0,100
            
            # Volume component (30% weight)
            volume_score = min(100, len(signals) * 2)  # More signals = more greed
            
            # Volatility component (30% weight) - inverted
            volatility = self._calculate_volatility_score(sentiments)
            volatility_score = (1 - volatility) * 100
            
            # Weighted average
            fear_greed = int(
                sentiment_score * 0.4 + 
                volume_score * 0.3 + 
                volatility_score * 0.3
            )
            
            return max(0, min(100, fear_greed))
            
        except Exception:
            return 50
    
    def _calculate_volatility_score(self, sentiments: List[float]) -> float:
        """Calculate sentiment volatility score"""
        try:
            if len(sentiments) < 2:
                return 0.0
            
            mean_sentiment = sum(sentiments) / len(sentiments)
            variance = sum((s - mean_sentiment) ** 2 for s in sentiments) / len(sentiments)
            std_dev = math.sqrt(variance)
            
            # Normalize to 0-1 scale (assuming max std dev of 1.0)
            return min(1.0, std_dev)
            
        except Exception:
            return 0.0
    
    def _calculate_trend_strength(self, signals: List[Dict[str, Any]]) -> float:
        """Calculate trend strength based on consistency and momentum"""
        try:
            if len(signals) < 3:
                return 0.0
            
            sentiments = [s.get("sentiment_score", 0.0) for s in signals]
            
            # Calculate trend using linear regression approximation
            n = len(sentiments)
            x_vals = list(range(n))
            
            sum_x = sum(x_vals)
            sum_y = sum(sentiments)
            sum_xy = sum(x * y for x, y in zip(x_vals, sentiments))
            sum_x2 = sum(x * x for x in x_vals)
            
            # Slope of trend line
            slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x) if (n * sum_x2 - sum_x * sum_x) != 0 else 0.0
            
            # R-squared for trend strength
            y_mean = sum_y / n
            ss_tot = sum((y - y_mean) ** 2 for y in sentiments)
            
            if ss_tot == 0:
                return 0.0
            
            # Predicted values
            y_pred = [(slope * x + (sum_y - slope * sum_x) / n) for x in x_vals]
            ss_res = sum((y - y_p) ** 2 for y, y_p in zip(sentiments, y_pred))
            
            r_squared = 1 - (ss_res / ss_tot)
            
            # Trend strength combines slope magnitude and fit quality
            trend_strength = abs(slope) * r_squared
            
            return min(1.0, trend_strength)
            
        except Exception:
            return 0.0
    
    def _calculate_risk_score(
        self, 
        signals: List[Dict[str, Any]], 
        sentiment: float, 
        volatility: float
    ) -> float:
        """Calculate overall risk score"""
        try:
            risk_factors = []
            
            # Sentiment extremes add risk
            if abs(sentiment) > 0.8:
                risk_factors.append(0.3)
            
            # High volatility adds risk
            if volatility > 0.7:
                risk_factors.append(0.4)
            
            # Check for scam/rug mentions
            scam_mentions = sum(
                1 for s in signals 
                if any(word in s.get("content", "").lower() 
                      for word in ["scam", "rug", "fraud", "ponzi"])
            )
            
            if scam_mentions > len(signals) * 0.1:  # >10% scam mentions
                risk_factors.append(0.5)
            
            # Low engagement quality adds risk
            engagement_scores = [s.get("engagement_score", 0.5) for s in signals]
            avg_engagement = sum(engagement_scores) / len(engagement_scores) if engagement_scores else 0.5
            
            if avg_engagement < 0.3:
                risk_factors.append(0.2)
            
            # Low signal count adds risk
            if len(signals) < 5:
                risk_factors.append(0.2)
            
            # Combine risk factors
            total_risk = sum(risk_factors)
            
            return min(1.0, total_risk)
            
        except Exception:
            return 0.5  # Default medium risk
    
    def generate_trading_signal(
        self, 
        sentiment_metrics: SentimentMetrics,
        current_price: Optional[float] = None,
        market_regime: Optional[MarketRegime] = None
    ) -> TradingSignal:
        """Generate trading signal based on sentiment analysis"""
        try:
            reasoning = []
            
            # Base signal from sentiment
            signal_type = "HOLD"
            strength = 0.0
            confidence = sentiment_metrics.engagement_quality
            
            # Strong bullish sentiment
            if (sentiment_metrics.overall_sentiment > 0.3 and 
                sentiment_metrics.sentiment_momentum > 0.2 and
                sentiment_metrics.trend_strength > 0.4):
                
                signal_type = "BUY"
                strength = min(1.0, sentiment_metrics.overall_sentiment + sentiment_metrics.trend_strength)
                reasoning.append("Strong bullish sentiment with positive momentum")
            
            # Strong bearish sentiment
            elif (sentiment_metrics.overall_sentiment < -0.3 and 
                  sentiment_metrics.sentiment_momentum < -0.2 and
                  sentiment_metrics.trend_strength > 0.4):
                
                signal_type = "SELL"
                strength = min(1.0, abs(sentiment_metrics.overall_sentiment) + sentiment_metrics.trend_strength)
                reasoning.append("Strong bearish sentiment with negative momentum")
            
            # Moderate bullish with good conditions
            elif (sentiment_metrics.overall_sentiment > 0.1 and
                  sentiment_metrics.fear_greed_index < 75 and  # Not too greedy
                  sentiment_metrics.risk_score < 0.6):
                
                signal_type = "BUY"
                strength = 0.5 + sentiment_metrics.overall_sentiment * 0.5
                reasoning.append("Moderate bullish sentiment with acceptable risk")
            
            # Apply risk adjustments
            if sentiment_metrics.risk_score > 0.7:
                if signal_type == "BUY":
                    signal_type = "HOLD"
                strength *= 0.5
                confidence *= 0.7
                reasoning.append("High risk detected - reducing signal strength")
            
            # Volatility adjustments
            if sentiment_metrics.volatility_score > 0.8:
                strength *= 0.8
                confidence *= 0.8
                reasoning.append("High volatility - reducing confidence")
            
            # Market regime adjustments
            if market_regime:
                if market_regime.regime_type == "BEAR" and signal_type == "BUY":
                    strength *= 0.7
                    reasoning.append("Bear market - reducing buy signal strength")
                elif market_regime.regime_type == "BULL" and signal_type == "SELL":
                    strength *= 0.7
                    reasoning.append("Bull market - reducing sell signal strength")
            
            # Time horizon based on trend strength
            if sentiment_metrics.trend_strength > 0.7:
                time_horizon = "LONG"
            elif sentiment_metrics.trend_strength > 0.4:
                time_horizon = "MEDIUM"
            else:
                time_horizon = "SHORT"
            
            # Price targets (if current price provided)
            price_target = None
            stop_loss = None
            
            if current_price and signal_type == "BUY":
                # Target based on sentiment strength
                target_return = 0.1 + (strength * 0.4)  # 10% to 50% target
                price_target = current_price * (1 + target_return)
                
                # Stop loss at 10-20% below entry
                stop_loss_pct = 0.1 + (sentiment_metrics.risk_score * 0.1)
                stop_loss = current_price * (1 - stop_loss_pct)
            
            elif current_price and signal_type == "SELL":
                # Target based on sentiment strength
                target_return = 0.1 + (strength * 0.3)  # 10% to 40% drop target
                price_target = current_price * (1 - target_return)
                
                # Stop loss at 5-15% above entry
                stop_loss_pct = 0.05 + (sentiment_metrics.risk_score * 0.1)
                stop_loss = current_price * (1 + stop_loss_pct)
            
            return TradingSignal(
                token_symbol=sentiment_metrics.token_symbol,
                signal_type=signal_type,
                strength=round(strength, 3),
                confidence=round(confidence, 3),
                sentiment_score=sentiment_metrics.overall_sentiment,
                price_target=round(price_target, 6) if price_target else None,
                stop_loss=round(stop_loss, 6) if stop_loss else None,
                time_horizon=time_horizon,
                reasoning=reasoning
            )
            
        except Exception as e:
            self.logger.error(f"Failed to generate trading signal: {e}")
            return TradingSignal(
                token_symbol=sentiment_metrics.token_symbol,
                signal_type="HOLD",
                strength=0.0,
                confidence=0.0,
                sentiment_score=0.0,
                reasoning=[f"Error generating signal: {str(e)}"]
            )
    
    def analyze_market_regime(
        self, 
        market_signals: List[Dict[str, Any]]
    ) -> MarketRegime:
        """Analyze current market regime"""
        try:
            if not market_signals:
                return MarketRegime(
                    regime_type="SIDEWAYS",
                    confidence=0.5,
                    duration_days=0,
                    key_indicators=[],
                    trading_strategy="Wait for clearer signals"
                )
            
            # Calculate overall market sentiment
            all_sentiments = []
            for signal_group in market_signals:
                if isinstance(signal_group, list):
                    all_sentiments.extend([s.get("sentiment_score", 0.0) for s in signal_group])
                else:
                    all_sentiments.append(signal_group.get("sentiment_score", 0.0))
            
            if not all_sentiments:
                return self._create_neutral_regime()
            
            avg_sentiment = sum(all_sentiments) / len(all_sentiments)
            volatility = self._calculate_volatility_score(all_sentiments)
            
            # Calculate Fear & Greed Index for market
            fear_greed = self._calculate_fear_greed_index(
                [{"sentiment_score": s} for s in all_sentiments]
            )
            
            # Determine regime
            key_indicators = []
            
            if fear_greed <= self.market_indicators["fear_greed_threshold"]:
                regime_type = "BEAR"
                key_indicators.append(f"Fear & Greed Index: {fear_greed} (Fear)")
                trading_strategy = "Defensive positioning, wait for capitulation"
                
            elif fear_greed >= self.market_indicators["greed_threshold"]:
                regime_type = "BULL"
                key_indicators.append(f"Fear & Greed Index: {fear_greed} (Greed)")
                trading_strategy = "Risk management, consider profit taking"
                
            elif volatility > self.market_indicators["volatility_threshold"]:
                regime_type = "VOLATILE"
                key_indicators.append(f"High volatility: {volatility:.2f}")
                trading_strategy = "Reduce position sizes, use tight stops"
                
            else:
                regime_type = "SIDEWAYS"
                key_indicators.append("Neutral sentiment and low volatility")
                trading_strategy = "Range trading, wait for breakout"
            
            # Add sentiment indicator
            if abs(avg_sentiment) > self.market_indicators["trend_threshold"]:
                direction = "Bullish" if avg_sentiment > 0 else "Bearish"
                key_indicators.append(f"{direction} sentiment trend: {avg_sentiment:.2f}")
            
            # Calculate confidence based on signal consistency
            confidence = 1.0 - volatility  # Lower volatility = higher confidence
            confidence = max(0.3, min(1.0, confidence))
            
            # Mock duration (in real implementation, would track regime changes)
            duration_days = 7
            
            return MarketRegime(
                regime_type=regime_type,
                confidence=round(confidence, 3),
                duration_days=duration_days,
                key_indicators=key_indicators,
                trading_strategy=trading_strategy
            )
            
        except Exception as e:
            self.logger.error(f"Failed to analyze market regime: {e}")
            return self._create_neutral_regime()
    
    def _create_neutral_metrics(self, token_symbol: str) -> SentimentMetrics:
        """Create neutral sentiment metrics"""
        return SentimentMetrics(
            token_symbol=token_symbol,
            overall_sentiment=0.0,
            sentiment_momentum=0.0,
            social_volume=0,
            engagement_quality=0.5,
            influencer_sentiment=0.0,
            retail_sentiment=0.0,
            fear_greed_index=50,
            volatility_score=0.0,
            trend_strength=0.0,
            risk_score=0.5
        )
    
    def _create_neutral_regime(self) -> MarketRegime:
        """Create neutral market regime"""
        return MarketRegime(
            regime_type="SIDEWAYS",
            confidence=0.5,
            duration_days=0,
            key_indicators=["Insufficient data"],
            trading_strategy="Wait for more data"
        )
    
    def batch_analyze_tokens(
        self, 
        token_signals: Dict[str, List[Dict[str, Any]]]
    ) -> Dict[str, TradingSignal]:
        """Analyze multiple tokens and generate trading signals"""
        try:
            results = {}
            
            for token_symbol, signals in token_signals.items():
                # Analyze sentiment metrics
                metrics = self.analyze_sentiment_metrics(signals)
                
                # Generate trading signal
                trading_signal = self.generate_trading_signal(metrics)
                
                results[token_symbol] = trading_signal
            
            return results
            
        except Exception as e:
            self.logger.error(f"Failed to batch analyze tokens: {e}")
            return {}
    
    def get_top_opportunities(
        self, 
        trading_signals: Dict[str, TradingSignal],
        limit: int = 5
    ) -> List[TradingSignal]:
        """Get top trading opportunities sorted by strength and confidence"""
        try:
            # Filter for buy signals only
            buy_signals = [
                signal for signal in trading_signals.values() 
                if signal.signal_type == "BUY"
            ]
            
            # Sort by combined score (strength * confidence)
            buy_signals.sort(
                key=lambda s: s.strength * s.confidence, 
                reverse=True
            )
            
            return buy_signals[:limit]
            
        except Exception as e:
            self.logger.error(f"Failed to get top opportunities: {e}")
            return []