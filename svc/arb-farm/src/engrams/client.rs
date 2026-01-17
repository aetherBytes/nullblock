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
        wallet.map(|w| w.to_string()).or(self.default_wallet.clone())
    }

    pub async fn create_engram(&self, request: CreateEngramRequest) -> Result<Engram, String> {
        if !self.is_configured() {
            return Err("Engrams service not configured".to_string());
        }

        let url = format!("{}/engrams", self.base_url);
        debug!("Creating engram: {} at {}", request.key, url);

        match self.http_client
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
                                let err_msg = resp.error.unwrap_or_else(|| "Unknown error".to_string());
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

        match self.http_client
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
                                let err_msg = resp.error.unwrap_or_else(|| "Unknown error".to_string());
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

        match self.http_client
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
                                let err_msg = resp.error.unwrap_or_else(|| "Unknown error".to_string());
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
                if trade.profit_lamports > 0 { "profit".to_string() } else { "loss".to_string() },
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
        self.save_strategy_full(wallet, strategy_id, strategy_name, strategy_type, venue_types, execution_mode, risk_params, true).await
    }

    pub async fn get_engram_by_wallet_key(&self, wallet: &str, key: &str) -> Result<Option<Engram>, String> {
        if !self.is_configured() {
            return Err("Engrams service not configured".to_string());
        }

        let url = format!("{}/engrams/wallet/{}/{}", self.base_url, wallet, key);
        debug!("Fetching engram by wallet/key: {}/{}", wallet, key);

        match self.http_client
            .get(&url)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<EngramResponse>().await {
                        Ok(resp) => {
                            if resp.success {
                                Ok(resp.data)
                            } else {
                                Ok(None) // Not found is OK
                            }
                        }
                        Err(_) => Ok(None),
                    }
                } else if response.status().as_u16() == 404 {
                    Ok(None)
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

    pub async fn update_engram(&self, id: &str, content: &str, tags: Option<Vec<String>>) -> Result<Engram, String> {
        if !self.is_configured() {
            return Err("Engrams service not configured".to_string());
        }

        let url = format!("{}/engrams/{}", self.base_url, id);
        debug!("Updating engram: {} at {}", id, url);

        let body = serde_json::json!({
            "content": content,
            "tags": tags,
        });

        match self.http_client
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
                                let err_msg = resp.error.unwrap_or_else(|| "Unknown error".to_string());
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
        if let Ok(Some(existing)) = self.get_engram_by_wallet_key(&request.wallet_address, &request.key).await {
            // Update existing
            return self.update_engram(&existing.id, &request.content, request.tags).await;
        }

        // Also check with "default" wallet for legacy data
        if let Ok(Some(existing)) = self.get_engram_by_wallet_key("default", &request.key).await {
            // Update existing legacy engram
            return self.update_engram(&existing.id, &request.content, request.tags).await;
        }

        // Try to create new, but handle duplicate key gracefully
        match self.create_engram(request.clone()).await {
            Ok(engram) => Ok(engram),
            Err(e) if e.contains("duplicate key") => {
                // Engram exists but we couldn't find it - this shouldn't happen, but treat as success
                warn!("Engram {} already exists (duplicate key) but wasn't found - treating as success", request.key);
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

        let mut tags = vec![
            "arb".to_string(),
            "edge".to_string(),
            edge_type.to_string(),
        ];
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
                if trust_score >= 70.0 { "high_trust".to_string() } else { "tracking".to_string() },
            ]),
            is_public: Some(true), // KOL profiles can be shared
        };

        self.create_engram(request).await
    }

    pub async fn get_discovered_kols(&self, wallet: &str) -> Result<Vec<KolDiscoveryEngram>, String> {
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
