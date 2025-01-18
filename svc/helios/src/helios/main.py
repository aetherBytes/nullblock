from fastapi import FastAPI
from fastapi.responses import HTMLResponse


app = FastAPI()

# Configure Solana client
# solana_client = Client(
#     "https://api.mainnet-beta.solana.com"
# )  # Use testnet or devnet for testing


# Phantom detection and connection
# async def detect_and_connect_phantom(request: Request):
#     if "phantom" not in request.cookies:
#         raise HTTPException(status_code=400, detail="Phantom wallet not detected")
#
#     phantom = json.loads(request.cookies.get("phantom", "{}"))
#     if not phantom.get("isConnected", False):
#         raise HTTPException(status_code=400, detail="Phantom wallet not connected")
#
#     return PublicKey(phantom["publicKey"])
#
#
@app.get("/", response_class=HTMLResponse)
async def index() -> str:
    return r""" __    __  ________  __        ______   ______    ______          ______   ________  _______   __     __  ________  _______  
    /  |  /  |/        |/  |      /      | /      \  /      \        /      \ /        |/       \ /  |   /  |/        |/       \ 
    $$ |  $$ |$$$$$$$$/ $$ |      $$$$$$/ /$$$$$$  |/$$$$$$  |      /$$$$$$  |$$$$$$$$/ $$$$$$$  |$$ |   $$ |$$$$$$$$/ $$$$$$$  |
    $$ |__$$ |$$ |__    $$ |        $$ |  $$ |  $$ |$$ \__$$/       $$ \__$$/ $$ |__    $$ |__$$ |$$ |   $$ |$$ |__    $$ |__$$ |
    $$    $$ |$$    |   $$ |        $$ |  $$ |  $$ |$$      \       $$      \ $$    |   $$    $$< $$  \ /$$/ $$    |   $$    $$< 
    $$$$$$$$ |$$$$$/    $$ |        $$ |  $$ |  $$ | $$$$$$  |       $$$$$$  |$$$$$/    $$$$$$$  | $$  /$$/  $$$$$/    $$$$$$$  |
    $$ |  $$ |$$ |_____ $$ |_____  _$$ |_ $$ \__$$ |/  \__$$ |      /  \__$$ |$$ |_____ $$ |  $$ |  $$ $$/   $$ |_____ $$ |  $$ |
    $$ |  $$ |$$       |$$       |/ $$   |$$    $$/ $$    $$/       $$    $$/ $$       |$$ |  $$ |   $$$/    $$       |$$ |  $$ |
    $$/   $$/ $$$$$$$$/ $$$$$$$$/ $$$$$$/  $$$$$$/   $$$$$$/         $$$$$$/  $$$$$$$$/ $$/   $$/     $/     $$$$$$$$/ $$/   $$/"""
