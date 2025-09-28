use axum::{
    middleware,
    routing::{delete, get, post},
    Router,
};

use super::auth::auth_middleware;
use super::handlers::{
    get_agent_card, get_authenticated_extended_card,
    send_message, send_streaming_message,
    get_task, list_tasks, cancel_task, resubscribe_task,
    set_push_notification_config, get_push_notification_config,
    list_push_notification_configs, delete_push_notification_config,
};
use super::jsonrpc::handle_jsonrpc;

pub fn create_a2a_routes() -> Router {
    Router::new()
        // Agent Card endpoints
        .route("/card", get(get_agent_card))
        .route("/.well-known/agent-card.json", get(get_agent_card))

        // Message endpoints
        .route("/messages", post(send_message))
        .route("/messages/stream", post(send_streaming_message))

        // Task endpoints
        .route("/tasks", get(list_tasks))
        .route("/tasks/:id", get(get_task))
        .route("/tasks/:id/cancel", post(cancel_task))
        .route("/tasks/:id/subscribe", post(resubscribe_task))

        // Push notification endpoints
        .route("/tasks/:id/pushNotificationConfigs", post(set_push_notification_config))
        .route("/tasks/:id/pushNotificationConfigs", get(list_push_notification_configs))
        .route("/tasks/:id/pushNotificationConfigs/:config_id", get(get_push_notification_config))
        .route("/tasks/:id/pushNotificationConfigs/:config_id", delete(delete_push_notification_config))

        // JSON-RPC endpoint
        .route("/jsonrpc", post(handle_jsonrpc))

        // Authenticated extended card (same for now)
        .route("/authenticated/card", get(get_authenticated_extended_card))

        // Apply auth middleware to all routes
        .layer(middleware::from_fn(auth_middleware))
}