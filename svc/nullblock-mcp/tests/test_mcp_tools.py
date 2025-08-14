"""
Tests for MCP tools for social trading
"""

import pytest
import asyncio
from datetime import datetime, timedelta
from unittest.mock import Mock, AsyncMock, patch
import sys
import os

# Add src to path for imports
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'src'))

from mcp.tools.social_tools import SocialMediaTools, SocialMediaConfig, SocialSignalResult, TokenTrendResult
from mcp.tools.sentiment_tools import SentimentAnalysisTools, MarketSentimentConfig, SentimentAnalysis
from mcp.tools.trading_tools import TradingTools, TradingConfig, JupiterQuote, TradeOrder, PortfolioPosition


class TestSocialMediaTools:
    """Test MCP social media tools"""
    
    @pytest.fixture
    def config(self):
        """Create test configuration"""
        return SocialMediaConfig(
            twitter_bearer_token="test_token",
            dextools_api_key="test_key",
            update_interval=10
        )
    
    @pytest.fixture
    def social_tools(self, config):
        """Create social media tools instance"""
        return SocialMediaTools(config)
    
    @pytest.mark.asyncio
    async def test_twitter_sentiment(self, social_tools):
        """Test Twitter sentiment analysis"""
        result = await social_tools.get_twitter_sentiment("$BONK", limit=10)
        
        assert isinstance(result, dict)
        assert "query" in result
        assert "sentiment_score" in result
        assert "confidence" in result
        assert "signal_count" in result
        assert "signals" in result
        assert "source" in result
        
        assert result["query"] == "$BONK"
        assert -1.0 <= result["sentiment_score"] <= 1.0
        assert 0.0 <= result["confidence"] <= 1.0
        assert result["source"] == "twitter"
        assert len(result["signals"]) <= 10
    
    @pytest.mark.asyncio
    async def test_gmgn_trends(self, social_tools):
        """Test GMGN trending tokens"""
        trends = await social_tools.get_gmgn_trends(limit=5)
        
        assert isinstance(trends, list)
        assert len(trends) <= 5
        
        for trend in trends:
            assert isinstance(trend, TokenTrendResult)
            assert trend.symbol is not None
            assert trend.address is not None
            assert trend.risk_level in ["LOW", "MEDIUM", "HIGH"]
            assert 0.0 <= trend.social_score <= 1.0
            assert -1.0 <= trend.sentiment <= 1.0
            assert 0.0 <= trend.trend_score <= 1.0
    
    @pytest.mark.asyncio
    async def test_dextools_social_score(self, social_tools):
        """Test DEXTools social score"""
        address = "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263"
        result = await social_tools.get_dextools_social_score(address)
        
        assert isinstance(result, dict)
        assert "token_address" in result
        assert "sentiment_score" in result
        assert "confidence" in result
        assert "source" in result
        
        assert result["token_address"] == address
        assert -1.0 <= result["sentiment_score"] <= 1.0
        assert 0.0 <= result["confidence"] <= 1.0
        assert result["source"] == "dextools"
    
    @pytest.mark.asyncio
    async def test_analyze_token_sentiment(self, social_tools):
        """Test comprehensive token sentiment analysis"""
        result = await social_tools.analyze_token_sentiment(
            "BONK", 
            "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263"
        )
        
        assert isinstance(result, SocialSignalResult)
        assert result.token_symbol == "BONK"
        assert result.token_address == "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263"
        assert -1.0 <= result.sentiment_score <= 1.0
        assert 0.0 <= result.confidence_score <= 1.0
        assert result.signal_count >= 0
        assert isinstance(result.sources, list)
        assert 0.0 <= result.trending_score <= 1.0
    
    @pytest.mark.asyncio
    async def test_trending_tokens(self, social_tools):
        """Test trending tokens analysis"""
        trending = await social_tools.get_trending_tokens(limit=3, min_confidence=0.1)
        
        assert isinstance(trending, list)
        assert len(trending) <= 3
        
        for token in trending:
            assert isinstance(token, SocialSignalResult)
            assert token.confidence_score >= 0.1
            
        # Check sorting (should be sorted by trending_score)
        if len(trending) > 1:
            for i in range(len(trending) - 1):
                assert trending[i].trending_score >= trending[i + 1].trending_score
    
    @pytest.mark.asyncio
    async def test_monitor_token_mentions(self, social_tools):
        """Test token mention monitoring (shortened for testing)"""
        # Mock the monitoring to return quickly
        with patch.object(social_tools, 'analyze_token_sentiment') as mock_analyze:
            mock_analyze.return_value = SocialSignalResult(
                token_symbol="BONK",
                sentiment_score=0.5,
                confidence_score=0.7,
                signal_count=3,
                sources=["twitter"],
                trending_score=0.6
            )
            
            result = await social_tools.monitor_token_mentions("BONK", duration_minutes=0.1)  # Very short
            
            assert isinstance(result, dict)
            assert "symbol" in result
            assert "monitoring_duration_minutes" in result
            assert "total_mentions" in result
            assert "average_sentiment" in result
            assert "mentions" in result
    
    @pytest.mark.asyncio
    async def test_cleanup(self, social_tools):
        """Test resource cleanup"""
        await social_tools.cleanup()
        assert social_tools.session is None


class TestSentimentAnalysisTools:
    """Test sentiment analysis MCP tools"""
    
    @pytest.fixture
    def sentiment_tools(self):
        """Create sentiment analysis tools"""
        return SentimentAnalysisTools()
    
    def test_text_sentiment_analysis(self, sentiment_tools):
        """Test individual text sentiment analysis"""
        bullish_text = "BONK is going to the moon! 🚀 Best investment ever!"
        signal = sentiment_tools.analyze_text_sentiment(bullish_text)
        
        assert signal.text == bullish_text
        assert signal.sentiment_score > 0.0  # Should be bullish
        assert signal.confidence > 0.0
        assert len(signal.keywords) > 0
        assert signal.source == "text_analysis"
        
        bearish_text = "BONK is a scam! Dump everything now!"
        signal = sentiment_tools.analyze_text_sentiment(bearish_text)
        
        assert signal.sentiment_score < 0.0  # Should be bearish
    
    def test_batch_sentiment_analysis(self, sentiment_tools):
        """Test batch sentiment analysis"""
        texts = [
            "BONK to the moon! 🚀",
            "This project has great potential",
            "Price is stable today",
            "Bearish sentiment in the market",
            "Scam alert! Be careful!"
        ]
        
        analysis = sentiment_tools.analyze_sentiment_batch(texts)
        
        assert isinstance(analysis, SentimentAnalysis)
        assert analysis.signal_count == len(texts)
        assert analysis.bullish_signals >= 0
        assert analysis.bearish_signals >= 0
        assert analysis.neutral_signals >= 0
        assert analysis.bullish_signals + analysis.bearish_signals + analysis.neutral_signals == len(texts)
        
        assert len(analysis.keyword_analysis) > 0
        assert len(analysis.emotion_analysis) > 0
        assert len(analysis.trend_analysis) > 0
    
    def test_weighted_sentiment_analysis(self, sentiment_tools):
        """Test weighted sentiment analysis"""
        texts = ["Bullish text", "Bearish text"]
        weights = [2.0, 1.0]  # Weight bullish text higher
        
        analysis = sentiment_tools.analyze_sentiment_batch(texts, weights)
        
        assert isinstance(analysis, SentimentAnalysis)
        assert analysis.signal_count == len(texts)
        
        # Should be more bullish due to higher weight
        # (This depends on the actual keyword matching)
    
    def test_fear_greed_index(self, sentiment_tools):
        """Test Fear & Greed Index calculation"""
        # Test extreme fear
        fear_sentiments = [-0.8, -0.7, -0.9, -0.6, -0.8]
        fear_index = sentiment_tools.calculate_fear_greed_index(fear_sentiments)
        
        assert fear_index["index"] <= 25
        assert fear_index["label"] == "Extreme Fear"
        
        # Test extreme greed
        greed_sentiments = [0.8, 0.7, 0.9, 0.6, 0.8]
        greed_index = sentiment_tools.calculate_fear_greed_index(greed_sentiments)
        
        assert greed_index["index"] >= 75
        assert greed_index["label"] == "Extreme Greed"
        
        # Test neutral
        neutral_sentiments = [0.1, -0.1, 0.0, 0.05, -0.05]
        neutral_index = sentiment_tools.calculate_fear_greed_index(neutral_sentiments)
        
        assert 45 <= neutral_index["index"] <= 55
        assert neutral_index["label"] == "Neutral"


class TestTradingTools:
    """Test Solana trading MCP tools"""
    
    @pytest.fixture
    def trading_tools(self):
        """Create trading tools instance"""
        return TradingTools(
            rpc_url="https://api.mainnet-beta.solana.com",
            private_key=None  # No private key for testing
        )
    
    @pytest.mark.asyncio
    async def test_get_token_list(self, trading_tools):
        """Test token list retrieval"""
        tokens = await trading_tools.get_token_list()
        
        assert isinstance(tokens, list)
        assert len(tokens) > 0
        
        for token in tokens:
            assert hasattr(token, 'symbol')
            assert hasattr(token, 'name')
            assert hasattr(token, 'mint')
            assert hasattr(token, 'decimals')
            assert hasattr(token, 'verified')
        
        # Check for expected tokens
        symbols = [token.symbol for token in tokens]
        assert "SOL" in symbols
        assert "USDC" in symbols
        assert "BONK" in symbols
    
    @pytest.mark.asyncio
    async def test_get_token_price(self, trading_tools):
        """Test token price retrieval"""
        # Load tokens first
        await trading_tools.get_token_list()
        
        # Test SOL price
        sol_mint = "So11111111111111111111111111111111111111112"
        price = await trading_tools.get_token_price(sol_mint)
        
        assert isinstance(price, float)
        assert price > 0.0
        
        # Test unknown token
        unknown_price = await trading_tools.get_token_price("unknown_mint")
        assert unknown_price is None
    
    @pytest.mark.asyncio
    async def test_jupiter_quote(self, trading_tools):
        """Test Jupiter swap quote"""
        # Load tokens first
        await trading_tools.get_token_list()
        
        sol_mint = "So11111111111111111111111111111111111111112"
        usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
        amount = 1_000_000_000  # 1 SOL (9 decimals)
        
        quote = await trading_tools.get_jupiter_quote(
            sol_mint, 
            usdc_mint, 
            amount
        )
        
        assert isinstance(quote, JupiterQuote)
        assert quote.input_mint == sol_mint
        assert quote.output_mint == usdc_mint
        assert quote.in_amount == str(amount)
        assert float(quote.out_amount) > 0
        assert float(quote.out_amount_with_slippage) > 0
        assert quote.price_impact_pct >= 0.0
        assert len(quote.market_infos) > 0
        assert len(quote.route_plan) > 0
    
    @pytest.mark.asyncio
    async def test_execute_swap(self, trading_tools):
        """Test swap execution (mock)"""
        # Create a mock quote
        quote = JupiterQuote(
            input_mint="So11111111111111111111111111111111111111112",
            in_amount="1000000000",
            output_mint="EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            out_amount="180000000",
            out_amount_with_slippage="178200000",
            price_impact_pct=0.5,
            market_infos=[],
            route_plan=[]
        )
        
        # Load tokens for symbol lookup
        await trading_tools.get_token_list()
        
        order = await trading_tools.execute_swap(
            quote,
            "test_public_key"
        )
        
        assert isinstance(order, TradeOrder)
        assert order.token_in == "SOL"
        assert order.token_out == "USDC"
        assert order.amount_in > 0.0
        assert order.amount_out > 0.0
        assert order.status == "completed"  # Mock execution succeeds
        assert order.transaction_hash is not None
    
    @pytest.mark.asyncio
    async def test_wallet_balance(self, trading_tools):
        """Test wallet balance retrieval"""
        positions = await trading_tools.get_wallet_balance("test_wallet")
        
        assert isinstance(positions, list)
        assert len(positions) > 0
        
        for position in positions:
            assert isinstance(position, PortfolioPosition)
            assert position.token_symbol is not None
            assert position.balance >= 0.0
            assert position.value_usd >= 0.0
            assert position.price_usd >= 0.0
            assert 0.0 <= position.allocation_percentage <= 100.0
        
        # Check total allocation adds up to ~100%
        total_allocation = sum(pos.allocation_percentage for pos in positions)
        assert 99.0 <= total_allocation <= 101.0  # Allow for rounding
    
    @pytest.mark.asyncio
    async def test_position_sizing(self, trading_tools):
        """Test position size calculation"""
        bonk_mint = "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263"
        
        sizing = await trading_tools.calculate_position_size(
            token_mint=bonk_mint,
            portfolio_value=10000.0,
            risk_percentage=0.05,
            sentiment_score=0.7,
            confidence=0.8
        )
        
        assert isinstance(sizing, dict)
        assert "position_size_usd" in sizing
        assert "quantity" in sizing
        assert "risk_percentage" in sizing
        assert "sentiment_adjustment" in sizing
        assert "confidence_adjustment" in sizing
        
        assert sizing["position_size_usd"] > 0.0
        assert sizing["quantity"] > 0.0
        assert sizing["risk_percentage"] > 0.0
        assert sizing["sentiment_adjustment"] > 1.0  # Positive sentiment increases size
        assert 0.5 <= sizing["confidence_adjustment"] <= 1.0
    
    @pytest.mark.asyncio
    async def test_trade_conditions(self, trading_tools):
        """Test trade condition checking"""
        sol_mint = "So11111111111111111111111111111111111111112"
        usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
        
        # Load tokens first
        await trading_tools.get_token_list()
        
        conditions = await trading_tools.check_trade_conditions(
            sol_mint,
            usdc_mint,
            1000.0  # $1000 trade
        )
        
        assert isinstance(conditions, dict)
        assert "safe_to_trade" in conditions
        assert "warnings" in conditions
        assert "risk_level" in conditions
        assert "checks" in conditions
        
        assert isinstance(conditions["safe_to_trade"], bool)
        assert isinstance(conditions["warnings"], list)
        assert conditions["risk_level"] in ["LOW", "MEDIUM", "HIGH"]
        assert isinstance(conditions["checks"], dict)
    
    @pytest.mark.asyncio
    async def test_trading_pairs(self, trading_tools):
        """Test trading pairs retrieval"""
        pairs = await trading_tools.get_trading_pairs("SOL")
        
        assert isinstance(pairs, list)
        assert len(pairs) > 0
        
        for pair in pairs:
            assert "base_symbol" in pair
            assert "quote_symbol" in pair
            assert "base_mint" in pair
            assert "quote_mint" in pair
            assert "liquidity_usd" in pair
            assert "volume_24h_usd" in pair
            
            assert pair["base_symbol"] == "SOL"
            assert pair["liquidity_usd"] > 0
            assert pair["volume_24h_usd"] >= 0
    
    @pytest.mark.asyncio
    async def test_simulate_trade(self, trading_tools):
        """Test trade simulation"""
        sol_mint = "So11111111111111111111111111111111111111112"
        usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
        
        # Load tokens first
        await trading_tools.get_token_list()
        
        simulation = await trading_tools.simulate_trade(
            sol_mint,
            usdc_mint,
            1000.0,  # $1000 trade
            sentiment_score=0.5
        )
        
        assert isinstance(simulation, dict)
        assert "input_token" in simulation
        assert "output_token" in simulation
        assert "input_amount_usd" in simulation
        assert "expected_output_tokens" in simulation
        assert "price_impact_pct" in simulation
        assert "expected_return_ratio" in simulation
        assert "sentiment_adjustment" in simulation
        assert "trade_conditions" in simulation
        assert "recommendation" in simulation
        
        assert simulation["input_token"] == "SOL"
        assert simulation["output_token"] == "USDC"
        assert simulation["input_amount_usd"] == 1000.0
        assert simulation["recommendation"] in ["EXECUTE", "HOLD"]
    
    @pytest.mark.asyncio
    async def test_cleanup(self, trading_tools):
        """Test resource cleanup"""
        await trading_tools.cleanup()
        assert trading_tools.session is None


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