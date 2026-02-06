use crate::models::{
    ApiResponse, BenchmarkResult, CreateBenchmarkParams, PerformanceMetrics
};
use crate::common::state::AppState;
use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;

// GET /benchmarks
pub async fn list_benchmarks(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<Vec<BenchmarkResult>>> {
    let benchmarks = state.benchmarks.read().await;
    Json(ApiResponse::success(benchmarks.clone()))
}

// POST /benchmarks
pub async fn create_benchmark(
    State(state): State<Arc<AppState>>,
    Json(params): Json<CreateBenchmarkParams>,
) -> Json<ApiResponse<BenchmarkResult>> {
    let mut benchmarks = state.benchmarks.write().await;
    
    let new_benchmark = BenchmarkResult {
        benchmark_id: uuid::Uuid::new_v4().to_string(),
        algorithm_id: params.config.algorithm_id,
        start_time: "".to_string(), // Not started yet
        end_time: None,
        metrics: PerformanceMetrics::default(),
        status: "pending".to_string(),
    };
    
    benchmarks.push(new_benchmark.clone());
    Json(ApiResponse::success(new_benchmark))
}

// GET /benchmarks/:id
pub async fn get_benchmark(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<BenchmarkResult>> {
    let benchmarks = state.benchmarks.read().await;
    if let Some(bench) = benchmarks.iter().find(|b| b.benchmark_id == id) {
        Json(ApiResponse::success(bench.clone()))
    } else {
        Json(ApiResponse::error(404, "Benchmark not found"))
    }
}

// POST /benchmarks/:id/start
pub async fn start_benchmark(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<String>> {
    let mut active = state.active_benchmark.write().await;
    if active.is_some() {
        return Json(ApiResponse::error(409, "Another benchmark is already running"));
    }

    let mut benchmarks = state.benchmarks.write().await;
    if let Some(bench) = benchmarks.iter_mut().find(|b| b.benchmark_id == id) {
        if bench.status == "running" {
             return Json(ApiResponse::error(400, "Benchmark is already running"));
        }
        
        bench.status = "running".to_string();
        bench.start_time = chrono::Utc::now().to_rfc3339();
        bench.end_time = None;
        *active = Some(id.clone());
        
        Json(ApiResponse::success("Benchmark started".to_string()))
    } else {
        Json(ApiResponse::error(404, "Benchmark not found"))
    }
}

// POST /benchmarks/:id/stop
pub async fn stop_benchmark(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<String>> {
    let mut active = state.active_benchmark.write().await;
    let mut benchmarks = state.benchmarks.write().await;
    
    if let Some(bench) = benchmarks.iter_mut().find(|b| b.benchmark_id == id) {
        if bench.status != "running" && bench.status != "paused" {
             return Json(ApiResponse::error(400, "Benchmark is not running"));
        }
        
        bench.status = "stopped".to_string();
        bench.end_time = Some(chrono::Utc::now().to_rfc3339());
        
        if let Some(active_id) = &*active {
            if active_id == &id {
                *active = None;
            }
        }
        
        Json(ApiResponse::success("Benchmark stopped".to_string()))
    } else {
        Json(ApiResponse::error(404, "Benchmark not found"))
    }
}

// POST /benchmarks/:id/pause
pub async fn pause_benchmark(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<String>> {
    let mut benchmarks = state.benchmarks.write().await;
    
    if let Some(bench) = benchmarks.iter_mut().find(|b| b.benchmark_id == id) {
        if bench.status != "running" {
             return Json(ApiResponse::error(400, "Benchmark is not running"));
        }
        
        bench.status = "paused".to_string();
        Json(ApiResponse::success("Benchmark paused".to_string()))
    } else {
        Json(ApiResponse::error(404, "Benchmark not found"))
    }
}

// DELETE /benchmarks/:id
pub async fn delete_benchmark(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<String>> {
    let mut benchmarks = state.benchmarks.write().await;
    
    if let Some(pos) = benchmarks.iter().position(|b| b.benchmark_id == id) {
        benchmarks.remove(pos);
        Json(ApiResponse::success("Benchmark deleted".to_string()))
    } else {
        Json(ApiResponse::error(404, "Benchmark not found"))
    }
}

