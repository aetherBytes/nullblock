use axum::{response::Json, routing::get, Router};
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod database;
mod error;
mod events;
mod generator;
mod handlers;
mod models;
mod repository;
mod routes;

use database::Database;
use events::{EventPublisher, HttpEventPublisher, NoOpPublisher};
use generator::engine::ContentGenerator;
use generator::templates::TemplateLoader;
use handlers::generate::AppState;

fn load_env() {
    if dotenvy::from_filename(".env.dev").is_ok() {
        println!("📁 Loaded environment from .env.dev");
    } else if dotenvy::dotenv().is_ok() {
        println!("📁 Loaded environment from .env");
    } else {
        println!("⚠️ No .env file found, using system environment variables");
    }
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    service: String,
    version: String,
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        service: "nullblock-content".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[tokio::main]
async fn main() {
    load_env();

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    info!("🚀 Starting NullBlock Content Service");

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let db = Database::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    if let Err(e) = db.health_check().await {
        error!("❌ Database health check failed: {}", e);
        std::process::exit(1);
    }

    info!("✅ Database connection healthy");

    let templates_path = std::env::var("TEMPLATES_PATH")
        .unwrap_or_else(|_| "config/templates.json".to_string());

    let template_config = TemplateLoader::load_from_json(&templates_path)
        .unwrap_or_else(|e| {
            info!("⚠️ Failed to load templates from {}: {}. Using defaults.", templates_path, e);
            TemplateLoader::seed_default_templates()
        });

    let generator = ContentGenerator::new(template_config);

    let event_publisher: Arc<dyn EventPublisher> = if let Ok(endpoint) = std::env::var("EVENT_ENDPOINT") {
        info!("📡 Event publishing enabled: {}", endpoint);
        Arc::new(HttpEventPublisher::new(endpoint))
    } else {
        info!("⚠️ Event publishing disabled (no EVENT_ENDPOINT)");
        Arc::new(NoOpPublisher)
    };

    let state = AppState {
        db: Arc::new(db),
        generator: Arc::new(generator),
        event_publisher,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(health_check))
        .merge(routes::create_router(state))
        .layer(cors);

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8002".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid u16");

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    info!("🎧 Listening on http://{}", addr);
    info!("📡 Health check: http://{}/health", addr);
    info!("📡 API docs: http://{}/api/content/", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app)
        .await
        .expect("Server failed");
}
