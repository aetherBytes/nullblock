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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletListResponse {
    pub supported_wallets: Vec<WalletInfo>,
}

// Generic wallet trait that all wallet implementations must implement
pub trait WalletProvider {
    fn get_wallet_info() -> WalletInfo;
    fn create_challenge_message(wallet_address: &str, challenge_id: &str) -> String;
    fn verify_signature(message: &str, signature: &str, wallet_address: &str) -> Result<bool, String>;
}

// Session management
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