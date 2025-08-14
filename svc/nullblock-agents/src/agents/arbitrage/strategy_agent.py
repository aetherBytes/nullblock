"""
Strategy analysis agent for arbitrage trading
"""

import logging
import asyncio
from typing import Dict, List, Optional, Any, Tuple
from datetime import datetime, timedelta
from dataclasses import dataclass
from pydantic import BaseModel, Field
import numpy as np
import pandas as pd
from scipy import stats

from .price_agent import ArbitrageOpportunity, PriceAgent

logger = logging.getLogger(__name__)


class RiskMetrics(BaseModel):
    """Risk assessment metrics for arbitrage strategy"""
    price_volatility: float = Field(..., description="Price volatility (standard deviation)")
    liquidity_risk: float = Field(..., description="Liquidity risk score (0-1)")
    execution_risk: float = Field(..., description="Execution risk score (0-1)")
    market_impact: float = Field(..., description="Estimated market impact")
    slippage_risk: float = Field(..., description="Slippage risk score (0-1)")
    gas_risk: float = Field(..., description="Gas price volatility risk")
    overall_risk_score: float = Field(..., description="Overall risk score (0-1)")


class StrategyParameters(BaseModel):
    """Parameters for arbitrage strategy execution"""
    max_trade_size: float = Field(default=10000.0, description="Maximum trade size in USD")
    min_profit_threshold: float = Field(default=0.5, description="Minimum profit percentage")
    max_slippage: float = Field(default=0.5, description="Maximum acceptable slippage %")
    gas_price_limit: float = Field(default=100.0, description="Maximum gas price in gwei")
    execution_timeout: int = Field(default=300, description="Execution timeout in seconds")
    risk_tolerance: str = Field(default="medium", description="Risk tolerance level")
    use_flashbots: bool = Field(default=True, description="Use MEV protection")


class ArbitrageStrategy(BaseModel):
    """Complete arbitrage strategy with analysis"""
    opportunity: ArbitrageOpportunity = Field(..., description="Arbitrage opportunity")
    risk_metrics: RiskMetrics = Field(..., description="Risk assessment")
    execution_plan: Dict[str, Any] = Field(..., description="Execution plan")
    expected_outcome: Dict[str, float] = Field(..., description="Expected profit/loss")
    confidence_level: float = Field(..., description="Strategy confidence (0-1)")
    recommended_action: str = Field(..., description="Recommended action")
    created_at: datetime = Field(default_factory=datetime.now)


class StrategyAgent:
    """Strategy analysis agent for arbitrage opportunities"""
    
    def __init__(self, price_agent: PriceAgent):
        self.price_agent = price_agent
        self.logger = logging.getLogger(__name__)
        
        # Strategy parameters (can be customized per user)
        self.default_params = StrategyParameters()
        
        # Historical performance tracking
        self.strategy_history: List[ArbitrageStrategy] = []
        self.performance_metrics: Dict[str, float] = {}
    
    async def analyze_opportunity(
        self, 
        opportunity: ArbitrageOpportunity,
        user_params: Optional[StrategyParameters] = None
    ) -> ArbitrageStrategy:
        """Analyze arbitrage opportunity and create strategy"""
        
        params = user_params or self.default_params
        
        # Perform risk assessment
        risk_metrics = await self._assess_risk(opportunity, params)
        
        # Create execution plan
        execution_plan = await self._create_execution_plan(opportunity, params, risk_metrics)
        
        # Calculate expected outcome
        expected_outcome = self._calculate_expected_outcome(opportunity, risk_metrics, params)
        
        # Determine confidence level
        confidence_level = self._calculate_confidence(opportunity, risk_metrics, params)
        
        # Make recommendation
        recommended_action = self._make_recommendation(
            opportunity, risk_metrics, expected_outcome, confidence_level, params
        )
        
        strategy = ArbitrageStrategy(
            opportunity=opportunity,
            risk_metrics=risk_metrics,
            execution_plan=execution_plan,
            expected_outcome=expected_outcome,
            confidence_level=confidence_level,
            recommended_action=recommended_action
        )
        
        # Store in history
        self.strategy_history.append(strategy)
        
        self.logger.info(
            f"Analyzed strategy for {opportunity.token_pair}: "
            f"action={recommended_action}, confidence={confidence_level:.2f}"
        )
        
        return strategy
    
    async def _assess_risk(
        self, 
        opportunity: ArbitrageOpportunity, 
        params: StrategyParameters
    ) -> RiskMetrics:
        """Assess risks associated with the arbitrage opportunity"""
        
        # Get historical price data for volatility analysis
        token_a, token_b = opportunity.token_pair.split('/')
        try:
            historical_data = await self.price_agent.get_historical_prices(token_a, token_b, hours=24)
            price_volatility = historical_data['price'].pct_change().std() * np.sqrt(24)  # Annualized
        except Exception:
            price_volatility = 0.02  # Default 2% volatility
        
        # Assess liquidity risk
        liquidity_risk = self._assess_liquidity_risk(opportunity)
        
        # Assess execution risk
        execution_risk = self._assess_execution_risk(opportunity, params)
        
        # Estimate market impact
        market_impact = self._estimate_market_impact(opportunity)
        
        # Assess slippage risk
        slippage_risk = self._assess_slippage_risk(opportunity, params)
        
        # Assess gas price risk
        gas_risk = self._assess_gas_risk(opportunity)
        
        # Calculate overall risk score
        risk_weights = {
            'price_volatility': 0.25,
            'liquidity_risk': 0.20,
            'execution_risk': 0.20,
            'market_impact': 0.15,
            'slippage_risk': 0.15,
            'gas_risk': 0.05
        }
        
        overall_risk_score = (
            price_volatility * risk_weights['price_volatility'] +
            liquidity_risk * risk_weights['liquidity_risk'] +
            execution_risk * risk_weights['execution_risk'] +
            market_impact * risk_weights['market_impact'] +
            slippage_risk * risk_weights['slippage_risk'] +
            gas_risk * risk_weights['gas_risk']
        )
        
        return RiskMetrics(
            price_volatility=price_volatility,
            liquidity_risk=liquidity_risk,
            execution_risk=execution_risk,
            market_impact=market_impact,
            slippage_risk=slippage_risk,
            gas_risk=gas_risk,
            overall_risk_score=min(overall_risk_score, 1.0)
        )
    
    def _assess_liquidity_risk(self, opportunity: ArbitrageOpportunity) -> float:
        """Assess liquidity risk based on available liquidity"""
        # Get current prices with liquidity data
        token_a, token_b = opportunity.token_pair.split('/')
        prices = self.price_agent.get_current_prices(token_a, token_b)
        
        if not prices:
            return 0.8  # High risk if no price data
        
        buy_data = prices.get(opportunity.buy_dex)
        sell_data = prices.get(opportunity.sell_dex)
        
        if not buy_data or not sell_data:
            return 0.8
        
        # Calculate liquidity adequacy
        min_liquidity = min(buy_data.liquidity, sell_data.liquidity)
        liquidity_ratio = opportunity.trade_amount / min_liquidity
        
        # Risk increases exponentially with liquidity usage
        if liquidity_ratio < 0.01:  # Less than 1% of liquidity
            return 0.1
        elif liquidity_ratio < 0.05:  # Less than 5% of liquidity
            return 0.3
        elif liquidity_ratio < 0.1:  # Less than 10% of liquidity
            return 0.6
        else:
            return 0.9
    
    def _assess_execution_risk(
        self, 
        opportunity: ArbitrageOpportunity, 
        params: StrategyParameters
    ) -> float:
        """Assess execution risk factors"""
        risk_score = 0.2  # Base execution risk
        
        # DEX-specific risks
        dex_risks = {
            'uniswap': 0.1,
            'sushiswap': 0.15,
            'balancer': 0.2,
            'curve': 0.1
        }
        
        buy_risk = dex_risks.get(opportunity.buy_dex, 0.3)
        sell_risk = dex_risks.get(opportunity.sell_dex, 0.3)
        risk_score += (buy_risk + sell_risk) / 2
        
        # Time sensitivity risk
        time_since_update = (datetime.now() - opportunity.timestamp).total_seconds()
        if time_since_update > 30:  # Stale data
            risk_score += 0.3
        elif time_since_update > 10:
            risk_score += 0.1
        
        # Profit margin risk (lower margins are riskier)
        if opportunity.profit_percentage < 1.0:
            risk_score += 0.2
        elif opportunity.profit_percentage < 0.5:
            risk_score += 0.4
        
        return min(risk_score, 1.0)
    
    def _estimate_market_impact(self, opportunity: ArbitrageOpportunity) -> float:
        """Estimate market impact of the trade"""
        # Simple market impact model based on trade size vs liquidity
        token_a, token_b = opportunity.token_pair.split('/')
        prices = self.price_agent.get_current_prices(token_a, token_b)
        
        if not prices:
            return 0.5  # Default impact
        
        buy_data = prices.get(opportunity.buy_dex)
        sell_data = prices.get(opportunity.sell_dex)
        
        if not buy_data or not sell_data:
            return 0.5
        
        # Market impact scales with square root of trade size
        buy_impact = np.sqrt(opportunity.trade_amount / buy_data.liquidity) * 0.1
        sell_impact = np.sqrt(opportunity.trade_amount / sell_data.liquidity) * 0.1
        
        return min((buy_impact + sell_impact) / 2, 1.0)
    
    def _assess_slippage_risk(
        self, 
        opportunity: ArbitrageOpportunity, 
        params: StrategyParameters
    ) -> float:
        """Assess slippage risk"""
        # Slippage risk based on trade size and volatility
        base_slippage = 0.1  # 0.1% base slippage
        
        # Higher trade amounts increase slippage risk
        size_factor = min(opportunity.trade_amount / 10000, 2.0)  # Cap at 2x
        
        # Volatile pairs have higher slippage risk
        volatility_factor = 1.0  # Would calculate from historical data
        
        estimated_slippage = base_slippage * size_factor * volatility_factor
        
        # Risk score based on slippage vs tolerance
        risk_score = estimated_slippage / params.max_slippage
        
        return min(risk_score, 1.0)
    
    def _assess_gas_risk(self, opportunity: ArbitrageOpportunity) -> float:
        """Assess gas price volatility risk"""
        # Simplified gas risk assessment
        gas_percentage = (opportunity.gas_cost / opportunity.profit_amount) * 100
        
        if gas_percentage < 10:  # Gas cost less than 10% of profit
            return 0.1
        elif gas_percentage < 25:
            return 0.3
        elif gas_percentage < 50:
            return 0.6
        else:
            return 0.9
    
    async def _create_execution_plan(
        self, 
        opportunity: ArbitrageOpportunity, 
        params: StrategyParameters,
        risk_metrics: RiskMetrics
    ) -> Dict[str, Any]:
        """Create detailed execution plan"""
        
        plan = {
            "strategy_type": "atomic_arbitrage",
            "execution_steps": [
                {
                    "step": 1,
                    "action": "buy",
                    "dex": opportunity.buy_dex,
                    "token_pair": opportunity.token_pair,
                    "amount": opportunity.trade_amount,
                    "expected_price": opportunity.buy_price,
                    "max_slippage": params.max_slippage,
                    "estimated_gas": opportunity.gas_cost / 2
                },
                {
                    "step": 2,
                    "action": "sell",
                    "dex": opportunity.sell_dex,
                    "token_pair": opportunity.token_pair,
                    "amount": opportunity.trade_amount,
                    "expected_price": opportunity.sell_price,
                    "max_slippage": params.max_slippage,
                    "estimated_gas": opportunity.gas_cost / 2
                }
            ],
            "timing": {
                "max_execution_time": params.execution_timeout,
                "estimated_execution_time": 60,  # seconds
                "optimal_execution_window": 30  # seconds from now
            },
            "protection": {
                "use_flashbots": params.use_flashbots,
                "mev_protection": True,
                "front_running_protection": params.use_flashbots
            },
            "fallback_plan": {
                "cancel_if_profit_below": opportunity.profit_percentage * 0.5,
                "max_price_deviation": 1.0,  # 1% max price movement
                "emergency_exit": True
            },
            "risk_management": {
                "position_sizing": "fixed",
                "max_loss": opportunity.trade_amount * 0.02,  # 2% max loss
                "stop_loss": opportunity.net_profit * 0.1  # Stop if profit drops 90%
            }
        }
        
        # Adjust plan based on risk assessment
        if risk_metrics.overall_risk_score > 0.7:
            plan["execution_steps"][0]["max_slippage"] *= 0.8  # Tighter slippage
            plan["execution_steps"][1]["max_slippage"] *= 0.8
            plan["fallback_plan"]["cancel_if_profit_below"] = opportunity.profit_percentage * 0.7
        
        return plan
    
    def _calculate_expected_outcome(
        self, 
        opportunity: ArbitrageOpportunity,
        risk_metrics: RiskMetrics, 
        params: StrategyParameters
    ) -> Dict[str, float]:
        """Calculate expected profit/loss scenarios"""
        
        base_profit = opportunity.net_profit
        
        # Adjust for risks
        risk_adjustment = 1.0 - (risk_metrics.overall_risk_score * 0.3)
        slippage_cost = opportunity.trade_amount * (params.max_slippage / 100) * 2  # Buy + sell
        market_impact_cost = opportunity.trade_amount * risk_metrics.market_impact
        
        # Expected scenarios
        optimistic_profit = base_profit * 1.1  # 10% better than expected
        realistic_profit = (base_profit * risk_adjustment) - slippage_cost - market_impact_cost
        pessimistic_profit = realistic_profit * 0.5  # 50% of realistic
        
        # Probability-weighted expected value
        expected_value = (
            optimistic_profit * 0.2 +
            realistic_profit * 0.6 +
            pessimistic_profit * 0.2
        )
        
        return {
            "optimistic_profit": max(optimistic_profit, 0),
            "realistic_profit": max(realistic_profit, 0),
            "pessimistic_profit": pessimistic_profit,  # Can be negative
            "expected_value": expected_value,
            "profit_probability": 0.8 if realistic_profit > 0 else 0.3,
            "max_loss": min(pessimistic_profit, -opportunity.trade_amount * 0.02)
        }
    
    def _calculate_confidence(
        self, 
        opportunity: ArbitrageOpportunity,
        risk_metrics: RiskMetrics, 
        params: StrategyParameters
    ) -> float:
        """Calculate overall confidence in the strategy"""
        
        base_confidence = opportunity.confidence
        
        # Adjust based on risk assessment
        risk_penalty = risk_metrics.overall_risk_score * 0.4
        
        # Adjust based on profit margin
        profit_bonus = min(opportunity.profit_percentage / 2.0, 0.2)  # Max 0.2 bonus
        
        # Adjust based on historical performance
        historical_bonus = self._get_historical_performance_bonus()
        
        confidence = base_confidence - risk_penalty + profit_bonus + historical_bonus
        
        return max(min(confidence, 1.0), 0.0)
    
    def _get_historical_performance_bonus(self) -> float:
        """Get bonus/penalty based on historical strategy performance"""
        if len(self.strategy_history) < 5:
            return 0.0  # Need more history
        
        # Analyze last 10 strategies
        recent_strategies = self.strategy_history[-10:]
        success_rate = sum(1 for s in recent_strategies if s.recommended_action == "execute") / len(recent_strategies)
        
        if success_rate > 0.8:
            return 0.1
        elif success_rate > 0.6:
            return 0.05
        elif success_rate < 0.4:
            return -0.1
        else:
            return 0.0
    
    def _make_recommendation(
        self,
        opportunity: ArbitrageOpportunity,
        risk_metrics: RiskMetrics,
        expected_outcome: Dict[str, float],
        confidence_level: float,
        params: StrategyParameters
    ) -> str:
        """Make final recommendation on whether to execute the strategy"""
        
        # Check minimum thresholds
        if opportunity.profit_percentage < params.min_profit_threshold:
            return "reject_insufficient_profit"
        
        if risk_metrics.overall_risk_score > self._get_risk_tolerance_threshold(params.risk_tolerance):
            return "reject_high_risk"
        
        if expected_outcome["realistic_profit"] <= 0:
            return "reject_negative_expected_value"
        
        if confidence_level < 0.6:
            return "reject_low_confidence"
        
        # Check execution conditions
        if opportunity.gas_cost > opportunity.profit_amount * 0.8:  # Gas cost > 80% of profit
            return "reject_high_gas_cost"
        
        # Final recommendation based on expected value and confidence
        if (expected_outcome["expected_value"] > opportunity.trade_amount * 0.001 and  # 0.1% return
            confidence_level > 0.7):
            return "execute"
        elif confidence_level > 0.6:
            return "execute_with_caution"
        else:
            return "monitor"
    
    def _get_risk_tolerance_threshold(self, risk_tolerance: str) -> float:
        """Get risk score threshold based on user's risk tolerance"""
        thresholds = {
            "low": 0.3,
            "medium": 0.6,
            "high": 0.8
        }
        return thresholds.get(risk_tolerance, 0.6)
    
    def get_strategy_performance_metrics(self) -> Dict[str, Any]:
        """Get performance metrics for strategy decisions"""
        if not self.strategy_history:
            return {"message": "No strategy history available"}
        
        total_strategies = len(self.strategy_history)
        executed_strategies = sum(1 for s in self.strategy_history if s.recommended_action == "execute")
        
        recent_strategies = self.strategy_history[-20:] if len(self.strategy_history) >= 20 else self.strategy_history
        avg_confidence = sum(s.confidence_level for s in recent_strategies) / len(recent_strategies)
        avg_risk_score = sum(s.risk_metrics.overall_risk_score for s in recent_strategies) / len(recent_strategies)
        
        return {
            "total_strategies_analyzed": total_strategies,
            "strategies_recommended_for_execution": executed_strategies,
            "execution_rate": executed_strategies / total_strategies if total_strategies > 0 else 0,
            "average_confidence": avg_confidence,
            "average_risk_score": avg_risk_score,
            "last_analysis": self.strategy_history[-1].created_at if self.strategy_history else None
        }