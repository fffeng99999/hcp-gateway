use crate::config::Config;
use crate::models::{BenchmarkResult, ConsensusConfig, Node, Transaction};
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use std::collections::HashMap;
use uuid::Uuid;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    // In-memory storage for mock data
    pub consensus_config: Arc<RwLock<ConsensusConfig>>,
    pub benchmarks: Arc<RwLock<HashMap<String, BenchmarkResult>>>,
    pub transactions: Arc<RwLock<HashMap<String, Transaction>>>,
    pub nodes: Arc<RwLock<HashMap<String, Node>>>,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let mut benchmarks = HashMap::new();
        let mut transactions = HashMap::new();
        let mut nodes = HashMap::new();

        // Initialize with mock data
        Self::init_mock_benchmarks(&mut benchmarks);
        Self::init_mock_transactions(&mut transactions);
        Self::init_mock_nodes(&mut nodes);

        AppState {
            config,
            consensus_config: Arc::new(RwLock::new(Self::create_default_consensus_config())),
            benchmarks: Arc::new(RwLock::new(benchmarks)),
            transactions: Arc::new(RwLock::new(transactions)),
            nodes: Arc::new(RwLock::new(nodes)),
        }
    }

    fn create_default_consensus_config() -> ConsensusConfig {
        let mut parameters = HashMap::new();
        parameters.insert("f".to_string(), serde_json::json!(1));
        parameters.insert("nodeSelectionMethod".to_string(), serde_json::json!("equity"));
        parameters.insert("hashVerification".to_string(), serde_json::json!(true));

        ConsensusConfig {
            current_algorithm: "tPBFT".to_string(),
            parameters: std::collections::HashMap::new(),
            is_active: true,
            last_updated: chrono::Utc::now().to_rfc3339(),
        }
    }

    fn init_mock_benchmarks(benchmarks: &mut HashMap<String, BenchmarkResult>) {
        let mut metrics = crate::models::PerformanceMetrics {
            throughput: 5000.0,
            latency: 150.0,
            latency_p99: 200.0,
            latency_p999: 250.0,
            finality_time: 300.0,
            network_bandwidth: 100.0,
            cpu_usage: 45.0,
            memory_usage: 512.0,
        };

        for i in 0..5 {
            let benchmark_id = Uuid::new_v4().to_string();
            let algorithm = match i % 4 {
                0 => "tPBFT",
                1 => "PBFT",
                2 => "HotStuff",
                _ => "Leios",
            };

            // Vary metrics slightly for different algorithms
            metrics.throughput = 4000.0 + (i as f64) * 500.0;
            metrics.latency = 100.0 + (i as f64) * 30.0;

            benchmarks.insert(
                benchmark_id.clone(),
                BenchmarkResult {
                    benchmark_id,
                    algorithm_id: algorithm.to_string(),
                    parameters: std::collections::HashMap::new(),
                    start_time: chrono::Utc::now().to_rfc3339(),
                    end_time: Some(chrono::Utc::now().to_rfc3339()),
                    duration: 600,
                    metrics: metrics.clone(),
                    status: "completed".to_string(),
                },
            );
        }
    }

    fn init_mock_transactions(transactions: &mut HashMap<String, Transaction>) {
        for i in 0..10 {
            let tx_id = Uuid::new_v4().to_string();
            transactions.insert(
                tx_id.clone(),
                Transaction {
                    id: tx_id,
                    status: if i % 2 == 0 { "success" } else { "pending" }.to_string(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    payload: serde_json::json!({
                        "from": format!("account_{}", i),
                        "to": format!("account_{}", (i + 1) % 10),
                        "amount": 1000 + i * 100,
                    }),
                    result: Some(serde_json::json!({
                        "block_height": 1000 + i as u64,
                        "tx_hash": format!("0x{:064x}", i),
                    })),
                },
            );
        }
    }

    fn init_mock_nodes(nodes: &mut HashMap<String, Node>) {
        for i in 0..5 {
            let node_id = format!("node_{}", i);
            nodes.insert(
                node_id.clone(),
                Node {
                    id: node_id,
                    name: format!("Node {}", i),
                    address: format!("192.168.1.{}", 100 + i),
                    status: if i < 4 { "online" } else { "offline" }.to_string(),
                    role: if i == 0 { "leader" } else { "validator" }.to_string(),
                    last_heartbeat: chrono::Utc::now().to_rfc3339(),
                },
            );
        }
    }
}
