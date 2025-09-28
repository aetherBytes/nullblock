use axum::Json;
use crate::protocols::a2a::types::AgentCard;

pub async fn get_agent_card() -> Json<AgentCard> {
    Json(AgentCard::default())
}

pub async fn get_authenticated_extended_card() -> Json<AgentCard> {
    // For now, return the same card as the public one
    // TODO: Implement authenticated extended card with additional details
    Json(AgentCard::default())
}