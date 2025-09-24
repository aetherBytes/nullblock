use std::fs;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn setup_logging() -> anyhow::Result<()> {
    // Create logs directory if it doesn't exist
    fs::create_dir_all("logs")?;

    // Build the subscriber with environment filter
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    // Check if we want JSON format
    let use_json = std::env::var("LOG_FORMAT")
        .map(|f| f.to_lowercase() == "json")
        .unwrap_or(false);

    if use_json {
        tracing_subscriber::registry()
            .with(fmt::layer()
                .json()
                .with_current_span(false)
                .with_span_list(true))
            .with(env_filter)
            .init();
    } else {
        tracing_subscriber::registry()
            .with(fmt::layer()
                .pretty()
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true))
            .with(env_filter)
            .init();
    }

    Ok(())
}

// Cyberpunk-themed logging macros similar to Python version
#[macro_export]
macro_rules! log_agent_startup {
    ($agent:expr, $version:expr) => {
        tracing::info!(
            agent = $agent,
            version = $version,
            "🚀 {} v{} initializing...",
            $agent,
            $version
        );
    };
}

#[macro_export]
macro_rules! log_agent_shutdown {
    ($agent:expr) => {
        tracing::info!(
            agent = $agent,
            "🛑 {} shutting down gracefully...",
            $agent
        );
    };
}

#[macro_export]
macro_rules! log_model_info {
    ($model:expr, $provider:expr, $cost:expr) => {
        tracing::info!(
            model = $model,
            provider = $provider,
            cost = $cost,
            "🧠 Model: {} via {} (cost: ${:.6})",
            $model,
            $provider,
            $cost
        );
    };
}

#[macro_export]
macro_rules! log_request_start {
    ($request_type:expr, $details:expr) => {
        tracing::info!(
            request_type = $request_type,
            details = $details,
            "📥 Processing {} request: {}",
            $request_type,
            $details
        );
    };
}

#[macro_export]
macro_rules! log_request_complete {
    ($request_type:expr, $latency_ms:expr, $success:expr) => {
        let emoji = if $success { "✅" } else { "❌" };
        tracing::info!(
            request_type = $request_type,
            latency_ms = $latency_ms,
            success = $success,
            "{} {} request completed in {:.0}ms",
            emoji,
            $request_type,
            $latency_ms
        );
    };
}
