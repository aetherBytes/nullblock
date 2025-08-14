"""
IPFS-based context storage for user preferences and agent settings
"""

import logging
import json
import hashlib
from typing import Dict, Any, Optional, List
from datetime import datetime, timedelta
from dataclasses import dataclass, asdict
from pathlib import Path
import tempfile
import os
import ipfshttpclient
from cryptography.fernet import Fernet
from pydantic import BaseModel, Field
import asyncio
import aiofiles

logger = logging.getLogger(__name__)


class UserContext(BaseModel):
    """User context data structure"""
    wallet_address: str = Field(..., description="User's wallet address")
    preferences: Dict[str, Any] = Field(default_factory=dict, description="User preferences")
    agent_settings: Dict[str, Any] = Field(default_factory=dict, description="Agent-specific settings")
    trading_profile: Dict[str, Any] = Field(default_factory=dict, description="Trading preferences")
    risk_profile: Dict[str, Any] = Field(default_factory=dict, description="Risk management settings")
    created_at: datetime = Field(default_factory=datetime.now)
    updated_at: datetime = Field(default_factory=datetime.now)
    version: int = Field(default=1, description="Context version for migration")
    
    def update(self, **kwargs):
        """Update context and timestamp"""
        for key, value in kwargs.items():
            if hasattr(self, key):
                setattr(self, key, value)
        self.updated_at = datetime.now()


class ArbitrageSettings(BaseModel):
    """Arbitrage-specific settings"""
    min_profit_threshold: float = Field(default=0.01, description="Minimum profit percentage (1%)")
    max_trade_amount: float = Field(default=1000.0, description="Maximum trade amount in USD")
    preferred_dexes: List[str] = Field(default_factory=lambda: ["uniswap", "sushiswap"])
    max_slippage: float = Field(default=0.005, description="Maximum acceptable slippage (0.5%)")
    gas_limit: int = Field(default=500000, description="Gas limit for transactions")
    enable_mev_protection: bool = Field(default=True, description="Use Flashbots for MEV protection")


class DeFiSettings(BaseModel):
    """DeFi yield farming settings"""
    risk_tolerance: str = Field(default="medium", description="Risk tolerance: low, medium, high")
    min_apy: float = Field(default=0.05, description="Minimum acceptable APY (5%)")
    max_pool_allocation: float = Field(default=0.25, description="Max allocation per pool (25%)")
    preferred_protocols: List[str] = Field(default_factory=lambda: ["aave", "compound"])
    auto_compound: bool = Field(default=True, description="Auto-compound rewards")


class ContextEncryption:
    """Encryption utilities for sensitive context data"""
    
    def __init__(self, encryption_key: Optional[bytes] = None):
        if encryption_key is None:
            self.key = Fernet.generate_key()
        else:
            self.key = encryption_key
        self.cipher = Fernet(self.key)
        self.logger = logging.getLogger(f"{__name__}.ContextEncryption")
    
    def encrypt_data(self, data: Dict[str, Any]) -> bytes:
        """Encrypt context data"""
        try:
            json_str = json.dumps(data, default=str)
            return self.cipher.encrypt(json_str.encode())
        except Exception as e:
            self.logger.error(f"Encryption failed: {e}")
            raise
    
    def decrypt_data(self, encrypted_data: bytes) -> Dict[str, Any]:
        """Decrypt context data"""
        try:
            decrypted_bytes = self.cipher.decrypt(encrypted_data)
            json_str = decrypted_bytes.decode()
            return json.loads(json_str)
        except Exception as e:
            self.logger.error(f"Decryption failed: {e}")
            raise


class IPFSContextStorage:
    """IPFS-based context storage with encryption"""
    
    def __init__(
        self, 
        ipfs_api: str = "/ip4/127.0.0.1/tcp/5001",
        encryption_key: Optional[bytes] = None,
        cache_dir: Optional[str] = None
    ):
        self.ipfs_api = ipfs_api
        self.encryption = ContextEncryption(encryption_key)
        self.cache_dir = Path(cache_dir or tempfile.gettempdir()) / "nullblock_context_cache"
        self.cache_dir.mkdir(exist_ok=True)
        self.logger = logging.getLogger(__name__)
        
        # Initialize IPFS client
        try:
            self.ipfs_client = ipfshttpclient.connect(ipfs_api)
            self.logger.info(f"Connected to IPFS at {ipfs_api}")
        except Exception as e:
            self.logger.error(f"Failed to connect to IPFS: {e}")
            self.ipfs_client = None
    
    def _get_cache_path(self, wallet_address: str) -> Path:
        """Get cache file path for wallet"""
        safe_address = hashlib.sha256(wallet_address.encode()).hexdigest()[:16]
        return self.cache_dir / f"context_{safe_address}.json"
    
    async def store_context(self, context: UserContext) -> Optional[str]:
        """Store user context on IPFS and return hash"""
        try:
            # Convert to dict and encrypt
            context_dict = context.model_dump()
            encrypted_data = self.encryption.encrypt_data(context_dict)
            
            # Store in temporary file for IPFS
            with tempfile.NamedTemporaryFile(delete=False) as temp_file:
                temp_file.write(encrypted_data)
                temp_file_path = temp_file.name
            
            try:
                if self.ipfs_client:
                    # Upload to IPFS
                    result = self.ipfs_client.add(temp_file_path)
                    ipfs_hash = result['Hash']
                    
                    # Cache locally
                    cache_path = self._get_cache_path(context.wallet_address)
                    async with aiofiles.open(cache_path, 'w') as f:
                        await f.write(json.dumps({
                            "ipfs_hash": ipfs_hash,
                            "last_updated": context.updated_at.isoformat(),
                            "cached_data": context_dict
                        }, default=str))
                    
                    self.logger.info(f"Stored context for {context.wallet_address} at {ipfs_hash}")
                    return ipfs_hash
                else:
                    # IPFS not available, store only in cache
                    cache_path = self._get_cache_path(context.wallet_address)
                    async with aiofiles.open(cache_path, 'w') as f:
                        await f.write(json.dumps({
                            "ipfs_hash": None,
                            "last_updated": context.updated_at.isoformat(),
                            "cached_data": context_dict
                        }, default=str))
                    
                    self.logger.warning(f"IPFS unavailable, stored {context.wallet_address} in cache only")
                    return None
                    
            finally:
                # Clean up temp file
                if os.path.exists(temp_file_path):
                    os.unlink(temp_file_path)
                    
        except Exception as e:
            self.logger.error(f"Failed to store context for {context.wallet_address}: {e}")
            return None
    
    async def retrieve_context(self, wallet_address: str, ipfs_hash: Optional[str] = None) -> Optional[UserContext]:
        """Retrieve user context from IPFS or cache"""
        try:
            cache_path = self._get_cache_path(wallet_address)
            
            # Try to load from cache first
            if cache_path.exists():
                async with aiofiles.open(cache_path, 'r') as f:
                    cache_data = json.loads(await f.read())
                
                # If IPFS hash provided, verify it matches cache
                if ipfs_hash and cache_data.get("ipfs_hash") != ipfs_hash:
                    self.logger.info(f"Cache mismatch for {wallet_address}, fetching from IPFS")
                else:
                    # Use cached data
                    context_dict = cache_data["cached_data"]
                    return UserContext(**context_dict)
            
            # Try to fetch from IPFS if hash provided and client available
            if ipfs_hash and self.ipfs_client:
                try:
                    encrypted_data = self.ipfs_client.cat(ipfs_hash)
                    context_dict = self.encryption.decrypt_data(encrypted_data)
                    
                    # Update cache
                    async with aiofiles.open(cache_path, 'w') as f:
                        await f.write(json.dumps({
                            "ipfs_hash": ipfs_hash,
                            "last_updated": datetime.now().isoformat(),
                            "cached_data": context_dict
                        }, default=str))
                    
                    return UserContext(**context_dict)
                    
                except Exception as e:
                    self.logger.error(f"Failed to fetch from IPFS hash {ipfs_hash}: {e}")
            
            # Create new context if none found
            self.logger.info(f"Creating new context for {wallet_address}")
            return UserContext(wallet_address=wallet_address)
            
        except Exception as e:
            self.logger.error(f"Failed to retrieve context for {wallet_address}: {e}")
            return None
    
    async def update_context(self, wallet_address: str, updates: Dict[str, Any]) -> Optional[str]:
        """Update existing context with new data"""
        try:
            # Retrieve existing context
            context = await self.retrieve_context(wallet_address)
            if not context:
                self.logger.error(f"No context found for {wallet_address}")
                return None
            
            # Apply updates
            context.update(**updates)
            
            # Store updated context
            return await self.store_context(context)
            
        except Exception as e:
            self.logger.error(f"Failed to update context for {wallet_address}: {e}")
            return None
    
    def set_arbitrage_settings(self, context: UserContext, settings: ArbitrageSettings):
        """Set arbitrage-specific settings in context"""
        context.agent_settings["arbitrage"] = settings.model_dump()
        context.updated_at = datetime.now()
    
    def get_arbitrage_settings(self, context: UserContext) -> ArbitrageSettings:
        """Get arbitrage settings from context"""
        settings_dict = context.agent_settings.get("arbitrage", {})
        return ArbitrageSettings(**settings_dict)
    
    def set_defi_settings(self, context: UserContext, settings: DeFiSettings):
        """Set DeFi-specific settings in context"""
        context.agent_settings["defi"] = settings.model_dump()
        context.updated_at = datetime.now()
    
    def get_defi_settings(self, context: UserContext) -> DeFiSettings:
        """Get DeFi settings from context"""
        settings_dict = context.agent_settings.get("defi", {})
        return DeFiSettings(**settings_dict)
    
    async def cleanup_old_cache(self, days: int = 7):
        """Clean up cache files older than specified days"""
        try:
            cutoff_date = datetime.now() - timedelta(days=days)
            cleaned_count = 0
            
            for cache_file in self.cache_dir.glob("context_*.json"):
                try:
                    # Check file modification time
                    file_mtime = datetime.fromtimestamp(cache_file.stat().st_mtime)
                    if file_mtime < cutoff_date:
                        cache_file.unlink()
                        cleaned_count += 1
                except Exception as e:
                    self.logger.warning(f"Failed to clean cache file {cache_file}: {e}")
            
            if cleaned_count > 0:
                self.logger.info(f"Cleaned up {cleaned_count} old cache files")
                
        except Exception as e:
            self.logger.error(f"Cache cleanup failed: {e}")


class ContextManager:
    """High-level context management interface"""
    
    def __init__(self, storage: IPFSContextStorage):
        self.storage = storage
        self.logger = logging.getLogger(__name__)
    
    async def get_user_context(self, wallet_address: str) -> UserContext:
        """Get or create user context"""
        context = await self.storage.retrieve_context(wallet_address)
        if not context:
            # Create new context with defaults
            context = UserContext(wallet_address=wallet_address)
            
            # Set default arbitrage settings
            default_arbitrage = ArbitrageSettings()
            self.storage.set_arbitrage_settings(context, default_arbitrage)
            
            # Set default DeFi settings
            default_defi = DeFiSettings()
            self.storage.set_defi_settings(context, default_defi)
            
            # Store the new context
            await self.storage.store_context(context)
            
            self.logger.info(f"Created new context for {wallet_address}")
        
        return context
    
    async def update_trading_preferences(
        self, 
        wallet_address: str, 
        preferences: Dict[str, Any]
    ) -> bool:
        """Update user's trading preferences"""
        try:
            context = await self.get_user_context(wallet_address)
            context.trading_profile.update(preferences)
            context.updated_at = datetime.now()
            
            result = await self.storage.store_context(context)
            return result is not None
            
        except Exception as e:
            self.logger.error(f"Failed to update trading preferences for {wallet_address}: {e}")
            return False
    
    async def update_risk_profile(
        self, 
        wallet_address: str, 
        risk_settings: Dict[str, Any]
    ) -> bool:
        """Update user's risk profile"""
        try:
            context = await self.get_user_context(wallet_address)
            context.risk_profile.update(risk_settings)
            context.updated_at = datetime.now()
            
            result = await self.storage.store_context(context)
            return result is not None
            
        except Exception as e:
            self.logger.error(f"Failed to update risk profile for {wallet_address}: {e}")
            return False