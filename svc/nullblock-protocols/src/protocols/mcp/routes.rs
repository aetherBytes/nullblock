use super::auth::mcp_auth_middleware;
use super::jsonrpc::handle_jsonrpc;
use crate::server::AppState;
use axum::{middleware, routing::post, Router};

pub fn create_mcp_routes(state: AppState) -> Router {
    Router::new()
        .route("/jsonrpc", post(handle_jsonrpc))
        .with_state(state)
        .layer(middleware::from_fn(mcp_auth_middleware))
}
