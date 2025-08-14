"""
Debug utilities and entry points for social trading development
"""

import logging
import asyncio
import json
import sys
from datetime import datetime, timedelta
from typing import Dict, List, Any, Optional
from pathlib import Path
import argparse

# Import our modules
from .social_monitor import SocialMonitorAgent, SocialSignal
from .sentiment_analyzer import SentimentAnalyzer, SentimentMetrics
from .risk_manager import RiskManager, RiskProfile
from ..arbitrage.price_agent import PriceAgent

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    handlers=[
        logging.StreamHandler(sys.stdout),
        logging.FileHandler('social_trading_debug.log')
    ]
)

logger = logging.getLogger(__name__)


class SocialTradingDebugger:
    """Debug utility for social trading system"""
    
    def __init__(self):
        self.logger = logging.getLogger(__name__)
        
        # Initialize components
        self.social_monitor = None
        self.sentiment_analyzer = None
        self.risk_manager = None
        
        # Debug data storage
        self.debug_data = {
            "session_start": datetime.now().isoformat(),
            "signals_collected": [],
            "sentiment_analyses": [],
            "trading_decisions": [],
            "risk_assessments": [],
            "errors": []
        }
    
    def setup_components(self, config: Dict[str, Any] = None):
        """Setup all social trading components"""
        try:
            # Default config
            default_config = {
                "twitter_bearer_token": None,
                "dextools_api_key": None,
                "update_interval": 30
            }
            
            if config:
                default_config.update(config)
            
            # Initialize components
            self.social_monitor = SocialMonitorAgent(default_config)
            self.sentiment_analyzer = SentimentAnalyzer()
            
            risk_profile = RiskProfile(
                risk_tolerance="MEDIUM",
                max_portfolio_risk=0.05,
                max_position_size=0.10
            )
            self.risk_manager = RiskManager(risk_profile)
            
            self.logger.info("All components initialized successfully")
            
        except Exception as e:
            self.logger.error(f"Failed to setup components: {e}")
            self.debug_data["errors"].append({
                "timestamp": datetime.now().isoformat(),
                "error": str(e),
                "component": "setup"
            })
    
    async def test_social_monitoring(self, duration_seconds: int = 60):
        """Test social media monitoring for a specified duration"""
        try:
            self.logger.info(f"Starting social monitoring test for {duration_seconds} seconds")
            
            if not self.social_monitor:
                self.setup_components()
            
            # Start monitoring
            await self.social_monitor.start_monitoring()
            
            start_time = datetime.now()
            end_time = start_time + timedelta(seconds=duration_seconds)
            
            while datetime.now() < end_time:
                # Check for new signals
                summary = self.social_monitor.get_market_summary()
                
                self.logger.info(f"Market Summary: {summary}")
                
                # Store debug data
                self.debug_data["signals_collected"].append({
                    "timestamp": datetime.now().isoformat(),
                    "summary": summary
                })
                
                await asyncio.sleep(10)  # Check every 10 seconds
            
            # Stop monitoring
            await self.social_monitor.stop_monitoring()
            
            self.logger.info("Social monitoring test completed")
            
        except Exception as e:
            self.logger.error(f"Social monitoring test failed: {e}")
            self.debug_data["errors"].append({
                "timestamp": datetime.now().isoformat(),
                "error": str(e),
                "component": "social_monitoring"
            })
    
    def test_sentiment_analysis(self, test_texts: List[str] = None):
        """Test sentiment analysis with sample texts"""
        try:
            if not self.sentiment_analyzer:
                self.setup_components()
            
            # Default test texts if none provided
            if not test_texts:
                test_texts = [
                    "BONK is going to the moon! 🚀 Best meme coin ever!",
                    "This token is a scam! Dump everything now!",
                    "Neutral update: BONK price is stable today",
                    "Amazing project with solid fundamentals and great team",
                    "Market manipulation everywhere. Be careful!",
                    "HODL diamond hands! 💎 Never selling!",
                    "Technical analysis shows bearish pattern forming",
                    "New listing on major exchange incoming!",
                    "Rug pull incoming. Dev wallet dumping",
                    "Community is amazing. Strong support levels"
                ]
            
            self.logger.info(f"Testing sentiment analysis with {len(test_texts)} texts")
            
            # Analyze individual texts
            individual_results = []
            for text in test_texts:
                signal = self.sentiment_analyzer.analyze_text_sentiment(text)
                individual_results.append({
                    "text": text,
                    "sentiment": signal.sentiment_score,
                    "confidence": signal.confidence,
                    "keywords": signal.keywords
                })
                
                self.logger.info(f"Text: '{text[:50]}...' -> Sentiment: {signal.sentiment_score:.2f}")
            
            # Batch analysis
            batch_analysis = self.sentiment_analyzer.analyze_sentiment_batch(test_texts)
            
            self.logger.info(f"Batch Analysis: Overall sentiment: {batch_analysis.overall_sentiment:.2f}")
            self.logger.info(f"Signals - Bullish: {batch_analysis.bullish_signals}, "
                           f"Bearish: {batch_analysis.bearish_signals}, "
                           f"Neutral: {batch_analysis.neutral_signals}")
            
            # Fear & Greed Index
            sentiments = [result["sentiment"] for result in individual_results]
            fear_greed = self.sentiment_analyzer.calculate_fear_greed_index(sentiments)
            
            self.logger.info(f"Fear & Greed Index: {fear_greed['index']} ({fear_greed['label']})")
            
            # Store debug data
            self.debug_data["sentiment_analyses"].append({
                "timestamp": datetime.now().isoformat(),
                "individual_results": individual_results,
                "batch_analysis": batch_analysis.model_dump(),
                "fear_greed_index": fear_greed
            })
            
        except Exception as e:
            self.logger.error(f"Sentiment analysis test failed: {e}")
            self.debug_data["errors"].append({
                "timestamp": datetime.now().isoformat(),
                "error": str(e),
                "component": "sentiment_analysis"
            })
    
    def test_risk_management(self, test_scenarios: List[Dict[str, Any]] = None):
        """Test risk management with various scenarios"""
        try:
            if not self.risk_manager:
                self.setup_components()
            
            # Default test scenarios
            if not test_scenarios:
                test_scenarios = [
                    {
                        "name": "Low Risk Scenario",
                        "token_symbol": "SOL",
                        "current_price": 180.50,
                        "portfolio_value": 10000.0,
                        "sentiment_score": 0.3,
                        "confidence": 0.8,
                        "volatility": 0.2,
                        "token_data": {
                            "market_cap_usd": 50_000_000_000,
                            "liquidity_usd": 10_000_000,
                            "volume_24h_usd": 500_000_000,
                            "price_change_24h": 3.2,
                            "holder_count": 50000
                        }
                    },
                    {
                        "name": "High Risk Meme Coin",
                        "token_symbol": "BONK",
                        "current_price": 0.000025,
                        "portfolio_value": 10000.0,
                        "sentiment_score": 0.8,
                        "confidence": 0.6,
                        "volatility": 0.8,
                        "token_data": {
                            "market_cap_usd": 50_000_000,
                            "liquidity_usd": 100_000,
                            "volume_24h_usd": 2_000_000,
                            "price_change_24h": 25.5,
                            "holder_count": 5000
                        }
                    },
                    {
                        "name": "Extreme Risk Scenario",
                        "token_symbol": "NEWCOIN",
                        "current_price": 0.001,
                        "portfolio_value": 10000.0,
                        "sentiment_score": 0.9,
                        "confidence": 0.3,
                        "volatility": 0.9,
                        "token_data": {
                            "market_cap_usd": 500_000,
                            "liquidity_usd": 10_000,
                            "volume_24h_usd": 50_000,
                            "price_change_24h": 150.0,
                            "holder_count": 100
                        }
                    }
                ]
            
            self.logger.info(f"Testing risk management with {len(test_scenarios)} scenarios")
            
            scenario_results = []
            
            for scenario in test_scenarios:
                self.logger.info(f"\n--- Testing: {scenario['name']} ---")
                
                # Analyze token risk
                risk_metrics = self.risk_manager.analyze_token_risk(
                    scenario["token_symbol"],
                    scenario["token_data"]
                )
                
                # Calculate position sizing
                position_sizing = self.risk_manager.calculate_position_size(
                    token_symbol=scenario["token_symbol"],
                    current_price=scenario["current_price"],
                    portfolio_value=scenario["portfolio_value"],
                    sentiment_score=scenario["sentiment_score"],
                    confidence=scenario["confidence"],
                    volatility=scenario["volatility"],
                    risk_metrics=risk_metrics
                )
                
                # Create stop loss and take profit orders
                stop_loss = self.risk_manager.create_stop_loss_order(
                    token_symbol=scenario["token_symbol"],
                    entry_price=scenario["current_price"],
                    risk_amount=position_sizing.risk_amount_usd,
                    position_size=position_sizing.final_size_usd
                )
                
                take_profit = self.risk_manager.create_take_profit_order(
                    token_symbol=scenario["token_symbol"],
                    entry_price=scenario["current_price"],
                    sentiment_score=scenario["sentiment_score"],
                    confidence=scenario["confidence"]
                )
                
                # Mock portfolio for trade decision
                portfolio_risk = self.risk_manager.analyze_portfolio_risk([
                    {
                        "symbol": scenario["token_symbol"],
                        "value_usd": position_sizing.final_size_usd,
                        "risk_score": risk_metrics.overall_risk_score,
                        "category": "test",
                        "liquidity_usd": scenario["token_data"]["liquidity_usd"],
                        "market_cap_usd": scenario["token_data"]["market_cap_usd"]
                    }
                ])
                
                # Final trade decision
                should_execute, reasons = self.risk_manager.should_execute_trade(
                    position_sizing,
                    risk_metrics,
                    portfolio_risk
                )
                
                result = {
                    "scenario": scenario["name"],
                    "risk_category": risk_metrics.risk_category,
                    "overall_risk_score": risk_metrics.overall_risk_score,
                    "position_size_usd": position_sizing.final_size_usd,
                    "position_percentage": position_sizing.position_as_portfolio_pct,
                    "stop_loss_price": stop_loss.stop_price,
                    "take_profit_price": take_profit.target_price,
                    "should_execute": should_execute,
                    "decision_reasons": reasons
                }
                
                scenario_results.append(result)
                
                self.logger.info(f"Risk Category: {risk_metrics.risk_category}")
                self.logger.info(f"Position Size: ${position_sizing.final_size_usd:.2f} "
                               f"({position_sizing.position_as_portfolio_pct:.2f}%)")
                self.logger.info(f"Stop Loss: ${stop_loss.stop_price:.6f}")
                self.logger.info(f"Take Profit: ${take_profit.target_price:.6f}")
                self.logger.info(f"Execute Trade: {should_execute}")
                self.logger.info(f"Reasons: {', '.join(reasons)}")
            
            # Store debug data
            self.debug_data["risk_assessments"].append({
                "timestamp": datetime.now().isoformat(),
                "scenario_results": scenario_results
            })
            
        except Exception as e:
            self.logger.error(f"Risk management test failed: {e}")
            self.debug_data["errors"].append({
                "timestamp": datetime.now().isoformat(),
                "error": str(e),
                "component": "risk_management"
            })
    
    async def test_end_to_end_pipeline(self, token_symbol: str = "BONK"):
        """Test complete end-to-end trading pipeline"""
        try:
            self.logger.info(f"Running end-to-end pipeline test for {token_symbol}")
            
            if not all([self.social_monitor, self.sentiment_analyzer, self.risk_manager]):
                self.setup_components()
            
            # 1. Collect social signals
            await self.social_monitor.start_monitoring()
            await asyncio.sleep(5)  # Let it collect some data
            
            signals = self.social_monitor.get_token_signals(token_symbol)
            if not signals:
                # Generate mock signals for testing
                signals = [
                    SocialSignal(
                        source="twitter",
                        token_symbol=token_symbol,
                        token_address="DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",
                        sentiment_score=0.7,
                        engagement_score=0.8,
                        content=f"{token_symbol} is trending! Great project 🚀",
                        author="crypto_bull",
                        timestamp=datetime.now()
                    ),
                    SocialSignal(
                        source="gmgn",
                        token_symbol=token_symbol,
                        token_address="DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",
                        sentiment_score=0.5,
                        engagement_score=0.9,
                        content=f"{token_symbol} +15% in 24h",
                        author="gmgn_api",
                        timestamp=datetime.now()
                    )
                ]
            
            await self.social_monitor.stop_monitoring()
            
            self.logger.info(f"Collected {len(signals)} signals for {token_symbol}")
            
            # 2. Analyze sentiment
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
            
            sentiment_metrics = self.sentiment_analyzer.analyze_sentiment_metrics(signal_data)
            self.logger.info(f"Sentiment Analysis: {sentiment_metrics.overall_sentiment:.2f} "
                           f"(confidence: {sentiment_metrics.engagement_quality:.2f})")
            
            # 3. Generate trading signal
            trading_signal = self.sentiment_analyzer.generate_trading_signal(
                sentiment_metrics,
                current_price=0.000025  # Mock price for BONK
            )
            
            self.logger.info(f"Trading Signal: {trading_signal.signal_type} "
                           f"(strength: {trading_signal.strength:.2f})")
            
            # 4. Risk analysis
            token_data = {
                "market_cap_usd": 180_000_000,
                "liquidity_usd": 500_000,
                "volume_24h_usd": 2_000_000,
                "price_change_24h": 12.5,
                "holder_count": 15000
            }
            
            risk_metrics = self.risk_manager.analyze_token_risk(
                token_symbol,
                token_data,
                signal_data
            )
            
            # 5. Position sizing
            position_sizing = self.risk_manager.calculate_position_size(
                token_symbol=token_symbol,
                current_price=0.000025,
                portfolio_value=10000.0,
                sentiment_score=sentiment_metrics.overall_sentiment,
                confidence=sentiment_metrics.engagement_quality,
                volatility=sentiment_metrics.volatility_score,
                risk_metrics=risk_metrics
            )
            
            # 6. Final decision
            portfolio_risk = self.risk_manager.analyze_portfolio_risk([
                {
                    "symbol": token_symbol,
                    "value_usd": position_sizing.final_size_usd,
                    "risk_score": risk_metrics.overall_risk_score,
                    "category": "meme",
                    "liquidity_usd": token_data["liquidity_usd"],
                    "market_cap_usd": token_data["market_cap_usd"]
                }
            ])
            
            should_execute, reasons = self.risk_manager.should_execute_trade(
                position_sizing,
                risk_metrics,
                portfolio_risk
            )
            
            # Log final decision
            self.logger.info(f"\n=== FINAL TRADING DECISION FOR {token_symbol} ===")
            self.logger.info(f"Sentiment: {sentiment_metrics.overall_sentiment:.2f}")
            self.logger.info(f"Trading Signal: {trading_signal.signal_type} (strength: {trading_signal.strength:.2f})")
            self.logger.info(f"Risk Category: {risk_metrics.risk_category}")
            self.logger.info(f"Position Size: ${position_sizing.final_size_usd:.2f}")
            self.logger.info(f"Should Execute: {should_execute}")
            self.logger.info(f"Reasons: {', '.join(reasons)}")
            
            # Store complete pipeline result
            pipeline_result = {
                "timestamp": datetime.now().isoformat(),
                "token_symbol": token_symbol,
                "signals_count": len(signals),
                "sentiment_metrics": sentiment_metrics.model_dump(),
                "trading_signal": {
                    "signal_type": trading_signal.signal_type,
                    "strength": trading_signal.strength,
                    "confidence": trading_signal.confidence,
                    "reasoning": trading_signal.reasoning
                },
                "risk_metrics": risk_metrics.model_dump(),
                "position_sizing": position_sizing.model_dump(),
                "should_execute": should_execute,
                "decision_reasons": reasons
            }
            
            self.debug_data["trading_decisions"].append(pipeline_result)
            
        except Exception as e:
            self.logger.error(f"End-to-end pipeline test failed: {e}")
            self.debug_data["errors"].append({
                "timestamp": datetime.now().isoformat(),
                "error": str(e),
                "component": "end_to_end_pipeline"
            })
    
    def save_debug_data(self, filename: str = None):
        """Save debug data to file"""
        try:
            if not filename:
                timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
                filename = f"social_trading_debug_{timestamp}.json"
            
            with open(filename, 'w') as f:
                json.dump(self.debug_data, f, indent=2, default=str)
            
            self.logger.info(f"Debug data saved to {filename}")
            
        except Exception as e:
            self.logger.error(f"Failed to save debug data: {e}")
    
    def print_summary(self):
        """Print debug session summary"""
        print("\n" + "="*60)
        print("SOCIAL TRADING DEBUG SESSION SUMMARY")
        print("="*60)
        print(f"Session Start: {self.debug_data['session_start']}")
        print(f"Signals Collected: {len(self.debug_data['signals_collected'])}")
        print(f"Sentiment Analyses: {len(self.debug_data['sentiment_analyses'])}")
        print(f"Trading Decisions: {len(self.debug_data['trading_decisions'])}")
        print(f"Risk Assessments: {len(self.debug_data['risk_assessments'])}")
        print(f"Errors Encountered: {len(self.debug_data['errors'])}")
        
        if self.debug_data['errors']:
            print("\nERRORS:")
            for error in self.debug_data['errors']:
                print(f"  [{error['timestamp']}] {error['component']}: {error['error']}")
        
        print("="*60)


async def main():
    """Main debug entry point"""
    parser = argparse.ArgumentParser(description="Social Trading Debug Tool")
    parser.add_argument("--test", choices=[
        "social", "sentiment", "risk", "pipeline", "all"
    ], default="all", help="Which test to run")
    parser.add_argument("--token", default="BONK", help="Token symbol to test")
    parser.add_argument("--duration", type=int, default=30, help="Test duration in seconds")
    parser.add_argument("--save", action="store_true", help="Save debug data to file")
    
    args = parser.parse_args()
    
    debugger = SocialTradingDebugger()
    
    try:
        if args.test in ["social", "all"]:
            print("Testing social monitoring...")
            await debugger.test_social_monitoring(args.duration)
        
        if args.test in ["sentiment", "all"]:
            print("Testing sentiment analysis...")
            debugger.test_sentiment_analysis()
        
        if args.test in ["risk", "all"]:
            print("Testing risk management...")
            debugger.test_risk_management()
        
        if args.test in ["pipeline", "all"]:
            print("Testing end-to-end pipeline...")
            await debugger.test_end_to_end_pipeline(args.token)
        
        # Print summary
        debugger.print_summary()
        
        # Save debug data if requested
        if args.save:
            debugger.save_debug_data()
    
    except KeyboardInterrupt:
        print("\nDebug session interrupted by user")
    except Exception as e:
        print(f"Debug session failed: {e}")
    finally:
        # Cleanup
        if debugger.social_monitor:
            await debugger.social_monitor.stop_monitoring()


if __name__ == "__main__":
    asyncio.run(main())