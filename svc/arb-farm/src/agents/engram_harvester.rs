use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use crate::error::AppResult;
use crate::events::{engram as engram_topics, AgentType, ArbEvent, EventSource};
use crate::models::{
    ArbEngram, AvoidanceContent, AvoidanceSeverity, EdgePatternContent, EngramMetadata,
    EngramQuery, EngramSearchResult, EngramSource, EngramType, PatternMatch, PatternMatchRequest,
};

pub struct EngramHarvester {
    id: Uuid,
    engrams: Arc<RwLock<HashMap<String, ArbEngram>>>,
    event_tx: broadcast::Sender<ArbEvent>,
    stats: Arc<RwLock<HarvesterStats>>,
}

impl Clone for EngramHarvester {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            engrams: Arc::clone(&self.engrams),
            event_tx: self.event_tx.clone(),
            stats: Arc::clone(&self.stats),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct HarvesterStats {
    pub total_engrams: u64,
    pub engrams_by_type: HashMap<String, u64>,
    pub patterns_matched: u64,
    pub avoidances_created: u64,
    pub last_harvest_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl EngramHarvester {
    pub fn new(event_tx: broadcast::Sender<ArbEvent>) -> Self {
        Self {
            id: Uuid::new_v4(),
            engrams: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            stats: Arc::new(RwLock::new(HarvesterStats::default())),
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub async fn store_engram(&self, engram: ArbEngram) -> AppResult<Uuid> {
        let engram_id = engram.id;
        let key = engram.key.clone();
        let engram_type = engram.engram_type;

        let mut engrams = self.engrams.write().await;
        engrams.insert(key.clone(), engram);

        let mut stats = self.stats.write().await;
        stats.total_engrams += 1;
        *stats
            .engrams_by_type
            .entry(engram_type.to_string())
            .or_insert(0) += 1;
        stats.last_harvest_at = Some(chrono::Utc::now());

        let _ = self.event_tx.send(ArbEvent::new(
            "engram_created",
            EventSource::Agent(AgentType::EngramHarvester),
            engram_topics::CREATED,
            serde_json::json!({
                "engram_id": engram_id,
                "key": key,
                "engram_type": engram_type.to_string(),
            }),
        ));

        Ok(engram_id)
    }

    pub async fn get_engram(&self, key: &str) -> Option<ArbEngram> {
        let mut engrams = self.engrams.write().await;
        if let Some(engram) = engrams.get_mut(key) {
            engram.metadata.access_count += 1;
            engram.metadata.last_accessed_at = Some(chrono::Utc::now());
            Some(engram.clone())
        } else {
            None
        }
    }

    pub async fn search_engrams(&self, query: &EngramQuery) -> EngramSearchResult {
        let engrams = self.engrams.read().await;

        let mut results: Vec<&ArbEngram> = engrams.values().collect();

        if let Some(ref engram_type) = query.engram_type {
            results.retain(|e| e.engram_type == *engram_type);
        }

        if let Some(ref prefix) = query.key_prefix {
            results.retain(|e| e.key.starts_with(prefix));
        }

        if let Some(ref tag) = query.tag {
            results.retain(|e| e.metadata.tags.contains(tag));
        }

        if let Some(min_conf) = query.min_confidence {
            results.retain(|e| e.confidence >= min_conf);
        }

        let total = results.len() as u64;

        let offset = query.offset.unwrap_or(0) as usize;
        let limit = query.limit.unwrap_or(50) as usize;

        let paginated: Vec<ArbEngram> = results
            .into_iter()
            .skip(offset)
            .take(limit)
            .cloned()
            .collect();

        EngramSearchResult {
            engrams: paginated,
            total,
            query: query.clone(),
        }
    }

    pub async fn find_matching_patterns(&self, request: &PatternMatchRequest) -> Vec<PatternMatch> {
        let engrams = self.engrams.read().await;

        let min_similarity = request.min_similarity.unwrap_or(0.5);

        let mut matches: Vec<PatternMatch> = engrams
            .values()
            .filter(|e| e.engram_type == EngramType::EdgePattern)
            .filter_map(|e| {
                let content: EdgePatternContent =
                    serde_json::from_value(e.content.clone()).ok()?;

                let mut similarity: f64 = 0.0;

                if content.edge_type == request.edge_type {
                    similarity += 0.4;
                }

                if content.venue_type == request.venue_type {
                    similarity += 0.3;
                }

                if let Some(ref token) = request.token_mint {
                    if e.metadata.related_tokens.contains(token) {
                        similarity += 0.3;
                    }
                } else {
                    similarity += 0.15;
                }

                if similarity >= min_similarity {
                    let recommended = if content.success_rate > 0.7 && e.confidence > 0.7 {
                        "EXECUTE - High confidence pattern".to_string()
                    } else if content.success_rate > 0.5 {
                        "CONSIDER - Moderate confidence".to_string()
                    } else {
                        "CAUTION - Low historical success".to_string()
                    };

                    Some(PatternMatch {
                        engram: e.clone(),
                        similarity_score: similarity,
                        recommended_action: recommended,
                    })
                } else {
                    None
                }
            })
            .collect();

        matches.sort_by(|a, b| {
            b.similarity_score
                .partial_cmp(&a.similarity_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if !matches.is_empty() {
            let mut stats = self.stats.write().await;
            stats.patterns_matched += 1;

            let _ = self.event_tx.send(ArbEvent::new(
                "pattern_matched",
                EventSource::Agent(AgentType::EngramHarvester),
                engram_topics::PATTERN_MATCHED,
                serde_json::json!({
                    "matches_found": matches.len(),
                    "edge_type": request.edge_type,
                    "venue_type": request.venue_type,
                }),
            ));
        }

        matches
    }

    pub async fn create_avoidance_engram(
        &self,
        entity_type: &str,
        address: &str,
        reason: &str,
        category: &str,
        severity: AvoidanceSeverity,
    ) -> AppResult<Uuid> {
        let key = format!("arb.avoid.{}.{}", entity_type, address);

        let content = AvoidanceContent {
            entity_type: entity_type.to_string(),
            address: address.to_string(),
            reason: reason.to_string(),
            category: category.to_string(),
            severity,
            evidence: vec![],
            reported_at: chrono::Utc::now(),
        };

        let confidence = match severity {
            AvoidanceSeverity::Critical => 1.0,
            AvoidanceSeverity::High => 0.9,
            AvoidanceSeverity::Medium => 0.7,
            AvoidanceSeverity::Low => 0.5,
        };

        let mut metadata = EngramMetadata::default();
        metadata.tags = vec![
            format!("avoidance:{}", category),
            format!("severity:{:?}", severity),
        ];

        if entity_type == "token" {
            metadata.related_tokens.push(address.to_string());
        } else if entity_type == "wallet" {
            metadata.related_wallets.push(address.to_string());
        }

        let engram = ArbEngram::new(
            key,
            EngramType::Avoidance,
            serde_json::to_value(content)?,
            EngramSource::ThreatDetection(address.to_string()),
        )
        .with_confidence(confidence)
        .with_metadata(metadata);

        let engram_id = self.store_engram(engram).await?;

        let mut stats = self.stats.write().await;
        stats.avoidances_created += 1;

        let _ = self.event_tx.send(ArbEvent::new(
            "avoidance_created",
            EventSource::Agent(AgentType::EngramHarvester),
            engram_topics::AVOIDANCE_CREATED,
            serde_json::json!({
                "engram_id": engram_id,
                "entity_type": entity_type,
                "address": address,
                "category": category,
                "severity": format!("{:?}", severity),
            }),
        ));

        Ok(engram_id)
    }

    pub async fn should_avoid(&self, entity_type: &str, address: &str) -> Option<AvoidanceContent> {
        let key = format!("arb.avoid.{}.{}", entity_type, address);

        if let Some(engram) = self.get_engram(&key).await {
            if engram.engram_type == EngramType::Avoidance {
                return serde_json::from_value(engram.content).ok();
            }
        }

        None
    }

    pub async fn create_edge_pattern_engram(
        &self,
        edge_type: &str,
        venue_type: &str,
        route_signature: &str,
        success_rate: f64,
        avg_profit_bps: f64,
        sample_count: u32,
    ) -> AppResult<Uuid> {
        let key = format!(
            "arb.pattern.{}.{}.{}",
            edge_type,
            venue_type,
            &route_signature[..8.min(route_signature.len())]
        );

        let content = EdgePatternContent {
            edge_type: edge_type.to_string(),
            venue_type: venue_type.to_string(),
            route_signature: route_signature.to_string(),
            avg_profit_bps,
            success_rate,
            sample_count,
            optimal_conditions: vec![],
            risk_factors: vec![],
        };

        let confidence = calculate_pattern_confidence(success_rate, sample_count);

        let mut metadata = EngramMetadata::default();
        metadata.tags = vec![
            format!("edge_type:{}", edge_type),
            format!("venue:{}", venue_type),
        ];
        metadata.effectiveness_score = Some(success_rate);

        let engram = ArbEngram::new(
            key,
            EngramType::EdgePattern,
            serde_json::to_value(content)?,
            EngramSource::Agent("engram_harvester".to_string()),
        )
        .with_confidence(confidence)
        .with_metadata(metadata);

        self.store_engram(engram).await
    }

    pub async fn get_stats(&self) -> HarvesterStats {
        self.stats.read().await.clone()
    }

    pub async fn delete_engram(&self, key: &str) -> bool {
        let mut engrams = self.engrams.write().await;
        engrams.remove(key).is_some()
    }

    pub async fn update_engram_confidence(&self, key: &str, new_confidence: f64) -> bool {
        let mut engrams = self.engrams.write().await;
        if let Some(engram) = engrams.get_mut(key) {
            engram.confidence = new_confidence.clamp(0.0, 1.0);
            engram.updated_at = chrono::Utc::now();
            true
        } else {
            false
        }
    }

    pub async fn save_strategies(&self, strategies: &[crate::models::Strategy]) -> AppResult<usize> {
        let mut count = 0;

        for strategy in strategies {
            let key = format!("arb.strategy.{}", strategy.id);

            let content = serde_json::json!({
                "id": strategy.id,
                "name": strategy.name,
                "strategy_type": strategy.strategy_type,
                "venue_types": strategy.venue_types,
                "execution_mode": strategy.execution_mode,
                "risk_params": strategy.risk_params,
                "is_active": strategy.is_active,
                "created_at": strategy.created_at,
                "updated_at": strategy.updated_at,
            });

            let mut metadata = EngramMetadata::default();
            metadata.tags = vec![
                format!("strategy_type:{}", strategy.strategy_type),
                format!("execution_mode:{}", strategy.execution_mode),
            ];

            let engram = ArbEngram::new(
                key,
                EngramType::Strategy,
                content,
                EngramSource::Agent("strategy_engine".to_string()),
            )
            .with_confidence(0.9)
            .with_metadata(metadata);

            self.store_engram(engram).await?;
            count += 1;
        }

        let _ = self.event_tx.send(ArbEvent::new(
            "strategies_saved_to_engrams",
            EventSource::Agent(AgentType::EngramHarvester),
            engram_topics::CREATED,
            serde_json::json!({
                "count": count,
                "saved_at": chrono::Utc::now(),
            }),
        ));

        Ok(count)
    }
}

fn calculate_pattern_confidence(success_rate: f64, sample_count: u32) -> f64 {
    let base_confidence = success_rate;

    let sample_factor = if sample_count >= 100 {
        1.0
    } else if sample_count >= 50 {
        0.9
    } else if sample_count >= 20 {
        0.8
    } else if sample_count >= 10 {
        0.7
    } else {
        0.5
    };

    (base_confidence * sample_factor).clamp(0.0, 1.0)
}
