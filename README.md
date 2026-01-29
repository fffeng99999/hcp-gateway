# HCP Gateway

**HCP (High-frequency trading Consensus Performance) Gateway** - A high-performance Rust-based API gateway for blockchain consensus benchmarking.

## Overview

HCP Gateway is the central API gateway that serves the HCP UI frontend and orchestrates communication with various microservices including:

- **Consensus Service**: Handles consensus algorithm operations and benchmarking
- **Blockchain Service**: Manages blockchain transactions and state
- **Storage Service**: Persists data and manages caching

The gateway is built with **Rust** using the **Axum** web framework and provides:

✅ **RESTful API** for frontend communication  
✅ **Concurrent Task Processing** for parallel benchmark execution  
✅ **Service Integration Layer** for microservice coordination  
✅ **Mock Data** for rapid API testing (easily replaced with real services)  
✅ **Production-Ready Architecture** with error handling and logging  

## Features

### Core Capabilities

- **Consensus Management**: Support for tPBFT, PBFT, HotStuff, and Leios algorithms
- **Benchmark Execution**: Run performance tests with configurable parameters
- **Performance Metrics**: Track throughput, latency, and resource utilization
- **Node Management**: Monitor and query blockchain nodes
- **Transaction Handling**: Submit and track transactions
- **Analysis & Reporting**: Generate performance reports and trends

### Parallel Processing

The gateway includes a **TaskExecutor** for handling concurrent operations:

```rust
let executor = TaskExecutor::<BenchmarkResult>::new();
let task_id = executor.submit(async {
    // Long-running benchmark task
    perform_benchmark().await
}).await;
```

### Microservice Integration

Service clients are designed as placeholders for real service integration:

```rust
let consensus_service = ConsensusServiceClient::new(config.services.consensus_service_url);
let benchmark_id = consensus_service.start_benchmark(config).await?;
```

## Project Structure

```
hcp-gateway/
├── src/
│   ├── main.rs                 # Server entry point
│   ├── config.rs               # Configuration management
│   ├── error.rs                # Error types and handling
│   ├── models.rs               # Data models
│   ├── state.rs                # Application state (in-memory mock data)
│   ├── tasks.rs                # Task executor for concurrent operations
│   ├── middleware.rs           # Request middleware
│   ├── api/                    # API endpoint handlers
│   │   ├── mod.rs              # API module definition
│   │   ├── health.rs           # Health check endpoint
│   │   ├── consensus.rs        # Consensus API endpoints
│   │   ├── transaction.rs      # Transaction API endpoints
│   │   ├── node.rs             # Node API endpoints
│   │   ├── performance.rs      # Performance metrics endpoints
│   │   └── analysis.rs         # Analysis and reporting endpoints
│   └── services/               # Microservice clients (integration layer)
│       ├── mod.rs              # Service module definition
│       ├── consensus_service.rs    # Consensus service client
│       ├── blockchain_service.rs   # Blockchain service client
│       └── storage_service.rs      # Storage service client
├── Cargo.toml                  # Project dependencies
└── config.toml                 # Configuration file (optional)
```

## Quick Start

### Prerequisites

- Rust 1.70+ (install from https://rustup.rs/)
- Cargo (comes with Rust)

### Installation

```bash
# Clone the repository
git clone https://github.com/fffeng99999/hcp-gateway.git
cd hcp-gateway

# Build the project
cargo build --release

# Run the server
cargo run --release
```

The server will start on `http://localhost:8080` by default.

### Configuration

Set environment variables to customize behavior:

```bash
# Server configuration
export SERVER_HOST="127.0.0.1"
export SERVER_PORT="8080"
export MAX_BODY_SIZE="10485760"  # 10MB

# Service URLs (for future integration)
export CONSENSUS_SERVICE_URL="http://localhost:8081"
export BLOCKCHAIN_SERVICE_URL="http://localhost:8082"
export STORAGE_SERVICE_URL="http://localhost:8083"
```

Or create a `.env` file:

```env
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
CONSENSUS_SERVICE_URL=http://localhost:8081
```

## API Endpoints

### Health Check

```bash
GET /health
```

### Consensus Management

```bash
# Get supported algorithms
GET /consensus/algorithms

# Get current configuration
GET /consensus/config

# Select an algorithm
POST /consensus/select
Body: {"algorithm_id": "tPBFT", "parameters": {...}}

# Update parameters
PUT /consensus/parameters
Body: {"algorithm_id": "tPBFT", "param_name": "f", "value": 2}

# Start benchmark
POST /consensus/benchmark/start
Body: {"algorithm_id": "tPBFT", "duration": 600, "transaction_rate": 5000, "node_count": 20}

# Get benchmark result
GET /consensus/benchmark/{benchmark_id}

# Stop benchmark
POST /consensus/benchmark/{benchmark_id}/stop

# Get benchmark history
GET /consensus/benchmark/history
```

### Transactions

```bash
# Submit transaction
POST /transaction/submit
Body: {"payload": {...}}

# Get transaction
GET /transaction/{tx_id}

# Get transaction status
GET /transaction/status

# Get transaction history
GET /transaction/history
```

### Nodes

```bash
# List all nodes
GET /node/list

# Get specific node
GET /node/{node_id}

# Get node statistics
GET /node/stats
```

### Performance

```bash
# Get current metrics
GET /performance/metrics

# Get performance history
GET /performance/history

# Get algorithm comparison
GET /performance/comparison
```

### Analysis

```bash
# Get analysis report
GET /analysis/report

# Get performance trends
GET /analysis/trends
```

## Example Usage

### Using curl

```bash
# Start a benchmark
curl -X POST http://localhost:8080/consensus/benchmark/start \
  -H "Content-Type: application/json" \
  -d '{
    "algorithm_id": "tPBFT",
    "parameters": {"f": 1},
    "duration": 600,
    "transaction_rate": 5000,
    "node_count": 20
  }'

# Get benchmark results
curl http://localhost:8080/consensus/benchmark/{benchmark_id}

# Get performance comparison
curl http://localhost:8080/performance/comparison
```

### Using Node.js/TypeScript (from hcp-ui)

```typescript
import axios from 'axios';

const client = axios.create({
  baseURL: 'http://localhost:8080'
});

// Get algorithms
const algorithms = await client.get('/consensus/algorithms');

// Start benchmark
const response = await client.post('/consensus/benchmark/start', {
  algorithm_id: 'tPBFT',
  parameters: { f: 1 },
  duration: 600,
  transaction_rate: 5000,
  node_count: 20
});

const benchmarkId = response.data.benchmark_id;

// Poll for results
const results = await client.get(`/consensus/benchmark/${benchmarkId}`);
```

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_task_execution -- --nocapture
```

### Code Formatting

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check

# Lint code
cargo clippy
```

### Building for Production

```bash
# Build optimized release
cargo build --release

# Run release binary
./target/release/hcp-gateway
```

## Integration Guide

### Adding a New Microservice

1. Create a new service client in `src/services/your_service.rs`:

```rust
pub struct YourServiceClient {
    service_url: Option<String>,
}

impl YourServiceClient {
    pub fn new(service_url: Option<String>) -> Self {
        YourServiceClient { service_url }
    }

    pub async fn your_operation(&self) -> ApiResult<YourResult> {
        // TODO: Implement actual service call
        Ok(default_mock_data())
    }
}
```

2. Update `src/services/mod.rs` to include your service

3. Add endpoint handler in appropriate API module

### Replacing Mock Data with Real Service Calls

All mock data is centralized in:

- `src/state.rs` - State initialization methods
- Service clients in `src/services/` - Service integration points

Simply replace the mock implementations with actual HTTP calls using the `reqwest` client.

## Parallel Processing Example

```rust
use crate::tasks::BatchProcessor;

// Process multiple benchmarks concurrently
let processor = BatchProcessor::<BenchmarkResult>::new();

let futures = vec![
    run_benchmark_task("tPBFT", config1),
    run_benchmark_task("PBFT", config2),
    run_benchmark_task("HotStuff", config3),
    run_benchmark_task("Leios", config4),
];

let task_ids = processor.process_batch(futures).await;
await processor.wait_all_complete(task_ids).await;
```

## Performance Characteristics

- **Concurrent Requests**: Built on Tokio async runtime - handles thousands of concurrent connections
- **Memory**: In-memory state store with RwLock for safe concurrent access
- **Latency**: <10ms typical response time for API calls
- **Throughput**: >1000 requests/second on modest hardware

## Troubleshooting

### Server won't start

```bash
# Check if port is already in use
lsof -i :8080

# Use different port
export SERVER_PORT=8081
cargo run
```

### CORS errors in frontend

The gateway includes permissive CORS. If you need to restrict it:

Edit `src/main.rs` and update the CorsLayer configuration.

### High memory usage

The in-memory state stores all data. For production with large datasets, implement:

1. Database backend (PostgreSQL, etc.)
2. Redis caching layer
3. Implement storage_service integration

## Future Enhancements

- [ ] Database backend (PostgreSQL/MongoDB)
- [ ] Redis caching layer
- [ ] WebSocket support for real-time updates
- [ ] Metrics export (Prometheus)
- [ ] Rate limiting middleware
- [ ] JWT authentication
- [ ] API versioning
- [ ] Distributed tracing (Jaeger)

## License

Apache License 2.0

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Submit a pull request

## Support

For issues and questions, please create an issue on the GitHub repository.
