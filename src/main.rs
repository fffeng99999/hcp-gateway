mod config;
mod error;
mod models;
mod state;
mod tasks;
mod middleware;
mod api;
mod services;
mod data;

use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("hcp_gateway=debug".parse()?)
                .add_directive("info".parse()?),
        )
        .init();

    tracing::info!("Starting HCP Gateway...");

    // Load mock data from JSON
    let mock_data = data::load_mock_data("data/mock_data.json")
        .await
        .map_err(|e| format!("Failed to load mock data: {}", e))?
        .unwrap_or_else(data::default_mock_data);

    // Initialize application state
    let app_state = Arc::new(state::AppState::new(mock_data));

    // Build router
    let app = Router::new()
        // Health check
        .route("/health", get(api::health::health_check))
        
        // Consensus API
        .route("/consensus/algorithms", get(api::consensus::get_algorithms))
        .route("/consensus/config", get(api::consensus::get_config))
        .route("/consensus/select", post(api::consensus::select_algorithm))
        .route(
            "/consensus/parameters",
            axum::routing::put(api::consensus::update_parameters),
        )
        .route(
            "/consensus/benchmark/start",
            post(api::consensus::start_benchmark),
        )
        .route(
            "/consensus/benchmark/:id",
            get(api::consensus::get_benchmark),
        )
        .route(
            "/consensus/benchmark/:id/stop",
            post(api::consensus::stop_benchmark),
        )
        .route(
            "/consensus/benchmark/history",
            get(api::consensus::get_benchmark_history),
        )
        
        // Transaction API
        .route("/transaction/submit", post(api::transaction::submit_transaction))
        .route("/transaction/:id", get(api::transaction::get_transaction))
        .route(
            "/transaction/status",
            get(api::transaction::get_transaction_status),
        )
        .route(
            "/transaction/history",
            get(api::transaction::get_transaction_history),
        )
        
        // Node API
        .route("/node/list", get(api::node::list_nodes))
        .route("/node/:id", get(api::node::get_node))
        .route("/node/stats", get(api::node::get_node_stats))
        
        // Performance API
        .route("/performance/metrics", get(api::performance::get_metrics))
        .route(
            "/performance/history",
            get(api::performance::get_performance_history),
        )
        .route(
            "/performance/comparison",
            get(api::performance::get_performance_comparison),
        )
        
        // Analysis API
        .route("/analysis/report", get(api::analysis::get_report))
        .route("/analysis/trends", get(api::analysis::get_trends))
        
        .layer(CorsLayer::permissive())
        .layer(axum::middleware::from_fn(middleware::logging_middleware))
        .with_state(app_state);

    // Start server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .map_err(|e| format!("Failed to bind to port 8080: {}", e))?
        .into_std()
        .map_err(|e| format!("Failed to convert listener: {}", e))?
        .into();

    tracing::info!("Server listening on http://127.0.0.1:8080");

    axum::serve(listener, app)
        .await
        .map_err(|e| format!("Server error: {}", e))?;

    Ok(())
}
