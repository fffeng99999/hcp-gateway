use hcp_gateway::{
    config, data, router, state,
};
use hcp_gateway::grpc_client::ConsensusClient;
use std::sync::atomic::AtomicBool;
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

    // Initialize Consensus Client
    let consensus_grpc_addr = std::env::var("HCP_CONSENSUS_GRPC_ADDR")
        .unwrap_or_else(|_| "tcp://127.0.0.1:50051".to_string());
    
    let consensus_healthy = Arc::new(AtomicBool::new(true));
    
    tracing::info!("Connecting to Consensus Service at {}", consensus_grpc_addr);
    let consensus_client = match ConsensusClient::connect(consensus_grpc_addr, consensus_healthy.clone()).await {
        Ok(client) => Some(client),
        Err(e) => {
            tracing::error!("Failed to connect to Consensus Service: {}", e);
            consensus_healthy.store(false, std::sync::atomic::Ordering::SeqCst);
            None
        }
    };

    // Initialize application state (mock data included)
    let app_state = Arc::new(state::AppState::new(mock_data, consensus_client, consensus_healthy));

    // Load configuration
    let config = config::Config::default();

    // Build router
    let app = router::create_router(app_state);

    // Start server
    let addr = format!("{}:{}", config.server_addr, config.server_port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| format!("Failed to bind to {}: {}", addr, e))?;

    tracing::info!("Server listening on http://{}", addr);

    axum::serve(listener, app)
        .await
        .map_err(|e| format!("Server error: {}", e))?;

    Ok(())
}
