mod api;
mod config;
mod error;
mod middleware;
mod models;
mod state;
mod tasks;
mod services;

use axum::{
    response::IntoResponse,
    routing::{get, post, put},
    Router,
};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tower_http::cors::CorsLayer;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    tracing::info!("Starting HCP Gateway...");

    // Load configuration
    let config = config::load_config().expect("Failed to load configuration");
    tracing::info!("Configuration loaded: {:?}", config);

    // Initialize shared state
    let app_state = state::AppState::new(config.clone());

    // Build router
    let app = Router::new()
        // Health check
        .route("/health", get(api::health::health_check))
        
        // Consensus endpoints
        .route("/consensus/algorithms", get(api::consensus::get_algorithms))
        .route("/consensus/config", get(api::consensus::get_config))
        .route("/consensus/select", post(api::consensus::select_algorithm))
        .route("/consensus/parameters", put(api::consensus::update_parameters))
        .route("/consensus/benchmark/start", post(api::consensus::start_benchmark))
        .route("/consensus/benchmark/:id", get(api::consensus::get_benchmark_result))
        .route("/consensus/benchmark/:id/stop", post(api::consensus::stop_benchmark))
        .route("/consensus/benchmark/history", get(api::consensus::get_benchmark_history))
        
        // Transaction endpoints
        .route("/transaction/submit", post(api::transaction::submit_transaction))
        .route("/transaction/:id", get(api::transaction::get_transaction))
        .route("/transaction/status", get(api::transaction::get_transaction_status))
        .route("/transaction/history", get(api::transaction::get_transaction_history))
        
        // Node endpoints
        .route("/node/list", get(api::node::list_nodes))
        .route("/node/:id", get(api::node::get_node))
        .route("/node/stats", get(api::node::get_node_stats))
        
        // Performance endpoints
        .route("/performance/metrics", get(api::performance::get_metrics))
        .route("/performance/history", get(api::performance::get_history))
        .route("/performance/comparison", get(api::performance::get_comparison))
        
        // Analysis endpoints
        .route("/analysis/report", get(api::analysis::get_report))
        .route("/analysis/trends", get(api::analysis::get_trends))
        
        // Add middleware
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    let addr = SocketAddr::from((config.server.host.parse::<std::net::IpAddr>().unwrap_or(std::net::IpAddr::from([127, 0, 0, 1])), config.server.port));
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(
        listener,
        app,
    )
    .await
    .expect("Server error");
}
