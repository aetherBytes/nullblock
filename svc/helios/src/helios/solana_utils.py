from solana.publickey import PublicKey
from solana.rpc.api import Client
from solana.rpc.commitment import Commitment
import config


def setup_solana_client():
    return Client(config.SOLANA_RPC_URL)


def get_account_info(client, address, commitment=Commitment("confirmed")):
    return client.get_account_info(PublicKey(address), commitment=commitment)


# Example function to get transaction details for the watched address
def get_transactions(client, address, limit=50):
    return client.get_signatures_for_address(PublicKey(address), limit=limit)
