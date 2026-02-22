use hcp_gateway::services::consensus_client::ConsensusClient;
use hcp_gateway::services::server_client::ServerClient;
use hcp_gateway::{api::router, common::state, config, utils::mock_data as data};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化 tracing 日志系统
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("hcp_gateway=debug".parse()?)
                .add_directive("info".parse()?),
        )
        .init();

    tracing::info!("Starting HCP Gateway...");

    // 从 JSON 文件中加载网关使用的模拟数据
    let mock_data = data::load_mock_data("data/mock_data.json")
        .await
        .map_err(|e| format!("Failed to load mock data: {}", e))?
        .unwrap_or_else(data::default_mock_data);

    // 初始化共识客户端（直接连接 Cosmos / CometBFT）
    let consensus_grpc_addr = std::env::var("HCP_CONSENSUS_GRPC_ADDR")
        .unwrap_or_else(|_| "http://127.0.0.1:9090".to_string());

    let consensus_healthy = Arc::new(AtomicBool::new(true));

    tracing::info!("Connecting to Consensus Service at {}", consensus_grpc_addr);
    let consensus_client = match ConsensusClient::connect(
        consensus_grpc_addr.clone(),
        consensus_healthy.clone(),
    )
    .await
    {
        Ok(client) => {
            tracing::info!("Connected to Consensus Service");
            Some(client)
        }
        Err(e) => {
            tracing::error!(
                "Failed to connect to Consensus Service at {}: {}",
                consensus_grpc_addr,
                e
            );
            consensus_healthy.store(false, std::sync::atomic::Ordering::SeqCst);
            None
        }
    };

    // 初始化后端服务客户端（HCP Server）
    let server_grpc_addr = std::env::var("HCP_SERVER_GRPC_ADDR")
        .unwrap_or_else(|_| "http://127.0.0.1:50051".to_string());

    let server_healthy = Arc::new(AtomicBool::new(true));

    tracing::info!("Connecting to Backend Server at {}", server_grpc_addr);
    let server_client =
        match ServerClient::connect(server_grpc_addr.clone(), server_healthy.clone()).await {
            Ok(client) => {
                tracing::info!("Connected to Backend Server");
                Some(client)
            }
            Err(e) => {
                tracing::error!(
                    "Failed to connect to Backend Server at {}: {}",
                    server_grpc_addr,
                    e
                );
                server_healthy.store(false, std::sync::atomic::Ordering::SeqCst);
                None
            }
        };

    // 初始化应用全局状态（包含模拟数据和后端客户端）
    let app_state = Arc::new(state::AppState::new(
        mock_data,
        consensus_client,
        server_client,
        consensus_healthy,
        server_healthy,
    ));

    // 加载网关配置
    let config = config::Config::default();

    // 构建 HTTP 路由
    let app = router::create_router(app_state);

    // 启动 HTTP 服务监听
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
