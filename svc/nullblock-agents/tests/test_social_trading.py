"""
Comprehensive tests for social trading agents
"""

import pytest
import asyncio
from datetime import datetime, timedelta
from unittest.mock import Mock, AsyncMock, patch
import sys
import os

# Add src to path for imports
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'src'))

from agents.social_trading.social_monitor import SocialMonitorAgent, SocialSignal
from agents.social_trading.sentiment_analyzer import SentimentAnalyzer, SentimentMetrics, TradingSignal
from agents.social_trading.risk_manager import RiskManager, RiskProfile, PositionSizing, RiskMetrics


class TestSocialMonitorAgent:
    """Test social media monitoring functionality"""
    
    @pytest.fixture
    def monitor_agent(self):
        """Create a test social monitor agent"""
        config = {
            "twitter_bearer_token": "test_token",
            "dextools_api_key": "test_key",
            "update_interval": 1  # Fast for testing
        }
        return SocialMonitorAgent(config)
    
    @pytest.mark.asyncio
    async def test_twitter_monitor_search(self, monitor_agent):
        """Test Twitter search functionality"""
        # Test with mock data
        signals = await monitor_agent.twitter_monitor.search_tweets("$BONK", limit=10)
        
        assert isinstance(signals, list)
        assert len(signals) > 0
        
        for signal in signals:
            assert isinstance(signal, SocialSignal)
            assert signal.source == "twitter"
            assert signal.token_symbol in ["BONK"]
            assert -1.0 <= signal.sentiment_score <= 1.0
            assert 0.0 <= signal.engagement_score <= 1.0
    
    @pytest.mark.asyncio
    async def test_gmgn_trending_tokens(self, monitor_agent):
        """Test GMGN trending tokens functionality"""
        signals = await monitor_agent.gmgn_monitor.get_trending_tokens(limit=5)
        
        assert isinstance(signals, list)
        assert len(signals) <= 5
        
        for signal in signals:
            assert isinstance(signal, SocialSignal)
            assert signal.source == "gmgn"
            assert signal.token_symbol is not None
            assert signal.token_address is not None
    
    @pytest.mark.asyncio
    async def test_dextools_hot_pairs(self, monitor_agent):
        """Test DEXTools hot pairs functionality"""
        signals = await monitor_agent.dextools_monitor.get_hot_pairs()
        
        assert isinstance(signals, list)
        
        for signal in signals:
            assert isinstance(signal, SocialSignal)
            assert signal.source == "dextools"
            assert signal.token_symbol is not None
    
    @pytest.mark.asyncio
    async def test_aggregate_sentiment(self, monitor_agent):
        """Test sentiment aggregation"""
        # Add some mock signals to cache
        monitor_agent.signal_cache = {
            "BONK": [
                SocialSignal(
                    source="twitter",
                    token_symbol="BONK",
                    token_address=None,
                    sentiment_score=0.8,
                    engagement_score=0.7,
                    content="BONK to the moon!",
                    author="test_user",
                    timestamp=datetime.now()
                ),
                SocialSignal(
                    source="gmgn",
                    token_symbol="BONK",
                    token_address="DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",
                    sentiment_score=0.6,
                    engagement_score=0.9,
                    content="BONK trending up",
                    author="gmgn_api",
                    timestamp=datetime.now()
                )
            ]
        }
        
        sentiment, confidence = monitor_agent.get_aggregated_sentiment("BONK")
        
        assert 0.0 <= sentiment <= 1.0  # Should be positive
        assert 0.0 <= confidence <= 1.0
        assert sentiment > 0.5  # Should be bullish based on mock data
    
    @pytest.mark.asyncio
    async def test_trending_tokens(self, monitor_agent):
        """Test trending tokens identification"""
        # Mock some signals
        monitor_agent.signal_cache = {
            "BONK": [SocialSignal(
                source="twitter", token_symbol="BONK", token_address=None,
                sentiment_score=0.8, engagement_score=0.7, content="test",
                author="test", timestamp=datetime.now()
            )] * 3,  # 3 signals
            "WIF": [SocialSignal(
                source="twitter", token_symbol="WIF", token_address=None,
                sentiment_score=-0.3, engagement_score=0.5, content="test",
                author="test", timestamp=datetime.now()
            )] * 2   # 2 signals
        }
        
        trending = monitor_agent.get_trending_tokens(min_signals=2, min_confidence=0.2)
        
        assert isinstance(trending, list)
        assert len(trending) == 2  # Both tokens should qualify
        
        # Check sorting (higher confidence * abs(sentiment) first)
        if len(trending) > 1:
            first_score = trending[0][2] * abs(trending[0][1])
            second_score = trending[1][2] * abs(trending[1][1])
            assert first_score >= second_score


class TestSentimentAnalyzer:
    """Test sentiment analysis functionality"""
    
    @pytest.fixture
    def sentiment_analyzer(self):
        """Create a test sentiment analyzer"""
        return SentimentAnalyzer()
    
    def test_text_sentiment_analysis(self, sentiment_analyzer):
        """Test basic text sentiment analysis"""
        # Test bullish text
        bullish_signal = sentiment_analyzer.analyze_text_sentiment(
            "BONK is going to the moon! 🚀 This is the best investment ever!"
        )
        
        assert isinstance(bullish_signal.sentiment_score, float)
        assert bullish_signal.sentiment_score > 0.0  # Should be positive
        assert bullish_signal.confidence > 0.0
        assert len(bullish_signal.keywords) > 0
        
        # Test bearish text
        bearish_signal = sentiment_analyzer.analyze_text_sentiment(
            "BONK is a scam! Dump everything now before it crashes to zero!"
        )
        
        assert bearish_signal.sentiment_score < 0.0  # Should be negative
        
        # Test neutral text
        neutral_signal = sentiment_analyzer.analyze_text_sentiment(
            "BONK price is stable today. No major movements expected."
        )
        
        assert abs(neutral_signal.sentiment_score) < 0.3  # Should be relatively neutral
    
    def test_batch_sentiment_analysis(self, sentiment_analyzer):
        """Test batch sentiment analysis"""
        texts = [
            "BONK to the moon! 🚀",
            "This token is going to crash",
            "Neutral update on price",
            "Amazing project with great potential!",
            "Scam alert! Don't buy this!"
        ]
        
        analysis = sentiment_analyzer.analyze_sentiment_batch(texts)
        
        assert isinstance(analysis, SentimentAnalysis)
        assert analysis.signal_count == len(texts)
        assert analysis.bullish_signals > 0
        assert analysis.bearish_signals > 0
        assert analysis.neutral_signals >= 0
        assert len(analysis.keyword_analysis) > 0
        assert len(analysis.emotion_analysis) > 0
    
    def test_fear_greed_index(self, sentiment_analyzer):
        """Test Fear & Greed Index calculation"""
        # Test with bullish sentiment
        bullish_sentiments = [0.8, 0.6, 0.7, 0.9, 0.5]
        
        fear_greed = sentiment_analyzer.calculate_fear_greed_index(bullish_sentiments)
        
        assert isinstance(fear_greed, dict)
        assert "index" in fear_greed
        assert "label" in fear_greed
        assert 0 <= fear_greed["index"] <= 100
        assert fear_greed["index"] > 50  # Should indicate greed
        
        # Test with bearish sentiment
        bearish_sentiments = [-0.8, -0.6, -0.7, -0.9, -0.5]
        
        fear_greed = sentiment_analyzer.calculate_fear_greed_index(bearish_sentiments)
        
        assert fear_greed["index"] < 50  # Should indicate fear
    
    def test_sentiment_metrics_analysis(self, sentiment_analyzer):
        """Test comprehensive sentiment metrics"""
        # Mock social signals
        mock_signals = [
            {
                "token_symbol": "BONK",
                "sentiment_score": 0.8,
                "source": "twitter",
                "author": "crypto_bull",
                "engagement_score": 0.7,
                "timestamp": datetime.now(),
                "content": "BONK is going to moon! 🚀"
            },
            {
                "token_symbol": "BONK",
                "sentiment_score": 0.6,
                "source": "gmgn",
                "author": "gmgn_api",
                "engagement_score": 0.9,
                "timestamp": datetime.now() - timedelta(minutes=30),
                "content": "BONK trending up +15%"
            },
            {
                "token_symbol": "BONK",
                "sentiment_score": -0.3,
                "source": "twitter",
                "author": "bear_trader",
                "engagement_score": 0.4,
                "timestamp": datetime.now() - timedelta(hours=1),
                "content": "BONK might be overvalued"
            }
        ]
        
        metrics = sentiment_analyzer.analyze_sentiment_metrics(mock_signals)
        
        assert isinstance(metrics, SentimentMetrics)
        assert metrics.token_symbol == "BONK"
        assert metrics.social_volume == len(mock_signals)
        assert 0.0 <= metrics.engagement_quality <= 1.0
        assert 0 <= metrics.fear_greed_index <= 100
        assert 0.0 <= metrics.volatility_score <= 1.0
    
    def test_trading_signal_generation(self, sentiment_analyzer):
        """Test trading signal generation"""
        # Create bullish sentiment metrics
        bullish_metrics = SentimentMetrics(
            token_symbol="BONK",
            overall_sentiment=0.7,
            sentiment_momentum=0.5,
            social_volume=10,
            engagement_quality=0.8,
            influencer_sentiment=0.6,
            retail_sentiment=0.7,
            fear_greed_index=75,
            volatility_score=0.3,
            trend_strength=0.6,
            risk_score=0.4
        )
        
        signal = sentiment_analyzer.generate_trading_signal(
            bullish_metrics, 
            current_price=0.000025
        )
        
        assert isinstance(signal, TradingSignal)
        assert signal.token_symbol == "BONK"
        assert signal.signal_type in ["BUY", "SELL", "HOLD"]
        assert 0.0 <= signal.strength <= 1.0
        assert 0.0 <= signal.confidence <= 1.0
        assert signal.time_horizon in ["SHORT", "MEDIUM", "LONG"]
        
        # For bullish metrics, should likely be a BUY signal
        assert signal.signal_type == "BUY"
        assert signal.strength > 0.5


class TestRiskManager:
    """Test risk management functionality"""
    
    @pytest.fixture
    def risk_profile(self):
        """Create a test risk profile"""
        return RiskProfile(
            risk_tolerance="MEDIUM",
            max_portfolio_risk=0.05,
            max_position_size=0.10,
            max_correlation_exposure=0.30,
            stop_loss_percentage=0.15,
            take_profit_percentage=0.50
        )
    
    @pytest.fixture
    def risk_manager(self, risk_profile):
        """Create a test risk manager"""
        return RiskManager(risk_profile)
    
    def test_position_sizing(self, risk_manager):
        """Test position sizing calculation"""
        position_sizing = risk_manager.calculate_position_size(
            token_symbol="BONK",
            current_price=0.000025,
            portfolio_value=10000.0,
            sentiment_score=0.7,
            confidence=0.8,
            volatility=0.3
        )
        
        assert isinstance(position_sizing, PositionSizing)
        assert position_sizing.final_size_usd > 0.0
        assert position_sizing.recommended_size_tokens > 0.0
        assert position_sizing.position_as_portfolio_pct <= 25.0  # Hard limit
        assert len(position_sizing.reasoning) > 0
        
        # Check that positive sentiment increases position size
        assert position_sizing.sentiment_adjusted_size >= position_sizing.recommended_size_usd
    
    def test_token_risk_analysis(self, risk_manager):
        """Test token risk analysis"""
        token_data = {
            "market_cap_usd": 50_000_000,  # Medium market cap
            "liquidity_usd": 100_000,
            "volume_24h_usd": 500_000,
            "price_change_24h": 15.5,  # High volatility
            "holder_count": 5000
        }
        
        social_signals = [
            {
                "sentiment_score": 0.6,
                "content": "Great project with solid fundamentals",
                "engagement_score": 0.7
            },
            {
                "sentiment_score": -0.2,
                "content": "Might be overvalued right now",
                "engagement_score": 0.4
            }
        ]
        
        risk_metrics = risk_manager.analyze_token_risk(
            "BONK", 
            token_data, 
            social_signals
        )
        
        assert isinstance(risk_metrics, RiskMetrics)
        assert risk_metrics.token_symbol == "BONK"
        assert risk_metrics.market_cap_usd == 50_000_000
        assert 0.0 <= risk_metrics.overall_risk_score <= 1.0
        assert risk_metrics.risk_category in ["LOW", "MEDIUM", "HIGH", "EXTREME"]
        assert 0.0 <= risk_metrics.max_recommended_allocation <= 1.0
    
    def test_stop_loss_creation(self, risk_manager):
        """Test stop loss order creation"""
        stop_loss = risk_manager.create_stop_loss_order(
            token_symbol="BONK",
            entry_price=0.000025,
            risk_amount=100.0,
            position_size=1000.0,
            trailing=True
        )
        
        assert stop_loss.token_symbol == "BONK"
        assert stop_loss.entry_price == 0.000025
        assert stop_loss.stop_price < stop_loss.entry_price
        assert 0.0 < stop_loss.stop_percentage < 1.0
        assert stop_loss.trailing is True
        assert stop_loss.trail_distance > 0.0
    
    def test_take_profit_creation(self, risk_manager):
        """Test take profit order creation"""
        take_profit = risk_manager.create_take_profit_order(
            token_symbol="BONK",
            entry_price=0.000025,
            sentiment_score=0.7,
            confidence=0.8,
            partial_exit=True
        )
        
        assert take_profit.token_symbol == "BONK"
        assert take_profit.entry_price == 0.000025
        assert take_profit.target_price > take_profit.entry_price
        assert take_profit.target_percentage > 0.0
        assert take_profit.partial_exit is True
        assert 0.0 < take_profit.exit_percentage <= 1.0
    
    def test_portfolio_risk_analysis(self, risk_manager):
        """Test portfolio risk analysis"""
        portfolio_positions = [
            {
                "symbol": "BONK",
                "value_usd": 1000.0,
                "risk_score": 0.6,
                "category": "meme",
                "liquidity_usd": 50000,
                "market_cap_usd": 50_000_000
            },
            {
                "symbol": "SOL",
                "value_usd": 5000.0,
                "risk_score": 0.3,
                "category": "layer1",
                "liquidity_usd": 10_000_000,
                "market_cap_usd": 50_000_000_000
            }
        ]
        
        historical_pnl = [100.0, 150.0, 120.0, 180.0, 160.0]
        
        portfolio_risk = risk_manager.analyze_portfolio_risk(
            portfolio_positions,
            historical_pnl
        )
        
        assert portfolio_risk.total_value_usd == 6000.0
        assert 0.0 <= portfolio_risk.total_risk_exposure <= 1.0
        assert portfolio_risk.daily_pnl == 160.0  # Last P&L value
        assert 0.0 <= portfolio_risk.concentration_risk <= 1.0
        assert isinstance(portfolio_risk.risk_warnings, list)
        assert isinstance(portfolio_risk.recommended_actions, list)
    
    def test_trade_execution_decision(self, risk_manager):
        """Test trade execution decision logic"""
        # Create valid position sizing
        position_sizing = PositionSizing(
            recommended_size_usd=1000.0,
            recommended_size_tokens=40_000_000,
            risk_amount_usd=150.0,
            position_as_portfolio_pct=10.0,
            confidence_adjusted_size=1000.0,
            sentiment_adjusted_size=1000.0,
            volatility_adjusted_size=1000.0,
            final_size_usd=1000.0,
            reasoning=["Valid position"]
        )
        
        # Create medium risk metrics
        risk_metrics = RiskMetrics(
            token_symbol="BONK",
            market_cap_usd=50_000_000,
            liquidity_score=0.6,
            volatility_score=0.4,
            sentiment_risk=0.3,
            technical_risk=0.4,
            social_risk=0.2,
            overall_risk_score=0.5,
            risk_category="MEDIUM",
            max_recommended_allocation=0.10
        )
        
        # Create acceptable portfolio risk
        portfolio_risk = PortfolioRisk(
            total_value_usd=10000.0,
            total_risk_exposure=0.4,
            daily_pnl=50.0,
            drawdown_from_peak=0.05,
            concentration_risk=0.20,
            correlation_risk=0.25,
            liquidity_risk=0.15,
            positions_at_risk=[],
            risk_warnings=[],
            recommended_actions=[]
        )
        
        should_execute, reasons = risk_manager.should_execute_trade(
            position_sizing,
            risk_metrics,
            portfolio_risk
        )
        
        assert isinstance(should_execute, bool)
        assert isinstance(reasons, list)
        assert len(reasons) > 0
        
        # Should execute with good parameters
        assert should_execute is True


class TestIntegration:
    """Integration tests for social trading system"""
    
    @pytest.mark.asyncio
    async def test_end_to_end_trading_decision(self):
        """Test complete trading decision pipeline"""
        # 1. Create components
        monitor = SocialMonitorAgent({"update_interval": 1})
        analyzer = SentimentAnalyzer()
        risk_profile = RiskProfile(risk_tolerance="MEDIUM")
        risk_manager = RiskManager(risk_profile)
        
        # 2. Mock social signals collection
        await monitor._collect_all_signals()
        
        # 3. Get signals for a token
        bonk_signals = monitor.get_token_signals("BONK")
        
        if bonk_signals:
            # 4. Analyze sentiment
            sentiment_metrics = analyzer.analyze_sentiment_metrics([
                {
                    "token_symbol": "BONK",
                    "sentiment_score": signal.sentiment_score,
                    "source": signal.source,
                    "author": signal.author,
                    "engagement_score": signal.engagement_score,
                    "timestamp": signal.timestamp,
                    "content": signal.content
                }
                for signal in bonk_signals
            ])
            
            # 5. Generate trading signal
            trading_signal = analyzer.generate_trading_signal(sentiment_metrics)
            
            # 6. Risk analysis
            token_data = {
                "market_cap_usd": 180_000_000,
                "liquidity_usd": 500_000,
                "volume_24h_usd": 2_000_000,
                "price_change_24h": 12.5,
                "holder_count": 15000
            }
            
            risk_metrics = risk_manager.analyze_token_risk(
                "BONK", 
                token_data, 
                bonk_signals
            )
            
            # 7. Position sizing
            position_sizing = risk_manager.calculate_position_size(
                token_symbol="BONK",
                current_price=0.000025,
                portfolio_value=10000.0,
                sentiment_score=sentiment_metrics.overall_sentiment,
                confidence=sentiment_metrics.engagement_quality,
                volatility=sentiment_metrics.volatility_score,
                risk_metrics=risk_metrics
            )
            
            # 8. Final decision
            portfolio_risk = PortfolioRisk(
                total_value_usd=10000.0,
                total_risk_exposure=0.3,
                daily_pnl=0.0,
                drawdown_from_peak=0.0,
                concentration_risk=0.1,
                correlation_risk=0.2,
                liquidity_risk=0.1,
                positions_at_risk=[],
                risk_warnings=[],
                recommended_actions=[]
            )
            
            should_execute, reasons = risk_manager.should_execute_trade(
                position_sizing,
                risk_metrics,
                portfolio_risk
            )
            
            # Verify the pipeline worked
            assert isinstance(sentiment_metrics, SentimentMetrics)
            assert isinstance(trading_signal, TradingSignal)
            assert isinstance(risk_metrics, RiskMetrics)
            assert isinstance(position_sizing, PositionSizing)
            assert isinstance(should_execute, bool)
            assert len(reasons) > 0
            
            print(f"Trading Decision for BONK:")
            print(f"  Signal: {trading_signal.signal_type}")
            print(f"  Strength: {trading_signal.strength:.2f}")
            print(f"  Risk Category: {risk_metrics.risk_category}")
            print(f"  Position Size: ${position_sizing.final_size_usd:.2f}")
            print(f"  Should Execute: {should_execute}")
            print(f"  Reasons: {reasons}")


# Test configuration
@pytest.fixture(scope="session")
def event_loop():
    """Create an instance of the default event loop for the test session."""
    loop = asyncio.get_event_loop_policy().new_event_loop()
    yield loop
    loop.close()


if __name__ == "__main__":
    # Run tests with pytest
    pytest.main([__file__, "-v", "--tb=short"])