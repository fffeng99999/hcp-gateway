use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

// ============== Consensus Models ==============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusAlgorithm {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    pub current_algorithm: String,
    pub parameters: std::collections::HashMap<String, serde_json::Value>,
    pub is_active: bool,
    pub last_updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectAlgorithmRequest {
    pub algorithm_id: String,
    pub parameters: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    pub algorithm_id: String,
    pub parameters: std::collections::HashMap<String, serde_json::Value>,
    pub duration: u64,
    pub transaction_rate: u64,
    pub node_count: u32,
    pub fault_injection_config: Option<FaultInjectionConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaultInjectionConfig {
    pub fault_type: String,
    pub severity: u32,
    pub affected_nodes: Vec<String>,
    pub duration: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub benchmark_id: String,
    pub algorithm_id: String,
    pub parameters: std::collections::HashMap<String, serde_json::Value>,
    pub start_time: String,
    pub end_time: Option<String>,
    pub duration: u64,
    pub metrics: PerformanceMetrics,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub throughput: f64,      // TPS (transactions per second)
    pub latency: f64,         // ms
    pub latency_p99: f64,     // 99th percentile latency
    pub latency_p999: f64,    // 99.9th percentile latency
    pub finality_time: f64,   // ms
    pub network_bandwidth: f64, // Mbps
    pub cpu_usage: f64,       // %
    pub memory_usage: f64,    // MB
}

// ============== Transaction Models ==============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub status: String,
    pub timestamp: String,
    pub payload: serde_json::Value,
    pub result: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitTransactionRequest {
    pub payload: serde_json::Value,
}

// ============== Node Models ==============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub name: String,
    pub address: String,
    pub status: String,
    pub role: String,
    pub last_heartbeat: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStats {
    pub node_id: String,
    pub online_count: u32,
    pub offline_count: u32,
    pub total_count: u32,
    pub average_latency: f64,
}

// ============== Generic Response Wrapper ==============

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T: Serialize> {
    pub code: u16,
    pub message: String,
    pub data: Option<T>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T, message: impl Into<String>) -> Self {
        ApiResponse {
            code: 200,
            message: message.into(),
            data: Some(data),
        }
    }

    pub fn empty_ok(message: impl Into<String>) -> ApiResponse<()> {
        ApiResponse {
            code: 200,
            message: message.into(),
            data: Some(()),
        }
    }
}

// ============== Update Parameters Request ==============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateParametersRequest {
    pub algorithm_id: String,
    pub param_name: String,
    pub value: serde_json::Value,
}

// ============== Stop Benchmark Response ==============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopBenchmarkResponse {
    pub benchmark_id: String,
    pub status: String,
    pub message: String,
}

// ============== Analysis Models ==============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisReport {
    pub report_id: String,
    pub created_at: String,
    pub title: String,
    pub summary: String,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendData {
    pub timestamp: String,
    pub algorithm: String,
    pub throughput: f64,
    pub latency: f64,
}
