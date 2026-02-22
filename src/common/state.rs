use crate::models::*;
use crate::services::consensus_client::ConsensusClient;
use crate::services::server_client::ServerClient;
use crate::utils::mock_data::MockData;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    // 共识相关状态
    pub algorithms: Arc<RwLock<Vec<ConsensusAlgorithm>>>,
    pub consensus_config: Arc<RwLock<ConsensusConfig>>,
    pub consensus_client: Option<ConsensusClient>,
    pub server_client: Option<ServerClient>,
    pub consensus_healthy: Arc<AtomicBool>,
    pub server_healthy: Arc<AtomicBool>,

    // 基准测试任务与当前运行中的基准测试 ID
    pub benchmarks: Arc<RwLock<Vec<BenchmarkResult>>>,
    pub active_benchmark: Arc<RwLock<Option<String>>>,

    // 交易列表
    pub transactions: Arc<RwLock<Vec<Transaction>>>,

    // 节点信息
    pub nodes: Arc<RwLock<Vec<Node>>>,

    // 性能指标历史（简化版）
    pub performance_history: Arc<RwLock<Vec<PerformanceMetrics>>>,

    // 反操纵配置与事件
    pub anti_manipulation_config: Arc<RwLock<AntiManipulationConfig>>,
    pub manipulation_events: Arc<RwLock<Vec<ManipulationEvent>>>,

    // 分析报告
    pub analysis_reports: Arc<RwLock<Vec<AnalysisReport>>>,

    // 设置相关状态
    pub general_settings: Arc<RwLock<GeneralSettings>>,
    pub network_settings: Arc<RwLock<NetworkSettings>>,
    pub storage_settings: Arc<RwLock<StorageSettings>>,
    pub security_settings: Arc<RwLock<SecuritySettings>>,
    pub notification_settings: Arc<RwLock<NotificationSettings>>,
    pub backup_settings: Arc<RwLock<BackupSettings>>,
    pub users: Arc<RwLock<Vec<SystemUser>>>,
    pub backups: Arc<RwLock<Vec<BackupRecord>>>,
    pub general_cache: Arc<RwLock<SettingsCache<GeneralSettings>>>,
    pub network_cache: Arc<RwLock<SettingsCache<NetworkSettings>>>,
    pub storage_cache: Arc<RwLock<SettingsCache<StorageSettings>>>,
    pub security_cache: Arc<RwLock<SettingsCache<SecuritySettings>>>,
    pub notification_cache: Arc<RwLock<SettingsCache<NotificationSettings>>>,
    pub backup_cache: Arc<RwLock<SettingsCache<BackupSettings>>>,
    pub users_cache: Arc<RwLock<SettingsCache<Vec<SystemUser>>>>,
    pub backups_cache: Arc<RwLock<SettingsCache<Vec<BackupRecord>>>>,
    // 全局配置版本号，所有设置写操作会自增
    pub config_version: Arc<AtomicU64>,
}

#[derive(Clone)]
pub struct SettingsCache<T> {
    pub value: Option<T>,
    pub updated_at: Option<Instant>,
}

impl<T> SettingsCache<T> {
    pub fn new() -> Self {
        Self {
            value: None,
            updated_at: None,
        }
    }
}

impl<T> Default for SettingsCache<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new(
        mock_data: MockData,
        consensus_client: Option<ConsensusClient>,
        server_client: Option<ServerClient>,
        consensus_healthy: Arc<AtomicBool>,
        server_healthy: Arc<AtomicBool>,
    ) -> Self {
        // 将从 JSON 中读取到的原始 MockData 转换为领域模型
        let algorithms: Vec<ConsensusAlgorithm> = mock_data
            .algorithms
            .into_iter()
            .map(|v| {
                serde_json::from_value(v).unwrap_or(ConsensusAlgorithm {
                    id: "unknown".to_string(),
                    name: "Unknown".to_string(),
                    description: "".to_string(),
                    category: "".to_string(),
                })
            })
            .collect();

        let benchmarks: Vec<BenchmarkResult> = mock_data
            .benchmarks
            .into_iter()
            .map(|v| {
                serde_json::from_value(v).unwrap_or(BenchmarkResult {
                    benchmark_id: "unknown".to_string(),
                    algorithm_id: "".to_string(),
                    start_time: "".to_string(),
                    end_time: None,
                    metrics: PerformanceMetrics::default(),
                    status: "failed".to_string(),
                })
            })
            .collect();

        let transactions: Vec<Transaction> = mock_data
            .transactions
            .into_iter()
            .map(|v| {
                serde_json::from_value(v).unwrap_or(Transaction {
                    id: "unknown".to_string(),
                    from: "".to_string(),
                    to: "".to_string(),
                    amount: 0.0,
                    status: "failed".to_string(),
                    timestamp: "".to_string(),
                    block_height: None,
                })
            })
            .collect();

        let nodes: Vec<Node> = mock_data
            .nodes
            .into_iter()
            .map(|v| {
                serde_json::from_value(v).unwrap_or(Node {
                    id: "unknown".to_string(),
                    name: "Unknown".to_string(),
                    address: "".to_string(),
                    status: "offline".to_string(),
                    role: "unknown".to_string(),
                    last_heartbeat: "".to_string(),
                    health_score: 0.0,
                })
            })
            .collect();

        let consensus_config =
            serde_json::from_value(mock_data.consensus_config).unwrap_or(ConsensusConfig {
                current_algorithm: "tPBFT".to_string(),
                parameters: HashMap::new(),
                is_active: true,
                last_updated: "".to_string(),
            });

        AppState {
            algorithms: Arc::new(RwLock::new(if algorithms.is_empty() {
                Self::default_algorithms()
            } else {
                algorithms
            })),
            consensus_config: Arc::new(RwLock::new(consensus_config)), // 使用文件中的配置，如为空则回退到默认配置
            consensus_client,
            server_client,
            consensus_healthy,
            server_healthy,

            benchmarks: Arc::new(RwLock::new(benchmarks)),
            active_benchmark: Arc::new(RwLock::new(None)),
            transactions: Arc::new(RwLock::new(if transactions.is_empty() {
                Self::default_transactions()
            } else {
                transactions
            })),
            nodes: Arc::new(RwLock::new(if nodes.is_empty() {
                Self::default_nodes()
            } else {
                nodes
            })),
            performance_history: Arc::new(RwLock::new(Vec::new())),
            anti_manipulation_config: Arc::new(RwLock::new(
                Self::default_anti_manipulation_config(),
            )),
            manipulation_events: Arc::new(RwLock::new(Vec::new())),
            analysis_reports: Arc::new(RwLock::new(Vec::new())),
            general_settings: Arc::new(RwLock::new(Self::default_general_settings())),
            network_settings: Arc::new(RwLock::new(Self::default_network_settings())),
            storage_settings: Arc::new(RwLock::new(Self::default_storage_settings())),
            security_settings: Arc::new(RwLock::new(Self::default_security_settings())),
            notification_settings: Arc::new(RwLock::new(Self::default_notification_settings())),
            backup_settings: Arc::new(RwLock::new(Self::default_backup_settings())),
            users: Arc::new(RwLock::new(Self::default_users())),
            backups: Arc::new(RwLock::new(Vec::new())),
            general_cache: Arc::new(RwLock::new(SettingsCache::new())),
            network_cache: Arc::new(RwLock::new(SettingsCache::new())),
            storage_cache: Arc::new(RwLock::new(SettingsCache::new())),
            security_cache: Arc::new(RwLock::new(SettingsCache::new())),
            notification_cache: Arc::new(RwLock::new(SettingsCache::new())),
            backup_cache: Arc::new(RwLock::new(SettingsCache::new())),
            users_cache: Arc::new(RwLock::new(SettingsCache::new())),
            backups_cache: Arc::new(RwLock::new(SettingsCache::new())),
            config_version: Arc::new(AtomicU64::new(1)),
        }
    }

    fn default_algorithms() -> Vec<ConsensusAlgorithm> {
        vec![
            ConsensusAlgorithm {
                id: "tPBFT".to_string(),
                name: "Trust-based PBFT".to_string(),
                description: "Optimized PBFT with trust mechanism".to_string(),
                category: "BFT".to_string(),
            },
            ConsensusAlgorithm {
                id: "HotStuff".to_string(),
                name: "HotStuff".to_string(),
                description: "Chained BFT consensus".to_string(),
                category: "BFT".to_string(),
            },
            ConsensusAlgorithm {
                id: "Raft".to_string(),
                name: "Raft".to_string(),
                description: "Crash fault tolerant consensus".to_string(),
                category: "CFT".to_string(),
            },
        ]
    }

    fn default_transactions() -> Vec<Transaction> {
        vec![
            Transaction {
                id: "tx_1".to_string(),
                from: "addr_1".to_string(),
                to: "addr_2".to_string(),
                amount: 100.0,
                status: "confirmed".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                block_height: Some(101),
            },
            Transaction {
                id: "tx_2".to_string(),
                from: "addr_3".to_string(),
                to: "addr_4".to_string(),
                amount: 50.0,
                status: "pending".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                block_height: None,
            },
        ]
    }

    fn default_nodes() -> Vec<Node> {
        vec![
            Node {
                id: "node_1".to_string(),
                name: "Validator 1".to_string(),
                address: "192.168.1.10:8000".to_string(),
                status: "online".to_string(),
                role: "validator".to_string(),
                last_heartbeat: chrono::Utc::now().to_rfc3339(),
                health_score: 98.5,
            },
            Node {
                id: "node_2".to_string(),
                name: "Validator 2".to_string(),
                address: "192.168.1.11:8000".to_string(),
                status: "online".to_string(),
                role: "validator".to_string(),
                last_heartbeat: chrono::Utc::now().to_rfc3339(),
                health_score: 99.0,
            },
            Node {
                id: "node_3".to_string(),
                name: "Observer 1".to_string(),
                address: "192.168.1.12:8000".to_string(),
                status: "offline".to_string(),
                role: "observer".to_string(),
                last_heartbeat: chrono::Utc::now().to_rfc3339(),
                health_score: 0.0,
            },
        ]
    }

    fn default_anti_manipulation_config() -> AntiManipulationConfig {
        let mut strategies = HashMap::new();
        strategies.insert("front_running_detection".to_string(), true);
        strategies.insert("wash_trading_detection".to_string(), true);

        let mut thresholds = HashMap::new();
        thresholds.insert("high_frequency_threshold".to_string(), 100.0);

        AntiManipulationConfig {
            strategies,
            thresholds,
        }
    }

    fn default_general_settings() -> GeneralSettings {
        GeneralSettings {
            system_name: "HCP-Bench".to_string(),
            version: "1.0.0".to_string(),
            debug_mode: true,
            language: "zh-CN".to_string(),
            timezone: "Asia/Shanghai".to_string(),
            log_level: "info".to_string(),
            data_retention: 30,
            auto_cleanup: true,
            performance_monitor: true,
            rate_limit: 1000,
        }
    }

    fn default_network_settings() -> NetworkSettings {
        NetworkSettings {
            listen_address: "0.0.0.0".to_string(),
            p2p_port: 26656,
            rpc_port: 26657,
            max_connections: 100,
            max_inbound: 80,
            max_outbound: 20,
            upnp: false,
            nat_traversal: false,
            upload_bandwidth: 1000,
            download_bandwidth: 1000,
            seed_nodes: vec![],
        }
    }

    fn default_storage_settings() -> StorageSettings {
        StorageSettings {
            data_path: "./data".to_string(),
            log_path: "./logs".to_string(),
            db_type: "leveldb".to_string(),
            cache_size: 1024,
            compression: true,
            compression_algo: "snappy".to_string(),
            auto_archive: false,
            archive_threshold: 30,
        }
    }

    fn default_security_settings() -> SecuritySettings {
        SecuritySettings {
            jwt_enabled: true,
            jwt_expiration: 3600,
            two_factor_auth: false,
            password_policy: vec![
                "min_length:8".to_string(),
                "require_uppercase".to_string(),
                "require_number".to_string(),
            ],
            access_log: true,
            login_lockout: true,
            lockout_threshold: 5,
            lockout_duration: 300,
            data_encryption: true,
            encryption_algo: "aes-256-gcm".to_string(),
            tls_enabled: false,
            tls_version: "1.2".to_string(),
            ip_whitelist: vec!["127.0.0.1".to_string()],
        }
    }

    fn default_notification_settings() -> NotificationSettings {
        NotificationSettings {
            email_enabled: false,
            smtp_host: "".to_string(),
            smtp_port: 25,
            sender_email: "".to_string(),
            recipients: vec![],
            webhook_enabled: false,
            webhook_url: "".to_string(),
            webhook_token: "".to_string(),
            system_events: vec![],
            performance_events: vec![],
            security_events: vec![],
        }
    }

    fn default_backup_settings() -> BackupSettings {
        BackupSettings {
            auto_backup: true,
            frequency: "daily".to_string(),
            retention_count: 7,
            backup_path: "./backups".to_string(),
        }
    }

    fn default_users() -> Vec<SystemUser> {
        vec![SystemUser {
            id: "user_admin".to_string(),
            username: "admin".to_string(),
            role: "admin".to_string(),
            email: "admin@hcp.com".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            status: "active".to_string(),
            last_login: None,
        }]
    }
}
