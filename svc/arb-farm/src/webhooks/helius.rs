use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone)]
pub struct HeliusWebhookClient {
    client: reqwest::Client,
    api_url: String,
    api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub webhook_url: String,
    pub webhook_type: WebhookType,
    pub transaction_types: Vec<TransactionType>,
    pub account_addresses: Vec<String>,
    pub auth_header: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum WebhookType {
    Enhanced,
    EnhancedDevnet,
    Raw,
    RawDevnet,
    Discord,
    DiscordDevnet,
}

impl Default for WebhookType {
    fn default() -> Self {
        Self::Enhanced
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionType {
    Any,
    NftSale,
    NftListing,
    NftBid,
    NftBidCancelled,
    NftMint,
    NftCancelListing,
    NftAuctionCreated,
    NftAuctionUpdated,
    NftAuctionCancelled,
    NftParticipationReward,
    NftMintRejected,
    Transfer,
    Burn,
    BurnNft,
    Swap,
    CompressedNftMint,
    CompressedNftTransfer,
    CompressedNftBurn,
    Unknown,
}

impl Default for TransactionType {
    fn default() -> Self {
        Self::Swap
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookRegistration {
    pub webhook_id: String,
    pub wallet_address: String,
    pub webhook_url: String,
    pub webhook_type: WebhookType,
    pub transaction_types: Vec<TransactionType>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CreateWebhookRequest {
    #[serde(rename = "webhookURL")]
    webhook_url: String,
    #[serde(rename = "webhookType")]
    webhook_type: WebhookType,
    #[serde(rename = "transactionTypes")]
    transaction_types: Vec<TransactionType>,
    #[serde(rename = "accountAddresses")]
    account_addresses: Vec<String>,
    #[serde(rename = "authHeader", skip_serializing_if = "Option::is_none")]
    auth_header: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct CreateWebhookResponse {
    #[serde(rename = "webhookID")]
    webhook_id: String,
}

#[derive(Debug, Clone, Deserialize)]
struct GetAllWebhooksResponse {
    webhooks: Vec<HeliusWebhook>,
}

#[derive(Debug, Clone, Deserialize)]
struct HeliusWebhook {
    #[serde(rename = "webhookID")]
    webhook_id: String,
    #[serde(rename = "webhookURL")]
    webhook_url: String,
    #[serde(rename = "webhookType")]
    webhook_type: String,
    #[serde(rename = "transactionTypes")]
    transaction_types: Vec<String>,
    #[serde(rename = "accountAddresses")]
    account_addresses: Vec<String>,
}

impl HeliusWebhookClient {
    pub fn new(api_url: String, api_key: Option<String>) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            api_url,
            api_key,
        }
    }

    pub fn is_configured(&self) -> bool {
        self.api_key.is_some()
    }

    pub async fn create_webhook(&self, config: &WebhookConfig) -> AppResult<WebhookRegistration> {
        let api_key = self
            .api_key
            .as_ref()
            .ok_or_else(|| AppError::Configuration("Helius API key not configured".to_string()))?;

        let request = CreateWebhookRequest {
            webhook_url: config.webhook_url.clone(),
            webhook_type: config.webhook_type,
            transaction_types: config.transaction_types.clone(),
            account_addresses: config.account_addresses.clone(),
            auth_header: config.auth_header.clone(),
        };

        let url = format!("{}/webhooks?api-key={}", self.api_url, api_key);

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Helius webhook creation failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ExternalApi(format!(
                "Helius webhook error: {}",
                error_text
            )));
        }

        let result: CreateWebhookResponse = response.json().await.map_err(|e| {
            AppError::ExternalApi(format!("Failed to parse Helius response: {}", e))
        })?;

        Ok(WebhookRegistration {
            webhook_id: result.webhook_id,
            wallet_address: config
                .account_addresses
                .first()
                .cloned()
                .unwrap_or_default(),
            webhook_url: config.webhook_url.clone(),
            webhook_type: config.webhook_type,
            transaction_types: config.transaction_types.clone(),
            created_at: chrono::Utc::now(),
            is_active: true,
        })
    }

    pub async fn delete_webhook(&self, webhook_id: &str) -> AppResult<()> {
        let api_key = self
            .api_key
            .as_ref()
            .ok_or_else(|| AppError::Configuration("Helius API key not configured".to_string()))?;

        let url = format!(
            "{}/webhooks/{}?api-key={}",
            self.api_url, webhook_id, api_key
        );

        let response =
            self.client.delete(&url).send().await.map_err(|e| {
                AppError::ExternalApi(format!("Helius webhook deletion failed: {}", e))
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ExternalApi(format!(
                "Helius webhook deletion error: {}",
                error_text
            )));
        }

        Ok(())
    }

    pub async fn list_webhooks(&self) -> AppResult<Vec<WebhookRegistration>> {
        let api_key = self
            .api_key
            .as_ref()
            .ok_or_else(|| AppError::Configuration("Helius API key not configured".to_string()))?;

        let url = format!("{}/webhooks?api-key={}", self.api_url, api_key);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Helius webhook list failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ExternalApi(format!(
                "Helius webhook list error: {}",
                error_text
            )));
        }

        let result: Vec<HeliusWebhook> = response.json().await.map_err(|e| {
            AppError::ExternalApi(format!("Failed to parse Helius response: {}", e))
        })?;

        let registrations = result
            .into_iter()
            .map(|w| WebhookRegistration {
                webhook_id: w.webhook_id,
                wallet_address: w.account_addresses.first().cloned().unwrap_or_default(),
                webhook_url: w.webhook_url,
                webhook_type: WebhookType::Enhanced,
                transaction_types: vec![TransactionType::Swap],
                created_at: chrono::Utc::now(),
                is_active: true,
            })
            .collect();

        Ok(registrations)
    }

    pub async fn find_webhook_for_address(&self, address: &str) -> AppResult<Option<String>> {
        let webhooks = self.list_webhooks().await?;

        for webhook in webhooks {
            if webhook.wallet_address == address {
                return Ok(Some(webhook.webhook_id));
            }
        }

        // Also check the raw webhook list for multiple addresses per webhook
        let api_key = self
            .api_key
            .as_ref()
            .ok_or_else(|| AppError::Configuration("Helius API key not configured".to_string()))?;

        let url = format!("{}/webhooks?api-key={}", self.api_url, api_key);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Helius webhook list failed: {}", e)))?;

        if !response.status().is_success() {
            return Ok(None);
        }

        let webhooks: Vec<HeliusWebhook> = response.json().await.unwrap_or_default();

        for webhook in webhooks {
            if webhook.account_addresses.contains(&address.to_string()) {
                return Ok(Some(webhook.webhook_id));
            }
        }

        Ok(None)
    }

    pub async fn add_addresses_to_webhook(
        &self,
        webhook_id: &str,
        addresses: Vec<String>,
    ) -> AppResult<()> {
        let api_key = self
            .api_key
            .as_ref()
            .ok_or_else(|| AppError::Configuration("Helius API key not configured".to_string()))?;

        let url = format!(
            "{}/webhooks/{}?api-key={}",
            self.api_url, webhook_id, api_key
        );

        let request = serde_json::json!({
            "accountAddresses": addresses
        });

        let response = self
            .client
            .put(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Helius webhook update failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ExternalApi(format!(
                "Helius webhook update error: {}",
                error_text
            )));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeliusWebhookPayload {
    #[serde(default)]
    pub webhook_id: Option<String>,
    pub events: Vec<EnhancedTransactionEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnhancedTransactionEvent {
    pub account_data: Vec<AccountData>,
    pub description: String,
    pub events: EventsData,
    pub fee: u64,
    pub fee_payer: String,
    pub instructions: Vec<InstructionData>,
    pub native_transfers: Vec<NativeTransfer>,
    pub signature: String,
    pub slot: u64,
    pub source: String,
    pub timestamp: i64,
    pub token_transfers: Vec<TokenTransfer>,
    #[serde(rename = "type")]
    pub transaction_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountData {
    pub account: String,
    pub native_balance_change: i64,
    pub token_balance_changes: Vec<TokenBalanceChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenBalanceChange {
    pub mint: String,
    pub raw_token_amount: RawTokenAmount,
    pub token_account: String,
    pub user_account: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawTokenAmount {
    pub decimals: u8,
    pub token_amount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EventsData {
    #[serde(default)]
    pub swap: Option<SwapEvent>,
    #[serde(default)]
    pub nft: Option<serde_json::Value>,
    #[serde(default)]
    pub compressed: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapEvent {
    pub native_input: Option<NativeAmount>,
    pub native_output: Option<NativeAmount>,
    pub token_inputs: Vec<TokenAmount>,
    pub token_outputs: Vec<TokenAmount>,
    pub token_fees: Vec<TokenAmount>,
    pub native_fees: Vec<NativeAmount>,
    pub inner_swaps: Vec<InnerSwap>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeAmount {
    pub account: String,
    pub amount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenAmount {
    pub user_account: String,
    pub token_account: String,
    pub mint: String,
    pub raw_token_amount: RawTokenAmount,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InnerSwap {
    pub program_info: ProgramInfo,
    pub token_inputs: Vec<TokenAmount>,
    pub token_outputs: Vec<TokenAmount>,
    pub native_input: Option<NativeAmount>,
    pub native_output: Option<NativeAmount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramInfo {
    pub source: String,
    pub account: String,
    #[serde(rename = "programName")]
    pub program_name: String,
    #[serde(rename = "instructionName")]
    pub instruction_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstructionData {
    pub accounts: Vec<String>,
    pub data: String,
    pub inner_instructions: Vec<InnerInstruction>,
    pub program_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InnerInstruction {
    pub accounts: Vec<String>,
    pub data: String,
    pub program_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeTransfer {
    pub amount: u64,
    pub from_user_account: String,
    pub to_user_account: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenTransfer {
    pub from_token_account: String,
    pub from_user_account: String,
    pub mint: String,
    pub to_token_account: String,
    pub to_user_account: String,
    pub token_amount: f64,
    pub token_standard: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_config_serialization() {
        let config = WebhookConfig {
            webhook_url: "https://example.com/webhook".to_string(),
            webhook_type: WebhookType::Enhanced,
            transaction_types: vec![TransactionType::Swap],
            account_addresses: vec!["address1".to_string()],
            auth_header: Some("secret".to_string()),
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("https://example.com/webhook"));
    }

    #[test]
    fn test_transaction_type_serialization() {
        let tx_type = TransactionType::Swap;
        let json = serde_json::to_string(&tx_type).unwrap();
        assert_eq!(json, "\"SWAP\"");
    }
}
