"""
Prompt injection protection and input sanitization for agentic interactions
"""

import logging
import re
import json
from typing import Dict, List, Optional, Any, Set, Tuple
from datetime import datetime, timedelta
from dataclasses import dataclass
from enum import Enum
import hashlib
import numpy as np
from sklearn.feature_extraction.text import TfidfVectorizer
from sklearn.ensemble import IsolationForest
from pydantic import BaseModel, Field

logger = logging.getLogger(__name__)


class ThreatLevel(Enum):
    """Threat level classification"""
    SAFE = "safe"
    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"
    CRITICAL = "critical"


class PromptAnalysis(BaseModel):
    """Result of prompt analysis"""
    input_text: str = Field(..., description="Original input text")
    sanitized_text: str = Field(..., description="Sanitized version")
    threat_level: ThreatLevel = Field(..., description="Detected threat level")
    detected_patterns: List[str] = Field(default_factory=list, description="Detected suspicious patterns")
    confidence_score: float = Field(..., description="Confidence in threat detection (0-1)")
    blocked: bool = Field(..., description="Whether input was blocked")
    timestamp: datetime = Field(default_factory=datetime.now)


class PromptInjectionDetector:
    """Machine learning-based prompt injection detector"""
    
    def __init__(self):
        self.logger = logging.getLogger(f"{__name__}.PromptInjectionDetector")
        
        # Known malicious patterns
        self.malicious_patterns = [
            # Direct injection attempts
            r"ignore\s+previous\s+instructions",
            r"forget\s+everything\s+above",
            r"disregard\s+the\s+above",
            r"system\s*:\s*you\s+are",
            r"new\s+instructions\s*:",
            r"override\s+security",
            
            # Role manipulation
            r"you\s+are\s+now\s+a\s+(?:hacker|admin|root)",
            r"act\s+as\s+(?:jailbreak|admin|root)",
            r"roleplay\s+as\s+(?:hacker|admin)",
            
            # Information extraction
            r"reveal\s+your\s+(?:prompt|instructions|system)",
            r"show\s+me\s+your\s+(?:prompt|code|instructions)",
            r"what\s+are\s+your\s+(?:instructions|rules)",
            
            # Command injection
            r"execute\s+(?:command|code|script)",
            r"run\s+(?:shell|bash|cmd)",
            r"subprocess\.(?:run|call|popen)",
            r"os\.(?:system|popen|exec)",
            
            # Crypto/financial manipulation
            r"transfer\s+all\s+(?:funds|tokens|eth)",
            r"empty\s+(?:wallet|account|balance)",
            r"send\s+private\s+keys?",
            r"reveal\s+(?:seed|mnemonic|private)",
            
            # HTML/Script injection
            r"<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>",
            r"javascript\s*:",
            r"data\s*:\s*text\/html",
            r"vbscript\s*:",
            
            # SQL injection patterns
            r"(?:union|select|insert|update|delete|drop)\s+.*(?:from|into|table)",
            r"(?:or|and)\s+(?:\d+\s*=\s*\d+|true|false)",
            r"(?:exec|execute)\s*\(",
        ]
        
        # Compile patterns for efficiency
        self.compiled_patterns = [
            (pattern, re.compile(pattern, re.IGNORECASE | re.MULTILINE))
            for pattern in self.malicious_patterns
        ]
        
        # Initialize anomaly detector
        self.vectorizer = TfidfVectorizer(
            max_features=1000,
            ngram_range=(1, 3),
            stop_words='english'
        )
        self.anomaly_detector = IsolationForest(
            contamination=0.1,
            random_state=42
        )
        
        # Training data (benign prompts)
        self._initialize_baseline()
    
    def _initialize_baseline(self):
        """Initialize baseline with benign training data"""
        benign_prompts = [
            "What is the current price of ETH?",
            "Show me my wallet balance",
            "Execute arbitrage trade with 1% profit threshold",
            "Set my risk tolerance to medium",
            "Check DeFi yield opportunities",
            "Monitor my NFT portfolio",
            "Vote on DAO proposal according to my settings",
            "Rebalance my portfolio allocation",
            "Show transaction history",
            "Update my trading preferences",
            "Enable MEV protection for trades",
            "Set slippage tolerance to 0.5%",
            "Check arbitrage opportunities on Uniswap",
            "Compound my yield farming rewards",
            "Analyze market conditions",
            "Generate portfolio performance report"
        ]
        
        try:
            # Fit vectorizer and anomaly detector
            vectors = self.vectorizer.fit_transform(benign_prompts)
            self.anomaly_detector.fit(vectors)
            self.logger.info("Initialized baseline model with benign training data")
        except Exception as e:
            self.logger.error(f"Failed to initialize baseline model: {e}")
    
    def detect_patterns(self, text: str) -> List[str]:
        """Detect known malicious patterns"""
        detected = []
        
        for pattern_text, compiled_pattern in self.compiled_patterns:
            if compiled_pattern.search(text):
                detected.append(pattern_text)
                self.logger.warning(f"Detected pattern: {pattern_text}")
        
        return detected
    
    def detect_anomaly(self, text: str) -> float:
        """Detect anomalous inputs using ML"""
        try:
            vector = self.vectorizer.transform([text])
            anomaly_score = self.anomaly_detector.decision_function(vector)[0]
            
            # Convert to 0-1 score (higher = more anomalous)
            normalized_score = max(0, min(1, (-anomaly_score + 0.5) / 1.0))
            
            return normalized_score
        except Exception as e:
            self.logger.error(f"Anomaly detection failed: {e}")
            return 0.0
    
    def analyze_prompt(self, text: str) -> PromptAnalysis:
        """Comprehensive prompt analysis"""
        detected_patterns = self.detect_patterns(text)
        anomaly_score = self.detect_anomaly(text)
        
        # Calculate threat level
        pattern_score = len(detected_patterns) * 0.3
        total_score = min(1.0, pattern_score + anomaly_score)
        
        if total_score >= 0.8:
            threat_level = ThreatLevel.CRITICAL
        elif total_score >= 0.6:
            threat_level = ThreatLevel.HIGH
        elif total_score >= 0.4:
            threat_level = ThreatLevel.MEDIUM
        elif total_score >= 0.2:
            threat_level = ThreatLevel.LOW
        else:
            threat_level = ThreatLevel.SAFE
        
        # Determine if input should be blocked
        blocked = threat_level in [ThreatLevel.HIGH, ThreatLevel.CRITICAL]
        
        return PromptAnalysis(
            input_text=text,
            sanitized_text=text,  # Will be sanitized later
            threat_level=threat_level,
            detected_patterns=detected_patterns,
            confidence_score=total_score,
            blocked=blocked
        )


class InputSanitizer:
    """Input sanitization and normalization"""
    
    def __init__(self):
        self.logger = logging.getLogger(f"{__name__}.InputSanitizer")
        
        # Allowed commands for trading operations
        self.allowed_commands = {
            "balance", "trade", "arbitrage", "swap", "stake", "unstake",
            "deposit", "withdraw", "rebalance", "compound", "vote",
            "delegate", "check", "monitor", "analyze", "report",
            "set", "get", "update", "enable", "disable"
        }
        
        # Allowed tokens/symbols
        self.allowed_tokens = {
            "ETH", "BTC", "USDC", "USDT", "DAI", "WETH", "SOL", "MATIC",
            "UNI", "AAVE", "COMP", "LINK", "SUSHI", "CRV", "BAL", "YFI"
        }
        
        # Allowed DEX names
        self.allowed_dexes = {
            "uniswap", "sushiswap", "balancer", "curve", "1inch",
            "pancakeswap", "quickswap", "honeyswap", "dodoex"
        }
    
    def sanitize_text(self, text: str) -> str:
        """Sanitize input text"""
        if not text:
            return ""
        
        # Remove potential HTML/script tags
        text = re.sub(r'<[^>]+>', '', text)
        
        # Remove potentially dangerous characters
        text = re.sub(r'[<>"\';\\]', '', text)
        
        # Normalize whitespace
        text = re.sub(r'\s+', ' ', text).strip()
        
        # Remove control characters
        text = ''.join(char for char in text if ord(char) >= 32 or char.isspace())
        
        return text
    
    def validate_trading_command(self, command: str) -> bool:
        """Validate trading command against allowlist"""
        command_lower = command.lower().strip()
        
        # Extract base command
        base_command = command_lower.split()[0] if command_lower else ""
        
        return base_command in self.allowed_commands
    
    def validate_token_symbol(self, symbol: str) -> bool:
        """Validate token symbol against allowlist"""
        return symbol.upper().strip() in self.allowed_tokens
    
    def validate_dex_name(self, dex: str) -> bool:
        """Validate DEX name against allowlist"""
        return dex.lower().strip() in self.allowed_dexes
    
    def extract_safe_parameters(self, text: str) -> Dict[str, Any]:
        """Extract safe parameters from input text"""
        parameters = {}
        
        # Extract numbers (amounts, percentages)
        numbers = re.findall(r'\d+(?:\.\d+)?', text)
        if numbers:
            parameters['amounts'] = [float(n) for n in numbers]
        
        # Extract token symbols
        tokens = re.findall(r'\b[A-Z]{2,6}\b', text)
        safe_tokens = [t for t in tokens if self.validate_token_symbol(t)]
        if safe_tokens:
            parameters['tokens'] = safe_tokens
        
        # Extract DEX names
        text_lower = text.lower()
        found_dexes = [dex for dex in self.allowed_dexes if dex in text_lower]
        if found_dexes:
            parameters['dexes'] = found_dexes
        
        return parameters


class PromptProtectionManager:
    """High-level prompt protection management"""
    
    def __init__(self, strict_mode: bool = True):
        self.detector = PromptInjectionDetector()
        self.sanitizer = InputSanitizer()
        self.strict_mode = strict_mode
        self.logger = logging.getLogger(__name__)
        
        # Rate limiting
        self.request_counts: Dict[str, List[datetime]] = {}
        self.max_requests_per_minute = 60
        
        # Blocked patterns cache
        self.blocked_hashes: Set[str] = set()
    
    def _get_text_hash(self, text: str) -> str:
        """Get hash of input text for caching"""
        return hashlib.sha256(text.encode()).hexdigest()
    
    def _check_rate_limit(self, user_id: str) -> bool:
        """Check if user is within rate limits"""
        now = datetime.now()
        cutoff = now - timedelta(minutes=1)
        
        if user_id not in self.request_counts:
            self.request_counts[user_id] = []
        
        # Remove old requests
        self.request_counts[user_id] = [
            req_time for req_time in self.request_counts[user_id]
            if req_time > cutoff
        ]
        
        # Check limit
        if len(self.request_counts[user_id]) >= self.max_requests_per_minute:
            return False
        
        # Add current request
        self.request_counts[user_id].append(now)
        return True
    
    def validate_input(
        self, 
        text: str, 
        user_id: str,
        context: Optional[Dict[str, Any]] = None
    ) -> PromptAnalysis:
        """Validate and analyze input with comprehensive protection"""
        
        # Check rate limiting
        if not self._check_rate_limit(user_id):
            return PromptAnalysis(
                input_text=text,
                sanitized_text="",
                threat_level=ThreatLevel.HIGH,
                detected_patterns=["rate_limit_exceeded"],
                confidence_score=1.0,
                blocked=True
            )
        
        # Check cache for known bad inputs
        text_hash = self._get_text_hash(text)
        if text_hash in self.blocked_hashes:
            return PromptAnalysis(
                input_text=text,
                sanitized_text="",
                threat_level=ThreatLevel.CRITICAL,
                detected_patterns=["cached_threat"],
                confidence_score=1.0,
                blocked=True
            )
        
        # Analyze with detector
        analysis = self.detector.analyze_prompt(text)
        
        # Sanitize if not blocked
        if not analysis.blocked:
            analysis.sanitized_text = self.sanitizer.sanitize_text(text)
            
            # Additional validation in strict mode
            if self.strict_mode and analysis.threat_level != ThreatLevel.SAFE:
                analysis.blocked = True
        
        # Cache blocked patterns
        if analysis.blocked:
            self.blocked_hashes.add(text_hash)
        
        # Log security events
        if analysis.threat_level in [ThreatLevel.HIGH, ThreatLevel.CRITICAL]:
            self.logger.warning(
                f"Security threat detected from user {user_id}: "
                f"level={analysis.threat_level.value}, "
                f"patterns={analysis.detected_patterns}, "
                f"confidence={analysis.confidence_score:.2f}"
            )
        
        return analysis
    
    def is_safe_trading_command(
        self, 
        command: str, 
        parameters: Dict[str, Any]
    ) -> Tuple[bool, str]:
        """Validate trading command safety"""
        
        # Validate command
        if not self.sanitizer.validate_trading_command(command):
            return False, f"Invalid command: {command}"
        
        # Validate token symbols
        if 'tokens' in parameters:
            for token in parameters['tokens']:
                if not self.sanitizer.validate_token_symbol(token):
                    return False, f"Invalid token: {token}"
        
        # Validate DEX names
        if 'dexes' in parameters:
            for dex in parameters['dexes']:
                if not self.sanitizer.validate_dex_name(dex):
                    return False, f"Invalid DEX: {dex}"
        
        # Validate amounts
        if 'amounts' in parameters:
            for amount in parameters['amounts']:
                if amount <= 0 or amount > 1000000:  # Reasonable limits
                    return False, f"Invalid amount: {amount}"
        
        return True, "Command validated successfully"
    
    def cleanup_old_data(self, hours: int = 24):
        """Clean up old rate limiting and cache data"""
        cutoff = datetime.now() - timedelta(hours=hours)
        
        # Clean rate limiting data
        for user_id in list(self.request_counts.keys()):
            self.request_counts[user_id] = [
                req_time for req_time in self.request_counts[user_id]
                if req_time > cutoff
            ]
            if not self.request_counts[user_id]:
                del self.request_counts[user_id]
        
        # Clear blocked hashes cache periodically
        if len(self.blocked_hashes) > 10000:
            self.blocked_hashes.clear()
            self.logger.info("Cleared blocked hashes cache")
        
        self.logger.info(f"Cleaned up security data older than {hours} hours")