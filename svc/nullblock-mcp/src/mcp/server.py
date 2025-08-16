"""
Main MCP server integrating wallet auth, context storage, Flashbots, and security
"""

import logging
import os
from typing import Dict, Any, Optional, List
from datetime import datetime
from fastapi import FastAPI, HTTPException, Depends, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.security import HTTPBearer, HTTPAuthorizationCredentials
from pydantic import BaseModel, Field
from web3 import Web3
import uvicorn

from .wallet.auth import WalletAuthenticator, WalletSession
from .context.storage import ContextManager, IPFSContextStorage, UserContext
from .flashbots.client import FlashbotsClient, MEVProtectionManager
from .security.prompt_protection import PromptProtectionManager, PromptAnalysis
from .tools.data_source_tools import DataSourceManager, DataSourceResponse

logger = logging.getLogger(__name__)


# Request/Response models
class AuthChallengeRequest(BaseModel):
    wallet_address: str = Field(..., description="Wallet address to authenticate")


class AuthChallengeResponse(BaseModel):
    message: str = Field(..., description="Message to sign")
    nonce: str = Field(..., description="Challenge nonce")
    expires_at: datetime = Field(..., description="Challenge expiration")


class AuthVerifyRequest(BaseModel):
    wallet_address: str = Field(..., description="Wallet address")
    signature: str = Field(..., description="Signed message")
    provider: str = Field(default="metamask", description="Wallet provider")


class AuthVerifyResponse(BaseModel):
    success: bool = Field(..., description="Authentication success")
    session_id: Optional[str] = Field(None, description="Session ID if successful")
    message: str = Field(..., description="Response message")


class ContextUpdateRequest(BaseModel):
    updates: Dict[str, Any] = Field(..., description="Context updates")


class TradingCommandRequest(BaseModel):
    command: str = Field(..., description="Trading command")
    parameters: Dict[str, Any] = Field(default_factory=dict, description="Command parameters")


class TradingCommandResponse(BaseModel):
    success: bool = Field(..., description="Command execution success")
    result: Any = Field(None, description="Command result")
    message: str = Field(..., description="Response message")
    protected: bool = Field(..., description="Whether MEV protection was used")


class DataSourceRequest(BaseModel):
    source_type: str = Field(..., description="Type of data source (price_oracle, defi_protocol, etc.)")
    source_name: str = Field(..., description="Specific source name (coingecko, uniswap, etc.)")
    parameters: Dict[str, Any] = Field(default_factory=dict, description="Query parameters")


class DataSourceListResponse(BaseModel):
    sources: Dict[str, List[str]] = Field(..., description="Available data sources by type")


class MCPServer:
    """Main Nullblock MCP Server"""
    
    def __init__(
        self,
        ethereum_rpc_url: str = "https://eth-mainnet.alchemyapi.io/v2/your-key",
        ipfs_api: str = "/ip4/127.0.0.1/tcp/5001",
        flashbots_private_key: Optional[str] = None,
        enable_mev_protection: bool = True
    ):
        self.app = FastAPI(
            title="Nullblock MCP Server",
            description="Model Context Protocol for secure Web3 agentic interactions",
            version="0.1.0"
        )
        
        # Initialize Web3
        self.web3 = Web3(Web3.HTTPProvider(ethereum_rpc_url))
        
        # Initialize components
        self.wallet_auth = WalletAuthenticator(self.web3)
        
        # Initialize context storage
        context_storage = IPFSContextStorage(ipfs_api)
        self.context_manager = ContextManager(context_storage)
        
        # Initialize Flashbots (if enabled and key provided)
        self.mev_protection = None
        if enable_mev_protection and flashbots_private_key:
            try:
                flashbots_client = FlashbotsClient(
                    self.web3,
                    flashbots_private_key
                )
                self.mev_protection = MEVProtectionManager(flashbots_client)
                logger.info("MEV protection enabled with Flashbots")
            except Exception as e:
                logger.error(f"Failed to initialize Flashbots: {e}")
                self.mev_protection = None
        
        # Initialize security
        self.prompt_protection = PromptProtectionManager(strict_mode=True)
        
        # Initialize data source manager
        self.data_source_manager = DataSourceManager()
        
        # Setup middleware
        self._setup_middleware()
        
        # Setup routes
        self._setup_routes()
        
        logger.info("Nullblock MCP Server initialized")
    
    def _setup_middleware(self):
        """Setup CORS and other middleware"""
        self.app.add_middleware(
            CORSMiddleware,
            allow_origins=["*"],  # Configure appropriately for production
            allow_credentials=True,
            allow_methods=["*"],
            allow_headers=["*"],
        )
    
    def _setup_routes(self):
        """Setup API routes"""
        
        @self.app.get("/health")
        async def health_check():
            """Health check endpoint"""
            return {
                "status": "healthy",
                "timestamp": datetime.now().isoformat(),
                "services": {
                    "web3": self.web3.is_connected(),
                    "mev_protection": self.mev_protection is not None,
                    "context_storage": True,
                    "security": True
                }
            }
        
        @self.app.post("/auth/challenge", response_model=AuthChallengeResponse)
        async def create_auth_challenge(request: AuthChallengeRequest):
            """Create authentication challenge for wallet"""
            try:
                challenge = self.wallet_auth.create_challenge(request.wallet_address)
                return AuthChallengeResponse(
                    message=challenge.message,
                    nonce=challenge.nonce,
                    expires_at=challenge.expires_at
                )
            except Exception as e:
                logger.error(f"Failed to create challenge: {e}")
                raise HTTPException(status_code=500, detail="Failed to create challenge")
        
        @self.app.post("/auth/verify", response_model=AuthVerifyResponse)
        async def verify_auth_challenge(request: AuthVerifyRequest):
            """Verify authentication challenge"""
            try:
                session = await self.wallet_auth.verify_challenge(
                    request.wallet_address,
                    request.signature,
                    request.provider
                )
                
                if session:
                    return AuthVerifyResponse(
                        success=True,
                        session_id=session.session_id,
                        message="Authentication successful"
                    )
                else:
                    return AuthVerifyResponse(
                        success=False,
                        message="Authentication failed"
                    )
                    
            except Exception as e:
                logger.error(f"Failed to verify challenge: {e}")
                raise HTTPException(status_code=500, detail="Verification failed")
        
        @self.app.get("/context")
        async def get_user_context(session: WalletSession = Depends(self._get_session)):
            """Get user context"""
            try:
                context = await self.context_manager.get_user_context(session.address)
                return context.model_dump()
            except Exception as e:
                logger.error(f"Failed to get context: {e}")
                raise HTTPException(status_code=500, detail="Failed to retrieve context")
        
        @self.app.post("/context/update")
        async def update_user_context(
            request: ContextUpdateRequest,
            session: WalletSession = Depends(self._get_session)
        ):
            """Update user context"""
            try:
                # Validate input for security
                analysis = self.prompt_protection.validate_input(
                    str(request.updates),
                    session.address
                )
                
                if analysis.blocked:
                    raise HTTPException(
                        status_code=403,
                        detail=f"Input blocked: {analysis.detected_patterns}"
                    )
                
                success = await self.context_manager.update_trading_preferences(
                    session.address,
                    request.updates
                )
                
                return {"success": success, "message": "Context updated"}
                
            except HTTPException:
                raise
            except Exception as e:
                logger.error(f"Failed to update context: {e}")
                raise HTTPException(status_code=500, detail="Failed to update context")
        
        @self.app.post("/trading/command", response_model=TradingCommandResponse)
        async def execute_trading_command(
            request: TradingCommandRequest,
            session: WalletSession = Depends(self._get_session)
        ):
            """Execute trading command with security validation"""
            try:
                # Security validation
                analysis = self.prompt_protection.validate_input(
                    f"{request.command} {str(request.parameters)}",
                    session.address
                )
                
                if analysis.blocked:
                    return TradingCommandResponse(
                        success=False,
                        message=f"Command blocked for security: {analysis.detected_patterns}",
                        protected=False
                    )
                
                # Validate trading command
                is_safe, validation_msg = self.prompt_protection.is_safe_trading_command(
                    request.command,
                    request.parameters
                )
                
                if not is_safe:
                    return TradingCommandResponse(
                        success=False,
                        message=f"Invalid command: {validation_msg}",
                        protected=False
                    )
                
                # Get user context for preferences
                context = await self.context_manager.get_user_context(session.address)
                
                # Execute command based on type
                result = await self._execute_command(
                    request.command,
                    request.parameters,
                    context,
                    session
                )
                
                return TradingCommandResponse(
                    success=True,
                    result=result,
                    message="Command executed successfully",
                    protected=self.mev_protection is not None
                )
                
            except HTTPException:
                raise
            except Exception as e:
                logger.error(f"Failed to execute command: {e}")
                return TradingCommandResponse(
                    success=False,
                    message=f"Command execution failed: {str(e)}",
                    protected=False
                )
        
        @self.app.get("/wallet/balance")
        async def get_wallet_balance(session: WalletSession = Depends(self._get_session)):
            """Get wallet balance"""
            try:
                balance = await self.wallet_auth.get_wallet_balance(session.session_id)
                return {"address": session.address, "balance": balance}
            except Exception as e:
                logger.error(f"Failed to get balance: {e}")
                raise HTTPException(status_code=500, detail="Failed to get balance")
        
        # Data Source Endpoints
        @self.app.get("/mcp/data-sources", response_model=DataSourceListResponse)
        async def get_data_sources():
            """Get list of available data sources"""
            try:
                sources = self.data_source_manager.get_available_sources()
                return DataSourceListResponse(sources=sources)
            except Exception as e:
                logger.error(f"Failed to get data sources: {e}")
                raise HTTPException(status_code=500, detail="Failed to get data sources")
        
        @self.app.post("/mcp/data/{source_type}/{source_name}")
        async def get_data_source_data(
            source_type: str,
            source_name: str,
            request: DataSourceRequest = None
        ):
            """Get data from specific data source"""
            try:
                # Use request body parameters if provided, otherwise use path parameters
                parameters = request.parameters if request else {}
                
                response = await self.data_source_manager.get_data(
                    source_type, source_name, parameters
                )
                
                if not response.success:
                    raise HTTPException(
                        status_code=400 if not response.rate_limited else 429,
                        detail=response.error or "Failed to get data"
                    )
                
                return {
                    "success": response.success,
                    "data": response.data,
                    "source": response.source,
                    "timestamp": response.timestamp.isoformat(),
                    "cached": response.cached
                }
            except HTTPException:
                raise
            except Exception as e:
                logger.error(f"Failed to get data from {source_type}/{source_name}: {e}")
                raise HTTPException(status_code=500, detail="Internal server error")
        
        @self.app.get("/mcp/data/{source_type}/{source_name}")
        async def get_data_source_data_get(
            source_type: str,
            source_name: str,
            symbols: str = None,
            timeframe: str = "24h",
            vs_currency: str = "usd",
            metrics: str = None,
            address: str = None,
            keywords: str = None
        ):
            """Get data from specific data source via GET (for simple queries)"""
            try:
                # Build parameters from query params
                parameters = {"timeframe": timeframe}
                
                if symbols:
                    parameters["symbols"] = symbols.split(",")
                if vs_currency:
                    parameters["vs_currency"] = vs_currency
                if metrics:
                    parameters["metrics"] = metrics.split(",")
                if address:
                    parameters["address"] = address
                if keywords:
                    parameters["keywords"] = keywords.split(",")
                
                response = await self.data_source_manager.get_data(
                    source_type, source_name, parameters
                )
                
                if not response.success:
                    raise HTTPException(
                        status_code=400 if not response.rate_limited else 429,
                        detail=response.error or "Failed to get data"
                    )
                
                return {
                    "success": response.success,
                    "data": response.data,
                    "source": response.source,
                    "timestamp": response.timestamp.isoformat(),
                    "cached": response.cached
                }
            except HTTPException:
                raise
            except Exception as e:
                logger.error(f"Failed to get data from {source_type}/{source_name}: {e}")
                raise HTTPException(status_code=500, detail="Internal server error")
        
        @self.app.get("/mcp/data-sources/status")
        async def get_data_source_status():
            """Get status of all data sources"""
            try:
                status = self.data_source_manager.get_source_status()
                return {
                    "status": "healthy",
                    "sources": status,
                    "timestamp": datetime.now().isoformat()
                }
            except Exception as e:
                logger.error(f"Failed to get data source status: {e}")
                raise HTTPException(status_code=500, detail="Failed to get status")
    
    async def _get_session(self, authorization: HTTPAuthorizationCredentials = Depends(HTTPBearer())) -> WalletSession:
        """Dependency to get authenticated session"""
        session_id = authorization.credentials
        session = self.wallet_auth.get_session(session_id)
        
        if not session:
            raise HTTPException(status_code=401, detail="Invalid or expired session")
        
        return session
    
    async def _execute_command(
        self,
        command: str,
        parameters: Dict[str, Any],
        context: UserContext,
        session: WalletSession
    ) -> Any:
        """Execute trading command based on type"""
        
        command_lower = command.lower()
        
        if command_lower == "balance":
            return await self.wallet_auth.get_wallet_balance(session.session_id)
        
        elif command_lower == "arbitrage":
            return await self._execute_arbitrage(parameters, context)
        
        elif command_lower in ["swap", "trade"]:
            return await self._execute_swap(parameters, context)
        
        elif command_lower == "rebalance":
            return await self._execute_rebalance(parameters, context)
        
        elif command_lower in ["set", "update"]:
            return await self._update_settings(parameters, context, session)
        
        else:
            raise ValueError(f"Unknown command: {command}")
    
    async def _execute_arbitrage(self, parameters: Dict[str, Any], context: UserContext) -> Dict[str, Any]:
        """Execute arbitrage operation"""
        # This would integrate with actual arbitrage logic
        logger.info(f"Executing arbitrage with parameters: {parameters}")
        
        return {
            "type": "arbitrage",
            "status": "simulated",  # In MVP, just simulate
            "parameters": parameters,
            "message": "Arbitrage simulation completed"
        }
    
    async def _execute_swap(self, parameters: Dict[str, Any], context: UserContext) -> Dict[str, Any]:
        """Execute token swap"""
        logger.info(f"Executing swap with parameters: {parameters}")
        
        return {
            "type": "swap",
            "status": "simulated",  # In MVP, just simulate
            "parameters": parameters,
            "message": "Swap simulation completed"
        }
    
    async def _execute_rebalance(self, parameters: Dict[str, Any], context: UserContext) -> Dict[str, Any]:
        """Execute portfolio rebalance"""
        logger.info(f"Executing rebalance with parameters: {parameters}")
        
        return {
            "type": "rebalance",
            "status": "simulated",  # In MVP, just simulate
            "parameters": parameters,
            "message": "Rebalance simulation completed"
        }
    
    async def _update_settings(
        self,
        parameters: Dict[str, Any],
        context: UserContext,
        session: WalletSession
    ) -> Dict[str, Any]:
        """Update user settings"""
        success = await self.context_manager.update_trading_preferences(
            session.address,
            parameters
        )
        
        return {
            "type": "settings_update",
            "success": success,
            "parameters": parameters,
            "message": "Settings updated successfully" if success else "Failed to update settings"
        }
    
    async def startup(self):
        """Initialize async components on startup"""
        try:
            await self.data_source_manager.initialize()
            logger.info("Data source manager initialized")
        except Exception as e:
            logger.error(f"Failed to initialize data source manager: {e}")
    
    async def shutdown(self):
        """Clean up async components on shutdown"""
        try:
            await self.data_source_manager.cleanup()
            logger.info("Data source manager cleaned up")
        except Exception as e:
            logger.error(f"Failed to cleanup data source manager: {e}")
    
    def run(self, host: str = "0.0.0.0", port: int = 8000, debug: bool = False):
        """Run the MCP server"""
        # Add startup and shutdown event handlers
        @self.app.on_event("startup")
        async def startup_event():
            await self.startup()
        
        @self.app.on_event("shutdown")
        async def shutdown_event():
            await self.shutdown()
        
        uvicorn.run(
            self.app,
            host=host,
            port=port,
            reload=debug,
            log_level="info"
        )


def create_server() -> MCPServer:
    """Factory function to create MCP server with environment configuration"""
    
    # Load configuration from environment
    ethereum_rpc_url = os.getenv(
        "ETHEREUM_RPC_URL",
        "https://eth-mainnet.alchemyapi.io/v2/your-key"
    )
    
    ipfs_api = os.getenv("IPFS_API", "/ip4/127.0.0.1/tcp/5001")
    
    flashbots_private_key = os.getenv("FLASHBOTS_PRIVATE_KEY")
    
    enable_mev_protection = os.getenv("ENABLE_MEV_PROTECTION", "true").lower() == "true"
    
    return MCPServer(
        ethereum_rpc_url=ethereum_rpc_url,
        ipfs_api=ipfs_api,
        flashbots_private_key=flashbots_private_key,
        enable_mev_protection=enable_mev_protection
    )


# Create module-level app for uvicorn
app = create_server().app

if __name__ == "__main__":
    server = create_server()
    server.run(debug=True)