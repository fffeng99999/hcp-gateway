use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

// ============== 通用 API 响应包装结构 ==============

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

// ============== 认证相关模型 ==============

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

// ============== 共识配置与请求模型 ==============

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

// ============== 基准测试相关模型 ==============

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
    pub status: String, // 运行中 running、已完成 completed、失败 failed、暂停 paused
}

// ============== 节点相关模型 ==============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub name: String,
    pub address: String,
    pub status: String, // 在线 online、离线 offline
    pub role: String,   // 角色：leader、validator、observer
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

// ============== 性能指标与导出模型 ==============

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerformanceMetrics {
    pub throughput: f64, // 吞吐量（TPS）
    pub latency: f64,    // 延迟（毫秒 ms）
    pub latency_p99: f64,
    pub latency_p999: f64,
    pub finality_time: f64,     // 交易最终确认时间（毫秒 ms）
    pub network_bandwidth: f64, // 网络带宽（Mbps）
    pub cpu_usage: f64,         // CPU 使用率（百分比）
    pub memory_usage: f64,      // 内存占用（MB）
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ExportParams {
    #[validate(length(min = 1))]
    pub format: String, // 导出格式：csv、json、excel
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct HistoryQueryParams {
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    #[validate(range(min = 1, max = 1000))]
    pub limit: Option<usize>,
}

// ============== 交易相关模型 ==============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub status: String, // 状态：待确认 pending、已确认 confirmed、失败 failed
    pub timestamp: String,
    pub block_height: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct TransactionSubmitRequest {
    pub payload: serde_json::Value, // 交易负载，结构可扩展
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

// ============== 反操纵检测相关模型 ==============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiManipulationConfig {
    pub strategies: HashMap<String, bool>, // 策略名称 -> 是否启用
    pub thresholds: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManipulationEvent {
    pub id: String,
    pub event_type: String, // 事件类型：front_running、wash_trading 等
    pub timestamp: String,
    pub severity: String, // 严重程度：low、medium、high
    pub details: serde_json::Value,
}

// ============== 分析与报告相关模型 ==============

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
    pub format: String, // 报告格式：pdf、docx
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendData {
    pub timestamp: String,
    pub metric: String,
    pub value: f64,
}

// ============== 网关设置与系统配置模型 ==============

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
    pub bandwidth_limit: u32, // 对等网络带宽上限（Mbps）
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageSettings {
    pub data_path: String,
    pub retention_days: u32,
    pub compression_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    pub jwt_secret: String, // JWT 密钥，在 API 响应中应做脱敏处理
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
    pub role: String, // 角色：管理员 admin、只读 viewer
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
    pub status: String, // 备份状态：成功 success 或失败 failed
}
