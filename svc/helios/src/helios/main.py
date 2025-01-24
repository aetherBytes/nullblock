import logging
from fastapi import FastAPI, Request, HTTPException
from fastapi.responses import HTMLResponse, RedirectResponse
from fastapi.middleware.cors import CORSMiddleware
import json
from helios.log import log_with_context as logwc
from pydantic import BaseModel


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


@app.get("/api/wallet/{public_key}", response_model=WalletData)  # type: ignore
async def get_wallet_data(public_key: str) -> WalletData:
    # client = Client("YOUR_SOLANA_RPC_URL")
    # balance = client.get_balance(public_key) / 1e9  # Convert lamports to SOL
    # transaction_count = client.get_transaction_count(public_key)

    return WalletData(balance=124.1, address=public_key, transactionCount=10)


# @app.get("/dashboard")
# async def dashboard(request: Request):
#     try:
#         # Check if Phantom wallet is connected
#         if "phantom" not in request.cookies:
#             raise HTTPException(status_code=400, detail="Phantom wallet not detected")
#
#         phantom = json.loads(request.cookies.get("phantom", "{}"))
#         if not phantom.get("isConnected", False):
#             raise HTTPException(status_code=400, detail="Phantom wallet not connected")
#
#         public_key = phantom["publicKey"]
#
#         logwc(
#             level="info",
#             message="User accessed dashboard",
#             logger=logging.getLogger(__name__),
#             context={"publicKey": public_key},
#         )
#
#         return {"message": f"Welcome, {public_key}"}
#     except HTTPException as e:
#         logwc(
#             level="error",
#             message="Failed to access dashboard",
#             logger=logging.getLogger(__name__),
#             context={"error": str(e)},
#         )
#         return RedirectResponse(url="/", status_code=303)
#
#
# @app.get("/logout")
# async def logout(request: Request):
#     response = RedirectResponse(url="/")
#     response.delete_cookie("phantom")
#
#     logwc(
#         level="info",
#         message="User logged out",
#         logger=logging.getLogger(__name__),
#         context={
#             "publicKey": request.cookies.get("phantom", {}).get("publicKey", "unknown")
#         },
#     )
#
#     return response


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
