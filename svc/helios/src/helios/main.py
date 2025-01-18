from fastapi import FastAPI, Request, HTTPException
from fastapi.responses import HTMLResponse
from solana.publickey import PublicKey
from solana.rpc.api import Client
from starlette.responses import RedirectResponse
import json
from solders.rpc.responses import GetBalanceResp

from helios.config import config

app = FastAPI()

# Configure Solana client
solana_client = Client(
    "https://api.mainnet-beta.solana.com"
)  # Use testnet or devnet for testing


# Phantom detection and connection
async def detect_and_connect_phantom(request: Request):
    if "phantom" not in request.cookies:
        raise HTTPException(status_code=400, detail="Phantom wallet not detected")

    phantom = json.loads(request.cookies.get("phantom", "{}"))
    if not phantom.get("isConnected", False):
        raise HTTPException(status_code=400, detail="Phantom wallet not connected")

    return PublicKey(phantom["publicKey"])


@app.get("/", response_class=HTMLResponse)
async def index():
    return """
    <!DOCTYPE html>
    <html>
    <head>
        <title>Helios: Phantom Login</title>
        <script src="https://unpkg.com/@solana/web3.js@latest/lib/index.iife.js"></script>
    </head>
    <body>
        <h1>Welcome to Helios</h1>
        <p>Please connect your Phantom wallet to access features.</p>
        <button onclick="connectPhantom()">Connect Phantom</button>
        <script>
        function connectPhantom() {
            if ('phantom' in window) {
                const phantom = window.phantom.solana;
                phantom.connect().then(({ publicKey }) => {
                    if (publicKey) {
                        document.cookie = `phantom=${JSON.stringify({publicKey: publicKey.toString(), isConnected: true})}; path=/`;
                        window.location.href = '/dashboard';
                    }
                });
            } else {
                alert('Please install Phantom wallet');
            }
        }
        </script>
    </body>
    </html>
    """


@app.get("/dashboard")
async def dashboard(request: Request):
    try:
        public_key = await detect_and_connect_phantom(request)
        balance = await get_balance(public_key)
        return {"message": f"Welcome, {public_key}", "balance": balance}
    except HTTPException:
        return RedirectResponse(url="/", status_code=303)


# Example interaction: get Solana balance
async def get_balance(public_key: PublicKey):
    balance_response: GetBalanceResp = await solana_client.get_balance(public_key)
    return balance_response.value / 1e9  # Convert lamports to SOL


@app.get("/logout")
async def logout(request: Request):
    response = RedirectResponse(url="/")
    response.delete_cookie("phantom")
    return response


# Main block for testing if run directly
if __name__ == "__main__":
    import uvicorn

    uvicorn.run(app, host="0.0.0.0", port=8000)
    service_name = r""" __    __  ________  __        ______   ______    ______          ______   ________  _______   __     __  ________  _______  
    /  |  /  |/        |/  |      /      | /      \  /      \        /      \ /        |/       \ /  |   /  |/        |/       \ 
    $$ |  $$ |$$$$$$$$/ $$ |      $$$$$$/ /$$$$$$  |/$$$$$$  |      /$$$$$$  |$$$$$$$$/ $$$$$$$  |$$ |   $$ |$$$$$$$$/ $$$$$$$  |
    $$ |__$$ |$$ |__    $$ |        $$ |  $$ |  $$ |$$ \__$$/       $$ \__$$/ $$ |__    $$ |__$$ |$$ |   $$ |$$ |__    $$ |__$$ |
    $$    $$ |$$    |   $$ |        $$ |  $$ |  $$ |$$      \       $$      \ $$    |   $$    $$< $$  \ /$$/ $$    |   $$    $$< 
    $$$$$$$$ |$$$$$/    $$ |        $$ |  $$ |  $$ | $$$$$$  |       $$$$$$  |$$$$$/    $$$$$$$  | $$  /$$/  $$$$$/    $$$$$$$  |
    $$ |  $$ |$$ |_____ $$ |_____  _$$ |_ $$ \__$$ |/  \__$$ |      /  \__$$ |$$ |_____ $$ |  $$ |  $$ $$/   $$ |_____ $$ |  $$ |
    $$ |  $$ |$$       |$$       |/ $$   |$$    $$/ $$    $$/       $$    $$/ $$       |$$ |  $$ |   $$$/    $$       |$$ |  $$ |
    $$/   $$/ $$$$$$$$/ $$$$$$$$/ $$$$$$/  $$$$$$/   $$$$$$/         $$$$$$/  $$$$$$$$/ $$/   $$/     $/     $$$$$$$$/ $$/   $$/"""
    print(service_name)
    # asyncio.run(main())
    config = config.config()
    print(config.test)
