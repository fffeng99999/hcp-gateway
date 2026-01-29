# HCP Gateway - Service Integration Guide

This guide explains how to integrate real microservices with the HCP Gateway.

## Current State: Mock Data Mode

The gateway currently operates in **mock mode**, where:
- All data is stored in-memory
- Service URLs are optional (`Option<String>`)
- API responses return pre-populated mock data
- Perfect for frontend development and testing

## Integration Steps

### Step 1: Start with Real Service URLs

#### Option A: Environment Variables

```bash
export CONSENSUS_SERVICE_URL="http://your-consensus-service:8081"
export BLOCKCHAIN_SERVICE_URL="http://your-blockchain-service:8082"
export STORAGE_SERVICE_URL="http://your-storage-service:8083"

cargo run --release
```

#### Option B: .env File

Create or update `.env`:

```env
CONSENSUS_SERVICE_URL=http://localhost:8081
BLOCKCHAIN_SERVICE_URL=http://localhost:8082
STORAGE_SERVICE_URL=http://localhost:8083
```

Then run:

```bash
cargo run --release
```

### Step 2: Update Service Client

#### Consensus Service Integration Example

**File: `src/services/consensus_service.rs`**

**Current (Mock) Implementation:**

```rust
pub async fn start_benchmark(&self, config: BenchmarkConfig) -> ApiResult<String> {
    // TODO: Call actual consensus service when available
    // For now, return mock data
    Ok(uuid::Uuid::new_v4().to_string())
}
```

**New (Real Service) Implementation:**

```rust
use reqwest::Client;

pub async fn start_benchmark(&self, config: BenchmarkConfig) -> ApiResult<String> {
    match &self.service_url {
        Some(url) => {
            let client = Client::new();
            
            let response = client
                .post(format!("{}benchmark/start", url))
                .json(&config)
                .send()
                .await
                .map_err(|e| ApiError::ServiceUnavailable(format!(
                    "Failed to reach consensus service: {}", e
                )))?
                .error_for_status()
                .map_err(|e| ApiError::InternalError(format!(
                    "Consensus service error: {}", e
                )))?;

            #[derive(serde::Deserialize)]
            struct BenchmarkResponse {
                benchmark_id: String,
            }

            let result: BenchmarkResponse = response
                .json()
                .await
                .map_err(|e| ApiError::InternalError(
                    format!("Failed to parse response: {}", e)
                ))?;

            Ok(result.benchmark_id)
        }
        None => {
            // Fallback to mock data
            tracing::info!("No consensus service URL configured, using mock data");
            Ok(uuid::Uuid::new_v4().to_string())
        }
    }
}
```

### Step 3: Add Retry Logic

**File: `src/services/consensus_service.rs`**

Add retry handler for resilience:

```rust
use std::time::Duration;

async fn call_with_retry<F, T>(
    mut f: F,
    max_retries: u32,
) -> ApiResult<T>
where
    F: FnMut() -> futures::future::BoxFuture<'static, ApiResult<T>>,
{
    let mut last_error = None;
    
    for attempt in 0..=max_retries {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);
                if attempt < max_retries {
                    let backoff = Duration::from_millis(100 * (2 ^ attempt));
                    tokio::time::sleep(backoff).await;
                }
            }
        }
    }
    
    Err(last_error.unwrap_or_else(|| 
        ApiError::InternalError("Unknown error".to_string())
    ))
}
```

### Step 4: Add Connection Pooling

Update `Cargo.toml` to include connection pooling:

```toml
[dependencies]
reqwest = { version = "0.12", features = ["json"] }
```

Update service client to use connection pool:

```rust
use reqwest::Client;
use std::sync::Arc;

pub struct ConsensusServiceClient {
    service_url: Option<String>,
    client: Arc<Client>,  // Add this
}

impl ConsensusServiceClient {
    pub fn new(service_url: Option<String>) -> Self {
        ConsensusServiceClient {
            service_url,
            client: Arc::new(Client::new()),  // Initialize here
        }
    }
    
    pub async fn start_benchmark(&self, config: BenchmarkConfig) -> ApiResult<String> {
        match &self.service_url {
            Some(url) => {
                let response = self.client  // Use pooled client
                    .post(format!("{}benchmark/start", url))
                    .json(&config)
                    .send()
                    .await
                    .map_err(|e| ApiError::ServiceUnavailable(e.to_string()))?
                    .json::<BenchmarkResponse>()
                    .await
                    .map_err(|e| ApiError::InternalError(e.to_string()))?;
                
                Ok(response.benchmark_id)
            }
            None => Ok(uuid::Uuid::new_v4().to_string()),
        }
    }
}
```

### Step 5: Implement Circuit Breaker Pattern (Optional)

For production reliability, add circuit breaker:

```rust
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct CircuitBreaker {
    failure_count: Arc<AtomicU32>,
    last_failure_time: Arc<std::sync::Mutex<u64>>,
    threshold: u32,
    timeout_secs: u64,
}

impl CircuitBreaker {
    pub fn new(threshold: u32, timeout_secs: u64) -> Self {
        CircuitBreaker {
            failure_count: Arc::new(AtomicU32::new(0)),
            last_failure_time: Arc::new(std::sync::Mutex::new(0)),
            threshold,
            timeout_secs,
        }
    }
    
    pub fn is_open(&self) -> bool {
        let count = self.failure_count.load(Ordering::Relaxed);
        if count < self.threshold {
            return false;
        }
        
        let last_time = *self.last_failure_time.lock().unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        now - last_time < self.timeout_secs
    }
    
    pub fn record_failure(&self) {
        self.failure_count.fetch_add(1, Ordering::Relaxed);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        *self.last_failure_time.lock().unwrap() = now;
    }
    
    pub fn record_success(&self) {
        self.failure_count.store(0, Ordering::Relaxed);
    }
}
```

### Step 6: Update AppState for Service Injection

**File: `src/state.rs`**

Inject service clients into app state:

```rust
use crate::services::{ConsensusServiceClient, BlockchainServiceClient, StorageServiceClient};

pub struct AppState {
    pub config: Config,
    pub consensus_client: ConsensusServiceClient,
    pub blockchain_client: BlockchainServiceClient,
    pub storage_client: StorageServiceClient,
    // ... existing state
}

impl AppState {
    pub fn new(config: Config) -> Self {
        AppState {
            consensus_client: ConsensusServiceClient::new(
                config.services.consensus_service_url.clone()
            ),
            blockchain_client: BlockchainServiceClient::new(
                config.services.blockchain_service_url.clone()
            ),
            storage_client: StorageServiceClient::new(
                config.services.storage_service_url.clone()
            ),
            config,
            // ... initialize other state
        }
    }
}
```

### Step 7: Update API Handlers

**File: `src/api/consensus.rs`**

Use service clients in handlers:

```rust
pub async fn get_algorithms(
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<ConsensusAlgorithm>>> {
    tracing::info!("Getting consensus algorithms from service");
    
    match state.consensus_client.get_algorithms().await {
        Ok(algorithms) => Ok(Json(algorithms)),
        Err(e) => {
            tracing::error!("Failed to get algorithms: {:?}", e);
            Err(e)
        }
    }
}

pub async fn start_benchmark(
    State(state): State<AppState>,
    Json(payload): Json<BenchmarkConfig>,
) -> ApiResult<Json<serde_json::Value>> {
    tracing::info!("Starting benchmark with service");
    
    let benchmark_id = state.consensus_client
        .start_benchmark(payload)
        .await?;
    
    Ok(Json(serde_json::json!({
        "benchmark_id": benchmark_id,
        "status": "started",
    })))
}
```

## Service Expectations

### Consensus Service API

Your consensus service should implement these endpoints:

```bash
# Get algorithms
GET /algorithms
Response: [
  {"id": "tPBFT", "name": "...", ...},
  ...
]

# Start benchmark
POST /benchmark/start
Body: {"algorithm_id": "tPBFT", "parameters": {...}, ...}
Response: {"benchmark_id": "550e8400-..."}

# Get results
GET /benchmark/{id}
Response: {"benchmark_id": "...", "metrics": {...}, ...}

# Get status
GET /benchmark/{id}/status
Response: {"status": "running|completed"}
```

### Blockchain Service API

```bash
# Submit transaction
POST /transaction/submit
Body: {"payload": {...}}
Response: {"tx_id": "...", "status": "submitted"}

# Query transaction
GET /transaction/{id}
Response: {"id": "...", "status": "...", ...}
```

### Storage Service API

```bash
# Save data
POST /store
Body: {"key": "...", "value": {...}}
Response: {"status": "success"}

# Get data
GET /store/{key}
Response: {"key": "...", "value": {...}}

# Delete data
DELETE /store/{key}
Response: {"status": "success"}
```

## Rollback to Mock Mode

If service integration fails:

```bash
# Remove service URLs
unset CONSENSUS_SERVICE_URL
unset BLOCKCHAIN_SERVICE_URL
unset STORAGE_SERVICE_URL

# Restart gateway - will use mock data
cargo run --release
```

## Testing Service Integration

```bash
# 1. Start all services
./start_services.sh  # Your service startup script

# 2. Set environment variables
export CONSENSUS_SERVICE_URL="http://localhost:8081"
export BLOCKCHAIN_SERVICE_URL="http://localhost:8082"

# 3. Start gateway
cargo run --release

# 4. Test API call
curl http://localhost:8080/consensus/algorithms

# 5. Check logs for service calls
RUST_LOG=debug cargo run --release
```

## Monitoring Service Integration

Add metrics collection:

```rust
use std::time::Instant;

pub async fn start_benchmark_monitored(
    service: &ConsensusServiceClient,
    config: BenchmarkConfig,
) -> ApiResult<String> {
    let start = Instant::now();
    
    let result = service.start_benchmark(config).await;
    
    let duration = start.elapsed();
    tracing::info!(
        "Consensus service call took {:.2}ms",
        duration.as_secs_f64() * 1000.0
    );
    
    result
}
```

## Troubleshooting

### Service Connection Refused

```bash
# Check service is running
curl http://your-service:port/health

# Check firewall
sudo ufw allow <port>

# Check service logs
journalctl -u your-service -f
```

### Timeout Errors

Increase timeout in reqwest:

```rust
let client = Client::builder()
    .timeout(Duration::from_secs(30))
    .build()?;
```

### JSON Parse Errors

Ensure response format matches:

```rust
#[derive(serde::Deserialize, Debug)]
struct ServiceResponse {
    benchmark_id: String,
    // Match actual service response fields
}
```

## Production Checklist

- ✅ Service URLs configured correctly
- ✅ Retry logic implemented
- ✅ Circuit breaker enabled
- ✅ Connection pooling configured
- ✅ Error handling comprehensive
- ✅ Logging enabled
- ✅ Metrics collection implemented
- ✅ Load testing completed
- ✅ Failover strategy tested
- ✅ Documentation updated

## Next Steps

1. Implement consensus service integration
2. Add blockchain service integration
3. Set up distributed storage
4. Deploy to staging environment
5. Load test with real services
6. Deploy to production
