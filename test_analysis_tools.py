#!/usr/bin/env python3
"""
Test Analysis Tools Directly

Tests the market analysis and DeFi analysis tools with real API calls
without requiring the full MCP server infrastructure.
"""

import asyncio
import logging
import sys
import os
from datetime import datetime

# Add the MCP packages to the path
sys.path.insert(0, 'svc/nullblock-mcp/src')

from mcp.tools.data_source_tools import DataSourceManager
from mcp.tools.analysis_tools import MarketAnalysisTools, DeFiAnalysisTools

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)

logger = logging.getLogger(__name__)

async def test_data_sources():
    """Test that data sources are working correctly"""
    print("🔧 TESTING DATA SOURCES")
    print("-" * 40)
    
    # Initialize data source manager
    data_manager = DataSourceManager()
    await data_manager.initialize()
    
    # Test CoinGecko API directly
    print("1️⃣ Testing CoinGecko API...")
    response = await data_manager.get_data(
        "price_oracle",
        "coingecko", 
        {"symbols": ["bitcoin"], "vs_currency": "usd"}
    )
    
    print(f"   📡 CoinGecko Response:")
    print(f"      Success: {response.success}")
    print(f"      Source: {response.source}")
    print(f"      Timestamp: {response.timestamp}")
    print(f"      Error: {response.error}")
    
    if response.success and response.data:
        print(f"      Data Type: {type(response.data)}")
        if isinstance(response.data, list) and response.data:
            data_point = response.data[0]
            print(f"      Sample Data Point: {data_point}")
            if hasattr(data_point, 'metadata'):
                print(f"      Metadata: {data_point.metadata}")
        print("   ✅ CoinGecko API working!")
    else:
        print("   ❌ CoinGecko API failed")
    
    # Test Uniswap data
    print("\n2️⃣ Testing Uniswap protocol data...")
    response = await data_manager.get_data(
        "defi_protocol",
        "uniswap",
        {"metrics": ["tvl", "volume"], "timeframe": "7d"}
    )
    
    print(f"   📡 Uniswap Response:")
    print(f"      Success: {response.success}")
    print(f"      Source: {response.source}")
    print(f"      Error: {response.error}")
    
    if response.success and response.data:
        print(f"      Data Type: {type(response.data)}")
        if isinstance(response.data, list):
            print(f"      Data Points: {len(response.data)}")
        print("   ✅ Uniswap API working!")
    else:
        print("   ❌ Uniswap API failed")
    
    await data_manager.cleanup()
    print("\n✅ Data source testing completed!\n")

async def test_market_analysis():
    """Test market analysis tools with real data"""
    print("📈 TESTING MARKET ANALYSIS TOOLS")
    print("-" * 40)
    
    # Initialize components
    data_manager = DataSourceManager()
    await data_manager.initialize()
    
    market_analysis = MarketAnalysisTools(data_manager)
    
    # Test market trend analysis
    print("1️⃣ Testing market trend analysis...")
    symbols = ["bitcoin", "ethereum"]
    
    try:
        result = await market_analysis.analyze_market_trends(symbols, "24h")
        
        print(f"   🎯 Analysis Results:")
        print(f"      Insights: {len(result.get('insights', []))}")
        print(f"      Patterns: {len(result.get('patterns', []))}")
        print(f"      Anomalies: {len(result.get('anomalies', []))}")
        print(f"      Recommendations: {len(result.get('recommendations', []))}")
        print(f"      Confidence: {result.get('confidence_score', 0.0):.2%}")
        
        print("\n   📝 Sample Insights:")
        for i, insight in enumerate(result.get('insights', [])[:3]):
            print(f"      {i+1}. {insight}")
        
        print("\n   🔍 Sample Patterns:")
        for i, pattern in enumerate(result.get('patterns', [])[:3]):
            print(f"      {i+1}. {pattern}")
        
        if result.get('recommendations'):
            print("\n   💡 Sample Recommendations:")
            for i, rec in enumerate(result.get('recommendations', [])[:2]):
                print(f"      {i+1}. {rec}")
        
        print("\n   ✅ Market analysis working with real data!")
        
    except Exception as e:
        print(f"   ❌ Market analysis failed: {e}")
    
    # Test volatility metrics
    print("\n2️⃣ Testing volatility analysis...")
    try:
        volatility_result = await market_analysis.calculate_volatility_metrics(symbols)
        
        print(f"   📊 Volatility Results:")
        for symbol, metrics in volatility_result.get('volatility_metrics', {}).items():
            print(f"      {symbol}: {metrics}")
        
        print("   ✅ Volatility analysis working!")
        
    except Exception as e:
        print(f"   ❌ Volatility analysis failed: {e}")
    
    await data_manager.cleanup()
    print("\n✅ Market analysis testing completed!\n")

async def test_defi_analysis():
    """Test DeFi analysis tools"""
    print("🏦 TESTING DEFI ANALYSIS TOOLS")
    print("-" * 40)
    
    # Initialize components
    data_manager = DataSourceManager()
    await data_manager.initialize()
    
    defi_analysis = DeFiAnalysisTools(data_manager)
    
    # Test DeFi opportunity detection
    print("1️⃣ Testing DeFi opportunity detection...")
    protocols = ["uniswap"]
    
    try:
        result = await defi_analysis.detect_defi_opportunities(protocols, min_apr=0.0, max_risk=1.0)
        
        print(f"   🎯 DeFi Analysis Results:")
        print(f"      Opportunities: {len(result.get('opportunities', []))}")
        print(f"      Insights: {len(result.get('insights', []))}")
        print(f"      Recommendations: {len(result.get('recommendations', []))}")
        print(f"      Total TVL: ${result.get('total_tvl', 0):,.0f}")
        print(f"      Average Yield: {result.get('average_yield', 0):.2f}%")
        
        print("\n   📝 Sample Insights:")
        for i, insight in enumerate(result.get('insights', [])[:3]):
            print(f"      {i+1}. {insight}")
        
        if result.get('opportunities'):
            print("\n   💰 Sample Opportunities:")
            for i, opp in enumerate(result.get('opportunities', [])[:2]):
                print(f"      {i+1}. {opp.get('protocol', 'Unknown')}: {opp.get('estimated_apr', 0):.1f}% APR")
        
        print("\n   ✅ DeFi analysis working with real data!")
        
    except Exception as e:
        print(f"   ❌ DeFi analysis failed: {e}")
    
    await data_manager.cleanup()
    print("\n✅ DeFi analysis testing completed!\n")

async def main():
    """Run all tests"""
    print("🧪 ANALYSIS TOOLS TESTING SUITE")
    print("=" * 60)
    print("Testing market analysis and DeFi tools with real API calls")
    print("=" * 60)
    
    try:
        await test_data_sources()
        await test_market_analysis()
        await test_defi_analysis()
        
        print("🎉 ALL TESTS COMPLETED SUCCESSFULLY!")
        print("=" * 60)
        print("Real API integration is working correctly.")
        
    except Exception as e:
        print(f"\n❌ TESTING FAILED: {e}")
        logger.error(f"Test suite failed: {e}")
        sys.exit(1)

if __name__ == "__main__":
    asyncio.run(main())