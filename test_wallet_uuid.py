#!/usr/bin/env python3
"""
Test script to verify wallet-to-UUID conversion logic.
This mimics the Rust implementation for verification.
"""

import hashlib
import uuid

def wallet_to_uuid(wallet_address, chain):
    # Create input string combining wallet and chain for uniqueness
    input_str = f"{wallet_address.lower()}:{chain.lower()}"

    # Generate SHA-256 hash
    hash_bytes = hashlib.sha256(input_str.encode()).digest()

    # Convert first 16 bytes of hash to UUID
    uuid_bytes = bytearray(hash_bytes[:16])

    # Set version (4) and variant bits to create a valid UUID v4
    uuid_bytes[6] = (uuid_bytes[6] & 0x0F) | 0x40  # Version 4
    uuid_bytes[8] = (uuid_bytes[8] & 0x3F) | 0x80  # Variant 10

    return uuid.UUID(bytes=bytes(uuid_bytes))

# Test with existing wallet addresses from the database
test_wallets = [
    ("9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM", "solana"),
    ("5wrmi85pTPmB4NDv7rUYncEMi1KqVo93bZn3XtXSbjYT", "solana"),
]

print("🧪 Testing wallet-to-UUID conversion:")
print()

for wallet, chain in test_wallets:
    result_uuid = wallet_to_uuid(wallet, chain)
    print(f"Wallet: {wallet}")
    print(f"Chain:  {chain}")
    print(f"UUID:   {result_uuid}")
    print()

# Test consistency - same wallet should always produce same UUID
print("🔄 Testing consistency:")
uuid1 = wallet_to_uuid(test_wallets[0][0], test_wallets[0][1])
uuid2 = wallet_to_uuid(test_wallets[0][0], test_wallets[0][1])
print(f"Same wallet, call 1: {uuid1}")
print(f"Same wallet, call 2: {uuid2}")
print(f"Consistent: {uuid1 == uuid2}")