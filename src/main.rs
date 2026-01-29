mod config;
mod error;
mod models;
mod state;
mod tasks;
mod middleware;
mod api;
mod services;
mod data;
mod router;

use std::sync::Arc;
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

    // Initialize application state (mock data included)
    let app_state = Arc::new(state::AppState::new(mock_data));

    // Build router
    let app = router::create_router(app_state);

    // Start server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .map_err(|e| format!("Failed to bind to port 8080: {}", e))?;

    tracing::info!("Server listening on http://127.0.0.1:8080");

    axum::serve(listener, app)
        .await
        .map_err(|e| format!("Server error: {}", e))?;

    Ok(())
}
