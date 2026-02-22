use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

// ============== Generic Response Wrapper ==============

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    pub timestamp: String,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        ApiResponse {
            code: 0,
            message: "success".to_string(),
            data: Some(data),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn error(code: i32, message: impl Into<String>) -> Self {
        ApiResponse {
            code,
            message: message.into(),
            data: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

// ============== Auth Models ==============

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 3, message = "Username too short"))]
    pub username: String,
    #[validate(length(min = 6, message = "Password too short"))]
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: SystemUser,
}

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
    pub parameters: HashMap<String, serde_json::Value>,
    pub is_active: bool,
    pub last_updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SelectAlgorithmRequest {
    #[serde(rename = "algorithmId")]
    #[validate(length(min = 1))]
    pub algorithm_id: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateParametersRequest {
    #[serde(rename = "algorithmId")]
    #[validate(length(min = 1))]
    pub algorithm_id: String,
    #[serde(rename = "paramName")]
    #[validate(length(min = 1))]
    pub param_name: String,
    pub value: serde_json::Value,
}

// ============== Benchmark Models ==============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    pub algorithm_id: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub duration: u64,
    pub transaction_rate: u64,
    pub node_count: u32,
    pub fault_injection_config: Option<FaultInjectionConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateBenchmarkParams {
    #[validate(length(min = 1))]
    pub name: String,
    pub config: BenchmarkConfig,
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
    pub start_time: String,
    pub end_time: Option<String>,
    pub metrics: PerformanceMetrics,
    pub status: String, // running, completed, failed, paused
}

// ============== Node Models ==============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub name: String,
    pub address: String,
    pub status: String, // online, offline
    pub role: String,   // leader, validator, observer
    pub last_heartbeat: String,
    pub health_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStats {
    pub online_count: u32,
    pub offline_count: u32,
    pub total_count: u32,
    pub average_latency: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct NodeRegistrationRequest {
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(url)]
    pub address: String,
    pub role: String,
    pub public_key: String,
}

// ============== Performance Models ==============

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerformanceMetrics {
    pub throughput: f64, // TPS
    pub latency: f64,    // ms
    pub latency_p99: f64,
    pub latency_p999: f64,
    pub finality_time: f64,     // ms
    pub network_bandwidth: f64, // Mbps
    pub cpu_usage: f64,         // %
    pub memory_usage: f64,      // MB
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ExportParams {
    #[validate(length(min = 1))]
    pub format: String, // csv, json, excel
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct HistoryQueryParams {
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    #[validate(range(min = 1, max = 1000))]
    pub limit: Option<usize>,
}

// ============== Transaction Models ==============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub status: String, // pending, confirmed, failed
    pub timestamp: String,
    pub block_height: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct TransactionSubmitRequest {
    pub payload: serde_json::Value, // Flexible payload
    #[validate(range(min = 1, max = 10000))]
    pub rate_limit: Option<u32>,
    #[validate(range(min = 1, max = 1000))]
    pub batch_size: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct TransactionQueryParams {
    pub status: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    #[validate(range(min = 1, max = 100))]
    pub limit: Option<usize>,
    #[validate(range(min = 0))]
    pub offset: Option<usize>,
}

// ============== Anti-Manipulation Models ==============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiManipulationConfig {
    pub strategies: HashMap<String, bool>, // strategy_name -> enabled
    pub thresholds: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManipulationEvent {
    pub id: String,
    pub event_type: String, // front_running, wash_trading, etc.
    pub timestamp: String,
    pub severity: String, // low, medium, high
    pub details: serde_json::Value,
}

// ============== Analysis Models ==============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisReport {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub created_at: String,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct GenerateReportRequest {
    #[validate(length(min = 1))]
    pub title: String,
    pub content: String,
    #[validate(length(min = 1))]
    pub format: String, // pdf, docx
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendData {
    pub timestamp: String,
    pub metric: String,
    pub value: f64,
}

// ============== Settings Models ==============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralSettings {
    pub system_name: String,
    pub version: String,
    pub debug_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSettings {
    pub p2p_port: u16,
    pub max_peers: u32,
    pub bandwidth_limit: u32, // Mbps
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageSettings {
    pub data_path: String,
    pub retention_days: u32,
    pub compression_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    pub jwt_secret: String, // Should be masked in responses
    pub ip_whitelist: Vec<String>,
    pub enable_ssl: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    pub email_enabled: bool,
    pub webhook_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSettings {
    pub auto_backup: bool,
    pub interval_hours: u32,
    pub max_backups: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemUser {
    pub id: String,
    pub username: String,
    pub role: String, // admin, viewer
    pub email: String,
    pub created_at: String,
    pub status: String,
    pub last_login: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRecord {
    pub id: String,
    pub filename: String,
    pub size_bytes: u64,
    pub created_at: String,
    pub status: String, // success, failed
}
