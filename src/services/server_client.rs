pub mod hcp {
    pub mod common {
        pub mod v1 {
            tonic::include_proto!("hcp.common.v1");
        }
    }
    pub mod transaction {
        pub mod v1 {
            tonic::include_proto!("hcp.transaction.v1");
        }
    }
    pub mod auth {
        pub mod v1 {
            tonic::include_proto!("hcp.auth.v1");
        }
    }
    pub mod block {
        pub mod v1 {
            tonic::include_proto!("hcp.block.v1");
        }
    }
    pub mod benchmark {
        pub mod v1 {
            tonic::include_proto!("hcp.benchmark.v1");
        }
    }
    pub mod metric {
        pub mod v1 {
            tonic::include_proto!("hcp.metric.v1");
        }
    }
    pub mod node {
        pub mod v1 {
            tonic::include_proto!("hcp.node.v1");
        }
    }
}

pub use hcp::benchmark::v1 as benchmark;
pub use hcp::block::v1 as block;
pub use hcp::auth::v1 as auth;
pub use hcp::metric::v1 as metric;
pub use hcp::node::v1 as node;
pub use hcp::transaction::v1 as transaction;

use auth::auth_service_client::AuthServiceClient;
use benchmark::benchmark_service_client::BenchmarkServiceClient;
use block::block_service_client::BlockServiceClient;
use metric::metric_service_client::MetricServiceClient;
use node::node_service_client::NodeServiceClient;
use transaction::transaction_service_client::TransactionServiceClient;

use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;
use tonic::transport::Channel;
use tonic::transport::Endpoint;

#[derive(Clone)]
pub struct ServerClient {
    pub tx_client: TransactionServiceClient<Channel>,
    pub block_client: BlockServiceClient<Channel>,
    pub benchmark_client: BenchmarkServiceClient<Channel>,
    pub metric_client: MetricServiceClient<Channel>,
    pub node_client: NodeServiceClient<Channel>,
    pub auth_client: AuthServiceClient<Channel>,
    pub healthy: Arc<AtomicBool>,
}

impl ServerClient {
    pub async fn connect(
        endpoint: String,
        healthy: Arc<AtomicBool>,
    ) -> Result<Self, tonic::transport::Error> {
        let channel = Endpoint::from_shared(endpoint)?
            .connect_timeout(Duration::from_secs(5))
            .connect()
            .await?;

        let tx_client = TransactionServiceClient::new(channel.clone())
            .max_decoding_message_size(16 * 1024 * 1024);
        let block_client =
            BlockServiceClient::new(channel.clone()).max_decoding_message_size(16 * 1024 * 1024);
        let benchmark_client = BenchmarkServiceClient::new(channel.clone())
            .max_decoding_message_size(16 * 1024 * 1024);
        let metric_client =
            MetricServiceClient::new(channel.clone()).max_decoding_message_size(16 * 1024 * 1024);
        let auth_client =
            AuthServiceClient::new(channel.clone()).max_decoding_message_size(16 * 1024 * 1024);
        let node_client =
            NodeServiceClient::new(channel).max_decoding_message_size(16 * 1024 * 1024);

        Ok(Self {
            tx_client,
            block_client,
            benchmark_client,
            metric_client,
            node_client,
            auth_client,
            healthy,
        })
    }
}
