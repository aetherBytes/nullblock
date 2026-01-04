use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ChainType {
    Evm,
    Solana,
}

impl std::fmt::Display for ChainType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChainType::Evm => write!(f, "evm"),
            ChainType::Solana => write!(f, "solana"),
        }
    }
}

impl std::str::FromStr for ChainType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "evm" | "ethereum" => Ok(ChainType::Evm),
            "solana" | "sol" => Ok(ChainType::Solana),
            _ => Err(format!("Unknown chain type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub supported_chains: Vec<ChainType>,
    pub install_url: String,
}

#[derive(Debug, Clone)]
pub struct ChallengeContext {
    pub wallet_address: String,
    pub challenge_id: String,
    pub chain: ChainType,
    pub timestamp: i64,
}

#[derive(Debug, Clone)]
pub enum WalletError {
    InvalidSignature(String),
    InvalidAddress(String),
    UnsupportedChain(ChainType),
    VerificationFailed(String),
}

impl std::fmt::Display for WalletError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WalletError::InvalidSignature(msg) => write!(f, "Invalid signature: {}", msg),
            WalletError::InvalidAddress(msg) => write!(f, "Invalid address: {}", msg),
            WalletError::UnsupportedChain(chain) => write!(f, "Unsupported chain: {:?}", chain),
            WalletError::VerificationFailed(msg) => write!(f, "Verification failed: {}", msg),
        }
    }
}

impl std::error::Error for WalletError {}

pub trait WalletAdapter: Send + Sync + Debug {
    fn id(&self) -> &'static str;

    fn info(&self) -> WalletInfo;

    fn supported_chains(&self) -> &[ChainType];

    fn supports_chain(&self, chain: &ChainType) -> bool {
        self.supported_chains().contains(chain)
    }

    fn validate_address(&self, address: &str, chain: &ChainType) -> bool;

    fn create_challenge_message(&self, context: &ChallengeContext) -> String;

    fn verify_signature(
        &self,
        message: &str,
        signature: &str,
        wallet_address: &str,
        chain: &ChainType,
    ) -> Result<bool, WalletError>;

    fn detect_chain_from_address(&self, address: &str) -> Option<ChainType>;

    fn get_network_for_chain(&self, chain: &ChainType) -> &'static str;
}
