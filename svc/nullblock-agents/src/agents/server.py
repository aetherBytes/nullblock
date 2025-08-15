"""
FastAPI server for Nullblock Agents
"""

import logging
from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
from typing import Dict, Any, Optional, List
import uvicorn

from .arbitrage.price_agent import PriceAgent

logger = logging.getLogger(__name__)

# Create FastAPI app
app = FastAPI(
    title="Nullblock Agents",
    description="Modular agentic army for Web3 automation",
    version="0.1.0"
)

# Add CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Initialize price agent
price_agent = PriceAgent()

# Health check model
class HealthResponse(BaseModel):
    status: str
    service: str
    version: str
    timestamp: str

# Arbitrage models
class ArbitrageOpportunityResponse(BaseModel):
    token_pair: str
    buy_dex: str
    sell_dex: str
    buy_price: float
    sell_price: float
    profit_percentage: float
    profit_amount: float
    trade_amount: float
    gas_cost: float
    net_profit: float
    confidence: float
    timestamp: str

class MarketSummaryResponse(BaseModel):
    pairs_monitored: int
    dexes_monitored: int
    last_update: str
    opportunities_found: int
    avg_profit: float
    best_opportunity: Optional[Dict[str, Any]]

@app.get("/health", response_model=HealthResponse)
async def health_check():
    """Health check endpoint"""
    import datetime
    return HealthResponse(
        status="healthy",
        service="nullblock-agents",
        version="0.1.0",
        timestamp=datetime.datetime.now().isoformat()
    )

@app.get("/")
async def root():
    """Root endpoint"""
    return {
        "service": "Nullblock Agents",
        "version": "0.1.0",
        "status": "running"
    }

@app.get("/arbitrage/opportunities", response_model=List[ArbitrageOpportunityResponse])
async def get_arbitrage_opportunities():
    """Get current arbitrage opportunities"""
    try:
        opportunities = price_agent.find_arbitrage_opportunities()
        
        # Convert to response format
        response_opportunities = []
        for opp in opportunities:
            response_opportunities.append(ArbitrageOpportunityResponse(
                token_pair=opp.token_pair,
                buy_dex=opp.buy_dex,
                sell_dex=opp.sell_dex,
                buy_price=opp.buy_price,
                sell_price=opp.sell_price,
                profit_percentage=opp.profit_percentage,
                profit_amount=opp.profit_amount,
                trade_amount=opp.trade_amount,
                gas_cost=opp.gas_cost,
                net_profit=opp.net_profit,
                confidence=opp.confidence,
                timestamp=opp.timestamp.isoformat()
            ))
        
        return response_opportunities
    except Exception as e:
        logger.error(f"Error getting arbitrage opportunities: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/arbitrage/summary", response_model=MarketSummaryResponse)
async def get_market_summary():
    """Get market summary"""
    try:
        summary = price_agent.get_market_summary()
        
        return MarketSummaryResponse(
            pairs_monitored=summary["pairs_monitored"],
            dexes_monitored=summary["dexes_monitored"],
            last_update=summary["last_update"].isoformat(),
            opportunities_found=summary["opportunities_found"],
            avg_profit=summary["avg_profit"],
            best_opportunity=summary["best_opportunity"]
        )
    except Exception as e:
        logger.error(f"Error getting market summary: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/arbitrage/execute")
async def execute_arbitrage(token_pair: str, trade_amount: float):
    """Execute arbitrage trade (simulation for MVP)"""
    try:
        # In MVP, this is a simulation
        # In production, this would integrate with execution agent
        
        return {
            "status": "simulated",
            "token_pair": token_pair,
            "trade_amount": trade_amount,
            "message": "Arbitrage execution simulated successfully"
        }
    except Exception as e:
        logger.error(f"Error executing arbitrage: {e}")
        raise HTTPException(status_code=500, detail=str(e))

def run_server(host: str = "0.0.0.0", port: int = 8002, debug: bool = False):
    """Run the agents server"""
    uvicorn.run(
        "agents.server:app",
        host=host,
        port=port,
        reload=debug,
        log_level="info"
    )

if __name__ == "__main__":
    run_server(debug=True)



