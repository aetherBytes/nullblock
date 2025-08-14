"""
Execution agent for arbitrage trades with MEV protection
"""

import logging
import asyncio
from typing import Dict, List, Optional, Any, Tuple
from datetime import datetime, timedelta
from dataclasses import dataclass
from pydantic import BaseModel, Field
from enum import Enum
import uuid

from .strategy_agent import ArbitrageStrategy

logger = logging.getLogger(__name__)


class ExecutionStatus(Enum):
    """Execution status for arbitrage trades"""
    PENDING = "pending"
    PREPARING = "preparing"
    EXECUTING = "executing"
    COMPLETED = "completed"
    FAILED = "failed"
    CANCELLED = "cancelled"
    PARTIAL = "partial"


class TransactionStatus(Enum):
    """Individual transaction status"""
    PENDING = "pending"
    SUBMITTED = "submitted"
    CONFIRMED = "confirmed"
    FAILED = "failed"
    REVERTED = "reverted"


class ExecutionResult(BaseModel):
    """Result of arbitrage execution"""
    execution_id: str = Field(default_factory=lambda: str(uuid.uuid4()))
    strategy_id: str = Field(..., description="Associated strategy ID")
    status: ExecutionStatus = Field(..., description="Execution status")
    transactions: List[Dict[str, Any]] = Field(default_factory=list)
    actual_profit: Optional[float] = Field(None, description="Actual profit realized")
    gas_used: Optional[float] = Field(None, description="Total gas used")
    execution_time: Optional[float] = Field(None, description="Execution time in seconds")
    slippage_experienced: Optional[float] = Field(None, description="Actual slippage %")
    error_message: Optional[str] = Field(None, description="Error details if failed")
    mev_protection_used: bool = Field(default=False, description="Whether MEV protection was used")
    started_at: Optional[datetime] = Field(None)
    completed_at: Optional[datetime] = Field(None)
    
    class Config:
        use_enum_values = True


class Transaction(BaseModel):
    """Individual transaction in arbitrage execution"""
    tx_id: str = Field(default_factory=lambda: str(uuid.uuid4()))
    tx_hash: Optional[str] = Field(None, description="Blockchain transaction hash")
    tx_type: str = Field(..., description="Transaction type (buy/sell)")
    dex: str = Field(..., description="DEX name")
    token_pair: str = Field(..., description="Token pair")
    amount: float = Field(..., description="Trade amount")
    expected_price: float = Field(..., description="Expected price")
    actual_price: Optional[float] = Field(None, description="Actual execution price")
    gas_price: Optional[float] = Field(None, description="Gas price used")
    gas_limit: Optional[int] = Field(None, description="Gas limit")
    gas_used: Optional[int] = Field(None, description="Actual gas used")
    status: TransactionStatus = Field(default=TransactionStatus.PENDING)
    submitted_at: Optional[datetime] = Field(None)
    confirmed_at: Optional[datetime] = Field(None)
    error: Optional[str] = Field(None)
    
    class Config:
        use_enum_values = True


class ExecutionAgent:
    """Execution agent for arbitrage trades with MEV protection"""
    
    def __init__(self, mcp_client=None, flashbots_client=None):
        self.logger = logging.getLogger(__name__)
        self.mcp_client = mcp_client  # Nullblock MCP client for security
        self.flashbots_client = flashbots_client  # Flashbots client for MEV protection
        
        # Execution tracking
        self.active_executions: Dict[str, ExecutionResult] = {}
        self.execution_history: List[ExecutionResult] = []
        
        # Configuration
        self.max_concurrent_executions = 3
        self.default_gas_limit = 500000
        self.max_gas_price = 300  # gwei
        
        # Performance metrics
        self.execution_metrics = {
            "total_executions": 0,
            "successful_executions": 0,
            "total_profit": 0.0,
            "total_gas_spent": 0.0,
            "average_execution_time": 0.0
        }
    
    async def execute_strategy(
        self, 
        strategy: ArbitrageStrategy,
        user_context: Optional[Dict[str, Any]] = None
    ) -> ExecutionResult:
        """Execute arbitrage strategy with MEV protection"""
        
        if strategy.recommended_action not in ["execute", "execute_with_caution"]:
            return ExecutionResult(
                strategy_id=str(strategy.opportunity.timestamp),  # Mock strategy ID
                status=ExecutionStatus.CANCELLED,
                error_message=f"Strategy not recommended for execution: {strategy.recommended_action}"
            )
        
        # Check concurrent execution limit
        if len(self.active_executions) >= self.max_concurrent_executions:
            return ExecutionResult(
                strategy_id=str(strategy.opportunity.timestamp),
                status=ExecutionStatus.FAILED,
                error_message="Maximum concurrent executions reached"
            )
        
        execution_result = ExecutionResult(
            strategy_id=str(strategy.opportunity.timestamp),
            status=ExecutionStatus.PENDING
        )
        
        self.active_executions[execution_result.execution_id] = execution_result
        
        try:
            await self._execute_arbitrage(strategy, execution_result, user_context)
        except Exception as e:
            execution_result.status = ExecutionStatus.FAILED
            execution_result.error_message = str(e)
            self.logger.error(f"Execution failed: {e}")
        finally:
            execution_result.completed_at = datetime.now()
            if execution_result.started_at:
                execution_result.execution_time = (
                    execution_result.completed_at - execution_result.started_at
                ).total_seconds()
            
            # Move to history
            del self.active_executions[execution_result.execution_id]
            self.execution_history.append(execution_result)
            
            # Update metrics
            self._update_metrics(execution_result)
        
        return execution_result
    
    async def _execute_arbitrage(
        self, 
        strategy: ArbitrageStrategy, 
        result: ExecutionResult,
        user_context: Optional[Dict[str, Any]]
    ):
        """Internal arbitrage execution logic"""
        
        result.status = ExecutionStatus.PREPARING
        result.started_at = datetime.now()
        
        opportunity = strategy.opportunity
        execution_plan = strategy.execution_plan
        
        self.logger.info(f"Starting arbitrage execution for {opportunity.token_pair}")
        
        # Validate current market conditions
        if not await self._validate_market_conditions(strategy):
            raise Exception("Market conditions no longer favorable")
        
        # Prepare transactions
        buy_tx = await self._prepare_buy_transaction(strategy, user_context)
        sell_tx = await self._prepare_sell_transaction(strategy, user_context)
        
        result.transactions = [buy_tx.model_dump(), sell_tx.model_dump()]
        result.status = ExecutionStatus.EXECUTING
        
        # Execute with MEV protection if available
        if (execution_plan.get("protection", {}).get("use_flashbots", False) and 
            self.flashbots_client):
            
            await self._execute_with_flashbots(strategy, buy_tx, sell_tx, result)
        else:
            await self._execute_sequential(strategy, buy_tx, sell_tx, result)
        
        # Calculate final results
        await self._calculate_final_results(result)
        
        self.logger.info(
            f"Arbitrage execution completed: {result.status}, "
            f"profit: {result.actual_profit}"
        )
    
    async def _validate_market_conditions(self, strategy: ArbitrageStrategy) -> bool:
        """Validate that market conditions are still favorable"""
        # In MVP, just simulate validation
        # In production, check current prices vs strategy expectations
        
        opportunity = strategy.opportunity
        
        # Check if opportunity is too old
        time_since_analysis = (datetime.now() - opportunity.timestamp).total_seconds()
        if time_since_analysis > 60:  # 1 minute threshold
            self.logger.warning("Opportunity too old, may no longer be valid")
            return False
        
        # Simulate price movement check
        import random
        price_still_valid = random.random() > 0.1  # 90% chance price is still valid
        
        if not price_still_valid:
            self.logger.warning("Price movement detected, opportunity may be invalid")
            return False
        
        return True
    
    async def _prepare_buy_transaction(
        self, 
        strategy: ArbitrageStrategy,
        user_context: Optional[Dict[str, Any]]
    ) -> Transaction:
        """Prepare buy transaction"""
        
        opportunity = strategy.opportunity
        execution_plan = strategy.execution_plan
        
        buy_step = next(step for step in execution_plan["execution_steps"] if step["action"] == "buy")
        
        transaction = Transaction(
            tx_type="buy",
            dex=buy_step["dex"],
            token_pair=buy_step["token_pair"],
            amount=buy_step["amount"],
            expected_price=buy_step["expected_price"],
            gas_limit=self.default_gas_limit
        )
        
        # Set gas price based on market conditions
        transaction.gas_price = await self._estimate_gas_price()
        
        self.logger.info(f"Prepared buy transaction: {transaction.amount} {transaction.token_pair} on {transaction.dex}")
        
        return transaction
    
    async def _prepare_sell_transaction(
        self, 
        strategy: ArbitrageStrategy,
        user_context: Optional[Dict[str, Any]]
    ) -> Transaction:
        """Prepare sell transaction"""
        
        opportunity = strategy.opportunity
        execution_plan = strategy.execution_plan
        
        sell_step = next(step for step in execution_plan["execution_steps"] if step["action"] == "sell")
        
        transaction = Transaction(
            tx_type="sell",
            dex=sell_step["dex"],
            token_pair=sell_step["token_pair"],
            amount=sell_step["amount"],
            expected_price=sell_step["expected_price"],
            gas_limit=self.default_gas_limit
        )
        
        transaction.gas_price = await self._estimate_gas_price()
        
        self.logger.info(f"Prepared sell transaction: {transaction.amount} {transaction.token_pair} on {transaction.dex}")
        
        return transaction
    
    async def _estimate_gas_price(self) -> float:
        """Estimate appropriate gas price"""
        # Mock implementation for MVP
        # In production, query current gas prices from network
        import random
        base_gas_price = 30.0  # gwei
        variance = random.uniform(0.8, 1.5)  # 20% to 50% variance
        
        estimated_price = base_gas_price * variance
        return min(estimated_price, self.max_gas_price)
    
    async def _execute_with_flashbots(
        self, 
        strategy: ArbitrageStrategy,
        buy_tx: Transaction,
        sell_tx: Transaction,
        result: ExecutionResult
    ):
        """Execute arbitrage using Flashbots for MEV protection"""
        
        result.mev_protection_used = True
        self.logger.info("Executing arbitrage with Flashbots MEV protection")
        
        try:
            # Simulate Flashbots bundle execution
            await asyncio.sleep(1)  # Simulate network delay
            
            # Mark transactions as submitted
            buy_tx.status = TransactionStatus.SUBMITTED
            buy_tx.submitted_at = datetime.now()
            buy_tx.tx_hash = f"0x{'a' * 64}"  # Mock transaction hash
            
            sell_tx.status = TransactionStatus.SUBMITTED
            sell_tx.submitted_at = datetime.now()
            sell_tx.tx_hash = f"0x{'b' * 64}"
            
            # Simulate bundle confirmation
            await asyncio.sleep(2)  # Simulate block time
            
            # Simulate execution success/failure
            import random
            success_rate = 0.85  # 85% success rate with Flashbots
            
            if random.random() < success_rate:
                # Successful execution
                buy_tx.status = TransactionStatus.CONFIRMED
                buy_tx.confirmed_at = datetime.now()
                buy_tx.actual_price = buy_tx.expected_price * random.uniform(0.999, 1.001)  # Small slippage
                buy_tx.gas_used = int(buy_tx.gas_limit * random.uniform(0.8, 1.0))
                
                sell_tx.status = TransactionStatus.CONFIRMED
                sell_tx.confirmed_at = datetime.now()
                sell_tx.actual_price = sell_tx.expected_price * random.uniform(0.999, 1.001)
                sell_tx.gas_used = int(sell_tx.gas_limit * random.uniform(0.8, 1.0))
                
                result.status = ExecutionStatus.COMPLETED
            else:
                # Failed execution
                buy_tx.status = TransactionStatus.FAILED
                buy_tx.error = "Bundle not included in block"
                sell_tx.status = TransactionStatus.FAILED
                sell_tx.error = "Bundle not included in block"
                
                result.status = ExecutionStatus.FAILED
                result.error_message = "Flashbots bundle failed to execute"
        
        except Exception as e:
            result.status = ExecutionStatus.FAILED
            result.error_message = f"Flashbots execution error: {str(e)}"
            raise
    
    async def _execute_sequential(
        self, 
        strategy: ArbitrageStrategy,
        buy_tx: Transaction,
        sell_tx: Transaction,
        result: ExecutionResult
    ):
        """Execute arbitrage sequentially (higher MEV risk)"""
        
        self.logger.info("Executing arbitrage sequentially (public mempool)")
        
        try:
            # Execute buy transaction first
            await self._execute_single_transaction(buy_tx)
            
            # Check if buy was successful before proceeding
            if buy_tx.status != TransactionStatus.CONFIRMED:
                result.status = ExecutionStatus.FAILED
                result.error_message = f"Buy transaction failed: {buy_tx.error}"
                return
            
            # Execute sell transaction
            await self._execute_single_transaction(sell_tx)
            
            # Determine final status
            if sell_tx.status == TransactionStatus.CONFIRMED:
                result.status = ExecutionStatus.COMPLETED
            elif sell_tx.status == TransactionStatus.FAILED:
                result.status = ExecutionStatus.PARTIAL
                result.error_message = f"Sell transaction failed: {sell_tx.error}"
            
        except Exception as e:
            result.status = ExecutionStatus.FAILED
            result.error_message = f"Sequential execution error: {str(e)}"
            raise
    
    async def _execute_single_transaction(self, transaction: Transaction):
        """Execute a single transaction"""
        
        try:
            transaction.status = TransactionStatus.SUBMITTED
            transaction.submitted_at = datetime.now()
            transaction.tx_hash = f"0x{hash(str(transaction.tx_id))}"  # Mock hash
            
            # Simulate network delay
            await asyncio.sleep(1)
            
            # Simulate transaction confirmation
            import random
            success_rate = 0.95  # 95% success rate for individual transactions
            
            if random.random() < success_rate:
                transaction.status = TransactionStatus.CONFIRMED
                transaction.confirmed_at = datetime.now()
                
                # Simulate price execution with slippage
                slippage_factor = random.uniform(0.995, 1.005)  # ±0.5% slippage
                transaction.actual_price = transaction.expected_price * slippage_factor
                transaction.gas_used = int(transaction.gas_limit * random.uniform(0.7, 1.0))
                
                self.logger.info(f"Transaction confirmed: {transaction.tx_hash}")
            else:
                transaction.status = TransactionStatus.FAILED
                transaction.error = "Transaction reverted"
                self.logger.error(f"Transaction failed: {transaction.tx_hash}")
        
        except Exception as e:
            transaction.status = TransactionStatus.FAILED
            transaction.error = str(e)
            raise
    
    async def _calculate_final_results(self, result: ExecutionResult):
        """Calculate final execution results"""
        
        if result.status not in [ExecutionStatus.COMPLETED, ExecutionStatus.PARTIAL]:
            return
        
        total_gas_used = 0
        total_gas_cost = 0
        actual_profit = 0
        total_slippage = 0
        
        confirmed_transactions = [
            tx for tx in result.transactions 
            if tx.get("status") == TransactionStatus.CONFIRMED.value
        ]
        
        if not confirmed_transactions:
            return
        
        # Calculate costs and slippage
        for tx_data in confirmed_transactions:
            gas_used = tx_data.get("gas_used", 0)
            gas_price = tx_data.get("gas_price", 0)
            gas_cost = (gas_used * gas_price) / 1e9  # Convert to ETH
            gas_cost_usd = gas_cost * 2450  # Assume ETH price for USD conversion
            
            total_gas_used += gas_used
            total_gas_cost += gas_cost_usd
            
            # Calculate slippage
            expected_price = tx_data.get("expected_price", 0)
            actual_price = tx_data.get("actual_price", 0)
            if expected_price > 0:
                slippage = abs(actual_price - expected_price) / expected_price * 100
                total_slippage += slippage
        
        # Calculate profit for successful arbitrage
        if (len(confirmed_transactions) == 2 and 
            result.status == ExecutionStatus.COMPLETED):
            
            buy_tx = confirmed_transactions[0] if confirmed_transactions[0]["tx_type"] == "buy" else confirmed_transactions[1]
            sell_tx = confirmed_transactions[1] if confirmed_transactions[1]["tx_type"] == "sell" else confirmed_transactions[0]
            
            buy_cost = buy_tx["amount"] * buy_tx["actual_price"]
            sell_revenue = sell_tx["amount"] * sell_tx["actual_price"]
            
            actual_profit = sell_revenue - buy_cost - total_gas_cost
        
        # Update result
        result.actual_profit = actual_profit
        result.gas_used = total_gas_cost
        result.slippage_experienced = total_slippage / len(confirmed_transactions) if confirmed_transactions else 0
        
        self.logger.info(
            f"Final results calculated: profit=${actual_profit:.2f}, "
            f"gas=${total_gas_cost:.2f}, slippage={result.slippage_experienced:.3f}%"
        )
    
    def _update_metrics(self, result: ExecutionResult):
        """Update execution performance metrics"""
        
        self.execution_metrics["total_executions"] += 1
        
        if result.status == ExecutionStatus.COMPLETED:
            self.execution_metrics["successful_executions"] += 1
            
            if result.actual_profit:
                self.execution_metrics["total_profit"] += result.actual_profit
            
            if result.gas_used:
                self.execution_metrics["total_gas_spent"] += result.gas_used
            
            if result.execution_time:
                current_avg = self.execution_metrics["average_execution_time"]
                total_executions = self.execution_metrics["successful_executions"]
                
                new_avg = ((current_avg * (total_executions - 1)) + result.execution_time) / total_executions
                self.execution_metrics["average_execution_time"] = new_avg
    
    def get_execution_status(self, execution_id: str) -> Optional[ExecutionResult]:
        """Get status of specific execution"""
        
        # Check active executions
        if execution_id in self.active_executions:
            return self.active_executions[execution_id]
        
        # Check history
        for result in self.execution_history:
            if result.execution_id == execution_id:
                return result
        
        return None
    
    def get_performance_metrics(self) -> Dict[str, Any]:
        """Get execution performance metrics"""
        
        metrics = self.execution_metrics.copy()
        
        if metrics["total_executions"] > 0:
            metrics["success_rate"] = metrics["successful_executions"] / metrics["total_executions"]
        else:
            metrics["success_rate"] = 0.0
        
        if metrics["successful_executions"] > 0:
            metrics["average_profit_per_execution"] = metrics["total_profit"] / metrics["successful_executions"]
            metrics["average_gas_per_execution"] = metrics["total_gas_spent"] / metrics["successful_executions"]
        else:
            metrics["average_profit_per_execution"] = 0.0
            metrics["average_gas_per_execution"] = 0.0
        
        metrics["net_profit"] = metrics["total_profit"] - metrics["total_gas_spent"]
        
        return metrics
    
    def get_recent_executions(self, limit: int = 10) -> List[ExecutionResult]:
        """Get recent execution results"""
        
        all_executions = list(self.active_executions.values()) + self.execution_history
        all_executions.sort(key=lambda x: x.started_at or datetime.min, reverse=True)
        
        return all_executions[:limit]