use crate::data::MockData;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    pub algorithms: Arc<RwLock<Vec<Value>>>,
    pub benchmarks: Arc<RwLock<Vec<Value>>>,
    pub transactions: Arc<RwLock<Vec<Value>>>,
    pub nodes: Arc<RwLock<Vec<Value>>>,
    pub consensus_config: Arc<RwLock<Value>>,
}

impl AppState {
    pub fn new(mock_data: MockData) -> Self {
        AppState {
            algorithms: Arc::new(RwLock::new(mock_data.algorithms)),
            benchmarks: Arc::new(RwLock::new(mock_data.benchmarks)),
            transactions: Arc::new(RwLock::new(mock_data.transactions)),
            nodes: Arc::new(RwLock::new(mock_data.nodes)),
            consensus_config: Arc::new(RwLock::new(mock_data.consensus_config)),
        }
    }
}
