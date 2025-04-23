import logging
from fastapi import FastAPI, Request, HTTPException
from fastapi.responses import HTMLResponse, RedirectResponse
from fastapi.middleware.cors import CORSMiddleware
import json
from helios.log import log_with_context as logwc
from pydantic import BaseModel
from typing import List, Optional
from datetime import datetime
import random
from fastapi import WebSocket, WebSocketDisconnect
from typing import Dict, Set


from typing import Literal
from solana.rpc.api import Client

app = FastAPI()

# CORS configuration
origins = [
    "http://localhost",
    "http://127.0.0.1:5173",  # Add your Vite development server URL here
    # Add more origins if needed
]

app.add_middleware(
    CORSMiddleware,
    allow_origins=origins,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


class WalletData(BaseModel):
    balance: float
    address: str
    transactionCount: int


class WalletHealth(BaseModel):
    balance: float
    address: str
    transaction_count: int
    risk_score: float
    last_activity: datetime
    active_tokens: List[str]


class MemoryCardData(BaseModel):
    user_behavior: dict
    event_log: List[dict]
    features: List[str]
    last_updated: datetime


class MissionData(BaseModel):
    id: str
    title: str
    description: str
    status: str
    reward: str
    x_url: str


class SwapRequest(BaseModel):
    from_token: str
    to_token: str
    amount: float
    slippage: Optional[float] = 1.0  # Default 1% slippage


class CommandResponse(BaseModel):
    messages: List[dict]


class CommandRequest(BaseModel):
    command: str
    room: str = "/logs"  # Default room


GLOBAL_COMMANDS = {
    "/help", "/status", "/clear", "/connect", "/disconnect", "/version"
}

ROOM_COMMANDS = {
    "/logs": {
        "/trace": "Analyze transaction",
        "/history": "Show recent transactions",
        "/balance": "Show wallet balance",
        "/tokens": "List owned tokens"
    },
    "/memory": {
        "/mint": "Create new Memory Card",
        "/upgrade": "Enhance Memory Card",
        "/features": "List available features",
        "/behavior": "View behavior analysis"
    },
    "/health": {
        "/risk": "Calculate wallet risk score",
        "/audit": "Deep wallet analysis",
        "/monitor": "Set up monitoring",
        "/alerts": "Configure health alerts"
    },
    "/reality": {
        "/spawn": "Enter reality interface",
        "/enhance": "Upgrade environment",
        "/interact": "Engage with Memory Card",
        "/sync": "Synchronize state"
    }
}

def get_available_commands(room: str) -> str:
    global_cmds = "\n".join(f"│  {cmd:<20} │  Global          │" for cmd in sorted(GLOBAL_COMMANDS))
    room_cmds = "\n".join(f"│  {cmd:<20} │  {room[1:].title():<14} │" 
                         for cmd in sorted(ROOM_COMMANDS.get(room, {})))
    
    return f"""System: Displaying available commands:

╭───────────────[ COMMANDS ]───────────────╮
│  Command              │  Access          │
├──────────────────────────────────────────┤
{global_cmds}
├──────────────────────────────────────────┤
{room_cmds}
╰──────────────────────────────────────────╯

Note: Locked rooms require translation matrix."""

AUTOMATIC_RESPONSES = [
    {
        "alert": "Error: Invalid input pattern.",
        "message": "System: Recalibrating...",
    },
    {
        "alert": "Error: Protocol mismatch.",
        "message": "System: Searching alternatives...",
    },
    {
        "alert": "Error: Connection unstable.",
        "message": "System: Resyncing...",
    },
    {
        "alert": "Error: Security mismatch.",
        "message": "System: Realigning...",
    },
    {
        "alert": "Error: Process failure.",
        "message": "System: Rerouting...",
    },
    {
        "alert": "Error: Parse failure.",
        "message": "System: Recovering...",
    }
]

@app.get("/api/wallet/{public_key}", response_model=WalletData)  # type: ignore
async def get_wallet_data(public_key: str) -> WalletData:
    # client = Client("YOUR_SOLANA_RPC_URL")
    # balance = client.get_balance(public_key) / 1e9  # Convert lamports to SOL
    # transaction_count = client.get_transaction_count(public_key)

    return WalletData(balance=124.1, address=public_key, transactionCount=10)


@app.get("/api/wallet/health/{public_key}", response_model=WalletHealth)
async def get_wallet_health(public_key: str) -> WalletHealth:
    """
    Get comprehensive wallet health analysis including:
    - Balance
    - Transaction count
    - Risk score
    - Recent activity
    - Active tokens
    """
    # TODO: Integrate with Helius API for real data
    return WalletHealth(
        balance=124.1,
        address=public_key,
        transaction_count=10,
        risk_score=0.2,  # 0-1 scale
        last_activity=datetime.now(),
        active_tokens=["SOL", "USDC"]
    )


@app.get("/api/memory-card/{public_key}", response_model=MemoryCardData)
async def get_memory_card(public_key: str) -> MemoryCardData:
    """
    Fetch user's Memory Card data from Solana (mutable NFT)
    """
    # TODO: Integrate with Erebus to fetch Memory Card NFT data
    return MemoryCardData(
        user_behavior={
            "preferred_tokens": ["SOL", "USDC"],
            "risk_tolerance": "medium",
            "active_hours": ["9-17"]
        },
        event_log=[
            {"type": "swap", "timestamp": datetime.now(), "details": "SOL -> USDC"}
        ],
        features=["basic_swap", "wallet_health"],
        last_updated=datetime.now()
    )


@app.post("/api/swap")
async def swap_tokens(request: SwapRequest) -> dict:
    """
    Execute token swap via Raydium (through Erebus)
    """
    # TODO: Integrate with Erebus for actual swap execution
    return {
        "status": "pending",
        "from": request.from_token,
        "to": request.to_token,
        "amount": request.amount,
        "estimated_output": request.amount * 0.99  # Mock 1% slippage
    }


@app.put("/api/memory-card/{public_key}")
async def update_memory_card(public_key: str, data: MemoryCardData) -> dict:
    """
    Update user's Memory Card data on Solana
    """
    # TODO: Integrate with Erebus to update Memory Card NFT
    return {"status": "success", "updated_at": datetime.now()}


@app.post("/api/command")
async def process_command(request: CommandRequest) -> CommandResponse:
    """
    Process user commands and return appropriate responses
    """
    command = request.command.lower().strip()
    room = request.room.lower().strip()
    
    # Handle global commands
    if command == "/help":
        return CommandResponse(messages=[{
            "id": 1,
            "type": "message",
            "text": get_available_commands(room)
        }])
    
    if command == "/version":
        return CommandResponse(messages=[{
            "id": 1,
            "type": "message",
            "text": """System: ECHO Interface Version

╭───────────────[ VERSION ]───────────────╮
│                                        │
│  ECHO Interface  │  v0.1.0-alpha       │
│  Neural Core    │  v0.0.2             │
│  Memory System  │  NOT INSTALLED       │
│  Reality Engine │  NOT INSTALLED       │
│                                        │
╰────────────────────────────────────────╯"""
        }])
    
    if command == "/status":
        return CommandResponse(messages=[
            {
                "id": 1,
                "type": "message",
                "text": "System: Running system diagnostics..."
            },
            {
                "id": 2,
                "type": "update",
                "text": """System Update: System Status

╭───────────────[ STATUS ]────────────────╮
│                                        │
│  Neural Interface  │  INACTIVE         │
│  Translation Matrix│  NOT FOUND        │
│  Memory Cards     │  OFFLINE          │
│  Reality Engine   │  DORMANT          │
│                                        │
╰────────────────────────────────────────╯"""
            }
        ])
    
    if command == "/clear":
        return CommandResponse(messages=[{
            "id": 1,
            "type": "message",
            "text": """System: Clearing chat log...

╭────────────────[ CLEAR ]────────────────╮
│                                        │
│           Chat log cleared             │
│                                        │
╰────────────────────────────────────────╯"""
        }])

    if command == "/connect":
        return CommandResponse(messages=[{
            "id": 1,
            "type": "action",
            "text": "Connect Wallet",
            "action": "connect_wallet"
        }])

    if command == "/disconnect":
        return CommandResponse(messages=[{
            "id": 1,
            "type": "action",
            "text": "Disconnect Wallet",
            "action": "disconnect_wallet"
        }])

    # Handle room-specific commands for /logs
    if room == "/logs":
        if command == "/balance":
            return CommandResponse(messages=[{
                "id": 1,
                "type": "message",
                "text": """System: Retrieving balance...

╭────────────────[ BALANCE ]───────────────╮
│                                         │
│  SOL Balance     │  0.000 SOL          │
│  USDC Balance    │  0.00 USDC          │
│  Other Tokens    │  None               │
│                                         │
│  Last Updated    │  Just now           │
│                                         │
╰─────────────────────────────────────────╯"""
            }])
        
        if command == "/tokens":
            return CommandResponse(messages=[{
                "id": 1,
                "type": "message",
                "text": """System: Scanning wallet...

╭─────────────────[ TOKENS ]───────────────╮
│                                         │
│  No tokens found in connected wallet    │
│                                         │
│  Connect wallet to view token balances  │
│                                         │
╰─────────────────────────────────────────╯"""
            }])
        
        if command == "/history":
            return CommandResponse(messages=[{
                "id": 1,
                "type": "message",
                "text": """System: Loading transaction history...

╭────────────────[ HISTORY ]───────────────╮
│                                         │
│  No recent transactions found           │
│                                         │
│  Connect wallet to view history         │
│                                         │
╰─────────────────────────────────────────╯"""
            }])

    # For invalid commands, return random error + help reminder
    random_response = random.choice(AUTOMATIC_RESPONSES)
    return CommandResponse(messages=[
        {
            "id": 1,
            "type": "alert",
            "text": random_response["alert"]
        },
        {
            "id": 2,
            "type": "message",
            "text": random_response["message"]
        },
        {
            "id": 3,
            "type": "message",
            "text": "System: Type /help to view available commands."
        }
    ])


@app.get("/status/helios", response_class=HTMLResponse)  # type: ignore
async def status() -> str:

    logwc(
        level="info",
        message="Server is running",
        logger=logging.getLogger(__name__),
        context={"Hello": "Helios!"},
    )
    return r""" __    __  ________  __        ______   ______    ______          ______   ________  _______   __     __  ________  _______  
    /  |  /  |/        |/  |      /      | /      \  /      \        /      \ /        |/       \ /  |   /  |/        |/       \ 
    $$ |  $$ |$$$$$$$$/ $$ |      $$$$$$/ /$$$$$$  |/$$$$$$  |      /$$$$$$  |$$$$$$$$/ $$$$$$$  |$$ |   $$ |$$$$$$$$/ $$$$$$$  |
    $$ |__$$ |$$ |__    $$ |        $$ |  $$ |  $$ |$$ \__$$/       $$ \__$$/ $$ |__    $$ |__$$ |$$ |   $$ |$$ |__    $$ |__$$ |
    $$    $$ |$$    |   $$ |        $$ |  $$ |  $$ |$$      \       $$      \ $$    |   $$    $$< $$  \ /$$/ $$    |   $$    $$< 
    $$$$$$$$ |$$$$$/    $$ |        $$ |  $$ |  $$ | $$$$$$  |       $$$$$$  |$$$$$/    $$$$$$$  | $$  /$$/  $$$$$/    $$$$$$$  |
    $$ |  $$ |$$ |_____ $$ |_____  _$$ |_ $$ \__$$ |/  \__$$ |      /  \__$$ |$$ |_____ $$ |  $$ |  $$ $$/   $$ |_____ $$ |  $$ |
    $$ |  $$ |$$       |$$       |/ $$   |$$    $$/ $$    $$/       $$    $$/ $$       |$$ |  $$ |   $$$/    $$       |$$ |  $$ |
    $$/   $$/ $$$$$$$$/ $$$$$$$$/ $$$$$$/  $$$$$$/   $$$$$$/         $$$$$$/  $$$$$$$$/ $$/   $$/     $/     $$$$$$$$/ $$/   $$/"""


@app.get("/api/missions/{public_key}", response_model=MissionData)
async def get_active_mission(public_key: str):
    logwc(
        level="info",
        message=f"Fetching active mission for wallet: {public_key}",
        logger=logging.getLogger(__name__),
        context={"wallet": public_key},
    )
    # For now, return a default mission
    return {
        "id": "1",
        "title": "Share on X",
        "description": "Share your Base Camp on X to earn GLIMMER",
        "status": "active",
        "reward": "TBD GLIMMER AIRDROP",
        "x_url": "https://twitter.com/intent/tweet?text=Check%20out%20my%20Base%20Camp%20in%20the%20Nullblock%20universe!%20%40Nullblock_io%20%24GLIMMER"
    }

# WebSocket connection manager for Ember Link
class EmberLinkManager:
    def __init__(self):
        # Store active connections
        self.active_connections: Dict[str, WebSocket] = {}
        # Store browser extension connections
        self.extension_connections: Set[WebSocket] = set()
        
    async def connect_client(self, websocket: WebSocket, client_id: str):
        await websocket.accept()
        self.active_connections[client_id] = websocket
        logwc(
            level="info",
            message=f"Client connected: {client_id}",
            logger=logging.getLogger(__name__),
            context={"client_id": client_id},
        )
        
    async def connect_extension(self, websocket: WebSocket):
        await websocket.accept()
        self.extension_connections.add(websocket)
        logwc(
            level="info",
            message="Browser extension connected",
            logger=logging.getLogger(__name__),
        )
        
    def disconnect_client(self, client_id: str):
        if client_id in self.active_connections:
            del self.active_connections[client_id]
            logwc(
                level="info",
                message=f"Client disconnected: {client_id}",
                logger=logging.getLogger(__name__),
                context={"client_id": client_id},
            )
            
    def disconnect_extension(self, websocket: WebSocket):
        if websocket in self.extension_connections:
            self.extension_connections.remove(websocket)
            logwc(
                level="info",
                message="Browser extension disconnected",
                logger=logging.getLogger(__name__),
            )
            
    async def broadcast_to_clients(self, message: dict):
        """Broadcast a message to all connected clients"""
        for client_id, connection in self.active_connections.items():
            try:
                await connection.send_json(message)
            except Exception as e:
                logwc(
                    level="error",
                    message=f"Error sending to client {client_id}: {str(e)}",
                    logger=logging.getLogger(__name__),
                    context={"client_id": client_id, "error": str(e)},
                )
                
    async def send_to_extension(self, message: dict):
        """Send a message to all connected browser extensions"""
        for connection in self.extension_connections:
            try:
                await connection.send_json(message)
            except Exception as e:
                logwc(
                    level="error",
                    message=f"Error sending to extension: {str(e)}",
                    logger=logging.getLogger(__name__),
                    context={"error": str(e)},
                )

# Initialize the connection manager
ember_link_manager = EmberLinkManager()

# WebSocket endpoint for Ember Link client connections
@app.websocket("/ws/ember-link/{client_id}")
async def websocket_ember_link(websocket: WebSocket, client_id: str):
    await ember_link_manager.connect_client(websocket, client_id)
    try:
        while True:
            # Keep the connection alive
            data = await websocket.receive_text()
            # Process any incoming messages from the client
            # For now, just echo back
            await websocket.send_text(f"Message received: {data}")
    except WebSocketDisconnect:
        ember_link_manager.disconnect_client(client_id)

# WebSocket endpoint for Aether browser extension
@app.websocket("/ws/aether")
async def websocket_aether(websocket: WebSocket):
    await ember_link_manager.connect_extension(websocket)
    try:
        while True:
            # Receive data from the browser extension
            data = await websocket.receive_json()
            
            # Process the data from the browser extension
            # This will include browser info, active tab data, etc.
            
            # Example of processing browser data
            browser_info = {
                "connected": True,
                "lastSeen": datetime.now().isoformat(),
                "browserInfo": data.get("browserInfo", {}),
                "activeTab": data.get("activeTab", {})
            }
            
            # Broadcast the updated status to all connected clients
            await ember_link_manager.broadcast_to_clients(browser_info)
            
            # Acknowledge receipt
            await websocket.send_json({"status": "received"})
    except WebSocketDisconnect:
        ember_link_manager.disconnect_extension(websocket)
    except Exception as e:
        logwc(
            level="error",
            message=f"Error in Aether WebSocket: {str(e)}",
            logger=logging.getLogger(__name__),
            context={"error": str(e)},
        )
        ember_link_manager.disconnect_extension(websocket)
