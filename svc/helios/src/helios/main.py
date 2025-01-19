import logging
from fastapi import FastAPI, Request, HTTPException
from fastapi.responses import HTMLResponse, RedirectResponse
import json
from helios.log import log_with_context as logwc

app = FastAPI()


@app.get("/", response_class=HTMLResponse)
async def index(request: Request):
    logwc(
        level="info",
        message="Index page accessed",
        logger=logging.getLogger(__name__),
        context={
            "user": "anonymous" if "phantom" not in request.cookies else "connected"
        },
    )
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
                }).catch(error => {
                    console.error('Error connecting to Phantom:', error);
                    alert('Failed to connect to Phantom wallet');
                });
            } else {
                alert('Please install Phantom wallet extension');
            }
        }
        </script>
    </body>
    </html>
    """


@app.get("/dashboard")
async def dashboard(request: Request):
    try:
        # Check if Phantom wallet is connected
        if "phantom" not in request.cookies:
            raise HTTPException(status_code=400, detail="Phantom wallet not detected")

        phantom = json.loads(request.cookies.get("phantom", "{}"))
        if not phantom.get("isConnected", False):
            raise HTTPException(status_code=400, detail="Phantom wallet not connected")

        public_key = phantom["publicKey"]

        logwc(
            level="info",
            message="User accessed dashboard",
            logger=logging.getLogger(__name__),
            context={"publicKey": public_key},
        )

        return {"message": f"Welcome, {public_key}"}
    except HTTPException as e:
        logwc(
            level="error",
            message="Failed to access dashboard",
            logger=logging.getLogger(__name__),
            context={"error": str(e)},
        )
        return RedirectResponse(url="/", status_code=303)


@app.get("/logout")
async def logout(request: Request):
    response = RedirectResponse(url="/")
    response.delete_cookie("phantom")

    logwc(
        level="info",
        message="User logged out",
        logger=logging.getLogger(__name__),
        context={
            "publicKey": request.cookies.get("phantom", {}).get("publicKey", "unknown")
        },
    )

    return response


@app.get("/status/helios", response_class=HTMLResponse)
async def index() -> str:

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
