use axum::{
    extract::{Path, State},
    Json,
};
use crate::{
    error::{ApiError, ApiResult},
    models::*,
    state::AppState,
};
use uuid::Uuid;

pub async fn get_algorithms(
    State(_state): State<AppState>,
) -> ApiResult<Json<Vec<ConsensusAlgorithm>>> {
    tracing::info!("Getting consensus algorithms");
    let algorithms = vec![
        ConsensusAlgorithm {
            id: "tPBFT".to_string(),
            name: "Trust-based PBFT".to_string(),
            description: "An improved PBFT with dynamic node selection".to_string(),
            category: "BFT-based".to_string(),
        },
        ConsensusAlgorithm {
            id: "PBFT".to_string(),
            name: "Practical Byzantine Fault Tolerance".to_string(),
            description: "Classic PBFT consensus".to_string(),
            category: "BFT-based".to_string(),
        },
        ConsensusAlgorithm {
            id: "HotStuff".to_string(),
            name: "HotStuff".to_string(),
            description: "Optimal resilience PBFT variant".to_string(),
            category: "Modern".to_string(),
        },
        ConsensusAlgorithm {
            id: "Leios".to_string(),
            name: "Leios".to_string(),
            description: "High-throughput consensus".to_string(),
            category: "Modern".to_string(),
        },
    ];
    Ok(Json(algorithms))
}

pub async fn get_config(
    State(state): State<AppState>,
) -> ApiResult<Json<ConsensusConfig>> {
    tracing::info!("Getting current consensus config");
    let config = state.consensus_config.read().await;
    Ok(Json(config.clone()))
}

pub async fn select_algorithm(
    State(state): State<AppState>,
    Json(payload): Json<SelectAlgorithmRequest>,
) -> ApiResult<Json<ConsensusConfig>> {
    tracing::info!("Selecting algorithm: {}", payload.algorithm_id);
    
    let mut config = state.consensus_config.write().await;
    config.current_algorithm = payload.algorithm_id;
    config.parameters = payload.parameters;
    config.last_updated = chrono::Utc::now().to_rfc3339();
    
    Ok(Json(config.clone()))
}

pub async fn update_parameters(
    State(state): State<AppState>,
    Json(payload): Json<UpdateParametersRequest>,
) -> ApiResult<Json<ConsensusConfig>> {
    tracing::info!(
        "Updating parameter {} for algorithm {}",
        payload.param_name,
        payload.algorithm_id
    );
    
    let mut config = state.consensus_config.write().await;
    config.parameters.insert(payload.param_name, payload.value);
    config.last_updated = chrono::Utc::now().to_rfc3339();
    
    Ok(Json(config.clone()))
}

pub async fn start_benchmark(
    State(state): State<AppState>,
    Json(payload): Json<BenchmarkConfig>,
) -> ApiResult<Json<serde_json::json!>> {
    tracing::info!("Starting benchmark for algorithm: {}", payload.algorithm_id);
    
    let benchmark_id = Uuid::new_v4().to_string();
    let result = BenchmarkResult {
        benchmark_id: benchmark_id.clone(),
        algorithm_id: payload.algorithm_id,
        parameters: payload.parameters,
        start_time: chrono::Utc::now().to_rfc3339(),
        end_time: None,
        duration: payload.duration,
        metrics: PerformanceMetrics {
            throughput: 0.0,
            latency: 0.0,
            latency_p99: 0.0,
            latency_p999: 0.0,
            finality_time: 0.0,
            network_bandwidth: 0.0,
            cpu_usage: 0.0,
            memory_usage: 0.0,
        },
        status: "running".to_string(),
    };
    
    let mut benchmarks = state.benchmarks.write().await;
    benchmarks.insert(benchmark_id.clone(), result);
    
    Ok(Json(serde_json::json!({
        "benchmark_id": benchmark_id,
        "status": "started",
    })))
}

pub async fn get_benchmark_result(
    State(state): State<AppState>,
    Path(benchmark_id): Path<String>,
) -> ApiResult<Json<BenchmarkResult>> {
    tracing::info!("Getting benchmark result: {}", benchmark_id);
    
    let benchmarks = state.benchmarks.read().await;
    benchmarks
        .get(&benchmark_id)
        .cloned()
        .map(Json)
        .ok_or_else(|| ApiError::NotFound(format!("Benchmark {} not found", benchmark_id)))
}

pub async fn stop_benchmark(
    State(state): State<AppState>,
    Path(benchmark_id): Path<String>,
) -> ApiResult<Json<StopBenchmarkResponse>> {
    tracing::info!("Stopping benchmark: {}", benchmark_id);
    
    let mut benchmarks = state.benchmarks.write().await;
    if let Some(benchmark) = benchmarks.get_mut(&benchmark_id) {
        benchmark.status = "stopped".to_string();
        benchmark.end_time = Some(chrono::Utc::now().to_rfc3339());
        
        Ok(Json(StopBenchmarkResponse {
            benchmark_id,
            status: "stopped".to_string(),
            message: "Benchmark stopped successfully".to_string(),
        }))
    } else {
        Err(ApiError::NotFound(format!("Benchmark {} not found", benchmark_id)))
    }
}

pub async fn get_benchmark_history(
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<BenchmarkResult>>> {
    tracing::info!("Getting benchmark history");
    
    let benchmarks = state.benchmarks.read().await;
    let history: Vec<BenchmarkResult> = benchmarks.values().cloned().collect();
    
    Ok(Json(history))
}
