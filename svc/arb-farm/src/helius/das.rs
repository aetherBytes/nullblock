use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{debug, error, info, warn};

use crate::error::{AppError, AppResult};
use crate::events::{topics, ArbEvent, EventBus, EventSource};
use super::client::HeliusClient;
use super::types::{Collection, Creator, TokenMetadata};

pub struct DasClient {
    client: Arc<HeliusClient>,
    event_bus: Arc<EventBus>,
}

impl DasClient {
    pub fn new(client: Arc<HeliusClient>, event_bus: Arc<EventBus>) -> Self {
        Self { client, event_bus }
    }

    pub async fn get_asset(&self, mint: &str) -> AppResult<TokenMetadata> {
        #[derive(Debug, Deserialize)]
        struct DasAsset {
            id: String,
            content: Option<AssetContent>,
            authorities: Option<Vec<Authority>>,
            compression: Option<Compression>,
            grouping: Option<Vec<Grouping>>,
            royalty: Option<Royalty>,
            creators: Option<Vec<DasCreator>>,
            ownership: Option<Ownership>,
            supply: Option<Supply>,
            mutable: Option<bool>,
            burnt: Option<bool>,
        }

        #[derive(Debug, Deserialize)]
        struct AssetContent {
            #[serde(rename = "$schema")]
            schema: Option<String>,
            json_uri: Option<String>,
            metadata: Option<Metadata>,
            links: Option<Links>,
        }

        #[derive(Debug, Deserialize)]
        struct Metadata {
            name: Option<String>,
            symbol: Option<String>,
            description: Option<String>,
            attributes: Option<Vec<Attribute>>,
        }

        #[derive(Debug, Deserialize)]
        struct Attribute {
            trait_type: String,
            value: serde_json::Value,
        }

        #[derive(Debug, Deserialize)]
        struct Links {
            image: Option<String>,
            external_url: Option<String>,
        }

        #[derive(Debug, Deserialize)]
        struct Authority {
            address: String,
            scopes: Vec<String>,
        }

        #[derive(Debug, Deserialize)]
        struct Compression {
            eligible: bool,
            compressed: bool,
            data_hash: Option<String>,
            creator_hash: Option<String>,
            asset_hash: Option<String>,
            tree: Option<String>,
            seq: Option<u64>,
            leaf_id: Option<u64>,
        }

        #[derive(Debug, Deserialize)]
        struct Grouping {
            group_key: String,
            group_value: String,
            verified: Option<bool>,
            collection_metadata: Option<CollectionMetadata>,
        }

        #[derive(Debug, Deserialize)]
        struct CollectionMetadata {
            name: Option<String>,
            symbol: Option<String>,
            image: Option<String>,
        }

        #[derive(Debug, Deserialize)]
        struct Royalty {
            royalty_model: String,
            target: Option<String>,
            percent: f64,
            basis_points: u64,
            primary_sale_happened: bool,
            locked: bool,
        }

        #[derive(Debug, Deserialize)]
        struct DasCreator {
            address: String,
            share: u8,
            verified: bool,
        }

        #[derive(Debug, Deserialize)]
        struct Ownership {
            frozen: bool,
            delegated: bool,
            delegate: Option<String>,
            ownership_model: String,
            owner: String,
        }

        #[derive(Debug, Deserialize)]
        struct Supply {
            print_max_supply: Option<u64>,
            print_current_supply: Option<u64>,
            edition_nonce: Option<u64>,
        }

        let asset: DasAsset = self
            .client
            .rpc_call("getAsset", json!({"id": mint}))
            .await?;

        let content = asset.content.as_ref();
        let metadata = content.and_then(|c| c.metadata.as_ref());

        let name = metadata
            .and_then(|m| m.name.clone())
            .unwrap_or_else(|| "Unknown".to_string());
        let symbol = metadata
            .and_then(|m| m.symbol.clone())
            .unwrap_or_else(|| "???".to_string());

        let attributes: HashMap<String, String> = metadata
            .and_then(|m| m.attributes.as_ref())
            .map(|attrs| {
                attrs
                    .iter()
                    .map(|a| {
                        let value = match &a.value {
                            serde_json::Value::String(s) => s.clone(),
                            other => other.to_string(),
                        };
                        (a.trait_type.clone(), value)
                    })
                    .collect()
            })
            .unwrap_or_default();

        let creators: Vec<Creator> = asset
            .creators
            .unwrap_or_default()
            .into_iter()
            .map(|c| Creator {
                address: c.address,
                verified: c.verified,
                share: c.share,
            })
            .collect();

        let collection: Option<Collection> = asset.grouping.and_then(|groups| {
            groups.into_iter().find(|g| g.group_key == "collection").map(|g| {
                Collection {
                    address: g.group_value,
                    name: g.collection_metadata.and_then(|m| m.name),
                    verified: g.verified.unwrap_or(false),
                }
            })
        });

        let image_uri = content
            .and_then(|c| c.links.as_ref())
            .and_then(|l| l.image.clone());

        let supply = asset
            .supply
            .and_then(|s| s.print_max_supply)
            .unwrap_or(1);

        let token_metadata = TokenMetadata {
            mint: mint.to_string(),
            name,
            symbol,
            decimals: 0,
            supply,
            creators,
            collection,
            attributes,
            image_uri,
        };

        let event = ArbEvent::new(
            "metadata_fetched",
            EventSource::External("helius_das".to_string()),
            topics::helius::das::METADATA_FETCHED,
            serde_json::to_value(&token_metadata).unwrap_or_default(),
        );

        if let Err(e) = self.event_bus.publish(event).await {
            warn!("Failed to publish DAS metadata event: {}", e);
        }

        Ok(token_metadata)
    }

    pub async fn get_assets_by_owner(
        &self,
        owner: &str,
        page: u32,
        limit: u32,
    ) -> AppResult<Vec<TokenMetadata>> {
        #[derive(Debug, Deserialize)]
        struct AssetList {
            total: u64,
            limit: u32,
            page: u32,
            items: Vec<serde_json::Value>,
        }

        let response: AssetList = self
            .client
            .rpc_call(
                "getAssetsByOwner",
                json!({
                    "ownerAddress": owner,
                    "page": page,
                    "limit": limit
                }),
            )
            .await?;

        let mut assets = Vec::new();
        for item in response.items {
            if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
                match self.get_asset(id).await {
                    Ok(metadata) => assets.push(metadata),
                    Err(e) => {
                        warn!("Failed to fetch asset {}: {}", id, e);
                    }
                }
            }
        }

        Ok(assets)
    }

    pub async fn search_assets(&self, query: AssetSearchQuery) -> AppResult<Vec<TokenMetadata>> {
        #[derive(Debug, Deserialize)]
        struct SearchResult {
            total: u64,
            limit: u32,
            page: u32,
            items: Vec<serde_json::Value>,
        }

        let mut params = serde_json::Map::new();

        if let Some(owner) = query.owner_address {
            params.insert("ownerAddress".to_string(), json!(owner));
        }
        if let Some(creator) = query.creator_address {
            params.insert("creatorAddress".to_string(), json!(creator));
        }
        if let Some(collection) = query.collection_address {
            params.insert("grouping".to_string(), json!([
                "collection",
                collection
            ]));
        }
        if let Some(frozen) = query.frozen {
            params.insert("frozen".to_string(), json!(frozen));
        }
        if let Some(burnt) = query.burnt {
            params.insert("burnt".to_string(), json!(burnt));
        }

        params.insert("page".to_string(), json!(query.page.unwrap_or(1)));
        params.insert("limit".to_string(), json!(query.limit.unwrap_or(50)));

        let response: SearchResult = self
            .client
            .rpc_call("searchAssets", serde_json::Value::Object(params))
            .await?;

        let mut assets = Vec::new();
        for item in response.items {
            if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
                match self.get_asset(id).await {
                    Ok(metadata) => assets.push(metadata),
                    Err(e) => {
                        warn!("Failed to fetch asset {}: {}", id, e);
                    }
                }
            }
        }

        Ok(assets)
    }

    pub async fn flag_creator(&self, creator_address: &str, reason: &str) {
        let event = ArbEvent::new(
            "creator_flagged",
            EventSource::External("helius_das".to_string()),
            topics::helius::das::CREATOR_FLAGGED,
            json!({
                "creator_address": creator_address,
                "reason": reason,
                "timestamp": chrono::Utc::now()
            }),
        );

        if let Err(e) = self.event_bus.publish(event).await {
            warn!("Failed to publish creator flagged event: {}", e);
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AssetSearchQuery {
    pub owner_address: Option<String>,
    pub creator_address: Option<String>,
    pub collection_address: Option<String>,
    pub frozen: Option<bool>,
    pub burnt: Option<bool>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}
