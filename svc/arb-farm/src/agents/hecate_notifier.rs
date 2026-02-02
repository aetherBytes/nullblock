use reqwest::Client;
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::events::{approval as approval_topics, ArbEvent};
use crate::models::HecateRecommendation;

pub struct HecateNotifier {
    client: Client,
    agents_service_url: String,
    event_rx: broadcast::Receiver<ArbEvent>,
    enabled: bool,
}

impl HecateNotifier {
    pub fn new(agents_service_url: String, event_rx: broadcast::Receiver<ArbEvent>) -> Self {
        Self {
            client: Client::new(),
            agents_service_url,
            event_rx,
            enabled: true,
        }
    }

    pub async fn start(mut self) {
        tracing::info!("🤖 HecateNotifier started - listening for approval events");

        while let Ok(event) = self.event_rx.recv().await {
            if !self.enabled {
                continue;
            }

            if event.topic == approval_topics::HECATE_NOTIFIED {
                if let Err(e) = self.handle_approval_notification(&event).await {
                    tracing::warn!("Failed to notify Hecate: {}", e);
                }
            }
        }

        tracing::info!("HecateNotifier stopped");
    }

    async fn handle_approval_notification(
        &self,
        event: &ArbEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let approval_id = event
            .payload
            .get("approval_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or("Missing approval_id in event")?;

        let approval_type = event
            .payload
            .get("approval_type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let context = event
            .payload
            .get("context")
            .cloned()
            .unwrap_or(serde_json::json!({}));

        tracing::debug!(
            approval_id = %approval_id,
            approval_type = %approval_type,
            "Requesting Hecate recommendation"
        );

        let recommendation = self
            .request_hecate_recommendation(approval_id, approval_type, &context)
            .await?;

        self.submit_recommendation(recommendation).await?;

        Ok(())
    }

    async fn request_hecate_recommendation(
        &self,
        approval_id: Uuid,
        approval_type: &str,
        context: &serde_json::Value,
    ) -> Result<HecateRecommendation, Box<dyn std::error::Error + Send + Sync>> {
        let prompt = self.build_recommendation_prompt(approval_type, context);

        let request_body = serde_json::json!({
            "message": prompt,
            "context": {
                "approval_id": approval_id.to_string(),
                "type": "arb_farm_approval_request"
            },
            "options": {
                "max_tokens": 500,
                "temperature": 0.3
            }
        });

        let response = self
            .client
            .post(format!("{}/api/agents/analyze", self.agents_service_url))
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            tracing::warn!("Hecate request failed: {} - {}", status, text);
            return Ok(HecateRecommendation {
                approval_id,
                decision: false,
                reasoning: format!("Could not reach Hecate service: {}", status),
                confidence: 0.0,
            });
        }

        let response_json: serde_json::Value = response.json().await?;

        let decision = response_json
            .get("recommendation")
            .and_then(|r| r.get("approve"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let reasoning = response_json
            .get("recommendation")
            .and_then(|r| r.get("reasoning"))
            .and_then(|v| v.as_str())
            .unwrap_or("No reasoning provided")
            .to_string();

        let confidence = response_json
            .get("recommendation")
            .and_then(|r| r.get("confidence"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.5);

        Ok(HecateRecommendation {
            approval_id,
            decision,
            reasoning,
            confidence,
        })
    }

    fn build_recommendation_prompt(
        &self,
        approval_type: &str,
        context: &serde_json::Value,
    ) -> String {
        let context_str = serde_json::to_string_pretty(context).unwrap_or_default();

        format!(
            r#"Analyze this ArbFarm trade approval request and provide a recommendation.

Approval Type: {}

Context:
{}

Based on the context, provide a recommendation in JSON format:
{{
  "approve": true/false,
  "reasoning": "Brief explanation of your recommendation",
  "confidence": 0.0-1.0
}}

Consider:
1. Risk vs reward ratio
2. Token/project legitimacy
3. Position size appropriateness
4. Market conditions
5. Any red flags in the context

Be conservative - when in doubt, recommend rejection."#,
            approval_type, context_str,
        )
    }

    async fn submit_recommendation(
        &self,
        recommendation: HecateRecommendation,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let arbfarm_url = std::env::var("ARB_FARM_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:9007".to_string());

        let response = self
            .client
            .post(format!("{}/approvals/hecate-recommendation", arbfarm_url))
            .json(&recommendation)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            tracing::warn!("Failed to submit recommendation: {}", status);
        } else {
            tracing::info!(
                approval_id = %recommendation.approval_id,
                decision = recommendation.decision,
                confidence = recommendation.confidence,
                "Hecate recommendation submitted"
            );
        }

        Ok(())
    }
}

pub fn spawn_hecate_notifier(
    agents_service_url: String,
    event_rx: broadcast::Receiver<ArbEvent>,
) -> tokio::task::JoinHandle<()> {
    let notifier = HecateNotifier::new(agents_service_url, event_rx);
    tokio::spawn(async move {
        notifier.start().await;
    })
}
