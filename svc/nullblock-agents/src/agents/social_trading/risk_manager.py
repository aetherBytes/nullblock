"""
Risk management system for social trading and meme coin trades
"""

import logging
import asyncio
import math
from typing import Dict, List, Optional, Any, Tuple, Union
from datetime import datetime, timedelta
from dataclasses import dataclass
from pydantic import BaseModel, Field
from decimal import Decimal
import numpy as np

logger = logging.getLogger(__name__)


class RiskProfile(BaseModel):
    """User risk profile configuration"""
    risk_tolerance: str = Field(..., description="LOW, MEDIUM, HIGH")
    max_portfolio_risk: float = Field(default=0.05, description="Maximum portfolio risk per trade (5%)")
    max_position_size: float = Field(default=0.10, description="Maximum position size as % of portfolio (10%)")
    max_correlation_exposure: float = Field(default=0.30, description="Maximum exposure to correlated assets (30%)")
    stop_loss_percentage: float = Field(default=0.15, description="Default stop loss percentage (15%)")
    take_profit_percentage: float = Field(default=0.50, description="Default take profit percentage (50%)")
    max_drawdown_limit: float = Field(default=0.20, description="Maximum portfolio drawdown before stopping (20%)")
    daily_loss_limit: float = Field(default=0.05, description="Daily loss limit (5%)")


class PositionSizing(BaseModel):
    """Position sizing calculation result"""
    recommended_size_usd: float
    recommended_size_tokens: float
    risk_amount_usd: float
    position_as_portfolio_pct: float
    confidence_adjusted_size: float
    sentiment_adjusted_size: float
    volatility_adjusted_size: float
    final_size_usd: float
    reasoning: List[str]


class RiskMetrics(BaseModel):
    """Risk metrics for a trading opportunity"""
    token_symbol: str
    market_cap_usd: Optional[float] = None
    liquidity_score: float = Field(..., ge=0.0, le=1.0)
    volatility_score: float = Field(..., ge=0.0, le=1.0)
    sentiment_risk: float = Field(..., ge=0.0, le=1.0)
    technical_risk: float = Field(..., ge=0.0, le=1.0)
    social_risk: float = Field(..., ge=0.0, le=1.0)
    overall_risk_score: float = Field(..., ge=0.0, le=1.0)
    risk_category: str = Field(..., description="LOW, MEDIUM, HIGH, EXTREME")
    max_recommended_allocation: float = Field(..., description="Maximum recommended portfolio allocation")


class PortfolioRisk(BaseModel):
    """Portfolio-level risk analysis"""
    total_value_usd: float
    total_risk_exposure: float
    daily_pnl: float
    drawdown_from_peak: float
    concentration_risk: float
    correlation_risk: float
    liquidity_risk: float
    positions_at_risk: List[str]
    risk_warnings: List[str]
    recommended_actions: List[str]


@dataclass
class StopLossOrder:
    """Stop loss order configuration"""
    token_symbol: str
    entry_price: float
    stop_price: float
    stop_percentage: float
    trailing: bool = False
    trail_distance: float = 0.05  # 5% trailing distance


@dataclass
class TakeProfitOrder:
    """Take profit order configuration"""
    token_symbol: str
    entry_price: float
    target_price: float
    target_percentage: float
    partial_exit: bool = False
    exit_percentage: float = 1.0  # 100% exit by default


class RiskManager:
    """Advanced risk management for social trading"""
    
    def __init__(self, risk_profile: RiskProfile):
        self.risk_profile = risk_profile
        self.logger = logging.getLogger(__name__)
        
        # Risk scoring weights
        self.risk_weights = {
            "market_cap": 0.25,
            "liquidity": 0.20,
            "volatility": 0.20,
            "sentiment": 0.15,
            "technical": 0.10,
            "social": 0.10
        }
        
        # Market cap risk tiers (USD)
        self.market_cap_tiers = {
            "micro": 1_000_000,      # < $1M = extreme risk
            "small": 10_000_000,     # < $10M = high risk
            "medium": 100_000_000,   # < $100M = medium risk
            "large": 1_000_000_000   # > $1B = low risk
        }
        
        # Meme coin specific risk factors
        self.meme_risk_factors = {
            "rug_pull_keywords": ["presale", "fair launch", "dev wallet", "locked liquidity"],
            "pump_dump_indicators": ["coordinated", "pump group", "signal group"],
            "scam_indicators": ["guaranteed returns", "passive income", "ponzi", "pyramid"]
        }
    
    def calculate_position_size(
        self,
        token_symbol: str,
        current_price: float,
        portfolio_value: float,
        sentiment_score: float,
        confidence: float,
        volatility: float,
        risk_metrics: Optional[RiskMetrics] = None
    ) -> PositionSizing:
        """Calculate optimal position size with multiple risk adjustments"""
        try:
            reasoning = []
            
            # Base position size from risk profile
            base_risk_amount = portfolio_value * self.risk_profile.max_portfolio_risk
            base_position_size = portfolio_value * self.risk_profile.max_position_size
            
            reasoning.append(f"Base risk: {self.risk_profile.max_portfolio_risk*100:.1f}% of portfolio")
            reasoning.append(f"Base position: {self.risk_profile.max_position_size*100:.1f}% of portfolio")
            
            # Start with base position size
            recommended_size = base_position_size
            
            # Sentiment adjustment (-50% to +100% based on sentiment)
            sentiment_multiplier = 0.5 + (sentiment_score + 1) * 0.75  # Maps -1,1 to 0.5,2.0
            sentiment_adjusted_size = recommended_size * sentiment_multiplier
            
            reasoning.append(f"Sentiment adjustment: {sentiment_multiplier:.2f}x (score: {sentiment_score:.2f})")
            
            # Confidence adjustment (50% to 100% based on confidence)
            confidence_multiplier = 0.5 + (confidence * 0.5)
            confidence_adjusted_size = sentiment_adjusted_size * confidence_multiplier
            
            reasoning.append(f"Confidence adjustment: {confidence_multiplier:.2f}x (confidence: {confidence:.2f})")
            
            # Volatility adjustment (reduce size for high volatility)
            volatility_multiplier = max(0.3, 1.0 - (volatility * 0.7))  # 30% to 100% based on volatility
            volatility_adjusted_size = confidence_adjusted_size * volatility_multiplier
            
            reasoning.append(f"Volatility adjustment: {volatility_multiplier:.2f}x (volatility: {volatility:.2f})")
            
            # Risk metrics adjustment
            if risk_metrics:
                risk_multiplier = max(0.2, 1.0 - risk_metrics.overall_risk_score)
                volatility_adjusted_size *= risk_multiplier
                reasoning.append(f"Risk metrics adjustment: {risk_multiplier:.2f}x (risk: {risk_metrics.overall_risk_score:.2f})")
            
            # Apply absolute limits
            final_size = min(
                volatility_adjusted_size,
                base_position_size,  # Never exceed max position size
                portfolio_value * 0.25  # Hard limit: 25% of portfolio
            )
            
            # Ensure minimum viable position
            min_position = portfolio_value * 0.001  # 0.1% minimum
            final_size = max(final_size, min_position)
            
            # Calculate token quantity
            tokens_quantity = final_size / current_price
            
            # Risk amount (for stop loss calculation)
            risk_amount = final_size * self.risk_profile.stop_loss_percentage
            
            reasoning.append(f"Final position size: ${final_size:.2f} ({final_size/portfolio_value*100:.2f}% of portfolio)")
            
            return PositionSizing(
                recommended_size_usd=base_position_size,
                recommended_size_tokens=base_position_size / current_price,
                risk_amount_usd=risk_amount,
                position_as_portfolio_pct=(final_size / portfolio_value) * 100,
                confidence_adjusted_size=confidence_adjusted_size,
                sentiment_adjusted_size=sentiment_adjusted_size,
                volatility_adjusted_size=volatility_adjusted_size,
                final_size_usd=final_size,
                reasoning=reasoning
            )
            
        except Exception as e:
            self.logger.error(f"Failed to calculate position size: {e}")
            return PositionSizing(
                recommended_size_usd=0.0,
                recommended_size_tokens=0.0,
                risk_amount_usd=0.0,
                position_as_portfolio_pct=0.0,
                confidence_adjusted_size=0.0,
                sentiment_adjusted_size=0.0,
                volatility_adjusted_size=0.0,
                final_size_usd=0.0,
                reasoning=[f"Error: {str(e)}"]
            )
    
    def analyze_token_risk(
        self,
        token_symbol: str,
        token_data: Dict[str, Any],
        social_signals: List[Dict[str, Any]] = None
    ) -> RiskMetrics:
        """Analyze comprehensive risk metrics for a token"""
        try:
            # Extract token data
            market_cap = token_data.get("market_cap_usd")
            liquidity_usd = token_data.get("liquidity_usd", 0)
            volume_24h = token_data.get("volume_24h_usd", 0)
            price_change_24h = token_data.get("price_change_24h", 0)
            holder_count = token_data.get("holder_count", 0)
            
            # Market cap risk
            market_cap_risk = self._calculate_market_cap_risk(market_cap)
            
            # Liquidity risk
            liquidity_risk = self._calculate_liquidity_risk(liquidity_usd, volume_24h)
            
            # Volatility risk
            volatility_risk = self._calculate_volatility_risk(price_change_24h)
            
            # Sentiment risk
            sentiment_risk = 0.5  # Default medium risk
            if social_signals:
                sentiment_risk = self._calculate_sentiment_risk(social_signals)
            
            # Technical risk (simplified)
            technical_risk = self._calculate_technical_risk(token_data)
            
            # Social risk (scam/rug indicators)
            social_risk = 0.3  # Default low-medium risk
            if social_signals:
                social_risk = self._calculate_social_risk(social_signals)
            
            # Calculate overall risk score
            overall_risk = (
                market_cap_risk * self.risk_weights["market_cap"] +
                liquidity_risk * self.risk_weights["liquidity"] +
                volatility_risk * self.risk_weights["volatility"] +
                sentiment_risk * self.risk_weights["sentiment"] +
                technical_risk * self.risk_weights["technical"] +
                social_risk * self.risk_weights["social"]
            )
            
            # Determine risk category
            if overall_risk <= 0.25:
                risk_category = "LOW"
                max_allocation = 0.15  # 15% max
            elif overall_risk <= 0.50:
                risk_category = "MEDIUM"
                max_allocation = 0.10  # 10% max
            elif overall_risk <= 0.75:
                risk_category = "HIGH"
                max_allocation = 0.05  # 5% max
            else:
                risk_category = "EXTREME"
                max_allocation = 0.02  # 2% max
            
            return RiskMetrics(
                token_symbol=token_symbol,
                market_cap_usd=market_cap,
                liquidity_score=1.0 - liquidity_risk,
                volatility_score=volatility_risk,
                sentiment_risk=sentiment_risk,
                technical_risk=technical_risk,
                social_risk=social_risk,
                overall_risk_score=overall_risk,
                risk_category=risk_category,
                max_recommended_allocation=max_allocation
            )
            
        except Exception as e:
            self.logger.error(f"Failed to analyze token risk for {token_symbol}: {e}")
            return RiskMetrics(
                token_symbol=token_symbol,
                liquidity_score=0.0,
                volatility_score=1.0,
                sentiment_risk=1.0,
                technical_risk=1.0,
                social_risk=1.0,
                overall_risk_score=1.0,
                risk_category="EXTREME",
                max_recommended_allocation=0.01
            )
    
    def _calculate_market_cap_risk(self, market_cap: Optional[float]) -> float:
        """Calculate risk based on market cap"""
        if not market_cap:
            return 1.0  # Maximum risk for unknown market cap
        
        if market_cap < self.market_cap_tiers["micro"]:
            return 0.9  # Extreme risk
        elif market_cap < self.market_cap_tiers["small"]:
            return 0.7  # High risk
        elif market_cap < self.market_cap_tiers["medium"]:
            return 0.5  # Medium risk
        elif market_cap < self.market_cap_tiers["large"]:
            return 0.3  # Low-medium risk
        else:
            return 0.1  # Low risk
    
    def _calculate_liquidity_risk(self, liquidity_usd: float, volume_24h: float) -> float:
        """Calculate risk based on liquidity and volume"""
        if liquidity_usd <= 0:
            return 1.0  # Maximum risk
        
        # Volume to liquidity ratio
        volume_ratio = volume_24h / liquidity_usd if liquidity_usd > 0 else 0
        
        # Low liquidity is high risk
        if liquidity_usd < 10_000:  # Less than $10k
            liquidity_risk = 0.9
        elif liquidity_usd < 50_000:  # Less than $50k
            liquidity_risk = 0.7
        elif liquidity_usd < 250_000:  # Less than $250k
            liquidity_risk = 0.5
        elif liquidity_usd < 1_000_000:  # Less than $1M
            liquidity_risk = 0.3
        else:
            liquidity_risk = 0.1
        
        # Very high volume ratio might indicate manipulation
        if volume_ratio > 10:  # Volume > 10x liquidity
            liquidity_risk = min(1.0, liquidity_risk + 0.3)
        
        return liquidity_risk
    
    def _calculate_volatility_risk(self, price_change_24h: float) -> float:
        """Calculate risk based on price volatility"""
        abs_change = abs(price_change_24h)
        
        if abs_change <= 5:  # ±5%
            return 0.1  # Low volatility
        elif abs_change <= 15:  # ±15%
            return 0.3  # Medium volatility
        elif abs_change <= 30:  # ±30%
            return 0.6  # High volatility
        elif abs_change <= 50:  # ±50%
            return 0.8  # Very high volatility
        else:
            return 1.0  # Extreme volatility
    
    def _calculate_sentiment_risk(self, social_signals: List[Dict[str, Any]]) -> float:
        """Calculate risk based on sentiment patterns"""
        if not social_signals:
            return 0.5
        
        sentiments = [s.get("sentiment_score", 0.0) for s in social_signals]
        
        # Extreme sentiment (both positive and negative) increases risk
        avg_sentiment = sum(sentiments) / len(sentiments)
        sentiment_volatility = np.std(sentiments) if len(sentiments) > 1 else 0.0
        
        # Risk increases with extreme sentiment and high volatility
        extreme_risk = min(0.4, abs(avg_sentiment) * 0.4)  # Max 0.4 from extremes
        volatility_risk = min(0.6, sentiment_volatility * 0.6)  # Max 0.6 from volatility
        
        return extreme_risk + volatility_risk
    
    def _calculate_technical_risk(self, token_data: Dict[str, Any]) -> float:
        """Calculate technical analysis risk"""
        # Simplified technical risk based on available data
        risk_factors = []
        
        # Concentration risk (few holders)
        holder_count = token_data.get("holder_count", 0)
        if holder_count < 100:
            risk_factors.append(0.4)
        elif holder_count < 1000:
            risk_factors.append(0.2)
        
        # New token risk
        created_date = token_data.get("created_date")
        if created_date:
            days_old = (datetime.now() - created_date).days
            if days_old < 7:
                risk_factors.append(0.5)  # Very new token
            elif days_old < 30:
                risk_factors.append(0.3)  # New token
        
        return min(1.0, sum(risk_factors))
    
    def _calculate_social_risk(self, social_signals: List[Dict[str, Any]]) -> float:
        """Calculate risk based on social indicators"""
        risk_score = 0.0
        total_signals = len(social_signals)
        
        if total_signals == 0:
            return 0.5
        
        scam_mentions = 0
        pump_mentions = 0
        
        for signal in social_signals:
            content = signal.get("content", "").lower()
            
            # Check for scam indicators
            for keyword in self.meme_risk_factors["scam_indicators"]:
                if keyword in content:
                    scam_mentions += 1
                    break
            
            # Check for pump indicators
            for keyword in self.meme_risk_factors["pump_dump_indicators"]:
                if keyword in content:
                    pump_mentions += 1
                    break
        
        # Calculate risk based on mention frequency
        scam_ratio = scam_mentions / total_signals
        pump_ratio = pump_mentions / total_signals
        
        risk_score = min(1.0, scam_ratio * 2.0 + pump_ratio * 1.5)
        
        return risk_score
    
    def create_stop_loss_order(
        self,
        token_symbol: str,
        entry_price: float,
        risk_amount: float,
        position_size: float,
        trailing: bool = False
    ) -> StopLossOrder:
        """Create stop loss order configuration"""
        try:
            # Calculate stop loss percentage to risk the specified amount
            stop_percentage = risk_amount / position_size
            stop_percentage = min(stop_percentage, self.risk_profile.stop_loss_percentage)
            
            stop_price = entry_price * (1 - stop_percentage)
            
            trail_distance = 0.05  # 5% trailing distance
            if trailing:
                # For meme coins, use wider trailing distance
                trail_distance = max(0.08, stop_percentage * 0.5)
            
            return StopLossOrder(
                token_symbol=token_symbol,
                entry_price=entry_price,
                stop_price=stop_price,
                stop_percentage=stop_percentage,
                trailing=trailing,
                trail_distance=trail_distance
            )
            
        except Exception as e:
            self.logger.error(f"Failed to create stop loss order: {e}")
            return StopLossOrder(
                token_symbol=token_symbol,
                entry_price=entry_price,
                stop_price=entry_price * 0.85,  # Default 15% stop
                stop_percentage=0.15,
                trailing=False
            )
    
    def create_take_profit_order(
        self,
        token_symbol: str,
        entry_price: float,
        sentiment_score: float,
        confidence: float,
        partial_exit: bool = True
    ) -> TakeProfitOrder:
        """Create take profit order configuration"""
        try:
            # Base take profit from risk profile
            base_target = self.risk_profile.take_profit_percentage
            
            # Adjust target based on sentiment and confidence
            sentiment_multiplier = 1.0 + max(0, sentiment_score) * 0.5  # Up to 50% higher for bullish sentiment
            confidence_multiplier = 0.7 + confidence * 0.3  # 70% to 100% based on confidence
            
            target_percentage = base_target * sentiment_multiplier * confidence_multiplier
            target_percentage = min(target_percentage, 2.0)  # Cap at 200% gain
            
            target_price = entry_price * (1 + target_percentage)
            
            # For meme coins, consider partial exits
            exit_percentage = 0.5 if partial_exit else 1.0  # Exit 50% or 100%
            
            return TakeProfitOrder(
                token_symbol=token_symbol,
                entry_price=entry_price,
                target_price=target_price,
                target_percentage=target_percentage,
                partial_exit=partial_exit,
                exit_percentage=exit_percentage
            )
            
        except Exception as e:
            self.logger.error(f"Failed to create take profit order: {e}")
            return TakeProfitOrder(
                token_symbol=token_symbol,
                entry_price=entry_price,
                target_price=entry_price * 1.5,  # Default 50% target
                target_percentage=0.5,
                partial_exit=partial_exit,
                exit_percentage=1.0
            )
    
    def analyze_portfolio_risk(
        self,
        portfolio_positions: List[Dict[str, Any]],
        historical_pnl: List[float] = None
    ) -> PortfolioRisk:
        """Analyze portfolio-level risk metrics"""
        try:
            if not portfolio_positions:
                return PortfolioRisk(
                    total_value_usd=0.0,
                    total_risk_exposure=0.0,
                    daily_pnl=0.0,
                    drawdown_from_peak=0.0,
                    concentration_risk=0.0,
                    correlation_risk=0.0,
                    liquidity_risk=0.0,
                    positions_at_risk=[],
                    risk_warnings=[],
                    recommended_actions=[]
                )
            
            # Calculate portfolio metrics
            total_value = sum(pos.get("value_usd", 0) for pos in portfolio_positions)
            
            # Risk exposure (simplified)
            risk_exposure = 0.0
            for pos in portfolio_positions:
                position_risk = pos.get("risk_score", 0.5)
                position_weight = pos.get("value_usd", 0) / total_value if total_value > 0 else 0
                risk_exposure += position_risk * position_weight
            
            # Daily P&L
            daily_pnl = 0.0
            if historical_pnl:
                daily_pnl = historical_pnl[-1] if historical_pnl else 0.0
            
            # Drawdown calculation
            drawdown = 0.0
            if historical_pnl and len(historical_pnl) > 1:
                peak = max(historical_pnl)
                current = historical_pnl[-1]
                drawdown = (peak - current) / peak if peak > 0 else 0.0
            
            # Concentration risk
            position_weights = [pos.get("value_usd", 0) / total_value for pos in portfolio_positions if total_value > 0]
            concentration_risk = max(position_weights) if position_weights else 0.0
            
            # Correlation risk (simplified - assume meme coins are correlated)
            meme_exposure = sum(
                pos.get("value_usd", 0) for pos in portfolio_positions 
                if pos.get("category") == "meme" or pos.get("market_cap_usd", 0) < 100_000_000
            )
            correlation_risk = meme_exposure / total_value if total_value > 0 else 0.0
            
            # Liquidity risk
            low_liquidity_exposure = sum(
                pos.get("value_usd", 0) for pos in portfolio_positions
                if pos.get("liquidity_usd", 0) < 100_000  # Less than $100k liquidity
            )
            liquidity_risk = low_liquidity_exposure / total_value if total_value > 0 else 0.0
            
            # Identify positions at risk
            positions_at_risk = []
            for pos in portfolio_positions:
                if (pos.get("risk_score", 0.5) > 0.7 or 
                    pos.get("drawdown", 0) > 0.2 or 
                    pos.get("liquidity_usd", 0) < 50_000):
                    positions_at_risk.append(pos.get("symbol", "UNKNOWN"))
            
            # Generate warnings and recommendations
            warnings = []
            recommendations = []
            
            if drawdown > self.risk_profile.max_drawdown_limit:
                warnings.append(f"Portfolio drawdown ({drawdown*100:.1f}%) exceeds limit")
                recommendations.append("Consider reducing position sizes")
            
            if concentration_risk > 0.25:
                warnings.append(f"High concentration risk ({concentration_risk*100:.1f}%)")
                recommendations.append("Diversify positions")
            
            if correlation_risk > self.risk_profile.max_correlation_exposure:
                warnings.append(f"High correlation risk ({correlation_risk*100:.1f}%)")
                recommendations.append("Reduce exposure to similar assets")
            
            if liquidity_risk > 0.20:
                warnings.append(f"High liquidity risk ({liquidity_risk*100:.1f}%)")
                recommendations.append("Increase allocation to liquid assets")
            
            if abs(daily_pnl / total_value) > self.risk_profile.daily_loss_limit:
                warnings.append("Daily loss limit exceeded")
                recommendations.append("Stop trading for today")
            
            return PortfolioRisk(
                total_value_usd=total_value,
                total_risk_exposure=risk_exposure,
                daily_pnl=daily_pnl,
                drawdown_from_peak=drawdown,
                concentration_risk=concentration_risk,
                correlation_risk=correlation_risk,
                liquidity_risk=liquidity_risk,
                positions_at_risk=positions_at_risk,
                risk_warnings=warnings,
                recommended_actions=recommendations
            )
            
        except Exception as e:
            self.logger.error(f"Failed to analyze portfolio risk: {e}")
            return PortfolioRisk(
                total_value_usd=0.0,
                total_risk_exposure=1.0,
                daily_pnl=0.0,
                drawdown_from_peak=0.0,
                concentration_risk=1.0,
                correlation_risk=1.0,
                liquidity_risk=1.0,
                positions_at_risk=[],
                risk_warnings=["Error analyzing portfolio risk"],
                recommended_actions=["Review risk management system"]
            )
    
    def should_execute_trade(
        self,
        position_sizing: PositionSizing,
        risk_metrics: RiskMetrics,
        portfolio_risk: PortfolioRisk,
        market_conditions: Dict[str, Any] = None
    ) -> Tuple[bool, List[str]]:
        """Determine if a trade should be executed based on risk analysis"""
        try:
            should_trade = True
            reasons = []
            
            # Check position size
            if position_sizing.final_size_usd <= 0:
                should_trade = False
                reasons.append("Position size is zero or negative")
            
            # Check risk category
            if risk_metrics.risk_category == "EXTREME":
                should_trade = False
                reasons.append("Token risk is extreme")
            
            # Check portfolio limits
            if portfolio_risk.drawdown_from_peak > self.risk_profile.max_drawdown_limit:
                should_trade = False
                reasons.append("Portfolio drawdown limit exceeded")
            
            if abs(portfolio_risk.daily_pnl / portfolio_risk.total_value_usd) > self.risk_profile.daily_loss_limit:
                should_trade = False
                reasons.append("Daily loss limit exceeded")
            
            # Check concentration
            new_allocation = position_sizing.position_as_portfolio_pct / 100
            if new_allocation > risk_metrics.max_recommended_allocation:
                should_trade = False
                reasons.append("Position would exceed recommended allocation")
            
            # Check correlation exposure
            if portfolio_risk.correlation_risk > self.risk_profile.max_correlation_exposure:
                should_trade = False
                reasons.append("Correlation exposure limit exceeded")
            
            # Market condition checks
            if market_conditions:
                market_volatility = market_conditions.get("volatility", 0.5)
                if market_volatility > 0.8 and risk_metrics.risk_category in ["HIGH", "EXTREME"]:
                    should_trade = False
                    reasons.append("High market volatility with high-risk token")
            
            # If trade is approved, add positive reasons
            if should_trade:
                reasons.append("All risk checks passed")
                reasons.append(f"Risk category: {risk_metrics.risk_category}")
                reasons.append(f"Position size: {position_sizing.position_as_portfolio_pct:.2f}% of portfolio")
            
            return should_trade, reasons
            
        except Exception as e:
            self.logger.error(f"Failed to evaluate trade execution: {e}")
            return False, [f"Error in risk evaluation: {str(e)}"]