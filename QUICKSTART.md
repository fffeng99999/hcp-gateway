# HCP Gateway - Quick Start Guide

## 5-Minute Setup

### 1. Prerequisites

```bash
# Check Rust is installed
rustc --version  # Should be 1.70 or higher
cargo --version

# If not installed, run:
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Clone and Build

```bash
# Clone repository
git clone https://github.com/fffeng99999/hcp-gateway.git
cd hcp-gateway

# Build the project
cargo build --release

# This may take 2-3 minutes on first run
```

### 3. Run the Server

```bash
# Start the gateway
cargo run --release

# Or run the compiled binary directly
./target/release/hcp-gateway
```

You should see:
```
Starting HCP Gateway...
Configuration loaded: Config { ... }
Server listening on 127.0.0.1:8080
```

### 4. Test the API

```bash
# Health check
curl http://localhost:8080/health

# Get algorithms
curl http://localhost:8080/consensus/algorithms

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
```

## Development Workflow

### Watch Mode (Hot Reload)

```bash
# Install cargo-watch
cargo install cargo-watch

# Recompile on file changes
cargo watch -x run
```

### Code Formatting

```bash
# Format your code
cargo fmt

# Check if code is formatted
cargo fmt -- --check
```

### Linting

```bash
# Run clippy linter
cargo clippy

# Run clippy with strict warnings
cargo clippy -- -D warnings
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_batch_processing -- --nocapture

# Run tests and show coverage
cargo tarpaulin
```

## Common Tasks

### Add a New API Endpoint

1. **Add handler in appropriate API module** (`src/api/*.rs`):

```rust
pub async fn my_endpoint(
    State(state): State<AppState>,
) -> ApiResult<Json<MyResponse>> {
    tracing::info!("My endpoint called");
    // Implementation
    Ok(Json(response))
}
```

2. **Register route in `src/main.rs`**:

```rust
.route("/my/endpoint", get(api::module::my_endpoint))
```

3. **Test it**:

```bash
curl http://localhost:8080/my/endpoint
```

### Add a New Data Model

1. **Define in `src/models.rs`**:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyModel {
    pub id: String,
    pub name: String,
}
```

2. **Use in handlers**:

```rust
pub async fn my_handler() -> ApiResult<Json<MyModel>> {
    Ok(Json(MyModel {
        id: Uuid::new_v4().to_string(),
        name: "example".to_string(),
    }))
}
```

### Integrate a Real Microservice

1. **Update service client in `src/services/your_service.rs`**:

```rust
use reqwest::Client;

pub async fn your_operation(&self) -> ApiResult<YourResult> {
    if let Some(url) = &self.service_url {
        let client = Client::new();
        let response = client
            .get(format!("{}your/endpoint", url))
            .send()
            .await
            .map_err(|e| ApiError::ServiceUnavailable(e.to_string()))?
            .json::<YourResult>()
            .await
            .map_err(|e| ApiError::InternalError(e.to_string()))?;
        
        Ok(response)
    } else {
        // Fallback to mock
        Ok(your_mock_data())
    }
}
```

2. **Update `AppState` if needed** in `src/state.rs`

3. **Test against real service**:

```bash
export CONSENSUS_SERVICE_URL=http://localhost:8081
cargo run --release
```

### Debug Issues

**Enable debug logging:**

```bash
RUST_LOG=debug cargo run
```

**For even more verbose output:**

```bash
RUST_LOG=debug,hcp_gateway=trace cargo run
```

### Performance Profiling

```bash
# Build with profiling symbols
RUSTFLAGS="-C debug-assertions" cargo build --release

# Run with perf (Linux)
perf record -g ./target/release/hcp-gateway
perf report
```

## Connecting Frontend (hcp-ui)

### 1. Ensure Gateway is Running

```bash
# Terminal 1: Start gateway
cd hcp-gateway
cargo run --release

# Terminal 2: Start frontend
cd ../hcp-ui
npm run dev
```

### 2. Verify CORS is Working

Frontend should connect to `http://localhost:8080` by default (see `.env.development`).

### 3. Check Network Requests

Open browser DevTools (F12 → Network tab) to see:
- API calls going to `http://localhost:8080`
- Response status codes
- Response data

### 4. Example Frontend API Call

```typescript
// From hcp-ui
import http from '@/api/http';

const benchmarks = await http.get('/consensus/benchmark/history');
console.log(benchmarks);
```

## Troubleshooting

### Port Already in Use

```bash
# Find process using port 8080
lsof -i :8080

# Kill the process
kill -9 <PID>

# Or use a different port
export SERVER_PORT=8081
cargo run
```

### Compilation Errors

```bash
# Update dependencies
cargo update

# Clean build
cargo clean
cargo build

# Check for issues
cargo check
```

### CORS Errors

If frontend can't reach backend:

```bash
# Check CORS is enabled in main.rs
# Should have: .layer(CorsLayer::permissive())

# For production, restrict CORS:
.layer(
    CorsLayer::new()
        .allow_origin("http://localhost:5173".parse()?)
        .allow_methods(vec![Method::GET, Method::POST, Method::PUT])
)
```

### API Returns Empty Data

- Check mock data initialization in `src/state.rs`
- Verify state is being populated in `AppState::new()`
- Add logging with `tracing::info!()`

```bash
RUST_LOG=debug cargo run
```

## Performance Tips

### Optimize Build Time

```bash
# Use mold linker (Linux) - much faster
cargo install mold
RUSTFLAGS="-C link-arg=-fuse-ld=mold" cargo build

# Incremental compilation
RUST_INCREMENTAL=1 cargo build
```

### Profile Production Build

```bash
# See build time breakdown
RUSTFLAGS="-Z timings" cargo build --release 2>&1 | tee build.log
```

### Monitor Memory Usage

```bash
# Watch memory in real-time
watch -n 1 'ps aux | grep hcp-gateway'
```

## Next Steps

1. ✅ **Server running**: Gateway is ready for API calls
2. 🔄 **Connect frontend**: Start hcp-ui and test integration
3. 🔌 **Add real services**: Replace mock implementations with actual Cosmos SDK endpoints
4. 📊 **Add monitoring**: Implement Prometheus metrics
5. 🚀 **Deploy**: containerize and deploy to production

## Useful Commands Reference

```bash
# Development
cargo run              # Run in debug mode
cargo run --release   # Run optimized
cargo watch -x run    # Auto-reload on changes

# Testing
cargo test            # Run all tests
cargo test -- --test-threads=1  # Single threaded

# Formatting
cargo fmt             # Format code
cargo fmt -- --check  # Check formatting

# Linting
cargo clippy          # Run linter
cargo clippy --all-targets  # Check tests and examples too

# Dependencies
cargo tree            # Show dependency tree
cargo outdated        # Check for updates
cargo audit           # Check for vulnerabilities

# Documentation
cargo doc --open      # Build and open API docs locally
```

## Getting Help

- **Rust Documentation**: https://doc.rust-lang.org/
- **Axum Web Framework**: https://github.com/tokio-rs/axum
- **Tokio Async Runtime**: https://tokio.rs/
- **Project Issues**: Check GitHub issues

## Success! 🎉

Your gateway is now running and ready to serve the frontend. Happy coding!
