use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletChallengeRequest {
    pub wallet_address: String,
    pub wallet_type: String,
    #[serde(default)]
    pub chain: Option<String>, // Optional chain type (evm, solana) - auto-detected if not provided
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletChallengeResponse {
    pub challenge_id: String,
    pub message: String,
    pub wallet_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletVerifyRequest {
    pub challenge_id: String,
    pub signature: String,
    pub wallet_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletVerifyResponse {
    pub success: bool,
    pub session_token: Option<String>,
    pub message: String,
    pub user_id: Option<String>,  // User UUID from registration
    pub registration_error: Option<String>,  // Error if registration failed
    #[serde(default)]
    pub network: Option<String>,  // Network identifier (ethereum, solana, etc.)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletListResponse {
    pub supported_wallets: Vec<WalletInfo>,
}

// New types for backend wallet interaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletDetectionRequest {
    pub user_agent: Option<String>,
    pub available_wallets: Vec<String>, // Frontend sends detected wallets
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletDetectionResponse {
    pub available_wallets: Vec<DetectedWallet>,
    pub recommended_wallet: Option<String>,
    pub install_prompts: Vec<InstallPrompt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedWallet {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub is_available: bool,
    pub install_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallPrompt {
    pub wallet_id: String,
    pub wallet_name: String,
    pub install_url: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConnectionRequest {
    pub wallet_type: String,
    pub wallet_address: String,
    pub public_key: Option<String>, // For Solana wallets
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConnectionResponse {
    pub success: bool,
    pub session_token: Option<String>,
    pub wallet_info: Option<WalletInfo>,
    pub message: String,
    pub next_step: Option<String>, // e.g., "sign_challenge", "complete"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletStatusResponse {
    pub connected: bool,
    pub wallet_type: Option<String>,
    pub wallet_address: Option<String>,
    pub session_valid: bool,
    pub session_expires_at: Option<String>,
}

// Generic wallet trait that all wallet implementations must implement
pub trait WalletProvider {
    fn get_wallet_info() -> WalletInfo;
    fn create_challenge_message(wallet_address: &str, challenge_id: &str) -> String;
    fn verify_signature(message: &str, signature: &str, wallet_address: &str) -> Result<bool, String>;
}

// Session management
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WalletSession {
    pub session_token: String,
    pub wallet_address: String,
    pub wallet_type: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

impl WalletSession {
    pub fn new(wallet_address: String, wallet_type: String, session_token: String) -> Self {
        let created_at = chrono::Utc::now();
        let expires_at = created_at + chrono::Duration::hours(24); // 24-hour session

        Self {
            session_token,
            wallet_address,
            wallet_type,
            created_at,
            expires_at,
        }
    }

    pub fn is_expired(&self) -> bool {
        chrono::Utc::now() > self.expires_at
    }
}