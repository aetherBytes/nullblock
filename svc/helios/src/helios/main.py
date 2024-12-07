import asyncio
from typing import Dict, Any, List
import websockets

# from helios.solana_utils import setup_solana_client, get_transactions

# from helios.processing_utils import process_and_clean_data, serialized_data
# from helios.config import config
import pandas as pd

# async def broadcast_data(
#     websocket: websockets.WebSocketServerProtocol, data: str
# ) -> None:
#     await websocket.send(data)
#
#
# async def handler(websocket: websockets.WebSocketServerProtocol, path: str) -> None:
#     client = setup_solana_client()
#     while True:
#         try:
#             # Fetch latest transactions for the watched address
#             raw_data: Dict[str, List[Dict[str, Any]]] = get_transactions(
#                 client, config.WATCH_ADDRESS
#             )
#
#             # Process and clean data
#             processed_data: pd.DataFrame = process_and_clean_data(raw_data["result"])
#
#             # Serialize data for WebSocket transmission
#             serialized_data: str = serialize_data(processed_data)
#
#             # Broadcast data to all connected clients
#             await broadcast_data(websocket, serialized_data)
#
#             # Wait for a bit before the next fetch to avoid overloading
#             await asyncio.sleep(60)  # 1 minute
#
#         except Exception as e:
#             print(f"Error in data processing or transmission: {e}")
#             # Continue the loop to try again after a short delay
#             await asyncio.sleep(10)  # 10 seconds delay on error
#
#
# async def main() -> None:
#     async with websockets.serve(handler, "localhost", 8765):
#         await asyncio.Future()  # run forever


if __name__ == "__main__":
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
