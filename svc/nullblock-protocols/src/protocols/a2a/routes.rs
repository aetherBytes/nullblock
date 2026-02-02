use axum::{
    middleware,
    routing::{delete, get, post},
    Router,
};

use super::auth::auth_middleware;
use super::handlers::{
    cancel_task, delete_push_notification_config, get_agent_card, get_authenticated_extended_card,
    get_push_notification_config, get_task, list_push_notification_configs, list_tasks,
    resubscribe_task, send_message, send_streaming_message, set_push_notification_config,
};
use super::jsonrpc::handle_jsonrpc;
use super::sse::{message_stream_handler, task_subscribe_handler};
use crate::server::AppState;

pub fn create_a2a_routes(state: AppState) -> Router {
    Router::new()
        // Agent Card endpoints
        .route("/card", get(get_agent_card))
        .route("/.well-known/agent-card.json", get(get_agent_card))
        // Message endpoints
        .route("/messages", post(send_message))
        .route("/messages/stream", post(send_streaming_message))
        .route("/messages/sse", get(message_stream_handler))
        // Task endpoints
        .route("/tasks", get(list_tasks))
        .route("/tasks/:id", get(get_task))
        .route("/tasks/:id/cancel", post(cancel_task))
        .route("/tasks/:id/subscribe", post(resubscribe_task))
        .route("/tasks/:id/sse", get(task_subscribe_handler))
        // Push notification endpoints
        .route(
            "/tasks/:id/pushNotificationConfigs",
            post(set_push_notification_config),
        )
        .route(
            "/tasks/:id/pushNotificationConfigs",
            get(list_push_notification_configs),
        )
        .route(
            "/tasks/:id/pushNotificationConfigs/:config_id",
            get(get_push_notification_config),
        )
        .route(
            "/tasks/:id/pushNotificationConfigs/:config_id",
            delete(delete_push_notification_config),
        )
        // JSON-RPC endpoint
        .route("/jsonrpc", post(handle_jsonrpc))
        // Authenticated extended card (same for now)
        .route("/authenticated/card", get(get_authenticated_extended_card))
        .with_state(state)
        .layer(middleware::from_fn(auth_middleware))
}
