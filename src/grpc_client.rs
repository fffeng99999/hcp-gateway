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
    pub mod block {
        pub mod v1 {
            tonic::include_proto!("hcp.block.v1");
        }
    }
}

pub use hcp::transaction::v1 as transaction;
pub use hcp::block::v1 as block;

use transaction::transaction_service_client::TransactionServiceClient;
use block::block_service_client::BlockServiceClient;
use tonic::transport::Channel;
use tonic::transport::Endpoint;
use std::time::Duration;
use tokio::time::sleep;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone)]
pub struct ConsensusClient {
    tx_client: TransactionServiceClient<Channel>,
    block_client: BlockServiceClient<Channel>,
    healthy: Arc<AtomicBool>,
}

pub type SubmitReply = transaction::CreateTransactionResponse;
pub type BlockReply = block::GetBlockResponse;

impl ConsensusClient {
    pub async fn connect(endpoint: String, healthy: Arc<AtomicBool>) -> Result<Self, tonic::transport::Error> {
        let channel = Endpoint::from_shared(endpoint)?
            .connect_timeout(Duration::from_secs(5))
            .connect()
            .await?;
        
        let tx_client = TransactionServiceClient::new(channel.clone())
            .max_decoding_message_size(16 * 1024 * 1024);
        let block_client = BlockServiceClient::new(channel)
            .max_decoding_message_size(16 * 1024 * 1024);
            
        Ok(Self { tx_client, block_client, healthy })
    }

    pub async fn submit_transaction(&mut self, req: transaction::CreateTransactionRequest) -> Result<SubmitReply, tonic::Status> {
        let mut retries = 0;
        let max_retries = 3;
        let mut backoff = Duration::from_millis(500);

        loop {
            let request = tonic::Request::new(req.clone());
            match self.tx_client.create_transaction(request).await {
                Ok(resp) => {
                    self.healthy.store(true, Ordering::SeqCst);
                    return Ok(resp.into_inner());
                }
                Err(status) => {
                    if (status.code() == tonic::Code::Unavailable || status.code() == tonic::Code::Unknown) && retries < max_retries {
                        retries += 1;
                        sleep(backoff).await;
                        backoff *= 2;
                        continue;
                    }
                    if retries >= max_retries {
                        self.healthy.store(false, Ordering::SeqCst);
                    }
                    return Err(status);
                }
            }
        }
    }

    pub async fn get_block(&mut self, height: i64) -> Result<BlockReply, tonic::Status> {
        let mut retries = 0;
        let max_retries = 3;
        let mut backoff = Duration::from_millis(500);

        loop {
            let request = tonic::Request::new(block::GetBlockRequest { height });
            match self.block_client.get_block(request).await {
                Ok(resp) => {
                    self.healthy.store(true, Ordering::SeqCst);
                    return Ok(resp.into_inner());
                }
                Err(status) => {
                    if (status.code() == tonic::Code::Unavailable || status.code() == tonic::Code::Unknown) && retries < max_retries {
                        retries += 1;
                        sleep(backoff).await;
                        backoff *= 2;
                        continue;
                    }
                    if retries >= max_retries {
                        self.healthy.store(false, Ordering::SeqCst);
                    }
                    return Err(status);
                }
            }
        }
    }
}
