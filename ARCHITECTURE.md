# HCP Gateway Architecture

## System Overview

```
┌──────────────────────────────────────────────────────────────┐
│                      hcp-ui (Frontend)                        │
│                    Vue 3 + TypeScript                         │
└────────────────────────┬─────────────────────────────────────┘
                         │ HTTP/REST API
                         │
┌────────────────────────▼─────────────────────────────────────┐
│                   hcp-gateway (API Gateway)                   │
│                  Rust + Axum Framework                        │
│                                                               │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │         API Handlers (req/res layer)                   │ │
│  │  • /consensus/* - Consensus Management API             │ │
│  │  • /transaction/* - Transaction API                    │ │
│  │  • /node/* - Node Management API                       │ │
│  │  • /performance/* - Performance Metrics API            │ │
│  │  • /analysis/* - Analysis & Reporting API              │ │
│  └─────────────────────────────────────────────────────────┘ │
│                         ▲                                     │
│                         │                                     │
│  ┌─────────────────────┴────────────────────────────────────┐ │
│  │    Service Integration Layer                             │ │
│  │                                                           │ │
│  │  ┌─────────────────────────────────────────────────────┐ │ │
│  │  │ Microservice Clients (placeholder implementations) │ │ │
│  │  │  • ConsensusServiceClient                          │ │ │
│  │  │  • BlockchainServiceClient                         │ │ │
│  │  │  • StorageServiceClient                            │ │ │
│  │  └─────────────────────────────────────────────────────┘ │ │
│  └────┬──────────────────┬──────────────────┬────────────────┘ │
│       │                  │                  │                 │
└───────┼──────────────────┼──────────────────┼─────────────────┘
        │                  │                  │
        ▼                  ▼                  ▼
    ┌────────┐        ┌────────┐        ┌────────┐
    │Consensus│       │Blockchain│      │Storage │
    │Service  │       │Service   │      │Service │
    └────────┘       └────────┘       └────────┘

```

## Module Breakdown

### 1. **Core Framework** (`main.rs`)

- Initializes the Axum web server
- Configures routes for all API endpoints
- Sets up middleware (CORS, tracing, error handling)
- Manages application lifecycle

### 2. **State Management** (`state.rs`)

**Responsibility**: Central state store with mock data initialization

```rust
pub struct AppState {
    pub consensus_config: Arc<RwLock<ConsensusConfig>>,
    pub benchmarks: Arc<RwLock<HashMap<String, BenchmarkResult>>>,
    pub transactions: Arc<RwLock<HashMap<String, Transaction>>>,
    pub nodes: Arc<RwLock<HashMap<String, Node>>>,
}
```

**Thread Safety**: Uses `Arc` (atomic reference counting) and `RwLock` (reader-writer lock) for safe concurrent access.

**Mock Data Initialization**:
- Pre-populates with 5 benchmarks (one per algorithm)
- Creates 10 sample transactions
- Initializes 5 mock blockchain nodes

### 3. **Task Processing** (`tasks.rs`)

**Purpose**: Handle concurrent operations for parallel benchmark execution

```rust
pub struct TaskExecutor<T> {
    tasks: Arc<RwLock<HashMap<String, Task<T>>>>,
    active_handles: Arc<RwLock<HashMap<String, JoinHandle<()>>>>,
}
```

**Key Features**:
- `submit()` - Spawn async tasks
- `wait_all()` - Wait for multiple tasks in parallel
- `cancel()` - Abort running tasks
- `BatchProcessor` - Process multiple independent futures concurrently

**Use Case**: Running multiple consensus benchmarks simultaneously without blocking

### 4. **API Layer** (`api/`)

Each module handles a specific domain:

#### `consensus.rs`
- Get/select algorithms
- Manage consensus configuration
- Start/stop benchmarks
- Query benchmark results
- Retrieve benchmark history

#### `transaction.rs`
- Submit transactions
- Query transaction status
- Retrieve transaction history

#### `node.rs`
- List all nodes
- Get individual node details
- Query cluster statistics

#### `performance.rs`
- Get current metrics
- Retrieve performance history
- Compare algorithm performance

#### `analysis.rs`
- Generate performance reports
- Analyze trends
- Provide recommendations

### 5. **Service Layer** (`services/`)

**Architecture Pattern**: Service Client Pattern

Each service is a wrapper around external service endpoints:

```rust
pub struct ConsensusServiceClient {
    service_url: Option<String>,  // NULL in mock mode, URL in production
}

impl ConsensusServiceClient {
    pub async fn get_algorithms(&self) -> ApiResult<Vec<ConsensusAlgorithm>> {
        match &self.service_url {
            Some(url) => {
                // Make HTTP call to real service
                // client.get(format!("{}"algorithms", url)).await
            },
            None => {
                // Return mock data (current behavior)
                Ok(vec![...])
            }
        }
    }
}
```

**Transition Path for Integration**:

1. **Current State**: Returns mock data
2. **Integration Step 1**: Add HTTP calls using `reqwest` client
3. **Integration Step 2**: Add connection pooling and retry logic
4. **Integration Step 3**: Add circuit breaker pattern for fault tolerance

### 6. **Error Handling** (`error.rs`)

```rust
pub enum ApiError {
    NotFound(String),
    InvalidInput(String),
    InternalError(String),
    ServiceUnavailable(String),
    Conflict(String),
    Unauthorized,
}
```

Automatic conversion to HTTP responses:
- `NotFound` → 404
- `InvalidInput` → 400
- `InternalError` → 500
- `ServiceUnavailable` → 503
- `Conflict` → 409
- `Unauthorized` → 401

### 7. **Models** (`models.rs`)

Data structures for:
- API requests/responses
- Domain entities (Consensus, Transaction, Node, etc.)
- Performance metrics
- Benchmark results

### 8. **Configuration** (`config.rs`)

Loads from multiple sources with fallback:
1. `config.toml` file
2. Environment variables
3. Default values

## Data Flow Examples

### Scenario 1: Start Benchmark

```
Frontend (hcp-ui)
    │
    ├─ POST /consensus/benchmark/start
    │  {
    │    "algorithm_id": "tPBFT",
    │    "duration": 600,
    │    "transaction_rate": 5000
    │  }
    │
    ▼
API Handler (consensus.rs::start_benchmark)
    │
    ├─ Create new BenchmarkResult
    ├─ Store in AppState.benchmarks (RwLock)
    │
    ▼ (Future: Integration)
ConsensusServiceClient
    │
    ├─ Make HTTP POST to consensus service
    │  POST http://localhost:8081/benchmark/start
    │
    ▼
Consensus Service
    │
    ├─ Run actual benchmark
    ├─ Update results periodically
    │
    └─ Poll endpoint for results

Response to Frontend:
    {
      "benchmark_id": "550e8400-e29b-41d4-a716-446655440000",
      "status": "started"
    }
```

### Scenario 2: Concurrent Benchmark Processing

```
Frontend
    │
    ├─ POST /consensus/benchmark/start (algorithm: tPBFT)
    │
    ├─ POST /consensus/benchmark/start (algorithm: PBFT)
    │
    └─ POST /consensus/benchmark/start (algorithm: HotStuff)

Gateway Task Executor
    │
    ├─ Submit task 1 ──┐
    │                  │
    ├─ Submit task 2 ──┼─── All running concurrently
    │                  │     (Tokio async runtime)
    └─ Submit task 3 ──┘

Wait for all to complete
    │
    ▼
Return combined results
```

## Concurrency Model

### Thread Safety

- **Shared State**: Protected by `Arc<RwLock<T>>`
  - `Arc`: Atomic reference counting for safe shared ownership
  - `RwLock`: Allows multiple readers OR one writer
- **Async Runtime**: Tokio runtime with work-stealing thread pool
- **Task Spawning**: `tokio::spawn` for concurrent task execution

### Performance Characteristics

- **Request Handling**: O(1) for most API operations
- **State Access**: RwLock overhead is minimal (~nanoseconds)
- **Concurrent Benchmarks**: Limited only by system resources
- **Memory**: O(n) where n = number of active benchmarks/transactions

## Future Integration Points

### 1. Consensus Service Integration

**File to modify**: `src/services/consensus_service.rs`

```rust
pub async fn start_benchmark(&self, config: BenchmarkConfig) -> ApiResult<String> {
    if let Some(url) = &self.service_url {
        let response = reqwest::Client::new()
            .post(format!("{}benchmark/start", url))
            .json(&config)
            .send()
            .await?;
        
        let result: BenchmarkResponse = response.json().await?;
        Ok(result.benchmark_id)
    } else {
        // Mock implementation
        Ok(uuid::Uuid::new_v4().to_string())
    }
}
```

### 2. Blockchain Service Integration

**File to modify**: `src/services/blockchain_service.rs`

Integrate with Cosmos SDK endpoints for transaction submission.

### 3. Storage Service Integration

**File to modify**: `src/services/storage_service.rs`

Replace in-memory HashMap with persistent database (PostgreSQL, MongoDB).

### 4. WebSocket Support (Future)

Add real-time updates for benchmark progress:

```rust
WS /ws/benchmark/{id}
    ↓
Browser receives real-time metrics updates
    ↓
UI automatically reflects live progress
```

## Deployment Considerations

### Development

```bash
cargo run
```

### Production

```bash
cargo build --release
./target/release/hcp-gateway
```

### Docker (Future)

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/hcp-gateway /usr/local/bin/
CMD ["hcp-gateway"]
```

### Horizontal Scaling (Future)

- Use shared Redis for distributed state
- Message queue (RabbitMQ/Kafka) for benchmark tasks
- Load balancer (Nginx) for multiple gateway instances

## Testing Strategy

### Unit Tests
- Task executor functionality
- Service client behavior
- Error handling

### Integration Tests
- API endpoint responses
- State management
- Concurrent operations

### Performance Tests
- Benchmark execution load
- Memory usage under concurrent load
- Response time percentiles

## Security Considerations

### Current (Development)
- Permissive CORS
- No authentication
- No rate limiting

### Production (TODO)
- Restrict CORS to frontend domain
- Implement JWT authentication
- Add rate limiting middleware
- Enable HTTPS/TLS
- Add request validation
- Implement secrets management
