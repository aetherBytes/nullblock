use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::handlers::{
    generate::{generate_content, AppState},
    metrics::get_metrics,
    queue::{delete_content, get_content, list_queue, update_status},
    templates::list_templates,
};

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/api/content/generate", post(generate_content))
        .route("/api/content/queue", get(list_queue))
        .route("/api/content/queue/:id", get(get_content))
        .route("/api/content/queue/:id", put(update_status))
        .route("/api/content/queue/:id", delete(delete_content))
        .route("/api/content/metrics/:id", get(get_metrics))
        .route("/api/content/templates", get(list_templates))
        .with_state(state)
}
