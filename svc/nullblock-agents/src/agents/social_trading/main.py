"""
Main entry point for social trading agent
"""

import logging
import asyncio
import json
import signal
import sys
from datetime import datetime, timedelta
from typing import Dict, List, Any, Optional
from pathlib import Path
import argparse

from .social_monitor import SocialMonitorAgent
from .sentiment_analyzer import SentimentAnalyzer
from .risk_manager import RiskManager, RiskProfile


# Configure logging
def setup_logging(log_level: str = "INFO", log_file: str = None):
    """Setup logging configuration"""
    handlers = [logging.StreamHandler(sys.stdout)]
    
    if log_file:
        handlers.append(logging.FileHandler(log_file))
    
    logging.basicConfig(
        level=getattr(logging, log_level.upper()),
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
        handlers=handlers
    )


class SocialTradingAgent:
    """Main social trading agent orchestrator"""
    
    def __init__(self, config: Dict[str, Any]):
        self.config = config
        self.logger = logging.getLogger(__name__)
        
        # Initialize components
        self.social_monitor = SocialMonitorAgent(config.get("social_monitor", {}))
        self.sentiment_analyzer = SentimentAnalyzer(config.get("sentiment_analyzer", {}))
        
        # Setup risk profile
        risk_config = config.get("risk_profile", {})
        risk_profile = RiskProfile(**risk_config)
        self.risk_manager = RiskManager(risk_profile)
        
        # Agent state
        self.is_running = False
        self.monitoring_task = None
        self.trading_task = None
        
        # Configuration
        self.update_interval = config.get("update_interval", 60)  # 1 minute
        self.min_signals_for_decision = config.get("min_signals_for_decision", 3)
        self.portfolio_value = config.get("portfolio_value", 10000.0)
        
        # Tokens to monitor
        self.monitored_tokens = config.get("monitored_tokens", [
            "BONK", "WIF", "POPCAT", "MYRO", "SOL", "RAY"
        ])
        
        # Trading decisions log
        self.trading_decisions = []
        
        # Performance metrics
        self.metrics = {
            "session_start": datetime.now(),
            "signals_processed": 0,
            "trading_decisions_made": 0,
            "successful_signals": 0,
            "failed_signals": 0
        }
    
    async def start(self):
        """Start the social trading agent"""
        if self.is_running:
            self.logger.warning("Agent is already running")
            return
        
        self.is_running = True
        self.logger.info("Starting Social Trading Agent...")
        
        try:
            # Start social monitoring
            await self.social_monitor.start_monitoring()
            
            # Start main trading loop
            self.monitoring_task = asyncio.create_task(self._monitoring_loop())
            self.trading_task = asyncio.create_task(self._trading_loop())
            
            self.logger.info("Social Trading Agent started successfully")
            
            # Wait for tasks to complete
            await asyncio.gather(self.monitoring_task, self.trading_task)
            
        except Exception as e:
            self.logger.error(f"Failed to start agent: {e}")
            await self.stop()
    
    async def stop(self):
        """Stop the social trading agent"""
        if not self.is_running:
            return
        
        self.is_running = False
        self.logger.info("Stopping Social Trading Agent...")
        
        try:
            # Stop social monitoring
            await self.social_monitor.stop_monitoring()
            
            # Cancel tasks
            if self.monitoring_task and not self.monitoring_task.done():
                self.monitoring_task.cancel()
                try:
                    await self.monitoring_task
                except asyncio.CancelledError:
                    pass
            
            if self.trading_task and not self.trading_task.done():
                self.trading_task.cancel()
                try:
                    await self.trading_task
                except asyncio.CancelledError:
                    pass
            
            self.logger.info("Social Trading Agent stopped")
            
        except Exception as e:
            self.logger.error(f"Error stopping agent: {e}")
    
    async def _monitoring_loop(self):
        """Main monitoring loop for collecting social signals"""
        while self.is_running:
            try:
                # Get market summary
                summary = self.social_monitor.get_market_summary()
                
                if summary["total_signals"] > 0:
                    self.logger.info(f"Market Summary: {summary['total_signals']} signals, "
                                   f"{summary['unique_tokens']} tokens, "
                                   f"sentiment: {summary['overall_sentiment']:.2f}")
                    
                    self.metrics["signals_processed"] += summary["total_signals"]
                
                await asyncio.sleep(self.update_interval)
                
            except asyncio.CancelledError:
                break
            except Exception as e:
                self.logger.error(f"Error in monitoring loop: {e}")
                await asyncio.sleep(30)  # Wait before retrying
    
    async def _trading_loop(self):
        """Main trading decision loop"""
        while self.is_running:
            try:
                # Process each monitored token
                for token_symbol in self.monitored_tokens:
                    await self._process_token(token_symbol)
                
                # Wait before next iteration
                await asyncio.sleep(self.update_interval * 2)  # Check trading less frequently
                
            except asyncio.CancelledError:
                break
            except Exception as e:
                self.logger.error(f"Error in trading loop: {e}")
                await asyncio.sleep(60)  # Wait longer on error
    
    async def _process_token(self, token_symbol: str):
        """Process trading decision for a specific token"""
        try:
            # Get signals for this token
            signals = self.social_monitor.get_token_signals(token_symbol)
            
            if len(signals) < self.min_signals_for_decision:
                self.logger.debug(f"Not enough signals for {token_symbol}: {len(signals)}")
                return
            
            # Convert signals to format for sentiment analyzer
            signal_data = [
                {
                    "token_symbol": signal.token_symbol,
                    "sentiment_score": signal.sentiment_score,
                    "source": signal.source,
                    "author": signal.author,
                    "engagement_score": signal.engagement_score,
                    "timestamp": signal.timestamp,
                    "content": signal.content
                }
                for signal in signals
            ]
            
            # Analyze sentiment
            sentiment_metrics = self.sentiment_analyzer.analyze_sentiment_metrics(signal_data)
            
            # Generate trading signal
            trading_signal = self.sentiment_analyzer.generate_trading_signal(
                sentiment_metrics,
                current_price=self._get_mock_price(token_symbol)  # In production, get real price
            )
            
            # Skip if signal is HOLD
            if trading_signal.signal_type == "HOLD":
                self.logger.debug(f"{token_symbol}: HOLD signal, skipping")
                return
            
            # Risk analysis
            token_data = self._get_mock_token_data(token_symbol)  # In production, get real data
            risk_metrics = self.risk_manager.analyze_token_risk(
                token_symbol,
                token_data,
                signal_data
            )
            
            # Position sizing
            position_sizing = self.risk_manager.calculate_position_size(
                token_symbol=token_symbol,
                current_price=self._get_mock_price(token_symbol),
                portfolio_value=self.portfolio_value,
                sentiment_score=sentiment_metrics.overall_sentiment,
                confidence=sentiment_metrics.engagement_quality,
                volatility=sentiment_metrics.volatility_score,
                risk_metrics=risk_metrics
            )
            
            # Portfolio risk check
            portfolio_risk = self.risk_manager.analyze_portfolio_risk([
                {
                    "symbol": token_symbol,
                    "value_usd": position_sizing.final_size_usd,
                    "risk_score": risk_metrics.overall_risk_score,
                    "category": "meme" if token_data["market_cap_usd"] < 100_000_000 else "established",
                    "liquidity_usd": token_data["liquidity_usd"],
                    "market_cap_usd": token_data["market_cap_usd"]
                }
            ])
            
            # Final decision
            should_execute, reasons = self.risk_manager.should_execute_trade(
                position_sizing,
                risk_metrics,
                portfolio_risk
            )
            
            # Create trading decision record
            decision = {
                "timestamp": datetime.now().isoformat(),
                "token_symbol": token_symbol,
                "signal_type": trading_signal.signal_type,
                "signal_strength": trading_signal.strength,
                "signal_confidence": trading_signal.confidence,
                "sentiment_score": sentiment_metrics.overall_sentiment,
                "risk_category": risk_metrics.risk_category,
                "position_size_usd": position_sizing.final_size_usd,
                "position_percentage": position_sizing.position_as_portfolio_pct,
                "should_execute": should_execute,
                "reasons": reasons,
                "signals_count": len(signals),
                "price_target": trading_signal.price_target,
                "stop_loss": trading_signal.stop_loss
            }
            
            # Log decision
            self.logger.info(f"\n=== TRADING DECISION: {token_symbol} ===")
            self.logger.info(f"Signal: {trading_signal.signal_type} (strength: {trading_signal.strength:.2f})")
            self.logger.info(f"Sentiment: {sentiment_metrics.overall_sentiment:.2f}")
            self.logger.info(f"Risk: {risk_metrics.risk_category}")
            self.logger.info(f"Position: ${position_sizing.final_size_usd:.2f} ({position_sizing.position_as_portfolio_pct:.1f}%)")
            self.logger.info(f"Execute: {should_execute}")
            self.logger.info(f"Reasons: {', '.join(reasons)}")
            
            if trading_signal.price_target:
                self.logger.info(f"Target: ${trading_signal.price_target:.6f}")
            if trading_signal.stop_loss:
                self.logger.info(f"Stop Loss: ${trading_signal.stop_loss:.6f}")
            
            # Store decision
            self.trading_decisions.append(decision)
            self.metrics["trading_decisions_made"] += 1
            
            if should_execute:
                self.metrics["successful_signals"] += 1
                # In production, execute the trade here
                await self._execute_trade(decision)
            else:
                self.metrics["failed_signals"] += 1
            
        except Exception as e:
            self.logger.error(f"Error processing token {token_symbol}: {e}")
    
    async def _execute_trade(self, decision: Dict[str, Any]):
        """Execute a trading decision (mock implementation)"""
        try:
            self.logger.info(f"EXECUTING TRADE: {decision['signal_type']} {decision['token_symbol']}")
            self.logger.info(f"Position Size: ${decision['position_size_usd']:.2f}")
            
            # In production, this would:
            # 1. Connect to Jupiter/Solana
            # 2. Execute the swap
            # 3. Set stop loss and take profit orders
            # 4. Monitor the position
            
            # For now, just log the action
            self.logger.info("Trade executed successfully (MOCK)")
            
        except Exception as e:
            self.logger.error(f"Failed to execute trade: {e}")
    
    def _get_mock_price(self, token_symbol: str) -> float:
        """Get mock price for testing (replace with real price feed)"""
        mock_prices = {
            "SOL": 180.50,
            "BONK": 0.000025,
            "WIF": 3.45,
            "POPCAT": 0.85,
            "MYRO": 0.125,
            "RAY": 2.15
        }
        return mock_prices.get(token_symbol, 1.0)
    
    def _get_mock_token_data(self, token_symbol: str) -> Dict[str, Any]:
        """Get mock token data for testing (replace with real data)"""
        mock_data = {
            "SOL": {
                "market_cap_usd": 50_000_000_000,
                "liquidity_usd": 100_000_000,
                "volume_24h_usd": 500_000_000,
                "price_change_24h": 3.2,
                "holder_count": 50000
            },
            "BONK": {
                "market_cap_usd": 180_000_000,
                "liquidity_usd": 500_000,
                "volume_24h_usd": 2_000_000,
                "price_change_24h": 15.5,
                "holder_count": 15000
            },
            "WIF": {
                "market_cap_usd": 650_000_000,
                "liquidity_usd": 1_000_000,
                "volume_24h_usd": 5_000_000,
                "price_change_24h": -2.8,
                "holder_count": 25000
            }
        }
        
        # Default for unknown tokens
        default_data = {
            "market_cap_usd": 50_000_000,
            "liquidity_usd": 100_000,
            "volume_24h_usd": 500_000,
            "price_change_24h": 10.0,
            "holder_count": 5000
        }
        
        return mock_data.get(token_symbol, default_data)
    
    def get_status(self) -> Dict[str, Any]:
        """Get current agent status"""
        uptime = datetime.now() - self.metrics["session_start"]
        
        return {
            "is_running": self.is_running,
            "uptime_seconds": int(uptime.total_seconds()),
            "metrics": self.metrics,
            "monitored_tokens": self.monitored_tokens,
            "recent_decisions": self.trading_decisions[-5:],  # Last 5 decisions
            "portfolio_value": self.portfolio_value
        }
    
    def save_session_data(self, filename: str = None):
        """Save session data to file"""
        try:
            if not filename:
                timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
                filename = f"social_trading_session_{timestamp}.json"
            
            session_data = {
                "config": self.config,
                "metrics": self.metrics,
                "trading_decisions": self.trading_decisions,
                "status": self.get_status()
            }
            
            with open(filename, 'w') as f:
                json.dump(session_data, f, indent=2, default=str)
            
            self.logger.info(f"Session data saved to {filename}")
            
        except Exception as e:
            self.logger.error(f"Failed to save session data: {e}")


def load_config(config_file: str) -> Dict[str, Any]:
    """Load configuration from file"""
    try:
        with open(config_file, 'r') as f:
            return json.load(f)
    except FileNotFoundError:
        print(f"Config file {config_file} not found, using defaults")
        return {}
    except json.JSONDecodeError as e:
        print(f"Invalid JSON in config file: {e}")
        return {}


def create_default_config() -> Dict[str, Any]:
    """Create default configuration"""
    return {
        "social_monitor": {
            "twitter_bearer_token": None,
            "dextools_api_key": None,
            "update_interval": 60
        },
        "sentiment_analyzer": {},
        "risk_profile": {
            "risk_tolerance": "MEDIUM",
            "max_portfolio_risk": 0.05,
            "max_position_size": 0.10,
            "max_correlation_exposure": 0.30,
            "stop_loss_percentage": 0.15,
            "take_profit_percentage": 0.50
        },
        "update_interval": 60,
        "min_signals_for_decision": 3,
        "portfolio_value": 10000.0,
        "monitored_tokens": ["BONK", "WIF", "POPCAT", "MYRO", "SOL"]
    }


async def main():
    """Main entry point"""
    parser = argparse.ArgumentParser(description="Social Trading Agent")
    parser.add_argument("--config", default="config.json", help="Configuration file")
    parser.add_argument("--log-level", default="INFO", choices=["DEBUG", "INFO", "WARNING", "ERROR"])
    parser.add_argument("--log-file", help="Log file path")
    parser.add_argument("--save-session", action="store_true", help="Save session data on exit")
    
    args = parser.parse_args()
    
    # Setup logging
    setup_logging(args.log_level, args.log_file)
    logger = logging.getLogger(__name__)
    
    # Load configuration
    config = load_config(args.config)
    if not config:
        config = create_default_config()
        logger.info("Using default configuration")
    
    # Create and start agent
    agent = SocialTradingAgent(config)
    
    # Setup signal handlers for graceful shutdown
    def signal_handler(signum, frame):
        logger.info(f"Received signal {signum}, shutting down...")
        asyncio.create_task(agent.stop())
    
    signal.signal(signal.SIGINT, signal_handler)
    signal.signal(signal.SIGTERM, signal_handler)
    
    try:
        # Start the agent
        await agent.start()
        
    except KeyboardInterrupt:
        logger.info("Keyboard interrupt received")
    except Exception as e:
        logger.error(f"Agent crashed: {e}")
    finally:
        # Ensure clean shutdown
        await agent.stop()
        
        # Save session data if requested
        if args.save_session:
            agent.save_session_data()
        
        # Print final status
        status = agent.get_status()
        print(f"\nSession Summary:")
        print(f"Uptime: {status['uptime_seconds']} seconds")
        print(f"Signals Processed: {status['metrics']['signals_processed']}")
        print(f"Trading Decisions: {status['metrics']['trading_decisions_made']}")
        print(f"Successful Signals: {status['metrics']['successful_signals']}")


if __name__ == "__main__":
    asyncio.run(main())