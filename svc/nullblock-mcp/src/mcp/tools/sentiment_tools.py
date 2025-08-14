"""
MCP tools for advanced sentiment analysis and signal processing
"""

import logging
import re
import asyncio
from typing import Dict, List, Optional, Any, Tuple
from datetime import datetime, timedelta
from pydantic import BaseModel, Field
import math

logger = logging.getLogger(__name__)


class SentimentSignal(BaseModel):
    """Individual sentiment signal"""
    text: str
    sentiment_score: float = Field(..., ge=-1.0, le=1.0)
    confidence: float = Field(..., ge=0.0, le=1.0)
    keywords: List[str] = Field(default_factory=list)
    source: str
    timestamp: datetime
    metadata: Dict[str, Any] = Field(default_factory=dict)


class SentimentAnalysis(BaseModel):
    """Comprehensive sentiment analysis result"""
    overall_sentiment: float = Field(..., ge=-1.0, le=1.0)
    confidence: float = Field(..., ge=0.0, le=1.0)
    signal_count: int
    bullish_signals: int
    bearish_signals: int
    neutral_signals: int
    keyword_analysis: Dict[str, float]
    emotion_analysis: Dict[str, float]
    trend_analysis: Dict[str, float]
    risk_indicators: List[str]


class MarketSentimentConfig(BaseModel):
    """Configuration for market sentiment analysis"""
    bullish_keywords: List[str] = Field(default=[
        "moon", "pump", "rocket", "bull", "buy", "hodl", "diamond hands",
        "breakout", "bullish", "rally", "surge", "explode", "gem", "lambo"
    ])
    bearish_keywords: List[str] = Field(default=[
        "dump", "crash", "bear", "sell", "short", "bearish", "drop",
        "decline", "collapse", "scam", "rug", "red", "blood", "rekt"
    ])
    risk_keywords: List[str] = Field(default=[
        "scam", "rug pull", "honeypot", "exit scam", "ponzi", "fraud",
        "manipulation", "whale dump", "insider trading", "fake"
    ])
    emotion_keywords: Dict[str, List[str]] = Field(default={
        "fear": ["scared", "worried", "panic", "afraid", "anxious", "nervous"],
        "greed": ["fomo", "ape in", "yolo", "all in", "moon mission", "lambo"],
        "euphoria": ["amazing", "incredible", "best", "perfect", "unbelievable"],
        "despair": ["hopeless", "done", "finished", "over", "dead", "rekt"]
    })


class SentimentAnalysisTools:
    """Advanced sentiment analysis tools for crypto markets"""
    
    def __init__(self, config: MarketSentimentConfig = None):
        self.config = config or MarketSentimentConfig()
        self.logger = logging.getLogger(__name__)
        
        # Compile regex patterns for efficiency
        self._compile_patterns()
    
    def _compile_patterns(self):
        """Compile regex patterns for keyword detection"""
        self.bullish_pattern = re.compile(
            r'\b(' + '|'.join(re.escape(word) for word in self.config.bullish_keywords) + r')\b',
            re.IGNORECASE
        )
        
        self.bearish_pattern = re.compile(
            r'\b(' + '|'.join(re.escape(word) for word in self.config.bearish_keywords) + r')\b',
            re.IGNORECASE
        )
        
        self.risk_pattern = re.compile(
            r'\b(' + '|'.join(re.escape(word) for word in self.config.risk_keywords) + r')\b',
            re.IGNORECASE
        )
        
        # Compile emotion patterns
        self.emotion_patterns = {}
        for emotion, words in self.config.emotion_keywords.items():
            self.emotion_patterns[emotion] = re.compile(
                r'\b(' + '|'.join(re.escape(word) for word in words) + r')\b',
                re.IGNORECASE
            )
    
    def analyze_text_sentiment(self, text: str) -> SentimentSignal:
        """
        Analyze sentiment of a single text
        
        Args:
            text: Text to analyze
            
        Returns:
            SentimentSignal with analysis results
        """
        try:
            # Clean text
            clean_text = self._clean_text(text)
            
            # Find keyword matches
            bullish_matches = self.bullish_pattern.findall(clean_text)
            bearish_matches = self.bearish_pattern.findall(clean_text)
            risk_matches = self.risk_pattern.findall(clean_text)
            
            # Calculate base sentiment
            bullish_score = len(bullish_matches) * 0.3
            bearish_score = len(bearish_matches) * 0.3
            risk_penalty = len(risk_matches) * 0.5
            
            base_sentiment = bullish_score - bearish_score - risk_penalty
            
            # Apply text analysis modifiers
            sentiment_modifiers = self._analyze_text_features(clean_text)
            final_sentiment = base_sentiment + sentiment_modifiers
            
            # Normalize to -1 to 1 range
            final_sentiment = max(-1.0, min(1.0, final_sentiment))
            
            # Calculate confidence based on signal strength
            signal_strength = abs(bullish_score) + abs(bearish_score) + abs(risk_penalty)
            confidence = min(1.0, signal_strength / 2.0)
            
            # Extract all matched keywords
            all_keywords = bullish_matches + bearish_matches + risk_matches
            
            return SentimentSignal(
                text=text,
                sentiment_score=round(final_sentiment, 3),
                confidence=round(confidence, 3),
                keywords=all_keywords,
                source="text_analysis",
                timestamp=datetime.now(),
                metadata={
                    "bullish_matches": len(bullish_matches),
                    "bearish_matches": len(bearish_matches),
                    "risk_matches": len(risk_matches),
                    "text_length": len(clean_text)
                }
            )
            
        except Exception as e:
            self.logger.error(f"Failed to analyze text sentiment: {e}")
            return SentimentSignal(
                text=text,
                sentiment_score=0.0,
                confidence=0.0,
                keywords=[],
                source="text_analysis",
                timestamp=datetime.now(),
                metadata={"error": str(e)}
            )
    
    def _clean_text(self, text: str) -> str:
        """Clean and normalize text for analysis"""
        # Remove URLs
        text = re.sub(r'http[s]?://(?:[a-zA-Z]|[0-9]|[$-_@.&+]|[!*\\(\\),]|(?:%[0-9a-fA-F][0-9a-fA-F]))+', '', text)
        
        # Remove excessive emojis (keep some for context)
        text = re.sub(r'(.)\1{3,}', r'\1\1', text)  # Reduce repeated characters
        
        # Normalize whitespace
        text = re.sub(r'\s+', ' ', text).strip()
        
        return text
    
    def _analyze_text_features(self, text: str) -> float:
        """Analyze additional text features for sentiment"""
        modifiers = 0.0
        
        # Excessive caps (usually indicates strong emotion)
        caps_ratio = sum(1 for c in text if c.isupper()) / max(1, len(text))
        if caps_ratio > 0.3:
            modifiers += 0.2 if caps_ratio > 0.5 else 0.1
        
        # Exclamation marks
        exclamation_count = text.count('!')
        modifiers += min(0.3, exclamation_count * 0.1)
        
        # Question marks (uncertainty)
        question_count = text.count('?')
        modifiers -= min(0.2, question_count * 0.05)
        
        # Emojis (positive sentiment)
        positive_emojis = ['🚀', '🌙', '💎', '📈', '💰', '🔥', '⬆️', '✅']
        negative_emojis = ['📉', '💩', '⬇️', '❌', '😭', '💀', '🩸']
        
        for emoji in positive_emojis:
            modifiers += text.count(emoji) * 0.1
        
        for emoji in negative_emojis:
            modifiers -= text.count(emoji) * 0.1
        
        return min(0.5, max(-0.5, modifiers))
    
    def analyze_sentiment_batch(
        self, 
        texts: List[str], 
        weights: Optional[List[float]] = None
    ) -> SentimentAnalysis:
        """
        Analyze sentiment for a batch of texts
        
        Args:
            texts: List of texts to analyze
            weights: Optional weights for each text
            
        Returns:
            Comprehensive sentiment analysis
        """
        try:
            if not texts:
                return SentimentAnalysis(
                    overall_sentiment=0.0,
                    confidence=0.0,
                    signal_count=0,
                    bullish_signals=0,
                    bearish_signals=0,
                    neutral_signals=0,
                    keyword_analysis={},
                    emotion_analysis={},
                    trend_analysis={},
                    risk_indicators=[]
                )
            
            # Default equal weights
            if weights is None:
                weights = [1.0] * len(texts)
            elif len(weights) != len(texts):
                weights = [1.0] * len(texts)
            
            # Analyze each text
            signals = []
            for text in texts:
                signal = self.analyze_text_sentiment(text)
                signals.append(signal)
            
            # Calculate weighted sentiment
            total_weighted_sentiment = 0.0
            total_weight = 0.0
            
            for signal, weight in zip(signals, weights):
                weighted_sentiment = signal.sentiment_score * signal.confidence * weight
                total_weighted_sentiment += weighted_sentiment
                total_weight += signal.confidence * weight
            
            overall_sentiment = total_weighted_sentiment / total_weight if total_weight > 0 else 0.0
            
            # Calculate signal distribution
            bullish_signals = sum(1 for s in signals if s.sentiment_score > 0.1)
            bearish_signals = sum(1 for s in signals if s.sentiment_score < -0.1)
            neutral_signals = len(signals) - bullish_signals - bearish_signals
            
            # Keyword analysis
            keyword_analysis = self._analyze_keywords(signals)
            
            # Emotion analysis
            emotion_analysis = self._analyze_emotions(texts)
            
            # Trend analysis
            trend_analysis = self._analyze_trends(signals)
            
            # Risk indicators
            risk_indicators = self._identify_risk_indicators(signals)
            
            # Calculate overall confidence
            avg_confidence = sum(s.confidence for s in signals) / len(signals) if signals else 0.0
            signal_consistency = 1.0 - (abs(bullish_signals - bearish_signals) / len(signals))
            overall_confidence = (avg_confidence + signal_consistency) / 2
            
            return SentimentAnalysis(
                overall_sentiment=round(overall_sentiment, 3),
                confidence=round(overall_confidence, 3),
                signal_count=len(signals),
                bullish_signals=bullish_signals,
                bearish_signals=bearish_signals,
                neutral_signals=neutral_signals,
                keyword_analysis=keyword_analysis,
                emotion_analysis=emotion_analysis,
                trend_analysis=trend_analysis,
                risk_indicators=risk_indicators
            )
            
        except Exception as e:
            self.logger.error(f"Failed to analyze sentiment batch: {e}")
            return SentimentAnalysis(
                overall_sentiment=0.0,
                confidence=0.0,
                signal_count=0,
                bullish_signals=0,
                bearish_signals=0,
                neutral_signals=0,
                keyword_analysis={},
                emotion_analysis={},
                trend_analysis={},
                risk_indicators=[]
            )
    
    def _analyze_keywords(self, signals: List[SentimentSignal]) -> Dict[str, float]:
        """Analyze keyword frequency and impact"""
        keyword_scores = {}
        keyword_counts = {}
        
        for signal in signals:
            for keyword in signal.keywords:
                keyword_lower = keyword.lower()
                if keyword_lower not in keyword_scores:
                    keyword_scores[keyword_lower] = 0.0
                    keyword_counts[keyword_lower] = 0
                
                keyword_scores[keyword_lower] += signal.sentiment_score
                keyword_counts[keyword_lower] += 1
        
        # Calculate average impact per keyword
        keyword_analysis = {}
        for keyword, total_score in keyword_scores.items():
            count = keyword_counts[keyword]
            avg_impact = total_score / count if count > 0 else 0.0
            keyword_analysis[keyword] = round(avg_impact, 3)
        
        # Sort by absolute impact
        sorted_keywords = sorted(
            keyword_analysis.items(), 
            key=lambda x: abs(x[1]), 
            reverse=True
        )
        
        return dict(sorted_keywords[:20])  # Top 20 keywords
    
    def _analyze_emotions(self, texts: List[str]) -> Dict[str, float]:
        """Analyze emotional content of texts"""
        emotion_scores = {emotion: 0.0 for emotion in self.emotion_patterns.keys()}
        total_texts = len(texts)
        
        for text in texts:
            clean_text = self._clean_text(text)
            
            for emotion, pattern in self.emotion_patterns.items():
                matches = pattern.findall(clean_text)
                emotion_scores[emotion] += len(matches)
        
        # Normalize by text count
        for emotion in emotion_scores:
            emotion_scores[emotion] = round(emotion_scores[emotion] / total_texts, 3)
        
        return emotion_scores
    
    def _analyze_trends(self, signals: List[SentimentSignal]) -> Dict[str, float]:
        """Analyze sentiment trends over time"""
        if len(signals) < 2:
            return {"trend": 0.0, "volatility": 0.0, "momentum": 0.0}
        
        # Sort signals by timestamp
        sorted_signals = sorted(signals, key=lambda x: x.timestamp)
        sentiments = [s.sentiment_score for s in sorted_signals]
        
        # Calculate trend (linear regression slope approximation)
        n = len(sentiments)
        x_vals = list(range(n))
        
        sum_x = sum(x_vals)
        sum_y = sum(sentiments)
        sum_xy = sum(x * y for x, y in zip(x_vals, sentiments))
        sum_x2 = sum(x * x for x in x_vals)
        
        trend = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x) if (n * sum_x2 - sum_x * sum_x) != 0 else 0.0
        
        # Calculate volatility (standard deviation)
        mean_sentiment = sum_y / n
        variance = sum((s - mean_sentiment) ** 2 for s in sentiments) / n
        volatility = math.sqrt(variance)
        
        # Calculate momentum (recent vs. early sentiment)
        recent_count = max(1, n // 3)
        recent_avg = sum(sentiments[-recent_count:]) / recent_count
        early_avg = sum(sentiments[:recent_count]) / recent_count
        momentum = recent_avg - early_avg
        
        return {
            "trend": round(trend, 3),
            "volatility": round(volatility, 3),
            "momentum": round(momentum, 3)
        }
    
    def _identify_risk_indicators(self, signals: List[SentimentSignal]) -> List[str]:
        """Identify potential risk indicators from signals"""
        risk_indicators = []
        
        # Check for scam/rug pull mentions
        scam_count = sum(1 for s in signals for k in s.keywords if "scam" in k.lower())
        if scam_count > len(signals) * 0.1:  # More than 10% mention scams
            risk_indicators.append("High scam mentions detected")
        
        # Check for extreme sentiment swings
        sentiments = [s.sentiment_score for s in signals]
        if sentiments:
            sentiment_range = max(sentiments) - min(sentiments)
            if sentiment_range > 1.5:
                risk_indicators.append("Extreme sentiment volatility")
        
        # Check for manipulation keywords
        manipulation_keywords = ["pump and dump", "coordinated", "manipulation", "artificial"]
        manipulation_count = sum(
            1 for s in signals 
            for keyword in manipulation_keywords 
            if keyword in s.text.lower()
        )
        
        if manipulation_count > 0:
            risk_indicators.append("Potential market manipulation signals")
        
        # Check for low confidence signals
        low_confidence_count = sum(1 for s in signals if s.confidence < 0.3)
        if low_confidence_count > len(signals) * 0.5:
            risk_indicators.append("High proportion of low-confidence signals")
        
        return risk_indicators
    
    def calculate_fear_greed_index(
        self, 
        sentiment_data: List[float],
        volume_data: Optional[List[float]] = None,
        volatility_data: Optional[List[float]] = None
    ) -> Dict[str, Any]:
        """
        Calculate a crypto Fear & Greed Index based on sentiment and market data
        
        Args:
            sentiment_data: List of sentiment scores
            volume_data: Optional trading volume data
            volatility_data: Optional price volatility data
            
        Returns:
            Fear & Greed Index with breakdown
        """
        try:
            if not sentiment_data:
                return {"index": 50, "label": "Neutral", "components": {}}
            
            # Sentiment component (40% weight)
            avg_sentiment = sum(sentiment_data) / len(sentiment_data)
            sentiment_score = (avg_sentiment + 1) * 50  # Convert -1,1 to 0,100
            
            components = {"sentiment": sentiment_score}
            total_weight = 0.4
            weighted_score = sentiment_score * 0.4
            
            # Volume component (30% weight) if available
            if volume_data:
                # Normalize volume (higher volume = more greed)
                avg_volume = sum(volume_data) / len(volume_data)
                max_volume = max(volume_data) if volume_data else avg_volume
                volume_score = min(100, (avg_volume / max_volume) * 100) if max_volume > 0 else 50
                
                components["volume"] = volume_score
                weighted_score += volume_score * 0.3
                total_weight += 0.3
            
            # Volatility component (30% weight) if available
            if volatility_data:
                # Higher volatility = more fear
                avg_volatility = sum(volatility_data) / len(volatility_data)
                # Normalize and invert (high volatility = low score)
                volatility_score = max(0, 100 - (avg_volatility * 100))
                
                components["volatility"] = volatility_score
                weighted_score += volatility_score * 0.3
                total_weight += 0.3
            
            # Calculate final index
            fear_greed_index = int(weighted_score / total_weight) if total_weight > 0 else 50
            
            # Determine label
            if fear_greed_index <= 25:
                label = "Extreme Fear"
            elif fear_greed_index <= 45:
                label = "Fear"
            elif fear_greed_index <= 55:
                label = "Neutral"
            elif fear_greed_index <= 75:
                label = "Greed"
            else:
                label = "Extreme Greed"
            
            return {
                "index": fear_greed_index,
                "label": label,
                "components": components,
                "recommendation": self._get_fear_greed_recommendation(fear_greed_index)
            }
            
        except Exception as e:
            self.logger.error(f"Failed to calculate Fear & Greed Index: {e}")
            return {"index": 50, "label": "Neutral", "components": {}, "error": str(e)}
    
    def _get_fear_greed_recommendation(self, index: int) -> str:
        """Get trading recommendation based on Fear & Greed Index"""
        if index <= 25:
            return "Extreme fear often presents buying opportunities for contrarian investors"
        elif index <= 45:
            return "Market fear may indicate oversold conditions - consider cautious accumulation"
        elif index <= 55:
            return "Neutral sentiment - wait for clearer directional signals"
        elif index <= 75:
            return "Market greed suggests caution - consider taking profits or reducing positions"
        else:
            return "Extreme greed often precedes corrections - consider defensive positioning"