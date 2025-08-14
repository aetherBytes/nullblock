"""
Price monitoring agent for arbitrage opportunities
"""

import logging
import asyncio
import aiohttp
import time
from typing import Dict, List, Optional, Any, Tuple
from datetime import datetime, timedelta
from dataclasses import dataclass
from pydantic import BaseModel, Field
import pandas as pd
import numpy as np

logger = logging.getLogger(__name__)


@dataclass
class PriceData:
    """Price data for a token pair on a DEX"""
    dex: str
    token_a: str
    token_b: str
    price: float
    liquidity: float
    volume_24h: float
    timestamp: datetime
    gas_cost: float = 0.0


class ArbitrageOpportunity(BaseModel):
    """Arbitrage opportunity data"""
    token_pair: str = Field(..., description="Token pair (e.g., ETH/USDC)")
    buy_dex: str = Field(..., description="DEX to buy from")
    sell_dex: str = Field(..., description="DEX to sell to")
    buy_price: float = Field(..., description="Buy price")
    sell_price: float = Field(..., description="Sell price")
    profit_percentage: float = Field(..., description="Profit percentage")
    profit_amount: float = Field(..., description="Estimated profit in USD")
    trade_amount: float = Field(..., description="Recommended trade amount")
    gas_cost: float = Field(..., description="Estimated gas cost")
    net_profit: float = Field(..., description="Net profit after gas")
    confidence: float = Field(..., description="Confidence score (0-1)")
    timestamp: datetime = Field(default_factory=datetime.now)


class DEXPriceProvider:
    """Base class for DEX price providers"""
    
    def __init__(self, dex_name: str):
        self.dex_name = dex_name
        self.logger = logging.getLogger(f"{__name__}.{dex_name}")
    
    async def get_price(self, token_a: str, token_b: str) -> Optional[PriceData]:
        """Get price for token pair"""
        raise NotImplementedError
    
    async def get_liquidity(self, token_a: str, token_b: str) -> float:
        """Get liquidity for token pair"""
        raise NotImplementedError


class UniswapPriceProvider(DEXPriceProvider):
    """Uniswap V3 price provider"""
    
    def __init__(self):
        super().__init__("uniswap")
        self.api_base = "https://api.thegraph.com/subgraphs/name/uniswap/uniswap-v3"
        self.session: Optional[aiohttp.ClientSession] = None
    
    async def _ensure_session(self):
        """Ensure HTTP session is active"""
        if not self.session:
            self.session = aiohttp.ClientSession()
    
    async def get_price(self, token_a: str, token_b: str) -> Optional[PriceData]:
        """Get Uniswap price for token pair"""
        try:
            await self._ensure_session()
            
            # Mock implementation for MVP
            # In production, query Uniswap subgraph
            mock_prices = {
                ("ETH", "USDC"): 2450.50,
                ("ETH", "USDT"): 2449.80,
                ("USDC", "USDT"): 0.9998,
                ("USDC", "DAI"): 1.0002
            }
            
            pair_key = (token_a, token_b)
            reverse_key = (token_b, token_a)
            
            if pair_key in mock_prices:
                price = mock_prices[pair_key]
            elif reverse_key in mock_prices:
                price = 1.0 / mock_prices[reverse_key]
            else:
                return None
            
            # Add some realistic variance
            variance = np.random.normal(0, 0.001)  # 0.1% variance
            price *= (1 + variance)
            
            return PriceData(
                dex="uniswap",
                token_a=token_a,
                token_b=token_b,
                price=price,
                liquidity=1000000.0,  # Mock liquidity
                volume_24h=5000000.0,  # Mock volume
                timestamp=datetime.now(),
                gas_cost=50.0  # Mock gas cost in USD
            )
            
        except Exception as e:
            self.logger.error(f"Failed to get Uniswap price for {token_a}/{token_b}: {e}")
            return None
    
    async def get_liquidity(self, token_a: str, token_b: str) -> float:
        """Get Uniswap liquidity for token pair"""
        # Mock implementation
        return 1000000.0


class SushiswapPriceProvider(DEXPriceProvider):
    """Sushiswap price provider"""
    
    def __init__(self):
        super().__init__("sushiswap")
    
    async def get_price(self, token_a: str, token_b: str) -> Optional[PriceData]:
        """Get Sushiswap price for token pair"""
        try:
            # Mock implementation for MVP
            mock_prices = {
                ("ETH", "USDC"): 2451.20,  # Slightly different from Uniswap
                ("ETH", "USDT"): 2450.30,
                ("USDC", "USDT"): 0.9999,
                ("USDC", "DAI"): 1.0001
            }
            
            pair_key = (token_a, token_b)
            reverse_key = (token_b, token_a)
            
            if pair_key in mock_prices:
                price = mock_prices[pair_key]
            elif reverse_key in mock_prices:
                price = 1.0 / mock_prices[reverse_key]
            else:
                return None
            
            # Add some realistic variance
            variance = np.random.normal(0, 0.001)
            price *= (1 + variance)
            
            return PriceData(
                dex="sushiswap",
                token_a=token_a,
                token_b=token_b,
                price=price,
                liquidity=800000.0,  # Slightly less liquidity than Uniswap
                volume_24h=3000000.0,
                timestamp=datetime.now(),
                gas_cost=55.0  # Slightly higher gas cost
            )
            
        except Exception as e:
            self.logger.error(f"Failed to get Sushiswap price for {token_a}/{token_b}: {e}")
            return None
    
    async def get_liquidity(self, token_a: str, token_b: str) -> float:
        """Get Sushiswap liquidity for token pair"""
        return 800000.0


class PriceAgent:
    """Price monitoring agent for arbitrage opportunities"""
    
    def __init__(self, update_interval: int = 5):
        self.update_interval = update_interval
        self.logger = logging.getLogger(__name__)
        
        # Initialize DEX providers
        self.providers = {
            "uniswap": UniswapPriceProvider(),
            "sushiswap": SushiswapPriceProvider()
        }
        
        # Price cache
        self.price_cache: Dict[str, Dict[str, PriceData]] = {}
        
        # Token pairs to monitor
        self.monitored_pairs = [
            ("ETH", "USDC"),
            ("ETH", "USDT"),
            ("USDC", "USDT"),
            ("USDC", "DAI")
        ]
        
        # Running state
        self.is_running = False
        self.monitoring_task: Optional[asyncio.Task] = None
    
    async def start_monitoring(self):
        """Start price monitoring"""
        if self.is_running:
            return
        
        self.is_running = True
        self.monitoring_task = asyncio.create_task(self._monitoring_loop())
        self.logger.info("Started price monitoring")
    
    async def stop_monitoring(self):
        """Stop price monitoring"""
        if not self.is_running:
            return
        
        self.is_running = False
        if self.monitoring_task:
            self.monitoring_task.cancel()
            try:
                await self.monitoring_task
            except asyncio.CancelledError:
                pass
        
        self.logger.info("Stopped price monitoring")
    
    async def _monitoring_loop(self):
        """Main price monitoring loop"""
        while self.is_running:
            try:
                await self._update_all_prices()
                await asyncio.sleep(self.update_interval)
            except asyncio.CancelledError:
                break
            except Exception as e:
                self.logger.error(f"Error in monitoring loop: {e}")
                await asyncio.sleep(1)
    
    async def _update_all_prices(self):
        """Update prices for all monitored pairs and DEXes"""
        tasks = []
        
        for dex_name, provider in self.providers.items():
            for token_a, token_b in self.monitored_pairs:
                task = asyncio.create_task(
                    self._update_price(dex_name, provider, token_a, token_b)
                )
                tasks.append(task)
        
        if tasks:
            await asyncio.gather(*tasks, return_exceptions=True)
    
    async def _update_price(
        self, 
        dex_name: str, 
        provider: DEXPriceProvider, 
        token_a: str, 
        token_b: str
    ):
        """Update price for specific pair and DEX"""
        try:
            price_data = await provider.get_price(token_a, token_b)
            if price_data:
                pair_key = f"{token_a}/{token_b}"
                if pair_key not in self.price_cache:
                    self.price_cache[pair_key] = {}
                self.price_cache[pair_key][dex_name] = price_data
                
        except Exception as e:
            self.logger.error(f"Failed to update price for {token_a}/{token_b} on {dex_name}: {e}")
    
    def get_current_prices(self, token_a: str, token_b: str) -> Dict[str, PriceData]:
        """Get current prices for token pair across all DEXes"""
        pair_key = f"{token_a}/{token_b}"
        return self.price_cache.get(pair_key, {})
    
    def find_arbitrage_opportunities(
        self, 
        min_profit_percentage: float = 0.5,
        max_trade_amount: float = 10000.0
    ) -> List[ArbitrageOpportunity]:
        """Find arbitrage opportunities across monitored pairs"""
        opportunities = []
        
        for pair_key, dex_prices in self.price_cache.items():
            if len(dex_prices) < 2:
                continue
            
            # Find best buy and sell prices
            best_buy_dex = None
            best_buy_price = float('inf')
            best_sell_dex = None
            best_sell_price = 0.0
            
            for dex_name, price_data in dex_prices.items():
                if price_data.price < best_buy_price:
                    best_buy_price = price_data.price
                    best_buy_dex = dex_name
                
                if price_data.price > best_sell_price:
                    best_sell_price = price_data.price
                    best_sell_dex = dex_name
            
            if best_buy_dex and best_sell_dex and best_buy_dex != best_sell_dex:
                # Calculate profit
                profit_percentage = ((best_sell_price - best_buy_price) / best_buy_price) * 100
                
                if profit_percentage >= min_profit_percentage:
                    # Calculate optimal trade amount
                    trade_amount = self._calculate_optimal_trade_amount(
                        pair_key, best_buy_dex, best_sell_dex, max_trade_amount
                    )
                    
                    # Calculate profit and costs
                    profit_amount = trade_amount * (best_sell_price - best_buy_price)
                    
                    # Estimate gas costs
                    buy_gas = dex_prices[best_buy_dex].gas_cost
                    sell_gas = dex_prices[best_sell_dex].gas_cost
                    total_gas_cost = buy_gas + sell_gas
                    
                    net_profit = profit_amount - total_gas_cost
                    
                    # Only include if net profit is positive
                    if net_profit > 0:
                        confidence = self._calculate_confidence(
                            dex_prices[best_buy_dex], 
                            dex_prices[best_sell_dex],
                            profit_percentage
                        )
                        
                        opportunity = ArbitrageOpportunity(
                            token_pair=pair_key,
                            buy_dex=best_buy_dex,
                            sell_dex=best_sell_dex,
                            buy_price=best_buy_price,
                            sell_price=best_sell_price,
                            profit_percentage=profit_percentage,
                            profit_amount=profit_amount,
                            trade_amount=trade_amount,
                            gas_cost=total_gas_cost,
                            net_profit=net_profit,
                            confidence=confidence
                        )
                        
                        opportunities.append(opportunity)
        
        # Sort by net profit descending
        opportunities.sort(key=lambda x: x.net_profit, reverse=True)
        
        return opportunities
    
    def _calculate_optimal_trade_amount(
        self, 
        pair_key: str, 
        buy_dex: str, 
        sell_dex: str, 
        max_amount: float
    ) -> float:
        """Calculate optimal trade amount based on liquidity"""
        dex_prices = self.price_cache.get(pair_key, {})
        
        if buy_dex not in dex_prices or sell_dex not in dex_prices:
            return min(max_amount, 1000.0)  # Default amount
        
        buy_liquidity = dex_prices[buy_dex].liquidity
        sell_liquidity = dex_prices[sell_dex].liquidity
        
        # Use 1% of minimum liquidity or max_amount, whichever is smaller
        max_liquidity_trade = min(buy_liquidity, sell_liquidity) * 0.01
        
        return min(max_amount, max_liquidity_trade, 5000.0)  # Cap at $5000 for safety
    
    def _calculate_confidence(
        self, 
        buy_price_data: PriceData, 
        sell_price_data: PriceData,
        profit_percentage: float
    ) -> float:
        """Calculate confidence score for arbitrage opportunity"""
        confidence = 0.5  # Base confidence
        
        # Higher liquidity increases confidence
        min_liquidity = min(buy_price_data.liquidity, sell_price_data.liquidity)
        if min_liquidity > 500000:
            confidence += 0.2
        elif min_liquidity > 100000:
            confidence += 0.1
        
        # Higher profit percentage increases confidence (up to a point)
        if profit_percentage > 2.0:
            confidence += 0.2
        elif profit_percentage > 1.0:
            confidence += 0.1
        
        # Fresher data increases confidence
        now = datetime.now()
        buy_age = (now - buy_price_data.timestamp).total_seconds()
        sell_age = (now - sell_price_data.timestamp).total_seconds()
        max_age = max(buy_age, sell_age)
        
        if max_age < 10:  # Very fresh data
            confidence += 0.2
        elif max_age < 30:  # Recent data
            confidence += 0.1
        elif max_age > 120:  # Stale data
            confidence -= 0.2
        
        return min(max(confidence, 0.0), 1.0)
    
    async def get_historical_prices(
        self, 
        token_a: str, 
        token_b: str, 
        hours: int = 24
    ) -> pd.DataFrame:
        """Get historical price data for analysis"""
        # Mock implementation for MVP
        # In production, this would query historical data
        
        timestamps = pd.date_range(
            end=datetime.now(),
            periods=hours * 12,  # 5-minute intervals
            freq='5T'
        )
        
        base_price = 2450.0 if token_a == "ETH" and token_b == "USDC" else 1.0
        
        # Generate realistic price movements
        returns = np.random.normal(0, 0.001, len(timestamps))  # 0.1% volatility
        prices = [base_price]
        
        for ret in returns[1:]:
            prices.append(prices[-1] * (1 + ret))
        
        df = pd.DataFrame({
            'timestamp': timestamps,
            'price': prices,
            'volume': np.random.uniform(1000, 10000, len(timestamps))
        })
        
        return df
    
    def get_market_summary(self) -> Dict[str, Any]:
        """Get market summary for all monitored pairs"""
        summary = {
            "pairs_monitored": len(self.monitored_pairs),
            "dexes_monitored": len(self.providers),
            "last_update": datetime.now(),
            "opportunities_found": 0,
            "avg_profit": 0.0,
            "best_opportunity": None
        }
        
        opportunities = self.find_arbitrage_opportunities()
        if opportunities:
            summary["opportunities_found"] = len(opportunities)
            summary["avg_profit"] = sum(op.profit_percentage for op in opportunities) / len(opportunities)
            summary["best_opportunity"] = opportunities[0].model_dump()
        
        return summary
    
    def start(self):
        """Start the price monitoring agent"""
        logger.info("Starting Nullblock Price Agent...")
        
        try:
            # Start the monitoring loop
            asyncio.run(self._monitor_prices())
        except KeyboardInterrupt:
            logger.info("Shutting down Price Agent...")
        except Exception as e:
            logger.error(f"Price Agent error: {e}")
    
    async def _monitor_prices(self):
        """Main monitoring loop"""
        logger.info("Price monitoring started")
        
        while True:
            try:
                # Find arbitrage opportunities
                opportunities = self.find_arbitrage_opportunities()
                
                if opportunities:
                    logger.info(f"Found {len(opportunities)} arbitrage opportunities")
                    for opp in opportunities[:3]:  # Log top 3
                        logger.info(f"Opportunity: {opp.token_pair} - {opp.profit_percentage:.2f}% profit")
                
                # Wait before next check
                await asyncio.sleep(30)  # Check every 30 seconds
                
            except Exception as e:
                logger.error(f"Error in price monitoring: {e}")
                await asyncio.sleep(60)  # Wait longer on error