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

## gRPC Client

The gateway communicates with `hcp-server` via gRPC using `tonic`.

### Requirements

- **Protoc**: Version 3.20 or higher is required for code generation.

### Configuration

The client connects to the consensus service using the following environment variable:

- `HCP_CONSENSUS_GRPC_ADDR`: Address of the consensus gRPC service (default: `tcp://127.0.0.1:50051`)

### Code Generation

The client code is automatically generated from `.proto` files in `../hcp-server/api/proto` during build via `build.rs`.

```bash
# Build automatically generates protos
cargo build
```

## Project Structure

```
hcp-gateway/
├── src/
│   ├── bin/
│   │   └── main.rs             # Server entry point
│   ├── lib.rs                  # Library entry point
│   ├── config.rs               # Configuration management
│   ├── error.rs                # Error types and handling
│   ├── models.rs               # Data models
│   ├── state.rs                # Application state
│   ├── grpc_client.rs          # gRPC client for hcp-consensus
│   ├── api/                    # API endpoint handlers
│   │   ├── block.rs            # Block API endpoints
│   │   ├── transaction.rs      # Transaction API endpoints
│   │   └── ...
├── Cargo.toml                  # Project dependencies
└── config.toml                 # Configuration file
```

## Quick Start

### Prerequisites

- Rust 1.70+ (install from https://rustup.rs/)
- Cargo (comes with Rust)
- Protoc >= 3.20

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
