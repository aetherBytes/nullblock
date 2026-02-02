use axum::{extract::Path, Json};

use crate::errors::ProtocolError;
use crate::protocols::a2a::types::{PushNotificationConfig, PushNotificationConfigRequest};

pub async fn set_push_notification_config(
    Path(task_id): Path<String>,
    Json(_request): Json<PushNotificationConfigRequest>,
) -> Result<Json<PushNotificationConfig>, ProtocolError> {
    // TODO: Integrate with Agents database task management system
    // This should:
    // 1. Validate task exists in Agents database
    // 2. Store notification config with webhook URL and event filters
    // 3. Set up webhook delivery system for task lifecycle events

    Err(ProtocolError::TaskNotFound(format!(
        "Push notification config for task {} not yet implemented - requires Agents database integration",
        task_id
    )))
}

pub async fn get_push_notification_config(
    Path((task_id, config_id)): Path<(String, String)>,
) -> Result<Json<PushNotificationConfig>, ProtocolError> {
    // TODO: Integrate with Agents database task management system
    // This should query notification configs stored in Agents database
    // Validate task ownership and return config details

    Err(ProtocolError::TaskNotFound(format!(
        "Notification config {} for task {} retrieval not yet implemented - requires Agents database integration",
        config_id, task_id
    )))
}

pub async fn list_push_notification_configs(
    Path(task_id): Path<String>,
) -> Result<Json<Vec<PushNotificationConfig>>, ProtocolError> {
    // TODO: Integrate with Agents database task management system
    // This should query all notification configs for a task from Agents database
    // Return list of webhook configurations and event subscriptions

    Err(ProtocolError::TaskNotFound(format!(
        "Notification config listing for task {} not yet implemented - requires Agents database integration",
        task_id
    )))
}

pub async fn delete_push_notification_config(
    Path((task_id, config_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, ProtocolError> {
    // TODO: Integrate with Agents database task management system
    // This should delete notification config from Agents database
    // Stop webhook delivery for the removed configuration

    Err(ProtocolError::TaskNotFound(format!(
        "Notification config {} deletion for task {} not yet implemented - requires Agents database integration",
        config_id, task_id
    )))
}
