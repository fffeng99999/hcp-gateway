use serde_json::{json, Value};
use std::path::Path;
use tokio::fs;

#[derive(Clone, Debug)]
pub struct MockData {
    pub algorithms: Vec<Value>,
    pub benchmarks: Vec<Value>,
    pub transactions: Vec<Value>,
    pub nodes: Vec<Value>,
    pub consensus_config: Value,
}

/// Load mock data from JSON file
pub async fn load_mock_data<P: AsRef<Path>>(
    path: P,
) -> Result<Option<MockData>, Box<dyn std::error::Error>> {
    let path = path.as_ref();

    // Check if file exists
    if !path.exists() {
        tracing::warn!("Mock data file not found at {:?}, using defaults", path);
        return Ok(None);
    }

    // Read file
    let content = fs::read_to_string(path).await?;
    let data: Value = serde_json::from_str(&content)?;

    // Extract data sections
    let algorithms = data
        .get("algorithms")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let benchmarks = data
        .get("benchmarks")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let transactions = data
        .get("transactions")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let nodes = data
        .get("nodes")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let consensus_config = data
        .get("consensus_config")
        .cloned()
        .unwrap_or_else(|| json!({}));

    tracing::info!(
        "Loaded mock data: {} algorithms, {} benchmarks, {} transactions, {} nodes",
        algorithms.len(),
        benchmarks.len(),
        transactions.len(),
        nodes.len()
    );

    Ok(Some(MockData {
        algorithms,
        benchmarks,
        transactions,
        nodes,
        consensus_config,
    }))
}

/// Generate default mock data if file not found
pub fn default_mock_data() -> MockData {
    tracing::info!("Using default mock data");

    let algorithms = vec![
        json!({
            "id": "tPBFT",
            "name": "Trust-based PBFT",
            "description": "An improved PBFT algorithm",
            "category": "BFT-based"
        }),
        json!({
            "id": "PBFT",
            "name": "Practical Byzantine Fault Tolerance",
            "description": "Classic PBFT consensus",
            "category": "BFT-based"
        }),
    ];

    let benchmarks = vec![json!({
        "benchmark_id": "550e8400-e29b-41d4-a716-446655440001",
        "algorithm_id": "tPBFT",
        "parameters": { "f": 1 },
        "start_time": "2026-01-29T06:00:00Z",
        "end_time": "2026-01-29T06:10:00Z",
        "duration": 600,
        "status": "completed"
    })];

    let transactions = vec![json!({
        "id": "tx_001",
        "status": "success",
        "timestamp": "2026-01-29T06:05:00Z",
    })];

    let nodes = vec![json!({
        "id": "node_0",
        "name": "Node 0",
        "status": "online",
    })];

    let consensus_config = json!({
        "current_algorithm": "tPBFT",
        "is_active": true,
    });

    MockData {
        algorithms,
        benchmarks,
        transactions,
        nodes,
        consensus_config,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_mock_data() {
        let data = default_mock_data();
        assert!(!data.algorithms.is_empty());
        assert!(!data.benchmarks.is_empty());
        assert!(!data.transactions.is_empty());
        assert!(!data.nodes.is_empty());
    }
}
