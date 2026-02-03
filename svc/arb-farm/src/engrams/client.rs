use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEngramRequest {
    pub wallet_address: String,
    pub engram_type: String,
    pub key: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_public: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Engram {
    pub id: String,
    pub wallet_address: String,
    pub engram_type: String,
    pub key: String,
    pub content: String,
    #[serde(default)]
    pub metadata: Option<Value>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub is_public: bool,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub version: Option<i32>,
    #[serde(default)]
    pub parent_id: Option<String>,
    #[serde(default)]
    pub lineage_root_id: Option<String>,
    #[serde(default)]
    pub is_mintable: Option<bool>,
    #[serde(default)]
    pub nft_token_id: Option<String>,
    #[serde(default)]
    pub price_mon: Option<String>,
    #[serde(default)]
    pub royalty_percent: Option<i32>,
    #[serde(default)]
    pub priority: Option<i32>,
    #[serde(default)]
    pub ttl_seconds: Option<i64>,
    #[serde(default)]
    pub created_by: Option<String>,
    #[serde(default)]
    pub accessed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngramResponse {
    pub success: bool,
    pub data: Option<Engram>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngramListResponse {
    pub success: bool,
    pub data: Option<Vec<Engram>>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeResultEngram {
    pub edge_id: String,
    pub strategy_id: Option<String>,
    pub profit_lamports: i64,
    pub execution_time_ms: u64,
    pub tx_signature: String,
    pub input_mint: String,
    pub output_mint: String,
    pub input_amount: u64,
    pub output_amount: u64,
    pub route_path: Vec<String>,
    pub venue: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternEngram {
    pub pattern_type: String,
    pub venue: String,
    pub token_pair: Option<(String, String)>,
    pub avg_profit_bps: f64,
    pub occurrence_count: u32,
    pub success_rate: f64,
    pub last_seen: String,
    pub characteristics: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvoidanceEngram {
    pub entity_type: String,
    pub address: String,
    pub reason: String,
    pub severity: String,
    pub detected_at: String,
    pub evidence: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStateEngram {
    pub scanner_active: bool,
    pub active_strategies: u32,
    pub pending_edges: u32,
    pub daily_trades: u32,
    pub daily_profit_lamports: i64,
    pub swarm_health: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallet_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engram_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
}

pub struct EngramsClient {
    base_url: String,
    http_client: Client,
    default_wallet: Option<String>,
}

impl EngramsClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            http_client: Client::new(),
            default_wallet: None,
        }
    }

    pub fn with_default_wallet(mut self, wallet: String) -> Self {
        self.default_wallet = Some(wallet);
        self
    }

    pub fn is_configured(&self) -> bool {
        !self.base_url.is_empty()
    }

    fn get_wallet(&self, wallet: Option<&str>) -> Option<String> {
        wallet
            .map(|w| w.to_string())
            .or(self.default_wallet.clone())
    }

    pub async fn create_engram(&self, request: CreateEngramRequest) -> Result<Engram, String> {
        if !self.is_configured() {
            return Err("Engrams service not configured".to_string());
        }

        let url = format!("{}/engrams", self.base_url);
        debug!("Creating engram: {} at {}", request.key, url);

        match self
            .http_client
            .post(&url)
            .json(&request)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<EngramResponse>().await {
                        Ok(resp) => {
                            if resp.success {
                                if let Some(engram) = resp.data {
                                    info!("Created engram: {} ({})", engram.key, engram.id);
                                    Ok(engram)
                                } else {
                                    Err("Engrams service returned success but no data".to_string())
                                }
                            } else {
                                let err_msg =
                                    resp.error.unwrap_or_else(|| "Unknown error".to_string());
                                error!("Engrams service error: {}", err_msg);
                                Err(err_msg)
                            }
                        }
                        Err(e) => {
                            error!("Failed to parse engram response: {}", e);
                            Err(format!("Failed to parse response: {}", e))
                        }
                    }
                } else {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    error!("Engrams service error {}: {}", status, body);
                    Err(format!("Engrams service error {}: {}", status, body))
                }
            }
            Err(e) => {
                error!("Failed to connect to Engrams service: {}", e);
                Err(format!("Connection error: {}", e))
            }
        }
    }

    pub async fn get_engrams_by_wallet(&self, wallet: &str) -> Result<Vec<Engram>, String> {
        if !self.is_configured() {
            return Err("Engrams service not configured".to_string());
        }

        let url = format!("{}/engrams/wallet/{}", self.base_url, wallet);
        debug!("Fetching engrams for wallet: {}", wallet);

        match self
            .http_client
            .get(&url)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<EngramListResponse>().await {
                        Ok(resp) => {
                            if resp.success {
                                let engrams = resp.data.unwrap_or_default();
                                debug!("Fetched {} engrams for wallet {}", engrams.len(), wallet);
                                Ok(engrams)
                            } else {
                                let err_msg =
                                    resp.error.unwrap_or_else(|| "Unknown error".to_string());
                                error!("Engrams wallet fetch error: {}", err_msg);
                                Err(err_msg)
                            }
                        }
                        Err(e) => {
                            error!("Failed to parse engrams response: {}", e);
                            Err(format!("Failed to parse response: {}", e))
                        }
                    }
                } else {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    Err(format!("Engrams service error {}: {}", status, body))
                }
            }
            Err(e) => {
                error!("Failed to connect to Engrams service: {}", e);
                Err(format!("Connection error: {}", e))
            }
        }
    }

    pub async fn search_engrams(&self, request: SearchRequest) -> Result<Vec<Engram>, String> {
        if !self.is_configured() {
            return Err("Engrams service not configured".to_string());
        }

        let url = format!("{}/engrams/search", self.base_url);
        debug!("Searching engrams with query: {:?}", request.query);

        match self
            .http_client
            .post(&url)
            .json(&request)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<EngramListResponse>().await {
                        Ok(resp) => {
                            if resp.success {
                                let engrams = resp.data.unwrap_or_default();
                                debug!("Found {} engrams matching query", engrams.len());
                                Ok(engrams)
                            } else {
                                let err_msg =
                                    resp.error.unwrap_or_else(|| "Unknown error".to_string());
                                error!("Engrams search error: {}", err_msg);
                                Err(err_msg)
                            }
                        }
                        Err(e) => {
                            error!("Failed to parse search response: {}", e);
                            Err(format!("Failed to parse response: {}", e))
                        }
                    }
                } else {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    Err(format!("Engrams service error {}: {}", status, body))
                }
            }
            Err(e) => {
                error!("Failed to connect to Engrams service: {}", e);
                Err(format!("Connection error: {}", e))
            }
        }
    }

    pub async fn save_trade_result(
        &self,
        wallet: &str,
        trade: TradeResultEngram,
    ) -> Result<Engram, String> {
        let key = format!("arb.trade.{}", trade.tx_signature);
        let content = serde_json::to_string(&trade)
            .map_err(|e| format!("Failed to serialize trade: {}", e))?;

        let metadata = serde_json::json!({
            "type": "trade_result",
            "profitable": trade.profit_lamports > 0,
            "venue": trade.venue,
            "execution_time_ms": trade.execution_time_ms,
        });

        let request = CreateEngramRequest {
            wallet_address: wallet.to_string(),
            engram_type: "strategy".to_string(),
            key,
            content,
            metadata: Some(metadata),
            tags: Some(vec![
                "arb".to_string(),
                "trade".to_string(),
                trade.venue.clone(),
                if trade.profit_lamports > 0 {
                    "profit".to_string()
                } else {
                    "loss".to_string()
                },
            ]),
            is_public: Some(false),
        };

        self.create_engram(request).await
    }

    pub async fn save_pattern(
        &self,
        wallet: &str,
        pattern: PatternEngram,
    ) -> Result<Engram, String> {
        let key = format!("arb.pattern.{}.{}", pattern.pattern_type, pattern.venue);
        let content = serde_json::to_string(&pattern)
            .map_err(|e| format!("Failed to serialize pattern: {}", e))?;

        let metadata = serde_json::json!({
            "type": "pattern",
            "pattern_type": pattern.pattern_type,
            "venue": pattern.venue,
            "success_rate": pattern.success_rate,
            "occurrence_count": pattern.occurrence_count,
        });

        let request = CreateEngramRequest {
            wallet_address: wallet.to_string(),
            engram_type: "strategy".to_string(),
            key,
            content,
            metadata: Some(metadata),
            tags: Some(vec![
                "arb".to_string(),
                "pattern".to_string(),
                pattern.pattern_type.clone(),
                pattern.venue.clone(),
            ]),
            is_public: Some(false),
        };

        self.create_engram(request).await
    }

    pub async fn save_avoidance(
        &self,
        wallet: &str,
        avoidance: AvoidanceEngram,
    ) -> Result<Engram, String> {
        let key = format!("arb.avoid.{}.{}", avoidance.entity_type, avoidance.address);
        let content = serde_json::to_string(&avoidance)
            .map_err(|e| format!("Failed to serialize avoidance: {}", e))?;

        let metadata = serde_json::json!({
            "type": "avoidance",
            "entity_type": avoidance.entity_type,
            "severity": avoidance.severity,
            "reason": avoidance.reason,
        });

        let request = CreateEngramRequest {
            wallet_address: wallet.to_string(),
            engram_type: "compliance".to_string(),
            key,
            content,
            metadata: Some(metadata),
            tags: Some(vec![
                "arb".to_string(),
                "avoid".to_string(),
                avoidance.entity_type.clone(),
                avoidance.severity.clone(),
            ]),
            is_public: Some(false),
        };

        self.create_engram(request).await
    }

    pub async fn save_agent_state(
        &self,
        wallet: &str,
        state: AgentStateEngram,
    ) -> Result<Engram, String> {
        let key = format!("arb.state.{}", state.timestamp);
        let content = serde_json::to_string(&state)
            .map_err(|e| format!("Failed to serialize state: {}", e))?;

        let metadata = serde_json::json!({
            "type": "agent_state",
            "scanner_active": state.scanner_active,
            "swarm_health": state.swarm_health,
            "daily_trades": state.daily_trades,
        });

        let request = CreateEngramRequest {
            wallet_address: wallet.to_string(),
            engram_type: "knowledge".to_string(),
            key,
            content,
            metadata: Some(metadata),
            tags: Some(vec![
                "arb".to_string(),
                "state".to_string(),
                "snapshot".to_string(),
            ]),
            is_public: Some(false),
        };

        self.create_engram(request).await
    }

    pub async fn check_avoidance(&self, wallet: &str, entity_type: &str, address: &str) -> bool {
        let search = SearchRequest {
            wallet_address: Some(wallet.to_string()),
            engram_type: Some("compliance".to_string()),
            query: Some(format!("arb.avoid.{}.{}", entity_type, address)),
            tags: Some(vec!["avoid".to_string()]),
            limit: Some(1),
            offset: None,
        };

        match self.search_engrams(search).await {
            Ok(results) => !results.is_empty(),
            Err(e) => {
                warn!("Failed to check avoidance: {}", e);
                false
            }
        }
    }

    pub async fn get_patterns(&self, wallet: &str) -> Result<Vec<PatternEngram>, String> {
        let search = SearchRequest {
            wallet_address: Some(wallet.to_string()),
            engram_type: Some("strategy".to_string()),
            query: None,
            tags: Some(vec!["pattern".to_string()]),
            limit: Some(100),
            offset: None,
        };

        let engrams = self.search_engrams(search).await?;

        let patterns: Vec<PatternEngram> = engrams
            .into_iter()
            .filter_map(|e| serde_json::from_str(&e.content).ok())
            .collect();

        Ok(patterns)
    }

    pub async fn save_strategy(
        &self,
        wallet: &str,
        strategy_id: &str,
        strategy_name: &str,
        strategy_type: &str,
        venue_types: &[String],
        execution_mode: &str,
        risk_params: &Value,
    ) -> Result<Engram, String> {
        self.save_strategy_full(
            wallet,
            strategy_id,
            strategy_name,
            strategy_type,
            venue_types,
            execution_mode,
            risk_params,
            true,
        )
        .await
    }

    pub async fn get_engram_by_wallet_key(
        &self,
        wallet: &str,
        key: &str,
    ) -> Result<Option<Engram>, String> {
        if !self.is_configured() {
            return Err("Engrams service not configured".to_string());
        }

        let url = format!("{}/engrams/wallet/{}/{}", self.base_url, wallet, key);
        debug!("Fetching engram by wallet/key: {}/{}", wallet, key);

        match self
            .http_client
            .get(&url)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<EngramResponse>().await {
                        Ok(resp) => {
                            if resp.success && resp.data.is_some() {
                                return Ok(resp.data);
                            }
                            // Direct lookup failed - try search as fallback
                            debug!(
                                "Direct lookup failed for {}/{}, trying search fallback",
                                wallet, key
                            );
                        }
                        Err(_) => {}
                    }
                }

                // Fallback: search for engram by key
                let search = SearchRequest {
                    wallet_address: Some(wallet.to_string()),
                    engram_type: None,
                    query: None,
                    tags: None,
                    limit: Some(100),
                    offset: None,
                };

                if let Ok(engrams) = self.search_engrams(search).await {
                    for engram in engrams {
                        if engram.key == key {
                            debug!("Found engram via search fallback: {} ({})", key, engram.id);
                            return Ok(Some(engram));
                        }
                    }
                }

                Ok(None) // Not found
            }
            Err(e) => {
                error!("Failed to connect to Engrams service: {}", e);
                Err(format!("Connection error: {}", e))
            }
        }
    }

    pub async fn update_engram(
        &self,
        id: &str,
        content: &str,
        tags: Option<Vec<String>>,
    ) -> Result<Engram, String> {
        if !self.is_configured() {
            return Err("Engrams service not configured".to_string());
        }

        let url = format!("{}/engrams/{}", self.base_url, id);
        debug!("Updating engram: {} at {}", id, url);

        let body = serde_json::json!({
            "content": content,
            "tags": tags,
        });

        match self
            .http_client
            .put(&url)
            .json(&body)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<EngramResponse>().await {
                        Ok(resp) => {
                            if resp.success {
                                if let Some(engram) = resp.data {
                                    info!("Updated engram: {} ({})", engram.key, engram.id);
                                    Ok(engram)
                                } else {
                                    Err("Engrams service returned success but no data".to_string())
                                }
                            } else {
                                let err_msg =
                                    resp.error.unwrap_or_else(|| "Unknown error".to_string());
                                error!("Engrams update error: {}", err_msg);
                                Err(err_msg)
                            }
                        }
                        Err(e) => {
                            error!("Failed to parse engram response: {}", e);
                            Err(format!("Failed to parse response: {}", e))
                        }
                    }
                } else {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    error!("Engrams service error {}: {}", status, body);
                    Err(format!("Engrams service error {}: {}", status, body))
                }
            }
            Err(e) => {
                error!("Failed to connect to Engrams service: {}", e);
                Err(format!("Connection error: {}", e))
            }
        }
    }

    pub async fn upsert_engram(&self, request: CreateEngramRequest) -> Result<Engram, String> {
        // Check if engram exists by wallet/key
        if let Ok(Some(existing)) = self
            .get_engram_by_wallet_key(&request.wallet_address, &request.key)
            .await
        {
            // Update existing
            return self
                .update_engram(&existing.id, &request.content, request.tags)
                .await;
        }

        // Also check with "default" wallet for legacy data
        if let Ok(Some(existing)) = self.get_engram_by_wallet_key("default", &request.key).await {
            // Update existing legacy engram
            return self
                .update_engram(&existing.id, &request.content, request.tags)
                .await;
        }

        // Try to create new, but handle duplicate key gracefully
        match self.create_engram(request.clone()).await {
            Ok(engram) => Ok(engram),
            Err(e) if e.contains("duplicate key") || e.contains("unique constraint") => {
                // Engram exists but we couldn't find it - try one more time to fetch and update
                warn!(
                    "Engram {} already exists but wasn't found initially - retrying fetch",
                    request.key
                );

                // Retry fetching after a small delay (race condition mitigation)
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;

                if let Ok(Some(existing)) = self
                    .get_engram_by_wallet_key(&request.wallet_address, &request.key)
                    .await
                {
                    return self
                        .update_engram(&existing.id, &request.content, request.tags)
                        .await;
                }

                // Still can't find it - return a synthetic success
                warn!(
                    "Engram {} still not found after duplicate key error - treating as success",
                    request.key
                );
                Ok(Engram {
                    id: String::new(),
                    wallet_address: request.wallet_address,
                    engram_type: request.engram_type,
                    key: request.key,
                    content: request.content,
                    metadata: request.metadata,
                    tags: request.tags.unwrap_or_default(),
                    is_public: request.is_public.unwrap_or(false),
                    created_at: chrono::Utc::now().to_rfc3339(),
                    updated_at: chrono::Utc::now().to_rfc3339(),
                    summary: None,
                    version: None,
                    parent_id: None,
                    lineage_root_id: None,
                    is_mintable: None,
                    nft_token_id: None,
                    price_mon: None,
                    royalty_percent: None,
                    priority: None,
                    ttl_seconds: None,
                    created_by: None,
                    accessed_at: None,
                })
            }
            Err(e) => Err(e),
        }
    }

    pub async fn save_strategy_full(
        &self,
        wallet: &str,
        strategy_id: &str,
        strategy_name: &str,
        strategy_type: &str,
        venue_types: &[String],
        execution_mode: &str,
        risk_params: &Value,
        is_active: bool,
    ) -> Result<Engram, String> {
        let key = format!("arb.strategy.{}", strategy_id);
        let content = serde_json::json!({
            "strategy_id": strategy_id,
            "name": strategy_name,
            "strategy_type": strategy_type,
            "venue_types": venue_types,
            "execution_mode": execution_mode,
            "risk_params": risk_params,
            "is_active": is_active,
            "created_at": chrono::Utc::now().to_rfc3339(),
            "updated_at": chrono::Utc::now().to_rfc3339(),
        });

        let metadata = serde_json::json!({
            "type": "strategy_config",
            "strategy_type": strategy_type,
            "execution_mode": execution_mode,
            "venue_count": venue_types.len(),
        });

        let request = CreateEngramRequest {
            wallet_address: wallet.to_string(),
            engram_type: "strategy".to_string(),
            key,
            content: serde_json::to_string(&content).unwrap_or_default(),
            metadata: Some(metadata),
            tags: Some(vec![
                "arb".to_string(),
                "strategy".to_string(),
                strategy_type.to_string(),
                execution_mode.to_string(),
            ]),
            is_public: Some(true), // Strategies can be shared/minted
        };

        self.upsert_engram(request).await
    }

    pub async fn save_edge(
        &self,
        wallet: &str,
        edge_id: &str,
        edge_type: &str,
        token_mint: Option<&str>,
        estimated_profit_lamports: i64,
        risk_score: i32,
        signal_data: &Value,
    ) -> Result<Engram, String> {
        let key = format!("arb.edge.{}", edge_id);
        let content = serde_json::json!({
            "edge_id": edge_id,
            "edge_type": edge_type,
            "token_mint": token_mint,
            "estimated_profit_lamports": estimated_profit_lamports,
            "risk_score": risk_score,
            "signal_data": signal_data,
            "detected_at": chrono::Utc::now().to_rfc3339(),
        });

        let metadata = serde_json::json!({
            "type": "edge_opportunity",
            "edge_type": edge_type,
            "profitable": estimated_profit_lamports > 0,
            "risk_level": if risk_score < 30 { "low" } else if risk_score < 60 { "medium" } else { "high" },
        });

        let mut tags = vec!["arb".to_string(), "edge".to_string(), edge_type.to_string()];
        if let Some(mint) = token_mint {
            tags.push(mint.chars().take(8).collect::<String>());
        }

        let request = CreateEngramRequest {
            wallet_address: wallet.to_string(),
            engram_type: "knowledge".to_string(),
            key,
            content: serde_json::to_string(&content).unwrap_or_default(),
            metadata: Some(metadata),
            tags: Some(tags),
            is_public: Some(false),
        };

        self.create_engram(request).await
    }

    pub async fn save_kol_discovery(
        &self,
        wallet: &str,
        kol_address: &str,
        display_name: Option<&str>,
        trust_score: f64,
        win_rate: f64,
        total_trades: i32,
        discovery_source: &str,
    ) -> Result<Engram, String> {
        let key = format!("arb.kol.{}", kol_address);
        let content = serde_json::json!({
            "kol_address": kol_address,
            "display_name": display_name,
            "trust_score": trust_score,
            "win_rate": win_rate,
            "total_trades": total_trades,
            "discovery_source": discovery_source,
            "discovered_at": chrono::Utc::now().to_rfc3339(),
        });

        let metadata = serde_json::json!({
            "type": "kol_profile",
            "trust_tier": if trust_score >= 80.0 { "elite" } else if trust_score >= 60.0 { "trusted" } else { "emerging" },
            "discovery_source": discovery_source,
        });

        let request = CreateEngramRequest {
            wallet_address: wallet.to_string(),
            engram_type: "knowledge".to_string(),
            key,
            content: serde_json::to_string(&content).unwrap_or_default(),
            metadata: Some(metadata),
            tags: Some(vec![
                "arb".to_string(),
                "kol".to_string(),
                discovery_source.to_string(),
                if trust_score >= 70.0 {
                    "high_trust".to_string()
                } else {
                    "tracking".to_string()
                },
            ]),
            is_public: Some(true), // KOL profiles can be shared
        };

        self.create_engram(request).await
    }

    pub async fn get_discovered_kols(
        &self,
        wallet: &str,
    ) -> Result<Vec<KolDiscoveryEngram>, String> {
        let search = SearchRequest {
            wallet_address: Some(wallet.to_string()),
            engram_type: Some("knowledge".to_string()),
            query: None,
            tags: Some(vec!["kol".to_string()]),
            limit: Some(100),
            offset: None,
        };

        let engrams = self.search_engrams(search).await?;

        let kols: Vec<KolDiscoveryEngram> = engrams
            .into_iter()
            .filter_map(|e| serde_json::from_str(&e.content).ok())
            .collect();

        info!("Restored {} KOL discoveries from engrams", kols.len());
        Ok(kols)
    }

    pub async fn get_saved_strategies(&self, wallet: &str) -> Result<Vec<StrategyEngram>, String> {
        let search = SearchRequest {
            wallet_address: Some(wallet.to_string()),
            engram_type: Some("strategy".to_string()),
            query: None,
            tags: Some(vec!["strategy".to_string()]),
            limit: Some(50),
            offset: None,
        };

        let engrams = self.search_engrams(search).await?;

        let strategies: Vec<StrategyEngram> = engrams
            .into_iter()
            .filter_map(|e| serde_json::from_str(&e.content).ok())
            .collect();

        info!("Restored {} strategies from engrams", strategies.len());
        Ok(strategies)
    }

    pub async fn get_avoidances(&self, wallet: &str) -> Result<Vec<AvoidanceEngram>, String> {
        let search = SearchRequest {
            wallet_address: Some(wallet.to_string()),
            engram_type: Some("compliance".to_string()),
            query: None,
            tags: Some(vec!["avoid".to_string()]),
            limit: Some(500),
            offset: None,
        };

        let engrams = self.search_engrams(search).await?;

        let avoidances: Vec<AvoidanceEngram> = engrams
            .into_iter()
            .filter_map(|e| serde_json::from_str(&e.content).ok())
            .collect();

        info!("Restored {} avoidances from engrams", avoidances.len());
        Ok(avoidances)
    }

    pub async fn restore_workflow_state(&self, wallet: &str) -> WorkflowState {
        let mut state = WorkflowState::default();

        match self.get_discovered_kols(wallet).await {
            Ok(kols) => state.discovered_kols = kols,
            Err(e) => warn!("Failed to restore KOL discoveries: {}", e),
        }

        match self.get_saved_strategies(wallet).await {
            Ok(strategies) => state.strategies = strategies,
            Err(e) => warn!("Failed to restore strategies: {}", e),
        }

        match self.get_avoidances(wallet).await {
            Ok(avoidances) => state.avoidances = avoidances,
            Err(e) => warn!("Failed to restore avoidances: {}", e),
        }

        match self.get_patterns(wallet).await {
            Ok(patterns) => state.patterns = patterns,
            Err(e) => warn!("Failed to restore patterns: {}", e),
        }

        info!(
            "Workflow state restored: {} KOLs, {} strategies, {} avoidances, {} patterns",
            state.discovered_kols.len(),
            state.strategies.len(),
            state.avoidances.len(),
            state.patterns.len()
        );

        state
    }

    pub async fn save_transaction_summary(
        &self,
        wallet: &str,
        summary: &crate::engrams::schemas::TransactionSummary,
    ) -> Result<Engram, String> {
        let key = crate::engrams::schemas::generate_transaction_key(&summary.tx_signature);
        let content = serde_json::to_string(summary)
            .map_err(|e| format!("Failed to serialize transaction summary: {}", e))?;

        let action_str = match summary.action {
            crate::engrams::schemas::TransactionAction::Buy => "buy",
            crate::engrams::schemas::TransactionAction::Sell => "sell",
        };

        let metadata = serde_json::json!({
            "type": "transaction_summary",
            "action": action_str,
            "venue": summary.venue,
            "profitable": summary.pnl_sol.unwrap_or(0.0) > 0.0,
            "execution_time_ms": summary.execution_time_ms,
        });

        let mut tags = vec![
            "arb".to_string(),
            "trade".to_string(),
            "summary".to_string(),
            summary.venue.clone(),
            action_str.to_string(),
        ];
        if summary.pnl_sol.unwrap_or(0.0) > 0.0 {
            tags.push("profit".to_string());
        } else if summary.pnl_sol.is_some() {
            tags.push("loss".to_string());
        }

        let request = CreateEngramRequest {
            wallet_address: wallet.to_string(),
            engram_type: "knowledge".to_string(),
            key,
            content,
            metadata: Some(metadata),
            tags: Some(tags),
            is_public: Some(false),
        };

        self.create_engram(request).await
    }

    pub async fn save_execution_error(
        &self,
        wallet: &str,
        error: &crate::engrams::schemas::ExecutionError,
    ) -> Result<Engram, String> {
        let key = crate::engrams::schemas::generate_error_key(&error.error_type, &error.timestamp);
        let content = serde_json::to_string(error)
            .map_err(|e| format!("Failed to serialize execution error: {}", e))?;

        let error_type_str = serde_json::to_string(&error.error_type)
            .unwrap_or_else(|_| "unknown".to_string())
            .trim_matches('"')
            .to_string();

        let metadata = serde_json::json!({
            "type": "execution_error",
            "error_type": error_type_str,
            "recoverable": error.recoverable,
            "venue": error.context.venue,
        });

        let request = CreateEngramRequest {
            wallet_address: wallet.to_string(),
            engram_type: "knowledge".to_string(),
            key,
            content,
            metadata: Some(metadata),
            tags: Some(vec![
                "arb".to_string(),
                "error".to_string(),
                error_type_str,
                if error.recoverable {
                    "recoverable".to_string()
                } else {
                    "fatal".to_string()
                },
            ]),
            is_public: Some(false),
        };

        self.create_engram(request).await
    }

    pub async fn save_daily_metrics(
        &self,
        wallet: &str,
        metrics: &crate::engrams::schemas::DailyMetrics,
    ) -> Result<Engram, String> {
        let key = crate::engrams::schemas::generate_metrics_key(&metrics.period);
        let content = serde_json::to_string(metrics)
            .map_err(|e| format!("Failed to serialize daily metrics: {}", e))?;

        let metadata = serde_json::json!({
            "type": "daily_metrics",
            "period": metrics.period,
            "total_trades": metrics.total_trades,
            "win_rate": metrics.win_rate,
            "total_pnl_sol": metrics.total_pnl_sol,
        });

        let request = CreateEngramRequest {
            wallet_address: wallet.to_string(),
            engram_type: "knowledge".to_string(),
            key,
            content,
            metadata: Some(metadata),
            tags: Some(vec![
                "arb".to_string(),
                "metrics".to_string(),
                "daily".to_string(),
                metrics.period.clone(),
            ]),
            is_public: Some(false),
        };

        self.upsert_engram(request).await
    }

    pub async fn save_recommendation(
        &self,
        wallet: &str,
        recommendation: &crate::engrams::schemas::Recommendation,
    ) -> Result<Engram, String> {
        let key =
            crate::engrams::schemas::generate_recommendation_key(&recommendation.recommendation_id);
        let content = serde_json::to_string(recommendation)
            .map_err(|e| format!("Failed to serialize recommendation: {}", e))?;

        let category_str = serde_json::to_string(&recommendation.category)
            .unwrap_or_else(|_| "unknown".to_string())
            .trim_matches('"')
            .to_string();

        let metadata = serde_json::json!({
            "type": "recommendation",
            "a2a_discoverable": true,
            "schema_version": "1.0",
            "content_type": "recommendation",
            "category": category_str,
            "confidence": recommendation.confidence,
            "status": serde_json::to_string(&recommendation.status).unwrap_or_default().trim_matches('"'),
        });

        let status_str = serde_json::to_string(&recommendation.status)
            .unwrap_or_else(|_| "pending".to_string())
            .trim_matches('"')
            .to_string();

        let request = CreateEngramRequest {
            wallet_address: wallet.to_string(),
            engram_type: "knowledge".to_string(),
            key,
            content,
            metadata: Some(metadata),
            tags: Some(vec![
                "arb".to_string(),
                crate::engrams::schemas::A2A_TAG_LEARNING.to_string(),
                crate::engrams::schemas::RECOMMENDATION_TAG.to_string(),
                format!("category.{}", category_str),
                format!("status.{}", status_str),
            ]),
            is_public: Some(false),
        };

        self.upsert_engram(request).await
    }

    pub async fn save_conversation_log(
        &self,
        wallet: &str,
        conversation: &crate::engrams::schemas::ConversationLog,
    ) -> Result<Engram, String> {
        let key = crate::engrams::schemas::generate_conversation_key(&conversation.session_id);
        let content = serde_json::to_string(conversation)
            .map_err(|e| format!("Failed to serialize conversation log: {}", e))?;

        let topic_str = serde_json::to_string(&conversation.topic)
            .unwrap_or_else(|_| "unknown".to_string())
            .trim_matches('"')
            .to_string();

        let metadata = serde_json::json!({
            "type": "conversation_log",
            "a2a_discoverable": true,
            "schema_version": "1.0",
            "content_type": "conversation",
            "topic": topic_str,
            "participants": conversation.participants,
            "message_count": conversation.messages.len(),
            "consensus_reached": conversation.outcome.consensus_reached,
        });

        let request = CreateEngramRequest {
            wallet_address: wallet.to_string(),
            engram_type: "knowledge".to_string(),
            key,
            content,
            metadata: Some(metadata),
            tags: Some(vec![
                "arb".to_string(),
                crate::engrams::schemas::A2A_TAG_LEARNING.to_string(),
                "conversation".to_string(),
                topic_str,
            ]),
            is_public: Some(false),
        };

        self.create_engram(request).await
    }

    pub async fn save_consensus_analysis(
        &self,
        wallet: &str,
        analysis: &crate::engrams::schemas::ConsensusAnalysis,
    ) -> Result<Engram, String> {
        let key = crate::engrams::schemas::generate_consensus_analysis_key(&analysis.analysis_id);
        let content = serde_json::to_string(analysis)
            .map_err(|e| format!("Failed to serialize consensus analysis: {}", e))?;

        let analysis_type_str = serde_json::to_string(&analysis.analysis_type)
            .unwrap_or_else(|_| "unknown".to_string())
            .trim_matches('"')
            .to_string();

        let metadata = serde_json::json!({
            "type": "consensus_analysis",
            "a2a_discoverable": true,
            "schema_version": "1.0",
            "content_type": "analysis",
            "analysis_type": analysis_type_str,
            "time_period": analysis.time_period,
            "trades_analyzed": analysis.total_trades_analyzed,
            "recommendations_count": analysis.recommendations_count,
            "avg_confidence": analysis.avg_confidence,
            "models_queried": analysis.models_queried,
            "latency_ms": analysis.total_latency_ms,
            "risk_alerts_count": analysis.risk_alerts.len(),
        });

        let request = CreateEngramRequest {
            wallet_address: wallet.to_string(),
            engram_type: "knowledge".to_string(),
            key,
            content,
            metadata: Some(metadata),
            tags: Some(vec![
                "arb".to_string(),
                crate::engrams::schemas::A2A_TAG_LEARNING.to_string(),
                "analysis".to_string(),
                "consensus".to_string(),
                analysis_type_str,
            ]),
            is_public: Some(false),
        };

        self.create_engram(request).await
    }

    pub async fn save_consensus_decision(
        &self,
        wallet: &str,
        decision: &crate::engrams::schemas::ConsensusDecision,
    ) -> Result<Engram, String> {
        let key = crate::engrams::schemas::generate_consensus_decision_key(&decision.decision_id);
        let content = serde_json::to_string(decision)
            .map_err(|e| format!("Failed to serialize consensus decision: {}", e))?;

        let metadata = serde_json::json!({
            "type": "consensus_decision",
            "a2a_discoverable": true,
            "schema_version": "1.0",
            "content_type": "decision",
            "edge_id": decision.edge_id.to_string(),
            "strategy_id": decision.strategy_id.map(|s| s.to_string()),
            "approved": decision.approved,
            "agreement_score": decision.agreement_score,
            "weighted_confidence": decision.weighted_confidence,
            "models_count": decision.model_votes.len(),
            "latency_ms": decision.total_latency_ms,
        });

        let request = CreateEngramRequest {
            wallet_address: wallet.to_string(),
            engram_type: "knowledge".to_string(),
            key,
            content,
            metadata: Some(metadata),
            tags: Some(vec![
                "arb".to_string(),
                crate::engrams::schemas::A2A_TAG_LEARNING.to_string(),
                "decision".to_string(),
                "consensus".to_string(),
                if decision.approved {
                    "approved"
                } else {
                    "rejected"
                }
                .to_string(),
            ]),
            is_public: Some(false),
        };

        self.create_engram(request).await
    }

    pub async fn get_learning_engrams(
        &self,
        wallet: &str,
        category: Option<&str>,
        limit: Option<i64>,
    ) -> Result<Vec<Engram>, String> {
        let mut tags = vec![crate::engrams::schemas::A2A_TAG_LEARNING.to_string()];
        if let Some(cat) = category {
            tags.push(cat.to_string());
        }

        let search = SearchRequest {
            wallet_address: Some(wallet.to_string()),
            engram_type: Some("knowledge".to_string()),
            query: None,
            tags: Some(tags),
            limit: limit.or(Some(50)),
            offset: None,
        };

        self.search_engrams(search).await
    }

    pub async fn get_recommendations(
        &self,
        wallet: &str,
        status: Option<&crate::engrams::schemas::RecommendationStatus>,
        limit: Option<i64>,
    ) -> Result<Vec<crate::engrams::schemas::Recommendation>, String> {
        let engrams = self
            .get_learning_engrams(wallet, Some("recommendation"), limit)
            .await?;

        let recommendations: Vec<crate::engrams::schemas::Recommendation> = engrams
            .into_iter()
            .filter_map(|e| serde_json::from_str(&e.content).ok())
            .filter(|r: &crate::engrams::schemas::Recommendation| {
                if let Some(s) = status {
                    &r.status == s
                } else {
                    true
                }
            })
            .collect();

        Ok(recommendations)
    }

    pub async fn get_conversations(
        &self,
        wallet: &str,
        limit: Option<i64>,
    ) -> Result<Vec<crate::engrams::schemas::ConversationLog>, String> {
        let engrams = self
            .get_learning_engrams(wallet, Some("conversation"), limit)
            .await?;

        let conversations: Vec<crate::engrams::schemas::ConversationLog> = engrams
            .into_iter()
            .filter_map(|e| serde_json::from_str(&e.content).ok())
            .collect();

        Ok(conversations)
    }

    pub async fn update_recommendation_status(
        &self,
        wallet: &str,
        recommendation_id: &uuid::Uuid,
        new_status: crate::engrams::schemas::RecommendationStatus,
    ) -> Result<Engram, String> {
        let key = crate::engrams::schemas::generate_recommendation_key(recommendation_id);

        if let Some(existing) = self.get_engram_by_wallet_key(wallet, &key).await? {
            let mut recommendation: crate::engrams::schemas::Recommendation =
                serde_json::from_str(&existing.content)
                    .map_err(|e| format!("Failed to parse recommendation: {}", e))?;

            recommendation.status = new_status.clone();
            if new_status == crate::engrams::schemas::RecommendationStatus::Applied {
                recommendation.applied_at = Some(chrono::Utc::now());
            }

            let new_content = serde_json::to_string(&recommendation)
                .map_err(|e| format!("Failed to serialize recommendation: {}", e))?;

            let status_str = serde_json::to_string(&new_status)
                .unwrap_or_default()
                .trim_matches('"')
                .to_string();

            let mut tags = existing.tags.clone();
            tags.retain(|t| {
                !["pending", "acknowledged", "applied", "rejected"].contains(&t.as_str())
            });
            tags.push(status_str);

            self.update_engram(&existing.id, &new_content, Some(tags))
                .await
        } else {
            Err(format!("Recommendation {} not found", recommendation_id))
        }
    }

    pub async fn get_trade_history(
        &self,
        wallet: &str,
        limit: Option<i64>,
    ) -> Result<Vec<crate::engrams::schemas::TransactionSummary>, String> {
        let search = SearchRequest {
            wallet_address: Some(wallet.to_string()),
            engram_type: Some("knowledge".to_string()),
            query: None,
            tags: Some(vec!["trade".to_string(), "summary".to_string()]),
            limit: limit.or(Some(100)),
            offset: None,
        };

        let engrams = self.search_engrams(search).await?;

        let trades: Vec<crate::engrams::schemas::TransactionSummary> = engrams
            .into_iter()
            .filter_map(|e| serde_json::from_str(&e.content).ok())
            .collect();

        Ok(trades)
    }

    pub async fn get_error_history(
        &self,
        wallet: &str,
        limit: Option<i64>,
    ) -> Result<Vec<crate::engrams::schemas::ExecutionError>, String> {
        let search = SearchRequest {
            wallet_address: Some(wallet.to_string()),
            engram_type: Some("knowledge".to_string()),
            query: None,
            tags: Some(vec!["error".to_string()]),
            limit: limit.or(Some(50)),
            offset: None,
        };

        let engrams = self.search_engrams(search).await?;

        let errors: Vec<crate::engrams::schemas::ExecutionError> = engrams
            .into_iter()
            .filter_map(|e| serde_json::from_str(&e.content).ok())
            .collect();

        Ok(errors)
    }

    pub async fn get_trade_history_with_metadata(
        &self,
        wallet: &str,
        limit: Option<i64>,
    ) -> Result<Vec<crate::engrams::schemas::TradeEngramWrapper>, String> {
        let search = SearchRequest {
            wallet_address: Some(wallet.to_string()),
            engram_type: Some("knowledge".to_string()),
            query: None,
            tags: Some(vec!["trade".to_string(), "summary".to_string()]),
            limit: limit.or(Some(100)),
            offset: None,
        };

        let engrams = self.search_engrams(search).await?;

        let trades: Vec<crate::engrams::schemas::TradeEngramWrapper> = engrams
            .into_iter()
            .filter_map(|e| {
                let trade: crate::engrams::schemas::TransactionSummary =
                    serde_json::from_str(&e.content).ok()?;
                Some(crate::engrams::schemas::TradeEngramWrapper {
                    engram_id: e.id,
                    engram_key: e.key,
                    tags: e.tags,
                    created_at: e.created_at,
                    trade,
                })
            })
            .collect();

        Ok(trades)
    }

    pub async fn get_recommendations_with_metadata(
        &self,
        wallet: &str,
        status: Option<&crate::engrams::schemas::RecommendationStatus>,
        limit: Option<i64>,
    ) -> Result<Vec<crate::engrams::schemas::RecommendationEngramWrapper>, String> {
        let engrams = self
            .get_learning_engrams(wallet, Some("recommendation"), limit)
            .await?;

        let recommendations: Vec<crate::engrams::schemas::RecommendationEngramWrapper> = engrams
            .into_iter()
            .filter_map(|e| {
                let rec: crate::engrams::schemas::Recommendation =
                    serde_json::from_str(&e.content).ok()?;
                if let Some(s) = status {
                    if &rec.status != s {
                        return None;
                    }
                }
                Some(crate::engrams::schemas::RecommendationEngramWrapper {
                    engram_id: e.id,
                    engram_key: e.key,
                    tags: e.tags,
                    created_at: e.created_at,
                    recommendation: rec,
                })
            })
            .collect();

        Ok(recommendations)
    }

    pub async fn get_error_history_with_metadata(
        &self,
        wallet: &str,
        limit: Option<i64>,
    ) -> Result<Vec<crate::engrams::schemas::ErrorEngramWrapper>, String> {
        let search = SearchRequest {
            wallet_address: Some(wallet.to_string()),
            engram_type: Some("knowledge".to_string()),
            query: None,
            tags: Some(vec!["error".to_string()]),
            limit: limit.or(Some(50)),
            offset: None,
        };

        let engrams = self.search_engrams(search).await?;

        let errors: Vec<crate::engrams::schemas::ErrorEngramWrapper> = engrams
            .into_iter()
            .filter_map(|e| {
                let error: crate::engrams::schemas::ExecutionError =
                    serde_json::from_str(&e.content).ok()?;
                Some(crate::engrams::schemas::ErrorEngramWrapper {
                    engram_id: e.id,
                    engram_key: e.key,
                    tags: e.tags,
                    created_at: e.created_at,
                    error,
                })
            })
            .collect();

        Ok(errors)
    }

    pub async fn get_engrams_by_ids(
        &self,
        wallet: &str,
        engram_ids: &[String],
    ) -> Result<Vec<Engram>, String> {
        let mut results = Vec::new();

        for id in engram_ids {
            let url = format!("{}/engrams/{}", self.base_url, id);

            match self
                .http_client
                .get(&url)
                .header("X-Wallet-Address", wallet)
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        if let Ok(engram_resp) = response.json::<EngramResponse>().await {
                            if let Some(engram) = engram_resp.data {
                                results.push(engram);
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to fetch engram {}: {}", id, e);
                }
            }
        }

        Ok(results)
    }

    pub async fn save_watchlist_token(
        &self,
        wallet: &str,
        token: &crate::engrams::schemas::WatchlistToken,
    ) -> Result<Engram, String> {
        let key = crate::engrams::schemas::generate_watchlist_key(&token.mint);
        let content = serde_json::to_string(token)
            .map_err(|e| format!("Failed to serialize watchlist token: {}", e))?;

        let metadata = serde_json::json!({
            "type": "watchlist_token",
            "venue": token.venue,
            "symbol": token.symbol,
            "last_progress": token.last_progress,
        });

        let request = CreateEngramRequest {
            wallet_address: wallet.to_string(),
            engram_type: "knowledge".to_string(),
            key,
            content,
            metadata: Some(metadata),
            tags: Some(vec![
                "arb".to_string(),
                crate::engrams::schemas::WATCHLIST_TAG.to_string(),
                "token".to_string(),
                token.venue.clone(),
            ]),
            is_public: Some(false),
        };

        self.upsert_engram(request).await
    }

    pub async fn get_watchlist_tokens(
        &self,
        wallet: &str,
    ) -> Result<Vec<crate::engrams::schemas::WatchlistToken>, String> {
        let search = SearchRequest {
            wallet_address: Some(wallet.to_string()),
            engram_type: Some("knowledge".to_string()),
            query: None,
            tags: Some(vec![crate::engrams::schemas::WATCHLIST_TAG.to_string()]),
            limit: Some(500),
            offset: None,
        };

        let engrams = self.search_engrams(search).await?;

        let tokens: Vec<crate::engrams::schemas::WatchlistToken> = engrams
            .into_iter()
            .filter_map(|e| serde_json::from_str(&e.content).ok())
            .collect();

        info!("Retrieved {} watchlist tokens from engrams", tokens.len());
        Ok(tokens)
    }

    pub async fn remove_watchlist_token(&self, wallet: &str, mint: &str) -> Result<bool, String> {
        let key = crate::engrams::schemas::generate_watchlist_key(mint);

        if let Some(existing) = self.get_engram_by_wallet_key(wallet, &key).await? {
            self.delete_engram(&existing.id).await?;
            info!("Removed watchlist token {} from engrams", mint);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn is_token_tracked(&self, wallet: &str, mint: &str) -> Result<bool, String> {
        let key = crate::engrams::schemas::generate_watchlist_key(mint);
        let result = self.get_engram_by_wallet_key(wallet, &key).await?;
        Ok(result.is_some())
    }

    pub async fn delete_engram(&self, id: &str) -> Result<(), String> {
        if !self.is_configured() {
            return Err("Engrams service not configured".to_string());
        }

        let url = format!("{}/engrams/{}", self.base_url, id);
        debug!("Deleting engram: {} at {}", id, url);

        match self
            .http_client
            .delete(&url)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    info!("Deleted engram: {}", id);
                    Ok(())
                } else {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    error!("Engrams service error {}: {}", status, body);
                    Err(format!("Engrams service error {}: {}", status, body))
                }
            }
            Err(e) => {
                error!("Failed to connect to Engrams service: {}", e);
                Err(format!("Connection error: {}", e))
            }
        }
    }

    pub async fn clear_all_watchlist_tokens(&self, wallet: &str) -> Result<usize, String> {
        let tokens = self.get_watchlist_tokens(wallet).await?;
        let mut removed = 0;

        for token in tokens {
            if self.remove_watchlist_token(wallet, &token.mint).await? {
                removed += 1;
            }
        }

        info!("Cleared {} watchlist tokens from engrams", removed);
        Ok(removed)
    }

    pub async fn save_trade_analysis(
        &self,
        wallet: &str,
        analysis: &crate::engrams::schemas::TradeAnalysis,
    ) -> Result<Engram, String> {
        let key = crate::engrams::schemas::generate_trade_analysis_key(&analysis.analysis_id);
        let content = serde_json::to_string(analysis)
            .map_err(|e| format!("Failed to serialize trade analysis: {}", e))?;

        let metadata = serde_json::json!({
            "type": "trade_analysis",
            "a2a_discoverable": true,
            "schema_version": "1.0",
            "content_type": "trade_analysis",
            "position_id": analysis.position_id.to_string(),
            "token_symbol": analysis.token_symbol,
            "venue": analysis.venue,
            "pnl_sol": analysis.pnl_sol,
            "exit_reason": analysis.exit_reason,
            "confidence": analysis.confidence,
        });

        let request = CreateEngramRequest {
            wallet_address: wallet.to_string(),
            engram_type: "knowledge".to_string(),
            key,
            content,
            metadata: Some(metadata),
            tags: Some(vec![
                "arb".to_string(),
                crate::engrams::schemas::A2A_TAG_LEARNING.to_string(),
                crate::engrams::schemas::TRADE_ANALYSIS_TAG.to_string(),
                analysis.venue.clone(),
                analysis.exit_reason.clone(),
            ]),
            is_public: Some(false),
        };

        self.create_engram(request).await
    }

    pub async fn save_pattern_summary(
        &self,
        wallet: &str,
        summary: &crate::engrams::schemas::StoredPatternSummary,
    ) -> Result<Engram, String> {
        let key = crate::engrams::schemas::generate_pattern_summary_key(&summary.summary_id);
        let content = serde_json::to_string(summary)
            .map_err(|e| format!("Failed to serialize pattern summary: {}", e))?;

        let metadata = serde_json::json!({
            "type": "pattern_summary",
            "a2a_discoverable": true,
            "schema_version": "1.0",
            "content_type": "pattern_summary",
            "trades_analyzed": summary.trades_analyzed,
            "time_period": summary.time_period,
            "losing_patterns_count": summary.losing_patterns.len(),
            "winning_patterns_count": summary.winning_patterns.len(),
            "recommendations_count": summary.config_recommendations.len(),
        });

        let request = CreateEngramRequest {
            wallet_address: wallet.to_string(),
            engram_type: "knowledge".to_string(),
            key,
            content,
            metadata: Some(metadata),
            tags: Some(vec![
                "arb".to_string(),
                crate::engrams::schemas::A2A_TAG_LEARNING.to_string(),
                crate::engrams::schemas::PATTERN_SUMMARY_TAG.to_string(),
            ]),
            is_public: Some(false),
        };

        self.create_engram(request).await
    }

    pub async fn get_trade_analyses(
        &self,
        wallet: &str,
        limit: Option<i64>,
    ) -> Result<Vec<crate::engrams::schemas::TradeAnalysis>, String> {
        let search = SearchRequest {
            wallet_address: Some(wallet.to_string()),
            engram_type: Some("knowledge".to_string()),
            query: None,
            tags: Some(vec![crate::engrams::schemas::TRADE_ANALYSIS_TAG.to_string()]),
            limit: limit.or(Some(50)),
            offset: None,
        };

        let engrams = self.search_engrams(search).await?;

        let analyses: Vec<crate::engrams::schemas::TradeAnalysis> = engrams
            .into_iter()
            .filter_map(|e| serde_json::from_str(&e.content).ok())
            .collect();

        Ok(analyses)
    }

    pub async fn get_pattern_summaries(
        &self,
        wallet: &str,
        limit: Option<i64>,
    ) -> Result<Vec<crate::engrams::schemas::StoredPatternSummary>, String> {
        let search = SearchRequest {
            wallet_address: Some(wallet.to_string()),
            engram_type: Some("knowledge".to_string()),
            query: None,
            tags: Some(vec![
                crate::engrams::schemas::PATTERN_SUMMARY_TAG.to_string()
            ]),
            limit: limit.or(Some(10)),
            offset: None,
        };

        let engrams = self.search_engrams(search).await?;

        let summaries: Vec<crate::engrams::schemas::StoredPatternSummary> = engrams
            .into_iter()
            .filter_map(|e| serde_json::from_str(&e.content).ok())
            .collect();

        Ok(summaries)
    }

    pub async fn get_latest_pattern_summary(
        &self,
        wallet: &str,
    ) -> Result<Option<crate::engrams::schemas::StoredPatternSummary>, String> {
        let summaries = self.get_pattern_summaries(wallet, Some(1)).await?;
        Ok(summaries.into_iter().next())
    }

    pub async fn save_web_research(
        &self,
        wallet: &str,
        research: &crate::engrams::schemas::WebResearchEngram,
    ) -> Result<Engram, String> {
        let key = crate::engrams::schemas::generate_web_research_key(&research.research_id);
        let content = serde_json::to_string(research)
            .map_err(|e| format!("Failed to serialize web research: {}", e))?;

        let source_type_str = serde_json::to_string(&research.source_type)
            .unwrap_or_else(|_| "unknown".to_string())
            .trim_matches('"')
            .to_string();

        let focus_str = serde_json::to_string(&research.analysis_focus)
            .unwrap_or_else(|_| "general".to_string())
            .trim_matches('"')
            .to_string();

        let metadata = serde_json::json!({
            "type": "web_research",
            "a2a_discoverable": true,
            "schema_version": "1.0",
            "content_type": "web_research",
            "source_type": source_type_str,
            "source_url": research.source_url,
            "focus": focus_str,
            "confidence": research.confidence,
            "insights_count": research.key_insights.len(),
            "strategies_count": research.extracted_strategies.len(),
            "tokens_found": research.extracted_tokens,
        });

        let request = CreateEngramRequest {
            wallet_address: wallet.to_string(),
            engram_type: "knowledge".to_string(),
            key,
            content,
            metadata: Some(metadata),
            tags: Some(vec![
                "arb".to_string(),
                crate::engrams::schemas::A2A_TAG_LEARNING.to_string(),
                crate::engrams::schemas::WEB_RESEARCH_TAG.to_string(),
                format!("source.{}", source_type_str),
                format!("focus.{}", focus_str),
            ]),
            is_public: Some(false),
        };

        self.create_engram(request).await
    }

    pub async fn get_web_research(
        &self,
        wallet: &str,
        limit: Option<i64>,
    ) -> Result<Vec<crate::engrams::schemas::WebResearchEngram>, String> {
        let search = SearchRequest {
            wallet_address: Some(wallet.to_string()),
            engram_type: Some("knowledge".to_string()),
            query: None,
            tags: Some(vec![crate::engrams::schemas::WEB_RESEARCH_TAG.to_string()]),
            limit: limit.or(Some(50)),
            offset: None,
        };

        let engrams = self.search_engrams(search).await?;

        let research: Vec<crate::engrams::schemas::WebResearchEngram> = engrams
            .into_iter()
            .filter_map(|e| serde_json::from_str(&e.content).ok())
            .collect();

        Ok(research)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KolDiscoveryEngram {
    pub kol_address: String,
    pub display_name: Option<String>,
    pub trust_score: f64,
    pub win_rate: f64,
    pub total_trades: i32,
    pub discovery_source: String,
    pub discovered_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyEngram {
    pub strategy_id: String,
    pub name: String,
    pub strategy_type: String,
    pub venue_types: Vec<String>,
    pub execution_mode: String,
    pub risk_params: Value,
    #[serde(default = "default_is_active")]
    pub is_active: bool,
    pub created_at: String,
    #[serde(default)]
    pub updated_at: Option<String>,
}

fn default_is_active() -> bool {
    true
}

#[derive(Debug, Clone, Default)]
pub struct WorkflowState {
    pub discovered_kols: Vec<KolDiscoveryEngram>,
    pub strategies: Vec<StrategyEngram>,
    pub avoidances: Vec<AvoidanceEngram>,
    pub patterns: Vec<PatternEngram>,
}
