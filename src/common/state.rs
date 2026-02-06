use crate::models::*;
use crate::utils::mock_data::MockData;
use crate::services::consensus_client::ConsensusClient;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    // Consensus
    pub algorithms: Arc<RwLock<Vec<ConsensusAlgorithm>>>,
    pub consensus_config: Arc<RwLock<ConsensusConfig>>,
    pub consensus_client: Option<ConsensusClient>,
    pub consensus_healthy: Arc<AtomicBool>,
    
    // Benchmarks
    pub benchmarks: Arc<RwLock<Vec<BenchmarkResult>>>,
    pub active_benchmark: Arc<RwLock<Option<String>>>, // ID of running benchmark
    
    // Transactions
    pub transactions: Arc<RwLock<Vec<Transaction>>>,
    
    // Nodes
    pub nodes: Arc<RwLock<Vec<Node>>>,
    
    // Performance
    pub performance_history: Arc<RwLock<Vec<PerformanceMetrics>>>, // Simplified history
    
    // Anti-Manipulation
    pub anti_manipulation_config: Arc<RwLock<AntiManipulationConfig>>,
    pub manipulation_events: Arc<RwLock<Vec<ManipulationEvent>>>,
    
    // Analysis
    pub analysis_reports: Arc<RwLock<Vec<AnalysisReport>>>,
    
    // Settings
    pub general_settings: Arc<RwLock<GeneralSettings>>,
    pub network_settings: Arc<RwLock<NetworkSettings>>,
    pub storage_settings: Arc<RwLock<StorageSettings>>,
    pub security_settings: Arc<RwLock<SecuritySettings>>,
    pub notification_settings: Arc<RwLock<NotificationSettings>>,
    pub backup_settings: Arc<RwLock<BackupSettings>>,
    pub users: Arc<RwLock<Vec<SystemUser>>>,
    pub backups: Arc<RwLock<Vec<BackupRecord>>>,
}

impl AppState {
    pub fn new(mock_data: MockData, consensus_client: Option<ConsensusClient>, consensus_healthy: Arc<AtomicBool>) -> Self {
        // Convert mock data to models
        let algorithms: Vec<ConsensusAlgorithm> = mock_data.algorithms.into_iter().map(|v| {
            serde_json::from_value(v).unwrap_or(ConsensusAlgorithm {
                id: "unknown".to_string(),
                name: "Unknown".to_string(),
                description: "".to_string(),
                category: "".to_string(),
            })
        }).collect();

        let benchmarks: Vec<BenchmarkResult> = mock_data.benchmarks.into_iter().map(|v| {
            serde_json::from_value(v).unwrap_or(BenchmarkResult {
                benchmark_id: "unknown".to_string(),
                algorithm_id: "".to_string(),
                start_time: "".to_string(),
                end_time: None,
                metrics: PerformanceMetrics::default(),
                status: "failed".to_string(),
            })
        }).collect();

        let transactions: Vec<Transaction> = mock_data.transactions.into_iter().map(|v| {
            serde_json::from_value(v).unwrap_or(Transaction {
                id: "unknown".to_string(),
                from: "".to_string(),
                to: "".to_string(),
                amount: 0.0,
                status: "failed".to_string(),
                timestamp: "".to_string(),
                block_height: None,
            })
        }).collect();

        let nodes: Vec<Node> = mock_data.nodes.into_iter().map(|v| {
            serde_json::from_value(v).unwrap_or(Node {
                id: "unknown".to_string(),
                name: "Unknown".to_string(),
                address: "".to_string(),
                status: "offline".to_string(),
                role: "unknown".to_string(),
                last_heartbeat: "".to_string(),
                health_score: 0.0,
            })
        }).collect();

        let consensus_config = serde_json::from_value(mock_data.consensus_config).unwrap_or(ConsensusConfig {
            current_algorithm: "tPBFT".to_string(),
            parameters: HashMap::new(),
            is_active: true,
            last_updated: "".to_string(),
        });

        AppState {
            algorithms: Arc::new(RwLock::new(if algorithms.is_empty() { Self::default_algorithms() } else { algorithms })),
            consensus_config: Arc::new(RwLock::new(consensus_config)), // Use loaded or default if empty (but check if empty is valid)
            consensus_client,
            consensus_healthy,
            
            benchmarks: Arc::new(RwLock::new(benchmarks)),
            active_benchmark: Arc::new(RwLock::new(None)),
            transactions: Arc::new(RwLock::new(if transactions.is_empty() { Self::default_transactions() } else { transactions })),
            nodes: Arc::new(RwLock::new(if nodes.is_empty() { Self::default_nodes() } else { nodes })),
            performance_history: Arc::new(RwLock::new(Vec::new())),
            anti_manipulation_config: Arc::new(RwLock::new(Self::default_anti_manipulation_config())),
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
            system_name: "HCP Gateway".to_string(),
            version: "1.0.0".to_string(),
            debug_mode: true,
        }
    }

    fn default_network_settings() -> NetworkSettings {
        NetworkSettings {
            p2p_port: 9000,
            max_peers: 50,
            bandwidth_limit: 1000,
        }
    }

    fn default_storage_settings() -> StorageSettings {
        StorageSettings {
            data_path: "./data".to_string(),
            retention_days: 30,
            compression_enabled: true,
        }
    }

    fn default_security_settings() -> SecuritySettings {
        SecuritySettings {
            jwt_secret: "secret_key".to_string(),
            ip_whitelist: vec!["127.0.0.1".to_string()],
            enable_ssl: false,
        }
    }

    fn default_notification_settings() -> NotificationSettings {
        NotificationSettings {
            email_enabled: false,
            webhook_url: "".to_string(),
        }
    }

    fn default_backup_settings() -> BackupSettings {
        BackupSettings {
            auto_backup: true,
            interval_hours: 24,
            max_backups: 7,
        }
    }

    fn default_users() -> Vec<SystemUser> {
        vec![
            SystemUser {
                id: "user_admin".to_string(),
                username: "admin".to_string(),
                role: "admin".to_string(),
                email: "admin@hcp.com".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
                status: "active".to_string(),
                last_login: None,
            },
        ]
    }
}
