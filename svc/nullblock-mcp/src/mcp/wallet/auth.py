"""
Wallet authentication module supporting MetaMask, WalletConnect, and Phantom
"""

import logging
from typing import Dict, Optional, Any, Protocol
from abc import ABC, abstractmethod
from datetime import datetime, timedelta
import json
from web3 import Web3
from eth_account.messages import encode_defunct
from eth_account import Account
from pydantic import BaseModel, Field
import secrets
import hashlib

logger = logging.getLogger(__name__)


class WalletProvider(Protocol):
    """Protocol for wallet providers"""
    
    async def verify_signature(
        self, 
        address: str, 
        message: str, 
        signature: str
    ) -> bool:
        """Verify wallet signature"""
        ...
    
    async def get_balance(self, address: str) -> float:
        """Get wallet balance"""
        ...


class AuthChallenge(BaseModel):
    """Authentication challenge for wallet verification"""
    nonce: str = Field(..., description="Random nonce for challenge")
    message: str = Field(..., description="Message to be signed")
    timestamp: datetime = Field(default_factory=datetime.now)
    expires_at: datetime = Field(..., description="Challenge expiration time")
    
    @classmethod
    def create(cls, address: str) -> "AuthChallenge":
        """Create a new authentication challenge"""
        nonce = secrets.token_hex(16)
        timestamp = datetime.now()
        expires_at = timestamp + timedelta(minutes=5)  # 5 minute expiry
        
        message = (
            f"Nullblock MCP Authentication\n"
            f"Wallet: {address}\n"
            f"Nonce: {nonce}\n"
            f"Timestamp: {timestamp.isoformat()}\n"
            f"Please sign this message to authenticate with Nullblock."
        )
        
        return cls(
            nonce=nonce,
            message=message,
            timestamp=timestamp,
            expires_at=expires_at
        )
    
    def is_expired(self) -> bool:
        """Check if challenge has expired"""
        return datetime.now() > self.expires_at


class WalletSession(BaseModel):
    """Authenticated wallet session"""
    address: str = Field(..., description="Wallet address")
    provider: str = Field(..., description="Wallet provider (metamask, walletconnect, phantom)")
    authenticated_at: datetime = Field(default_factory=datetime.now)
    last_activity: datetime = Field(default_factory=datetime.now)
    session_id: str = Field(..., description="Unique session identifier")
    
    @classmethod
    def create(cls, address: str, provider: str) -> "WalletSession":
        """Create a new wallet session"""
        session_id = hashlib.sha256(
            f"{address}:{provider}:{datetime.now().isoformat()}:{secrets.token_hex(8)}"
            .encode()
        ).hexdigest()
        
        return cls(
            address=address,
            provider=provider,
            session_id=session_id
        )
    
    def update_activity(self):
        """Update last activity timestamp"""
        self.last_activity = datetime.now()
    
    def is_expired(self, timeout_minutes: int = 60) -> bool:
        """Check if session has expired"""
        return datetime.now() > self.last_activity + timedelta(minutes=timeout_minutes)


class BaseWalletProvider(ABC):
    """Base class for wallet providers"""
    
    def __init__(self, name: str):
        self.name = name
        self.logger = logging.getLogger(f"{__name__}.{name}")
    
    @abstractmethod
    async def verify_signature(
        self, 
        address: str, 
        message: str, 
        signature: str
    ) -> bool:
        """Verify wallet signature"""
        pass
    
    @abstractmethod
    async def get_balance(self, address: str) -> float:
        """Get wallet balance"""
        pass


class MetaMaskProvider(BaseWalletProvider):
    """MetaMask wallet provider for Ethereum-compatible chains"""
    
    def __init__(self, web3_provider: Optional[Web3] = None):
        super().__init__("metamask")
        self.web3 = web3_provider or Web3()
    
    async def verify_signature(
        self, 
        address: str, 
        message: str, 
        signature: str
    ) -> bool:
        """Verify MetaMask signature using eth_account"""
        try:
            # Create the message hash that MetaMask signs
            message_hash = encode_defunct(text=message)
            
            # Recover the address from the signature
            recovered_address = Account.recover_message(message_hash, signature=signature)
            
            # Compare addresses (case-insensitive)
            return recovered_address.lower() == address.lower()
            
        except Exception as e:
            self.logger.error(f"Signature verification failed: {e}")
            return False
    
    async def get_balance(self, address: str) -> float:
        """Get ETH balance for address"""
        try:
            if not self.web3.is_connected():
                self.logger.warning("Web3 not connected, returning 0 balance")
                return 0.0
            
            balance_wei = self.web3.eth.get_balance(address)
            balance_eth = self.web3.from_wei(balance_wei, 'ether')
            return float(balance_eth)
            
        except Exception as e:
            self.logger.error(f"Failed to get balance for {address}: {e}")
            return 0.0


class WalletConnectProvider(BaseWalletProvider):
    """WalletConnect provider for multi-wallet support"""
    
    def __init__(self, web3_provider: Optional[Web3] = None):
        super().__init__("walletconnect")
        self.web3 = web3_provider or Web3()
    
    async def verify_signature(
        self, 
        address: str, 
        message: str, 
        signature: str
    ) -> bool:
        """Verify WalletConnect signature"""
        # WalletConnect uses the same signature verification as MetaMask
        try:
            message_hash = encode_defunct(text=message)
            recovered_address = Account.recover_message(message_hash, signature=signature)
            return recovered_address.lower() == address.lower()
            
        except Exception as e:
            self.logger.error(f"WalletConnect signature verification failed: {e}")
            return False
    
    async def get_balance(self, address: str) -> float:
        """Get balance for WalletConnect address"""
        try:
            if not self.web3.is_connected():
                self.logger.warning("Web3 not connected, returning 0 balance")
                return 0.0
            
            balance_wei = self.web3.eth.get_balance(address)
            balance_eth = self.web3.from_wei(balance_wei, 'ether')
            return float(balance_eth)
            
        except Exception as e:
            self.logger.error(f"Failed to get balance for {address}: {e}")
            return 0.0


class PhantomProvider(BaseWalletProvider):
    """Phantom wallet provider for Solana"""
    
    def __init__(self, solana_rpc_url: Optional[str] = None):
        super().__init__("phantom")
        # TODO: Implement Solana RPC client for balance checking
        self.solana_rpc_url = solana_rpc_url or "https://api.mainnet-beta.solana.com"
    
    async def verify_signature(
        self, 
        address: str, 
        message: str, 
        signature: str
    ) -> bool:
        """Verify Phantom (Solana) signature"""
        # TODO: Implement Solana signature verification
        # For MVP, we'll implement basic validation
        self.logger.warning("Phantom signature verification not fully implemented")
        return len(signature) > 0 and len(address) > 0
    
    async def get_balance(self, address: str) -> float:
        """Get SOL balance for address"""
        # TODO: Implement Solana balance checking
        self.logger.warning("Phantom balance checking not fully implemented")
        return 0.0


class WalletAuthenticator:
    """Main wallet authentication class"""
    
    def __init__(self, web3_provider: Optional[Web3] = None):
        self.providers: Dict[str, BaseWalletProvider] = {
            "metamask": MetaMaskProvider(web3_provider),
            "walletconnect": WalletConnectProvider(web3_provider),
            "phantom": PhantomProvider()
        }
        self.active_challenges: Dict[str, AuthChallenge] = {}
        self.active_sessions: Dict[str, WalletSession] = {}
        self.logger = logging.getLogger(__name__)
    
    def create_challenge(self, address: str) -> AuthChallenge:
        """Create authentication challenge for wallet"""
        challenge = AuthChallenge.create(address)
        
        # Store challenge with address as key
        self.active_challenges[address.lower()] = challenge
        
        self.logger.info(f"Created auth challenge for {address}")
        return challenge
    
    async def verify_challenge(
        self, 
        address: str, 
        signature: str, 
        provider_name: str = "metamask"
    ) -> Optional[WalletSession]:
        """Verify challenge response and create session"""
        address_key = address.lower()
        
        # Check if challenge exists
        if address_key not in self.active_challenges:
            self.logger.error(f"No active challenge for {address}")
            return None
        
        challenge = self.active_challenges[address_key]
        
        # Check if challenge has expired
        if challenge.is_expired():
            self.logger.error(f"Challenge expired for {address}")
            del self.active_challenges[address_key]
            return None
        
        # Get the appropriate provider
        if provider_name not in self.providers:
            self.logger.error(f"Unknown provider: {provider_name}")
            return None
        
        provider = self.providers[provider_name]
        
        # Verify signature
        is_valid = await provider.verify_signature(
            address, challenge.message, signature
        )
        
        if not is_valid:
            self.logger.error(f"Signature verification failed for {address}")
            return None
        
        # Create session
        session = WalletSession.create(address, provider_name)
        self.active_sessions[session.session_id] = session
        
        # Clean up challenge
        del self.active_challenges[address_key]
        
        self.logger.info(f"Successfully authenticated {address} with {provider_name}")
        return session
    
    def get_session(self, session_id: str) -> Optional[WalletSession]:
        """Get active session by ID"""
        if session_id not in self.active_sessions:
            return None
        
        session = self.active_sessions[session_id]
        
        # Check if session has expired
        if session.is_expired():
            del self.active_sessions[session_id]
            return None
        
        # Update activity
        session.update_activity()
        return session
    
    def revoke_session(self, session_id: str) -> bool:
        """Revoke an active session"""
        if session_id in self.active_sessions:
            del self.active_sessions[session_id]
            self.logger.info(f"Revoked session {session_id}")
            return True
        return False
    
    async def get_wallet_balance(self, session_id: str) -> float:
        """Get balance for authenticated wallet"""
        session = self.get_session(session_id)
        if not session:
            return 0.0
        
        provider = self.providers.get(session.provider)
        if not provider:
            return 0.0
        
        return await provider.get_balance(session.address)
    
    def cleanup_expired(self):
        """Clean up expired challenges and sessions"""
        # Clean up expired challenges
        expired_challenges = [
            addr for addr, challenge in self.active_challenges.items()
            if challenge.is_expired()
        ]
        for addr in expired_challenges:
            del self.active_challenges[addr]
        
        # Clean up expired sessions
        expired_sessions = [
            session_id for session_id, session in self.active_sessions.items()
            if session.is_expired()
        ]
        for session_id in expired_sessions:
            del self.active_sessions[session_id]
        
        if expired_challenges or expired_sessions:
            self.logger.info(
                f"Cleaned up {len(expired_challenges)} challenges and "
                f"{len(expired_sessions)} sessions"
            )